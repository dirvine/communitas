// Copyright (c) 2025 Saorsa Labs Limited
//
// Update Manager Service
//
// Handles automatic updates from GitHub releases with signature verification

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub checking: bool,
    pub error: Option<String>,
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self {
            available: false,
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            latest_version: None,
            download_url: None,
            release_notes: None,
            checking: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgress {
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percent: Option<f32>,
}

/// Update settings that can be configured by the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    /// Enable automatic updates
    pub auto_update_enabled: bool,
    /// Check frequency in hours
    pub check_frequency: u64,
    /// Update channel (stable/beta)
    pub update_channel: UpdateChannel,
    /// Last time updates were checked
    pub last_checked: Option<String>, // ISO 8601 timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_update_enabled: true,
            check_frequency: 24, // Daily by default
            update_channel: UpdateChannel::Stable,
            last_checked: None,
        }
    }
}

impl UpdateSettings {
    /// Get settings file path in app data directory
    fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
        app.path()
            .app_data_dir()
            .map(|p| p.join("update_settings.json"))
            .map_err(|e| format!("Failed to get app data dir: {}", e))
    }

    /// Load settings from disk
    pub async fn load(app: &AppHandle) -> Self {
        let path = match Self::settings_path(app) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Failed to get settings path: {}", e);
                return Self::default();
            }
        };

        match tokio::fs::read_to_string(&path).await {
            Ok(content) => match serde_json::from_str::<UpdateSettings>(&content) {
                Ok(settings) => {
                    tracing::info!("Loaded update settings from {:?}", path);
                    settings
                }
                Err(e) => {
                    tracing::warn!("Failed to parse settings: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::debug!("No settings file found ({}), using defaults", e);
                Self::default()
            }
        }
    }

    /// Save settings to disk
    pub async fn save(&self, app: &AppHandle) -> Result<(), String> {
        let path = Self::settings_path(app)?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        tokio::fs::write(&path, content)
            .await
            .map_err(|e| format!("Failed to write settings: {}", e))?;

        tracing::info!("Saved update settings to {:?}", path);
        Ok(())
    }
}

/// Shared update state
pub type UpdateState = Arc<RwLock<UpdateStatus>>;

/// Shared settings state
pub type SettingsState = Arc<RwLock<UpdateSettings>>;

/// Check for available updates
#[tauri::command]
pub async fn check_for_updates(
    app: AppHandle,
    state: State<'_, UpdateState>,
) -> Result<UpdateStatus, String> {
    tracing::info!("Checking for updates...");

    // Set checking status
    {
        let mut status = state.write().await;
        status.checking = true;
        status.error = None;
    }

    // Check for updates
    let result = match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                let status = UpdateStatus {
                    available: true,
                    current_version: update.current_version.clone(),
                    latest_version: Some(update.version.clone()),
                    download_url: Some(update.download_url.to_string()),
                    release_notes: update.body.clone(),
                    checking: false,
                    error: None,
                };

                tracing::info!(
                    "Update available: {} -> {}",
                    status.current_version,
                    status
                        .latest_version
                        .as_ref()
                        .unwrap_or(&"unknown".to_string())
                );

                Ok(status)
            }
            Ok(None) => {
                tracing::info!("No updates available");
                Ok(UpdateStatus {
                    checking: false,
                    ..Default::default()
                })
            }
            Err(e) => {
                tracing::error!("Error checking for updates: {}", e);
                Err(format!("Failed to check for updates: {}", e))
            }
        },
        Err(e) => {
            tracing::warn!("Updater not available: {}", e);
            Err(format!("Updater not configured: {}", e))
        }
    };

    // Update state
    let status = result.clone().unwrap_or_else(|e| UpdateStatus {
        checking: false,
        error: Some(e),
        ..Default::default()
    });

    *state.write().await = status.clone();

    result
}

