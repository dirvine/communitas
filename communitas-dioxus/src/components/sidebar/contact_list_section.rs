//! Reusable sidebar section for displaying contact lists.
//!
//! This component consolidates the contact list patterns in the sidebar,
//! handling filtering and rendering of contacts with presence indicators.

use communitas_ui_api::UnifiedContact;
use dioxus::prelude::*;

use crate::components::app_shell::{ContactNavItem, QuickActionButton, SidebarSection};

/// Filter contacts by search query.
pub fn filter_contacts(contacts: Vec<UnifiedContact>, search_filter: &str) -> Vec<UnifiedContact> {
    if search_filter.is_empty() {
        contacts
    } else {
        let query = search_filter.to_lowercase();
        contacts
            .into_iter()
            .filter(|c| c.display_name.to_lowercase().contains(&query))
            .collect()
    }
}

/// Reusable sidebar section for displaying contact lists.
///
/// Handles filtering, rendering ContactNavItem for each contact, and selection state.
///
/// # Example
///
/// ```rust,ignore
/// ContactListSection {
///     title: "Direct Messages".to_string(),
///     contacts: contacts,
///     search_filter: search_query(),
///     add_button_label: Some("Add Contact".to_string()),
///     is_selected: move |contact| is_contact_selected(&contact),
///     on_navigate: move |contact| navigator.push(contact_route(&contact)),
///     on_add: move |_| { /* open add contact modal */ },
/// }
/// ```
#[component]
pub fn ContactListSection(
    /// Section title
    title: String,
    /// Contacts to display in this section
    contacts: Vec<UnifiedContact>,
    /// Current search filter string
    #[props(default)]
    search_filter: String,
    /// Label for the add button (if None, no button shown)
    #[props(default)]
    add_button_label: Option<String>,
    /// Callback to check if a contact is selected
    is_selected: Callback<UnifiedContact, bool>,
    /// Callback when a contact is clicked (for navigation)
    on_navigate: EventHandler<UnifiedContact>,
    /// Callback when add button is clicked
    #[props(default)]
    on_add: EventHandler<()>,
) -> Element {
    // Filter contacts by search
    let filtered_contacts = filter_contacts(contacts, &search_filter);

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

            {filtered_contacts.into_iter().map(|contact| {
                let selected = is_selected.call(contact.clone());
                let presence = contact.presence;
                let contact_for_nav = contact.clone();
                rsx! {
                    ContactNavItem {
                        key: "{contact.id}",
                        contact: contact.clone(),
                        selected: selected,
                        unread_count: 0,
                        presence: presence,
                        onclick: move |_| {
                            on_navigate.call(contact_for_nav.clone());
                        },
                    }
                }
            })}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_api::PresenceStatus;

    #[test]
    fn test_filter_contacts_empty_query() {
        let contacts = vec![UnifiedContact {
            id: "1".to_string(),
            display_name: "Alice".to_string(),
            status: "online".to_string(),
            presence: PresenceStatus::Online,
        }];

        let filtered = filter_contacts(contacts.clone(), "");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_contacts_with_query() {
        let contacts = vec![
            UnifiedContact {
                id: "1".to_string(),
                display_name: "Alice".to_string(),
                status: "online".to_string(),
                presence: PresenceStatus::Online,
            },
            UnifiedContact {
                id: "2".to_string(),
                display_name: "Bob".to_string(),
                status: "offline".to_string(),
                presence: PresenceStatus::Offline,
            },
        ];

        let filtered = filter_contacts(contacts, "alice");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].display_name, "Alice");
    }

    #[test]
    fn test_filter_contacts_case_insensitive() {
        let contacts = vec![UnifiedContact {
            id: "1".to_string(),
            display_name: "Alice Johnson".to_string(),
            status: "online".to_string(),
            presence: PresenceStatus::Online,
        }];

        let filtered = filter_contacts(contacts, "ALICE");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_filter_contacts_no_match() {
        let contacts = vec![UnifiedContact {
            id: "1".to_string(),
            display_name: "Alice".to_string(),
            status: "online".to_string(),
            presence: PresenceStatus::Online,
        }];

        let filtered = filter_contacts(contacts, "xyz");
        assert_eq!(filtered.len(), 0);
    }
}
