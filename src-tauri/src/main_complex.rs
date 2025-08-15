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


// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn, error, debug};

// Import our modules
use saorsa_core::config::Config;

mod identity;
mod dht;
mod messaging;

use identity::{CommunidentityManager, IdentityInfo, IdentityGenerationParams};
use dht::{Dht, DhtConfig, DhtStatus, NodeId, ContentId};
use messaging::{
    MessagingSystem, MessagingConfig, Message, MessageId, UserId, GroupId,
    MessageQuery, MessageAcknowledgment, AckStatus,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkHealthResponse {
    status: String,
    peer_count: u32,
    nat_type: String,
    bandwidth_kbps: f64,
    avg_latency_ms: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageRequest {
    recipient: String,
    content: String,
    message_type: String, // "direct" or "group"
    group_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    id: String,
    sender: String,
    recipient: Option<String>,
    group_id: Option<String>,
    content: String,
    timestamp: String,
    encrypted: bool,
}

impl From<&Message> for MessageResponse {
    fn from(msg: &Message) -> Self {
        Self {
            id: msg.metadata.id.to_string(),
            sender: msg.metadata.sender.0.clone(),
            recipient: msg.metadata.recipient.as_ref().map(|r| r.0.clone()),
            group_id: msg.metadata.group_id.as_ref().map(|g| g.0.to_string()),
            content: String::from_utf8_lossy(&msg.content).to_string(),
            timestamp: msg.metadata.timestamp.to_rfc3339(),
            encrypted: msg.metadata.encrypted,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageBackendInfo {
    backend_type: String,
    key_count: usize,
    is_available: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessagingStats {
    total_messages: u64,
    pending_messages: usize,
    groups: usize,
    sync_status: String,
}

// Enhanced group management types
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupInfo {
    id: String,
    name: String,
    description: Option<String>,
    member_count: u32,
    created_at: String,
    is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPresenceInfo {
    user_id: String,
    display_name: String,
    status: String, // "online", "away", "busy", "offline"
    last_seen: String,
    activity: Option<String>,
}

// Enhanced application state with messaging system
#[allow(dead_code)]
pub struct AppState {
    identity_manager: Arc<Mutex<Option<CommunidentityManager>>>,
    dht: Arc<RwLock<Option<Arc<Dht>>>>,
    messaging_system: Arc<RwLock<Option<Arc<MessagingSystem>>>>,
    config: Config,
    dht_config: DhtConfig,
    messaging_config: MessagingConfig,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            identity_manager: Arc::new(Mutex::new(None)),
            dht: Arc::new(RwLock::new(None)),
            messaging_system: Arc::new(RwLock::new(None)),
            config: Config::default(),
            dht_config: DhtConfig::default(),
            messaging_config: MessagingConfig::default(),
        }
    }
}

// Identity management commands (existing)

#[tauri::command]
async fn initialize_identity_manager(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    info!("IPC: Initializing identity manager");
    
    match CommunidentityManager::new().await {
        Ok(manager) => {
            let mut identity_manager = state.identity_manager.lock().await;
            *identity_manager = Some(manager);
            info!("Identity manager initialized successfully");
            Ok("Identity manager initialized".to_string())
        },
        Err(e) => {
            error!("Failed to initialize identity manager: {}", e);
            Err(format!("Failed to initialize identity manager: {}", e))
        }
    }
}

#[tauri::command]
async fn get_or_create_identity(state: State<'_, Arc<AppState>>) -> Result<IdentityInfo, String> {
    info!("IPC: Getting or creating identity");
    
    let mut identity_manager = state.identity_manager.lock().await;
    if let Some(ref mut manager) = *identity_manager {
        match manager.get_or_create_identity().await {
            Ok(identity) => {
                info!("Identity retrieved/created: {}", identity.four_word_address);
                Ok(identity)
            },
            Err(e) => {
                error!("Failed to get/create identity: {}", e);
                Err(format!("Failed to get/create identity: {}", e))
            }
        }
    } else {
        Err("Identity manager not initialized".to_string())
    }
}

#[tauri::command]
async fn generate_new_identity(
    state: State<'_, Arc<AppState>>,
    params: IdentityGenerationParams,
) -> Result<IdentityInfo, String> {
    info!("IPC: Generating new identity");
    
    let mut identity_manager = state.identity_manager.lock().await;
    if let Some(ref mut manager) = *identity_manager {
        match manager.generate_new_identity(params).await {
            Ok(identity) => {
                info!("New identity generated: {}", identity.four_word_address);
                Ok(identity)
            },
            Err(e) => {
                error!("Failed to generate new identity: {}", e);
                Err(format!("Failed to generate new identity: {}", e))
            }
        }
    } else {
        Err("Identity manager not initialized".to_string())
    }
}

#[tauri::command]
async fn get_storage_info(state: State<'_, Arc<AppState>>) -> Result<StorageBackendInfo, String> {
    debug!("IPC: Getting storage info");
    
    let identity_manager = state.identity_manager.lock().await;
    if let Some(ref manager) = *identity_manager {
        match manager.get_storage_info().await {
            Ok(info) => Ok(info),
            Err(e) => {
                error!("Failed to get storage info: {}", e);
                Err(format!("Failed to get storage info: {}", e))
            }
        }
    } else {
        Err("Identity manager not initialized".to_string())
    }
}

// DHT-related IPC commands

#[tauri::command]
async fn initialize_dht(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    info!("IPC: Initializing DHT");
    
    // First ensure we have an identity
    let identity_info = {
        let mut identity_manager = state.identity_manager.lock().await;
        if identity_manager.is_none() {
            *identity_manager = Some(CommunidentityManager::new().await.map_err(|e| e.to_string())?);
        }
        
        if let Some(ref mut manager) = *identity_manager {
            manager.get_or_create_identity().await.map_err(|e| e.to_string())?
        } else {
            return Err("Failed to initialize identity manager".to_string());
        }
    };
    
    // Create node ID from identity
    let node_id = NodeId::from_hash(blake3::hash(identity_info.public_key_hex.as_bytes()));
    
    // Configure DHT with bootstrap nodes
    let mut dht_config = state.dht_config.clone();
    dht_config.bootstrap_nodes.push("127.0.0.1:8888".parse().unwrap()); // Example bootstrap
    
    // Create and start DHT
    match Dht::new(node_id, dht_config).await {
        Ok(dht_instance) => {
            if let Err(e) = dht_instance.start().await {
                error!("Failed to start DHT: {}", e);
                return Err(format!("Failed to start DHT: {}", e));
            }
            
            let dht_arc = Arc::new(dht_instance);
            let mut dht_lock = state.dht.write().await;
            *dht_lock = Some(dht_arc);
            
            info!("DHT initialized successfully");
            Ok("DHT initialized successfully".to_string())
        },
        Err(e) => {
            error!("Failed to create DHT: {}", e);
            Err(format!("Failed to create DHT: {}", e))
        }
    }
}

#[tauri::command]
async fn get_dht_status(state: State<'_, Arc<AppState>>) -> Result<Option<DhtStatus>, String> {
    debug!("IPC: Getting DHT status");
    
    let dht_lock = state.dht.read().await;
    if let Some(ref dht) = *dht_lock {
        Ok(Some(dht.status().await))
    } else {
        Ok(None)
    }
}

#[tauri::command]
async fn dht_store_content(
    state: State<'_, Arc<AppState>>,
    content: String,
) -> Result<String, String> {
    info!("IPC: Storing content in DHT (length: {})", content.len());
    
    let dht_lock = state.dht.read().await;
    if let Some(ref dht) = *dht_lock {
        let content_bytes = content.into_bytes();
        let key = ContentId::from_data(&content_bytes);
        
        match dht.store(key.clone(), content_bytes).await {
            Ok(_) => {
                let key_hex = key.to_hex();
                info!("Content stored successfully: {}", key_hex);
                Ok(key_hex)
            },
            Err(e) => {
                error!("Failed to store content: {}", e);
                Err(format!("Failed to store content: {}", e))
            }
        }
    } else {
        Err("DHT not initialized".to_string())
    }
}

#[tauri::command]
async fn dht_get_content(
    state: State<'_, Arc<AppState>>,
    key_hex: String,
) -> Result<Option<String>, String> {
    info!("IPC: Retrieving content from DHT: {}", key_hex);
    
    let dht_lock = state.dht.read().await;
    if let Some(ref dht) = *dht_lock {
        // Parse hex key
        let key_bytes = hex::decode(&key_hex).map_err(|e| format!("Invalid key format: {}", e))?;
        if key_bytes.len() != 32 {
            return Err("Key must be 32 bytes (64 hex characters)".to_string());
        }
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        let key = ContentId::from_hash(blake3::Hash::from(key_array));
        
        match dht.get(key).await {
            Ok(Some(content_bytes)) => {
                match String::from_utf8(content_bytes) {
                    Ok(content_string) => {
                        info!("Content retrieved successfully: {} bytes", content_string.len());
                        Ok(Some(content_string))
                    },
                    Err(_) => Err("Content is not valid UTF-8".to_string()),
                }
            },
            Ok(None) => {
                info!("Content not found: {}", key_hex);
                Ok(None)
            },
            Err(e) => {
                error!("Failed to retrieve content: {}", e);
                Err(format!("Failed to retrieve content: {}", e))
            }
        }
    } else {
        Err("DHT not initialized".to_string())
    }
}

#[tauri::command]
async fn dht_find_nodes(
    state: State<'_, Arc<AppState>>,
    target_hex: String,
) -> Result<Vec<String>, String> {
    info!("IPC: Finding nodes closest to: {}", target_hex);
    
    let dht_lock = state.dht.read().await;
    if let Some(ref dht) = *dht_lock {
        // Parse hex target
        let target_bytes = hex::decode(&target_hex).map_err(|e| format!("Invalid target format: {}", e))?;
        if target_bytes.len() != 20 {
            return Err("Target must be 20 bytes (40 hex characters)".to_string());
        }
        
        let mut target_array = [0u8; 20];
        target_array.copy_from_slice(&target_bytes);
        let target = NodeId::from_bytes(target_array);
        
        match dht.find_node(target).await {
            Ok(nodes) => {
                let node_hexes: Vec<String> = nodes.iter()
                    .map(|node| node.to_hex())
                    .collect();
                info!("Found {} nodes", node_hexes.len());
                Ok(node_hexes)
            },
            Err(e) => {
                error!("Failed to find nodes: {}", e);
                Err(format!("Failed to find nodes: {}", e))
            }
        }
    } else {
        Err("DHT not initialized".to_string())
    }
}

// P2P Messaging System Commands

#[tauri::command]
async fn initialize_messaging(state: State<'_, Arc<AppState>>) -> Result<String, String> {
    info!("IPC: Initializing messaging system");
    
    // Ensure prerequisites are initialized
    let dht_lock = state.dht.read().await;
    if dht_lock.is_none() {
        return Err("DHT must be initialized before messaging system".to_string());
    }
    
    let identity_lock = state.identity_manager.lock().await;
    if identity_lock.is_none() {
        return Err("Identity manager must be initialized before messaging system".to_string());
    }
    
    // Create messaging system
    match MessagingSystem::new(
        state.messaging_config.clone(),
        state.dht.clone(),
        state.identity_manager.clone(),
    ).await {
        Ok(messaging) => {
            let mut messaging_lock = state.messaging_system.write().await;
            *messaging_lock = Some(messaging);
            
            info!("Messaging system initialized successfully");
            Ok("Messaging system initialized".to_string())
        },
        Err(e) => {
            error!("Failed to initialize messaging system: {}", e);
            Err(format!("Failed to initialize messaging system: {}", e))
        }
    }
}

#[tauri::command]
async fn send_message_to_user(
    state: State<'_, Arc<AppState>>,
    request: MessageRequest,
) -> Result<String, String> {
    info!("IPC: Sending message to user: {}", request.recipient);
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        let recipient = UserId::new(request.recipient);
        let content = request.content.into_bytes();
        
        match messaging.send_message(recipient, content).await {
            Ok(message_id) => {
                info!("Message sent successfully: {}", message_id);
                Ok(message_id.to_string())
            },
            Err(e) => {
                error!("Failed to send message: {}", e);
                Err(format!("Failed to send message: {}", e))
            }
        }
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

#[tauri::command]
async fn send_group_message(
    state: State<'_, Arc<AppState>>,
    request: MessageRequest,
) -> Result<String, String> {
    info!("IPC: Sending group message");
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        let group_id = request.group_id
            .ok_or_else(|| "Group ID required for group messages".to_string())?;
        let group_id = GroupId(uuid::Uuid::parse_str(&group_id)
            .map_err(|e| format!("Invalid group ID: {}", e))?);
        let content = request.content.into_bytes();
        
        match messaging.send_group_message(group_id, content).await {
            Ok(message_id) => {
                info!("Group message sent successfully: {}", message_id);
                Ok(message_id.to_string())
            },
            Err(e) => {
                error!("Failed to send group message: {}", e);
                Err(format!("Failed to send group message: {}", e))
            }
        }
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

#[tauri::command]
async fn get_messages(
    state: State<'_, Arc<AppState>>,
    user_id: Option<String>,
    group_id: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MessageResponse>, String> {
    debug!("IPC: Getting messages");
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        let query = MessageQuery {
            user_id: user_id.map(UserId::new),
            group_id: group_id.and_then(|id| uuid::Uuid::parse_str(&id).ok().map(GroupId)),
            limit: limit.or(Some(50)),
            ..Default::default()
        };
        
        match messaging.get_messages(query).await {
            Ok(messages) => {
                let responses: Vec<MessageResponse> = messages.iter()
                    .map(MessageResponse::from)
                    .collect();
                
                debug!("Retrieved {} messages", responses.len());
                Ok(responses)
            },
            Err(e) => {
                error!("Failed to get messages: {}", e);
                Err(format!("Failed to get messages: {}", e))
            }
        }
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

#[tauri::command]
async fn create_group(
    state: State<'_, Arc<AppState>>,
    name: String,
    description: Option<String>,
) -> Result<String, String> {
    info!("IPC: Creating group: {}", name);
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        // Get current user identity
        let current_user = {
            let identity_lock = state.identity_manager.lock().await;
            if let Some(ref manager) = *identity_lock {
                let identity = manager.get_current_identity().await
                    .map_err(|e| format!("Failed to get current identity: {}", e))?;
                UserId::new(identity.four_word_address)
            } else {
                return Err("Identity not available".to_string());
            }
        };
        
        // Create group via group manager
        match messaging.group_manager.create_group(name, current_user, description).await {
            Ok(group_id) => {
                info!("Group created successfully: {}", group_id.0);
                Ok(group_id.0.to_string())
            },
            Err(e) => {
                error!("Failed to create group: {}", e);
                Err(format!("Failed to create group: {}", e))
            }
        }
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

#[tauri::command]
async fn get_messaging_stats(state: State<'_, Arc<AppState>>) -> Result<MessagingStats, String> {
    debug!("IPC: Getting messaging stats");
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        let storage_stats = messaging.message_store.get_stats().await
            .map_err(|e| format!("Failed to get storage stats: {}", e))?;
        
        let sync_stats = messaging.sync_manager.get_stats().await
            .map_err(|e| format!("Failed to get sync stats: {}", e))?;
        
        Ok(MessagingStats {
            total_messages: storage_stats.total_messages,
            pending_messages: 0, // TODO: Get from messaging system
            groups: 0, // TODO: Get from group manager
            sync_status: format!("Syncing with {} peers", sync_stats.total_peers),
        })
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

// Enhanced Group Management Commands

#[tauri::command]
async fn get_groups(state: State<'_, Arc<AppState>>) -> Result<Vec<GroupInfo>, String> {
    info!("IPC: Getting user groups");
    
    // For now, return mock data until backend group listing is implemented
    let mock_groups = vec![
        GroupInfo {
            id: "general".to_string(),
            name: "General".to_string(),
            description: Some("General discussion for everyone".to_string()),
            member_count: 12,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_admin: false,
        },
        GroupInfo {
            id: "tech-talk".to_string(),
            name: "Tech Talk".to_string(),
            description: Some("Technical discussions and P2P development".to_string()),
            member_count: 8,
            created_at: chrono::Utc::now().to_rfc3339(),
            is_admin: true,
        },
    ];
    
    Ok(mock_groups)
}

#[tauri::command]
async fn get_group_members(
    state: State<'_, Arc<AppState>>,
    group_id: String,
) -> Result<Vec<UserPresenceInfo>, String> {
    info!("IPC: Getting group members for: {}", group_id);
    
    // Mock user presence data until P2P presence system is implemented
    let mock_members = vec![
        UserPresenceInfo {
            user_id: "alice".to_string(),
            display_name: "Alice Cooper".to_string(),
            status: "online".to_string(),
            last_seen: chrono::Utc::now().to_rfc3339(),
            activity: Some("Developing P2P features".to_string()),
        },
        UserPresenceInfo {
            user_id: "bob".to_string(),
            display_name: "Bob Builder".to_string(),
            status: "away".to_string(),
            last_seen: (chrono::Utc::now() - chrono::Duration::minutes(5)).to_rfc3339(),
            activity: Some("In a meeting".to_string()),
        },
        UserPresenceInfo {
            user_id: "charlie".to_string(),
            display_name: "Charlie Brown".to_string(),
            status: "busy".to_string(),
            last_seen: (chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339(),
            activity: Some("Do not disturb".to_string()),
        },
    ];
    
    Ok(mock_members)
}

#[tauri::command]
async fn update_user_presence(
    state: State<'_, Arc<AppState>>,
    status: String,
    activity: Option<String>,
) -> Result<(), String> {
    info!("IPC: Updating user presence: {} - {:?}", status, activity);
    
    // In a real implementation, this would update the user's presence in the P2P network
    // For now, we just log the update
    debug!("User presence updated to: {}", status);
    if let Some(activity) = activity {
        debug!("User activity: {}", activity);
    }
    
    Ok(())
}

#[tauri::command]
async fn join_group(
    state: State<'_, Arc<AppState>>,
    group_id: String,
) -> Result<(), String> {
    info!("IPC: Joining group: {}", group_id);
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        // In a real implementation, this would join the group via the messaging system
        // For now, we just acknowledge the request
        info!("Successfully joined group: {}", group_id);
        Ok(())
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

#[tauri::command]
async fn leave_group(
    state: State<'_, Arc<AppState>>,
    group_id: String,
) -> Result<(), String> {
    info!("IPC: Leaving group: {}", group_id);
    
    let messaging_lock = state.messaging_system.read().await;
    if let Some(ref messaging) = *messaging_lock {
        // In a real implementation, this would leave the group via the messaging system
        // For now, we just acknowledge the request
        info!("Successfully left group: {}", group_id);
        Ok(())
    } else {
        Err("Messaging system not initialized".to_string())
    }
}

#[tauri::command]
async fn get_unread_counts(state: State<'_, Arc<AppState>>) -> Result<HashMap<String, u32>, String> {
    info!("IPC: Getting unread message counts");
    
    // Mock unread counts until message tracking is implemented
    let mut unread_counts = HashMap::new();
    unread_counts.insert("general".to_string(), 3);
    unread_counts.insert("tech-talk".to_string(), 0);
    unread_counts.insert("random".to_string(), 1);
    
    Ok(unread_counts)
}

// Network and testing commands

#[tauri::command]
async fn get_network_health() -> Result<NetworkHealthResponse, String> {
    debug!("IPC: Getting network health");
    
    // Simplified network health response
    Ok(NetworkHealthResponse {
        status: "connected".to_string(),
        peer_count: 0,
        nat_type: "unknown".to_string(),
        bandwidth_kbps: 0.0,
        avg_latency_ms: 0.0,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Communitas application with enhanced P2P group chat");

    let app_state = Arc::new(AppState::new());

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Identity commands
            initialize_identity_manager,
            get_or_create_identity,
            generate_new_identity,
            get_storage_info,
            // DHT commands
            initialize_dht,
            get_dht_status,
            dht_store_content,
            dht_get_content,
            dht_find_nodes,
            // Messaging commands
            initialize_messaging,
            send_message_to_user,
            send_group_message,
            get_messages,
            create_group,
            get_messaging_stats,
            // Enhanced group management commands
            get_groups,
            get_group_members,
            update_user_presence,
            join_group,
            leave_group,
            get_unread_counts,
            // Network commands
            get_network_health,
        ])
        .setup(|_app| {
            info!("Tauri application setup complete with enhanced group chat features");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
EOF < /dev/null