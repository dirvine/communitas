//! Messaging service for thread and message operations with reactive subscriptions.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event, Query, QueryResponse, Subscription};
use communitas_ui_api::{
    Message, MessageSendStatus, PendingMessage, SearchResult, SyncState, ThreadSummary,
};

use crate::presence::PresenceSnapshot;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use thiserror::Error;
use tokio::sync::{broadcast, watch};
use tracing::{debug, instrument, trace, warn};

use crate::storage::{JsonFile, UiStorage};

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::messaging_convert::{
    core_entity_type_to_ui, core_message_to_ui, core_search_result_to_ui,
};
use communitas_core::legacy_crdt::EntityType;

/// Errors returned by the messaging service.
#[derive(Debug, Error)]
pub enum MessagingError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("thread not found: {0}")]
    ThreadNotFound(String),
    #[error("message send failed: {0}")]
    SendFailed(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Snapshot of messaging state for reactive UI updates.
#[derive(Debug, Clone, Default)]
pub struct MessagingSnapshot {
    /// All conversation threads.
    pub threads: Vec<ThreadSummary>,
    /// Whether threads are currently being loaded.
    pub loading: bool,
    /// Messages pending send (offline queue).
    pub pending_messages: Vec<PendingMessage>,
}

/// Persisted unread message counts per thread.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnreadCounts {
    /// Map from thread_id to unread count.
    pub counts: HashMap<String, u32>,
}

impl UnreadCounts {
    /// Increment the unread count for a thread.
    pub fn increment(&mut self, thread_id: &str) {
        *self.counts.entry(thread_id.to_string()).or_insert(0) += 1;
    }

    /// Reset the unread count for a thread to zero.
    pub fn reset(&mut self, thread_id: &str) {
        self.counts.insert(thread_id.to_string(), 0);
    }

    /// Get the unread count for a thread.
    pub fn get(&self, thread_id: &str) -> u32 {
        self.counts.get(thread_id).copied().unwrap_or(0)
    }
}

/// Maximum number of pinned threads allowed.
pub const MAX_PINNED_THREADS: usize = 5;

/// Persisted list of pinned thread IDs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PinnedThreads {
    /// Ordered list of pinned thread IDs (most recently pinned last).
    pub thread_ids: Vec<String>,
}

impl PinnedThreads {
    /// Check if a thread is pinned.
    pub fn is_pinned(&self, thread_id: &str) -> bool {
        self.thread_ids.contains(&thread_id.to_string())
    }

    /// Pin a thread. Returns true if newly pinned, false if already pinned.
    /// Returns an error message if the limit would be exceeded.
    pub fn pin(&mut self, thread_id: &str) -> Result<bool, &'static str> {
        if self.is_pinned(thread_id) {
            return Ok(false);
        }
        if self.thread_ids.len() >= MAX_PINNED_THREADS {
            return Err("Maximum number of pinned threads reached");
        }
        self.thread_ids.push(thread_id.to_string());
        Ok(true)
    }

    /// Unpin a thread. Returns true if unpinned, false if wasn't pinned.
    pub fn unpin(&mut self, thread_id: &str) -> bool {
        let initial_len = self.thread_ids.len();
        self.thread_ids.retain(|id| id != thread_id);
        self.thread_ids.len() < initial_len
    }
}

/// Typing indicator state with auto-expire tracking.
///
/// Tracks which users are typing in each thread, with timestamps for auto-expiration.
/// Typing indicators expire after 3 seconds of inactivity.
#[derive(Debug, Clone, Default)]
pub struct TypingState {
    /// Map from thread_id to (peer_id -> last_typing_time).
    state: HashMap<String, HashMap<String, Instant>>,
}

/// Duration after which typing indicators expire.
const TYPING_EXPIRE_DURATION: Duration = Duration::from_secs(3);

impl TypingState {
    /// Update typing state for a peer in a thread.
    ///
    /// If `is_typing` is true, records the current time. If false, removes the entry.
    pub fn update(&mut self, thread_id: &str, peer_id: &str, is_typing: bool) {
        if is_typing {
            self.state
                .entry(thread_id.to_string())
                .or_default()
                .insert(peer_id.to_string(), Instant::now());
        } else if let Some(thread_typing) = self.state.get_mut(thread_id) {
            thread_typing.remove(peer_id);
            if thread_typing.is_empty() {
                self.state.remove(thread_id);
            }
        }
    }

    /// Get the list of currently typing users for a thread (excluding expired).
    pub fn get_typing_users(&self, thread_id: &str) -> Vec<String> {
        let now = Instant::now();
        self.state
            .get(thread_id)
            .map(|thread_typing| {
                thread_typing
                    .iter()
                    .filter(|(_, timestamp)| {
                        now.duration_since(**timestamp) < TYPING_EXPIRE_DURATION
                    })
                    .map(|(peer_id, _)| peer_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Remove all expired typing indicators.
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.state.retain(|_, thread_typing| {
            thread_typing
                .retain(|_, timestamp| now.duration_since(*timestamp) < TYPING_EXPIRE_DURATION);
            !thread_typing.is_empty()
        });
    }
}

/// Persisted pending messages (offline queue).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingMessageQueue {
    /// Messages waiting to be sent.
    pub messages: Vec<PendingMessage>,
}

/// Maximum retry attempts for pending messages.
pub const MAX_PENDING_RETRIES: u32 = 5;

/// Service for thread listing, message retrieval, and message sending.
pub struct MessagingService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    storage: Arc<UiStorage>,
    tx: watch::Sender<MessagingSnapshot>,
    rx: watch::Receiver<MessagingSnapshot>,
    /// Persistent unread counts (shared between service and event loop).
    unread_counts: Arc<RwLock<UnreadCounts>>,
    /// Currently active thread (messages in this thread don't increment unread).
    active_thread: Arc<RwLock<Option<String>>>,
    /// Typing indicator state (shared between service and event loop).
    typing_state: Arc<RwLock<TypingState>>,
    /// Persistent pinned threads (shared between service and event loop).
    pinned_threads: Arc<RwLock<PinnedThreads>>,
    /// Presence state receiver for DM thread contact presence.
    presence_rx: watch::Receiver<PresenceSnapshot>,
    /// Pending messages (offline send queue).
    pending_messages: Arc<RwLock<PendingMessageQueue>>,
}

impl MessagingService {
    /// Create a new messaging service linked to the auth controller and core app.
    ///
    /// Automatically subscribes to message events from the core app and updates
    /// the watch channel reactively when messages are sent, received, edited,
    /// deleted, or when reactions are added/removed.
    ///
    /// # Arguments
    /// * `auth` - Shared authentication controller for checking login state
    /// * `app` - Shared reference to the core application
    /// * `storage` - UI storage for persisting unread counts
    /// * `presence_rx` - Receiver for presence state updates (for DM thread contact presence)
    pub fn new(
        auth: Arc<AuthController>,
        app: Arc<CommunitasApp>,
        storage: Arc<UiStorage>,
        presence_rx: watch::Receiver<PresenceSnapshot>,
    ) -> Self {
        let (tx, rx) = watch::channel(MessagingSnapshot::default());

        // Load persisted unread counts
        let unread_counts = match JsonFile::load(&storage.unread_counts_file()) {
            Ok(Some(counts)) => counts,
            Ok(None) => UnreadCounts::default(),
            Err(e) => {
                warn!(error = %e, "Failed to load unread counts, using defaults");
                UnreadCounts::default()
            }
        };
        let unread_counts = Arc::new(RwLock::new(unread_counts));
        let active_thread = Arc::new(RwLock::new(None));
        let typing_state = Arc::new(RwLock::new(TypingState::default()));

        // Load persisted pinned threads
        let pinned_threads = match JsonFile::load(&storage.pinned_threads_file()) {
            Ok(Some(pinned)) => pinned,
            Ok(None) => PinnedThreads::default(),
            Err(e) => {
                warn!(error = %e, "Failed to load pinned threads, using defaults");
                PinnedThreads::default()
            }
        };
        let pinned_threads = Arc::new(RwLock::new(pinned_threads));

        // Load persisted pending messages
        let pending_messages = match JsonFile::load(&storage.pending_messages_file()) {
            Ok(Some(queue)) => queue,
            Ok(None) => PendingMessageQueue::default(),
            Err(e) => {
                warn!(error = %e, "Failed to load pending messages, using defaults");
                PendingMessageQueue::default()
            }
        };
        let pending_messages = Arc::new(RwLock::new(pending_messages));

        // Subscribe to message events for reactive updates
        let event_rx = app.subscribe(Subscription::MessageEvents);

        // Clone what we need for the background task
        let tx_clone = tx.clone();
        let app_clone = app.clone();
        let auth_clone = auth.clone();
        let storage_clone = storage.clone();
        let unread_clone = unread_counts.clone();
        let active_clone = active_thread.clone();
        let typing_clone = typing_state.clone();
        let pinned_clone = pinned_threads.clone();
        let presence_clone = presence_rx.clone();
        let pending_clone = pending_messages.clone();

        // Spawn background task to process events
        tokio::spawn(async move {
            Self::event_loop(
                event_rx,
                tx_clone,
                app_clone,
                auth_clone,
                storage_clone,
                unread_clone,
                active_clone,
                typing_clone,
                pinned_clone,
                presence_clone,
                pending_clone,
            )
            .await;
        });

        Self {
            auth,
            app,
            storage,
            tx,
            rx,
            unread_counts,
            active_thread,
            typing_state,
            pinned_threads,
            presence_rx,
            pending_messages,
        }
    }

