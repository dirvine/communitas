// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reusable sidebar section for displaying entity lists.
//!
//! This component consolidates the repetitive entity list patterns in the sidebar,
//! handling both expandable (organizations, communities) and simple (projects, groups) variants.

use std::collections::{HashMap, HashSet};

use communitas_ui_api::UnifiedEntity;
use dioxus::prelude::*;

use crate::components::app_shell::{
    EntityNavItem, ExpandableEntityNavItem, QuickActionButton, SidebarSection,
};
use crate::design_tokens::{radius, semantic, spacing, typography};

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

/// Build a parent→children index from a list of entities.
///
/// This allows O(1) child lookup instead of O(n) filtering per parent,
/// reducing complexity from O(n*m) to O(n) when rendering expandable entities.
pub fn build_children_index(entities: &[UnifiedEntity]) -> HashMap<String, Vec<UnifiedEntity>> {
    let mut index: HashMap<String, Vec<UnifiedEntity>> = HashMap::new();
    for entity in entities {
        if let Some(parent_id) = &entity.parent_id {
            index
                .entry(parent_id.clone())
                .or_default()
                .push(entity.clone());
        }
    }
    index
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
    /// Section title.
    title: String,
    /// Entities to display in this section.
    entities: Vec<UnifiedEntity>,
    /// All entities (needed for finding children of expandable entities).
    #[props(default)]
    all_entities: Vec<UnifiedEntity>,
    /// Current search filter string.
    #[props(default)]
    search_filter: String,
    /// Signal tracking expanded entity IDs (for expandable sections).
    #[props(default)]
    expanded_ids: Signal<HashSet<String>>,
    /// Whether entities in this section are expandable (have children).
    #[props(default = false)]
    expandable: bool,
    /// Whether entities are currently loading.
    #[props(default = false)]
    loading: bool,
    /// Label for the add button (if None, no button shown).
    #[props(default)]
    add_button_label: Option<String>,
    /// Callback to check if an entity is selected.
    is_selected: Callback<UnifiedEntity, bool>,
    /// Callback when an entity is clicked (for navigation).
    on_navigate: EventHandler<UnifiedEntity>,
    /// Callback when add button is clicked.
    #[props(default)]
    on_add: EventHandler<()>,
) -> Element {
    // Filter entities by search
    let filtered_entities = filter_entities(entities.clone(), &search_filter);

    // Build parent→children index once per render (O(n) instead of O(n*m))
    // Only needed for expandable sections
    let children_index = if expandable {
        build_children_index(&all_entities)
    } else {
        HashMap::new()
    };

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

            if loading {
                EntityListSkeleton {}
            } else if filtered_entities.is_empty() {
                EntityListEmpty {}
            } else if expandable {
                {filtered_entities.into_iter().map(|entity| {
                    let selected = is_selected.call(entity.clone());
                    // Use index for O(1) child lookup instead of O(n) filtering
                    let children = children_index
                        .get(&entity.id)
                        .cloned()
                        .unwrap_or_default();
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

/// Skeleton placeholder for entity list during loading.
#[component]
fn EntityListSkeleton() -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; flex-direction: column; gap: {}; padding: {} {};",
                spacing::XS, spacing::XS, spacing::SM
            ),
            role: "status",
            aria_busy: "true",
            aria_label: "Loading entities",
            for i in 0..4 {
                EntitySkeletonItem { key: "{i}" }
            }
        }

        style {
            r#"
            @keyframes entityPulse {{
                0%, 100% {{ opacity: 1; }}
                50% {{ opacity: 0.5; }}
            }}
            "#
        }
    }
}

/// Single entity skeleton item matching sidebar nav item layout.
#[component]
fn EntitySkeletonItem() -> Element {
    rsx! {
        div {
            style: format!(
                "display: flex; align-items: center; gap: {}; padding: {} {};",
                spacing::SM, spacing::SM, spacing::BASE
            ),
            // Icon skeleton
            div {
                style: format!(
                    "width: 24px; height: 24px; border-radius: {}; background: {}; \
                     flex-shrink: 0; animation: entityPulse 1.5s ease-in-out infinite;",
                    radius::MD, semantic::BG_TERTIARY
                ),
            }
            // Text skeleton
            div {
                style: format!(
                    "flex: 1; display: flex; flex-direction: column; gap: {};",
                    spacing::XXS
                ),
                div {
                    style: format!(
                        "width: 70%; height: 12px; border-radius: {}; background: {}; \
                         animation: entityPulse 1.5s ease-in-out infinite;",
                        radius::SM, semantic::BG_TERTIARY
                    ),
                }
            }
        }
    }
}

/// Empty state when no entities exist in this section.
#[component]
fn EntityListEmpty() -> Element {
    rsx! {
        div {
            style: format!(
                "padding: {} {}; text-align: center;",
                spacing::MD, spacing::BASE
            ),
            p {
                style: format!(
                    "font-size: {}; color: {}; margin: 0;",
                    typography::SIZE_XS, semantic::TEXT_MUTED
                ),
                "Nothing here yet"
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
    fn test_build_children_index() {
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
                name: "Another Channel".to_string(),
                entity_type: UnifiedEntityType::Channel,
                description: String::new(),
                member_count: 0,
                parent_id: Some("org1".to_string()),
                category: None,
            },
            UnifiedEntity {
                id: "channel3".to_string(),
                name: "Other Org Channel".to_string(),
                entity_type: UnifiedEntityType::Channel,
                description: String::new(),
                member_count: 0,
                parent_id: Some("org2".to_string()),
                category: None,
            },
        ];

        let index = build_children_index(&entities);

        // org1 should have 2 children
        let org1_children = index.get("org1").expect("org1 should have children");
        assert_eq!(org1_children.len(), 2);
        assert!(org1_children.iter().any(|e| e.id == "channel1"));
        assert!(org1_children.iter().any(|e| e.id == "channel2"));

        // org2 should have 1 child
        let org2_children = index.get("org2").expect("org2 should have children");
        assert_eq!(org2_children.len(), 1);
        assert_eq!(org2_children[0].id, "channel3");

        // Non-existent parent should not be in index
        assert!(!index.contains_key("org3"));
    }

    #[test]
    fn test_build_children_index_empty() {
        let entities = vec![UnifiedEntity {
            id: "org1".to_string(),
            name: "Parent Org".to_string(),
            entity_type: UnifiedEntityType::Organization,
            description: String::new(),
            member_count: 0,
            parent_id: None,
            category: None,
        }];

        let index = build_children_index(&entities);
        assert!(index.is_empty());
    }
}
