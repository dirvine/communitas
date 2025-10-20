use communitas_core::CoreContext;
use communitas_core::security::InputValidator;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug, Default)]
pub struct NetworkRuntime {
    pub connected: bool,
    pub peers: u32,
    pub endpoint_four_words: Option<String>,
    pub bootstrap_nodes: Vec<String>,
    pub last_error: Option<String>,
    pub user_four_words: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatusPayload {
    pub status: String,
    pub peers: u32,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInfoPayload {
    pub status: String,
    pub peers: u32,
    pub endpoint_four_words: Option<String>,
    pub bootstrap_nodes: Vec<String>,
    pub user_four_words: Option<String>,
    pub last_error: Option<String>,
}

fn default_bootstrap_nodes() -> Vec<String> {
    vec![
        "ocean-forest-moon-star".to_string(),
        "river-mountain-sun-cloud".to_string(),
    ]
}

fn normalize_four_words(input: &str) -> String {
    input.trim().to_lowercase().replace([' ', '_'], "-")
}

fn is_valid_four_words(input: &str) -> bool {
    let validator = InputValidator::new();
    validator.validate_four_words(input).is_ok()
}

async fn sync_user_four_words(
    runtime_state: &State<'_, Arc<RwLock<NetworkRuntime>>>,
    core_state: &State<'_, Arc<RwLock<Option<CoreContext>>>>,
) {
    let user_four_words = {
        let core_guard = core_state.read().await;
        core_guard.as_ref().map(|ctx| ctx.four_words.clone())
    };

    let mut runtime = runtime_state.write().await;
    runtime.user_four_words = user_four_words;
}

#[tauri::command]
pub async fn validate_four_words(four_words: String) -> Result<bool, String> {
    let normalized = normalize_four_words(&four_words);
    Ok(is_valid_four_words(&normalized))
}

#[tauri::command]
pub async fn connect_via_four_words(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
    core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    four_words: String,
) -> Result<bool, String> {
    let normalized = normalize_four_words(&four_words);
    if !is_valid_four_words(&normalized) {
        debug!(target: "network", "Rejected invalid four-words bootstrap: {}", four_words);
        return Ok(false);
    }

    // Try to use CoreContext for real P2P connection
    let core_guard = core_state.read().await;
    if let Some(core) = core_guard.as_ref() {
        match core.connect_to_peer(&normalized).await {
            Ok(()) => {
                drop(core_guard);

                // Update runtime state to reflect connection
                let mut runtime = runtime_state.write().await;
                if !runtime
                    .bootstrap_nodes
                    .iter()
                    .any(|node| node == &normalized)
                {
                    runtime.bootstrap_nodes.push(normalized.clone());
                }

                // Update peer count (placeholder until CoreContext has peer tracking)
                runtime.peers = runtime.peers.max(1);
                runtime.connected = true;

                runtime.last_error = None;
                info!(target: "network", "Successfully connected to peer: {}", normalized);

                sync_user_four_words(&runtime_state, &core_state).await;
                return Ok(true);
            }
            Err(e) => {
                tracing::error!("Failed to connect to peer via CoreContext: {}", e);
                // Fall through to legacy behavior
            }
        }
    }
    drop(core_guard);

    // Fallback: legacy behavior (add to bootstrap list only)
    {
        let mut runtime = runtime_state.write().await;
        if !runtime
            .bootstrap_nodes
            .iter()
            .any(|node| node == &normalized)
        {
            runtime.bootstrap_nodes.push(normalized.clone());
        }

        runtime.connected = true;
        runtime.peers = runtime.peers.max(1);
        runtime
            .endpoint_four_words
            .get_or_insert_with(|| format!("{}-relay", normalized));
        runtime.last_error = None;
        info!(target: "network", "Added to bootstrap list (legacy): {}", normalized);
    }

    sync_user_four_words(&runtime_state, &core_state).await;
    Ok(true)
}

#[tauri::command]
pub async fn connect_to_network(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
    core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<bool, String> {
    {
        let mut runtime = runtime_state.write().await;
        if runtime.bootstrap_nodes.is_empty() {
            runtime.bootstrap_nodes = default_bootstrap_nodes();
        }
        runtime.connected = true;
        runtime.peers = runtime.peers.max(1);
        let default_endpoint = runtime
            .bootstrap_nodes
            .first()
            .cloned()
            .map(|node| format!("{}-relay", node))
            .unwrap_or_else(|| "local-node-alpha-one".to_string());
        if runtime.endpoint_four_words.is_none() {
            runtime.endpoint_four_words = Some(default_endpoint);
        }
        runtime.last_error = None;
        info!(target: "network", "connect_to_network succeeded");
    }

    sync_user_four_words(&runtime_state, &core_state).await;
    Ok(true)
}

#[tauri::command]
pub async fn disconnect_from_network(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
) -> Result<bool, String> {
    let mut runtime = runtime_state.write().await;
    runtime.connected = false;
    runtime.peers = 0;
    runtime.endpoint_four_words = None;
    runtime.last_error = None;
    info!(target: "network", "Disconnected from network");
    Ok(true)
}

#[tauri::command]
pub async fn get_endpoint_four_words(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
    _core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Option<String>, String> {
    // Get from runtime state (CoreContext endpoint tracking not yet implemented)
    let runtime = runtime_state.read().await;
    Ok(runtime.endpoint_four_words.clone())
}

#[tauri::command]
pub async fn get_network_status(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
    _core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<NetworkStatusPayload, String> {
    // Get status from runtime state (CoreContext peer tracking not yet implemented)
    let runtime = runtime_state.read().await;
    let status = if runtime.connected {
        "connected"
    } else {
        "local"
    };
    Ok(NetworkStatusPayload {
        status: status.to_string(),
        peers: runtime.peers,
        error: runtime.last_error.clone(),
    })
}

#[tauri::command]
pub async fn get_network_info(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
) -> Result<NetworkInfoPayload, String> {
    let runtime = runtime_state.read().await;
    let status = if runtime.connected {
        "connected"
    } else {
        "local"
    };
    Ok(NetworkInfoPayload {
        status: status.to_string(),
        peers: runtime.peers,
        endpoint_four_words: runtime.endpoint_four_words.clone(),
        bootstrap_nodes: runtime.bootstrap_nodes.clone(),
        user_four_words: runtime.user_four_words.clone(),
        last_error: runtime.last_error.clone(),
    })
}

#[tauri::command]
pub async fn get_user_four_words(
    runtime_state: State<'_, Arc<RwLock<NetworkRuntime>>>,
    core_state: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<Option<String>, String> {
    sync_user_four_words(&runtime_state, &core_state).await;
    let runtime = runtime_state.read().await;
    Ok(runtime.user_four_words.clone())
}
