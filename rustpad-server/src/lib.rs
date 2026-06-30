//! Server backend for the Rustpad collaborative text editor.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use base64::Engine;
use dashmap::DashMap;
use log::{error, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::time;
use warp::{filters::BoxedFilter, http::StatusCode, ws::Ws, Filter, Rejection, Reply};

use crate::{
    database::Database,
    rustpad::{max_document_bytes, Rustpad},
};

pub mod database;
mod ot;
mod rustpad;

/// An entry stored in the global server map.
///
/// Each entry corresponds to a single document. This is garbage collected by a
/// background task once no client has it open for a short idle period, so that
/// neither server memory nor the database grows without bound. Idle activity is
/// tracked on the `Rustpad` itself (see `Rustpad::idle_for`).
struct Document {
    rustpad: Arc<Rustpad>,
}

impl Document {
    fn new(rustpad: Arc<Rustpad>) -> Self {
        Self { rustpad }
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        self.rustpad.kill();
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct CustomReject(anyhow::Error);

impl warp::reject::Reject for CustomReject {}

/// The shared state of the server, accessible from within request handlers.
#[derive(Clone)]
struct ServerState {
    /// Concurrent map storing in-memory documents.
    documents: Arc<DashMap<String, Document>>,
    /// Connection to the database pool, if persistence is enabled.
    database: Option<Database>,
}

/// Statistics about the server, returned from an API endpoint.
#[derive(Serialize)]
struct Stats {
    /// System time when the server started, in seconds since Unix epoch.
    start_time: u64,
    /// Number of documents currently tracked by the server.
    num_documents: usize,
    /// Number of documents persisted in the database.
    database_size: usize,
}

/// Server configuration.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Number of days to clean up documents after inactivity.
    pub expiry_days: u32,
    /// Database object, for persistence if desired.
    pub database: Option<Database>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            expiry_days: 1,
            database: None,
        }
    }
}

/// A combined filter handling all server routes.
pub fn server(config: ServerConfig) -> BoxedFilter<(impl Reply,)> {
    warp::path("api")
        .and(backend(config))
        .or(frontend())
        .boxed()
}

/// Construct routes for static files from React.
fn frontend() -> BoxedFilter<(impl Reply,)> {
    warp::fs::dir("dist").boxed()
}

/// Construct backend routes, including WebSocket handlers.
fn backend(config: ServerConfig) -> BoxedFilter<(impl Reply,)> {
    let state = ServerState {
        documents: Default::default(),
        database: config.database,
    };
    tokio::spawn(cleaner(state.clone(), idle_clear_duration()));
    tokio::spawn(session_cleaner(state.clone()));

    let state_filter = warp::any().map(move || state.clone());

    let socket = warp::path!("socket" / String)
        .and(warp::ws())
        .and(state_filter.clone())
        .and_then(socket_handler);

    // HTTP long-polling fallback for clients behind proxies that block the
    // WebSocket upgrade. These are ordinary HTTPS requests carrying the same
    // protocol messages as the WebSocket transport.
    let connect = warp::path!("connect" / String)
        .and(warp::post())
        .and(state_filter.clone())
        .and_then(connect_handler);

    let poll = warp::path!("poll" / String)
        .and(warp::query::<PollQuery>())
        .and(state_filter.clone())
        .and_then(poll_handler);

    // Allow a body large enough to carry a full-document paste (JSON-escaped,
    // so roughly double the raw text), tied to the configured document cap.
    let body_limit = (max_document_bytes() as u64).saturating_mul(2);
    let send = warp::path!("send" / String)
        .and(warp::query::<PollQuery>())
        .and(warp::post())
        .and(warp::body::content_length_limit(body_limit))
        .and(warp::body::bytes())
        .and(state_filter.clone())
        .and_then(send_handler);

    let text = warp::path!("text" / String)
        .and(state_filter.clone())
        .and_then(text_handler);

    let start_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime returned before UNIX_EPOCH")
        .as_secs();
    let stats = warp::path!("stats")
        .and(warp::any().map(move || start_time))
        .and(state_filter)
        .and_then(stats_handler);

    socket
        .or(connect)
        .or(poll)
        .or(send)
        .or(text)
        .or(stats)
        .boxed()
}

/// Query parameters for the long-polling `poll` and `send` endpoints.
#[derive(Deserialize)]
struct PollQuery {
    /// Opaque session token returned by the `connect` endpoint.
    session: String,
}

