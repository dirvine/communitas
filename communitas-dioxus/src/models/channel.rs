// SPDX-License-Identifier: MIT OR Apache-2.0

//! Channel and thread messaging data types.
//!
//! These types map to the JSON wire format used in gossip payloads
//! and the GUI-compatible x0x channel store metadata.

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_root: Option<String>,
    /// Whether this thread reply was also broadcast to the channel.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub broadcast: bool,
    /// Local UI state: true when a delete event has been applied.
    /// Not part of the wire payload.
    #[serde(default, skip_serializing)]
    pub is_deleted: bool,
    /// Local UI state, not part of the frozen x0x wire payload.
    #[serde(default, skip_serializing)]
    pub reply_count: u32,
    /// Local UI state, not part of the frozen x0x wire payload.
    #[serde(default, skip_serializing)]
    pub reactions: HashMap<String, u32>,
}

/// Canonical channel metadata stored in `channels_index`.
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
}

/// Legacy pre-freeze schema kept only for one-time compatibility migration.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub(crate) struct LegacyChannelIndex {
    #[serde(default)]
    pub channels: Vec<String>,
    #[serde(default)]
    pub categories: HashMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_message_serialization_omits_ui_only_fields() {
        let message = ChatMessage {
            id: "msg-1".to_string(),
            text: "hello".to_string(),
            sender_name: "Alice".to_string(),
            sender_id: "agent-1".to_string(),
            timestamp: 1710000000000,
            channel: "general".to_string(),
            thread_root: None,
            broadcast: false,
            is_deleted: false,
            reply_count: 3,
            reactions: HashMap::from([(":wave:".to_string(), 1)]),
        };

        let value = serde_json::to_value(&message).expect("message should serialize");

        assert_eq!(value["id"], "msg-1");
        assert_eq!(value["channel"], "general");
        assert!(value.get("thread_root").is_none());
        assert!(value.get("broadcast").is_none());
        assert!(value.get("reply_count").is_none());
        assert!(value.get("reactions").is_none());
    }

    #[test]
    fn thread_reply_serialization_keeps_frozen_contract_fields() {
        let message = ChatMessage {
            id: "msg-2".to_string(),
            text: "reply".to_string(),
            sender_name: "Alice".to_string(),
            sender_id: "agent-1".to_string(),
            timestamp: 1710000000001,
            channel: "general".to_string(),
            thread_root: Some("root-1".to_string()),
            broadcast: true,
            is_deleted: false,
            reply_count: 0,
            reactions: HashMap::new(),
        };

        let value = serde_json::to_value(&message).expect("thread reply should serialize");

        assert_eq!(value["thread_root"], "root-1");
        assert_eq!(value["broadcast"], true);
        assert_eq!(value["channel"], "general");
        assert!(value.get("reply_count").is_none());
        assert!(value.get("reactions").is_none());
    }
}
