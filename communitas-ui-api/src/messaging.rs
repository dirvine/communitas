//! Messaging and presence DTOs for thread lists, messages, and contact status.

use crate::{SyncState, UnifiedContact, UnifiedEntityType};

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
    /// Whether this is a direct message thread (1:1 conversation).
    pub is_dm: bool,
    /// Users currently typing in this thread.
    pub typing_users: Vec<String>,
    /// Whether this thread is pinned to the top of the list.
    pub is_pinned: bool,
    /// Presence status for DM threads (None for entity threads).
    pub contact_presence: Option<PresenceStatus>,
    /// Synchronization state for this thread.
    pub sync_state: SyncState,
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
    /// Whether this message has been pinned in the thread.
    pub is_pinned: bool,
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
    /// Whether the contact is currently in a call.
    pub is_in_call: bool,
    /// Entity name of the call (channel/group name) if in a call.
    pub call_entity_name: Option<String>,
    /// Whether the contact is currently screen sharing (presenting).
    pub is_screen_sharing: bool,
}

/// Search result containing a matching message with context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    /// The matching message.
    pub message: Message,
    /// Thread ID where the message was found.
    pub thread_id: String,
    /// Display name of the thread for context.
    pub thread_name: String,
    /// Number of matches in this message.
    pub match_count: usize,
    /// Excerpt around the match for preview.
    pub match_excerpt: String,
}

/// Status of a message being sent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MessageSendStatus {
    /// Message is being sent right now.
    Sending,
    /// Message send failed, waiting for retry.
    Pending,
    /// Message failed after max retries.
    Failed(String),
}

impl MessageSendStatus {
    /// Whether the message is still attempting to send.
    pub fn is_sending(&self) -> bool {
        matches!(self, Self::Sending)
    }

    /// Whether the message is pending retry.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Whether the message has permanently failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

/// A message queued for sending (offline queue).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PendingMessage {
    /// Unique identifier for this pending message.
    pub id: String,
    /// Target thread ID.
    pub thread_id: String,
    /// Message text content.
    pub text: String,
    /// Optional reply-to message ID.
    pub reply_to_id: Option<String>,
    /// When the message was queued (Unix milliseconds).
    pub queued_at: u64,
    /// Number of retry attempts so far.
    pub retry_count: u32,
    /// Current send status.
    pub status: MessageSendStatus,
    /// Last error message if failed.
    pub last_error: Option<String>,
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
            is_dm: false,
            typing_users: vec![],
            is_pinned: false,
            contact_presence: None,
            sync_state: SyncState::default(),
        };
        let t2 = t1.clone();
        assert_eq!(t1, t2);
    }

    #[test]
    fn thread_summary_dm_thread() {
        let dm = ThreadSummary {
            thread_id: "dm:alice-bob-cat-dog".to_string(),
            entity_id: None,
            entity_type: None,
            contact_id: Some("alice-bob-cat-dog".to_string()),
            display_name: "Alice".to_string(),
            last_message_preview: "Hey!".to_string(),
            last_message_timestamp: 1234567890,
            unread_count: 1,
            is_muted: false,
            is_dm: true,
            typing_users: vec![],
            is_pinned: false,
            contact_presence: Some(PresenceStatus::Online),
            sync_state: SyncState::Synced,
        };
        assert!(dm.is_dm);
        assert!(dm.contact_id.is_some());
        assert!(dm.entity_id.is_none());
        assert_eq!(dm.contact_presence, Some(PresenceStatus::Online));
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
            is_pinned: false,
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
            presence: PresenceStatus::Online,
        };
        let cwp = ContactWithPresence {
            contact: contact.clone(),
            presence: PresenceStatus::Online,
            last_seen: None,
            is_in_call: false,
            call_entity_name: None,
            is_screen_sharing: false,
        };
        assert_eq!(cwp.contact.id, "alice");
        assert_eq!(cwp.presence, PresenceStatus::Online);
        assert!(!cwp.is_in_call);
        assert!(cwp.call_entity_name.is_none());
        assert!(!cwp.is_screen_sharing);
    }

    #[test]
    fn contact_with_presence_in_call() {
        let contact = UnifiedContact {
            id: "bob".to_string(),
            display_name: "Bob".to_string(),
            status: "in_call".to_string(),
            presence: PresenceStatus::Busy,
        };
        let cwp = ContactWithPresence {
            contact,
            presence: PresenceStatus::Busy,
            last_seen: None,
            is_in_call: true,
            call_entity_name: Some("Team Standup".to_string()),
            is_screen_sharing: false,
        };
        assert!(cwp.is_in_call);
        assert_eq!(cwp.call_entity_name, Some("Team Standup".to_string()));
        assert!(!cwp.is_screen_sharing);
    }

    #[test]
    fn message_send_status_predicates() {
        assert!(MessageSendStatus::Sending.is_sending());
        assert!(!MessageSendStatus::Sending.is_pending());
        assert!(!MessageSendStatus::Sending.is_failed());

        assert!(!MessageSendStatus::Pending.is_sending());
        assert!(MessageSendStatus::Pending.is_pending());
        assert!(!MessageSendStatus::Pending.is_failed());

        let failed = MessageSendStatus::Failed("Network error".to_string());
        assert!(!failed.is_sending());
        assert!(!failed.is_pending());
        assert!(failed.is_failed());
    }

    #[test]
    fn pending_message_construction() {
        let pending = PendingMessage {
            id: "pending-1".to_string(),
            thread_id: "thread-1".to_string(),
            text: "Hello offline".to_string(),
            reply_to_id: None,
            queued_at: 1234567890,
            retry_count: 0,
            status: MessageSendStatus::Pending,
            last_error: None,
        };
        assert_eq!(pending.id, "pending-1");
        assert_eq!(pending.thread_id, "thread-1");
        assert!(pending.status.is_pending());
        assert_eq!(pending.retry_count, 0);
    }
}
