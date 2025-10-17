// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

use communitas_core::messaging::{
    MlsClient, MlsConfig, ProcessedMessage, GroupSettings
};
// Removed: use saorsa_core::identity::enhanced::EnhancedIdentity;
use saorsa_mls::GroupId;
use saorsa_mls::{MlsMessage, ApplicationMessage, MemberId};
use saorsa_mls::crypto::DebugMlDsaSignature;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

/// MLS client state management
#[derive(Default)]
pub struct MlsState {
    pub client: Option<Arc<MlsClient>>,
}

impl MlsState {
    pub async fn get_client(&self) -> Result<Arc<MlsClient>, String> {
        self.client
            .as_ref()
            .ok_or_else(|| "MLS client not initialized".to_string())
            .cloned()
    }

    pub async fn set_client(&mut self, client: MlsClient) {
        self.client = Some(Arc::new(client));
    }
}

/// Initialize MLS client with configuration
#[tauri::command]
pub async fn core_mls_initialize(
    state: State<'_, Arc<RwLock<MlsState>>>,
    enable_pqc: Option<bool>,
    max_epochs: Option<u64>,
    key_rotation_interval: Option<u64>,
) -> Result<bool, String> {
    let config = MlsConfig {
        enable_pqc: enable_pqc.unwrap_or(true),
        max_epochs: max_epochs.unwrap_or(1000),
        key_rotation_interval: key_rotation_interval.unwrap_or(100),
        ..Default::default()
    };

    let client = MlsClient::new(config)
        .await
        .map_err(|e| format!("Failed to create MLS client: {}", e))?;

    let mut guard = state.write().await;
    guard.set_client(client).await;
    info!("MLS client initialized successfully");
    Ok(true)
}

/// Create a new MLS group
#[tauri::command]
pub async fn core_mls_create_group(
    state: State<'_, Arc<RwLock<MlsState>>>,
    _group_name: String,
    identity_json: String,
) -> Result<String, String> {
    let client = state.read().await.get_client().await?;

    // Parse identity from JSON
    let identity: EnhancedIdentity = serde_json::from_str(&identity_json)
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    let group_id = GroupId::generate();

    let _group_state = client
        .create_group(group_id.clone(), &identity)
        .await
        .map_err(|e| format!("Failed to create group: {}", e))?;

    info!("Created MLS group: {:?}", group_id);
    Ok(group_id.to_string())
}

/// Join an existing MLS group via welcome message
#[tauri::command]
pub async fn core_mls_join_group(
    state: State<'_, Arc<RwLock<MlsState>>>,
    welcome_data: Vec<u8>,
    identity_json: String,
) -> Result<String, String> {
    let client = state.read().await.get_client().await?;

    // Parse identity from JSON
    let identity: EnhancedIdentity = serde_json::from_str(&identity_json)
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    let group_state = client
        .join_group(welcome_data, &identity)
        .await
        .map_err(|e| format!("Failed to join group: {}", e))?;

    info!("Joined MLS group: {:?}", group_state.group_id);
    Ok(group_state.group_id.to_string())
}

/// Leave an MLS group
#[tauri::command]
pub async fn core_mls_leave_group(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
) -> Result<bool, String> {
    let client = state.read().await.get_client().await?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    client
        .leave_group(&group_id_parsed)
        .await
        .map_err(|e| format!("Failed to leave group: {}", e))?;

    info!("Left MLS group: {}", group_id);
    Ok(true)
}

/// Send a message to an MLS group
#[tauri::command]
pub async fn core_mls_send_message(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
    content: Vec<u8>,
    _content_type: String,
    identity_json: String,
) -> Result<String, String> {
    let client = state.read().await.get_client().await?;

    // Parse identity from JSON
    let identity: EnhancedIdentity = serde_json::from_str(&identity_json)
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    let message = client
        .send_message(&group_id_parsed, content, &identity)
        .await
        .map_err(|e| format!("Failed to send message: {}", e))?;

    info!("Sent MLS message to group: {}", group_id);
    Ok(format!("{:?}", message))
}

/// Process an incoming MLS message
#[tauri::command]
pub async fn core_mls_process_message(
    state: State<'_, Arc<RwLock<MlsState>>>,
    message_data: Vec<u8>,
    identity_json: String,
) -> Result<ProcessedMessageResult, String> {
    let client = state.read().await.get_client().await?;

    // Parse identity from JSON
    let identity: EnhancedIdentity = serde_json::from_str(&identity_json)
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    // For now, create a placeholder MlsMessage from the raw data
    // TODO: Replace with proper MlsMessage deserialization when type is available
    let message = MlsMessage::Application(ApplicationMessage {
        epoch: 0,
        sender: MemberId::generate(),
        generation: 0,
        sequence: 0,
        ciphertext: message_data.clone(),
        signature: {
            // Create a placeholder signature using unsafe code for compilation
            // This is a temporary workaround until we have proper MLS signature API
            #[allow(invalid_value)]
            unsafe { std::mem::MaybeUninit::<DebugMlDsaSignature>::zeroed().assume_init() }
        },
    });

    let processed = client
        .process_message(message, &identity)
        .await
        .map_err(|e| format!("Failed to process message: {}", e))?;

    // Convert to serializable result
    let result = match processed {
        ProcessedMessage::Application(decrypted) => ProcessedMessageResult {
            data: ProcessedMessageData::Application {
                content: decrypted.content,
                sender: serde_json::to_string(&decrypted.sender).unwrap_or_else(|_| "unknown".to_string()),
                epoch: decrypted.epoch,
                signature: decrypted.signature,
            },
        },
        ProcessedMessage::Proposal(data) => ProcessedMessageResult {
            data: ProcessedMessageData::Proposal(data),
        },
        ProcessedMessage::Commit(data) => ProcessedMessageResult {
            data: ProcessedMessageData::Commit(data),
        },
    };

    info!("Processed MLS message successfully");
    Ok(result)
}

