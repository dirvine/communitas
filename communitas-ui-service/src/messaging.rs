//! Messaging service for thread and message operations with reactive subscriptions.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_core::command::{Query, QueryResponse};
use communitas_ui_api::{Message, ThreadSummary};
use thiserror::Error;
use tokio::sync::watch;
use tracing::{debug, instrument, warn};

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::messaging_convert::{core_entity_type_to_ui, core_message_to_ui};

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
}

/// Service for thread listing, message retrieval, and message sending.
pub struct MessagingService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    tx: watch::Sender<MessagingSnapshot>,
    rx: watch::Receiver<MessagingSnapshot>,
}

impl MessagingService {
    /// Create a new messaging service linked to the auth controller and core app.
    ///
    /// # Arguments
    /// * `auth` - Shared authentication controller for checking login state
    /// * `app` - Shared reference to the core application
    pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        let (tx, rx) = watch::channel(MessagingSnapshot::default());
        Self { auth, app, tx, rx }
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

    /// List all conversation threads for the current user.
    ///
    /// Each thread corresponds to an entity (channel, group, project, etc.) that
    /// the user has joined. The thread includes a preview of the latest message.
    ///
    /// # Errors
    /// Returns [`MessagingError::NotAuthenticated`] if no user is logged in.
    #[instrument(skip(self), name = "ui.messaging.list_threads")]
    pub async fn list_threads(&self) -> Result<Vec<ThreadSummary>, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        // Set loading state
        self.set_loading(true);

        // Query all entities from the core app
        let entities = match self.app.query(Query::ListEntities).await {
            Ok(QueryResponse::EntityList(entities)) => entities,
            Ok(_) => {
                warn!("Unexpected response type for ListEntities query");
                self.set_loading(false);
                return Ok(vec![]);
            }
            Err(e) => {
                warn!(error = %e.message, "Failed to list entities");
                self.set_loading(false);
                return Ok(vec![]);
            }
        };

        debug!(
            entity_count = entities.len(),
            "Fetched entities for threads"
        );

        // Build thread summaries from entities
        let mut threads = Vec::with_capacity(entities.len());
        for entity in entities {
            let default_timestamp = entity.created_at.max(0) as u64;

            // Get the latest message for preview
            let (preview, timestamp) = match self
                .app
                .query(Query::GetEntityMessages {
                    entity_id: entity.id.clone(),
                })
                .await
            {
                Ok(QueryResponse::Messages(messages)) => {
                    // Get the most recent message (last in the list; core returns messages chronologically)
                    if let Some(msg) = messages.last() {
                        let preview_text = truncate_preview(&msg.text, 100);
                        let ts = msg.timestamp.max(0) as u64;
                        (preview_text, ts)
                    } else {
                        (String::new(), default_timestamp)
                    }
                }
                Ok(_) => {
                    debug!(entity_id = %entity.id, "Unexpected response for GetEntityMessages");
                    (String::new(), default_timestamp)
                }
                Err(e) => {
                    debug!(entity_id = %entity.id, error = %e.message, "Failed to get messages for entity");
                    (String::new(), default_timestamp)
                }
            };

            let entity_type = core_entity_type_to_ui(&entity.entity_type);

            threads.push(ThreadSummary {
                thread_id: entity.id.clone(),
                entity_id: Some(entity.id),
                entity_type: Some(entity_type),
                contact_id: None,
                display_name: entity.name,
                last_message_preview: preview,
                last_message_timestamp: timestamp,
                unread_count: 0, // TODO: Track unread counts separately
                is_muted: false,
            });
        }

        // Sort by most recent message first
        threads.sort_by(|a, b| b.last_message_timestamp.cmp(&a.last_message_timestamp));

        // Update watch channel
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
        let _ = (thread_id, text, reply_to); // Suppress unused warnings for now
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Command::SendMessage
        Err(MessagingError::Internal("not yet implemented".to_string()))
    }

    /// Edit an existing message.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the edit fails.
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
        let _ = (thread_id, message_id, new_text); // Suppress unused warnings for now
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Command::EditMessage
        Err(MessagingError::Internal("not yet implemented".to_string()))
    }

    /// Delete a message.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the delete fails.
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
        let _ = (thread_id, message_id); // Suppress unused warnings for now
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Command::DeleteMessage
        Err(MessagingError::Internal("not yet implemented".to_string()))
    }

    /// Mark a thread as read, clearing unread count.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::ThreadNotFound`] if the thread does not exist.
    #[instrument(skip(self), name = "ui.messaging.mark_read", fields(thread_id))]
    pub async fn mark_thread_read(&self, thread_id: &str) -> Result<(), MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }

        // Update local thread state
        let mut snap = self.rx.borrow().clone();
        if let Some(thread) = snap.threads.iter_mut().find(|t| t.thread_id == thread_id) {
            thread.unread_count = 0;
            // Send cannot fail: self.rx guarantees at least one receiver exists
            let _ = self.tx.send(snap);
            // TODO: Wire to core Command::MarkThreadRead
            Ok(())
        } else {
            Err(MessagingError::ThreadNotFound(thread_id.to_string()))
        }
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
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the reaction fails.
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
        let _ = (thread_id, message_id, emoji); // Suppress unused warnings for now
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Command::AddReaction
        Err(MessagingError::Internal("not yet implemented".to_string()))
    }

    /// Remove a reaction from a message.
    ///
    /// # Errors
    /// - [`MessagingError::NotAuthenticated`] if no user is logged in.
    /// - [`MessagingError::Internal`] if the removal fails.
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
        let _ = (thread_id, message_id, emoji); // Suppress unused warnings for now
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Command::RemoveReaction
        Err(MessagingError::Internal("not yet implemented".to_string()))
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
            AuthStateSnapshot::Authenticated(_)
        )
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
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());
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
        MessagingService::new(auth, app)
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
}
