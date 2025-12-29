// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Sidebar UI state for expansion and selection.

use std::collections::HashSet;

/// Sidebar section identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SidebarSection {
    /// My Organizations - entities the user owns.
    MyOrganizations,
    /// My Communities - entities the user is a member of (but not owner).
    MyCommunities,
    /// Personal - personal groups and spaces.
    Personal,
    /// Direct Messages - contacts for 1:1 messaging.
    DirectMessages,
}

/// Sidebar UI state.
#[derive(Debug, Clone, Default)]
pub struct SidebarState {
    /// Expanded sections.
    pub expanded_sections: HashSet<SidebarSection>,
    /// Expanded organization IDs.
    pub expanded_orgs: HashSet<String>,
    /// Selected entity ID.
    pub selected_entity: Option<String>,
    /// Contact search query.
    pub contact_search: String,
}

impl SidebarState {
    /// Create a new sidebar state with defaults.
    #[must_use]
    pub fn new() -> Self {
        let mut state = Self::default();
        // Expand all sections by default
        state
            .expanded_sections
            .insert(SidebarSection::MyOrganizations);
        state
            .expanded_sections
            .insert(SidebarSection::MyCommunities);
        state.expanded_sections.insert(SidebarSection::Personal);
        state
            .expanded_sections
            .insert(SidebarSection::DirectMessages);
        state
    }

    /// Check if a section is expanded.
    #[must_use]
    pub fn is_section_expanded(&self, section: SidebarSection) -> bool {
        self.expanded_sections.contains(&section)
    }

    /// Toggle a section's expansion.
    pub fn toggle_section(&mut self, section: SidebarSection) {
        if self.expanded_sections.contains(&section) {
            self.expanded_sections.remove(&section);
        } else {
            self.expanded_sections.insert(section);
        }
    }

    /// Check if an organization is expanded.
    #[must_use]
    pub fn is_org_expanded(&self, org_id: &str) -> bool {
        self.expanded_orgs.contains(org_id)
    }

    /// Toggle an organization's expansion.
    pub fn toggle_org(&mut self, org_id: String) {
        if self.expanded_orgs.contains(&org_id) {
            self.expanded_orgs.remove(&org_id);
        } else {
            self.expanded_orgs.insert(org_id);
        }
    }

    /// Select an entity.
    pub fn select_entity(&mut self, entity_id: Option<String>) {
        self.selected_entity = entity_id;
    }

    /// Check if an entity is selected.
    #[must_use]
    pub fn is_selected(&self, entity_id: &str) -> bool {
        self.selected_entity
            .as_ref()
            .is_some_and(|id| id == entity_id)
    }
}
