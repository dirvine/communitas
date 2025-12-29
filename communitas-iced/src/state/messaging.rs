// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Messaging state for chat and threading.

use std::collections::HashMap;

/// A reaction on a message.
#[derive(Debug, Clone, Default)]
pub struct MessageReaction {
    /// Emoji for this reaction.
    pub emoji: String,
    /// Users who reacted (four-word identities).
    pub users: Vec<String>,
}

/// A chat message.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    /// Unique message ID.
    pub id: String,
    /// Entity ID this message belongs to.
    pub entity_id: String,
    /// Author's four-word identity.
    pub author: String,
    /// Author's display name.
    pub author_display_name: Option<String>,
    /// Message text content.
    pub text: String,
    /// Parent message ID (for thread replies).
    pub reply_to_id: Option<String>,
    /// Timestamp (Unix epoch).
    pub timestamp: i64,
    /// Whether this message has been edited.
    pub is_edited: bool,
    /// Reactions on this message (emoji -> list of users).
    pub reactions: HashMap<String, Vec<String>>,
    /// Whether this message has been deleted (soft delete).
    pub is_deleted: bool,
}

impl ChatMessage {
    /// Check if this is a thread reply.
    #[must_use]
    pub fn is_reply(&self) -> bool {
        self.reply_to_id.is_some()
    }

    /// Get formatted timestamp.
    #[must_use]
    pub fn formatted_time(&self) -> String {
        use chrono::{DateTime, Local, Utc};
        let dt = DateTime::<Utc>::from_timestamp(self.timestamp, 0).unwrap_or_else(Utc::now);
        let local: DateTime<Local> = dt.into();
        local.format("%H:%M").to_string()
    }

    /// Get the short author display.
    #[must_use]
    pub fn short_author(&self) -> String {
        self.author_display_name.clone().unwrap_or_else(|| {
            let words: Vec<&str> = self.author.split('-').collect();
            if words.len() >= 2 {
                format!("{}..{}", words[0], words[words.len() - 1])
            } else {
                self.author.clone()
            }
        })
    }

    /// Check if user has reacted with a specific emoji.
    #[must_use]
    pub fn has_reacted(&self, emoji: &str, user: &str) -> bool {
        self.reactions
            .get(emoji)
            .is_some_and(|users| users.contains(&user.to_string()))
    }

    /// Get total reaction count.
    #[must_use]
    pub fn reaction_count(&self) -> usize {
        self.reactions.values().map(Vec::len).sum()
    }

    /// Get reactions as a sorted list of (emoji, count) pairs.
    #[must_use]
    pub fn sorted_reactions(&self) -> Vec<(String, usize)> {
        let mut reactions: Vec<(String, usize)> = self
            .reactions
            .iter()
            .map(|(emoji, users)| (emoji.clone(), users.len()))
            .collect();
        reactions.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        reactions
    }
}

/// Thread state for Slack-style thread panel.
#[derive(Debug, Clone)]
pub struct ThreadState {
    /// Parent message.
    pub parent_message: ChatMessage,
    /// Entity ID.
    pub entity_id: String,
    /// Replies in this thread.
    pub replies: Vec<ChatMessage>,
    /// Compose text for thread reply.
    pub compose_text: String,
}

impl ThreadState {
    /// Create a new thread state from a parent message.
    #[must_use]
    pub fn new(parent: ChatMessage, entity_id: String) -> Self {
        Self {
            parent_message: parent,
            entity_id,
            replies: Vec::new(),
            compose_text: String::new(),
        }
    }

    /// Get the reply count.
    #[must_use]
    pub fn reply_count(&self) -> usize {
        self.replies.len()
    }
}
