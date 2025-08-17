/**
 * Saorsa Storage System - Group Management
 * Implements shared encryption keys and member management for group storage
 */

use crate::saorsa_storage::errors::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit};
use chacha20poly1305::aead::{Aead, AeadCore, OsRng};
use hkdf::Hkdf;
use sha2::Sha256;
// use rand::RngCore;  // Currently unused

/// Group member with permissions and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: String,
    pub public_key: Vec<u8>,
    pub role: GroupRole,
    pub joined_at: DateTime<Utc>,
    pub permissions: GroupPermissions,
}

/// Group member roles
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GroupRole {
    Owner,
    Admin,
    Member,
    ReadOnly,
}

/// Group member permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPermissions {
    pub can_read: bool,
    pub can_write: bool,
    pub can_invite: bool,
    pub can_remove: bool,
    pub can_rotate_keys: bool,
}

impl Default for GroupPermissions {
    fn default() -> Self {
        Self {
            can_read: true,
            can_write: false,
            can_invite: false,
            can_remove: false,
            can_rotate_keys: false,
        }
    }
}

impl GroupPermissions {
    pub fn for_role(role: &GroupRole) -> Self {
        match role {
            GroupRole::Owner => Self {
                can_read: true,
                can_write: true,
                can_invite: true,
                can_remove: true,
                can_rotate_keys: true,
            },
            GroupRole::Admin => Self {
                can_read: true,
                can_write: true,
                can_invite: true,
                can_remove: true,
                can_rotate_keys: false,
            },
            GroupRole::Member => Self {
                can_read: true,
                can_write: true,
                can_invite: false,
                can_remove: false,
                can_rotate_keys: false,
            },
            GroupRole::ReadOnly => Self {
                can_read: true,
                can_write: false,
                can_invite: false,
                can_remove: false,
                can_rotate_keys: false,
            },
        }
    }
}

/// Group encryption key with metadata
#[derive(Debug, Clone)]
pub struct GroupKey {
    pub key_id: String,
    pub key_material: [u8; 32],
    pub created_at: DateTime<Utc>,
    pub version: u32,
    pub created_by: String,
}

/// Encrypted group key for distribution to members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedGroupKey {
    pub key_id: String,
    pub encrypted_key: Vec<u8>,
    pub recipient_id: String,
    pub nonce: [u8; 12],
    pub version: u32,
}

/// Group metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupInfo {
    pub group_id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub max_members: u32,
    pub key_rotation_interval_days: u32,
    pub require_approval: bool,
}

/// Group key rotation status
#[derive(Debug, Clone)]
pub struct KeyRotationStatus {
    pub in_progress: bool,
    pub started_at: Option<DateTime<Utc>>,
    pub completion_percentage: f32,
    pub failed_members: Vec<String>,
}

/// Group manager for shared encryption and member management
pub struct GroupManager {
    groups: Arc<RwLock<HashMap<String, GroupInfo>>>,
    members: Arc<RwLock<HashMap<String, Vec<GroupMember>>>>,
    group_keys: Arc<RwLock<HashMap<String, GroupKey>>>,
    encrypted_keys: Arc<RwLock<HashMap<String, Vec<EncryptedGroupKey>>>>,
    key_rotations: Arc<RwLock<HashMap<String, KeyRotationStatus>>>,
    master_key: [u8; 32],
}

