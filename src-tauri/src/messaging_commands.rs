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


//! # Messaging Commands for Tauri Integration
//! 
//! Provides secure P2P messaging functionality with saorsa-fec encryption.
//! This module implements the Tauri command interface for the messaging system,
//! ensuring all encryption operations go through the saorsa-fec crate.

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::secure_fec::{SecureFecManager, FecConfig};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use blake3;

/// Error types for messaging operations
#[derive(Debug, thiserror::Error)]
pub enum MessagingError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Message not found")]
    MessageNotFound,
    
    #[error("Invalid message format: {0}")]
    InvalidFormat(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Simple message structure for secure messaging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMessage {
    pub id: String,
    pub sender: String,
    pub recipient: Option<String>,
    pub group_id: Option<String>,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub encrypted: bool,
    pub message_type: String,
}

/// DTO for creating direct messages
#[derive(Debug, Deserialize)]
pub struct CreateDirectMessageRequest {
    pub recipient: String,
    pub content: String,
}

/// DTO for creating group messages
#[derive(Debug, Deserialize)]
pub struct CreateGroupMessageRequest {
    pub group_id: String,
    pub content: String,
}

/// DTO for message response
#[derive(Debug, Serialize)]
pub struct MessageDto {
    pub id: String,
    pub sender: String,
    pub recipient: Option<String>,
    pub group_id: Option<String>,
    pub content: String,
    pub timestamp: String,
    pub message_type: String,
    pub encrypted: bool,
}

/// DTO for group information
#[derive(Debug, Serialize)]
pub struct GroupDto {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub created_at: String,
}

/// Group information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub creator: String,
}

/// Simple in-memory storage for messages (replace with persistent storage later)
#[derive(Debug, Default)]
pub struct MessageStorage {
    messages: Mutex<HashMap<String, SecureMessage>>,
    user_messages: Mutex<HashMap<String, Vec<String>>>, // user_id -> message_ids
    groups: Mutex<HashMap<String, GroupInfo>>,
    group_members: Mutex<HashMap<String, Vec<String>>>, // group_id -> user_ids
}

impl MessageStorage {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(HashMap::new()),
            user_messages: Mutex::new(HashMap::new()),
            groups: Mutex::new(HashMap::new()),
            group_members: Mutex::new(HashMap::new()),
        }
    }

    pub fn store_message(&self, message: SecureMessage) -> Result<(), MessagingError> {
        let mut messages = self.messages.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        let mut user_messages = self.user_messages.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;

        let message_id = message.id.clone();
        let sender = message.sender.clone();

        // Add to sender's messages
        user_messages.entry(sender).or_insert_with(Vec::new).push(message_id.clone());
        
        // Add to recipient's messages if direct message
        if let Some(ref recipient) = message.recipient {
            user_messages.entry(recipient.clone()).or_insert_with(Vec::new).push(message_id.clone());
        }
        
        // Add to all group members if group message
        if let Some(ref group_id) = message.group_id {
            let group_members = self.group_members.lock()
                .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
            
            if let Some(members) = group_members.get(group_id) {
                for member in members {
                    user_messages.entry(member.clone()).or_insert_with(Vec::new).push(message_id.clone());
                }
            }
        }

        messages.insert(message_id, message);
        Ok(())
    }

    pub fn get_message(&self, message_id: &str) -> Result<SecureMessage, MessagingError> {
        let messages = self.messages.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        messages.get(message_id)
            .cloned()
            .ok_or(MessagingError::MessageNotFound)
    }

    pub fn get_user_messages(&self, user_id: &str, limit: Option<u32>) -> Result<Vec<SecureMessage>, MessagingError> {
        let messages = self.messages.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        let user_messages = self.user_messages.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;

        let empty_vec = Vec::new();
        let message_ids = user_messages.get(user_id).unwrap_or(&empty_vec);
        
        let mut user_msgs = Vec::new();
        for msg_id in message_ids {
            if let Some(msg) = messages.get(msg_id) {
                user_msgs.push(msg.clone());
            }
        }

        // Sort by timestamp (newest first)
        user_msgs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        // Apply limit if specified
        if let Some(limit) = limit {
            user_msgs.truncate(limit as usize);
        }

        Ok(user_msgs)
    }

    pub fn create_group(&self, group_info: GroupInfo) -> Result<(), MessagingError> {
        let mut groups = self.groups.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        let mut group_members = self.group_members.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;

        let group_id = group_info.id.clone();
        let members = group_info.members.clone();

        groups.insert(group_id.clone(), group_info);
        group_members.insert(group_id, members);

        Ok(())
    }

    pub fn get_group(&self, group_id: &str) -> Result<GroupInfo, MessagingError> {
        let groups = self.groups.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        groups.get(group_id)
            .cloned()
            .ok_or(MessagingError::MessageNotFound)
    }

    pub fn get_user_groups(&self, user_id: &str) -> Result<Vec<GroupInfo>, MessagingError> {
        let groups = self.groups.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        let user_groups: Vec<GroupInfo> = groups.values()
            .filter(|group| group.members.contains(&user_id.to_string()) || group.creator == *user_id)
            .cloned()
            .collect();

        Ok(user_groups)
    }

    pub fn add_group_member(&self, group_id: &str, user_id: &str) -> Result<(), MessagingError> {
        let mut groups = self.groups.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        let mut group_members = self.group_members.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;

        // Update group info
        if let Some(group) = groups.get_mut(group_id) {
            if !group.members.contains(&user_id.to_string()) {
                group.members.push(user_id.to_string());
            }
        } else {
            return Err(MessagingError::MessageNotFound);
        }

        // Update group members lookup
        if let Some(members) = group_members.get_mut(group_id) {
            if !members.contains(&user_id.to_string()) {
                members.push(user_id.to_string());
            }
        }

        Ok(())
    }

    pub fn remove_group_member(&self, group_id: &str, user_id: &str) -> Result<(), MessagingError> {
        let mut groups = self.groups.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;
        
        let mut group_members = self.group_members.lock()
            .map_err(|e| MessagingError::StorageError(format!("Lock error: {}", e)))?;

        // Update group info
        if let Some(group) = groups.get_mut(group_id) {
            group.members.retain(|member| member != user_id);
        } else {
            return Err(MessagingError::MessageNotFound);
        }

        // Update group members lookup
        if let Some(members) = group_members.get_mut(group_id) {
            members.retain(|member| member != user_id);
        }

        Ok(())
    }
}

