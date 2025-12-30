// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Self-update module for the Iced desktop application.
//!
//! Provides functionality to check for updates from GitHub releases
//! and update the application binary in the background.

use semver::Version;
use std::path::PathBuf;

/// Status of the update check.
#[derive(Debug, Clone)]
pub enum UpdateCheckResult {
    /// A new version is available.
    UpdateAvailable(UpdateInfo),
    /// Already running the latest version.
    UpToDate,
    /// Update check failed.
    Error(String),
}

/// Information about an available update.
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// The new version available.
    pub new_version: String,
    /// The current version.
    pub current_version: String,
    /// Release notes (if available).
    pub release_notes: Option<String>,
    /// Download URL for the release.
    pub download_url: Option<String>,
}

/// Status of the update download/install process.
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// Idle, no update in progress.
    Idle,
    /// Checking for updates.
    Checking,
    /// Update available, waiting for user confirmation.
    Available(UpdateInfo),
    /// Downloading update.
    Downloading {
        /// Download progress percentage (0-100).
        progress: u8,
    },
    /// Installing update.
    Installing,
    /// Update completed, restart required.
    Completed {
        /// The new version that was installed.
        new_version: String,
    },
    /// Update failed.
    Failed(String),
    /// User dismissed the update notification.
    Dismissed,
    /// User chose to skip this version.
    Skipped(String),
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Configuration for the update system.
#[derive(Debug, Clone)]
pub struct UpdateConfig {
    /// GitHub repository owner.
    pub repo_owner: String,
    /// GitHub repository name.
    pub repo_name: String,
    /// Binary name to look for in releases.
    pub bin_name: String,
    /// Current version of the application.
    pub current_version: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            repo_owner: std::env::var("COMMUNITAS_UPDATE_REPO_OWNER")
                .unwrap_or_else(|_| "saorsa-labs".to_string()),
            repo_name: std::env::var("COMMUNITAS_UPDATE_REPO_NAME")
                .unwrap_or_else(|_| "communitas".to_string()),
            bin_name: "communitas-iced".to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Check for available updates from GitHub releases.
///
/// This function is async and can be run in a background task.
pub async fn check_for_update(config: &UpdateConfig) -> UpdateCheckResult {
    // Run the blocking self_update check in a blocking task
    let config_clone = config.clone();

    match tokio::task::spawn_blocking(move || check_for_update_blocking(&config_clone)).await {
        Ok(result) => result,
        Err(e) => UpdateCheckResult::Error(format!("Task join error: {e}")),
    }
}

/// Blocking version of update check (used internally).
fn check_for_update_blocking(config: &UpdateConfig) -> UpdateCheckResult {
    use self_update::backends::github::Update;

    let mut update_config = Update::configure();
    let builder = update_config
        .repo_owner(&config.repo_owner)
        .repo_name(&config.repo_name)
        .bin_name(&config.bin_name)
        .current_version(&config.current_version)
        .no_confirm(true)
        .show_output(false);

    match builder.build() {
        Ok(updater) => {
            // Check if there's a newer version without downloading
            match updater.get_latest_release() {
                Ok(release) => {
                    let latest_version = release.version.trim_start_matches('v');

                    // Compare versions using semver
                    let current = match Version::parse(&config.current_version) {
                        Ok(v) => v,
                        Err(e) => {
                            return UpdateCheckResult::Error(format!(
                                "Failed to parse current version: {e}"
                            ));
                        }
                    };

                    let latest = match Version::parse(latest_version) {
                        Ok(v) => v,
                        Err(e) => {
                            return UpdateCheckResult::Error(format!(
                                "Failed to parse latest version: {e}"
                            ));
                        }
                    };

                    if latest > current {
                        UpdateCheckResult::UpdateAvailable(UpdateInfo {
                            new_version: latest_version.to_string(),
                            current_version: config.current_version.clone(),
                            release_notes: release.body.clone(),
                            download_url: release.assets.first().map(|a| a.download_url.clone()),
                        })
                    } else {
                        UpdateCheckResult::UpToDate
                    }
                }
                Err(e) => UpdateCheckResult::Error(format!("Failed to get latest release: {e}")),
            }
        }
        Err(e) => UpdateCheckResult::Error(format!("Failed to configure update: {e}")),
    }
}

/// Perform the update, downloading and installing the new version.
///
/// Returns the path to the backup of the old binary for rollback purposes.
pub async fn perform_update(config: &UpdateConfig) -> Result<UpdateResult, String> {
    let config_clone = config.clone();

    match tokio::task::spawn_blocking(move || perform_update_blocking(&config_clone)).await {
        Ok(result) => result,
        Err(e) => Err(format!("Task join error: {e}")),
    }
}

/// Result of a successful update.
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// The new version that was installed.
    pub new_version: String,
    /// Path to the backup of the old binary (for rollback).
    pub backup_path: Option<PathBuf>,
}