impl GroupManager {
    /// Create a new group manager with master key for key derivation
    pub fn new(master_key: [u8; 32]) -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            members: Arc::new(RwLock::new(HashMap::new())),
            group_keys: Arc::new(RwLock::new(HashMap::new())),
            encrypted_keys: Arc::new(RwLock::new(HashMap::new())),
            key_rotations: Arc::new(RwLock::new(HashMap::new())),
            master_key,
        }
    }

    /// Create a new group with the creator as owner
    pub async fn create_group(
        &self,
        group_id: &str,
        name: &str,
        description: &str,
        creator_id: &str,
        creator_public_key: &[u8],
    ) -> GroupResult<()> {
        // Check if group already exists
        let groups = self.groups.read().await;
        if groups.contains_key(group_id) {
            return Err(GroupError::GroupAlreadyExists {
                group_id: group_id.to_string(),
            });
        }
        drop(groups);

        // Create group info
        let group_info = GroupInfo {
            group_id: group_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            created_by: creator_id.to_string(),
            max_members: 100, // Default limit
            key_rotation_interval_days: 90, // Rotate every 3 months
            require_approval: false,
        };

        // Create owner member
        let owner = GroupMember {
            user_id: creator_id.to_string(),
            public_key: creator_public_key.to_vec(),
            role: GroupRole::Owner,
            joined_at: Utc::now(),
            permissions: GroupPermissions::for_role(&GroupRole::Owner),
        };

        // Generate initial group key
        let group_key = self.generate_group_key(group_id, creator_id, 1)?;

        // Encrypt key for owner
        let encrypted_key = self.encrypt_key_for_member(&group_key, &owner)?;

        // Store everything
        let mut groups = self.groups.write().await;
        groups.insert(group_id.to_string(), group_info);

        let mut members = self.members.write().await;
        members.insert(group_id.to_string(), vec![owner]);

        let mut group_keys = self.group_keys.write().await;
        group_keys.insert(group_id.to_string(), group_key);

        let mut encrypted_keys = self.encrypted_keys.write().await;
        encrypted_keys.insert(group_id.to_string(), vec![encrypted_key]);

        Ok(())
    }

    /// Add a member to a group
    pub async fn add_member(
        &self,
        group_id: &str,
        user_id: &str,
        public_key: &[u8],
        role: GroupRole,
        added_by: &str,
    ) -> GroupResult<()> {
        // Verify group exists
        let groups = self.groups.read().await;
        if !groups.contains_key(group_id) {
            return Err(GroupError::GroupNotFound {
                group_id: group_id.to_string(),
            });
        }
        drop(groups);

        // Check if user adding has permission
        self.verify_permission(group_id, added_by, "can_invite").await?;

        // Check if user is already a member
        let mut members = self.members.write().await;
        if let Some(group_members) = members.get(group_id) {
            if group_members.iter().any(|m| m.user_id == user_id) {
                return Err(GroupError::UserNotMember {
                    user_id: format!("User {} already exists", user_id),
                    group_id: group_id.to_string(),
                });
            }

            // Check group size limit
            let group_info = self.groups.read().await;
            if let Some(info) = group_info.get(group_id) {
                if group_members.len() >= info.max_members as usize {
                    return Err(GroupError::MaxGroupSizeExceeded);
                }
            }
        }

        // Create new member
        let new_member = GroupMember {
            user_id: user_id.to_string(),
            public_key: public_key.to_vec(),
            role: role.clone(),
            joined_at: Utc::now(),
            permissions: GroupPermissions::for_role(&role),
        };

        // Get current group key and encrypt for new member
        let group_keys = self.group_keys.read().await;
        if let Some(group_key) = group_keys.get(group_id) {
            let encrypted_key = self.encrypt_key_for_member(group_key, &new_member)?;

            // Add member and encrypted key
            if let Some(group_members) = members.get_mut(group_id) {
                group_members.push(new_member);
            }

            let mut encrypted_keys = self.encrypted_keys.write().await;
            if let Some(keys) = encrypted_keys.get_mut(group_id) {
                keys.push(encrypted_key);
            }
        } else {
            return Err(GroupError::GroupKeyNotFound {
                group_id: group_id.to_string(),
            });
        }

        Ok(())
    }

    /// Remove a member from a group
    pub async fn remove_member(
        &self,
        group_id: &str,
        user_id: &str,
        removed_by: &str,
    ) -> GroupResult<()> {
        // Check permissions
        self.verify_permission(group_id, removed_by, "can_remove").await?;

        // Remove member
        let mut members = self.members.write().await;
        if let Some(group_members) = members.get_mut(group_id) {
            let initial_len = group_members.len();
            group_members.retain(|m| m.user_id != user_id);
            
            if group_members.len() == initial_len {
                return Err(GroupError::UserNotMember {
                    user_id: user_id.to_string(),
                    group_id: group_id.to_string(),
                });
            }
        }

        // Remove encrypted keys for this user
        let mut encrypted_keys = self.encrypted_keys.write().await;
        if let Some(keys) = encrypted_keys.get_mut(group_id) {
            keys.retain(|k| k.recipient_id != user_id);
        }

        Ok(())
    }

    /// Get group key for a member (decrypt their encrypted copy)
    pub async fn get_group_key(
        &self,
        group_id: &str,
        user_id: &str,
        user_private_key: &[u8; 32],
    ) -> GroupResult<[u8; 32]> {
        // Verify user is member
        self.verify_member(group_id, user_id).await?;

        // Find encrypted key for this user
        let encrypted_keys = self.encrypted_keys.read().await;
        if let Some(keys) = encrypted_keys.get(group_id) {
            for encrypted_key in keys {
                if encrypted_key.recipient_id == user_id {
                    return self.decrypt_group_key(encrypted_key, user_private_key);
                }
            }
        }

        Err(GroupError::GroupKeyNotFound {
            group_id: group_id.to_string(),
        })
    }

    /// Rotate group key (requires owner/admin permission)
    pub async fn rotate_group_key(
        &self,
        group_id: &str,
        rotated_by: &str,
    ) -> GroupResult<()> {
        // Check permissions
        self.verify_permission(group_id, rotated_by, "can_rotate_keys").await?;

        // Start rotation process
        let mut rotations = self.key_rotations.write().await;
        rotations.insert(group_id.to_string(), KeyRotationStatus {
            in_progress: true,
            started_at: Some(Utc::now()),
            completion_percentage: 0.0,
            failed_members: Vec::new(),
        });
        drop(rotations);

        // Get current version
        let group_keys = self.group_keys.read().await;
        let current_version = group_keys
            .get(group_id)
            .map(|k| k.version)
            .unwrap_or(0);
        drop(group_keys);

        // Generate new key
        let new_key = self.generate_group_key(group_id, rotated_by, current_version + 1)?;

        // Get all members for re-encryption
        let members = self.members.read().await;
        let group_members = members.get(group_id)
            .ok_or_else(|| GroupError::GroupNotFound {
                group_id: group_id.to_string(),
            })?
            .clone();
        drop(members);

        // Encrypt new key for all members
        let mut new_encrypted_keys = Vec::new();
        let mut failed_members = Vec::new();

        for member in &group_members {
            match self.encrypt_key_for_member(&new_key, member) {
                Ok(encrypted_key) => new_encrypted_keys.push(encrypted_key),
                Err(_) => failed_members.push(member.user_id.clone()),
            }
        }

        // Update rotation status
        let completion_percentage = if group_members.is_empty() {
            100.0
        } else {
            (new_encrypted_keys.len() as f32 / group_members.len() as f32) * 100.0
        };

        let mut rotations = self.key_rotations.write().await;
        if let Some(status) = rotations.get_mut(group_id) {
            status.completion_percentage = completion_percentage;
            status.failed_members = failed_members;
            status.in_progress = completion_percentage < 100.0;
        }

        // Update keys if successful
        if completion_percentage == 100.0 {
            let mut group_keys = self.group_keys.write().await;
            group_keys.insert(group_id.to_string(), new_key);

            let mut encrypted_keys = self.encrypted_keys.write().await;
            encrypted_keys.insert(group_id.to_string(), new_encrypted_keys);
        }

        Ok(())
    }

    /// Get group members list
    pub async fn get_members(&self, group_id: &str) -> GroupResult<Vec<GroupMember>> {
        let members = self.members.read().await;
        members
            .get(group_id)
            .cloned()
            .ok_or_else(|| GroupError::GroupNotFound {
                group_id: group_id.to_string(),
            })
    }

    /// Get group information
    pub async fn get_group_info(&self, group_id: &str) -> GroupResult<GroupInfo> {
        let groups = self.groups.read().await;
        groups
            .get(group_id)
            .cloned()
            .ok_or_else(|| GroupError::GroupNotFound {
                group_id: group_id.to_string(),
            })
    }

    /// Check if user is member of group
    pub async fn is_member(&self, group_id: &str, user_id: &str) -> bool {
        let members = self.members.read().await;
        if let Some(group_members) = members.get(group_id) {
            group_members.iter().any(|m| m.user_id == user_id)
        } else {
            false
        }
    }

    /// Get groups where user is a member
    pub async fn get_user_groups(&self, user_id: &str) -> Vec<String> {
        let members = self.members.read().await;
        let mut user_groups = Vec::new();

        for (group_id, group_members) in members.iter() {
            if group_members.iter().any(|m| m.user_id == user_id) {
                user_groups.push(group_id.clone());
            }
        }

        user_groups
    }

    // Private helper methods

    fn generate_group_key(
        &self,
        group_id: &str,
        created_by: &str,
        version: u32,
    ) -> GroupResult<GroupKey> {
        // Derive deterministic but secure group key
        let hkdf = Hkdf::<Sha256>::new(None, &self.master_key);
        let info = format!("group:{}:v{}", group_id, version);
        
        let mut key_material = [0u8; 32];
        hkdf.expand(info.as_bytes(), &mut key_material)
            .map_err(|_| GroupError::KeyWrappingFailed)?;

        Ok(GroupKey {
            key_id: format!("{}:v{}", group_id, version),
            key_material,
            created_at: Utc::now(),
            version,
            created_by: created_by.to_string(),
        })
    }

    fn encrypt_key_for_member(
        &self,
        group_key: &GroupKey,
        member: &GroupMember,
    ) -> GroupResult<EncryptedGroupKey> {
        // Use member's public key for encryption (simplified - would use proper key exchange)
        let member_key = Key::from_slice(&member.public_key[..32.min(member.public_key.len())]);
        let cipher = ChaCha20Poly1305::new(member_key);
        
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, group_key.key_material.as_ref())
            .map_err(|_| GroupError::KeyWrappingFailed)?;

        Ok(EncryptedGroupKey {
            key_id: group_key.key_id.clone(),
            encrypted_key: ciphertext,
            recipient_id: member.user_id.clone(),
            nonce: nonce.into(),
            version: group_key.version,
        })
    }

    fn decrypt_group_key(
        &self,
        encrypted_key: &EncryptedGroupKey,
        user_private_key: &[u8; 32],
    ) -> GroupResult<[u8; 32]> {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(user_private_key));
        let nonce = Nonce::from_slice(&encrypted_key.nonce);
        
        let plaintext = cipher
            .decrypt(nonce, encrypted_key.encrypted_key.as_ref())
            .map_err(|_| GroupError::KeyUnwrappingFailed)?;

        if plaintext.len() != 32 {
            return Err(GroupError::KeyUnwrappingFailed);
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&plaintext);
        Ok(key)
    }

    async fn verify_member(&self, group_id: &str, user_id: &str) -> GroupResult<()> {
        if !self.is_member(group_id, user_id).await {
            return Err(GroupError::UserNotMember {
                user_id: user_id.to_string(),
                group_id: group_id.to_string(),
            });
        }
        Ok(())
    }

    async fn verify_permission(
        &self,
        group_id: &str,
        user_id: &str,
        permission: &str,
    ) -> GroupResult<()> {
        let members = self.members.read().await;
        if let Some(group_members) = members.get(group_id) {
            for member in group_members {
                if member.user_id == user_id {
                    let has_permission = match permission {
                        "can_invite" => member.permissions.can_invite,
                        "can_remove" => member.permissions.can_remove,
                        "can_rotate_keys" => member.permissions.can_rotate_keys,
                        "can_write" => member.permissions.can_write,
                        "can_read" => member.permissions.can_read,
                        _ => false,
                    };

                    if has_permission {
                        return Ok(());
                    } else {
                        return Err(GroupError::InsufficientPermissions);
                    }
                }
            }
        }

        Err(GroupError::UserNotMember {
            user_id: user_id.to_string(),
            group_id: group_id.to_string(),
        })
    }
}

