//! Type conversions between core messaging types and UI messaging types.
//!
//! This module provides conversion functions for translating between the internal
//! `communitas_core` types used for storage and networking, and the `communitas_ui_api`
//! types used by UI components.
//!
//! Note: Rust's orphan rules prevent implementing `From` traits here since both
//! source and target types are defined in external crates. Use the explicit
//! conversion functions instead.

use communitas_core::command::{
    MessageResponse, ReactionResponse, SearchResult as CoreSearchResult,
};
use communitas_core::legacy_crdt::EntityType;
use communitas_ui_api::{Message, MessageReaction, SearchResult, UnifiedEntityType};

/// Convert a core `MessageResponse` to a UI `Message`.
///
/// Note: The `sender_name` field is set to the `author` ID since the core type
/// does not include display name information. Callers should resolve display names
/// separately if needed.
#[must_use]
pub fn core_message_to_ui(msg: &MessageResponse) -> Message {
    Message {
        id: msg.id.clone(),
        thread_id: msg.entity_id.clone(),
        sender_id: msg.author.clone(),
        sender_name: msg.author.clone(), // Core type lacks display name, use ID
        text: msg.text.clone(),
        timestamp: timestamp_i64_to_u64(msg.timestamp),
        edited: msg.edited_at.is_some(),
        reply_to_id: msg.reply_to_id.clone(),
        reactions: msg.reactions.iter().map(core_reaction_to_ui).collect(),
        // Pin state is maintained by the UI service layer, not the core.
        // Callers that need accurate pin state should call
        // `MessagingService::is_message_pinned` and update this field.
        is_pinned: false,
    }
}

/// Convert a core `ReactionResponse` to a UI `MessageReaction`.
#[must_use]
pub fn core_reaction_to_ui(reaction: &ReactionResponse) -> MessageReaction {
    MessageReaction {
        emoji: reaction.emoji.clone(),
        count: reaction.count,
        reacted_by_me: reaction.user_reacted,
    }
}

/// Convert a core `EntityType` to a UI `UnifiedEntityType`.
#[must_use]
pub fn core_entity_type_to_ui(et: &EntityType) -> UnifiedEntityType {
    match et {
        EntityType::Person => UnifiedEntityType::Person,
        EntityType::Group => UnifiedEntityType::Group,
        EntityType::Project => UnifiedEntityType::Project,
        EntityType::Channel => UnifiedEntityType::Channel,
        EntityType::Organisation => UnifiedEntityType::Organization,
    }
}

/// Convert a core `SearchResult` to a UI `SearchResult`.
#[must_use]
pub fn core_search_result_to_ui(result: &CoreSearchResult) -> SearchResult {
    SearchResult {
        message: core_message_to_ui(&result.message),
        thread_id: result.thread_id.clone(),
        thread_name: result.thread_name.clone(),
        match_count: result.match_count,
        match_excerpt: result.match_excerpt.clone(),
    }
}

/// Convert a UI `UnifiedEntityType` to a core `EntityType`.
#[must_use]
pub fn ui_entity_type_to_core(et: &UnifiedEntityType) -> EntityType {
    match et {
        UnifiedEntityType::Person => EntityType::Person,
        UnifiedEntityType::Group => EntityType::Group,
        UnifiedEntityType::Project => EntityType::Project,
        UnifiedEntityType::Channel => EntityType::Channel,
        UnifiedEntityType::Organization => EntityType::Organisation,
    }
}

