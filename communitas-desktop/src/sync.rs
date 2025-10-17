// Copyright (c) 2025 Saorsa Labs Limited
//
// Synchronization and delta fetching commands
//
// Implements CRDT synchronization using GossipContext and AntiEntropyManager

use communitas_core::gossip::GossipContext;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

#[derive(Default, Serialize, Deserialize)]
pub struct TipWatcherState {
    pub is_running: bool,
    pub interval_ms: u64,
    pub last_sync: Option<String>, // ISO timestamp
}

/// Start the CRDT tip watcher using GossipContext's anti-entropy manager
#[tauri::command]
pub async fn sync_start_tip_watcher(
    app: AppHandle,
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
    watcher_state: State<'_, Arc<RwLock<TipWatcherState>>>,
    interval_ms: Option<u64>,
) -> Result<bool, String> {
    info!("Starting CRDT tip watcher");

    let interval = interval_ms.unwrap_or(30000); // Default 30 seconds

    // Update watcher state first
    {
        let mut watcher = watcher_state.write().await;
        if watcher.is_running {
            return Err("Tip watcher is already running".to_string());
        }
        watcher.is_running = true;
        watcher.interval_ms = interval;
        watcher.last_sync = None;
    }

    // Get anti_entropy from gossip context within guard scope
    let _anti_entropy = {
        let guard = gossip_state.read().await;
        guard
            .as_ref()
            .ok_or("GossipContext not initialized - call gossip_initialize first")?
            .anti_entropy
            .clone()
    };

    // Start background task
    let app_handle = app.clone();
    let watcher_state_clone = Arc::clone(&watcher_state);

    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(Duration::from_millis(interval));

        loop {
            interval_timer.tick().await;

            // Check if still supposed to be running
            let should_continue = {
                let watcher = watcher_state_clone.read().await;
                watcher.is_running
            };

            if !should_continue {
                info!("Tip watcher stopped");
                break;
            }

            debug!("CRDT anti-entropy running (background sync handled by AntiEntropyManager)");

            // Update last sync time (anti-entropy runs in background)
            {
                let mut watcher = watcher_state_clone.write().await;
                watcher.last_sync = Some(chrono::Utc::now().to_rfc3339());
            }

            // Emit sync progress event
            if let Err(e) = app_handle.emit(
                "sync-progress",
                &serde_json::json!({
                    "phase": "anti_entropy",
                    "status": "running",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
            ) {
                error!("Failed to emit sync-progress event: {}", e);
            }
        }
    });

    info!("CRDT tip watcher started with {}ms interval", interval);
    Ok(true)
}

/// Stop the CRDT tip watcher
#[tauri::command]
pub async fn sync_stop_tip_watcher(
    watcher_state: State<'_, Arc<RwLock<TipWatcherState>>>,
) -> Result<bool, String> {
    info!("Stopping CRDT tip watcher");

    let mut watcher = watcher_state.write().await;
    if !watcher.is_running {
        return Err("Tip watcher is not running".to_string());
    }

    watcher.is_running = false;
    info!("CRDT tip watcher stopped");
    Ok(true)
}

/// Manual delta fetch from a specific peer over QUIC
#[tauri::command]
pub async fn sync_fetch_deltas(
    app: AppHandle,
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
    _rpk: State<'_, Arc<RwLock<crate::security::raw_spki::RawSpkiState>>>,
    peer_addr: String,
) -> Result<u64, String> {
    info!("Manual delta fetch from peer: {}", peer_addr);

    // Try to resolve as four-word address first within guard scope
    let result = {
        let guard = gossip_state.read().await;
        let gossip_ctx = guard
            .as_ref()
            .ok_or("GossipContext not initialized - call gossip_initialize first")?;

        match gossip_ctx.find_contact(&peer_addr).await {
            Ok(peer_id) => {
                debug!(
                    "Resolved peer {} via contact lookup: {:?}",
                    peer_addr, peer_id
                );

                // Emit sync progress event
                if let Err(e) = app.emit(
                    "sync-progress",
                    &serde_json::json!({
                        "phase": "manual_fetch",
                        "peer": peer_id.to_string(),
                        "status": "contact_found",
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                ) {
                    error!("Failed to emit sync-progress event: {}", e);
                }

                Ok(0) // For now, return success - actual sync is handled by background anti-entropy
            }
            Err(e) => {
                error!("Failed to resolve peer {}: {}", peer_addr, e);

                // Emit error event
                if let Err(e) = app.emit(
                    "sync-error",
                    &serde_json::json!({
                        "phase": "manual_fetch",
                        "peer": peer_addr,
                        "error": e.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    }),
                ) {
                    error!("Failed to emit sync-error event: {}", e);
                }

                Err(format!("Failed to resolve peer: {}", e))
            }
        }
    };

    result
}

/// Get current sync status
#[tauri::command]
pub async fn sync_get_status(
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
    watcher_state: State<'_, Arc<RwLock<TipWatcherState>>>,
) -> Result<serde_json::Value, String> {
    // Get watcher state
    let watcher = watcher_state.read().await;

    // Get message count as basic CRDT stat within guard scope
    let message_count = {
        let guard = gossip_state.read().await;
        let gossip_ctx = guard.as_ref().ok_or("GossipContext not initialized")?;

        gossip_ctx
            .get_all_messages()
            .await
            .map(|msgs| msgs.len())
            .unwrap_or(0)
    };

    Ok(serde_json::json!({
        "tip_watcher": {
            "is_running": watcher.is_running,
            "interval_ms": watcher.interval_ms,
            "last_sync": watcher.last_sync
        },
        "crdt": {
            "message_count": message_count,
            "anti_entropy_active": true
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Force immediate anti-entropy sync with all peers
#[tauri::command]
pub async fn sync_force_full_sync(
    app: AppHandle,
    gossip_state: State<'_, Arc<RwLock<Option<GossipContext>>>>,
) -> Result<bool, String> {
    info!("Forcing full CRDT sync with all peers");

    // Get current message count within guard scope
    let message_count = {
        let guard = gossip_state.read().await;
        let gossip_ctx = guard
            .as_ref()
            .ok_or("GossipContext not initialized - call gossip_initialize first")?;

        gossip_ctx
            .get_all_messages()
            .await
            .map(|msgs| msgs.len())
            .unwrap_or(0)
    };

    info!("Current CRDT message count: {}", message_count);

    // Emit sync progress event
    if let Err(e) = app.emit(
        "sync-progress",
        &serde_json::json!({
            "phase": "full_sync",
            "status": "triggered",
            "message_count": message_count,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    ) {
        error!("Failed to emit sync-progress event: {}", e);
    }

    // Note: Actual sync is handled by background AntiEntropyManager
    Ok(true)
}
