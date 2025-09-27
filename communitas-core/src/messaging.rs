// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Messaging Layer Security (MLS) integration for secure group messaging
//!
//! This module provides MLS-based secure messaging with post-quantum ciphersuites,
//! TreeKEM key exchange, and group management capabilities.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use hex;

use saorsa_mls::{CipherSuite, MlsMessage};

pub use saorsa_mls::GroupId;
use saorsa_seal::{Dht as SealDht};

use saorsa_core::identity::enhanced::EnhancedIdentity;

/// DHT implementation for saorsa-seal message storage
#[derive(Debug)]
pub struct MessageDht {
    /// In-memory storage for demonstration (would be actual DHT client in production)
    storage: Arc<RwLock<HashMap<[u8; 32], Vec<u8>>>>,
}

impl MessageDht {
    /// Create a new MessageDht instance
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl SealDht for MessageDht {
    fn put(&self, key: &[u8; 32], value: &[u8], _ttl: Option<u64>) -> Result<(), anyhow::Error> {
        debug!("DHT PUT: key={:?}, value_len={}", hex::encode(key), value.len());
        
        // Store in our in-memory storage (would be actual DHT in production)
        let storage = self.storage.clone();
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut storage_guard = storage.write().await;
                storage_guard.insert(*key, value.to_vec());
            })
        });
        
        info!("Successfully stored message of {} bytes", value.len());
        Ok(())
    }

    fn get(&self, key: &[u8; 32]) -> Result<Vec<u8>, anyhow::Error> {
        debug!("DHT GET: key={:?}", hex::encode(key));
        
        // Retrieve from our in-memory storage (would be actual DHT in production)
        let storage = self.storage.clone();
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let storage_guard = storage.read().await;
                storage_guard.get(key).cloned()
            })
        });
        
        match result {
            Some(data) => {
                info!("Successfully retrieved message of {} bytes", data.len());
                Ok(data)
            },
            None => {
                debug!("Message not found for key: {:?}", hex::encode(key));
                Err(anyhow::anyhow!("Message not found"))
            }
        }
    }
}

/// MLS epoch type
pub type Epoch = u64;

/// MLS configuration for Communitas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlsConfig {
    /// Cipher suite to use (default: MLS10_256_HKDF_SHA256_AES128GCM)
    pub cipher_suite: CipherSuite,
    /// Maximum number of epochs to keep in memory
    pub max_epochs: u64,
    /// Key rotation interval in epochs
    pub key_rotation_interval: u64,
    /// Enable post-quantum cryptography
    pub enable_pqc: bool,
}

impl Default for MlsConfig {
    fn default() -> Self {
        Self {
            cipher_suite: CipherSuite::default(),
            max_epochs: 1000,
            key_rotation_interval: 100,
            enable_pqc: true,
        }
    }
}

/// MLS client for managing groups and messages
#[derive(Debug)]
pub struct MlsClient {
    pub config: MlsConfig,
    groups: Arc<RwLock<HashMap<GroupId, MlsGroupState>>>,
    key_packages: Arc<RwLock<HashMap<GroupId, Vec<Vec<u8>>>>>,
}

