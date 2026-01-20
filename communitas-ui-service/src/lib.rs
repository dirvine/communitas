//! Shared Rust service layer consumed by all Communitas UI surfaces (Dioxus + MCP).

pub mod auth;
pub mod call;
pub mod canvas;
pub mod directory;
pub mod drive;
pub mod kanban;
pub mod messaging;
pub mod messaging_convert;
pub mod navigation;
pub mod presence;
pub mod storage;

use std::sync::Arc;

use auth::AuthController;
use call::CallService;
use canvas::CanvasService;
use communitas_core::app::CommunitasApp;
use communitas_core::generate_id_words;
use directory::DirectoryService;
use drive::DriveService;
use kanban::KanbanService;
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
    kanban: Arc<KanbanService>,
    canvas: Arc<CanvasService>,
    drive: Arc<DriveService>,
    call: Arc<CallService>,
}

impl UiServices {
    /// Bootstrap UiServices with auto-discovered storage and a generated identity.
    ///
    /// This is a convenience method for applications that need to start without
    /// prior authentication. A temporary identity is created, and the user can
    /// log in or create a new identity through the auth service later.
    ///
    /// # Errors
    /// Returns an error if storage discovery fails or the app cannot be initialized.
    pub fn bootstrap() -> Result<Self, UiServiceInitError> {
        let storage = UiStorage::discover()?;
        let storage_path = storage.root_string()?;

        // Generate a bootstrap identity
        let id_words =
            generate_id_words().map_err(|e| UiServiceInitError::AppInit(e.to_string()))?;

        // Create the app using a blocking runtime since bootstrap is called from sync context
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| UiServiceInitError::AppInit(format!("failed to create runtime: {e}")))?;

        let app = rt
            .block_on(async {
                CommunitasApp::new(
                    id_words,
                    "Bootstrap User".to_string(),
                    "Desktop".to_string(),
                    storage_path,
                )
                .await
            })
            .map_err(|e| UiServiceInitError::AppInit(e.to_string()))?;

