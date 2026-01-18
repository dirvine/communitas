//! Shared Rust service layer consumed by all Communitas UI surfaces (Dioxus + MCP).

pub mod auth;
pub mod directory;
pub mod navigation;
pub mod storage;

use std::sync::Arc;

use auth::AuthController;
use directory::DirectoryService;
use navigation::NavigationStore;
use storage::UiStorage;
use thiserror::Error;

/// Aggregates shared UI services for convenient dependency injection.
#[derive(Clone)]
pub struct UiServices {
    storage: UiStorage,
    auth: Arc<AuthController>,
    navigation: Arc<NavigationStore>,
    directory: Arc<DirectoryService>,
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
        Ok(Self {
            storage,
            auth,
            navigation,
            directory,
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
