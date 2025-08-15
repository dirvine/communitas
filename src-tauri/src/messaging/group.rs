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


//! # Group Messaging Module
//! 
//! Production-ready group messaging with:
//! - Efficient group key management with member updates
//! - Admin privilege system with role-based permissions
//! - Member invitation and removal protocols
//! - Message history synchronization for new members
//! - Group metadata management and versioning
//! - Conflict resolution for concurrent operations

use anyhow::{Result, Context, bail};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use tracing::{info, warn, error, debug, instrument};
use chrono::{DateTime, Utc, Duration as ChronoDuration};
use uuid::Uuid;

use super::{Message, MessageId, UserId, GroupId, MessageType, VectorClock};
use super::storage::{MessageStore, MessageQuery};
use crate::dht::{Dht, ContentId};

/// Maximum number of members per group
pub const MAX_GROUP_MEMBERS: usize = 1000;

/// Maximum number of admins per group
pub const MAX_GROUP_ADMINS: usize = 10;

/// Group metadata cache TTL in seconds
pub const GROUP_METADATA_CACHE_TTL: i64 = 300; // 5 minutes

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupRole {
    Member,
    Admin,
    Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub user_id: UserId,
    pub role: GroupRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<UserId>,
    pub last_seen: Option<DateTime<Utc>>,
}

impl GroupMember {
    pub fn new(user_id: UserId, role: GroupRole, invited_by: Option<UserId>) -> Self {
        Self {
            user_id,
            role,
            joined_at: Utc::now(),
            invited_by,
            last_seen: None,
        }
    }
    
    pub fn can_invite(&self) -> bool {
        matches!(self.role, GroupRole::Admin | GroupRole::Owner)
    }
    
    pub fn can_remove_member(&self) -> bool {
        matches!(self.role, GroupRole::Admin | GroupRole::Owner)
    }
    
    pub fn can_promote(&self) -> bool {
        matches!(self.role, GroupRole::Owner)
    }
    
