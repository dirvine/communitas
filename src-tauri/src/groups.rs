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

// Copyright 2024 P2P Foundation
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Group messaging module for Communitas
//! Implements Slack-like group chat without channels

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: String,
    pub admin: String,        // Four-word address of admin
    pub members: Vec<String>, // Four-word addresses
    pub created_at: i64,
    pub updated_at: i64,
}

/// Message in a group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub id: String,
    pub group_id: String,
    pub sender: String, // Four-word address
    pub sender_name: String,
    pub content: MessageContent,
    pub timestamp: i64,
    pub reply_to: Option<String>,
    pub reactions: Vec<Reaction>,
    pub edited: bool,
    pub edited_at: Option<i64>,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text {
        text: String,
    },
    Image {
        hash: String,
        caption: Option<String>,
        mime_type: String,
    },
    File {
        hash: String,
        filename: String,
        size: u64,
        mime_type: String,
    },
    VoiceNote {
        hash: String,
        duration: u32,
    },
    System {
        message: String,
    },
}

/// Message reaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: String,
    pub users: Vec<String>, // Four-word addresses
}

/// Group manager
#[derive(Debug)]
pub struct GroupManager {
    groups: Arc<RwLock<HashMap<String, Group>>>,
    messages: Arc<RwLock<HashMap<String, Vec<GroupMessage>>>>,
    user_groups: Arc<RwLock<HashMap<String, Vec<String>>>>, // user -> group IDs
}

impl GroupManager {
    /// Create a new group manager
    pub fn new() -> Self {
        Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            user_groups: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new group
    pub async fn create_group(
        &self,
        name: String,
        description: String,
        admin: String,
    ) -> Result<Group> {
        let group = Group {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            admin: admin.clone(),
            members: vec![admin.clone()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        // Store group
        self.groups
            .write()
            .await
            .insert(group.id.clone(), group.clone());

        // Update user's group list
        let mut user_groups = self.user_groups.write().await;
        user_groups
            .entry(admin)
            .or_insert_with(Vec::new)
            .push(group.id.clone());

        // Initialize empty message list
        self.messages
            .write()
            .await
            .insert(group.id.clone(), Vec::new());

        Ok(group)
    }

    /// Add member to group
    pub async fn add_member(&self, group_id: String, member: String) -> Result<()> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(&group_id).context("Group not found")?;

        if !group.members.contains(&member) {
            group.members.push(member.clone());
            group.updated_at = chrono::Utc::now().timestamp();

            // Update user's group list
            let mut user_groups = self.user_groups.write().await;
            user_groups
                .entry(member)
                .or_insert_with(Vec::new)
                .push(group_id.clone());
        }

        Ok(())
    }

    /// Remove member from group
    #[allow(dead_code)]
    pub async fn remove_member(&self, group_id: String, member: String) -> Result<()> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(&group_id).context("Group not found")?;

        group.members.retain(|m| m != &member);
        group.updated_at = chrono::Utc::now().timestamp();

        // Update user's group list
        let mut user_groups = self.user_groups.write().await;
        if let Some(groups) = user_groups.get_mut(&member) {
            groups.retain(|g| g != &group_id);
        }

        Ok(())
    }

    /// Send message to group
    pub async fn send_message(
        &self,
        group_id: String,
        sender: String,
        sender_name: String,
        content: MessageContent,
        reply_to: Option<String>,
    ) -> Result<GroupMessage> {
        // Verify sender is member
        let groups = self.groups.read().await;
        let group = groups.get(&group_id).context("Group not found")?;

        if !group.members.contains(&sender) {
            anyhow::bail!("Sender is not a member of this group");
        }

        let message = GroupMessage {
            id: Uuid::new_v4().to_string(),
            group_id: group_id.clone(),
            sender,
            sender_name,
            content,
            timestamp: chrono::Utc::now().timestamp_millis(),
            reply_to,
            reactions: Vec::new(),
            edited: false,
            edited_at: None,
        };

        // Store message
        let mut messages = self.messages.write().await;
        messages
            .entry(group_id)
            .or_insert_with(Vec::new)
            .push(message.clone());

        Ok(message)
    }

    /// Get messages for a group
    pub async fn get_messages(
        &self,
        group_id: String,
        limit: usize,
        before: Option<i64>,
    ) -> Result<Vec<GroupMessage>> {
        let messages = self.messages.read().await;
        let group_messages = messages.get(&group_id).context("Group not found")?;

        let filtered: Vec<GroupMessage> = if let Some(before_ts) = before {
            group_messages
                .iter()
                .filter(|m| m.timestamp < before_ts)
                .rev()
                .take(limit)
                .cloned()
                .collect()
        } else {
            group_messages.iter().rev().take(limit).cloned().collect()
        };

        Ok(filtered.into_iter().rev().collect())
    }

    /// Add reaction to message
    pub async fn add_reaction(
        &self,
        message_id: String,
        emoji: String,
        user: String,
    ) -> Result<()> {
        let mut messages = self.messages.write().await;

        for group_messages in messages.values_mut() {
            if let Some(message) = group_messages.iter_mut().find(|m| m.id == message_id) {
                if let Some(reaction) = message.reactions.iter_mut().find(|r| r.emoji == emoji) {
                    if !reaction.users.contains(&user) {
                        reaction.users.push(user);
                    }
                } else {
                    message.reactions.push(Reaction {
                        emoji,
                        users: vec![user],
                    });
                }
                return Ok(());
            }
        }

        anyhow::bail!("Message not found")
    }

    /// Get user's groups
    pub async fn get_user_groups(&self, user: String) -> Result<Vec<Group>> {
        let user_groups = self.user_groups.read().await;
        let group_ids = user_groups.get(&user).cloned().unwrap_or_default();

        let groups = self.groups.read().await;
        let user_group_list: Vec<Group> = group_ids
            .iter()
            .filter_map(|id| groups.get(id).cloned())
            .collect();

        Ok(user_group_list)
    }

    /// Get group by ID
    #[allow(dead_code)]
    pub async fn get_group(&self, group_id: String) -> Result<Group> {
        let groups = self.groups.read().await;
        groups.get(&group_id).cloned().context("Group not found")
    }
}
