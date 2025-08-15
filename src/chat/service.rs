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


//! Chat service implementation

use super::{ChatError, Group, GroupId, Message, MessageContent, MessageId};
use crate::network::NetworkIntegration;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main chat service
pub struct ChatService {
    /// Active groups
    groups: Arc<RwLock<HashMap<GroupId, Group>>>,
    /// Message store
    message_store: Arc<sled::Db>,
    /// Network integration
    network: Arc<NetworkIntegration>,
}

impl std::fmt::Debug for ChatService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChatService")
            .field("groups", &"Arc<RwLock<HashMap<GroupId, Group>>>")
            .field("message_store", &"Arc<sled::Db>")
            .field("network", &"Arc<NetworkIntegration>")
            .finish()
    }
}

impl ChatService {
    /// Create new chat service
    pub async fn new(network: Arc<NetworkIntegration>) -> Result<Self> {
        let db_path = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?
            .join("communitas")
            .join("messages");

        std::fs::create_dir_all(&db_path)?;
        let message_store = Arc::new(sled::open(db_path)?);

        Ok(Self {
            groups: Arc::new(RwLock::new(HashMap::new())),
            message_store,
            network,
        })
    }

    /// Create a new group
    pub async fn create_group(&self, name: &str) -> Result<Group> {
        let group = Group::new(name.to_string());

        let mut groups = self.groups.write().await;
        groups.insert(group.id.clone(), group.clone());

        // Store in DHT
        self.network.store_group(&group).await?;

        Ok(group)
    }

    /// Send a message to a group
    pub async fn send_message(&self, group_id: &str, content: &str) -> Result<MessageId> {
        let groups = self.groups.read().await;
        let group_id_obj = GroupId(group_id.to_string());
        let group = groups
            .get(&group_id_obj)
            .ok_or_else(|| ChatError::GroupNotFound(group_id.to_string()))?;

        let message = Message::new(
            self.network.get_identity().await?,
            MessageContent::Text(content.to_string()),
        );

        // Store locally
        self.store_message(&message, group_id).await?;

        // Send to network
        self.network.broadcast_message(&message, group).await?;

        Ok(message.id.clone())
    }

    /// Store message locally
    async fn store_message(&self, message: &Message, group_id: &str) -> Result<()> {
        let key = format!("{}:{}", group_id, message.id.0);
        let value = serde_json::to_vec(message)?;
        self.message_store.insert(key, value)?;
        Ok(())
    }
}