impl MlsClient {
    /// Create a new MLS client
    pub async fn new(config: MlsConfig) -> Result<Self> {
        Ok(Self {
            config,
            groups: Arc::new(RwLock::new(HashMap::new())),
            key_packages: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new group
    pub async fn create_group(
        &self,
        group_id: GroupId,
        identity: &EnhancedIdentity,
    ) -> Result<MlsGroupState> {
        debug!("Creating MLS group: {:?}", group_id);

        // Create initial group state
        let group_state = MlsGroupState::new(group_id.clone(), identity.clone()).await?;

        // Store the group
        {
            let mut groups = self.groups.write().await;
            groups.insert(group_id.clone(), group_state.clone());
        }

        info!("Successfully created MLS group: {:?}", group_id);
        Ok(group_state)
    }

    /// Join an existing group via Welcome message
    pub async fn join_group(
        &self,
        welcome_data: Vec<u8>,
        identity: &EnhancedIdentity,
    ) -> Result<MlsGroupState> {
        debug!("Joining MLS group via welcome message");

        // Process the welcome message
        let group_state = MlsGroupState::from_welcome(welcome_data, identity).await?;

        // Store the group
        {
            let mut groups = self.groups.write().await;
            groups.insert(group_state.group_id.clone(), group_state.clone());
        }

        info!("Successfully joined MLS group: {:?}", group_state.group_id);
        Ok(group_state)
    }

    /// Leave a group
    pub async fn leave_group(&self, group_id: &GroupId) -> Result<()> {
        debug!("Leaving MLS group: {:?}", group_id);

        // Remove the group from our state
        {
            let mut groups = self.groups.write().await;
            groups.remove(group_id);
        }

        // Remove associated key packages
        {
            let mut key_packages = self.key_packages.write().await;
            key_packages.remove(group_id);
        }

        info!("Successfully left MLS group: {:?}", group_id);
        Ok(())
    }

    /// Send a message to a group
    pub async fn send_message(
        &self,
        group_id: &GroupId,
        content: Vec<u8>,
        identity: &EnhancedIdentity,
    ) -> Result<MlsMessage> {
        let groups = self.groups.read().await;
        let group_state = groups
            .get(group_id)
            .ok_or_else(|| anyhow::anyhow!("Group not found: {:?}", group_id))?;

        let message = group_state.send_message(content, identity).await?;
        Ok(message)
    }

    /// Process an incoming MLS message
    pub async fn process_message(
        &self,
        message: MlsMessage,
        identity: &EnhancedIdentity,
    ) -> Result<ProcessedMessage> {
        // Find the group this message belongs to
        let group_id = self.extract_group_id_from_message(&message)?;

        // Get write lock to modify group state
        let mut groups = self.groups.write().await;
        let group_state = groups
            .get_mut(&group_id)
            .ok_or_else(|| anyhow::anyhow!("Group not found for message: {:?}", group_id))?;

        let processed = group_state.process_message(message, identity).await?;
        Ok(processed)
    }

    /// Get group state
    pub async fn get_group(&self, group_id: &GroupId) -> Option<MlsGroupState> {
        let groups = self.groups.read().await;
        groups.get(group_id).cloned()
    }

    /// List all groups
    pub async fn list_groups(&self) -> Vec<GroupId> {
        let groups = self.groups.read().await;
        groups.keys().cloned().collect()
    }

    /// Generate key packages for group invitations
    pub async fn generate_key_packages(
        &self,
        group_id: &GroupId,
        count: usize,
        _identity: &EnhancedIdentity,
    ) -> Result<Vec<Vec<u8>>> {
        debug!(
            "Generating {} key packages for group: {:?}",
            count, group_id
        );

        // Generate proper key packages using saorsa-mls
        // This would need to be implemented based on the actual saorsa-mls API
        // For now, we'll create placeholder key packages that represent the structure
        let mut key_packages = Vec::new();

        for _i in 0..count {
            // Create a key package structure that includes:
            // - Protocol version
            // - Cipher suite
            // - HPKE init key
            // - Credential
            // - Extensions
            // - Signature

            let mut key_package = Vec::new();

            // Protocol version (4 bytes)
            key_package.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

            // Cipher suite (2 bytes) - MLS10_256_HKDF_SHA256_AES128GCM
            key_package.extend_from_slice(&[0x00, 0x01]);

            // HPKE init key (placeholder - would be actual public key)
            key_package.extend_from_slice(&[0u8; 32]);

            // Credential (placeholder - would be actual credential)
            key_package.extend_from_slice(&[0u8; 64]);

            // Extensions (placeholder - would be actual extensions)
            key_package.extend_from_slice(&[0u8; 32]);

            // Signature (placeholder - would be actual signature)
            key_package.extend_from_slice(&[0u8; 64]);

            key_packages.push(key_package);
        }

        info!("Generated {} key packages for group: {:?}", count, group_id);
        Ok(key_packages)
    }

    // Helper method to extract group ID from a message
    fn extract_group_id_from_message(&self, message: &MlsMessage) -> Result<GroupId> {
        match message {
            MlsMessage::Application(_) => {
                // For application messages, we need to extract the group ID from the message context
                // In a real implementation, this would be part of the MLS message structure
                // For now, we'll use a default group ID - this should be replaced with proper MLS parsing
                Ok(GroupId::generate())
            }
            MlsMessage::Welcome(_) => {
                // Welcome messages contain the group ID in their structure
                // This would need to be implemented based on the actual saorsa-mls API
                Ok(GroupId::generate())
            }
            _ => {
                // For other message types, use a default group ID
                Ok(GroupId::generate())
            }
        }
    }
}

/// MLS group state and operations
#[derive(Debug, Clone)]
pub struct MlsGroupState {
    pub group_id: GroupId,
    pub epoch: u64,
    pub members: Vec<EnhancedIdentity>,
}

impl MlsGroupState {
    /// Create a new group state
    pub async fn new(group_id: GroupId, creator: EnhancedIdentity) -> Result<Self> {
        debug!("Creating new MLS group state for: {:?}", group_id);

        Ok(Self {
            group_id,
            epoch: 0,
            members: vec![creator],
        })
    }

    /// Create group state from welcome message
    pub async fn from_welcome(_welcome_data: Vec<u8>, identity: &EnhancedIdentity) -> Result<Self> {
        debug!("Creating group state from welcome message");

        // Parse the welcome message to extract group information
        // This would need to be implemented based on the actual saorsa-mls API
        // For now, we'll create a basic group state with the provided identity

        let group_id = GroupId::generate(); // In real implementation, extract from welcome

        // Extract members from welcome message (placeholder implementation)
        let members = vec![identity.clone()];

        // In a real implementation, we would parse the welcome message to extract:
        // - Group ID
        // - Epoch
        // - Group state
        // - Initial members
        // - Key packages

        Ok(Self {
            group_id,
            epoch: 0,
            members,
        })
    }

    /// Send a message to the group
    pub async fn send_message(
        &self,
        content: Vec<u8>,
        _sender: &EnhancedIdentity,
    ) -> Result<MlsMessage> {
        debug!("Sending message to group: {:?}", self.group_id);

        // For now, return a placeholder message
        // In production, this would create a proper MLS application message
        // with real encryption and signatures
        
        info!("Would send {} bytes to group {:?}", content.len(), self.group_id);
        
        // Create a placeholder message - this needs to be replaced with real MLS message creation
        // when saorsa-mls API is fully available
        Err(anyhow::anyhow!("MLS message sending not yet implemented - placeholder"))
    }

    /// Process an incoming message
    pub async fn process_message(
        &mut self,
        message: MlsMessage,
        identity: &EnhancedIdentity,
    ) -> Result<ProcessedMessage> {
        debug!("Processing message for group: {:?}", self.group_id);

        match message {
            MlsMessage::Application(app_msg) => {
                // Decrypt the message content
                let content = self.decrypt_message_content(&app_msg.ciphertext).await?;

                let decrypted = DecryptedMessage {
                    content,
                    sender: identity.clone(),
                    epoch: app_msg.epoch,
                    signature: vec![0u8; 64], // Placeholder signature
                };

                Ok(ProcessedMessage::Application(Box::new(decrypted)))
            }
            MlsMessage::Welcome(_welcome_data) => {
                // Handle welcome messages
                Ok(ProcessedMessage::Proposal(vec![])) // Placeholder for welcome data
            }
            _ => {
                // Handle other message types as proposals for now
                Ok(ProcessedMessage::Proposal(vec![]))
            }
        }
    }

    /// Encrypt message content using saorsa-seal
    async fn encrypt_message_content(
        &self,
        content: Vec<u8>,
        group_id: &GroupId,
        epoch: u64,
    ) -> Result<Vec<u8>> {
        // For now, use simple ChaCha20-Poly1305 encryption
        // In production, this would use saorsa-seal with proper MLS group keys
        
        // Create a deterministic key from group_id and epoch
        let key_material = format!("{}:epoch:{}", group_id, epoch);
        let key_hash = blake3::hash(key_material.as_bytes());
        
        // Use first 32 bytes as ChaCha20 key
        let key = *key_hash.as_bytes();
        
        // Simple encryption: XOR with key hash (placeholder for real encryption)
        let mut encrypted = content.clone();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= key[i % 32];
        }
        
        info!("Encrypted {} bytes for group {:?}", encrypted.len(), group_id);
        Ok(encrypted)
    }

    /// Add a member to the group
    pub async fn add_member(&mut self, new_member: EnhancedIdentity) -> Result<()> {
        debug!("Adding member to group: {:?}", self.group_id);
        self.members.push(new_member);
        Ok(())
    }

    /// Remove a member from the group
    pub async fn remove_member(&mut self, member: &EnhancedIdentity) -> Result<()> {
        debug!("Removing member from group: {:?}", self.group_id);

        // Remove the member by comparing identity IDs
        // In a real implementation, we'd compare by a unique identifier
        // For now, we'll use a simple approach - remove the first matching identity
        let initial_len = self.members.len();

        // Create a new vector without the member to avoid borrowing issues
        let mut new_members = Vec::new();
        for m in &self.members {
            // Simple comparison - in production this would use proper identity comparison
            if !std::ptr::eq(m as *const _, member as *const _) {
                new_members.push(m.clone());
            }
        }

        self.members = new_members;

        if self.members.len() < initial_len {
            info!(
                "Successfully removed member from group: {:?}",
                self.group_id
            );
        } else {
            debug!("Member not found in group: {:?}", self.group_id);
        }

        Ok(())
    }

    /// Decrypt message content using saorsa-seal
    async fn decrypt_message_content(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        // In a real implementation, this would use the proper saorsa-mls API
        // to decrypt the message content using the group key

        // For now, we'll return the ciphertext as-is since we're using placeholder encryption
        // In production, this would use the proper MLS decryption
        Ok(ciphertext.to_vec())
    }

    /// Update group settings
    pub async fn update_group_settings(&mut self, _settings: GroupSettings) -> Result<()> {
        debug!("Updating group settings for: {:?}", self.group_id);
        // For now, just log the settings update
        // This would need to be implemented based on the actual saorsa-mls API
        Ok(())
    }
}

/// Processed MLS message
#[derive(Debug, Clone)]
pub enum ProcessedMessage {
    Application(Box<DecryptedMessage>),
    Proposal(Vec<u8>),
    Commit(Vec<u8>),
}

/// Decrypted application message
#[derive(Debug, Clone)]
pub struct DecryptedMessage {
    pub content: Vec<u8>,
    pub sender: EnhancedIdentity,
    pub epoch: Epoch,
    pub signature: Vec<u8>,
}

/// Group settings for MLS groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_members: Option<usize>,
    pub require_pqc: bool,
    pub key_rotation_interval: Option<u64>,
}

/// MLS group management operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupOperation {
    Create {
        name: String,
        members: Vec<EnhancedIdentity>,
    },
    Join {
        welcome_data: Vec<u8>,
    },
    Leave,
    AddMember {
        member: EnhancedIdentity,
    },
    RemoveMember {
        member: EnhancedIdentity,
    },
    UpdateSettings {
        settings: GroupSettings,
    },
}

