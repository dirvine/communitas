//! Application Configuration Management
//!
//! Manages app-level configuration separate from encrypted vaults.
//! Stores preferences like last used identity, UI settings, etc.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Application-level configuration (unencrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Last successfully authenticated identity
    pub last_identity: Option<String>,

    /// Enable auto-login using platform keyring
    pub auto_login_enabled: bool,

    /// Enable platform keyring for password storage
    pub keyring_enabled: bool,

    /// Enable biometric authentication (Touch ID, Face ID, Windows Hello)
    pub biometric_enabled: bool,

    /// List of recently used identities (for quick access)
    pub recent_identities: Vec<RecentIdentity>,

    /// UI preferences
    pub ui_preferences: UiPreferences,

    /// Configuration version for migration
    pub version: u32,
}

/// Information about a recently used identity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentIdentity {
    pub four_words: String,
    pub display_name: String,
    pub last_used: u64,
    pub has_passkey: bool,
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
            biometric_enabled: true,
            recent_identities: Vec::new(),
            ui_preferences: UiPreferences::default(),
            version: 1,
        }
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
                has_passkey: false,
            });
        }

        // Keep only 10 most recent
        self.config
            .recent_identities
            .sort_by(|a, b| b.last_used.cmp(&a.last_used));
        self.config.recent_identities.truncate(10);

        self.save().await
    }

    /// Mark identity as having passkey
    pub async fn set_identity_has_passkey(
        &mut self,
        four_words: &str,
        has_passkey: bool,
    ) -> Result<()> {
        if let Some(identity) = self
            .config
            .recent_identities
            .iter_mut()
            .find(|r| r.four_words == four_words)
        {
            identity.has_passkey = has_passkey;
            self.save().await?;
        }
        Ok(())
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

    /// Enable/disable biometric authentication
    pub async fn set_biometric_enabled(&mut self, enabled: bool) -> Result<()> {
        self.config.biometric_enabled = enabled;
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
    async fn test_passkey_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = AppConfigManager::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();

        // Add identity
        manager
            .set_last_identity("test-identity".to_string(), "Test User".to_string())
            .await
            .unwrap();

        // Initially no passkey
        assert!(!manager.get_config().recent_identities[0].has_passkey);

        // Set passkey
        manager
            .set_identity_has_passkey("test-identity", true)
            .await
            .unwrap();

        assert!(manager.get_config().recent_identities[0].has_passkey);
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
}
