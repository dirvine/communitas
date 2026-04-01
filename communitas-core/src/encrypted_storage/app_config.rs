// SPDX-License-Identifier: MIT OR Apache-2.0

//! Application Configuration Management
//!
//! Manages app-level configuration separate from encrypted vaults.
//! Stores preferences like last used identity, UI settings, etc.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Default for auto_update_enabled field
fn default_auto_update_enabled() -> bool {
    true
}

/// Application-level configuration (unencrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Last successfully authenticated identity
    pub last_identity: Option<String>,

    /// Enable auto-login using platform keyring
    pub auto_login_enabled: bool,

    /// Enable platform keyring for password storage
    pub keyring_enabled: bool,

    /// List of recently used identities (for quick access)
    pub recent_identities: Vec<RecentIdentity>,

    /// Thread IDs that are pinned to the top of the thread list (max 5)
    #[serde(default)]
    pub pinned_threads: Vec<String>,

    /// UI preferences
    pub ui_preferences: UiPreferences,

    /// Configuration version for migration
    pub version: u32,

    // Beta features
    /// Whether the onboarding tour has been completed
    #[serde(default)]
    pub onboarding_completed: bool,

    /// Version when onboarding tour was last shown/completed
    #[serde(default)]
    pub onboarding_version: Option<String>,

    /// Whether to automatically check for updates (defaults to true)
    #[serde(default = "default_auto_update_enabled")]
    pub auto_update_enabled: bool,

    /// ISO 8601 timestamp of last update check
    #[serde(default)]
    pub last_update_check: Option<String>,

    /// Version to skip update prompts for
    #[serde(default)]
    pub skip_version: Option<String>,
}

/// Information about a recently used identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentIdentity {
    pub four_words: String,
    pub display_name: String,
    pub last_used: u64,
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: Theme,
    pub language: String,
    pub show_identity_picker_on_startup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            last_identity: None,
            auto_login_enabled: true,
            keyring_enabled: true,
            recent_identities: Vec::new(),
            pinned_threads: Vec::new(),
            ui_preferences: UiPreferences::default(),
            version: 1,
            onboarding_completed: false,
            onboarding_version: None,
            auto_update_enabled: true,
            last_update_check: None,
            skip_version: None,
        }
    }
}

impl AppConfig {
    /// Determines if onboarding should be shown based on completion status and version.
    ///
    /// Returns true if:
    /// - Onboarding has never been completed, OR
    /// - Onboarding was completed for a different version
    pub fn should_show_onboarding(&self, current_version: &str) -> bool {
        if !self.onboarding_completed {
            return true;
        }
        match &self.onboarding_version {
            Some(v) => v != current_version,
            None => true,
        }
    }

    /// Marks onboarding as completed for the given version.
    pub fn mark_onboarding_completed(&mut self, version: &str) {
        self.onboarding_completed = true;
        self.onboarding_version = Some(version.to_string());
    }

    /// Determines if an update check should be performed.
    ///
    /// Returns true if:
    /// - auto_update_enabled is true, AND
    /// - Either no previous check exists, or enough time has passed
    ///
    /// Note: Time comparison is the caller's responsibility. This method returns
    /// true if auto-update is enabled and there's no previous check timestamp.
    pub fn should_check_update(&self) -> bool {
        self.auto_update_enabled && self.last_update_check.is_none()
    }

    /// Records that an update check was performed at the given ISO 8601 timestamp.
    pub fn record_update_check(&mut self, timestamp: &str) {
        self.last_update_check = Some(timestamp.to_string());
    }

    /// Sets a version to skip update prompts for.
    pub fn set_skip_version(&mut self, version: &str) {
        self.skip_version = Some(version.to_string());
    }

    /// Checks if the given version should be skipped.
    pub fn should_skip_version(&self, version: &str) -> bool {
        self.skip_version.as_deref() == Some(version)
    }

    /// Clears the skip version setting.
    pub fn clear_skip_version(&mut self) {
        self.skip_version = None;
    }
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            language: "en".to_string(),
            show_identity_picker_on_startup: true,
        }
    }
}

