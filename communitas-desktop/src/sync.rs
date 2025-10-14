// Copyright (c) 2025 Saorsa Labs Limited
//
// Synchronization and delta fetching commands (placeholder)
//
// TODO: Implement with new gossip-based architecture

use crate::container::EngineState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::RwLock;

#[derive(Default, Serialize, Deserialize)]
pub struct TipWatcherState {
    handle: Option<String>,    // Placeholder - was JoinHandle
    cancel_tx: Option<String>, // Placeholder - was oneshot::Sender
    last_tip: Option<String>,  // Placeholder - was Tip
}

#[tauri::command]
pub async fn sync_start_tip_watcher(
    _app: AppHandle,
    _container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    _watcher: State<'_, Arc<RwLock<TipWatcherState>>>,
    _interval_ms: Option<u64>,
) -> Result<bool, String> {
    Err("Sync tip watcher not yet implemented with new architecture".to_string())
}

#[tauri::command]
pub async fn sync_stop_tip_watcher(
    _watcher: State<'_, Arc<RwLock<TipWatcherState>>>,
) -> Result<bool, String> {
    Err("Sync tip watcher not yet implemented with new architecture".to_string())
}

/// Delta fetcher over QUIC (IPv4-first)
#[tauri::command]
pub async fn sync_fetch_deltas(
    _app: AppHandle,
    _container: State<'_, Arc<RwLock<Option<EngineState>>>>,
    _rpk: State<'_, Arc<RwLock<crate::security::raw_spki::RawSpkiState>>>,
    _peer_addr: String,
) -> Result<u64, String> {
    Err("Delta fetching not yet implemented with new architecture".to_string())
}
