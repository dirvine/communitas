// Copyright (c) 2025 Saorsa Labs Limited
//
// Update Manager Service
//
// Handles automatic updates from GitHub releases with signature verification

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
use tauri_plugin_updater::UpdaterExt;
use tokio::sync::RwLock;
use std::sync::Arc;

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

/// Shared update state
pub type UpdateState = Arc<RwLock<UpdateStatus>>;

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
        Ok(updater) => {
            match updater.check().await {
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
                        status.latest_version.as_ref().unwrap_or(&"unknown".to_string())
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
            }
        }
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
pub async fn install_update(
    app: AppHandle,
    _state: State<'_, UpdateState>,
) -> Result<(), String> {
    tracing::info!("Installing update...");

    let updater = app.updater()
        .map_err(|e| format!("Updater not configured: {}", e))?;

    // Check for update first
    let update = updater.check().await
        .map_err(|e| format!("Failed to check for updates: {}", e))?
        .ok_or_else(|| "No update available".to_string())?;

    tracing::info!("Downloading update version {}", update.version);

    // Download and install
    // The updater will handle signature verification automatically
    update.download_and_install(
        |chunk_length, content_length| {
            let percent = content_length
                .map(|total| (chunk_length as f32 / total as f32) * 100.0);

            tracing::debug!(
                "Downloaded {} bytes ({}%)",
                chunk_length,
                percent.map(|p| format!("{:.1}", p)).unwrap_or_else(|| "?".to_string())
            );
        },
        || {
            tracing::info!("Download complete, installing...");
        }
    ).await
    .map_err(|e| format!("Failed to install update: {}", e))?;

    tracing::info!("Update installed successfully - restart required");

    Ok(())
}

/// Get current update status
#[tauri::command]
pub async fn get_update_status(
    state: State<'_, UpdateState>,
) -> Result<UpdateStatus, String> {
    Ok(state.read().await.clone())
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
            if let Ok(updater) = app.updater() {
                if let Ok(Some(update)) = updater.check().await {
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
        }
    });
}