    /// Background event loop that processes message events and updates the watch channel.
    ///
    /// This runs continuously, refreshing the thread list whenever a relevant event occurs.
    /// For MessageReceived events, it also increments the unread count if the message
    /// is not in the currently active thread.
    /// For TypingIndicatorReceived events, it updates the typing state for the thread.
    #[allow(clippy::too_many_arguments)]
    async fn event_loop(
        mut event_rx: broadcast::Receiver<Event>,
        tx: watch::Sender<MessagingSnapshot>,
        app: Arc<CommunitasApp>,
        auth: Arc<AuthController>,
        storage: Arc<UiStorage>,
        unread_counts: Arc<RwLock<UnreadCounts>>,
        active_thread: Arc<RwLock<Option<String>>>,
        typing_state: Arc<RwLock<TypingState>>,
        pinned_threads: Arc<RwLock<PinnedThreads>>,
        presence_rx: watch::Receiver<PresenceSnapshot>,
        pending_messages: Arc<RwLock<PendingMessageQueue>>,
    ) {
        loop {
            match event_rx.recv().await {
                Ok(event) => {
                    // Handle unread count increment for received messages
                    if let Event::MessageReceived { entity_id, .. } = &event {
                        let active = active_thread.read().ok().and_then(|guard| (*guard).clone());
                        let is_active = active.as_ref().is_some_and(|t| t == entity_id);

                        if !is_active {
                            // Increment unread count for non-active thread
                            if let Ok(mut counts) = unread_counts.write() {
                                counts.increment(entity_id);
                            }
                            // Persist to disk (best effort)
                            if let Ok(counts) = unread_counts.read() {
                                let counts_snapshot = (*counts).clone();
                                if let Err(e) =
                                    JsonFile::save(&storage.unread_counts_file(), &counts_snapshot)
                                {
                                    warn!(error = %e, "Failed to persist unread counts");
                                }
                            }
                        }
                    }

                    // Handle typing indicator updates
                    if let Event::TypingIndicatorReceived {
                        thread_id,
                        peer_id,
                        is_typing,
                    } = &event
                    {
                        if let Ok(mut state) = typing_state.write() {
                            state.update(thread_id, peer_id, *is_typing);
                            // Cleanup expired indicators while we have the lock
                            state.cleanup_expired();
                        }
                        trace!(thread_id, peer_id, is_typing, "Typing indicator updated");
                    }

                    let should_refresh = matches!(
                        event,
                        Event::MessageSent { .. }
                            | Event::MessageReceived { .. }
                            | Event::MessageDeleted { .. }
                            | Event::MessageEdited { .. }
                            | Event::ReactionAdded { .. }
                            | Event::ReactionRemoved { .. }
                            | Event::TypingIndicatorReceived { .. }
                    );

                    if should_refresh {
                        trace!(?event, "Message event received, refreshing threads");
                        Self::refresh_threads_internal(
                            &tx,
                            &app,
                            &auth,
                            &unread_counts,
                            &typing_state,
                            &pinned_threads,
                            &presence_rx,
                            &pending_messages,
                        )
                        .await;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    // We missed some events, refresh to catch up
                    debug!(
                        missed_events = n,
                        "Event receiver lagged, refreshing threads"
                    );
                    Self::refresh_threads_internal(
                        &tx,
                        &app,
                        &auth,
                        &unread_counts,
                        &typing_state,
                        &pinned_threads,
                        &presence_rx,
                        &pending_messages,
                    )
                    .await;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    // Channel closed, stop the loop
                    debug!("Event channel closed, stopping event loop");
                    break;
                }
            }
        }
    }

    /// Internal helper to refresh threads and update the watch channel.
    ///
    /// Used by both the event loop and explicit refresh calls.
    #[allow(clippy::too_many_arguments)]
    async fn refresh_threads_internal(
        tx: &watch::Sender<MessagingSnapshot>,
        app: &Arc<CommunitasApp>,
        auth: &Arc<AuthController>,
        unread_counts: &Arc<RwLock<UnreadCounts>>,
        typing_state: &Arc<RwLock<TypingState>>,
        pinned_threads: &Arc<RwLock<PinnedThreads>>,
        presence_rx: &watch::Receiver<PresenceSnapshot>,
        pending_messages: &Arc<RwLock<PendingMessageQueue>>,
    ) {
        let is_authenticated = matches!(
            &*auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated { .. }
        );

        if !is_authenticated {
            trace!("Not authenticated, skipping thread refresh");
            return;
        }

        let counts = unread_counts
            .read()
            .ok()
            .map(|guard| (*guard).clone())
            .unwrap_or_default();

        let typing = typing_state
            .read()
            .ok()
            .map(|guard| (*guard).clone())
            .unwrap_or_default();

        let pinned = pinned_threads
            .read()
            .ok()
            .map(|guard| (*guard).clone())
            .unwrap_or_default();

        let presence = presence_rx.borrow().clone();

        let pending = pending_messages
            .read()
            .ok()
            .map(|guard| guard.messages.clone())
            .unwrap_or_default();

        let Some(threads) = Self::fetch_threads(app, &counts, &typing, &pinned, &presence).await
        else {
            trace!("Failed to fetch threads in refresh");
            return;
        };

        let _ = tx.send(MessagingSnapshot {
            threads,
            loading: false,
            pending_messages: pending,
        });

        trace!("Thread list refreshed via event");
    }

    /// Fetch and build thread summaries from the core app.
    ///
    /// Returns `None` if the query fails, allowing callers to handle errors appropriately.
    /// Threads are sorted with pinned threads first, then by last message timestamp.
    async fn fetch_threads(
        app: &Arc<CommunitasApp>,
        unread_counts: &UnreadCounts,
        typing_state: &TypingState,
        pinned_threads: &PinnedThreads,
        presence_snapshot: &PresenceSnapshot,
    ) -> Option<Vec<ThreadSummary>> {
        let entities = match app.query(Query::ListEntities).await {
            Ok(QueryResponse::EntityList(entities)) => entities,
            Ok(_) => return None,
            Err(_) => return None,
        };

        let mut threads = Vec::with_capacity(entities.len());
        for entity in entities {
            let default_timestamp = entity.created_at.max(0) as u64;

            let (preview, timestamp) = match app
                .query(Query::GetEntityMessages {
                    entity_id: entity.id.clone(),
                })
                .await
            {
                Ok(QueryResponse::Messages(messages)) => {
                    if let Some(msg) = messages.last() {
                        (
                            truncate_preview(&msg.text, 100),
                            msg.timestamp.max(0) as u64,
                        )
                    } else {
                        (String::new(), default_timestamp)
                    }
                }
                _ => (String::new(), default_timestamp),
            };

            let thread_id = entity.id.clone();
            // Determine sync state based on pending messages
            let has_pending = false; // TODO: Wire to CRDT sync metadata when available
            let sync_state = if has_pending {
                SyncState::Queued
            } else {
                SyncState::Synced
            };
            threads.push(ThreadSummary {
                thread_id: thread_id.clone(),
                entity_id: Some(entity.id),
                entity_type: Some(core_entity_type_to_ui(&entity.entity_type)),
                contact_id: None,
                display_name: entity.name,
                last_message_preview: preview,
                last_message_timestamp: timestamp,
                unread_count: unread_counts.get(&thread_id),
                is_muted: false,
                is_dm: false,
                typing_users: typing_state.get_typing_users(&thread_id),
                is_pinned: pinned_threads.is_pinned(&thread_id),
                contact_presence: None,
                sync_state,
            });
        }

        // Also fetch DM threads from contacts
        if let Ok(QueryResponse::ContactList(contacts)) = app.query(Query::ListContacts).await {
            for contact in contacts {
                // Use the contact's four_words as the contact_id for DMs
                let contact_id = contact
                    .four_words
                    .clone()
                    .unwrap_or_else(|| contact.id.clone());

                // Query DM messages with this contact to get preview and timestamp
                let (preview, timestamp) = match app
                    .query(Query::GetDirectMessages {
                        other_peer_id: contact_id.clone(),
                    })
                    .await
                {
                    Ok(QueryResponse::Messages(messages)) => {
                        if let Some(msg) = messages.last() {
                            (
                                truncate_preview(&msg.text, 100),
                                msg.timestamp.max(0) as u64,
                            )
                        } else {
                            // No DM history with this contact - skip
                            continue;
                        }
                    }
                    _ => {
                        // No DMs or query failed - skip
                        continue;
                    }
                };

                let thread_id = format!("dm:{contact_id}");
                // Look up presence status for this contact
                let contact_presence = presence_snapshot.statuses.get(&contact_id).copied();
                // Determine sync state for DM thread
                let sync_state = SyncState::Synced; // TODO: Wire to CRDT sync metadata when available
                threads.push(ThreadSummary {
                    thread_id: thread_id.clone(),
                    entity_id: None,
                    entity_type: None,
                    contact_id: Some(contact_id),
                    display_name: contact.display_name,
                    last_message_preview: preview,
                    last_message_timestamp: timestamp,
                    unread_count: unread_counts.get(&thread_id),
                    is_muted: false,
                    is_dm: true,
                    typing_users: typing_state.get_typing_users(&thread_id),
                    is_pinned: pinned_threads.is_pinned(&thread_id),
                    contact_presence,
                    sync_state,
                });
            }
        }

        // Sort: pinned threads first, then by last message timestamp (newest first)
        threads.sort_by(|a, b| match (a.is_pinned, b.is_pinned) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.last_message_timestamp.cmp(&a.last_message_timestamp),
        });

        Some(threads)
    }

