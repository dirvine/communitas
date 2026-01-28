//! Reusable sidebar section for displaying entity lists.
//!
//! This component consolidates the repetitive entity list patterns in the sidebar,
//! handling both expandable (organizations, communities) and simple (projects, groups) variants.

use std::collections::HashSet;

use communitas_ui_api::UnifiedEntity;
use dioxus::prelude::*;

use crate::components::app_shell::{
    EntityNavItem, ExpandableEntityNavItem, QuickActionButton, SidebarSection,
};

/// Get child entities for a parent entity.
pub fn get_entity_children(entities: &[UnifiedEntity], parent_id: &str) -> Vec<UnifiedEntity> {
    entities
        .iter()
        .filter(|e| e.parent_id.as_deref() == Some(parent_id))
        .cloned()
        .collect()
}

/// Filter entities by search query.
pub fn filter_entities(entities: Vec<UnifiedEntity>, search_filter: &str) -> Vec<UnifiedEntity> {
    if search_filter.is_empty() {
        entities
    } else {
        let query = search_filter.to_lowercase();
        entities
            .into_iter()
            .filter(|e| e.name.to_lowercase().contains(&query))
            .collect()
    }
}

/// Reusable sidebar section for displaying entity lists.
///
/// Handles both expandable entities (organizations, communities) and
/// simple entities (projects, groups) based on the `expandable` prop.
///
/// # Example
///
/// ```rust,ignore
/// EntityListSection {
///     title: "My Organizations".to_string(),
///     entities: organizations,
///     all_entities: dir_snapshot.entities.clone(),
///     search_filter: search_query(),
///     expanded_ids: expanded_entities,
///     expandable: true,
///     add_button_label: Some("Create Organization".to_string()),
///     is_selected: move |entity| is_entity_selected(&entity),
///     on_navigate: move |entity| navigator.push(entity_route(&entity)),
///     on_add: move |_| {
///         show_create_modal.set(Some(CreateEntityType::Organization));
///     },
/// }
/// ```
#[component]
pub fn EntityListSection(
    /// Section title
    title: String,
    /// Entities to display in this section
    entities: Vec<UnifiedEntity>,
    /// All entities (needed for finding children of expandable entities)
    #[props(default)]
    all_entities: Vec<UnifiedEntity>,
    /// Current search filter string
    #[props(default)]
    search_filter: String,
    /// Signal tracking expanded entity IDs (for expandable sections)
    #[props(default)]
    expanded_ids: Signal<HashSet<String>>,
    /// Whether entities in this section are expandable (have children)
    #[props(default = false)]
    expandable: bool,
    /// Label for the add button (if None, no button shown)
    #[props(default)]
    add_button_label: Option<String>,
    /// Callback to check if an entity is selected
    is_selected: Callback<UnifiedEntity, bool>,
    /// Callback when an entity is clicked (for navigation)
    on_navigate: EventHandler<UnifiedEntity>,
    /// Callback when add button is clicked
    #[props(default)]
    on_add: EventHandler<()>,
) -> Element {
    // Filter entities by search
    let filtered_entities = filter_entities(entities.clone(), &search_filter);

    // Build action button if configured
    let action = add_button_label.map(|label| {
        rsx! {
            QuickActionButton {
                icon: "+".to_string(),
                label: label,
                onclick: move |_| {
                    on_add.call(());
                },
            }
        }
    });

    rsx! {
        SidebarSection {
            title: title,
            action: action,

            if expandable {
                {filtered_entities.into_iter().map(|entity| {
                    let selected = is_selected.call(entity.clone());
                    let children = get_entity_children(&all_entities, &entity.id);
                    let has_children = !children.is_empty();
                    let is_expanded = expanded_ids().contains(&entity.id);
                    let entity_id = entity.id.clone();
                    let entity_id_toggle = entity.id.clone();
                    let entity_for_nav = entity.clone();

                    rsx! {
                        ExpandableEntityNavItem {
                            key: "{entity_id}",
                            entity: entity.clone(),
                            selected: selected,
                            unread_count: 0,
                            expanded: is_expanded,
                            has_children: has_children,
                            onclick: move |_| {
                                on_navigate.call(entity_for_nav.clone());
                            },
                            ontoggle: move |_expanded| {
                                let id = entity_id_toggle.clone();
                                if expanded_ids().contains(&id) {
                                    expanded_ids.write().remove(&id);
                                } else {
                                    expanded_ids.write().insert(id);
                                }
                            },

                            // Child entities
                            {children.into_iter().map(|child| {
                                let child_selected = is_selected.call(child.clone());
                                let child_for_nav = child.clone();
                                rsx! {
                                    EntityNavItem {
                                        key: "{child.id}",
                                        entity: child.clone(),
                                        selected: child_selected,
                                        unread_count: 0,
                                        onclick: move |_| {
                                            on_navigate.call(child_for_nav.clone());
                                        },
                                    }
                                }
                            })}
                        }
                    }
                })}
            } else {
                {filtered_entities.into_iter().map(|entity| {
                    let selected = is_selected.call(entity.clone());
                    let entity_for_nav = entity.clone();
                    rsx! {
                        EntityNavItem {
                            key: "{entity.id}",
                            entity: entity.clone(),
                            selected: selected,
                            unread_count: 0,
                            onclick: move |_| {
                                on_navigate.call(entity_for_nav.clone());
                            },
                        }
                    }
                })}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_api::UnifiedEntityType;

    #[test]
    fn test_filter_entities_empty_query() {
        let entities = vec![UnifiedEntity {
            id: "1".to_string(),
            name: "Test".to_string(),
            entity_type: UnifiedEntityType::Organization,
            description: String::new(),
            member_count: 0,
            parent_id: None,
            category: None,
        }];

        let filtered = filter_entities(entities.clone(), "");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_entities_with_query() {
        let entities = vec![
            UnifiedEntity {
                id: "1".to_string(),
                name: "Alpha Org".to_string(),
                entity_type: UnifiedEntityType::Organization,
                description: String::new(),
                member_count: 0,
                parent_id: None,
                category: None,
            },
            UnifiedEntity {
                id: "2".to_string(),
                name: "Beta Org".to_string(),
                entity_type: UnifiedEntityType::Organization,
                description: String::new(),
                member_count: 0,
                parent_id: None,
                category: None,
            },
        ];

        let filtered = filter_entities(entities, "alpha");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Alpha Org");
    }

    #[test]
    fn test_get_entity_children() {
        let entities = vec![
            UnifiedEntity {
                id: "org1".to_string(),
                name: "Parent Org".to_string(),
                entity_type: UnifiedEntityType::Organization,
                description: String::new(),
                member_count: 0,
                parent_id: None,
                category: None,
            },
            UnifiedEntity {
                id: "channel1".to_string(),
                name: "Child Channel".to_string(),
                entity_type: UnifiedEntityType::Channel,
                description: String::new(),
                member_count: 0,
                parent_id: Some("org1".to_string()),
                category: None,
            },
            UnifiedEntity {
                id: "channel2".to_string(),
                name: "Other Channel".to_string(),
                entity_type: UnifiedEntityType::Channel,
                description: String::new(),
                member_count: 0,
                parent_id: Some("org2".to_string()),
                category: None,
            },
        ];

        let children = get_entity_children(&entities, "org1");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "channel1");
    }
}