/// Manages application configuration
pub struct AppConfigManager {
    config_path: PathBuf,
    config: AppConfig,
}

impl AppConfigManager {
    /// Create new config manager
    pub async fn new(config_dir: PathBuf) -> Result<Self> {
        // Ensure config directory exists
        fs::create_dir_all(&config_dir)
            .await
            .context("Failed to create config directory")?;

        let config_path = config_dir.join("app_config.json");

        // Load existing config or create default
        let config = if config_path.exists() {
            let data = fs::read_to_string(&config_path)
                .await
                .context("Failed to read config file")?;
            serde_json::from_str(&data).context("Failed to parse config file")?
        } else {
            AppConfig::default()
        };

        Ok(Self {
            config_path,
            config,
        })
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// Get mutable reference to configuration
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Update last used identity
    pub async fn set_last_identity(
        &mut self,
        four_words: String,
        display_name: String,
    ) -> Result<()> {
        self.config.last_identity = Some(four_words.clone());

        // Update or add to recent identities
        let now = current_timestamp();
        if let Some(existing) = self
            .config
            .recent_identities
            .iter_mut()
            .find(|r| r.four_words == four_words)
        {
            existing.last_used = now;
            existing.display_name = display_name;
        } else {
            self.config.recent_identities.push(RecentIdentity {
                four_words,
                display_name,
                last_used: now,
            });
        }

        // Keep only 10 most recent
        self.config
            .recent_identities
            .sort_by(|a, b| b.last_used.cmp(&a.last_used));
        self.config.recent_identities.truncate(10);

        self.save().await
    }

    /// Enable/disable auto-login
    pub async fn set_auto_login(&mut self, enabled: bool) -> Result<()> {
        self.config.auto_login_enabled = enabled;
        self.save().await
    }

    /// Enable/disable keyring
    pub async fn set_keyring_enabled(&mut self, enabled: bool) -> Result<()> {
        self.config.keyring_enabled = enabled;
        self.save().await
    }

    /// Update UI preferences
    pub async fn set_ui_preferences(&mut self, preferences: UiPreferences) -> Result<()> {
        self.config.ui_preferences = preferences;
        self.save().await
    }

    /// Clear last identity (logout all)
    pub async fn clear_last_identity(&mut self) -> Result<()> {
        self.config.last_identity = None;
        self.save().await
    }

    /// Remove identity from recent list
    pub async fn remove_recent_identity(&mut self, four_words: &str) -> Result<()> {
        self.config
            .recent_identities
            .retain(|r| r.four_words != four_words);

        // If this was the last identity, clear it
        if self.config.last_identity.as_deref() == Some(four_words) {
            self.config.last_identity = None;
        }

        self.save().await
    }

    /// Maximum number of pinned threads allowed
    pub const MAX_PINNED_THREADS: usize = 5;

    /// Pin a thread to the top of the thread list.
    ///
    /// Returns `Ok(true)` if the thread was pinned, `Ok(false)` if it was already pinned.
    /// Returns an error if the maximum number of pinned threads would be exceeded.
    pub async fn pin_thread(&mut self, thread_id: &str) -> Result<bool> {
        // Check if already pinned
        if self.config.pinned_threads.contains(&thread_id.to_string()) {
            return Ok(false);
        }

        // Check if we're at the limit
        if self.config.pinned_threads.len() >= Self::MAX_PINNED_THREADS {
            anyhow::bail!(
                "Maximum number of pinned threads ({}) reached",
                Self::MAX_PINNED_THREADS
            );
        }

        self.config.pinned_threads.push(thread_id.to_string());
        self.save().await?;
        Ok(true)
    }

    /// Unpin a thread from the top of the thread list.
    ///
    /// Returns `Ok(true)` if the thread was unpinned, `Ok(false)` if it wasn't pinned.
    pub async fn unpin_thread(&mut self, thread_id: &str) -> Result<bool> {
        let initial_len = self.config.pinned_threads.len();
        self.config.pinned_threads.retain(|id| id != thread_id);

        if self.config.pinned_threads.len() < initial_len {
            self.save().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Check if a thread is pinned.
    pub fn is_thread_pinned(&self, thread_id: &str) -> bool {
        self.config.pinned_threads.contains(&thread_id.to_string())
    }

    /// Get the list of pinned thread IDs.
    pub fn get_pinned_threads(&self) -> &[String] {
        &self.config.pinned_threads
    }

    /// Save configuration to disk
    async fn save(&self) -> Result<()> {
        let json =
            serde_json::to_string_pretty(&self.config).context("Failed to serialize config")?;

        fs::write(&self.config_path, json)
            .await
            .context("Failed to write config file")?;

        Ok(())
    }
}

/// Get current timestamp in seconds since UNIX epoch
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        assert!(manager.get_config().last_identity.is_none());
        assert!(manager.get_config().auto_login_enabled);
        assert!(manager.get_config().keyring_enabled);
    }

    #[tokio::test]
    async fn test_set_last_identity() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        manager
            .set_last_identity("ocean-forest-moon-star".to_string(), "Alice".to_string())
            .await
            .unwrap();

        assert_eq!(
            manager.get_config().last_identity,
            Some("ocean-forest-moon-star".to_string())
        );
        assert_eq!(manager.get_config().recent_identities.len(), 1);
        assert_eq!(
            manager.get_config().recent_identities[0].four_words,
            "ocean-forest-moon-star"
        );
    }