    /// Get a reference to the core app.
    pub fn app(&self) -> &Arc<CommunitasApp> {
        &self.app
    }

    /// Subscribe to messaging state updates.
    pub fn subscribe(&self) -> watch::Receiver<MessagingSnapshot> {
        self.rx.clone()
    }

    /// Get the current messaging snapshot without subscribing.
    pub fn current_snapshot(&self) -> MessagingSnapshot {
        self.rx.borrow().clone()
    }

    /// Manually refresh the thread list from the core app.
    ///
    /// This is typically not needed as the service automatically refreshes
    /// when message events are received. Use this for explicit user-triggered
    /// refreshes or to ensure the latest data after authentication changes.
    pub async fn refresh_threads(&self) {
        Self::refresh_threads_internal(
            &self.tx,
            &self.app,
            &self.auth,
            &self.unread_counts,
            &self.typing_state,
            &self.pinned_threads,
            &self.presence_rx,
            &self.pending_messages,
        )
        .await;
    }

    /// List all conversation threads for the current user.
    ///
    /// Each thread corresponds to an entity (channel, group, project, etc.) that
    /// the user has joined. The thread includes a preview of the latest message.
    /// Pinned threads appear first in the list.
    ///
    /// # Errors
    /// Returns [`MessagingError::NotAuthenticated`] if no user is logged in.
    #[instrument(skip(self), name = "ui.messaging.list_threads")]
    pub async fn list_threads(&self) -> Result<Vec<ThreadSummary>, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        self.set_loading(true);

        let counts = self
            .unread_counts
            .read()
            .ok()
            .map(|guard| (*guard).clone())
            .unwrap_or_default();

        let typing = self
            .typing_state
            .read()
            .ok()
            .map(|guard| (*guard).clone())
            .unwrap_or_default();

        let pinned = self
            .pinned_threads
            .read()
            .ok()
            .map(|guard| (*guard).clone())
            .unwrap_or_default();

        let presence = self.presence_rx.borrow().clone();

        let threads = Self::fetch_threads(&self.app, &counts, &typing, &pinned, &presence)
            .await
            .unwrap_or_default();

        self.set_threads(threads.clone());

        debug!(thread_count = threads.len(), "Returning threads");
        Ok(threads)
    }

