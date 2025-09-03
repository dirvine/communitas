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

//! Message types and structures

use crate::Identity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Unique message identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MessageId(pub String);

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique identifier
    pub id: MessageId,
    /// Sender identity
    pub sender: Identity,
    /// Message content
    pub content: MessageContent,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Digital signatures
    pub signatures: Vec<Vec<u8>>,
}

/// Message content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    /// Text message
    Text(String),
    /// File attachment
    File(FileMetadata),
    /// Voice call invitation
    VoiceCall(CallInfo),
    /// Video call invitation
    VideoCall(CallInfo),
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// File name
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// BLAKE3 hash
    pub hash: String,
    /// MIME type
    pub mime_type: String,
}

/// Call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfo {
    /// Call ID
    pub call_id: String,
    /// SDP offer/answer
    pub sdp: String,
}

impl Message {
    /// Create a new message
    pub fn new(sender: Identity, content: MessageContent) -> Self {
        Self {
            id: MessageId(format!("msg-{}", chrono::Utc::now().timestamp())),
            sender,
            content,
            timestamp: Utc::now(),
            signatures: Vec::new(),
        }
    }
}
