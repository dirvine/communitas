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

mod group;
mod message;
mod service;

pub use group::{Group, GroupId};
pub use message::{Message, MessageContent, MessageId};
pub use service::ChatService;

/// Chat-related errors
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    /// Group not found
    #[error("Group not found: {0}")]
    GroupNotFound(String),

    /// Message delivery failed
    #[error("Message delivery failed: {0}")]
    DeliveryFailed(String),

    /// Invalid group size
    #[error("Group size exceeds maximum of 20 participants")]
    GroupSizeLimitExceeded,

    /// Storage error
    #[error("Storage error: {0}")]
    Storage(#[from] sled::Error),
}
