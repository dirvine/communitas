//! Channel and thread messaging data types.
//!
//! These types map to the JSON wire format used in gossip payloads
//! and x0x KvStore metadata for channels and threads.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chat message published on a channel or thread gossip topic.
///
/// Messages are JSON-serialized and base64-encoded before publishing
/// to the gossip layer via `X0xWebSocket::publish`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Unique message identifier (UUID v4).
    pub id: String,
    /// Message text content.
    pub text: String,
    /// Human-readable display name of the sender.
    pub sender_name: String,
    /// Agent ID (hex) of the sender.
    pub sender_id: String,
    /// Unix timestamp in milliseconds.
    pub timestamp: u64,
    /// Channel name this message belongs to.
    pub channel: String,
    /// If this message is a thread reply, the ID of the root message.
    #[serde(default)]
    pub thread_root: Option<String>,
    /// Whether this thread reply was also broadcast to the channel.
    #[serde(default)]
    pub broadcast: bool,
    /// Number of replies in this message's thread (only set on root messages).
    #[serde(default)]
    pub reply_count: u32,
    /// Emoji reactions: emoji -> count.
    #[serde(default)]
    pub reactions: HashMap<String, u32>,
}

/// Metadata for a channel, stored in x0x KvStore with key `channel:{name}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelMeta {
    /// Channel name (URL-safe slug, e.g. "dev", "general").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Agent ID of the channel creator.
    pub creator: String,
    /// Unix timestamp in milliseconds when the channel was created.
    pub created_at: u64,
    /// Full gossip topic string for this channel.
    pub topic: String,
    /// Whether this channel requires an invite to join.
    #[serde(default)]
    pub is_private: bool,
    /// Whether this channel is archived (read-only).
    #[serde(default)]
    pub is_archived: bool,
}

/// Index of all channels in a space, stored in KvStore with key `channels_index`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChannelIndex {
    /// Ordered list of channel names.
    pub channels: Vec<String>,
    /// Channels organized by category name.
    #[serde(default)]
    pub categories: HashMap<String, Vec<String>>,
}
