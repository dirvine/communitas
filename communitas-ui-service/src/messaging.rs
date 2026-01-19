//! Messaging service for thread and message operations with reactive subscriptions.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_ui_api::{Message, ThreadSummary};
use thiserror::Error;
use tokio::sync::watch;
use tracing::instrument;

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};

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
    /// # Errors
    /// Returns [`MessagingError::NotAuthenticated`] if no user is logged in.
    #[instrument(skip(self), name = "ui.messaging.list_threads")]
    pub async fn list_threads(&self) -> Result<Vec<ThreadSummary>, MessagingError> {
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Query::ListThreads when available
        Ok(self.rx.borrow().threads.clone())
    }

    /// Get messages for a thread with pagination.
    ///
    /// # Errors
    /// Returns [`MessagingError::NotAuthenticated`] if no user is logged in.
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
        let _ = (thread_id, limit, before); // Suppress unused warnings for now
        if !self.is_authenticated() {
            return Err(MessagingError::NotAuthenticated);
        }
        // TODO: Wire to core Query::GetMessages
        Ok(vec![])
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
}