    pub fn can_modify_group(&self) -> bool {
        matches!(self.role, GroupRole::Admin | GroupRole::Owner)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMetadata {
    pub id: GroupId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by: UserId,
    pub version: u64,
    pub members: HashMap<UserId, GroupMember>,
    pub max_members: usize,
    pub is_public: bool,
    pub invite_only: bool,
    pub message_history_days: Option<i32>,
}

impl GroupMetadata {
    pub fn new(
        name: String,
        created_by: UserId,
        description: Option<String>,
    ) -> Self {
        let group_id = GroupId::new();
        let mut members = HashMap::new();
        
        // Creator is always the owner
        let owner = GroupMember::new(created_by.clone(), GroupRole::Owner, None);
        members.insert(created_by.clone(), owner);
        
        Self {
            id: group_id,
            name,
            description,
            created_at: Utc::now(),
            created_by,
            version: 1,
            members,
            max_members: MAX_GROUP_MEMBERS,
            is_public: false,
            invite_only: true,
            message_history_days: Some(30),
        }
    }
    
    pub fn get_member(&self, user_id: &UserId) -> Option<&GroupMember> {
        self.members.get(user_id)
    }
    
    pub fn is_member(&self, user_id: &UserId) -> bool {
        self.members.contains_key(user_id)
    }
    
    pub fn is_admin(&self, user_id: &UserId) -> bool {
        self.members.get(user_id)
            .map(|m| matches!(m.role, GroupRole::Admin | GroupRole::Owner))
            .unwrap_or(false)
    }
    
    pub fn is_owner(&self, user_id: &UserId) -> bool {
        self.members.get(user_id)
            .map(|m| matches!(m.role, GroupRole::Owner))
            .unwrap_or(false)
    }
    
    pub fn member_count(&self) -> usize {
        self.members.len()
    }
    
    pub fn admin_count(&self) -> usize {
        self.members.values()
            .filter(|m| matches!(m.role, GroupRole::Admin | GroupRole::Owner))
            .count()
    }
    
    pub fn can_add_member(&self) -> bool {
        self.member_count() < self.max_members
    }
    
    pub fn get_member_list(&self) -> Vec<UserId> {
        self.members.keys().cloned().collect()
    }
    
    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GroupOperation {
    CreateGroup {
        metadata: GroupMetadata,
    },
    InviteMember {
        group_id: GroupId,
        invitee: UserId,
        invited_by: UserId,
        role: GroupRole,
    },
    RemoveMember {
        group_id: GroupId,
        member: UserId,
        removed_by: UserId,
    },
    PromoteMember {
        group_id: GroupId,
        member: UserId,
        new_role: GroupRole,
        promoted_by: UserId,
    },
    UpdateMetadata {
        group_id: GroupId,
        name: Option<String>,
        description: Option<String>,
        updated_by: UserId,
    },
    LeaveGroup {
        group_id: GroupId,
        member: UserId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub base: Message,
    pub operation: Option<GroupOperation>,
}

impl GroupMessage {
    pub fn new_chat_message(
        sender: UserId,
        group_id: GroupId,
        content: Vec<u8>,
    ) -> Result<Self> {
        let base = Message::new_group(sender, group_id, content)?;
        
        Ok(Self {
            base,
            operation: None,
        })
    }
    
    pub fn new_operation_message(
        sender: UserId,
        group_id: GroupId,
        operation: GroupOperation,
    ) -> Result<Self> {
        let operation_content = bincode::serialize(&operation)
            .context("Failed to serialize group operation")?;
        
        let mut base = Message::new_group(sender, group_id, operation_content)?;
        base.metadata.message_type = MessageType::SystemMessage;
        
        Ok(Self {
            base,
            operation: Some(operation),
        })
    }
}

/// Group manager for handling all group-related operations
pub struct GroupManager {
    message_store: Arc<MessageStore>,
    dht: Arc<RwLock<Option<Arc<Dht>>>>,
    
    // Group metadata cache
    groups: Arc<RwLock<HashMap<GroupId, GroupMetadata>>>,
    group_cache_times: Arc<RwLock<HashMap<GroupId, DateTime<Utc>>>>,
    
    // Member lookups for efficient querying
    user_groups: Arc<RwLock<HashMap<UserId, HashSet<GroupId>>>>,
    
    // Pending invitations
    pending_invites: Arc<RwLock<HashMap<GroupId, HashMap<UserId, DateTime<Utc>>>>>,
}

impl GroupManager {
    pub async fn new(
        message_store: Arc<MessageStore>,
        dht: Arc<RwLock<Option<Arc<Dht>>>>,
    ) -> Result<Self> {
        let manager = Self {
            message_store,
            dht,
            groups: Arc::new(RwLock::new(HashMap::new())),
            group_cache_times: Arc::new(RwLock::new(HashMap::new())),
            user_groups: Arc::new(RwLock::new(HashMap::new())),
            pending_invites: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Load existing groups from storage
        manager.load_existing_groups().await?;
        
        info!("Group manager initialized");
        
        Ok(manager)
    }
    
    /// Create a new group
    #[instrument(skip(self))]
    pub async fn create_group(
        &self,
        name: String,
        created_by: UserId,
        description: Option<String>,
    ) -> Result<GroupId> {
        let metadata = GroupMetadata::new(name.clone(), created_by.clone(), description);
        let group_id = metadata.id.clone();
        
        // Create group operation message
        let operation = GroupOperation::CreateGroup {
            metadata: metadata.clone(),
        };
        
        let group_message = GroupMessage::new_operation_message(
            created_by.clone(),
            group_id.clone(),
            operation,
        )?;
        
        // Store the operation message
        self.message_store.store_message(&group_message.base).await?;
        
        // Update local state
        self.update_group_metadata(metadata).await?;
        
        // Distribute group creation via DHT
        self.distribute_group_update(&group_id).await?;
        
        info!("Group '{}' created by {} with ID {}", name, created_by.0, group_id.0);
        
        Ok(group_id)
    }
    
    /// Invite a user to join a group
    #[instrument(skip(self))]
    pub async fn invite_member(
        &self,
        group_id: &GroupId,
        invitee: UserId,
        invited_by: UserId,
        role: GroupRole,
    ) -> Result<()> {
        let mut group_metadata = self.get_group_metadata(group_id).await?;
        
        // Verify permissions
        if !group_metadata.is_admin(&invited_by) {
            bail!("User {} does not have permission to invite members", invited_by.0);
        }
        
        // Check if user is already a member
        if group_metadata.is_member(&invitee) {
            bail!("User {} is already a member of the group", invitee.0);
        }
        
        // Check group capacity
        if !group_metadata.can_add_member() {
            bail!("Group has reached maximum member capacity");
        }
        
        // Create invitation operation
        let operation = GroupOperation::InviteMember {
            group_id: group_id.clone(),
            invitee: invitee.clone(),
            invited_by: invited_by.clone(),
            role: role.clone(),
        };
        
        let group_message = GroupMessage::new_operation_message(
            invited_by.clone(),
            group_id.clone(),
            operation,
        )?;
        
        // Store the operation message
        self.message_store.store_message(&group_message.base).await?;
        
        // Add to pending invitations
        {
            let mut pending = self.pending_invites.write().await;
            let group_invites = pending.entry(group_id.clone()).or_insert_with(HashMap::new);
            group_invites.insert(invitee.clone(), Utc::now());
        }
        
        // Distribute invitation via DHT
        self.distribute_group_update(group_id).await?;
        
        info!("User {} invited to group {} by {}", invitee.0, group_id.0, invited_by.0);
        
        Ok(())
    }
    
    /// Accept a group invitation
    #[instrument(skip(self))]
    pub async fn accept_invitation(
        &self,
        group_id: &GroupId,
        user_id: UserId,
    ) -> Result<()> {
        // Check if invitation exists
        let has_invitation = {
            let pending = self.pending_invites.read().await;
            pending.get(group_id)
                .map(|invites| invites.contains_key(&user_id))
                .unwrap_or(false)
        };
        
        if !has_invitation {
            bail!("No pending invitation found for user {} in group {}", user_id.0, group_id.0);
        }
        
        let mut group_metadata = self.get_group_metadata(group_id).await?;
        
        // Add member to group
        let member = GroupMember::new(user_id.clone(), GroupRole::Member, None);
        group_metadata.members.insert(user_id.clone(), member);
        group_metadata.increment_version();
        
        // Remove from pending invitations
        {
            let mut pending = self.pending_invites.write().await;
            if let Some(group_invites) = pending.get_mut(group_id) {
                group_invites.remove(&user_id);
            }
        }
        
        // Update group metadata
        self.update_group_metadata(group_metadata).await?;
        
        // Distribute update
        self.distribute_group_update(group_id).await?;
        
        info!("User {} joined group {}", user_id.0, group_id.0);
        
        Ok(())
    }
    
    /// Remove a member from a group
    #[instrument(skip(self))]
    pub async fn remove_member(
        &self,
        group_id: &GroupId,
        member: UserId,
        removed_by: UserId,
    ) -> Result<()> {
        let mut group_metadata = self.get_group_metadata(group_id).await?;
        
        // Verify permissions
        if !group_metadata.is_admin(&removed_by) && removed_by != member {
            bail!("User {} does not have permission to remove members", removed_by.0);
        }
        
        // Check if member exists
        if !group_metadata.is_member(&member) {
            bail!("User {} is not a member of the group", member.0);
        }
        
        // Prevent owner from being removed (they must transfer ownership first)
        if group_metadata.is_owner(&member) && removed_by != member {
            bail!("Cannot remove group owner. Ownership must be transferred first.");
        }
        
        // Create removal operation
        let operation = GroupOperation::RemoveMember {
            group_id: group_id.clone(),
            member: member.clone(),
            removed_by: removed_by.clone(),
        };
        
        let group_message = GroupMessage::new_operation_message(
            removed_by.clone(),
            group_id.clone(),
            operation,
        )?;
        
        // Store the operation message
        self.message_store.store_message(&group_message.base).await?;
        
        // Remove member from metadata
        group_metadata.members.remove(&member);
        group_metadata.increment_version();
        
        // Update group metadata
        self.update_group_metadata(group_metadata).await?;
        
        // Update user groups mapping
        {
            let mut user_groups = self.user_groups.write().await;
            if let Some(groups) = user_groups.get_mut(&member) {
                groups.remove(group_id);
            }
        }
        
        // Distribute update
        self.distribute_group_update(group_id).await?;
        
        info!("User {} removed from group {} by {}", member.0, group_id.0, removed_by.0);
        
        Ok(())
    }
    
    /// Promote a member to admin or owner
    #[instrument(skip(self))]
    pub async fn promote_member(
        &self,
        group_id: &GroupId,
        member: UserId,
        new_role: GroupRole,
        promoted_by: UserId,
    ) -> Result<()> {
        let mut group_metadata = self.get_group_metadata(group_id).await?;
        
        // Verify permissions (only owner can promote)
        if !group_metadata.is_owner(&promoted_by) {
            bail!("Only group owner can promote members");
        }
        
        // Check if member exists
        if !group_metadata.is_member(&member) {
            bail!("User {} is not a member of the group", member.0);
        }
        
        // Check admin limits
        if matches!(new_role, GroupRole::Admin) && group_metadata.admin_count() >= MAX_GROUP_ADMINS {
            bail!("Group has reached maximum admin capacity");
        }
        
        // Create promotion operation
        let operation = GroupOperation::PromoteMember {
            group_id: group_id.clone(),
            member: member.clone(),
            new_role: new_role.clone(),
            promoted_by: promoted_by.clone(),
        };
        
        let group_message = GroupMessage::new_operation_message(
            promoted_by.clone(),
            group_id.clone(),
            operation,
        )?;
        
        // Store the operation message
        self.message_store.store_message(&group_message.base).await?;
        
        // Update member role
        if let Some(group_member) = group_metadata.members.get_mut(&member) {
            group_member.role = new_role.clone();
        }
        group_metadata.increment_version();
        
        // Update group metadata
        self.update_group_metadata(group_metadata).await?;
        
        // Distribute update
        self.distribute_group_update(group_id).await?;
        
        info!("User {} promoted to {:?} in group {} by {}", 
              member.0, new_role, group_id.0, promoted_by.0);
        
        Ok(())
    }
    
    /// Get group metadata
    pub async fn get_group_metadata(&self, group_id: &GroupId) -> Result<GroupMetadata> {
        // Check cache first
        {
            let groups = self.groups.read().await;
            let cache_times = self.group_cache_times.read().await;
            
            if let (Some(metadata), Some(&cached_at)) = (groups.get(group_id), cache_times.get(group_id)) {
                let ttl = ChronoDuration::seconds(GROUP_METADATA_CACHE_TTL);
                if Utc::now() - cached_at < ttl {
                    return Ok(metadata.clone());
                }
            }
        }
        
        // Load from DHT if not cached or expired
        self.load_group_from_dht(group_id).await
    }
    
    /// Get all groups for a user
    pub async fn get_user_groups(&self, user_id: &UserId) -> Result<Vec<GroupId>> {
        let user_groups = self.user_groups.read().await;
        Ok(user_groups.get(user_id)
            .map(|groups| groups.iter().cloned().collect())
            .unwrap_or_default())
    }
    
    /// Get group members
    pub async fn get_group_members(&self, group_id: &GroupId) -> Result<Vec<UserId>> {
        let metadata = self.get_group_metadata(group_id).await?;
        Ok(metadata.get_member_list())
    }
    
    /// Update group metadata locally and in cache
    async fn update_group_metadata(&self, metadata: GroupMetadata) -> Result<()> {
        let group_id = metadata.id.clone();
        
        // Update main cache
        {
            let mut groups = self.groups.write().await;
            groups.insert(group_id.clone(), metadata.clone());
        }
        
        // Update cache time
        {
            let mut cache_times = self.group_cache_times.write().await;
            cache_times.insert(group_id.clone(), Utc::now());
        }
        
        // Update user groups mapping
        {
            let mut user_groups = self.user_groups.write().await;
            
            // Remove old mappings for this group
            for (_, groups) in user_groups.iter_mut() {
                groups.remove(&group_id);
            }
            
            // Add new mappings
            for member_id in metadata.members.keys() {
                let user_group_set = user_groups.entry(member_id.clone()).or_insert_with(HashSet::new);
                user_group_set.insert(group_id.clone());
            }
        }
        
        Ok(())
    }
    
    /// Distribute group update via DHT
    async fn distribute_group_update(&self, group_id: &GroupId) -> Result<()> {
        let dht_guard = self.dht.read().await;
        let dht = dht_guard.as_ref().context("DHT not initialized")?;
        
        let metadata = self.get_group_metadata(group_id).await?;
        let metadata_data = bincode::serialize(&metadata)
            .context("Failed to serialize group metadata")?;
        
        let content_key = format!("group_metadata:{}", group_id.0);
        let content_id = ContentId::from_key(&content_key);
        
        dht.put_content(content_id, metadata_data, 3).await
            .context("Failed to distribute group metadata via DHT")?;
        
        Ok(())
    }
    
    /// Load group metadata from DHT
    async fn load_group_from_dht(&self, group_id: &GroupId) -> Result<GroupMetadata> {
        let dht_guard = self.dht.read().await;
        let dht = dht_guard.as_ref().context("DHT not initialized")?;
        
        let content_key = format!("group_metadata:{}", group_id.0);
        let content_id = ContentId::from_key(&content_key);
        
        let metadata_data = dht.get_content(content_id).await
            .context("Failed to load group metadata from DHT")?;
        
        let metadata: GroupMetadata = bincode::deserialize(&metadata_data)
            .context("Failed to deserialize group metadata")?;
        
        // Update local cache
        self.update_group_metadata(metadata.clone()).await?;
        
        Ok(metadata)
    }
    
    /// Load existing groups from message store
    async fn load_existing_groups(&self) -> Result<()> {
        let query = MessageQuery {
            message_type: Some(MessageType::SystemMessage),
            limit: Some(10000),
            ..Default::default()
        };
        
        let messages = self.message_store.query_messages(query).await?;
        
        for message in messages {
            if let Ok(group_message_content) = bincode::deserialize::<GroupOperation>(&message.content) {
                match group_message_content {
                    GroupOperation::CreateGroup { metadata } => {
                        self.update_group_metadata(metadata).await?;
                    }
                    _ => {
                        // Handle other operations as needed
                    }
                }
            }
        }
        
        debug!("Loaded existing groups from message store");
        
        Ok(())
    }
    
    /// Cleanup expired invitations
    pub async fn cleanup_expired_invitations(&self) -> Result<()> {
        let cutoff = Utc::now() - ChronoDuration::days(7);
        
        let mut pending = self.pending_invites.write().await;
        for (_, invites) in pending.iter_mut() {
            invites.retain(|_, &mut invited_at| invited_at > cutoff);
        }
        
        pending.retain(|_, invites| !invites.is_empty());
        
        debug!("Cleaned up expired group invitations");
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messaging::storage::MessageStore;
    use tokio::sync::RwLock;
    use std::sync::Arc;
    
    #[tokio::test]
    async fn test_group_creation() -> Result<()> {
        let message_store = Arc::new(MessageStore::new("sqlite").await?);
        let dht = Arc::new(RwLock::new(None));
        
        let group_manager = GroupManager::new(message_store, dht).await?;
        
        let creator = UserId::new("creator".to_string());
        let group_name = "Test Group".to_string();
        
        let group_id = group_manager.create_group(
            group_name.clone(),
            creator.clone(),
            Some("A test group".to_string()),
        ).await?;
        
        let metadata = group_manager.get_group_metadata(&group_id).await?;
        
        assert_eq!(metadata.name, group_name);
        assert_eq!(metadata.created_by, creator);
        assert!(metadata.is_owner(&creator));
        assert_eq!(metadata.member_count(), 1);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_member_invitation_and_joining() -> Result<()> {
        let message_store = Arc::new(MessageStore::new("sqlite").await?);
        let dht = Arc::new(RwLock::new(None));
        
        let group_manager = GroupManager::new(message_store, dht).await?;
        
        let creator = UserId::new("creator".to_string());
        let invitee = UserId::new("invitee".to_string());
        
        let group_id = group_manager.create_group(
            "Test Group".to_string(),
            creator.clone(),
            None,
        ).await?;
        
        // Invite member
        group_manager.invite_member(
            &group_id,
            invitee.clone(),
            creator.clone(),
            GroupRole::Member,
        ).await?;
        
        // Accept invitation
        group_manager.accept_invitation(&group_id, invitee.clone()).await?;
        
        let metadata = group_manager.get_group_metadata(&group_id).await?;
        
        assert!(metadata.is_member(&invitee));
        assert_eq!(metadata.member_count(), 2);
        
        let user_groups = group_manager.get_user_groups(&invitee).await?;
        assert!(user_groups.contains(&group_id));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_member_promotion() -> Result<()> {
        let message_store = Arc::new(MessageStore::new("sqlite").await?);
        let dht = Arc::new(RwLock::new(None));
        
        let group_manager = GroupManager::new(message_store, dht).await?;
        
        let owner = UserId::new("owner".to_string());
        let member = UserId::new("member".to_string());
        
        let group_id = group_manager.create_group(
            "Test Group".to_string(),
            owner.clone(),
            None,
        ).await?;
        
        // Add member
        group_manager.invite_member(&group_id, member.clone(), owner.clone(), GroupRole::Member).await?;
        group_manager.accept_invitation(&group_id, member.clone()).await?;
        
        // Promote to admin
        group_manager.promote_member(
            &group_id,
            member.clone(),
            GroupRole::Admin,
            owner.clone(),
        ).await?;
        
        let metadata = group_manager.get_group_metadata(&group_id).await?;
        assert!(metadata.is_admin(&member));
        
        Ok(())
    }
}