// Thread-safe implementations
unsafe impl Send for GroupManager {}
unsafe impl Sync for GroupManager {}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_manager() -> GroupManager {
        let master_key = [42u8; 32];
        GroupManager::new(master_key)
    }

    #[tokio::test]
    async fn test_create_group() {
        let manager = setup_manager();
        let creator_key = [1u8; 32];

        let result = manager
            .create_group("team1", "Team 1", "Test team", "alice", &creator_key)
            .await;
        assert!(result.is_ok());

        let info = manager.get_group_info("team1").await.unwrap();
        assert_eq!(info.name, "Team 1");
        assert_eq!(info.created_by, "alice");
    }

    #[tokio::test]
    async fn test_add_remove_member() {
        let manager = setup_manager();
        let creator_key = [1u8; 32];
        let member_key = [2u8; 32];

        // Create group
        manager
            .create_group("team1", "Team 1", "Test team", "alice", &creator_key)
            .await
            .unwrap();

        // Add member
        let result = manager
            .add_member("team1", "bob", &member_key, GroupRole::Member, "alice")
            .await;
        assert!(result.is_ok());

        // Verify member exists
        assert!(manager.is_member("team1", "bob").await);

        // Remove member
        let result = manager.remove_member("team1", "bob", "alice").await;
        assert!(result.is_ok());

        // Verify member removed
        assert!(!manager.is_member("team1", "bob").await);
    }

    #[tokio::test]
    async fn test_group_key_rotation() {
        let manager = setup_manager();
        let creator_key = [1u8; 32];

        // Create group
        manager
            .create_group("team1", "Team 1", "Test team", "alice", &creator_key)
            .await
            .unwrap();

        // Rotate key
        let result = manager.rotate_group_key("team1", "alice").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_permissions() {
        let owner_perms = GroupPermissions::for_role(&GroupRole::Owner);
        assert!(owner_perms.can_rotate_keys);
        assert!(owner_perms.can_remove);

        let member_perms = GroupPermissions::for_role(&GroupRole::Member);
        assert!(!member_perms.can_rotate_keys);
        assert!(member_perms.can_read);
    }
}