//! Shared UI-facing models for Communitas front-ends.

/// Small helper used by the Dioxus prototype to display generated identity words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleWords {
    words: String,
}

impl SampleWords {
    /// Create a new sample words value, accepting anything convertible into `String`.
    pub fn new(words: impl Into<String>) -> Self {
        Self {
            words: words.into(),
        }
    }

    /// Returns the raw words payload for display.
    pub fn as_str(&self) -> &str {
        self.words.as_str()
    }
}

/// Snapshot of identity metadata shown in nav bars and headers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnifiedIdentity {
    pub display_name: String,
    pub four_words: String,
}

/// Logical classification for organization-style entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizationCategory {
    Organization,
    Community,
}

/// Entity type enumeration used by UI layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedEntityType {
    Organization,
    Project,
    Group,
    Channel,
    Person,
}

/// View-model for entities shown across navigation, home, and detail screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedEntity {
    pub id: String,
    pub entity_type: UnifiedEntityType,
    pub name: String,
    pub description: String,
    pub member_count: u64,
    pub parent_id: Option<String>,
    pub category: Option<OrganizationCategory>,
}

impl UnifiedEntity {
    pub fn is_personal_group(&self) -> bool {
        matches!(self.entity_type, UnifiedEntityType::Group) && self.parent_id.is_none()
    }
}

/// Simplified contact view rendered in navigation and contact lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedContact {
    pub id: String,
    pub display_name: String,
    pub status: String,
}
