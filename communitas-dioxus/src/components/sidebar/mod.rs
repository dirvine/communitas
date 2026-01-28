//! Sidebar components for the main application shell.
//!
//! This module contains reusable components for the sidebar navigation,
//! including entity list sections and contact sections.

pub mod contact_list_section;
pub mod entity_list_section;

pub use contact_list_section::{ContactListSection, filter_contacts};
pub use entity_list_section::{EntityListSection, filter_entities, get_entity_children};