        Self::new(storage, Arc::new(app))
    }

    /// Create services using the provided storage configuration and core app.
    ///
    /// # Errors
    /// Returns [`UiServiceInitError`] if the auth controller or navigation store
    /// cannot be initialized from the provided storage.
    pub fn new(storage: UiStorage, app: Arc<CommunitasApp>) -> Result<Self, UiServiceInitError> {
        let auth = Arc::new(AuthController::new(storage.clone())?);
        let navigation = Arc::new(NavigationStore::new(storage.clone())?);
        let directory = Arc::new(DirectoryService::new(auth.clone()));
        let messaging = Arc::new(MessagingService::new(auth.clone(), app.clone()));
        let presence = Arc::new(PresenceService::new(auth.clone(), directory.clone()));
        let kanban = Arc::new(KanbanService::new(auth.clone()));
        let canvas = Arc::new(CanvasService::new(auth.clone()));
        let drive = Arc::new(DriveService::new(auth.clone(), app.clone()));
        let call = Arc::new(CallService::new(auth.clone()));
        Ok(Self {
            storage,
            auth,
            navigation,
            directory,
            messaging,
            presence,
            kanban,
            canvas,
            drive,
            call,
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

    /// Directory service for entities and contacts.
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

    /// Kanban boards and cards service.
    pub fn kanban(&self) -> Arc<KanbanService> {
        self.kanban.clone()
    }

    /// Canvas drawing and visual surfaces service.
    pub fn canvas(&self) -> Arc<CanvasService> {
        self.canvas.clone()
    }

    /// Drive and virtual disk service.
    pub fn drive(&self) -> Arc<DriveService> {
        self.drive.clone()
    }

    /// Real-time voice/video call service.
    pub fn call(&self) -> Arc<CallService> {
        self.call.clone()
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
    #[error("app initialization failed: {0}")]
    AppInit(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{AuthService, AuthStateSnapshot};
    use crate::navigation::{EntityNavigationKey, NavigationService};
    use tempfile::TempDir;

    async fn make_services(temp: &TempDir) -> UiServices {
        let storage = UiStorage::from_path(temp.path()).unwrap();
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
        UiServices::new(storage, app).unwrap()
    }

    #[tokio::test]
    async fn ui_services_constructs_all_components() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        // All services should be accessible
        let _ = services.storage();
        let _ = services.auth();
        let _ = services.navigation();
        let _ = services.directory();
        let _ = services.messaging();
        let _ = services.presence();
        let _ = services.kanban();
        let _ = services.canvas();
        let _ = services.drive();
        let _ = services.call();
    }

    #[tokio::test]
    async fn presence_starts_empty() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let snap = services.presence().current_snapshot();
        assert!(snap.statuses.is_empty());
        assert!(snap.last_seen.is_empty());
    }

    #[tokio::test]
    async fn services_share_storage_path() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let storage_root = services.storage().root();
        assert_eq!(storage_root, temp.path());
    }

    #[tokio::test]
    async fn auth_starts_logged_out() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let rx = services.auth().subscribe();
        match &*rx.borrow() {
            AuthStateSnapshot::LoggedOut => {} // expected
            other => panic!("expected LoggedOut, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn directory_starts_with_empty_snapshot() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        // Use async snapshot() instead of current_snapshot() to avoid blocking_read in async context
        let snap = services.directory().snapshot().await;
        assert!(snap.identity.is_none());
        assert!(snap.entities.is_empty());
        assert!(snap.contacts.is_empty());
    }

    #[tokio::test]
    async fn navigation_starts_with_empty_recents() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        // Use async snapshot() instead of current_snapshot() to avoid blocking_read in async context
        let snap = services.navigation().snapshot().await;
        assert!(snap.recent_entities.is_empty());
        assert!(snap.recent_contacts.is_empty());
        assert!(snap.starred_entities.is_empty());
        assert!(snap.starred_contacts.is_empty());
    }

    #[tokio::test]
    async fn messaging_starts_with_empty_threads() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let snap = services.messaging().current_snapshot();
        assert!(snap.threads.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn kanban_starts_with_empty_boards() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let snap = services.kanban().current_snapshot();
        assert!(snap.boards.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn canvas_starts_with_empty_scene() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let snap = services.canvas().current_snapshot();
        assert!(snap.elements.is_empty());
        assert!(snap.selected_ids.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn drive_starts_with_empty_snapshot() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

        let snap = services.drive().current_snapshot();
        assert!(snap.uploads.is_empty());
        assert!(snap.downloads.is_empty());
        assert!(snap.current_directory.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn navigation_updates_propagate_to_subscribers() {
        let temp = TempDir::new().unwrap();
        let services = make_services(&temp).await;

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

    // Uses a thread with larger stack (8MB) to avoid stack overflow from
    // large async state machine in CommunitasApp + DirectoryService::refresh_all chain
    #[test]
    fn directory_refresh_fails_without_auth() {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let temp = TempDir::new().unwrap();
                    let services = make_services(&temp).await;

                    // Not authenticated, so refresh should fail
                    let result = services.directory().refresh_all().await;
                    assert!(result.is_err());
                });
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[tokio::test]
    async fn services_clone_shares_underlying_arcs() {
        let temp = TempDir::new().unwrap();
        let services1 = make_services(&temp).await;
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
        assert!(Arc::ptr_eq(&services1.kanban(), &services2.kanban()));
        assert!(Arc::ptr_eq(&services1.canvas(), &services2.canvas()));
        assert!(Arc::ptr_eq(&services1.drive(), &services2.drive()));
    }

    #[tokio::test]
    async fn cloned_services_share_state_updates() {
        let temp = TempDir::new().unwrap();
        let services1 = make_services(&temp).await;
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