    /// Get messages for a thread with pagination.
    ///
    /// Messages are returned sorted by timestamp descending (newest first).
    ///
    /// # Arguments
    /// * `thread_id` - The entity ID of the thread
    /// * `limit` - Maximum number of messages to return
    /// * `before` - Optional cursor: only return messages with timestamp < before
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::ThreadNotFound`] if the thread does not exist.
    #[instrument(
        skip(self),
        name = "ui.messaging.get_messages",
        fields(thread_id, limit, before)
    )]
    pub async fn get_messages(
        &self,
        thread_id: &str,
        limit: usize,
        before: Option<u64>,
    ) -> Result<Vec<Message>, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        // Query messages for the entity.
        // Note: The core API doesn't distinguish "entity not found" from other query failures,
        // so we map all errors to ThreadNotFound. An unexpected response type (Ok(_)) indicates
        // a programming error or API mismatch and is logged at warn level.
        let messages = match self
            .app
            .query(Query::GetEntityMessages {
                entity_id: thread_id.to_string(),
            })
            .await
        {
            Ok(QueryResponse::Messages(msgs)) => msgs,
            Ok(_) => {
                warn!(thread_id, "Unexpected response type for GetEntityMessages");
                return Err(MessagingError::ThreadNotFound(thread_id.to_string()));
            }
            Err(e) => {
                debug!(thread_id, error = %e.message, "Thread not found or query failed");
                return Err(MessagingError::ThreadNotFound(thread_id.to_string()));
            }
        };

        // Convert to UI types
        let mut ui_messages: Vec<Message> = messages.iter().map(core_message_to_ui).collect();

        // Apply pagination (filter, sort, truncate)
        apply_pagination(&mut ui_messages, limit, before);

        debug!(
            thread_id,
            message_count = ui_messages.len(),
            "Returning messages for thread"
        );

        Ok(ui_messages)
    }

    /// Search messages across all threads or within a specific thread.
    ///
    /// Returns messages matching the query with context about where they were found.
    /// Results are sorted by match relevance (match count) and recency.
    ///
    /// # Arguments
    /// * `query` - The search string to match against message content.
    /// * `thread_id` - Optional thread ID to limit search scope. If None, searches all threads.
    /// * `limit` - Maximum number of results to return.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the search query fails.
    #[instrument(
        skip(self),
        name = "ui.messaging.search",
        fields(query, thread_id, limit)
    )]
    pub async fn search_messages(
        &self,
        query: &str,
        thread_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        // Execute search query
        let results = match self
            .app
            .query(Query::SearchMessages {
                query: query.to_string(),
                thread_id: thread_id.map(|s| s.to_string()),
                limit,
            })
            .await
        {
            Ok(QueryResponse::SearchResults(results)) => results,
            Ok(_) => {
                warn!(query, "Unexpected response type for SearchMessages");
                return Err(MessagingError::Internal(
                    "Unexpected search response".to_string(),
                ));
            }
            Err(e) => {
                debug!(query, error = %e.message, "Search query failed");
                return Err(MessagingError::Internal(format!(
                    "Search failed: {}",
                    e.message
                )));
            }
        };

        // Convert to UI types
        let ui_results: Vec<SearchResult> = results.iter().map(core_search_result_to_ui).collect();

        debug!(query, result_count = ui_results.len(), "Search completed");

        Ok(ui_results)
    }

    /// Send a message to a thread.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::SendFailed`] if the message could not be delivered.
    /// - [`MessagingError::Internal`] for other failures.
    #[instrument(skip(self, text), name = "ui.messaging.send", fields(thread_id, has_reply = reply_to.is_some()))]
    pub async fn send_message(
        &self,
        thread_id: &str,
        text: &str,
        reply_to: Option<&str>,
    ) -> Result<Message, MessagingError> {
        // Get the authenticated user's display name for the author field
        let author = match &*self.auth.subscribe().borrow() {
            AuthStateSnapshot::Authenticated { session, .. } => session.display_name.clone(),
            _ => return Err(MessagingError::NotAuthenticated),
        };

        let entity_type = self.resolve_entity_type(thread_id).await;

        // Build and execute the SendMessage command
        let cmd = Command::SendMessage {
            entity_id: thread_id.to_string(),
            entity_type,
            text: text.to_string(),
            author,
            reply_to_id: reply_to.map(|s| s.to_string()),
            attachments: None,
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| MessagingError::SendFailed(e.message))?;

        // Extract message_id from the MessageSent event
        let message_id = events
            .iter()
            .find_map(|event| match event {
                Event::MessageSent { message_id, .. } => Some(message_id.clone()),
                _ => None,
            })
            .ok_or_else(|| {
                MessagingError::SendFailed("No MessageSent event returned".to_string())
            })?;

        debug!(thread_id, message_id = %message_id, "Message sent successfully");

        // Query the newly created message to return the full Message struct
        let message = match self
            .app
            .query(Query::GetMessage {
                entity_id: thread_id.to_string(),
                message_id: message_id.clone(),
            })
            .await
        {
            Ok(QueryResponse::Message(msg)) => core_message_to_ui(&msg),
            Ok(_) => {
                warn!(message_id, "Unexpected response type for GetMessage");
                return Err(MessagingError::Internal(
                    "Failed to retrieve sent message".to_string(),
                ));
            }
            Err(e) => {
                warn!(message_id, error = %e.message, "Failed to retrieve sent message");
                return Err(MessagingError::Internal(format!(
                    "Message sent but retrieval failed: {}",
                    e.message
                )));
            }
        };

        Ok(message)
    }

    /// Edit an existing message.
    ///
    /// # Arguments
    /// * `thread_id` - The entity ID of the thread containing the message.
    /// * `message_id` - The ID of the message to edit.
    /// * `new_text` - The new text content to replace the existing message.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the edit command fails (e.g., message not found,
    ///   permission denied, or no confirmation event received).
    #[instrument(
        skip(self, new_text),
        name = "ui.messaging.edit",
        fields(thread_id, message_id)
    )]
    pub async fn edit_message(
        &self,
        thread_id: &str,
        message_id: &str,
        new_text: &str,
    ) -> Result<Message, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        let entity_type = self.resolve_entity_type(thread_id).await;

        // Build and execute the EditMessage command
        let cmd = Command::EditMessage {
            entity_id: thread_id.to_string(),
            entity_type,
            message_id: message_id.to_string(),
            new_text: new_text.to_string(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| MessagingError::Internal(format!("Edit failed: {}", e.message)))?;

        // Verify the MessageEdited event was returned
        let edited = events.iter().any(|event| {
            matches!(event, Event::MessageEdited { message_id: mid, .. } if mid == message_id)
        });

        if !edited {
            return Err(MessagingError::Internal(
                "No MessageEdited event returned".to_string(),
            ));
        }

        debug!(thread_id, message_id, "Message edited successfully");

        // Query the updated message to return the full Message struct
        match self
            .app
            .query(Query::GetMessage {
                entity_id: thread_id.to_string(),
                message_id: message_id.to_string(),
            })
            .await
        {
            Ok(QueryResponse::Message(msg)) => Ok(core_message_to_ui(&msg)),
            Ok(_) => {
                warn!(message_id, "Unexpected response type for GetMessage");
                Err(MessagingError::Internal(
                    "Failed to retrieve edited message".to_string(),
                ))
            }
            Err(e) => {
                warn!(message_id, error = %e.message, "Failed to retrieve edited message");
                Err(MessagingError::Internal(format!(
                    "Message edited but retrieval failed: {}",
                    e.message
                )))
            }
        }
    }

    /// Delete a message.
    ///
    /// # Arguments
    /// * `thread_id` - The entity ID of the thread containing the message.
    /// * `message_id` - The ID of the message to delete.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the delete command fails (e.g., message not found,
    ///   permission denied, or no confirmation event received).
    #[instrument(
        skip(self),
        name = "ui.messaging.delete",
        fields(thread_id, message_id)
    )]
    pub async fn delete_message(
        &self,
        thread_id: &str,
        message_id: &str,
    ) -> Result<(), MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        let entity_type = self.resolve_entity_type(thread_id).await;

        // Build and execute the DeleteMessage command
        let cmd = Command::DeleteMessage {
            entity_id: thread_id.to_string(),
            entity_type,
            message_id: message_id.to_string(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| MessagingError::Internal(format!("Delete failed: {}", e.message)))?;

        // Verify the MessageDeleted event was returned
        let deleted = events.iter().any(|event| {
            matches!(event, Event::MessageDeleted { message_id: mid, .. } if mid == message_id)
        });

        if !deleted {
            return Err(MessagingError::Internal(
                "No MessageDeleted event returned".to_string(),
            ));
        }

        debug!(thread_id, message_id, "Message deleted successfully");
        Ok(())
    }

    /// Mark a thread as read, clearing unread count.
    ///
    /// This updates both the in-memory state, persists to disk, and broadcasts
    /// a `ThreadMarkedRead` event via the core command system.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::ThreadNotFound`] if the thread does not exist.
    #[instrument(skip(self), name = "ui.messaging.mark_read", fields(thread_id))]
    pub async fn mark_thread_read(&self, thread_id: &str) -> Result<(), MessagingError> {
        // Get current user's identity for the command
        let identity = match &*self.auth.subscribe().borrow() {
            AuthStateSnapshot::Authenticated { session, .. } => session.four_words.clone(),
            _ => return Err(MessagingError::NotAuthenticated),
        };

        // Update local thread state
        let mut snap = self.rx.borrow().clone();
        if let Some(thread) = snap.threads.iter_mut().find(|t| t.thread_id == thread_id) {
            thread.unread_count = 0;
            // Send cannot fail: self.rx guarantees at least one receiver exists
            let _ = self.tx.send(snap);

            // Reset persisted unread count
            if let Ok(mut counts) = self.unread_counts.write() {
                counts.reset(thread_id);
            }

            // Persist to disk (best effort)
            if let Ok(counts) = self.unread_counts.read() {
                let counts_snapshot = (*counts).clone();
                if let Err(e) = JsonFile::save(&self.storage.unread_counts_file(), &counts_snapshot)
                {
                    warn!(error = %e, "Failed to persist unread counts after mark_read");
                }
            }

            // Execute core command to broadcast the event
            let cmd = Command::MarkThreadRead {
                thread_id: thread_id.to_string(),
                identity,
            };
            if let Err(e) = self.app.execute(cmd).await {
                warn!(error = %e.message, "Failed to execute MarkThreadRead command");
            }

            debug!(thread_id, "Thread marked as read");
            Ok(())
        } else {
            Err(MessagingError::ThreadNotFound(thread_id.to_string()))
        }
    }

    /// Set the currently active thread.
    ///
    /// Messages received in the active thread will not increment unread counts.
    /// Call with `None` when leaving a thread view.
    pub fn set_active_thread(&self, thread_id: Option<String>) {
        if let Ok(mut guard) = self.active_thread.write() {
            *guard = thread_id;
        }
    }

    /// Get the currently active thread ID, if any.
    pub fn get_active_thread(&self) -> Option<String> {
        self.active_thread
            .read()
            .ok()
            .and_then(|guard| (*guard).clone())
    }

    /// Pin a thread to the top of the thread list.
    ///
    /// Pinned threads appear at the top of the thread list, sorted by most
    /// recently pinned. A maximum of 5 threads can be pinned at once.
    ///
    /// # Arguments
    /// * `thread_id` - The ID of the thread to pin.
    ///
    /// # Returns
    /// - `Ok(true)` if the thread was newly pinned.
    /// - `Ok(false)` if the thread was already pinned.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the maximum number of pinned threads
    ///   would be exceeded or if persistence fails.
    #[instrument(skip(self), name = "ui.messaging.pin_thread", fields(thread_id))]
    pub async fn pin_thread(&self, thread_id: &str) -> Result<bool, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        // Update in-memory state
        let was_pinned = {
            let mut pinned = self.pinned_threads.write().map_err(|_| {
                MessagingError::Internal("Failed to acquire pinned threads lock".to_string())
            })?;
            match pinned.pin(thread_id) {
                Ok(newly_pinned) => newly_pinned,
                Err(e) => return Err(MessagingError::Internal(e.to_string())),
            }
        };

        if was_pinned {
            // Persist to disk
            if let Ok(pinned) = self.pinned_threads.read() {
                let pinned_snapshot = (*pinned).clone();
                if let Err(e) =
                    JsonFile::save(&self.storage.pinned_threads_file(), &pinned_snapshot)
                {
                    warn!(error = %e, "Failed to persist pinned threads");
                }
            }

            // Refresh thread list to reflect new pin state
            self.refresh_threads().await;

            debug!(thread_id, "Thread pinned successfully");
        }

        Ok(was_pinned)
    }

    /// Unpin a thread from the top of the thread list.
    ///
    /// # Arguments
    /// * `thread_id` - The ID of the thread to unpin.
    ///
    /// # Returns
    /// - `Ok(true)` if the thread was unpinned.
    /// - `Ok(false)` if the thread wasn't pinned.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if persistence fails.
    #[instrument(skip(self), name = "ui.messaging.unpin_thread", fields(thread_id))]
    pub async fn unpin_thread(&self, thread_id: &str) -> Result<bool, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        // Update in-memory state
        let was_unpinned = {
            let mut pinned = self.pinned_threads.write().map_err(|_| {
                MessagingError::Internal("Failed to acquire pinned threads lock".to_string())
            })?;
            pinned.unpin(thread_id)
        };

        if was_unpinned {
            // Persist to disk
            if let Ok(pinned) = self.pinned_threads.read() {
                let pinned_snapshot = (*pinned).clone();
                if let Err(e) =
                    JsonFile::save(&self.storage.pinned_threads_file(), &pinned_snapshot)
                {
                    warn!(error = %e, "Failed to persist pinned threads");
                }
            }

            // Refresh thread list to reflect new pin state
            self.refresh_threads().await;

            debug!(thread_id, "Thread unpinned successfully");
        }

        Ok(was_unpinned)
    }

    /// Check if a thread is pinned.
    ///
    /// # Arguments
    /// * `thread_id` - The ID of the thread to check.
    ///
    /// # Returns
    /// `true` if the thread is pinned, `false` otherwise.
    pub fn is_thread_pinned(&self, thread_id: &str) -> bool {
        self.pinned_threads
            .read()
            .ok()
            .map(|pinned| pinned.is_pinned(thread_id))
            .unwrap_or(false)
    }

    /// Get the list of pinned thread IDs.
    pub fn get_pinned_threads(&self) -> Vec<String> {
        self.pinned_threads
            .read()
            .ok()
            .map(|pinned| pinned.thread_ids.clone())
            .unwrap_or_default()
    }

    // ==================== Typing Indicator Methods ====================

    /// Get the list of users currently typing in a thread.
    ///
    /// Returns display names of users who have sent typing indicators within
    /// the last TYPING_EXPIRE_DURATION (typically a few seconds).
    pub fn get_typing_users(&self, thread_id: &str) -> Vec<String> {
        self.typing_state
            .read()
            .ok()
            .map(|state| state.get_typing_users(thread_id))
            .unwrap_or_default()
    }

    // ==================== Offline Send Queue Methods ====================

    /// Get the list of pending messages awaiting send.
    pub fn get_pending_messages(&self) -> Vec<PendingMessage> {
        self.pending_messages
            .read()
            .ok()
            .map(|queue| queue.messages.clone())
            .unwrap_or_default()
    }

    /// Queue a message for sending (used when offline or send fails).
    ///
    /// The message will be retried automatically when connectivity is restored.
    /// Messages are persisted to disk for durability across app restarts.
    ///
    /// # Arguments
    /// * `thread_id` - Target thread ID
    /// * `text` - Message content
    /// * `reply_to_id` - Optional ID of message being replied to
    ///
    /// # Returns
    /// The ID of the queued pending message.
    #[instrument(skip(self, text), name = "ui.messaging.queue", fields(thread_id))]
    pub fn queue_message(&self, thread_id: &str, text: &str, reply_to_id: Option<&str>) -> String {
        // Use atomic counter to ensure unique IDs even when called rapidly
        static PENDING_COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = PENDING_COUNTER.fetch_add(1, Ordering::Relaxed);

        let id = format!(
            "pending-{}-{}-{}",
            thread_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            counter
        );

        let pending = PendingMessage {
            id: id.clone(),
            thread_id: thread_id.to_string(),
            text: text.to_string(),
            reply_to_id: reply_to_id.map(|s| s.to_string()),
            queued_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            retry_count: 0,
            status: MessageSendStatus::Pending,
            last_error: None,
        };

        if let Ok(mut queue) = self.pending_messages.write() {
            queue.messages.push(pending);

            // Persist to disk
            if let Err(e) = JsonFile::save(&self.storage.pending_messages_file(), &*queue) {
                warn!(error = %e, "Failed to persist pending messages");
            }
        }

        // Refresh to update snapshot
        self.broadcast_pending_update();

        debug!(id = %id, thread_id, "Message queued for sending");
        id
    }

    /// Remove a pending message from the queue (after successful send or manual cancel).
    ///
    /// # Arguments
    /// * `pending_id` - The ID of the pending message to remove
    ///
    /// # Returns
    /// `true` if the message was removed, `false` if not found.
    #[instrument(skip(self), name = "ui.messaging.remove_pending", fields(pending_id))]
    pub fn remove_pending_message(&self, pending_id: &str) -> bool {
        let removed = if let Ok(mut queue) = self.pending_messages.write() {
            let initial_len = queue.messages.len();
            queue.messages.retain(|m| m.id != pending_id);
            let removed = queue.messages.len() < initial_len;

            if removed {
                // Persist to disk
                if let Err(e) = JsonFile::save(&self.storage.pending_messages_file(), &*queue) {
                    warn!(error = %e, "Failed to persist pending messages");
                }
            }

            removed
        } else {
            false
        };

        if removed {
            self.broadcast_pending_update();
            debug!(pending_id, "Pending message removed");
        }

        removed
    }

    /// Mark a pending message as failed with an error message.
    ///
    /// This is called after exhausting retry attempts.
    ///
    /// # Arguments
    /// * `pending_id` - The ID of the pending message
    /// * `error` - The error message describing the failure
    #[instrument(skip(self), name = "ui.messaging.mark_failed", fields(pending_id))]
    pub fn mark_pending_failed(&self, pending_id: &str, error: &str) {
        if let Ok(mut queue) = self.pending_messages.write()
            && let Some(msg) = queue.messages.iter_mut().find(|m| m.id == pending_id)
        {
            msg.status = MessageSendStatus::Failed(error.to_string());
            msg.last_error = Some(error.to_string());

            // Persist to disk
            if let Err(e) = JsonFile::save(&self.storage.pending_messages_file(), &*queue) {
                warn!(error = %e, "Failed to persist pending messages");
            }
        }

        self.broadcast_pending_update();
        debug!(pending_id, error, "Pending message marked as failed");
    }

    /// Retry sending a pending message.
    ///
    /// This increments the retry count and attempts to send again.
    /// If max retries are exceeded, the message is marked as failed.
    ///
    /// # Arguments
    /// * `pending_id` - The ID of the pending message to retry
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the message is not found or send fails.
    #[instrument(skip(self), name = "ui.messaging.retry_pending", fields(pending_id))]
    pub async fn retry_pending_message(&self, pending_id: &str) -> Result<Message, MessagingError> {
        // Get the pending message details
        let pending = {
            let queue = self.pending_messages.read().map_err(|_| {
                MessagingError::Internal("Failed to acquire pending messages lock".to_string())
            })?;
            queue
                .messages
                .iter()
                .find(|m| m.id == pending_id)
                .cloned()
                .ok_or_else(|| {
                    MessagingError::Internal(format!("Pending message not found: {pending_id}"))
                })?
        };

        // Check retry count
        if pending.retry_count >= MAX_PENDING_RETRIES {
            self.mark_pending_failed(
                pending_id,
                &format!("Max retries ({MAX_PENDING_RETRIES}) exceeded"),
            );
            return Err(MessagingError::SendFailed(format!(
                "Max retries ({MAX_PENDING_RETRIES}) exceeded"
            )));
        }

        // Update retry count and status
        if let Ok(mut queue) = self.pending_messages.write()
            && let Some(msg) = queue.messages.iter_mut().find(|m| m.id == pending_id)
        {
            msg.retry_count += 1;
            msg.status = MessageSendStatus::Sending;

            if let Err(e) = JsonFile::save(&self.storage.pending_messages_file(), &*queue) {
                warn!(error = %e, "Failed to persist pending messages");
            }
        }

        self.broadcast_pending_update();

        // Attempt to send
        match self
            .send_message(
                &pending.thread_id,
                &pending.text,
                pending.reply_to_id.as_deref(),
            )
            .await
        {
            Ok(message) => {
                // Success - remove from queue
                self.remove_pending_message(pending_id);
                debug!(pending_id, message_id = %message.id, "Pending message sent successfully");
                Ok(message)
            }
            Err(e) => {
                // Failed - update status back to pending or failed
                let retry_count = {
                    self.pending_messages
                        .read()
                        .ok()
                        .and_then(|q| {
                            q.messages
                                .iter()
                                .find(|m| m.id == pending_id)
                                .map(|m| m.retry_count)
                        })
                        .unwrap_or(0)
                };

                if retry_count >= MAX_PENDING_RETRIES {
                    self.mark_pending_failed(pending_id, &e.to_string());
                } else {
                    // Revert to pending status
                    if let Ok(mut queue) = self.pending_messages.write()
                        && let Some(msg) = queue.messages.iter_mut().find(|m| m.id == pending_id)
                    {
                        msg.status = MessageSendStatus::Pending;
                        msg.last_error = Some(e.to_string());

                        if let Err(e) =
                            JsonFile::save(&self.storage.pending_messages_file(), &*queue)
                        {
                            warn!(error = %e, "Failed to persist pending messages");
                        }
                    }
                    self.broadcast_pending_update();
                }

                debug!(pending_id, error = %e, "Pending message retry failed");
                Err(e)
            }
        }
    }

    /// Retry all pending messages that haven't exceeded max retries.
    ///
    /// This is typically called when connectivity is restored.
    ///
    /// # Returns
    /// A list of (pending_id, result) pairs indicating success or failure for each retry.
    #[instrument(skip(self), name = "ui.messaging.retry_all_pending")]
    pub async fn retry_all_pending(&self) -> Vec<(String, Result<Message, MessagingError>)> {
        let pending_ids: Vec<String> = self
            .get_pending_messages()
            .into_iter()
            .filter(|m| !m.status.is_failed() && m.retry_count < MAX_PENDING_RETRIES)
            .map(|m| m.id)
            .collect();

        let mut results = Vec::with_capacity(pending_ids.len());

        for pending_id in pending_ids {
            let result = self.retry_pending_message(&pending_id).await;
            results.push((pending_id, result));
        }

        debug!(
            retried_count = results.len(),
            "Retried all pending messages"
        );
        results
    }

    /// Internal helper to broadcast pending message updates to subscribers.
    fn broadcast_pending_update(&self) {
        let pending = self
            .pending_messages
            .read()
            .ok()
            .map(|queue| queue.messages.clone())
            .unwrap_or_default();

        let mut snap = self.rx.borrow().clone();
        snap.pending_messages = pending;
        let _ = self.tx.send(snap);
    }

    /// Send a typing indicator for the specified thread.
    ///
    /// This broadcasts a typing indicator event to other participants
    /// in the thread. Typing indicators automatically expire after 3 seconds
    /// on the receiving end.
    ///
    /// # Arguments
    /// * `thread_id` - The thread where the user is typing
    /// * `is_typing` - Whether the user is currently typing
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the command execution fails.
    #[instrument(skip(self), name = "ui.messaging.typing", fields(thread_id, is_typing))]
    pub async fn send_typing_indicator(
        &self,
        thread_id: &str,
        is_typing: bool,
    ) -> Result<(), MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        let cmd = Command::SendTypingIndicator {
            thread_id: thread_id.to_string(),
            is_typing,
        };

        self.app.execute(cmd).await.map_err(|e| {
            MessagingError::Internal(format!("Send typing indicator failed: {}", e.message))
        })?;

        debug!(thread_id, is_typing, "Typing indicator sent");
        Ok(())
    }

    /// Get unread count for a specific thread.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::ThreadNotFound`] if the thread does not exist.
    #[instrument(skip(self), name = "ui.messaging.get_unread", fields(thread_id))]
    pub async fn get_thread_unread_count(&self, thread_id: &str) -> Result<u32, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        let snap = self.rx.borrow();
        if let Some(thread) = snap.threads.iter().find(|t| t.thread_id == thread_id) {
            Ok(thread.unread_count)
        } else {
            Err(MessagingError::ThreadNotFound(thread_id.to_string()))
        }
    }

    /// Add a reaction to a message.
    ///
    /// # Arguments
    /// * `thread_id` - The entity ID of the thread containing the message.
    /// * `message_id` - The ID of the message to react to.
    /// * `emoji` - The emoji to add as a reaction (e.g., "👍", "❤️").
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the reaction fails (e.g., message not found,
    ///   invalid emoji, or no confirmation event received).
    #[instrument(
        skip(self),
        name = "ui.messaging.react",
        fields(thread_id, message_id, emoji)
    )]
    pub async fn add_reaction(
        &self,
        thread_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        let entity_type = self.resolve_entity_type(thread_id).await;

        // Build and execute the AddReaction command
        let cmd = Command::AddReaction {
            entity_id: thread_id.to_string(),
            entity_type,
            message_id: message_id.to_string(),
            emoji: emoji.to_string(),
        };

        let events =
            self.app.execute(cmd).await.map_err(|e| {
                MessagingError::Internal(format!("Add reaction failed: {}", e.message))
            })?;

        // Verify the ReactionAdded event was returned
        let added = events.iter().any(|event| {
            matches!(event, Event::ReactionAdded { message_id: mid, emoji: e, .. } if mid == message_id && e == emoji)
        });

        if !added {
            return Err(MessagingError::Internal(
                "No ReactionAdded event returned".to_string(),
            ));
        }

        debug!(thread_id, message_id, emoji, "Reaction added successfully");
        Ok(())
    }

    /// Remove a reaction from a message.
    ///
    /// # Arguments
    /// * `thread_id` - The entity ID of the thread containing the message.
    /// * `message_id` - The ID of the message to remove the reaction from.
    /// * `emoji` - The emoji to remove (e.g., "👍", "❤️").
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the removal fails (e.g., message not found,
    ///   reaction not present, or no confirmation event received).
    #[instrument(
        skip(self),
        name = "ui.messaging.unreact",
        fields(thread_id, message_id, emoji)
    )]
    pub async fn remove_reaction(
        &self,
        thread_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        let entity_type = self.resolve_entity_type(thread_id).await;

        // Build and execute the RemoveReaction command
        let cmd = Command::RemoveReaction {
            entity_id: thread_id.to_string(),
            entity_type,
            message_id: message_id.to_string(),
            emoji: emoji.to_string(),
        };

        let events = self.app.execute(cmd).await.map_err(|e| {
            MessagingError::Internal(format!("Remove reaction failed: {}", e.message))
        })?;

        // Verify the ReactionRemoved event was returned
        let removed = events.iter().any(|event| {
            matches!(event, Event::ReactionRemoved { message_id: mid, emoji: e, .. } if mid == message_id && e == emoji)
        });

        if !removed {
            return Err(MessagingError::Internal(
                "No ReactionRemoved event returned".to_string(),
            ));
        }

        debug!(
            thread_id,
            message_id, emoji, "Reaction removed successfully"
        );
        Ok(())
    }

    /// Internal: update the thread list (called by core events).
    pub fn set_threads(&self, threads: Vec<ThreadSummary>) {
        let mut snap = self.rx.borrow().clone();
        snap.threads = threads;
        snap.loading = false;
        // Send cannot fail: self.rx guarantees at least one receiver exists
        let _ = self.tx.send(snap);
    }

    /// Internal: set loading state.
    pub fn set_loading(&self, loading: bool) {
        let mut snap = self.rx.borrow().clone();
        snap.loading = loading;
        // Send cannot fail: self.rx guarantees at least one receiver exists
        let _ = self.tx.send(snap);
    }

    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated { .. }
        )
    }

    /// Resolve the entity type for a thread, defaulting to Channel if unavailable.
    ///
    /// This is used by message operations that need the entity type for commands.
    async fn resolve_entity_type(&self, thread_id: &str) -> EntityType {
        match self
            .app
            .query(Query::GetEntity {
                entity_id: thread_id.to_string(),
            })
            .await
        {
            Ok(QueryResponse::Entity(entity)) => entity.entity_type,
            Ok(_) => {
                warn!(thread_id, "Unexpected response type for GetEntity");
                EntityType::Channel
            }
            Err(e) => {
                debug!(thread_id, error = %e.message, "Could not determine entity type, using Channel");
                EntityType::Channel
            }
        }
    }
}

