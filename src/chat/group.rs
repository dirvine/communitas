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


//! Group chat structures

use crate::Identity;
use serde::{Deserialize, Serialize};

/// Group identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct GroupId(pub String);

/// Chat group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Unique identifier
    pub id: GroupId,
    /// Group name
    pub name: String,
    /// Participants
    pub participants: Vec<Identity>,
    /// Threshold public key for group operations
    pub threshold_key: Option<Vec<u8>>,
    /// Group settings
    pub settings: GroupSettings,
}

/// Group settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    /// Maximum participants (default 20)
    pub max_participants: usize,
    /// Message retention in days (default 7)
    pub message_retention_days: u32,
    /// Maximum file size in MB (default 5)
    pub max_file_size_mb: u32,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            max_participants: 20,
            message_retention_days: 7,
            max_file_size_mb: 5,
        }
    }
}

impl Group {
    /// Create a new group
    pub fn new(name: String) -> Self {
        Self {
            id: GroupId(format!("group-{}", chrono::Utc::now().timestamp())),
            name,
            participants: Vec::new(),
            threshold_key: None,
            settings: GroupSettings::default(),
        }
    }

    /// Add a participant
    pub fn add_participant(&mut self, identity: Identity) -> Result<(), super::ChatError> {
        if self.participants.len() >= self.settings.max_participants {
            return Err(super::ChatError::GroupSizeLimitExceeded);
        }

        if !self
            .participants
            .iter()
            .any(|p| p.four_word_address == identity.four_word_address)
        {
            self.participants.push(identity);
        }

        Ok(())
    }
}