/// Message encryption helper using saorsa-fec
struct MessageEncryption;

impl MessageEncryption {
    async fn encrypt_content(content: &str, sender: &str, recipient: &str) -> Result<String, MessagingError> {
        let config = FecConfig::default();
        let fec_manager = SecureFecManager::new(config)
            .map_err(|e| MessagingError::EncryptionFailed(format!("FEC manager creation failed: {}", e)))?;
        
        // Create deterministic key ID for this conversation
        let key_material = format!("conversation:{}:{}", sender, recipient);
        let key_id = hex::encode(blake3::hash(key_material.as_bytes()).as_bytes());
        
        let encrypted_content = fec_manager.encrypt_data(content.as_bytes(), Some(key_id)).await
            .map_err(|e| MessagingError::EncryptionFailed(format!("Encryption failed: {}", e)))?;
        
        let serialized = bincode::serialize(&encrypted_content)
            .map_err(|e| MessagingError::EncryptionFailed(format!("Serialization failed: {}", e)))?;
        
        Ok(BASE64.encode(serialized))
    }

    async fn decrypt_content(encrypted_content: &str, sender: &str, recipient: &str) -> Result<String, MessagingError> {
        let serialized_bytes = BASE64.decode(encrypted_content)
            .map_err(|e| MessagingError::DecryptionFailed(format!("Base64 decode failed: {}", e)))?;
        
        let encrypted_data: crate::secure_fec::EncryptedContent = bincode::deserialize(&serialized_bytes)
            .map_err(|e| MessagingError::DecryptionFailed(format!("Deserialization failed: {}", e)))?;
        
        let config = FecConfig::default();
        let fec_manager = SecureFecManager::new(config)
            .map_err(|e| MessagingError::DecryptionFailed(format!("FEC manager creation failed: {}", e)))?;
        
        // Recreate the same key ID used for encryption
        let key_material = format!("conversation:{}:{}", sender, recipient);
        let key_id = hex::encode(blake3::hash(key_material.as_bytes()).as_bytes());
        
        let decrypted_bytes = fec_manager.decrypt_data(&encrypted_data, &key_id).await
            .map_err(|e| MessagingError::DecryptionFailed(format!("Decryption failed: {}", e)))?;
        
        String::from_utf8(decrypted_bytes)
            .map_err(|e| MessagingError::DecryptionFailed(format!("UTF-8 decode failed: {}", e)))
    }
}

