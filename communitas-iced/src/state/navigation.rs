// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Navigation state for routing between views.

use super::EntityType;

/// Active view routing - mirrors Swift's ActiveView enum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveView {
    /// Home/welcome view.
    Home,
    /// Entity chat view.
    Chat {
        /// The type of entity (org, project, channel, group).
        entity_type: String,
        /// The unique entity identifier.
        entity_id: String,
        /// The display name of the entity.
        entity_name: String,
    },
    /// Contact 1:1 chat.
    ContactChat {
        /// The four-word address of the contact.
        four_words: String,
        /// The optional display name.
        display_name: Option<String>,
    },
    /// Drive/file browser view.
    Drive {
        /// The type of entity.
        entity_type: String,
        /// The unique entity identifier.
        entity_id: String,
    },
    /// Active call view.
    Call {
        /// The four-word address of the peer.
        peer_four_words: String,
    },
    /// Project view (with Kanban).
    Project {
        /// The unique project identifier.
        project_id: String,
    },
    /// Network status panel.
    NetworkPanel,
}

impl Default for ActiveView {
    fn default() -> Self {
        Self::Home
    }
}

/// Detail tab within entity detail pane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DetailTab {
    /// Kanban board (projects only).
    Board,
    /// Chat/messaging.
    #[default]
    Chat,
    /// File storage.
    Drive,
    /// Documents.
    Documents,
    /// Entity details/metadata.
    Details,
}

impl DetailTab {
    /// Get tabs available for an entity type.
    #[must_use]
    pub fn available_for(entity_type: EntityType) -> Vec<Self> {
        match entity_type {
            EntityType::Project => vec![
                Self::Board,
                Self::Chat,
                Self::Drive,
                Self::Documents,
                Self::Details,
            ],
            _ => vec![Self::Chat, Self::Drive, Self::Documents, Self::Details],
        }
    }

    /// Get the display label for this tab.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Board => "Board",
            Self::Chat => "Chat",
            Self::Drive => "Drive",
            Self::Documents => "Documents",
            Self::Details => "Details",
        }
    }
}

/// Navigation state.
#[derive(Debug, Clone, Default)]
pub struct NavigationState {
    /// Current active view.
    pub active_view: ActiveView,
    /// Selected detail tab.
    pub selected_tab: DetailTab,
    /// Navigation history for back button.
    pub history: Vec<ActiveView>,
}

impl NavigationState {
    /// Navigate to a new view.
    pub fn navigate(&mut self, view: ActiveView) {
        if self.active_view != view {
            self.history.push(self.active_view.clone());
            self.active_view = view;
        }
    }

    /// Go back to previous view.
    pub fn go_back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.active_view = prev;
        }
    }

    /// Check if we can go back.
    #[must_use]
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }
}
