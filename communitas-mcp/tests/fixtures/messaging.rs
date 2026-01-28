// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Messaging test fixtures

#![allow(dead_code)]

use serde_json::{Value, json};

/// Generate a message fixture
pub fn message_fixture(content: &str) -> Value {
    json!({
        "content": content
    })
}

/// Generate a message with recipient
pub fn message_to_fixture(recipient_id: &str, content: &str) -> Value {
    json!({
        "to": recipient_id,
        "content": content
    })
}

/// Thread fixture
pub fn thread_fixture(subject: &str) -> Value {
    json!({
        "subject": subject
    })
}

/// Thread with participants
pub fn thread_with_participants(subject: &str, participants: &[&str]) -> Value {
    json!({
        "subject": subject,
        "participants": participants
    })
}

/// Reply to thread fixture
pub fn thread_reply_fixture(thread_id: &str, content: &str) -> Value {
    json!({
        "thread_id": thread_id,
        "content": content
    })
}

/// Reaction fixture
pub fn reaction_fixture(message_id: &str, emoji: &str) -> Value {
    json!({
        "message_id": message_id,
        "emoji": emoji
    })
}

/// Common emoji reactions
pub fn common_reactions() -> Vec<&'static str> {
    vec![
        "thumbsup", "heart", "smile", "thinking", "clap", "fire", "100",
    ]
}

/// Typing indicator fixture
pub fn typing_indicator_fixture(thread_id: &str) -> Value {
    json!({
        "thread_id": thread_id
    })
}

/// Message search fixture
pub fn message_search_fixture(query: &str) -> Value {
    json!({
        "query": query,
        "limit": 20
    })
}

/// Message list fixture
pub fn message_list_fixture(thread_id: &str, limit: u32) -> Value {
    json!({
        "thread_id": thread_id,
        "limit": limit
    })
}

/// Offline message queue fixture
pub fn offline_message_fixture(to: &str, content: &str) -> Value {
    json!({
        "to": to,
        "content": content,
        "offline": true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_fixture() {
        let msg = message_fixture("Hello, World!");
        assert_eq!(msg["content"], "Hello, World!");
    }

    #[test]
    fn test_thread_with_participants() {
        let thread = thread_with_participants("Team Discussion", &["user1", "user2", "user3"]);
        let participants = thread["participants"].as_array().unwrap();
        assert_eq!(participants.len(), 3);
    }

    #[test]
    fn test_common_reactions() {
        let reactions = common_reactions();
        assert!(!reactions.is_empty());
        assert!(reactions.contains(&"thumbsup"));
    }
}