// Tauri commands for secure messaging functionality

#[tauri::command]
pub async fn initialize_messaging_system() -> Result<String, String> {
    info!("Initializing secure messaging system with saorsa-fec encryption");
    Ok("Secure messaging system initialized".to_string())
}

#[tauri::command]
pub async fn send_direct_message(
    storage: State<'_, MessageStorage>,
    sender: String,
    recipient: String,
    content: String,
) -> Result<String, String> {
    let message_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now();

    // Encrypt content using saorsa-fec
    let encrypted_content = MessageEncryption::encrypt_content(&content, &sender, &recipient).await
        .map_err(|e| format!("Failed to encrypt message: {}", e))?;

    let message = SecureMessage {
        id: message_id.clone(),
        sender,
        recipient: Some(recipient),
        group_id: None,
        content: encrypted_content,
        timestamp,
        encrypted: true,
        message_type: "direct".to_string(),
    };

    storage.store_message(message)
        .map_err(|e| format!("Failed to store message: {}", e))?;

    info!("Direct message {} sent successfully", message_id);
    Ok(message_id)
}

#[tauri::command]
pub async fn send_group_message_secure(
    storage: State<'_, MessageStorage>,
    sender: String,
    group_id: String,
    content: String,
) -> Result<String, String> {
    let message_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now();

    // For group messages, use group ID as part of encryption key
    let group_key_material = format!("group:{}:{}", group_id, sender);
    let key_id = hex::encode(blake3::hash(group_key_material.as_bytes()).as_bytes());
    
    let config = FecConfig::default();
    let fec_manager = SecureFecManager::new(config)
        .map_err(|e| format!("Failed to create FEC manager: {}", e))?;
    
    let encrypted_data = fec_manager.encrypt_data(content.as_bytes(), Some(key_id)).await
        .map_err(|e| format!("Failed to encrypt group message: {}", e))?;
    
    let serialized = bincode::serialize(&encrypted_data)
        .map_err(|e| format!("Failed to serialize encrypted data: {}", e))?;
    
    let encrypted_content = BASE64.encode(serialized);

    let message = SecureMessage {
        id: message_id.clone(),
        sender,
        recipient: None,
        group_id: Some(group_id),
        content: encrypted_content,
        timestamp,
        encrypted: true,
        message_type: "group".to_string(),
    };

    storage.store_message(message)
        .map_err(|e| format!("Failed to store group message: {}", e))?;

    info!("Group message {} sent successfully", message_id);
    Ok(message_id)
}

#[tauri::command]
pub async fn get_messages_secure(
    storage: State<'_, MessageStorage>,
    user_id: String,
    limit: Option<u32>,
) -> Result<Vec<MessageDto>, String> {
    let messages = storage.get_user_messages(&user_id, limit)
        .map_err(|e| format!("Failed to retrieve messages: {}", e))?;
    
    let mut message_dtos = Vec::new();
    
    for message in messages {
        // Decrypt message content if encrypted
        let content = if message.encrypted {
            if let Some(ref recipient) = message.recipient {
                // Direct message - decrypt using sender/recipient key
                match MessageEncryption::decrypt_content(&message.content, &message.sender, recipient).await {
                    Ok(decrypted) => decrypted,
                    Err(e) => {
                        warn!("Failed to decrypt message {}: {}", message.id, e);
                        continue; // Skip messages we can't decrypt
                    }
                }
            } else {
                // Group message - decrypt using group key
                match message.group_id {
                    Some(ref group_id) => {
                        let group_key_material = format!("group:{}:{}", group_id, message.sender);
                        let key_id = hex::encode(blake3::hash(group_key_material.as_bytes()).as_bytes());
                        
                        let serialized_bytes = match BASE64.decode(&message.content) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                warn!("Failed to decode group message {}: {}", message.id, e);
                                continue;
                            }
                        };
                        
                        let encrypted_data: crate::secure_fec::EncryptedContent = match bincode::deserialize(&serialized_bytes) {
                            Ok(data) => data,
                            Err(e) => {
                                warn!("Failed to deserialize group message {}: {}", message.id, e);
                                continue;
                            }
                        };
                        
                        let config = FecConfig::default();
                        let fec_manager = match SecureFecManager::new(config) {
                            Ok(manager) => manager,
                            Err(e) => {
                                warn!("Failed to create FEC manager for message {}: {}", message.id, e);
                                continue;
                            }
                        };
                        
                        match fec_manager.decrypt_data(&encrypted_data, &key_id).await {
                            Ok(decrypted_bytes) => match String::from_utf8(decrypted_bytes) {
                                Ok(decrypted) => decrypted,
                                Err(e) => {
                                    warn!("Failed to decode UTF-8 for message {}: {}", message.id, e);
                                    continue;
                                }
                            },
                            Err(e) => {
                                warn!("Failed to decrypt group message {}: {}", message.id, e);
                                continue;
                            }
                        }
                    }
                    None => {
                        warn!("Encrypted message {} has no recipient or group_id", message.id);
                        continue;
                    }
                }
            }
        } else {
            message.content
        };

        message_dtos.push(MessageDto {
            id: message.id,
            sender: message.sender,
            recipient: message.recipient,
            group_id: message.group_id,
            content,
            timestamp: message.timestamp.to_rfc3339(),
            message_type: message.message_type,
            encrypted: message.encrypted,
        });
    }

    debug!("Retrieved {} messages for user {}", message_dtos.len(), user_id);
    Ok(message_dtos)
}