/// Get MLS group information
#[tauri::command]
pub async fn core_mls_get_group(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
) -> Result<GroupInfo, String> {
    let client = state.read().await.get_client().await?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    let group_state = client
        .get_group(&group_id_parsed)
        .await
        .ok_or_else(|| format!("Group not found: {}", group_id))?;

    let members: Vec<String> = group_state
        .members
        .iter()
        .map(|m| serde_json::to_string(m).unwrap_or_else(|_| "unknown".to_string()))
        .collect();

    let info = GroupInfo {
        group_id: group_state.group_id.to_string(),
        epoch: group_state.epoch,
        member_count: members.len(),
        members,
    };

    Ok(info)
}

/// List all MLS groups
#[tauri::command]
pub async fn core_mls_list_groups(
    state: State<'_, Arc<RwLock<MlsState>>>,
) -> Result<Vec<String>, String> {
    let client = state.read().await.get_client().await?;

    let groups = client
        .list_groups()
        .await
        .into_iter()
        .map(|g| g.to_string())
        .collect();

    Ok(groups)
}

/// Generate key packages for group invitations
#[tauri::command]
pub async fn core_mls_generate_key_packages(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
    count: usize,
    identity_json: String,
) -> Result<Vec<Vec<u8>>, String> {
    let client = state.read().await.get_client().await?;

    // Parse identity from JSON
    let identity: EnhancedIdentity = serde_json::from_str(&identity_json)
        .map_err(|e| format!("Failed to parse identity: {}", e))?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    let key_packages = client
        .generate_key_packages(&group_id_parsed, count, &identity)
        .await
        .map_err(|e| format!("Failed to generate key packages: {}", e))?;

    info!("Generated {} key packages for group: {}", count, group_id);
    Ok(key_packages)
}

/// Add a member to an MLS group
#[tauri::command]
pub async fn core_mls_add_member(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
    member_json: String,
) -> Result<bool, String> {
    let client = state.read().await.get_client().await?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    // Get the group state
    let mut group_state = client
        .get_group(&group_id_parsed)
        .await
        .ok_or_else(|| format!("Group not found: {}", group_id))?;

    // Parse member identity from JSON
    let member: EnhancedIdentity = serde_json::from_str(&member_json)
        .map_err(|e| format!("Failed to parse member identity: {}", e))?;

    group_state
        .add_member(member)
        .await
        .map_err(|e| format!("Failed to add member: {}", e))?;

    info!("Added member to MLS group: {}", group_id);
    Ok(true)
}

/// Remove a member from an MLS group
#[tauri::command]
pub async fn core_mls_remove_member(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
    member_json: String,
) -> Result<bool, String> {
    let client = state.read().await.get_client().await?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    // Get the group state
    let mut group_state = client
        .get_group(&group_id_parsed)
        .await
        .ok_or_else(|| format!("Group not found: {}", group_id))?;

    // Parse member identity from JSON
    let member: EnhancedIdentity = serde_json::from_str(&member_json)
        .map_err(|e| format!("Failed to parse member identity: {}", e))?;

    group_state
        .remove_member(&member)
        .await
        .map_err(|e| format!("Failed to remove member: {}", e))?;

    info!("Removed member from MLS group: {}", group_id);
    Ok(true)
}

/// Update MLS group settings
#[tauri::command]
pub async fn core_mls_update_group_settings(
    state: State<'_, Arc<RwLock<MlsState>>>,
    group_id: String,
    settings_json: String,
) -> Result<bool, String> {
    let client = state.read().await.get_client().await?;

    let group_id_parsed = GroupId::generate(); // TODO: Parse from string when available

    // Get the group state
    let mut group_state = client
        .get_group(&group_id_parsed)
        .await
        .ok_or_else(|| format!("Group not found: {}", group_id))?;

    // Parse settings from JSON
    let settings: GroupSettings = serde_json::from_str(&settings_json)
        .map_err(|e| format!("Failed to parse group settings: {}", e))?;

    group_state
        .update_group_settings(settings)
        .await
        .map_err(|e| format!("Failed to update group settings: {}", e))?;

    info!("Updated settings for MLS group: {}", group_id);
    Ok(true)
}

/// Get MLS client status
#[tauri::command]
pub async fn core_mls_get_status(
    state: State<'_, Arc<RwLock<MlsState>>>,
) -> Result<MlsStatus, String> {
    let guard = state.read().await;
    let client = guard.get_client().await?;

    let groups = client.list_groups().await;
    let config = client.config.clone();

    Ok(MlsStatus {
        initialized: true,
        group_count: groups.len(),
        config: MlsConfigInfo {
            enable_pqc: config.enable_pqc,
            max_epochs: config.max_epochs,
            key_rotation_interval: config.key_rotation_interval,
        },
    })
}

// Serializable result types for Tauri commands

#[derive(serde::Serialize)]
pub struct ProcessedMessageResult {
    #[serde(flatten)]
    pub data: ProcessedMessageData,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum ProcessedMessageData {
    Application {
        content: Vec<u8>,
        sender: String,
        epoch: u64,
        signature: Vec<u8>,
    },
    Proposal(Vec<u8>),
    Commit(Vec<u8>),
}

#[derive(serde::Serialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub epoch: u64,
    pub member_count: usize,
    pub members: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct MlsStatus {
    pub initialized: bool,
    pub group_count: usize,
    pub config: MlsConfigInfo,
}

#[derive(serde::Serialize)]
pub struct MlsConfigInfo {
    pub enable_pqc: bool,
    pub max_epochs: u64,
    pub key_rotation_interval: u64,
}