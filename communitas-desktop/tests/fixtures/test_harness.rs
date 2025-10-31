// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Test Harness for Integration Testing
//!
//! Provides `TestFixture` and builder utilities for creating test environments
//! with CoreContext, AppState, and temporary storage.

use communitas::commands::auth::AppState;
use communitas_core::{CoreContext, types::DeviceType};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;

/// Configuration for test fixtures
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Enable gossip networking (false for offline tests)
    pub enable_networking: bool,

    /// Bootstrap nodes for networking tests
    pub bootstrap_nodes: Vec<String>,

    /// Storage directory override
    pub storage_dir: Option<PathBuf>,

    /// Test timeout duration
    pub timeout: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            enable_networking: false,
            bootstrap_nodes: Vec::new(),
            storage_dir: None,
            timeout: Duration::from_secs(30),
        }
    }
}

/// Main test fixture providing test environment setup and cleanup
///
/// # Examples
///
/// ```no_run
/// use communitas_desktop::tests::fixtures::test_harness::TestFixture;
///
/// #[tokio::test]
/// async fn test_example() {
///     let fixture = TestFixture::new()
///         .unwrap()
///         .with_core_context()
///         .await
///         .unwrap();
///
///     let ctx = fixture.core_context();
///     // Use ctx for testing...
///     // Automatic cleanup on drop
/// }
/// ```
pub struct TestFixture {
    /// Temporary directory (auto-cleaned on drop)
    pub temp_dir: TempDir,

    /// Optional CoreContext for P2P/storage tests
    pub core_context: Option<Arc<RwLock<CoreContext>>>,

    /// Optional AppState for auth tests
    pub app_state: Option<AppState>,

    /// Test-specific configuration
    pub config: TestConfig,
}

impl TestFixture {
    /// Create a minimal test fixture with just temp directory
    ///
    /// # Errors
    ///
    /// Returns error if temporary directory cannot be created
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        Ok(Self {
            temp_dir,
            core_context: None,
            app_state: None,
            config: TestConfig::default(),
        })
    }

    /// Builder: Add CoreContext with default test identity
    ///
    /// Creates CoreContext with:
    /// - Four-words: "ocean-forest-moon-star"
    /// - Display name: "Test User"
    /// - Device name: "Test Device"
    /// - Storage in temp directory
    ///
    /// # Errors
    ///
    /// Returns error if CoreContext initialization fails
    pub async fn with_core_context(self) -> Result<Self, Box<dyn std::error::Error>> {
        self.with_core_context_custom(
            "ocean-forest-moon-star".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
        )
        .await
    }

    /// Builder: Add CoreContext with custom identity
    ///
    /// # Arguments
    ///
    /// * `four_words` - Four-word identity (e.g., "ocean-forest-moon-star")
    /// * `display_name` - Human-readable display name
    /// * `device_name` - Device identifier
    ///
    /// # Errors
    ///
    /// Returns error if CoreContext initialization fails
    pub async fn with_core_context_custom(
        mut self,
        four_words: String,
        display_name: String,
        device_name: String,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let storage_dir = self
            .config
            .storage_dir
            .clone()
            .unwrap_or_else(|| self.temp_dir.path().join("core_storage"));

        // Ensure storage directory exists
        std::fs::create_dir_all(&storage_dir)?;

        let ctx = CoreContext::initialize(
            four_words,
            display_name,
            device_name,
            DeviceType::Desktop,
            storage_dir,
        )
        .await
        .map_err(|e| -> Box<dyn std::error::Error> {
            Box::new(std::io::Error::other(format!(
                "Failed to initialize CoreContext: {}",
                e
            )))
        })?;

        self.core_context = Some(Arc::new(RwLock::new(ctx)));

        Ok(self)
    }

    /// Builder: Add AppState for auth tests
    ///
    /// Creates AppState with temporary storage
    ///
    /// # Errors
    ///
    /// Returns error if AppState initialization fails
    pub async fn with_app_state(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        let app_state = AppState::new();

        // Note: AppState initialization handled by the auth commands themselves
        // during test execution. We just create the empty state here.

        self.app_state = Some(app_state);

        Ok(self)
    }

    /// Builder: Enable networking with bootstrap nodes
    ///
    /// # Arguments
    ///
    /// * `bootstrap_nodes` - List of bootstrap node addresses
    pub fn with_networking(mut self, bootstrap_nodes: Vec<String>) -> Self {
        self.config.enable_networking = true;
        self.config.bootstrap_nodes = bootstrap_nodes;
        self
    }

    /// Builder: Set custom test timeout
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum duration for test operations
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Builder: Set custom storage directory
    ///
    /// # Arguments
    ///
    /// * `dir` - Path to storage directory
    pub fn with_storage_dir(mut self, dir: PathBuf) -> Self {
        self.config.storage_dir = Some(dir);
        self
    }

    /// Get CoreContext or panic with helpful message
    ///
    /// # Panics
    ///
    /// Panics if CoreContext was not initialized via `with_core_context()`
    pub fn core_context(&self) -> Arc<RwLock<CoreContext>> {
        self.core_context
            .clone()
            .expect("CoreContext not initialized - call with_core_context() first")
    }

    /// Get AppState or panic with helpful message
    ///
    /// # Panics
    ///
    /// Panics if AppState was not initialized via `with_app_state()`
    pub fn app_state(&self) -> &AppState {
        self.app_state
            .as_ref()
            .expect("AppState not initialized - call with_app_state() first")
    }

    /// Get temporary directory path
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Get test configuration
    pub fn config(&self) -> &TestConfig {
        &self.config
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Cleanup networking if enabled
        if self.config.enable_networking {
            // TODO: Disconnect gossip networking when we implement it
            tracing::debug!("Cleaning up networking connections");
        }

        // CoreContext and AppState Drop impls handle their own cleanup
        // TempDir Drop impl handles directory cleanup automatically

        tracing::debug!("TestFixture cleanup complete");
    }
}
