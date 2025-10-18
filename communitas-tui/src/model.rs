//! Application model implementing the Update trait for tui-realm
//!
//! This module defines the core application state and message handling logic.
//! Following TDD principles, we start with tests and implement to pass them.

use crate::backend::Backend;
use crate::messages::{ComponentId, Msg, NetworkStatus};
use tuirealm::{Application, EventListenerCfg, Update};

/// Main application model that holds all state
pub struct Model {
    /// Should the application quit?
    pub quit: bool,
    /// Should the UI be redrawn?
    pub redraw: bool,
    /// Current status message
    pub status_message: String,
    /// Current error message if any
    pub error_message: Option<String>,
    /// Current user identity (four-word address)
    pub identity: Option<String>,
    /// Network connection status
    pub network_status: NetworkStatus,
    /// Backend integration
    pub backend: Backend,
    /// Component application (using NoUserEvent for now)
    pub app: Application<ComponentId, Msg, tuirealm::NoUserEvent>,
}

impl Model {
    /// Create a new Model instance
    pub fn new(backend: Backend) -> Self {
        Self {
            quit: false,
            redraw: true,
            status_message: "Welcome to Communitas TUI".to_string(),
            error_message: None,
            identity: None,
            network_status: NetworkStatus::Disconnected,
            backend,
            app: Application::init(EventListenerCfg::default()),
        }
    }

    /// Set the status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
        self.redraw = true;
    }

    /// Set an error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
        self.redraw = true;
    }

    /// Clear the error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
        self.redraw = true;
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.identity.is_some()
    }
}

impl Update<Msg> for Model {
    /// Update the model based on a message
    ///
    /// This is the core message handling logic. Returns an optional message
    /// for chaining (allows one message to trigger another).
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        // Mark for redraw whenever we process a message
        self.redraw = true;

        match msg {
            Some(Msg::AppClose) => {
                self.quit = true;
                None
            }
            Some(Msg::StatusUpdated(message)) => {
                self.set_status(message);
                None
            }
            Some(Msg::ErrorOccurred(error)) => {
                self.set_error(error);
                None
            }
            Some(Msg::AuthenticationSuccess { four_words }) => {
                self.identity = Some(four_words.clone());
                self.set_status(format!("Authenticated as {}", four_words));
                Some(Msg::NetworkStatusChanged(NetworkStatus::Connecting))
            }
            Some(Msg::NetworkStatusChanged(status)) => {
                self.network_status = status.clone();
                match status {
                    NetworkStatus::Connected => {
                        self.set_status("Connected to network");
                    }
                    NetworkStatus::Connecting => {
                        self.set_status("Connecting to network...");
                    }
                    NetworkStatus::Disconnected => {
                        self.set_status("Disconnected from network");
                    }
                    NetworkStatus::Error(err) => {
                        self.set_error(format!("Network error: {}", err));
                    }
                }
                None
            }
            Some(Msg::None) | None => {
                // No-op
                None
            }
            _ => {
                // TODO: Handle other messages
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper to create a test model
    async fn create_test_model() -> (Model, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let backend = Backend::new(temp_dir.path().to_path_buf(), true)
            .await
            .expect("Failed to create backend");
        let model = Model::new(backend);
        (model, temp_dir)
    }

    #[tokio::test]
    async fn test_model_initialization() {
        let (model, _temp) = create_test_model().await;

        assert!(!model.quit, "Model should not be set to quit initially");
        assert!(model.redraw, "Model should need redraw initially");
        assert!(!model.is_authenticated(), "Model should not be authenticated initially");
        assert_eq!(model.network_status, NetworkStatus::Disconnected);
        assert!(model.error_message.is_none(), "Should have no error initially");
    }

    #[tokio::test]
    async fn test_app_close_message() {
        let (mut model, _temp) = create_test_model().await;

        let result = model.update(Some(Msg::AppClose));

        assert!(model.quit, "Model should be set to quit after AppClose");
        assert_eq!(result, None, "AppClose should not return a chained message");
    }

    #[tokio::test]
    async fn test_status_update_message() {
        let (mut model, _temp) = create_test_model().await;

        let status_text = "Test status message";
        let result = model.update(Some(Msg::StatusUpdated(status_text.to_string())));

        assert_eq!(model.status_message, status_text);
        assert!(model.redraw, "Should mark for redraw");
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_error_message() {
        let (mut model, _temp) = create_test_model().await;

        let error_text = "Test error";
        let result = model.update(Some(Msg::ErrorOccurred(error_text.to_string())));

        assert_eq!(model.error_message, Some(error_text.to_string()));
        assert!(model.redraw, "Should mark for redraw");
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_authentication_success() {
        let (mut model, _temp) = create_test_model().await;

        let four_words = "ocean-forest-moon-star".to_string();
        let result = model.update(Some(Msg::AuthenticationSuccess {
            four_words: four_words.clone()
        }));

        assert!(model.is_authenticated(), "Should be authenticated");
        assert_eq!(model.identity, Some(four_words.clone()));
        assert!(model.status_message.contains(&four_words));

        // Should chain a network status change message
        assert_eq!(result, Some(Msg::NetworkStatusChanged(NetworkStatus::Connecting)));
    }

    #[tokio::test]
    async fn test_network_status_connected() {
        let (mut model, _temp) = create_test_model().await;

        let result = model.update(Some(Msg::NetworkStatusChanged(NetworkStatus::Connected)));

        assert_eq!(model.network_status, NetworkStatus::Connected);
        assert!(model.status_message.contains("Connected"));
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_network_status_error() {
        let (mut model, _temp) = create_test_model().await;

        let error_msg = "Connection failed".to_string();
        let result = model.update(Some(Msg::NetworkStatusChanged(
            NetworkStatus::Error(error_msg.clone())
        )));

        assert!(matches!(model.network_status, NetworkStatus::Error(_)));
        assert!(model.error_message.is_some());
        assert!(model.error_message.as_ref().unwrap().contains("Connection failed"));
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_message_chaining() {
        let (mut model, _temp) = create_test_model().await;

        // Authentication should trigger network connection
        let msg1 = model.update(Some(Msg::AuthenticationSuccess {
            four_words: "test-identity-four-words".to_string()
        }));

        assert!(msg1.is_some(), "Should return chained message");

        // Process the chained message
        let msg2 = model.update(msg1);

        assert_eq!(model.network_status, NetworkStatus::Connecting);
        assert!(msg2.is_none(), "Network status change should not chain");
    }

    #[tokio::test]
    async fn test_none_message() {
        let (mut model, _temp) = create_test_model().await;

        let initial_status = model.status_message.clone();
        let result = model.update(Some(Msg::None));

        assert_eq!(model.status_message, initial_status, "Status should not change");
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_set_status_method() {
        let (mut model, _temp) = create_test_model().await;

        model.redraw = false;
        model.set_status("New status");

        assert_eq!(model.status_message, "New status");
        assert!(model.redraw, "Should mark for redraw");
    }

    #[tokio::test]
    async fn test_error_management() {
        let (mut model, _temp) = create_test_model().await;

        // Set error
        model.set_error("Test error");
        assert_eq!(model.error_message, Some("Test error".to_string()));

        // Clear error
        model.clear_error();
        assert!(model.error_message.is_none());
    }
}
