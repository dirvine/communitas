//! Update service for managing application auto-updates.
//!
//! Provides types and a service trait for checking, downloading, and
//! installing application updates via the Tauri updater system.

use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// The new version string (e.g., "1.2.0").
    pub version: String,
    /// Release notes or changelog for this update.
    pub release_notes: String,
    /// URL to download the update package.
    pub download_url: String,
    /// Signature for verifying the update package.
    pub signature: String,
    /// ISO 8601 timestamp when the update was published.
    pub published_at: String,
}

/// Current status of the update system.
#[derive(Debug, Clone, Default)]
pub enum UpdateStatus {
    /// No update activity in progress.
    #[default]
    Idle,
    /// Currently checking for updates.
    Checking,
    /// An update is available.
    Available(UpdateInfo),
    /// Update is being downloaded.
    Downloading {
        /// Download progress from 0.0 to 1.0.
        progress: f32,
    },
    /// Update has been downloaded and is ready to install.
    ReadyToInstall,
    /// The application is already up to date.
    UpToDate,
    /// An error occurred during update operations.
    Error(String),
}

/// Errors that can occur during update operations.
#[derive(Debug, Error)]
pub enum UpdateError {
    /// Failed to check for updates.
    #[error("failed to check for updates: {0}")]
    CheckFailed(String),
    /// Failed to download update.
    #[error("failed to download update: {0}")]
    DownloadFailed(String),
    /// Failed to install update.
    #[error("failed to install update: {0}")]
    InstallFailed(String),
    /// Update is not ready for installation.
    #[error("no update ready to install")]
    NotReady,
    /// Internal synchronization error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Service for managing application updates.
///
/// Provides methods to check for updates, download them, and trigger
/// installation. The implementation wraps the Tauri updater API.
pub trait UpdateService: Send + Sync {
    /// Check for available updates.
    ///
    /// Returns `Some(UpdateInfo)` if an update is available, `None` if up to date.
    fn check_for_updates(
        &self,
    ) -> impl std::future::Future<Output = Result<Option<UpdateInfo>, UpdateError>> + Send;

    /// Download an available update.
    ///
    /// Progress can be monitored via `get_status()`.
    fn download_update(
        &self,
        info: &UpdateInfo,
    ) -> impl std::future::Future<Output = Result<(), UpdateError>> + Send;

    /// Install a downloaded update.
    ///
    /// This typically restarts the application.
    fn install_update(&self) -> impl std::future::Future<Output = Result<(), UpdateError>> + Send;

    /// Get the current application version.
    fn current_version(&self) -> &str;

    /// Get the current update status.
    fn get_status(&self) -> UpdateStatus;
}

/// Mock implementation of `UpdateService` for development and testing.
///
/// In production, this would be replaced with a Tauri-based implementation
/// that uses the actual updater API.
pub struct UpdateServiceImpl {
    status: Arc<RwLock<UpdateStatus>>,
    current_version: String,
}

impl UpdateServiceImpl {
    /// Create a new UpdateService instance.
    #[must_use]
    pub fn new(current_version: String) -> Self {
        Self {
            status: Arc::new(RwLock::new(UpdateStatus::Idle)),
            current_version,
        }
    }

    fn set_status(&self, status: UpdateStatus) -> Result<(), UpdateError> {
        let mut guard = self
            .status
            .write()
            .map_err(|e| UpdateError::Internal(format!("lock poisoned: {e}")))?;
        *guard = status;
        Ok(())
    }
}

impl Default for UpdateServiceImpl {
    fn default() -> Self {
        Self::new(env!("CARGO_PKG_VERSION").to_string())
    }
}

impl UpdateService for UpdateServiceImpl {
    async fn check_for_updates(&self) -> Result<Option<UpdateInfo>, UpdateError> {
        self.set_status(UpdateStatus::Checking)?;

        // Mock implementation - in production, this would call Tauri updater API
        // For now, always report up-to-date
        self.set_status(UpdateStatus::UpToDate)?;

        Ok(None)
    }

