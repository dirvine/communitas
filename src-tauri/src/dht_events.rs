// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use saorsa_core::dht::DHT;
use tauri::{AppHandle, Emitter};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// ============= Event Types =============

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DHTSyncEvent {
    // Organization events
    OrganizationCreated { organization: serde_json::Value },
    OrganizationUpdated { organization: serde_json::Value },
    OrganizationDeleted { id: String },
    
    // Group events
    GroupCreated { group: serde_json::Value },
    GroupUpdated { group: serde_json::Value },
    GroupDeleted { id: String },
    
    // Project events
    ProjectCreated { project: serde_json::Value },
    ProjectUpdated { project: serde_json::Value },
    ProjectDeleted { id: String },
    
    // Member events
    MemberJoined { entity_type: String, entity_id: String, member: serde_json::Value },
    MemberLeft { entity_type: String, entity_id: String, user_id: String },
    MemberRoleChanged { entity_type: String, entity_id: String, user_id: String, new_role: String },
    
    // File events
    FileUploaded { project_id: String, file: serde_json::Value },
    FileDeleted { project_id: String, file_id: String },
    FileShared { file_id: String, shared_with: Vec<String> },
    
    // Connection events
    PeerConnected { peer_id: String, address: String },
    PeerDisconnected { peer_id: String },
    NetworkStatusChanged { status: NetworkStatus },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub connected: bool,
    pub peer_count: usize,
    pub syncing: bool,
    pub last_sync: Option<DateTime<Utc>>,
}

// ============= Event Listener =============

pub struct DHTEventListener {
    dht: Arc<RwLock<DHT>>,
    app_handle: AppHandle,
    event_tx: mpsc::Sender<DHTSyncEvent>,
    event_rx: Option<mpsc::Receiver<DHTSyncEvent>>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // entity_id -> [user_ids]
    is_running: Arc<RwLock<bool>>,
}

impl std::fmt::Debug for DHTEventListener {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DHTEventListener")
            .field("is_running", &self.is_running)
            .field("subscriptions_count", &"<async>")
            .finish()
    }
}

impl DHTEventListener {
    pub fn new(dht: Arc<RwLock<DHT>>, app_handle: AppHandle) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        Self {
            dht,
            app_handle,
            event_tx,
            event_rx: Some(event_rx),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// Start listening for DHT events
    pub async fn start(&mut self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("DHT event listener is already running");
            return Ok(());
        }
        *is_running = true;
        drop(is_running);
        
        info!("Starting DHT event listener");
        
        // Take the receiver (can only have one)
        let mut event_rx = self.event_rx.take()
            .ok_or_else(|| anyhow::anyhow!("Event receiver already taken"))?;
        
        let app_handle = self.app_handle.clone();
        let dht = self.dht.clone();
        let subscriptions = self.subscriptions.clone();
        let is_running = self.is_running.clone();
        
        // Spawn the event processing task
        tokio::spawn(async move {
            info!("DHT event processor started");
            
            while *is_running.read().await {
                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        debug!("Processing DHT event: {:?}", event);
                        Self::process_event(&app_handle, event, &subscriptions).await;
                    }
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                        // Periodic check for DHT changes
                        Self::poll_dht_changes(&dht, &app_handle).await;
                    }
                }
            }
            
