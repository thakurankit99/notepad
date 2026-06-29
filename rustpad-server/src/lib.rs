//! Server backend for the Rustpad collaborative text editor.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use dashmap::DashMap;
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::time::{self, Instant};
use warp::{filters::BoxedFilter, http::StatusCode, ws::Ws, Filter, Rejection, Reply};

use crate::{database::Database, rustpad::Rustpad};

pub mod database;
mod ot;
mod rustpad;

/// An entry stored in the global server map.
///
/// Each entry corresponds to a single document. This is garbage collected by a
/// background task after one day of inactivity, to avoid server memory usage
/// growing without bound.
struct Document {
    last_accessed: Instant,
    rustpad: Arc<Rustpad>,
}

impl Document {
    fn new(rustpad: Arc<Rustpad>) -> Self {
        Self {
            last_accessed: Instant::now(),
            rustpad,
        }
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
    tokio::spawn(cleaner(state.clone(), config.expiry_days));
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

    let send = warp::path!("send" / String)
        .and(warp::query::<PollQuery>())
        .and(warp::post())
        .and(warp::body::content_length_limit(512 * 1024))
        .and(warp::body::json())
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
async fn get_document(state: &ServerState, id: &str) -> Arc<Rustpad> {
    use dashmap::mapref::entry::Entry;

    let mut entry = match state.documents.entry(id.to_string()) {
        Entry::Occupied(e) => e.into_ref(),
        Entry::Vacant(e) => {
            let rustpad = Arc::new(match &state.database {
                Some(db) => db.load(id).await.map(Rustpad::from).unwrap_or_default(),
                None => Rustpad::default(),
            });
            if let Some(db) = &state.database {
                tokio::spawn(persister(id.to_string(), Arc::clone(&rustpad), db.clone()));
            }
            e.insert(Document::new(rustpad))
        }
    };

    let value = entry.value_mut();
    value.last_accessed = Instant::now();
    Arc::clone(&value.rustpad)
}

/// Handler for the `/api/socket/{id}` endpoint.
async fn socket_handler(id: String, ws: Ws, state: ServerState) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await;
    Ok(ws.on_upgrade(|socket| async move { rustpad.on_connection(socket).await }))
}

/// Handler for the `/api/connect/{id}` endpoint (long-polling).
///
/// Establishes a new polling session and returns its token together with the
/// initial burst of messages.
async fn connect_handler(id: String, state: ServerState) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await;
    let (session, messages) = rustpad.new_poll_session();
    Ok(warp::reply::json(
        &serde_json::json!({ "session": session, "messages": messages }),
    ))
}

/// Long-poll hold duration. Kept under typical proxy idle timeouts (Zscaler and
/// similar) so the request looks like an ordinary, promptly-returning HTTPS GET.
const POLL_TIMEOUT: Duration = Duration::from_secs(20);

/// Handler for the `/api/poll/{id}` endpoint (long-polling).
async fn poll_handler(
    id: String,
    query: PollQuery,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await;
    match rustpad.poll_session(&query.session, POLL_TIMEOUT).await {
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
    body: serde_json::Value,
    state: ServerState,
) -> Result<impl Reply, Rejection> {
    let rustpad = get_document(&state, &id).await;
    match rustpad.apply_poll_message(&query.session, &body.to_string()) {
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
                db.load(&id)
                    .await
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

const HOUR: Duration = Duration::from_secs(3600);

/// Reclaims memory for documents.
async fn cleaner(state: ServerState, expiry_days: u32) {
    loop {
        time::sleep(HOUR).await;
        let mut keys = Vec::new();
        for entry in &*state.documents {
            if entry.last_accessed.elapsed() > HOUR * 24 * expiry_days {
                keys.push(entry.key().clone());
            }
        }
        info!("cleaner removing keys: {:?}", keys);
        for key in keys {
            state.documents.remove(&key);
        }
    }
}

/// How often to sweep for idle long-polling sessions.
const SESSION_SWEEP_INTERVAL: Duration = Duration::from_secs(15);

/// A polling session is dropped if it has not polled or sent within this window.
/// Comfortably larger than `POLL_TIMEOUT` so a client mid-poll is never reaped.
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
    let mut last_revision = 0;
    while !rustpad.killed() {
        let interval = PERSIST_INTERVAL
            + rand::thread_rng().gen_range(Duration::ZERO..=PERSIST_INTERVAL_JITTER);
        time::sleep(interval).await;
        let revision = rustpad.revision();
        if revision > last_revision {
            info!("persisting revision {} for id = {}", revision, id);
            if let Err(e) = db.store(&id, &rustpad.snapshot()).await {
                error!("when persisting document {}: {}", id, e);
            } else {
                last_revision = revision;
            }
        }
    }
}