/// Blocking version of update (used internally).
fn perform_update_blocking(config: &UpdateConfig) -> Result<UpdateResult, String> {
    use self_update::backends::github::Update;

    // Create backup of current binary for rollback
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {e}"))?;

    let backup_path = current_exe.with_extension("backup");

    // Create backup
    if let Err(e) = std::fs::copy(&current_exe, &backup_path) {
        tracing::warn!("Failed to create backup: {e}");
        // Continue anyway - update is still possible
    }

    let mut update_config = Update::configure();
    let builder = update_config
        .repo_owner(&config.repo_owner)
        .repo_name(&config.repo_name)
        .bin_name(&config.bin_name)
        .current_version(&config.current_version)
        .no_confirm(true)
        .show_output(false);

    match builder.build() {
        Ok(updater) => match updater.update() {
            Ok(status) => {
                let new_version = status.version().to_string();
                tracing::info!("Update successful: {new_version}");
                Ok(UpdateResult {
                    new_version,
                    backup_path: Some(backup_path),
                })
            }
            Err(e) => {
                // Try to restore from backup on failure
                if backup_path.exists()
                    && let Err(restore_err) = std::fs::copy(&backup_path, &current_exe)
                {
                    tracing::error!("Failed to restore backup: {restore_err}");
                }
                Err(format!("Update failed: {e}"))
            }
        },
        Err(e) => Err(format!("Failed to configure update: {e}")),
    }
}

/// Rollback to the previous version using the backup.
pub fn rollback(backup_path: &PathBuf) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {e}"))?;

    if !backup_path.exists() {
        return Err("Backup file not found".to_string());
    }

    std::fs::copy(backup_path, &current_exe)
        .map_err(|e| format!("Failed to restore backup: {e}"))?;

    // Clean up backup file
    let _ = std::fs::remove_file(backup_path);

    tracing::info!("Rollback successful");
    Ok(())
}

/// Get the path where skipped versions are stored.
fn skipped_versions_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("communitas")
        .join("skipped_versions.txt")
}

/// Check if a version has been skipped by the user.
pub fn is_version_skipped(version: &str) -> bool {
    let path = skipped_versions_path();
    if let Ok(contents) = std::fs::read_to_string(&path) {
        contents.lines().any(|line| line.trim() == version)
    } else {
        false
    }
}

/// Mark a version as skipped.
pub fn skip_version(version: &str) -> Result<(), String> {
    let path = skipped_versions_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    // Append version to file
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open skipped versions file: {e}"))?;

    writeln!(file, "{version}")
        .map_err(|e| format!("Failed to write skipped version: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = UpdateConfig::default();
        assert_eq!(config.repo_owner, "saorsa-labs");
        assert_eq!(config.repo_name, "communitas");
        assert_eq!(config.bin_name, "communitas-iced");
        assert!(!config.current_version.is_empty());
    }

    #[test]
    fn test_update_status_default() {
        let status = UpdateStatus::default();
        assert!(matches!(status, UpdateStatus::Idle));
    }

    /// Integration test that checks update detection against GitHub releases API.
    /// Uses an artificially old version to ensure update is always detected.
    #[tokio::test]
    async fn test_update_check_github_api() {
        let config = UpdateConfig {
            repo_owner: "saorsa-labs".to_string(),
            repo_name: "communitas".to_string(),
            bin_name: "communitas-iced".to_string(),
            // Use an old version to ensure update is detected
            current_version: "0.0.1".to_string(),
        };

        let result = check_for_update(&config).await;

        match result {
            UpdateCheckResult::UpdateAvailable(info) => {
                // Verify we got valid update info
                assert!(!info.new_version.is_empty(), "new_version should not be empty");
                assert_eq!(info.current_version, "0.0.1");
                assert!(
                    info.new_version != "0.0.1",
                    "new_version should differ from current"
                );
                println!(
                    "Update available: {} -> {}",
                    info.current_version, info.new_version
                );
            }
            UpdateCheckResult::UpToDate => {
                panic!("Expected update to be available for version 0.0.1");
            }
            UpdateCheckResult::Error(e) => {
                // Network errors are OK in CI environments without internet
                if e.contains("network") || e.contains("connect") || e.contains("DNS") {
                    println!("Skipping test due to network error: {e}");
                } else {
                    panic!("Unexpected error: {e}");
                }
            }
        }
    }
}