#[tauri::command]
pub async fn subscribe_to_messages() -> Result<(), String> {
    info!("Message subscription established (placeholder for real-time functionality)");
    Ok(())
}

#[tauri::command]
pub async fn create_group_secure(
    storage: State<'_, MessageStorage>,
    creator: String,
    name: String,
    members: Vec<String>,
) -> Result<String, String> {
    let group_id = Uuid::new_v4().to_string();
    let timestamp = Utc::now();

    let group_info = GroupInfo {
        id: group_id.clone(),
        name,
        members,
        created_at: timestamp,
        creator,
    };

    storage.create_group(group_info)
        .map_err(|e| format!("Failed to create group: {}", e))?;

    info!("Group {} created successfully", group_id);
    Ok(group_id)
}

#[tauri::command]
pub async fn add_group_member(
    storage: State<'_, MessageStorage>,
    group_id: String,
    user_id: String,
) -> Result<(), String> {
    storage.add_group_member(&group_id, &user_id)
        .map_err(|e| format!("Failed to add group member: {}", e))?;

    info!("Added user {} to group {}", user_id, group_id);
    Ok(())
}

#[tauri::command]
pub async fn remove_group_member(
    storage: State<'_, MessageStorage>,
    group_id: String,
    user_id: String,
) -> Result<(), String> {
    storage.remove_group_member(&group_id, &user_id)
        .map_err(|e| format!("Failed to remove group member: {}", e))?;

    info!("Removed user {} from group {}", user_id, group_id);
    Ok(())
}

#[tauri::command]
pub async fn get_groups(
    storage: State<'_, MessageStorage>,
    user_id: String,
) -> Result<Vec<GroupDto>, String> {
    let groups = storage.get_user_groups(&user_id)
        .map_err(|e| format!("Failed to retrieve groups: {}", e))?;
    
    let group_dtos: Vec<GroupDto> = groups.into_iter().map(|group| GroupDto {
        id: group.id,
        name: group.name,
        members: group.members,
        created_at: group.created_at.to_rfc3339(),
    }).collect();

    debug!("Retrieved {} groups for user {}", group_dtos.len(), user_id);
    Ok(group_dtos)
}

/// Initialize storage state for Tauri
pub fn init_messaging_storage() -> MessageStorage {
    MessageStorage::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_message_storage() {
        let storage = MessageStorage::new();
        
        let message = SecureMessage {
            id: "test_id".to_string(),
            sender: "sender".to_string(),
            recipient: Some("recipient".to_string()),
            group_id: None,
            content: "test content".to_string(),
            timestamp: Utc::now(),
            encrypted: false,
            message_type: "direct".to_string(),
        };

        storage.store_message(message.clone()).unwrap();
        let retrieved = storage.get_message("test_id").unwrap();
        
        assert_eq!(retrieved.id, message.id);
        assert_eq!(retrieved.content, message.content);
    }

    #[tokio::test]
    async fn test_message_encryption() {
        let content = "Hello, secure world!";
        let sender = "alice";
        let recipient = "bob";

        let encrypted = MessageEncryption::encrypt_content(content, sender, recipient).await.unwrap();
        let decrypted = MessageEncryption::decrypt_content(&encrypted, sender, recipient).await.unwrap();

        assert_eq!(content, decrypted);
    }
}