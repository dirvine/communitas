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

    /// Update contact search query.
    pub fn set_contact_search(&mut self, query: String) {
        self.contact_search = query;
    }

    /// Clear contact search.
    pub fn clear_contact_search(&mut self) {
        self.contact_search.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════════════
    // SIDEBAR SECTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_sidebar_section_enum_variants() {
        // Verify all 4 sidebar sections exist
        let sections = [
            SidebarSection::MyOrganizations,
            SidebarSection::MyCommunities,
            SidebarSection::Personal,
            SidebarSection::DirectMessages,
        ];
        assert_eq!(sections.len(), 4);
    }

    #[test]
    fn test_sidebar_section_equality() {
        assert_eq!(
            SidebarSection::MyOrganizations,
            SidebarSection::MyOrganizations
        );
        assert_ne!(SidebarSection::MyOrganizations, SidebarSection::Personal);
    }

    #[test]
    fn test_sidebar_section_hash() {
        let mut set = HashSet::new();
        set.insert(SidebarSection::MyOrganizations);
        set.insert(SidebarSection::MyOrganizations); // Duplicate
        assert_eq!(set.len(), 1);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SIDEBAR STATE INITIALIZATION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_sidebar_state_new_all_sections_expanded() {
        let state = SidebarState::new();

        // All 4 sections should be expanded by default
        assert!(state.is_section_expanded(SidebarSection::MyOrganizations));
        assert!(state.is_section_expanded(SidebarSection::MyCommunities));
        assert!(state.is_section_expanded(SidebarSection::Personal));
        assert!(state.is_section_expanded(SidebarSection::DirectMessages));
    }

    #[test]
    fn test_sidebar_state_new_no_orgs_expanded() {
        let state = SidebarState::new();
        assert!(state.expanded_orgs.is_empty());
    }

    #[test]
    fn test_sidebar_state_new_no_entity_selected() {
        let state = SidebarState::new();
        assert!(state.selected_entity.is_none());
    }

    #[test]
    fn test_sidebar_state_new_empty_search() {
        let state = SidebarState::new();
        assert!(state.contact_search.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // SECTION TOGGLE TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_toggle_section_collapse() {
        let mut state = SidebarState::new();
        assert!(state.is_section_expanded(SidebarSection::MyOrganizations));

        state.toggle_section(SidebarSection::MyOrganizations);
        assert!(!state.is_section_expanded(SidebarSection::MyOrganizations));
    }

    #[test]
    fn test_toggle_section_expand() {
        let mut state = SidebarState::new();
        state.toggle_section(SidebarSection::Personal); // Collapse
        assert!(!state.is_section_expanded(SidebarSection::Personal));

        state.toggle_section(SidebarSection::Personal); // Expand
        assert!(state.is_section_expanded(SidebarSection::Personal));
    }

    #[test]
    fn test_toggle_section_independent() {
        let mut state = SidebarState::new();
        state.toggle_section(SidebarSection::MyOrganizations);

        // Other sections should remain expanded
        assert!(state.is_section_expanded(SidebarSection::MyCommunities));
        assert!(state.is_section_expanded(SidebarSection::Personal));
        assert!(state.is_section_expanded(SidebarSection::DirectMessages));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ORGANIZATION EXPANSION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_toggle_org_expand() {
        let mut state = SidebarState::new();
        let org_id = "org-123".to_string();

        assert!(!state.is_org_expanded(&org_id));
        state.toggle_org(org_id.clone());
        assert!(state.is_org_expanded(&org_id));
    }

    #[test]
    fn test_toggle_org_collapse() {
        let mut state = SidebarState::new();
        let org_id = "org-123".to_string();

        state.toggle_org(org_id.clone()); // Expand
        state.toggle_org(org_id.clone()); // Collapse
        assert!(!state.is_org_expanded(&org_id));
    }

    #[test]
    fn test_multiple_orgs_expanded() {
        let mut state = SidebarState::new();

        state.toggle_org("org-1".to_string());
        state.toggle_org("org-2".to_string());
        state.toggle_org("org-3".to_string());

        assert!(state.is_org_expanded("org-1"));
        assert!(state.is_org_expanded("org-2"));
        assert!(state.is_org_expanded("org-3"));
        assert_eq!(state.expanded_orgs.len(), 3);
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // ENTITY SELECTION TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_select_entity() {
        let mut state = SidebarState::new();

        state.select_entity(Some("entity-123".to_string()));
        assert!(state.is_selected("entity-123"));
        assert!(!state.is_selected("entity-456"));
    }

    #[test]
    fn test_select_entity_deselect() {
        let mut state = SidebarState::new();

        state.select_entity(Some("entity-123".to_string()));
        state.select_entity(None);
        assert!(!state.is_selected("entity-123"));
    }

    #[test]
    fn test_select_entity_change_selection() {
        let mut state = SidebarState::new();

        state.select_entity(Some("entity-1".to_string()));
        state.select_entity(Some("entity-2".to_string()));

        assert!(!state.is_selected("entity-1"));
        assert!(state.is_selected("entity-2"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // CONTACT SEARCH TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_set_contact_search() {
        let mut state = SidebarState::new();

        state.set_contact_search("alice".to_string());
        assert_eq!(state.contact_search, "alice");
    }

    #[test]
    fn test_clear_contact_search() {
        let mut state = SidebarState::new();

        state.set_contact_search("bob".to_string());
        state.clear_contact_search();
        assert!(state.contact_search.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // NAVIGATION WORKFLOW TESTS
    // ═══════════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_navigation_workflow_org_to_project() {
        let mut state = SidebarState::new();

        // 1. Start with all sections expanded, nothing selected
        assert!(state.selected_entity.is_none());

        // 2. Click on organization to expand it
        state.toggle_org("my-org".to_string());
        assert!(state.is_org_expanded("my-org"));

        // 3. Select the organization
        state.select_entity(Some("my-org".to_string()));
        assert!(state.is_selected("my-org"));

        // 4. Select a child project
        state.select_entity(Some("my-project".to_string()));
        assert!(state.is_selected("my-project"));
        assert!(!state.is_selected("my-org"));

        // 5. Organization should still be expanded
        assert!(state.is_org_expanded("my-org"));
    }

    #[test]
    fn test_navigation_workflow_contact_search() {
        let mut state = SidebarState::new();

        // 1. Ensure Direct Messages section is expanded
        assert!(state.is_section_expanded(SidebarSection::DirectMessages));

        // 2. Search for a contact
        state.set_contact_search("alice".to_string());
        assert_eq!(state.contact_search, "alice");

        // 3. Select a contact (using four-words as ID)
        state.select_entity(Some("ocean-forest-moon-star".to_string()));
        assert!(state.is_selected("ocean-forest-moon-star"));

        // 4. Clear search (contact should still be selected)
        state.clear_contact_search();
        assert!(state.contact_search.is_empty());
        assert!(state.is_selected("ocean-forest-moon-star"));
    }

    #[test]
    fn test_navigation_workflow_section_collapse() {
        let mut state = SidebarState::new();

        // 1. Select an entity in Personal section
        state.select_entity(Some("personal-space".to_string()));

        // 2. Collapse the Personal section
        state.toggle_section(SidebarSection::Personal);
        assert!(!state.is_section_expanded(SidebarSection::Personal));

        // 3. Entity should still be selected (even if section is collapsed)
        assert!(state.is_selected("personal-space"));

        // 4. Expand section again
        state.toggle_section(SidebarSection::Personal);
        assert!(state.is_section_expanded(SidebarSection::Personal));
        assert!(state.is_selected("personal-space"));
    }
}