/// Look up the document for `id`, loading or creating it if necessary.
///
/// Mirrors the get-or-create logic previously inlined in `socket_handler`, so
/// the WebSocket and long-polling endpoints share a single code path.
async fn get_document(state: &ServerState, id: &str) -> Result<Arc<Rustpad>, Rejection> {
    use dashmap::mapref::entry::Entry;

    let entry = match state.documents.entry(id.to_string()) {
        Entry::Occupied(e) => e.into_ref(),
        Entry::Vacant(e) => {
            let rustpad = match &state.database {
                Some(db) => match db.load(id).await {
                    Ok(Some(doc)) => Rustpad::from(doc),
                    Ok(None) => Rustpad::default(),
                    // A real database error must not be turned into an empty
                    // document: that could clobber real saved content on the
                    // next persist. Reject so the client simply retries.
                    Err(err) => {
                        error!("failed to load document {}: {}", id, err);
                        return Err(warp::reject::custom(CustomReject(err)));
                    }
                },
                None => Rustpad::default(),
            };
            let rustpad = Arc::new(rustpad);
            if let Some(db) = &state.database {
                tokio::spawn(persister(id.to_string(), Arc::clone(&rustpad), db.clone()));
            }
            e.insert(Document::new(rustpad))
        }
    };

    let rustpad = Arc::clone(&entry.rustpad);
    rustpad.touch();
    Ok(rustpad)
}

/// Handler for the `/api/socket/{id}` endpoint.
async fn socket_handler(id: String, ws: Ws, state: ServerState) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await?;
    // Permit messages large enough to carry a full-document history (e.g. a
    // multi-megabyte log pasted in one operation), sized from the document cap.
    let limit = max_document_bytes().saturating_mul(2);
    let ws = ws.max_message_size(limit).max_frame_size(limit);
    Ok(ws.on_upgrade(|socket| async move { rustpad.on_connection(socket).await }))
}

/// Handler for the `/api/connect/{id}` endpoint (long-polling).
///
/// Establishes a new polling session and returns its token together with the
/// initial burst of messages.
async fn connect_handler(id: String, state: ServerState) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await?;
    let (session, messages) = rustpad.new_poll_session();
    Ok(warp::reply::json(
        &serde_json::json!({ "session": session, "messages": messages }),
    ))
}

/// Default long-poll hold duration, in seconds. Kept under typical intermediary
/// idle timeouts so the request always returns before any proxy or load
/// balancer resets it, and still looks like an ordinary, promptly-returning
/// HTTPS GET. Override with `POLL_TIMEOUT_SECS`.
const DEFAULT_POLL_TIMEOUT_SECS: u64 = 15;

/// The configured long-poll hold duration.
fn poll_timeout() -> Duration {
    let secs = std::env::var("POLL_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_POLL_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

/// Handler for the `/api/poll/{id}` endpoint (long-polling).
async fn poll_handler(
    id: String,
    query: PollQuery,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await?;
    match rustpad.poll_session(&query.session, poll_timeout()).await {
        Some(messages) => Ok(warp::reply::with_status(
            warp::reply::json(&messages),
            StatusCode::OK,
        )),
        None => Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({ "error": "unknown session" })),
            StatusCode::CONFLICT,
        )),
    }
}

/// Handler for the `/api/send/{id}` endpoint (long-polling).
async fn send_handler(
    id: String,
    query: PollQuery,
    body: bytes::Bytes,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await?;
    // The body is base64-encoded JSON (so an intermediary WAF cannot match the
    // document content). Decode it back to the original message.
    let decoded = match base64::engine::general_purpose::STANDARD.decode(body.as_ref()) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };
    let message = match std::str::from_utf8(&decoded) {
        Ok(text) => text,
        Err(_) => return Ok(StatusCode::BAD_REQUEST),
    };
    match rustpad.apply_poll_message(&query.session, message) {
        Ok(()) => Ok(StatusCode::OK),
        Err(_) => Ok(StatusCode::CONFLICT),
    }
}

