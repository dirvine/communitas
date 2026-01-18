//! Shared Rust service layer consumed by all Communitas UI surfaces (Dioxus + MCP).

pub mod auth;
pub mod directory;
pub mod messaging;
pub mod navigation;
pub mod presence;
pub mod storage;

use std::sync::Arc;

use auth::AuthController;
use directory::DirectoryService;
use messaging::MessagingService;
use navigation::NavigationStore;
use presence::PresenceService;
use storage::UiStorage;
use thiserror::Error;

/// Aggregates shared UI services for convenient dependency injection.
#[derive(Clone)]
pub struct UiServices {
    storage: UiStorage,
    auth: Arc<AuthController>,
    navigation: Arc<NavigationStore>,
    directory: Arc<DirectoryService>,
    messaging: Arc<MessagingService>,
    presence: Arc<PresenceService>,
}

impl UiServices {
    /// Discover standard Communitas storage paths and create all service controllers.
    pub fn bootstrap() -> Result<Self, UiServiceInitError> {
        let storage = UiStorage::discover()?;
        Self::new(storage)
    }

    /// Create services using the provided storage configuration.
    pub fn new(storage: UiStorage) -> Result<Self, UiServiceInitError> {
        let auth = Arc::new(AuthController::new(storage.clone())?);
        let navigation = Arc::new(NavigationStore::new(storage.clone())?);
        let directory = Arc::new(DirectoryService::new(auth.clone()));
        let messaging = Arc::new(MessagingService::new(auth.clone()));
        let presence = Arc::new(PresenceService::new(auth.clone(), directory.clone()));
        Ok(Self {
            storage,
            auth,
            navigation,
            directory,
            messaging,
            presence,
        })
    }

    /// Access the storage configuration.
    pub fn storage(&self) -> &UiStorage {
        &self.storage
    }

    /// Authentication/session controller.
    pub fn auth(&self) -> Arc<AuthController> {
        self.auth.clone()
    }

    /// Navigation preferences/state controller.
    pub fn navigation(&self) -> Arc<NavigationStore> {
        self.navigation.clone()
    }

    pub fn directory(&self) -> Arc<DirectoryService> {
        self.directory.clone()
    }

    /// Messaging threads and messages service.
    pub fn messaging(&self) -> Arc<MessagingService> {
        self.messaging.clone()
    }

    /// Presence status tracking for contacts.
    pub fn presence(&self) -> Arc<PresenceService> {
        self.presence.clone()
    }
}

/// Errors that can occur when initializing the shared service layer.
#[derive(Debug, Error)]
pub enum UiServiceInitError {
    #[error("storage initialization failed: {0}")]
    Storage(#[from] storage::StorageError),
    #[error("auth controller failed: {0}")]
    Auth(#[from] auth::AuthError),
    #[error("navigation controller failed: {0}")]
    Navigation(#[from] navigation::NavigationError),
    #[error("directory controller failed: {0}")]
    Directory(#[from] directory::DirectoryError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthService, AuthStateSnapshot};
    use crate::navigation::{EntityNavigationKey, NavigationService};
    use tempfile::TempDir;

    fn make_services(temp: &TempDir) -> UiServices {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        UiServices::new(storage).unwrap()
    }

    #[test]
    fn ui_services_constructs_all_components() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        // All services should be accessible
        let _ = services.storage();
        let _ = services.auth();
        let _ = services.navigation();
        let _ = services.directory();
        let _ = services.messaging();
        let _ = services.presence();
    }

    #[test]
    fn presence_starts_empty() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let snap = services.presence().current_snapshot();
        assert!(snap.statuses.is_empty());
        assert!(snap.last_seen.is_empty());
    }

    #[test]
    fn services_share_storage_path() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let storage_root = services.storage().root();
        assert_eq!(storage_root, temp.path());
    }

    #[test]
    fn auth_starts_logged_out() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let rx = services.auth().subscribe();
        match &*rx.borrow() {
            AuthStateSnapshot::LoggedOut => {} // expected
            other => panic!("expected LoggedOut, got {other:?}"),
        }
    }

    #[test]
    fn directory_starts_with_empty_snapshot() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let snap = services.directory().current_snapshot();
        assert!(snap.identity.is_none());
        assert!(snap.entities.is_empty());
        assert!(snap.contacts.is_empty());
    }

    #[test]
    fn navigation_starts_with_empty_recents() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let snap = services.navigation().current_snapshot();
        assert!(snap.recent_entities.is_empty());
        assert!(snap.recent_contacts.is_empty());
        assert!(snap.starred_entities.is_empty());
        assert!(snap.starred_contacts.is_empty());
    }

    #[test]
    fn messaging_starts_with_empty_threads() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let snap = services.messaging().current_snapshot();
        assert!(snap.threads.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn navigation_updates_propagate_to_subscribers() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        let mut rx = services.navigation().subscribe();

        // Record an entity
        let key = EntityNavigationKey::new("channel", "ch1");
        services
            .navigation()
            .record_entity(key.clone())
            .await
            .unwrap();

        // Wait for update
        rx.changed().await.unwrap();

        let snap = rx.borrow().clone();
        assert_eq!(snap.recent_entities.len(), 1);
        assert_eq!(snap.recent_entities[0], key);
    }

    #[tokio::test]
    async fn directory_refresh_fails_without_auth() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp);

        // Not authenticated, so refresh should fail
        let result = services.directory().refresh_all().await;
        assert!(result.is_err());
    }

    #[test]
    fn services_clone_shares_underlying_arcs() {
        let temp = TempDir::new().unwrap();
        let services1 = make_services(&temp);
        let services2 = services1.clone();

        // Both should point to the same underlying Arc
        assert!(Arc::ptr_eq(&services1.auth(), &services2.auth()));
        assert!(Arc::ptr_eq(
            &services1.navigation(),
            &services2.navigation()
        ));
        assert!(Arc::ptr_eq(&services1.directory(), &services2.directory()));
        assert!(Arc::ptr_eq(&services1.messaging(), &services2.messaging()));
        assert!(Arc::ptr_eq(&services1.presence(), &services2.presence()));
    }

    #[tokio::test]
    async fn cloned_services_share_state_updates() {
        let temp = TempDir::new().unwrap();
        let services1 = make_services(&temp);
        let services2 = services1.clone();

        // Subscribe via services1
        let mut rx = services1.navigation().subscribe();

        // Record via services2
        services2
            .navigation()
            .record_contact("alice".to_string())
            .await
            .unwrap();

        // Should see update via services1's subscription
        rx.changed().await.unwrap();
        let snap = rx.borrow().clone();
        assert_eq!(snap.recent_contacts, vec!["alice".to_string()]);
    }
}