/// Convert an i64 timestamp to u64, clamping negative values to 0.
#[must_use]
fn timestamp_i64_to_u64(ts: i64) -> u64 {
    if ts < 0 { 0 } else { ts as u64 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_message_response() -> MessageResponse {
        MessageResponse {
            id: "msg-123".to_string(),
            entity_id: "thread-456".to_string(),
            author: "user-789".to_string(),
            text: "Hello, world!".to_string(),
            timestamp: 1700000000000,
            reply_to_id: Some("msg-001".to_string()),
            reactions: vec![
                ReactionResponse {
                    emoji: "👍".to_string(),
                    count: 3,
                    user_reacted: true,
                    peer_ids: vec!["peer1".to_string(), "peer2".to_string()],
                },
                ReactionResponse {
                    emoji: "❤️".to_string(),
                    count: 1,
                    user_reacted: false,
                    peer_ids: vec!["peer3".to_string()],
                },
            ],
            edited_at: Some(1700000001000),
        }
    }

    #[test]
    fn test_core_message_to_ui() {
        let core_msg = make_test_message_response();
        let ui_msg = core_message_to_ui(&core_msg);

        assert_eq!(ui_msg.id, "msg-123");
        assert_eq!(ui_msg.thread_id, "thread-456");
        assert_eq!(ui_msg.sender_id, "user-789");
        assert_eq!(ui_msg.sender_name, "user-789"); // Falls back to ID
        assert_eq!(ui_msg.text, "Hello, world!");
        assert_eq!(ui_msg.timestamp, 1700000000000);
        assert!(ui_msg.edited);
        assert_eq!(ui_msg.reply_to_id, Some("msg-001".to_string()));
        assert_eq!(ui_msg.reactions.len(), 2);
    }

    #[test]
    fn test_core_message_to_ui_not_edited() {
        let core_msg = MessageResponse {
            id: "msg-1".to_string(),
            entity_id: "thread-1".to_string(),
            author: "user-1".to_string(),
            text: "Test".to_string(),
            timestamp: 1000,
            reply_to_id: None,
            reactions: vec![],
            edited_at: None,
        };
        let ui_msg = core_message_to_ui(&core_msg);

        assert!(!ui_msg.edited);
        assert!(ui_msg.reply_to_id.is_none());
        assert!(ui_msg.reactions.is_empty());
    }

    #[test]
    fn test_core_reaction_to_ui() {
        let core_reaction = ReactionResponse {
            emoji: "🎉".to_string(),
            count: 5,
            user_reacted: true,
            peer_ids: vec!["p1".to_string(), "p2".to_string()],
        };
        let ui_reaction = core_reaction_to_ui(&core_reaction);

        assert_eq!(ui_reaction.emoji, "🎉");
        assert_eq!(ui_reaction.count, 5);
        assert!(ui_reaction.reacted_by_me);
    }

    #[test]
    fn test_core_reaction_to_ui_not_reacted() {
        let core_reaction = ReactionResponse {
            emoji: "😢".to_string(),
            count: 2,
            user_reacted: false,
            peer_ids: vec![],
        };
        let ui_reaction = core_reaction_to_ui(&core_reaction);

        assert_eq!(ui_reaction.emoji, "😢");
        assert_eq!(ui_reaction.count, 2);
        assert!(!ui_reaction.reacted_by_me);
    }

    #[test]
    fn test_core_entity_type_to_ui() {
        assert_eq!(
            core_entity_type_to_ui(&EntityType::Person),
            UnifiedEntityType::Person
        );
        assert_eq!(
            core_entity_type_to_ui(&EntityType::Group),
            UnifiedEntityType::Group
        );
        assert_eq!(
            core_entity_type_to_ui(&EntityType::Project),
            UnifiedEntityType::Project
        );
        assert_eq!(
            core_entity_type_to_ui(&EntityType::Channel),
            UnifiedEntityType::Channel
        );
        // Note: Organisation (British) -> Organization (American)
        assert_eq!(
            core_entity_type_to_ui(&EntityType::Organisation),
            UnifiedEntityType::Organization
        );
    }

    #[test]
    fn test_ui_entity_type_to_core() {
        assert_eq!(
            ui_entity_type_to_core(&UnifiedEntityType::Person),
            EntityType::Person
        );
        assert_eq!(
            ui_entity_type_to_core(&UnifiedEntityType::Group),
            EntityType::Group
        );
        assert_eq!(
            ui_entity_type_to_core(&UnifiedEntityType::Project),
            EntityType::Project
        );
        assert_eq!(
            ui_entity_type_to_core(&UnifiedEntityType::Channel),
            EntityType::Channel
        );
        // Note: Organization (American) -> Organisation (British)
        assert_eq!(
            ui_entity_type_to_core(&UnifiedEntityType::Organization),
            EntityType::Organisation
        );
    }

    #[test]
    fn test_entity_type_roundtrip() {
        let entity_types = [
            EntityType::Person,
            EntityType::Group,
            EntityType::Project,
            EntityType::Channel,
            EntityType::Organisation,
        ];
        for et in entity_types {
            let ui_type = core_entity_type_to_ui(&et);
            let back = ui_entity_type_to_core(&ui_type);
            assert_eq!(et, back);
        }
    }

    #[test]
    fn test_timestamp_negative_clamped() {
        let core_msg = MessageResponse {
            id: "msg-neg".to_string(),
            entity_id: "thread-1".to_string(),
            author: "user-1".to_string(),
            text: "Negative timestamp".to_string(),
            timestamp: -100,
            reply_to_id: None,
            reactions: vec![],
            edited_at: None,
        };
        let ui_msg = core_message_to_ui(&core_msg);

        assert_eq!(ui_msg.timestamp, 0);
    }

    #[test]
    fn test_reactions_preserve_order() {
        let core_msg = make_test_message_response();
        let ui_msg = core_message_to_ui(&core_msg);

        assert_eq!(ui_msg.reactions[0].emoji, "👍");
        assert_eq!(ui_msg.reactions[0].count, 3);
        assert!(ui_msg.reactions[0].reacted_by_me);

        assert_eq!(ui_msg.reactions[1].emoji, "❤️");
        assert_eq!(ui_msg.reactions[1].count, 1);
        assert!(!ui_msg.reactions[1].reacted_by_me);
    }
}