/// Apply pagination to a list of messages: filter by cursor, sort descending, and truncate.
///
/// This is extracted for testability. The function:
/// 1. Filters messages with `timestamp < before` if a cursor is provided
/// 2. Sorts by timestamp descending (newest first)
/// 3. Truncates to the specified limit
fn apply_pagination(messages: &mut Vec<Message>, limit: usize, before: Option<u64>) {
    // Filter by before cursor
    if let Some(before_ts) = before {
        messages.retain(|m| m.timestamp < before_ts);
    }

    // Sort by timestamp descending (newest first)
    messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    messages.truncate(limit);
}

/// Truncate text to max length, adding ellipsis if truncated.
fn truncate_preview(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_len.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use communitas_ui_api::UnifiedEntityType;
    use tempfile::TempDir;

    async fn make_service(temp: &TempDir) -> MessagingService {
        let storage = Arc::new(UiStorage::from_path(temp.path()).unwrap());
        let auth = Arc::new(AuthController::new((*storage).clone()).unwrap());
        let app = Arc::new(
            CommunitasApp::new(
                "ocean-forest-moon-star".to_string(),
                "TestUser".to_string(),
                "TestDevice".to_string(),
                temp.path()
                    .join("app_storage")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            .unwrap(),
        );
        // Create a dummy presence receiver for tests
        let (_presence_tx, presence_rx) = watch::channel(PresenceSnapshot::default());
        MessagingService::new(auth, app, storage, presence_rx)
    }

    #[tokio::test]
    async fn messaging_service_starts_empty() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let snap = service.current_snapshot();
        assert!(snap.threads.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn list_threads_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.list_threads().await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_messages_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_messages("thread1", 50, None).await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn send_message_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.send_message("thread1", "Hello", None).await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn edit_message_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.edit_message("thread1", "msg1", "new text").await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn delete_message_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.delete_message("thread1", "msg1").await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn subscribe_returns_receiver() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let rx = service.subscribe();
        let snap = rx.borrow().clone();
        assert!(snap.threads.is_empty());
    }

    #[tokio::test]
    async fn set_threads_updates_subscribers() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let rx = service.subscribe();

        let threads = vec![ThreadSummary {
            thread_id: "t1".to_string(),
            entity_id: Some("e1".to_string()),
            entity_type: Some(UnifiedEntityType::Channel),
            contact_id: None,
            display_name: "General".to_string(),
            last_message_preview: "Hello".to_string(),
            last_message_timestamp: 1234567890,
            unread_count: 3,
            is_muted: false,
            is_dm: false,
            typing_users: vec![],
            is_pinned: false,
            contact_presence: None,
            sync_state: SyncState::Synced,
        }];

        service.set_threads(threads);

        let snap = rx.borrow().clone();
        assert_eq!(snap.threads.len(), 1);
        assert_eq!(snap.threads[0].thread_id, "t1");
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn set_loading_updates_subscribers() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let rx = service.subscribe();

        service.set_loading(true);
        assert!(rx.borrow().loading);

        service.set_loading(false);
        assert!(!rx.borrow().loading);
    }

    #[test]
    fn truncate_preview_short_text() {
        let text = "Hello";
        assert_eq!(truncate_preview(text, 100), "Hello");
    }

    #[test]
    fn truncate_preview_exact_length() {
        let text = "A".repeat(100);
        assert_eq!(truncate_preview(&text, 100), text);
    }

    #[test]
    fn truncate_preview_long_text() {
        let text = "A".repeat(150);
        let result = truncate_preview(&text, 100);
        assert!(result.ends_with("..."));
        assert_eq!(result.chars().count(), 100);
    }

    #[test]
    fn truncate_preview_unicode() {
        let text = "🎉".repeat(50); // 50 emoji characters
        let result = truncate_preview(&text, 20);
        assert!(result.ends_with("..."));
        // Should have 17 emoji + "..."
        assert_eq!(result.chars().count(), 20);
    }

    // Helper to create test messages with specific timestamps
    fn make_test_message(id: &str, timestamp: u64) -> Message {
        Message {
            id: id.to_string(),
            thread_id: "thread1".to_string(),
            sender_id: "sender1".to_string(),
            sender_name: "Sender".to_string(),
            text: format!("Message {id}"),
            timestamp,
            edited: false,
            reply_to_id: None,
            reactions: vec![],
        }
    }

    #[test]
    fn apply_pagination_empty_input() {
        let mut messages: Vec<Message> = vec![];
        apply_pagination(&mut messages, 10, None);
        assert!(messages.is_empty());
    }

    #[test]
    fn apply_pagination_limit_only() {
        let mut messages = vec![
            make_test_message("1", 100),
            make_test_message("2", 200),
            make_test_message("3", 300),
            make_test_message("4", 400),
            make_test_message("5", 500),
        ];
        apply_pagination(&mut messages, 3, None);

        assert_eq!(messages.len(), 3);
        // Should be sorted descending (newest first) and limited to 3
        assert_eq!(messages[0].id, "5");
        assert_eq!(messages[1].id, "4");
        assert_eq!(messages[2].id, "3");
    }

    #[test]
    fn apply_pagination_before_cursor_only() {
        let mut messages = vec![
            make_test_message("1", 100),
            make_test_message("2", 200),
            make_test_message("3", 300),
            make_test_message("4", 400),
            make_test_message("5", 500),
        ];
        // Only return messages with timestamp < 350
        apply_pagination(&mut messages, 100, Some(350));

        assert_eq!(messages.len(), 3);
        // Messages 1, 2, 3 should remain (timestamps 100, 200, 300)
        // Sorted descending: 3, 2, 1
        assert_eq!(messages[0].id, "3");
        assert_eq!(messages[1].id, "2");
        assert_eq!(messages[2].id, "1");
    }

    #[test]
    fn apply_pagination_limit_and_before() {
        let mut messages = vec![
            make_test_message("1", 100),
            make_test_message("2", 200),
            make_test_message("3", 300),
            make_test_message("4", 400),
            make_test_message("5", 500),
        ];
        // Filter to timestamp < 450, then limit to 2
        apply_pagination(&mut messages, 2, Some(450));

        assert_eq!(messages.len(), 2);
        // Messages 1, 2, 3, 4 pass filter (timestamp < 450)
        // Sorted descending: 4, 3, 2, 1
        // Limited to 2: 4, 3
        assert_eq!(messages[0].id, "4");
        assert_eq!(messages[1].id, "3");
    }

    #[test]
    fn apply_pagination_sorts_descending() {
        // Input in random order
        let mut messages = vec![
            make_test_message("3", 300),
            make_test_message("1", 100),
            make_test_message("5", 500),
            make_test_message("2", 200),
            make_test_message("4", 400),
        ];
        apply_pagination(&mut messages, 100, None);

        // Should be sorted by timestamp descending
        assert_eq!(messages.len(), 5);
        assert_eq!(messages[0].timestamp, 500);
        assert_eq!(messages[1].timestamp, 400);
        assert_eq!(messages[2].timestamp, 300);
        assert_eq!(messages[3].timestamp, 200);
        assert_eq!(messages[4].timestamp, 100);
    }

    #[test]
    fn apply_pagination_before_excludes_exact_match() {
        let mut messages = vec![
            make_test_message("1", 100),
            make_test_message("2", 200),
            make_test_message("3", 300),
        ];
        // timestamp < 200 should exclude message with timestamp 200
        apply_pagination(&mut messages, 100, Some(200));

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "1");
    }

    #[test]
    fn apply_pagination_limit_zero() {
        let mut messages = vec![make_test_message("1", 100), make_test_message("2", 200)];
        apply_pagination(&mut messages, 0, None);
        assert!(messages.is_empty());
    }

    // ==================== Read Operations Tests ====================
    // These tests verify list_threads and get_messages behavior when authenticated.

    /// Helper to create an authenticated messaging service using demo mode.
    async fn make_authenticated_service(temp: &TempDir) -> MessagingService {
        let service = make_service(temp).await;
        service.auth.enable_demo_mode();
        service
    }

    #[tokio::test]
    async fn test_list_threads_empty_for_new_user() {
        // A freshly authenticated user with no joined entities should see empty threads
        let temp = TempDir::new().unwrap();
        let service = make_authenticated_service(&temp).await;

        let threads = service
            .list_threads()
            .await
            .expect("should succeed when authenticated");

        // A new user hasn't joined any entities, so threads should be empty
        assert!(threads.is_empty(), "new user should have no threads");

        // Verify snapshot was also updated
        let snap = service.current_snapshot();
        assert!(snap.threads.is_empty());
        assert!(
            !snap.loading,
            "loading should be false after list_threads completes"
        );
    }

    #[tokio::test]
    async fn test_list_threads_returns_joined_entities() {
        // Test that set_threads properly updates state and is reflected in list output.
        // Since creating real entities through the core API is complex, we test the
        // service's state management by manually setting threads and verifying they
        // persist correctly.
        let temp = TempDir::new().unwrap();
        let service = make_authenticated_service(&temp).await;
        let rx = service.subscribe();

        // Simulate entities being loaded by setting threads directly
        let test_threads = vec![
            ThreadSummary {
                thread_id: "entity-1".to_string(),
                entity_id: Some("entity-1".to_string()),
                entity_type: Some(UnifiedEntityType::Channel),
                contact_id: None,
                display_name: "General".to_string(),
                last_message_preview: "Hello everyone".to_string(),
                last_message_timestamp: 2000,
                unread_count: 5,
                is_muted: false,
                is_dm: false,
                typing_users: vec![],
                is_pinned: false,
                contact_presence: None,
                sync_state: SyncState::Synced,
            },
            ThreadSummary {
                thread_id: "entity-2".to_string(),
                entity_id: Some("entity-2".to_string()),
                entity_type: Some(UnifiedEntityType::Group),
                contact_id: None,
                display_name: "Project Team".to_string(),
                last_message_preview: "Meeting at 3pm".to_string(),
                last_message_timestamp: 3000,
                unread_count: 0,
                is_muted: true,
                is_dm: false,
                typing_users: vec![],
                is_pinned: false,
                contact_presence: None,
                sync_state: SyncState::Syncing,
            },
        ];

        service.set_threads(test_threads.clone());

        // Verify subscription received the update
        let snap = rx.borrow().clone();
        assert_eq!(snap.threads.len(), 2);
        assert_eq!(snap.threads[0].thread_id, "entity-1");
        assert_eq!(snap.threads[1].thread_id, "entity-2");
        assert!(!snap.loading);

        // Verify current_snapshot returns same data
        let current = service.current_snapshot();
        assert_eq!(current.threads.len(), 2);
        assert_eq!(current.threads[0].display_name, "General");
        assert_eq!(current.threads[1].display_name, "Project Team");
    }

    #[tokio::test]
    async fn test_get_messages_authenticated_path() {
        // Verify get_messages works when authenticated (ordering is tested by apply_pagination unit tests)
        let temp = TempDir::new().unwrap();
        let service = make_authenticated_service(&temp).await;

        let result = service.get_messages("non-existent-thread", 50, None).await;

        // Should not fail with NotAuthenticated - either ThreadNotFound or empty is acceptable
        assert!(
            !matches!(result, Err(MessagingError::NotAuthenticated)),
            "should be authenticated via demo mode"
        );
    }

    #[tokio::test]
    async fn test_get_messages_accepts_pagination_params() {
        // Verify get_messages accepts pagination parameters without error
        // (actual pagination behavior is tested by apply_pagination unit tests)
        let temp = TempDir::new().unwrap();
        let service = make_authenticated_service(&temp).await;

        let result = service.get_messages("test-thread", 10, Some(1000)).await;

        // Should not fail with NotAuthenticated - pagination params should be accepted
        assert!(
            !matches!(result, Err(MessagingError::NotAuthenticated)),
            "should be authenticated via demo mode"
        );
    }

    #[tokio::test]
    async fn test_get_messages_nonexistent_thread() {
        // Verify querying a non-existent thread returns ThreadNotFound or empty messages
        let temp = TempDir::new().unwrap();
        let service = make_authenticated_service(&temp).await;

        let result = service
            .get_messages("definitely-not-a-real-thread-id", 50, None)
            .await;

        match result {
            Err(MessagingError::ThreadNotFound(id)) => {
                assert_eq!(id, "definitely-not-a-real-thread-id");
            }
            Ok(messages) => {
                assert!(messages.is_empty(), "non-existent thread should be empty");
            }
            Err(MessagingError::NotAuthenticated) => {
                panic!("should be authenticated via demo mode");
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }

    #[tokio::test]
    async fn add_reaction_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.add_reaction("thread1", "msg1", "👍").await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn remove_reaction_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.remove_reaction("thread1", "msg1", "👍").await;
        assert!(result.is_err());
        match result {
            Err(MessagingError::NotAuthenticated) => {}
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn refresh_threads_updates_watch_channel() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        // Initial state should be empty
        let snap = service.current_snapshot();
        assert!(snap.threads.is_empty());
        assert!(!snap.loading);

        // Refresh (not authenticated, so threads remain empty)
        service.refresh_threads().await;

        let snap = service.current_snapshot();
        assert!(snap.threads.is_empty());
        assert!(!snap.loading);
    }

    // ==================== Pending Message Queue Tests ====================

    #[tokio::test]
    async fn pending_messages_starts_empty() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let pending = service.get_pending_messages();
        assert!(pending.is_empty());

        let snap = service.current_snapshot();
        assert!(snap.pending_messages.is_empty());
    }

    #[tokio::test]
    async fn queue_message_adds_to_pending() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let id = service.queue_message("thread-1", "Hello world", None);

        assert!(!id.is_empty());
        assert!(id.starts_with("pending-thread-1-"));

        let pending = service.get_pending_messages();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, id);
        assert_eq!(pending[0].thread_id, "thread-1");
        assert_eq!(pending[0].text, "Hello world");
        assert!(pending[0].reply_to_id.is_none());
        assert_eq!(pending[0].retry_count, 0);
        assert!(pending[0].status.is_pending());

        // Should be reflected in snapshot
        let snap = service.current_snapshot();
        assert_eq!(snap.pending_messages.len(), 1);
    }

    #[tokio::test]
    async fn queue_message_with_reply_to() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let _id = service.queue_message("thread-1", "Reply text", Some("msg-123"));

        let pending = service.get_pending_messages();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].reply_to_id, Some("msg-123".to_string()));
    }

    #[tokio::test]
    async fn remove_pending_message_removes_from_queue() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let id = service.queue_message("thread-1", "Hello", None);
        assert_eq!(service.get_pending_messages().len(), 1);

        let removed = service.remove_pending_message(&id);
        assert!(removed);

        let pending = service.get_pending_messages();
        assert!(pending.is_empty());

        // Should be reflected in snapshot
        let snap = service.current_snapshot();
        assert!(snap.pending_messages.is_empty());
    }

    #[tokio::test]
    async fn remove_pending_message_nonexistent_returns_false() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let removed = service.remove_pending_message("nonexistent-id");
        assert!(!removed);
    }

    #[tokio::test]
    async fn mark_pending_failed_updates_status() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let id = service.queue_message("thread-1", "Hello", None);
        service.mark_pending_failed(&id, "Network error");

        let pending = service.get_pending_messages();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].status.is_failed());
        assert_eq!(pending[0].last_error, Some("Network error".to_string()));
    }

    #[tokio::test]
    async fn pending_messages_persist_across_service_instances() {
        let temp = TempDir::new().unwrap();

        // Create first service and queue a message
        let id = {
            let service = make_service(&temp).await;
            let id = service.queue_message("thread-1", "Persisted message", None);

            // Verify it's in the queue
            assert_eq!(service.get_pending_messages().len(), 1);
            id
        };

        // Create a new service instance pointing to the same storage
        {
            let service = make_service(&temp).await;

            // The message should still be there
            let pending = service.get_pending_messages();
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].id, id);
            assert_eq!(pending[0].text, "Persisted message");
        }
    }

    #[tokio::test]
    async fn multiple_pending_messages_queue_and_remove() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;

        let id1 = service.queue_message("thread-1", "Message 1", None);
        let id2 = service.queue_message("thread-1", "Message 2", None);
        let id3 = service.queue_message("thread-2", "Message 3", None);

        assert_eq!(service.get_pending_messages().len(), 3);

        // Remove middle message
        service.remove_pending_message(&id2);
        let pending = service.get_pending_messages();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].id, id1);
        assert_eq!(pending[1].id, id3);
    }
}
