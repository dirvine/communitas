// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Contact state for managing contacts and presence.

/// Contact online status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContactStatus {
    /// Contact is online.
    Online,
    /// Contact is away.
    Away,
    /// Contact is offline.
    #[default]
    Offline,
}

impl ContactStatus {
    /// Get the color for this status.
    #[must_use]
    pub fn color(&self) -> iced::Color {
        match self {
            Self::Online => iced::Color::from_rgb(0.2, 0.8, 0.3),
            Self::Away => iced::Color::from_rgb(0.9, 0.6, 0.2),
            Self::Offline => iced::Color::from_rgb(0.5, 0.5, 0.5),
        }
    }
}

/// A contact in the system.
#[derive(Debug, Clone)]
pub struct Contact {
    /// Unique identifier.
    pub id: String,
    /// Four-word identity (if network-linked).
    pub four_words: Option<String>,
    /// Display name.
    pub display_name: String,
    /// Current online status.
    pub status: ContactStatus,
    /// Whether this is a local-only contact.
    pub is_local_only: bool,
    /// Whether this contact is favorited.
    pub is_favorite: bool,
    /// Last seen timestamp (if known).
    pub last_seen: Option<i64>,
}

impl Contact {
    /// Create a new local-only contact.
    #[must_use]
    pub fn new_local(id: String, display_name: String) -> Self {
        Self {
            id,
            four_words: None,
            display_name,
            status: ContactStatus::Offline,
            is_local_only: true,
            is_favorite: false,
            last_seen: None,
        }
    }

    /// Create a new network-linked contact.
    #[must_use]
    pub fn new_linked(id: String, four_words: String, display_name: String) -> Self {
        Self {
            id,
            four_words: Some(four_words),
            display_name,
            status: ContactStatus::Offline,
            is_local_only: false,
            is_favorite: false,
            last_seen: None,
        }
    }

    /// Get the short display for the four-word identity.
    #[must_use]
    pub fn short_identity(&self) -> String {
        self.four_words
            .as_ref()
            .map(|fw| {
                let words: Vec<&str> = fw.split('-').collect();
                if words.len() >= 2 {
                    format!("{}..{}", words[0], words[words.len() - 1])
                } else {
                    fw.clone()
                }
            })
            .unwrap_or_else(|| "local".to_string())
    }
}