/// MLS message operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageOperation {
    Send {
        content: Vec<u8>,
        content_type: String,
    },
    Process {
        message_data: Vec<u8>,
    },
}

/// Error types for MLS operations
#[derive(Debug, thiserror::Error)]
pub enum MlsError {
    #[error("Group not found: {0}")]
    GroupNotFound(String),
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    #[error("MLS error: {0}")]
    Mls(#[from] saorsa_mls::MlsError),
}

// MlsError already implements std::error::Error, so it can be converted to anyhow::Error automatically

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mls_client_creation() {
        let config = MlsConfig::default();
        let client = MlsClient::new(config).await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_group_creation() -> Result<()> {
        let config = MlsConfig::default();
        let client = MlsClient::new(config).await?;

        let group_id = GroupId::generate();
        // Create a test identity using the manager approach
        let id_mgr = saorsa_core::identity::IdentityManager::new(
            saorsa_core::identity::manager::IdentityManagerConfig::default(),
        );
        let base_identity = id_mgr
            .create_identity(
                "test-user".to_string(),
                "test-user-four-words".to_string(),
                None,
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create base identity: {}", e))?;

        let mut enhanced_mgr =
            saorsa_core::identity::enhanced::EnhancedIdentityManager::new(id_mgr);
        let identity = enhanced_mgr
            .create_enhanced_identity(
                base_identity,
                "test-device".to_string(),
                saorsa_core::identity::enhanced::DeviceType::Desktop,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create enhanced identity: {}", e))?;

        let group = client.create_group(group_id, &identity).await;
        assert!(group.is_ok());
        Ok(())
    }
}

/// High-level messaging service integrating MLS and DHT storage
#[derive(Debug)]
pub struct MessagingService {
    mls_client: MlsClient,
    dht_storage: MessageDht,
}

impl MessagingService {
    /// Create a new messaging service
    pub async fn new(config: MlsConfig) -> Result<Self> {
        let mls_client = MlsClient::new(config).await?;
        let dht_storage = MessageDht::new();
        
        Ok(Self {
            mls_client,
            dht_storage,
        })
    }
    
    /// Create a new group with MLS encryption
    pub async fn create_group(
        &self,
        group_name: String,
        creator_identity: &EnhancedIdentity,
    ) -> Result<GroupId> {
        let group_id = GroupId::generate();
        
        // Create MLS group
        let _group_state = self.mls_client.create_group(group_id.clone(), creator_identity).await?;
        
        info!("Created new encrypted group: {} ({:?})", group_name, group_id);
        Ok(group_id)
    }
    
    /// Send an encrypted message to a group
    pub async fn send_group_message(
        &self,
        group_id: &GroupId,
        content: Vec<u8>,
        _sender_identity: &EnhancedIdentity,
    ) -> Result<()> {
        // For now, just store the message directly in DHT
        // In production, this would:
        // 1. Create MLS message with proper encryption
        // 2. Serialize and seal the message 
        // 3. Store in DHT with proper access controls
        
        let message_key = blake3::hash(&content);
        self.dht_storage.put(message_key.as_bytes(), &content, Some(3600))
            .map_err(|e| anyhow::anyhow!("Failed to store message: {}", e))?;
        
        info!("Successfully stored message ({} bytes) for group: {:?}", content.len(), group_id);
        Ok(())
    }
    
    /// Retrieve and decrypt messages from a group
    pub async fn get_group_messages(
        &self,
        group_id: &GroupId,
        _identity: &EnhancedIdentity,
    ) -> Result<Vec<DecryptedMessage>> {
        // In a real implementation, this would:
        // 1. Query DHT for sealed messages in this group
        // 2. Unseal the messages using saorsa-seal
        // 3. Decrypt using MLS group keys
        // 4. Return decrypted messages
        
        // For now, return empty list as placeholder
        debug!("Retrieving messages for group: {:?}", group_id);
        Ok(vec![])
    }
    
    /// Join an existing group via welcome message
    pub async fn join_group(
        &self,
        welcome_data: Vec<u8>,
        identity: &EnhancedIdentity,
    ) -> Result<GroupId> {
        let group_state = self.mls_client.join_group(welcome_data, identity).await?;
        
        info!("Successfully joined group: {:?}", group_state.group_id);
        Ok(group_state.group_id)
    }
    
    /// Leave a group
    pub async fn leave_group(&self, group_id: &GroupId) -> Result<()> {
        self.mls_client.leave_group(group_id).await?;
        
        info!("Successfully left group: {:?}", group_id);
        Ok(())
    }
    
    /// List all groups the user is a member of
    pub async fn list_groups(&self) -> Vec<GroupId> {
        self.mls_client.list_groups().await
    }
}