/// Download and install update
#[tauri::command]
pub async fn install_update(app: AppHandle, _state: State<'_, UpdateState>) -> Result<(), String> {
    tracing::info!("Installing update...");

    let updater = app
        .updater()
        .map_err(|e| format!("Updater not configured: {}", e))?;

    // Check for update first
    let update = updater
        .check()
        .await
        .map_err(|e| format!("Failed to check for updates: {}", e))?
        .ok_or_else(|| "No update available".to_string())?;

    tracing::info!("Downloading update version {}", update.version);

    // Download and install
    // The updater will handle signature verification automatically
    update
        .download_and_install(
            |chunk_length, content_length| {
                let percent =
                    content_length.map(|total| (chunk_length as f32 / total as f32) * 100.0);

                tracing::debug!(
                    "Downloaded {} bytes ({}%)",
                    chunk_length,
                    percent
                        .map(|p| format!("{:.1}", p))
                        .unwrap_or_else(|| "?".to_string())
                );
            },
            || {
                tracing::info!("Download complete, installing...");
            },
        )
        .await
        .map_err(|e| format!("Failed to install update: {}", e))?;

    tracing::info!("Update installed successfully - restart required");

    Ok(())
}

/// Get current update status
#[tauri::command]
pub async fn get_update_status(state: State<'_, UpdateState>) -> Result<UpdateStatus, String> {
    Ok(state.read().await.clone())
}

/// Get update settings
#[tauri::command]
pub async fn get_update_settings(
    settings_state: State<'_, SettingsState>,
) -> Result<UpdateSettings, String> {
    Ok(settings_state.read().await.clone())
}

/// Save update settings
#[tauri::command]
pub async fn set_update_settings(
    app: AppHandle,
    settings: UpdateSettings,
    settings_state: State<'_, SettingsState>,
) -> Result<(), String> {
    tracing::info!("Updating settings: {:?}", settings);

    // Save to disk
    settings.save(&app).await?;

    // Update state
    *settings_state.write().await = settings.clone();

    // If auto-update is enabled, reschedule checks with new frequency
    if settings.auto_update_enabled {
        // Note: In a full implementation, we'd restart the scheduler here
        tracing::info!(
            "Auto-update enabled with {}h frequency",
            settings.check_frequency
        );
    }

    Ok(())
}

/// Set auto-update enabled/disabled
#[tauri::command]
pub async fn set_auto_update(
    app: AppHandle,
    enabled: bool,
    settings_state: State<'_, SettingsState>,
) -> Result<(), String> {
    let mut settings = settings_state.write().await;
    settings.auto_update_enabled = enabled;
    settings.save(&app).await?;
    tracing::info!(
        "Auto-update {}",
        if enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

/// Set update check frequency (in hours)
#[tauri::command]
pub async fn set_check_frequency(
    app: AppHandle,
    hours: u64,
    settings_state: State<'_, SettingsState>,
) -> Result<(), String> {
    let mut settings = settings_state.write().await;
    settings.check_frequency = hours;
    settings.save(&app).await?;
    tracing::info!("Update check frequency set to {} hours", hours);
    Ok(())
}

/// Set update channel (stable/beta)
#[tauri::command]
pub async fn set_update_channel(
    app: AppHandle,
    channel: UpdateChannel,
    settings_state: State<'_, SettingsState>,
) -> Result<(), String> {
    let mut settings = settings_state.write().await;
    settings.update_channel = channel.clone();
    settings.save(&app).await?;
    tracing::info!("Update channel set to {:?}", channel);
    Ok(())
}

/// Schedule automatic update checks
pub fn schedule_update_checks(app: AppHandle, state: UpdateState) {
    tokio::spawn(async move {
        // Check for updates every 6 hours
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(6 * 60 * 60));

        loop {
            interval.tick().await;

            tracing::debug!("Scheduled update check");

            // Manual update check without using tauri::State
            if let Ok(updater) = app.updater()
                && let Ok(Some(update)) = updater.check().await
            {
                let status = UpdateStatus {
                    available: true,
                    current_version: update.current_version.clone(),
                    latest_version: Some(update.version.clone()),
                    download_url: Some(update.download_url.to_string()),
                    release_notes: update.body.clone(),
                    checking: false,
                    error: None,
                };

                *state.write().await = status;
                tracing::info!("Scheduled check: Update available!");
            }
        }
    });
}