            info!("DHT event processor stopped");
        });
        
        // Start monitoring DHT for changes
        self.start_dht_monitor().await?;
        
        Ok(())
    }
    
    /// Stop listening for DHT events
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping DHT event listener");
        let mut is_running = self.is_running.write().await;
        *is_running = false;
        Ok(())
    }
    
    /// Subscribe to events for a specific entity
    pub async fn subscribe(&self, entity_id: String, user_id: String) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        subs.entry(entity_id.clone())
            .or_insert_with(Vec::new)
            .push(user_id.clone());
        
        debug!("User {} subscribed to entity {}", user_id, entity_id);
        Ok(())
    }
    
    /// Unsubscribe from events for a specific entity
    pub async fn unsubscribe(&self, entity_id: &str, user_id: &str) -> Result<()> {
        let mut subs = self.subscriptions.write().await;
        if let Some(users) = subs.get_mut(entity_id) {
            users.retain(|id| id != user_id);
            if users.is_empty() {
                subs.remove(entity_id);
            }
        }
        
        debug!("User {} unsubscribed from entity {}", user_id, entity_id);
        Ok(())
    }
    
    /// Emit an event to the event queue
    pub async fn emit_event(&self, event: DHTSyncEvent) -> Result<()> {
        self.event_tx.send(event).await
            .map_err(|e| anyhow::anyhow!("Failed to send event: {}", e))?;
        Ok(())
    }
    
    /// Process a single event
    async fn process_event(
        app_handle: &AppHandle,
        event: DHTSyncEvent,
        subscriptions: &Arc<RwLock<HashMap<String, Vec<String>>>>,
    ) {
        // Determine which users should receive this event
        let relevant_users = Self::get_relevant_users(&event, subscriptions).await;
        
        // Emit to frontend via Tauri
        match serde_json::to_value(&event) {
            Ok(event_data) => {
                // Broadcast to all windows
                if let Err(e) = app_handle.emit("dht-sync-event", event_data.clone()) {
                    warn!("Failed to emit DHT event to all windows: {}", e);
                }
                
                // Also emit targeted events for specific users
                for user_id in relevant_users {
                    let user_event = format!("dht-sync-event:{}", user_id);
                    if let Err(e) = app_handle.emit(&user_event, event_data.clone()) {
                        warn!("Failed to emit DHT event to user {}: {}", user_id, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to serialize DHT event: {}", e);
            }
        }
    }
    
    /// Determine which users should receive an event
    async fn get_relevant_users(
        event: &DHTSyncEvent,
        subscriptions: &Arc<RwLock<HashMap<String, Vec<String>>>>,
    ) -> Vec<String> {
        let subs = subscriptions.read().await;
        
        let entity_id = match event {
            DHTSyncEvent::OrganizationCreated { organization } |
            DHTSyncEvent::OrganizationUpdated { organization } => {
                organization.get("id").and_then(|v| v.as_str()).map(String::from)
            }
            DHTSyncEvent::OrganizationDeleted { id } => Some(id.clone()),
            
            DHTSyncEvent::GroupCreated { group } |
            DHTSyncEvent::GroupUpdated { group } => {
                group.get("id").and_then(|v| v.as_str()).map(String::from)
            }
            DHTSyncEvent::GroupDeleted { id } => Some(id.clone()),
            
            DHTSyncEvent::ProjectCreated { project } |
            DHTSyncEvent::ProjectUpdated { project } => {
                project.get("id").and_then(|v| v.as_str()).map(String::from)
            }
            DHTSyncEvent::ProjectDeleted { id } => Some(id.clone()),
            
            DHTSyncEvent::MemberJoined { entity_id, .. } |
            DHTSyncEvent::MemberLeft { entity_id, .. } |
            DHTSyncEvent::MemberRoleChanged { entity_id, .. } => Some(entity_id.clone()),
            
            DHTSyncEvent::FileUploaded { project_id, .. } |
            DHTSyncEvent::FileDeleted { project_id, .. } => Some(project_id.clone()),
            
            _ => None,
        };
        
        if let Some(id) = entity_id {
            subs.get(&id).cloned().unwrap_or_default()
        } else {
            // For network events, notify all subscribed users
            subs.values().flatten().cloned().collect()
        }
    }
    
    /// Start monitoring DHT for changes
    async fn start_dht_monitor(&self) -> Result<()> {
        let dht = self.dht.clone();
        let _event_tx = self.event_tx.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut last_check = Utc::now();
            
            while *is_running.read().await {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                
                // Check for DHT changes since last check
                let now = Utc::now();
                let _dht_guard = dht.read().await;
                
                // In a real implementation, the DHT would provide change events
                // For now, we'll simulate by checking timestamps
                // This would be replaced with actual DHT event subscription
                
                debug!("Checking for DHT changes since {}", last_check);
                
                // Example: Check for new/updated organizations
                // In reality, the DHT would emit events when data changes
                
                last_check = now;
            }
        });
        
        Ok(())
    }
    
    /// Poll DHT for changes (fallback mechanism)
    async fn poll_dht_changes(_dht: &Arc<RwLock<DHT>>, _app_handle: &AppHandle) {
        // This is a placeholder for actual DHT polling logic
        // In a real implementation, the DHT would push events
        debug!("Polling DHT for changes");
    }
}

// ============= Integration with Tauri =============

/// Initialize the DHT event system
pub async fn init_dht_events(
    dht: Arc<RwLock<DHT>>,
    app_handle: AppHandle,
) -> Result<Arc<RwLock<DHTEventListener>>> {
    let mut listener = DHTEventListener::new(dht, app_handle);
    listener.start().await?;
    
    Ok(Arc::new(RwLock::new(listener)))
}

// ============= Tauri Commands =============

#[tauri::command]
pub async fn subscribe_to_entity(
    entity_id: String,
    user_id: String,
    listener: tauri::State<'_, Arc<RwLock<DHTEventListener>>>,
) -> Result<(), String> {
    listener.read().await
        .subscribe(entity_id, user_id).await
        .map_err(|e| format!("Failed to subscribe: {}", e))
}

#[tauri::command]
pub async fn unsubscribe_from_entity(
    entity_id: String,
    user_id: String,
    listener: tauri::State<'_, Arc<RwLock<DHTEventListener>>>,
) -> Result<(), String> {
    listener.read().await
        .unsubscribe(&entity_id, &user_id).await
        .map_err(|e| format!("Failed to unsubscribe: {}", e))
}

#[tauri::command]
pub async fn get_sync_status(
    _listener: tauri::State<'_, Arc<RwLock<DHTEventListener>>>,
) -> Result<NetworkStatus, String> {
    // Get current sync status
    // TODO: Get actual status from the DHT event listener
    Ok(NetworkStatus {
        connected: true, // Would check actual DHT connection
        peer_count: 0,   // Would get from P2P node
        syncing: false,  // Would check if actively syncing
        last_sync: Some(Utc::now()),
    })
}