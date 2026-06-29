//! Eventually consistent server-side logic for Rustpad.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use dashmap::DashMap;
use futures::prelude::*;
use log::{info, warn};
use operational_transform::OperationSeq;
use parking_lot::{Mutex, RwLock, RwLockUpgradableReadGuard};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex as AsyncMutex, Notify};
use warp::ws::{Message, WebSocket};

use crate::{database::PersistedDocument, ot::transform_index};

/// Interval at which the server pings idle WebSocket clients to keep the
/// connection from being reset by an intermediary with an idle timeout.
const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// Default maximum document size, in Unicode code points. Large enough to hold
/// a sizeable log (tens of thousands of lines) while still bounding memory.
const DEFAULT_MAX_DOC_BYTES: usize = 16 * 1024 * 1024;

/// The maximum size a document may reach, in Unicode code points. Bounds memory
/// use and rejects pathological input. Override with `MAX_DOC_BYTES`.
pub fn max_document_bytes() -> usize {
    static MAX: OnceLock<usize> = OnceLock::new();
    *MAX.get_or_init(|| {
        std::env::var("MAX_DOC_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(DEFAULT_MAX_DOC_BYTES)
    })
}

/// State for a single HTTP long-polling client.
///
/// A polling session is the long-poll equivalent of a WebSocket connection. It
/// exists so that clients behind proxies that block the WebSocket `Upgrade`
/// handshake can still participate using ordinary HTTPS requests. Each session
/// owns a user id and a broadcast receiver that persist across the individual,
/// short-lived poll requests.
struct PollSession {
    /// The user id assigned to this session, as used in the shared state.
    user_id: u64,
    /// Receiver for metadata updates, held across separate poll requests.
    receiver: AsyncMutex<broadcast::Receiver<ServerMsg>>,
    /// The last operation revision already delivered to this session.
    last_revision: AtomicUsize,
    /// When this session was last seen, used to garbage collect idle sessions.
    last_seen: Mutex<Instant>,
}

/// The main object representing a collaborative session.
pub struct Rustpad {
    /// State modified by critical sections of the code.
    state: RwLock<State>,
    /// Incremented to obtain unique user IDs.
    count: AtomicU64,
    /// Used to notify clients of new text operations.
    notify: Notify,
    /// Used to inform all clients of metadata updates.
    update: broadcast::Sender<ServerMsg>,
    /// Set to true when the document is destroyed.
    killed: AtomicBool,
    /// Active HTTP long-polling sessions, keyed by an opaque session token.
    sessions: DashMap<String, Arc<PollSession>>,
    /// Last time a client connected, polled, or sent a message. Used to detect
    /// documents that no machine has open so they can be cleared.
    last_active: Mutex<Instant>,
}

/// Shared state involving multiple users, protected by a lock.
#[derive(Default)]
struct State {
    operations: Vec<UserOperation>,
    text: String,
    language: Option<String>,
    users: HashMap<u64, UserInfo>,
    cursors: HashMap<u64, CursorData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserOperation {
    id: u64,
    operation: OperationSeq,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct UserInfo {
    name: String,
    hue: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CursorData {
    cursors: Vec<u32>,
    selections: Vec<(u32, u32)>,
}

/// A message received from the client over WebSocket.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum ClientMsg {
    /// Represents a sequence of local edits from the user.
    Edit {
        revision: usize,
        operation: OperationSeq,
    },
    /// Sets the language of the editor.
    SetLanguage(String),
    /// Sets the user's current information.
    ClientInfo(UserInfo),
    /// Sets the user's cursor and selection positions.
    CursorData(CursorData),
}

/// A message sent to the client over WebSocket.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum ServerMsg {
    /// Informs the client of their unique socket ID.
    Identity(u64),
    /// Broadcasts text operations to all clients.
    History {
        start: usize,
        operations: Vec<UserOperation>,
    },
    /// Broadcasts the current language, last writer wins.
    Language(String),
    /// Broadcasts a user's information, or `None` on disconnect.
    UserInfo { id: u64, info: Option<UserInfo> },
    /// Broadcasts a user's cursor position.
    UserCursor { id: u64, data: CursorData },
}

impl From<ServerMsg> for Message {
    fn from(msg: ServerMsg) -> Self {
        let serialized = serde_json::to_string(&msg).expect("failed serialize");
        Message::text(serialized)
    }
}

impl Default for Rustpad {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(16);
        Self {
            state: Default::default(),
            count: Default::default(),
            notify: Default::default(),
            update: tx,
            killed: AtomicBool::new(false),
            sessions: Default::default(),
            last_active: Mutex::new(Instant::now()),
        }
    }
}

impl From<PersistedDocument> for Rustpad {
    fn from(document: PersistedDocument) -> Self {
        let mut operation = OperationSeq::default();
        operation.insert(&document.text);

        let rustpad = Self::default();
        {
            let mut state = rustpad.state.write();
            state.text = document.text;
            state.language = document.language;
            state.operations.push(UserOperation {
                id: u64::MAX,
                operation,
            })
        }
        rustpad
    }
}

impl Rustpad {
    /// Handle a connection from a WebSocket.
    pub async fn on_connection(&self, socket: WebSocket) {
        let id = self.count.fetch_add(1, Ordering::Relaxed);
        self.touch();
        info!("connection! id = {}", id);
        if let Err(e) = self.handle_connection(id, socket).await {
            warn!("connection terminated early: {}", e);
        }
        info!("disconnection, id = {}", id);
        self.state.write().users.remove(&id);
        self.state.write().cursors.remove(&id);
        self.update
            .send(ServerMsg::UserInfo { id, info: None })
            .ok();
    }

    /// Returns a snapshot of the latest text.
    pub fn text(&self) -> String {
        let state = self.state.read();
        state.text.clone()
    }

    /// Returns a snapshot of the current document for persistence.
    pub fn snapshot(&self) -> PersistedDocument {
        let state = self.state.read();
        PersistedDocument {
            text: state.text.clone(),
            language: state.language.clone(),
        }
    }

    /// Returns the current revision.
    pub fn revision(&self) -> usize {
        let state = self.state.read();
        state.operations.len()
    }

    /// Kill this object immediately, dropping all current connections.
    pub fn kill(&self) {
        self.killed.store(true, Ordering::Relaxed);
        self.notify.notify_waiters();
    }

    /// Returns if this Rustpad object has been killed.
    pub fn killed(&self) -> bool {
        self.killed.load(Ordering::Relaxed)
    }

    /// Record that a client is actively using this document right now.
    pub fn touch(&self) {
        *self.last_active.lock() = Instant::now();
    }

    /// How long since any client last connected, polled, or sent a message.
    pub fn idle_for(&self) -> Duration {
        self.last_active.lock().elapsed()
    }

    /// Create a new HTTP long-polling session.
    ///
    /// Returns an opaque session token and the initial burst of messages
    /// (serialized as a JSON array), mirroring what a freshly connected
    /// WebSocket client would receive.
    pub fn new_poll_session(&self) -> (String, serde_json::Value) {
        self.touch();
        let user_id = self.count.fetch_add(1, Ordering::Relaxed);
        info!("poll connection! id = {}", user_id);
        // Subscribe before snapshotting so no metadata update is missed.
        let receiver = self.update.subscribe();
        let (messages, revision) = self.build_initial(user_id);
        let token = format!("{:016x}", rand::thread_rng().gen::<u64>());
        self.sessions.insert(
            token.clone(),
            Arc::new(PollSession {
                user_id,
                receiver: AsyncMutex::new(receiver),
                last_revision: AtomicUsize::new(revision),
                last_seen: Mutex::new(Instant::now()),
            }),
        );
        let value = serde_json::to_value(&messages).expect("failed to serialize initial messages");
        (token, value)
    }

    /// Long-poll for new messages on a session, blocking up to `timeout`.
    ///
    /// Returns `None` if the session token is unknown (expired or invalid), in
    /// which case the client should reconnect. Otherwise returns a JSON array
    /// of new messages, which may be empty if the timeout elapsed first.
    pub async fn poll_session(&self, token: &str, timeout: Duration) -> Option<serde_json::Value> {
        let session = match self.sessions.get(token) {
            Some(s) => Arc::clone(s.value()),
            None => return None,
        };
        self.touch();
        *session.last_seen.lock() = Instant::now();
        // A single in-flight poll per session; serialized by this lock.
        let mut receiver = session.receiver.lock().await;
        let deadline = tokio::time::Instant::now() + timeout;
        let mut pending: Vec<ServerMsg> = Vec::new();

        loop {
            // Register interest in notifications *before* inspecting state, the
            // same ordering the WebSocket handler uses, so an operation applied
            // concurrently between the check and the wait cannot be lost.
            let notified = self.notify.notified();

            if self.killed() {
                let value =
                    serde_json::to_value(&pending).expect("failed to serialize messages");
                return Some(value);
            }

            // Collect any new operation history beyond what this session has seen.
            let last_rev = session.last_revision.load(Ordering::Relaxed);
            let cur_rev = self.revision();
            if cur_rev > last_rev {
                let operations = {
                    let state = self.state.read();
                    state.operations[last_rev..].to_owned()
                };
                pending.push(ServerMsg::History {
                    start: last_rev,
                    operations,
                });
                session.last_revision.store(cur_rev, Ordering::Relaxed);
            }

            // Drain any buffered metadata updates without blocking.
            loop {
                match receiver.try_recv() {
                    Ok(msg) => pending.push(msg),
                    Err(broadcast::error::TryRecvError::Empty) => break,
                    Err(broadcast::error::TryRecvError::Closed) => break,
                    Err(broadcast::error::TryRecvError::Lagged(_)) => self.push_resync(&mut pending),
                }
            }

            if !pending.is_empty() {
                let value =
                    serde_json::to_value(&pending).expect("failed to serialize messages");
                return Some(value);
            }

            // Nothing ready: wait for an operation, a metadata update, or timeout.
            tokio::select! {
                _ = notified => {}
                result = receiver.recv() => {
                    match result {
                        Ok(msg) => pending.push(msg),
                        Err(broadcast::error::RecvError::Lagged(_)) => self.push_resync(&mut pending),
                        Err(broadcast::error::RecvError::Closed) => {}
                    }
                }
                _ = tokio::time::sleep_until(deadline) => {
                    let value =
                        serde_json::to_value(&pending).expect("failed to serialize messages");
                    return Some(value);
                }
            }
        }
    }

    /// Apply a client message received over the long-polling `send` endpoint.
    ///
    /// Returns an error if the session token is unknown or the message is
    /// invalid.
    pub fn apply_poll_message(&self, token: &str, text: &str) -> Result<()> {
        self.touch();
        let user_id = match self.sessions.get(token) {
            Some(session) => {
                *session.last_seen.lock() = Instant::now();
                session.user_id
            }
            None => bail!("unknown session"),
        };
        let msg: ClientMsg =
            serde_json::from_str(text).context("failed to deserialize message")?;
        self.process_client_msg(user_id, msg)
    }

    /// Remove idle long-polling sessions, cleaning up their presence.
    ///
    /// A session whose last poll or send was longer ago than `max_idle` is
    /// dropped, and its user is removed from the shared state exactly as a
    /// WebSocket disconnect would do.
    pub fn sweep_sessions(&self, max_idle: Duration) {
        let now = Instant::now();
        let stale: Vec<String> = self
            .sessions
            .iter()
            .filter(|entry| now.duration_since(*entry.value().last_seen.lock()) > max_idle)
            .map(|entry| entry.key().clone())
            .collect();
        for token in stale {
            if let Some((_, session)) = self.sessions.remove(&token) {
                let id = session.user_id;
                info!("poll disconnection, id = {}", id);
                self.state.write().users.remove(&id);
                self.state.write().cursors.remove(&id);
                self.update
                    .send(ServerMsg::UserInfo { id, info: None })
                    .ok();
            }
        }
    }

    async fn handle_connection(&self, id: u64, mut socket: WebSocket) -> Result<()> {
        let mut update_rx = self.update.subscribe();

        let mut revision: usize = self.send_initial(id, &mut socket).await?;

        // Periodically ping the client so an idle connection is not reaped by
        // an intermediary (load balancer / CDN) that resets connections with no
        // traffic. The first tick fires immediately, so consume it up front.
        let mut keepalive = tokio::time::interval(KEEPALIVE_INTERVAL);
        keepalive.tick().await;

        loop {
            // In order to avoid the "lost wakeup" problem, we first request a
            // notification, **then** check the current state for new revisions.
            // This is the same approach that `tokio::sync::watch` takes.
            let notified = self.notify.notified();
            if self.killed() {
                break;
            }
            if self.revision() > revision {
                revision = self.send_history(revision, &mut socket).await?
            }

            tokio::select! {
                _ = notified => {}
                _ = keepalive.tick() => {
                    // A live connection counts as activity, so the document is
                    // not cleared out from under a client that is simply idle.
                    self.touch();
                    socket.send(Message::ping(Vec::new())).await?;
                }
                update = update_rx.recv() => {
                    socket.send(update?.into()).await?;
                }
                result = socket.next() => {
                    match result {
                        None => break,
                        Some(message) => {
                            self.touch();
                            self.handle_message(id, message?).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Build the initial burst of messages for a newly connected client.
    ///
    /// Returns the messages (starting with the client's `Identity`) and the
    /// current revision, so the caller can track which operations have already
    /// been delivered. Shared by both the WebSocket and long-polling paths.
    fn build_initial(&self, id: u64) -> (Vec<ServerMsg>, usize) {
        let mut messages = vec![ServerMsg::Identity(id)];
        let state = self.state.read();
        if !state.operations.is_empty() {
            messages.push(ServerMsg::History {
                start: 0,
                operations: state.operations.clone(),
            });
        }
        if let Some(language) = &state.language {
            messages.push(ServerMsg::Language(language.clone()));
        }
        for (&id, info) in &state.users {
            messages.push(ServerMsg::UserInfo {
                id,
                info: Some(info.clone()),
            });
        }
        for (&id, data) in &state.cursors {
            messages.push(ServerMsg::UserCursor {
                id,
                data: data.clone(),
            });
        }
        (messages, state.operations.len())
    }

    /// Push a full snapshot of metadata (language, users, cursors) onto `out`.
    ///
    /// Used to recover a long-polling session that fell too far behind the
    /// metadata broadcast channel (a `Lagged` error).
    fn push_resync(&self, out: &mut Vec<ServerMsg>) {
        let state = self.state.read();
        if let Some(language) = &state.language {
            out.push(ServerMsg::Language(language.clone()));
        }
        for (&id, info) in &state.users {
            out.push(ServerMsg::UserInfo {
                id,
                info: Some(info.clone()),
            });
        }
        for (&id, data) in &state.cursors {
            out.push(ServerMsg::UserCursor {
                id,
                data: data.clone(),
            });
        }
    }

    async fn send_initial(&self, id: u64, socket: &mut WebSocket) -> Result<usize> {
        let (messages, revision) = self.build_initial(id);
        for msg in messages {
            socket.send(msg.into()).await?;
        }
        Ok(revision)
    }

    async fn send_history(&self, start: usize, socket: &mut WebSocket) -> Result<usize> {
        let operations = {
            let state = self.state.read();
            let len = state.operations.len();
            if start < len {
                state.operations[start..].to_owned()
            } else {
                Vec::new()
            }
        };
        let num_ops = operations.len();
        if num_ops > 0 {
            let msg = ServerMsg::History { start, operations };
            socket.send(msg.into()).await?;
        }
        Ok(start + num_ops)
    }

    async fn handle_message(&self, id: u64, message: Message) -> Result<()> {
        let msg: ClientMsg = match message.to_str() {
            Ok(text) => serde_json::from_str(text).context("failed to deserialize message")?,
            Err(()) => return Ok(()), // Ignore non-text messages
        };
        self.process_client_msg(id, msg)
    }

    /// Apply a single decoded client message, regardless of transport.
    ///
    /// Shared by the WebSocket handler and the HTTP long-polling `send`
    /// endpoint, so both paths drive the same operational-transform logic.
    fn process_client_msg(&self, id: u64, msg: ClientMsg) -> Result<()> {
        match msg {
            ClientMsg::Edit {
                revision,
                operation,
            } => {
                self.apply_edit(id, revision, operation)
                    .context("invalid edit operation")?;
                self.notify.notify_waiters();
            }
            ClientMsg::SetLanguage(language) => {
                self.state.write().language = Some(language.clone());
                self.update.send(ServerMsg::Language(language)).ok();
            }
            ClientMsg::ClientInfo(info) => {
                self.state.write().users.insert(id, info.clone());
                let msg = ServerMsg::UserInfo {
                    id,
                    info: Some(info),
                };
                self.update.send(msg).ok();
            }
            ClientMsg::CursorData(data) => {
                self.state.write().cursors.insert(id, data.clone());
                let msg = ServerMsg::UserCursor { id, data };
                self.update.send(msg).ok();
            }
        }
        Ok(())
    }

    fn apply_edit(&self, id: u64, revision: usize, mut operation: OperationSeq) -> Result<()> {
        info!(
            "edit: id = {}, revision = {}, base_len = {}, target_len = {}",
            id,
            revision,
            operation.base_len(),
            operation.target_len()
        );
        let state = self.state.upgradable_read();
        let len = state.operations.len();
        if revision > len {
            bail!("got revision {}, but current is {}", revision, len);
        }
        for history_op in &state.operations[revision..] {
            operation = operation.transform(&history_op.operation)?.0;
        }
        let max_len = max_document_bytes();
        if operation.target_len() > max_len {
            bail!(
                "target length {} is greater than maximum of {}",
                operation.target_len(),
                max_len
            );
        }
        let new_text = operation.apply(&state.text)?;
        let mut state = RwLockUpgradableReadGuard::upgrade(state);
        for (_, data) in state.cursors.iter_mut() {
            for cursor in data.cursors.iter_mut() {
                *cursor = transform_index(&operation, *cursor);
            }
            for (start, end) in data.selections.iter_mut() {
                *start = transform_index(&operation, *start);
                *end = transform_index(&operation, *end);
            }
        }
        state.operations.push(UserOperation { id, operation });
        state.text = new_text;
        Ok(())
    }
}