/// Handler for the `/api/text/{id}` endpoint.
async fn text_handler(id: String, state: ServerState) -> Result<impl Reply, Rejection> {
    Ok(match state.documents.get(&id) {
        Some(value) => value.rustpad.text(),
        None => {
            if let Some(db) = &state.database {
                // On a database error, fall back to empty rather than failing
                // the read; this endpoint is a best-effort text snapshot.
                db.load(&id)
                    .await
                    .ok()
                    .flatten()
                    .map(|document| document.text)
                    .unwrap_or_default()
            } else {
                String::new()
            }
        }
    })
}

/// Handler for the `/api/stats` endpoint.
async fn stats_handler(start_time: u64, state: ServerState) -> Result<impl Reply, Rejection> {
    let num_documents = state.documents.len();
    let database_size = match state.database {
        None => 0,
        Some(db) => match db.count().await {
            Ok(size) => size,
            Err(e) => return Err(warp::reject::custom(CustomReject(e))),
        },
    };
    Ok(warp::reply::json(&Stats {
        start_time,
        num_documents,
        database_size,
    }))
}

/// How often to sweep for idle documents.
const CLEAN_INTERVAL: Duration = Duration::from_secs(60);

/// Default idle period after which a document with no clients is cleared.
const DEFAULT_IDLE_CLEAR_SECS: u64 = 300;

/// The idle period after which an unused document is cleared from memory and
/// deleted from the database. Override with `IDLE_CLEAR_SECS`.
fn idle_clear_duration() -> Duration {
    let secs = std::env::var("IDLE_CLEAR_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_IDLE_CLEAR_SECS);
    Duration::from_secs(secs)
}

/// Clears documents that no client has had open for `idle`, removing them from
/// memory and deleting their stored row so neither grows without bound.
async fn cleaner(state: ServerState, idle: Duration) {
    loop {
        time::sleep(CLEAN_INTERVAL).await;
        let keys: Vec<String> = state
            .documents
            .iter()
            .filter(|entry| entry.rustpad.idle_for() > idle)
            .map(|entry| entry.key().clone())
            .collect();
        if keys.is_empty() {
            continue;
        }
        info!("cleaner clearing idle documents: {:?}", keys);
        for key in keys {
            // Drop from memory first (this kills the document and stops its
            // persister), then delete the stored row.
            state.documents.remove(&key);
            if let Some(db) = &state.database {
                if let Err(e) = db.delete(&key).await {
                    warn!("when deleting idle document {}: {}", key, e);
                }
            }
        }
    }
}

/// How often to sweep for idle long-polling sessions.
const SESSION_SWEEP_INTERVAL: Duration = Duration::from_secs(15);

/// A polling session is dropped if it has not polled or sent within this window.
/// Comfortably larger than the long-poll hold so a client mid-poll is never
/// reaped (keep above `POLL_TIMEOUT_SECS` if that is raised).
const SESSION_MAX_IDLE: Duration = Duration::from_secs(45);

/// Removes idle long-polling sessions across all documents, freeing their
/// presence (users and cursors) just as a WebSocket disconnect would.
async fn session_cleaner(state: ServerState) {
    loop {
        time::sleep(SESSION_SWEEP_INTERVAL).await;
        for entry in &*state.documents {
            entry.rustpad.sweep_sessions(SESSION_MAX_IDLE);
        }
    }
}

const PERSIST_INTERVAL: Duration = Duration::from_secs(3);
const PERSIST_INTERVAL_JITTER: Duration = Duration::from_secs(1);

/// Persists changed documents after a fixed time interval.
async fn persister(id: String, rustpad: Arc<Rustpad>, db: Database) {
    // Start from the current revision so a freshly loaded, unedited document is
    // not needlessly re-written; only genuine new edits are persisted.
    let mut last_revision = rustpad.revision();
    while !rustpad.killed() {
        let interval = PERSIST_INTERVAL
            + rand::thread_rng().gen_range(Duration::ZERO..=PERSIST_INTERVAL_JITTER);
        time::sleep(interval).await;
        let revision = rustpad.revision();
        if revision > last_revision {
            let snapshot = rustpad.snapshot();
            // Keep storage minimal: a document that has been emptied is deleted
            // rather than stored as a blank row.
            let result = if snapshot.text.is_empty() && snapshot.language.is_none() {
                db.delete(&id).await
            } else {
                info!("persisting revision {} for id = {}", revision, id);
                db.store(&id, &snapshot).await
            };
            match result {
                Ok(()) => last_revision = revision,
                Err(e) => error!("when persisting document {}: {}", id, e),
            }
        }
    }
}