    async fn download_update(&self, _info: &UpdateInfo) -> Result<(), UpdateError> {
        // Mock implementation - simulate download progress
        for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
            self.set_status(UpdateStatus::Downloading { progress })?;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        self.set_status(UpdateStatus::ReadyToInstall)?;
        Ok(())
    }

    async fn install_update(&self) -> Result<(), UpdateError> {
        let status = self
            .status
            .read()
            .map_err(|e| UpdateError::Internal(format!("lock poisoned: {e}")))?
            .clone();

        match status {
            UpdateStatus::ReadyToInstall => {
                // Mock implementation - in production, this would trigger app restart
                self.set_status(UpdateStatus::Idle)?;
                Ok(())
            }
            _ => Err(UpdateError::NotReady),
        }
    }

    fn current_version(&self) -> &str {
        &self.current_version
    }

    fn get_status(&self) -> UpdateStatus {
        self.status
            .read()
            .map(|s| s.clone())
            .unwrap_or_else(|_| UpdateStatus::Error("failed to read status".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_for_updates() {
        let service = UpdateServiceImpl::new("0.1.0".to_string());
        let result = service.check_for_updates().await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert!(matches!(service.get_status(), UpdateStatus::UpToDate));
    }

    #[tokio::test]
    async fn test_current_version() {
        let service = UpdateServiceImpl::new("1.2.3".to_string());
        assert_eq!(service.current_version(), "1.2.3");
    }

    #[tokio::test]
    async fn test_download_update() {
        let service = UpdateServiceImpl::new("0.1.0".to_string());
        let info = UpdateInfo {
            version: "1.0.1".to_string(),
            release_notes: "Test release".to_string(),
            download_url: "https://example.com/update".to_string(),
            signature: "abc123".to_string(),
            published_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = service.download_update(&info).await;
        assert!(result.is_ok());
        assert!(matches!(service.get_status(), UpdateStatus::ReadyToInstall));
    }

    #[tokio::test]
    async fn test_install_update_requires_download() {
        let service = UpdateServiceImpl::new("0.1.0".to_string());
        let result = service.install_update().await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UpdateError::NotReady));
    }

    #[tokio::test]
    async fn test_install_update_after_download() {
        let service = UpdateServiceImpl::new("0.1.0".to_string());
        let info = UpdateInfo {
            version: "1.0.1".to_string(),
            release_notes: "Test release".to_string(),
            download_url: "https://example.com/update".to_string(),
            signature: "abc123".to_string(),
            published_at: "2024-01-01T00:00:00Z".to_string(),
        };

        service.download_update(&info).await.unwrap();
        let result = service.install_update().await;
        assert!(result.is_ok());
        assert!(matches!(service.get_status(), UpdateStatus::Idle));
    }

    #[test]
    fn test_update_status_variants() {
        let idle = UpdateStatus::Idle;
        let checking = UpdateStatus::Checking;
        let downloading = UpdateStatus::Downloading { progress: 0.5 };
        let ready = UpdateStatus::ReadyToInstall;
        let up_to_date = UpdateStatus::UpToDate;
        let error = UpdateStatus::Error("test error".to_string());

        assert!(matches!(idle, UpdateStatus::Idle));
        assert!(matches!(checking, UpdateStatus::Checking));
        assert!(matches!(downloading, UpdateStatus::Downloading { .. }));
        assert!(matches!(ready, UpdateStatus::ReadyToInstall));
        assert!(matches!(up_to_date, UpdateStatus::UpToDate));
        assert!(matches!(error, UpdateStatus::Error(_)));
    }

    #[test]
    fn test_default_implementation() {
        let service = UpdateServiceImpl::default();
        assert!(!service.current_version().is_empty());
        assert!(matches!(service.get_status(), UpdateStatus::Idle));
    }
}