    #[tokio::test]
    async fn test_recent_identities_limit() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Add 15 identities
        for i in 0..15 {
            manager
                .set_last_identity(format!("identity-{}", i), format!("User {}", i))
                .await
                .unwrap();
        }

        // Should keep only 10 most recent
        assert_eq!(manager.get_config().recent_identities.len(), 10);
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Create and save config
        {
            let mut manager = AppConfigManager::new(config_dir.clone()).await.unwrap();

            manager
                .set_last_identity("test-identity".to_string(), "Test User".to_string())
                .await
                .unwrap();

            manager.set_auto_login(false).await.unwrap();
        }

        // Load config in new manager
        let manager = AppConfigManager::new(config_dir).await.unwrap();

        assert_eq!(
            manager.get_config().last_identity,
            Some("test-identity".to_string())
        );
        assert!(!manager.get_config().auto_login_enabled);
    }

    #[tokio::test]
    async fn test_remove_recent_identity() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Add identity
        manager
            .set_last_identity("test-identity".to_string(), "Test User".to_string())
            .await
            .unwrap();

        assert_eq!(manager.get_config().recent_identities.len(), 1);

        // Remove it
        manager
            .remove_recent_identity("test-identity")
            .await
            .unwrap();

        assert_eq!(manager.get_config().recent_identities.len(), 0);
        assert!(manager.get_config().last_identity.is_none());
    }

    #[tokio::test]
    async fn test_pin_thread() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Initially no pinned threads
        assert!(manager.get_pinned_threads().is_empty());

        // Pin a thread
        let result = manager.pin_thread("thread-1").await.unwrap();
        assert!(result); // Was pinned
        assert!(manager.is_thread_pinned("thread-1"));
        assert_eq!(manager.get_pinned_threads().len(), 1);

        // Pin same thread again - should return false
        let result = manager.pin_thread("thread-1").await.unwrap();
        assert!(!result); // Already pinned
        assert_eq!(manager.get_pinned_threads().len(), 1);
    }

    #[tokio::test]
    async fn test_unpin_thread() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Pin a thread
        manager.pin_thread("thread-1").await.unwrap();
        assert!(manager.is_thread_pinned("thread-1"));

        // Unpin it
        let result = manager.unpin_thread("thread-1").await.unwrap();
        assert!(result); // Was unpinned
        assert!(!manager.is_thread_pinned("thread-1"));

        // Unpin again - should return false
        let result = manager.unpin_thread("thread-1").await.unwrap();
        assert!(!result); // Wasn't pinned
    }

    #[tokio::test]
    async fn test_pin_thread_limit() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Pin up to the limit
        for i in 0..AppConfigManager::MAX_PINNED_THREADS {
            manager.pin_thread(&format!("thread-{}", i)).await.unwrap();
        }

        assert_eq!(
            manager.get_pinned_threads().len(),
            AppConfigManager::MAX_PINNED_THREADS
        );

        // Try to pin one more - should fail
        let result = manager.pin_thread("thread-overflow").await;
        assert!(result.is_err());
        assert_eq!(
            manager.get_pinned_threads().len(),
            AppConfigManager::MAX_PINNED_THREADS
        );
    }

    #[tokio::test]
    async fn test_pinned_threads_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Create and pin threads
        {
            let mut manager = AppConfigManager::new(config_dir.clone()).await.unwrap();
            manager.pin_thread("thread-a").await.unwrap();
            manager.pin_thread("thread-b").await.unwrap();
        }

        // Load in new manager and verify persistence
        let manager = AppConfigManager::new(config_dir).await.unwrap();
        assert!(manager.is_thread_pinned("thread-a"));
        assert!(manager.is_thread_pinned("thread-b"));
        assert_eq!(manager.get_pinned_threads().len(), 2);
    }

    // Beta feature tests

    #[test]
    fn test_should_show_onboarding_first_run() {
        let config = AppConfig::default();
        assert!(
            config.should_show_onboarding("1.0.0"),
            "Onboarding should show on first run"
        );
    }

    #[test]
    fn test_should_show_onboarding_after_version_upgrade() {
        let mut config = AppConfig::default();
        config.mark_onboarding_completed("1.0.0");

        assert!(
            !config.should_show_onboarding("1.0.0"),
            "Onboarding should not show for same version"
        );
        assert!(
            config.should_show_onboarding("1.1.0"),
            "Onboarding should show for new version"
        );
    }

    #[test]
    fn test_mark_onboarding_completed() {
        let mut config = AppConfig::default();
        config.mark_onboarding_completed("2.0.0");

        assert!(config.onboarding_completed);
        assert_eq!(config.onboarding_version, Some("2.0.0".to_string()));
    }

    #[test]
    fn test_should_check_update_default() {
        let config = AppConfig::default();
        assert!(
            config.should_check_update(),
            "Should check update when enabled and no previous check"
        );
    }

    #[test]
    fn test_should_check_update_disabled() {
        let config = AppConfig {
            auto_update_enabled: false,
            ..Default::default()
        };
        assert!(
            !config.should_check_update(),
            "Should not check when disabled"
        );
    }

    #[test]
    fn test_should_check_update_has_previous() {
        let mut config = AppConfig::default();
        config.record_update_check("2026-01-23T12:00:00Z");
        assert!(
            !config.should_check_update(),
            "Should not immediately check again after recent check"
        );
    }

    #[test]
    fn test_skip_version() {
        let mut config = AppConfig::default();
        assert!(!config.should_skip_version("1.5.0"));

        config.set_skip_version("1.5.0");
        assert!(config.should_skip_version("1.5.0"));
        assert!(!config.should_skip_version("1.6.0"));

        config.clear_skip_version();
        assert!(!config.should_skip_version("1.5.0"));
    }

    #[test]
    fn test_backward_compat_deserialize() {
        // Old config without new fields should deserialize with defaults
        let json = r#"{
            "last_identity": "test-words",
            "auto_login_enabled": true,
            "keyring_enabled": true,
            "recent_identities": [],
            "pinned_threads": [],
            "ui_preferences": {
                "theme": "System",
                "language": "en",
                "show_identity_picker_on_startup": true
            },
            "version": 1
        }"#;

        let config: AppConfig = serde_json::from_str(json).expect("Should deserialize old config");
        assert!(!config.onboarding_completed);
        assert!(config.auto_update_enabled);
        assert!(config.last_update_check.is_none());
        assert!(config.skip_version.is_none());
    }
}
