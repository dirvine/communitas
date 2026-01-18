//! Messaging and presence DTOs for thread lists, messages, and contact status.

use crate::{UnifiedContact, UnifiedEntityType};

/// Thread summary shown in thread list sidebar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThreadSummary {
    /// Unique thread identifier.
    pub thread_id: String,
    /// Entity ID if this is an entity thread (channel, group, etc.).
    pub entity_id: Option<String>,
    /// Entity type for entity threads.
    pub entity_type: Option<UnifiedEntityType>,
    /// Contact ID for direct message threads.
    pub contact_id: Option<String>,
    /// Display name shown in thread list.
    pub display_name: String,
    /// Preview of the last message (truncated).
    pub last_message_preview: String,
    /// Timestamp of last message in Unix milliseconds.
    pub last_message_timestamp: u64,
    /// Number of unread messages.
    pub unread_count: u32,
    /// Whether notifications are muted for this thread.
    pub is_muted: bool,
}

/// A message in a conversation thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Unique message identifier.
    pub id: String,
    /// Thread this message belongs to.
    pub thread_id: String,
    /// Sender's identity ID.
    pub sender_id: String,
    /// Sender's display name for UI rendering.
    pub sender_name: String,
    /// Message text content.
    pub text: String,
    /// Timestamp in Unix milliseconds.
    pub timestamp: u64,
    /// Whether the message has been edited.
    pub edited: bool,
    /// ID of the message being replied to, if any.
    pub reply_to_id: Option<String>,
    /// Reactions on this message.
    pub reactions: Vec<MessageReaction>,
}

/// A reaction on a message (emoji + count).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageReaction {
    /// Emoji character(s) for this reaction.
    pub emoji: String,
    /// Total number of users who reacted with this emoji.
    pub count: u32,
    /// Whether the current user has reacted with this emoji.
    pub reacted_by_me: bool,
}

/// Presence status for contacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresenceStatus {
    /// Status is unknown (not yet received from network).
    #[default]
    Unknown,
    /// Contact is online and active.
    Online,
    /// Contact is online but idle.
    Away,
    /// Contact is online but busy/do-not-disturb.
    Busy,
    /// Contact is offline.
    Offline,
}

/// Contact with presence information for UI rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactWithPresence {
    /// The contact details.
    pub contact: UnifiedContact,
    /// Current presence status.
    pub presence: PresenceStatus,
    /// Last seen timestamp in Unix milliseconds (if offline).
    pub last_seen: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_summary_equality() {
        let t1 = ThreadSummary {
            thread_id: "t1".to_string(),
            entity_id: Some("e1".to_string()),
            entity_type: Some(UnifiedEntityType::Channel),
            contact_id: None,
            display_name: "General".to_string(),
            last_message_preview: "Hello".to_string(),
            last_message_timestamp: 1234567890,
            unread_count: 5,
            is_muted: false,
        };
        let t2 = t1.clone();
        assert_eq!(t1, t2);
    }

    #[test]
    fn message_with_reactions() {
        let msg = Message {
            id: "m1".to_string(),
            thread_id: "t1".to_string(),
            sender_id: "u1".to_string(),
            sender_name: "Alice".to_string(),
            text: "Hello world".to_string(),
            timestamp: 1234567890,
            edited: false,
            reply_to_id: None,
            reactions: vec![MessageReaction {
                emoji: "👍".to_string(),
                count: 3,
                reacted_by_me: true,
            }],
        };
        assert_eq!(msg.reactions.len(), 1);
        assert!(msg.reactions[0].reacted_by_me);
    }

    #[test]
    fn presence_status_default() {
        let status = PresenceStatus::default();
        assert_eq!(status, PresenceStatus::Unknown);
    }

    #[test]
    fn contact_with_presence_construction() {
        let contact = UnifiedContact {
            id: "alice".to_string(),
            display_name: "Alice".to_string(),
            status: "available".to_string(),
        };
        let cwp = ContactWithPresence {
            contact: contact.clone(),
            presence: PresenceStatus::Online,
            last_seen: None,
        };
        assert_eq!(cwp.contact.id, "alice");
        assert_eq!(cwp.presence, PresenceStatus::Online);
    }
}
