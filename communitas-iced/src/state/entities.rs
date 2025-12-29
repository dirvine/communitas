// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Entity state for organizations, projects, channels, and groups.

/// Member role in an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MemberRole {
    /// Owner - full control, can delete entity.
    Owner,
    /// Admin - can manage members and settings.
    Admin,
    /// Member - standard access.
    #[default]
    Member,
    /// Guest - limited/read-only access.
    Guest,
}

impl MemberRole {
    /// Get the display name for this role.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Admin => "Admin",
            Self::Member => "Member",
            Self::Guest => "Guest",
        }
    }

    /// Get the icon symbol for this role.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Owner => "👑",   // Crown
            Self::Admin => "🛡️",   // Shield
            Self::Member => "👤",  // Person
            Self::Guest => "👁️",   // Eye
        }
    }

    /// Get the badge color for this role.
    #[must_use]
    pub fn color(&self) -> iced::Color {
        match self {
            Self::Owner => iced::Color::from_rgb(0.9, 0.6, 0.2),   // Orange/gold
            Self::Admin => iced::Color::from_rgb(0.3, 0.5, 0.9),   // Blue
            Self::Member => iced::Color::from_rgb(0.5, 0.5, 0.5),  // Gray
            Self::Guest => iced::Color::from_rgb(0.6, 0.6, 0.7),   // Secondary gray
        }
    }

    /// Get a short label for compact displays.
    #[must_use]
    pub fn short_label(&self) -> &'static str {
        match self {
            Self::Owner => "Own",
            Self::Admin => "Adm",
            Self::Member => "Mem",
            Self::Guest => "Gst",
        }
    }

    /// Check if this role has edit permissions.
    #[must_use]
    pub fn can_edit(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member)
    }

    /// Check if this role has admin permissions.
    #[must_use]
    pub fn can_admin(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin)
    }

    /// Check if this role is the owner.
    #[must_use]
    pub fn is_owner(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Get all role options for dropdowns.
    #[must_use]
    pub fn all() -> [Self; 4] {
        [Self::Owner, Self::Admin, Self::Member, Self::Guest]
    }
}

/// Entity type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// Organization (top-level container).
    Organisation,
    /// Project (with Kanban board).
    Project,
    /// Channel (messaging).
    Channel,
    /// Group (team).
    Group,
}

impl EntityType {
    /// Get the display name for this entity type.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Organisation => "Organisation",
            Self::Project => "Project",
            Self::Channel => "Channel",
            Self::Group => "Group",
        }
    }

    /// Get the icon name for this entity type.
    #[must_use]
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Organisation => "building",
            Self::Project => "folder",
            Self::Channel => "hash",
            Self::Group => "users",
        }
    }
}

/// An entity in the system.
#[derive(Debug, Clone)]
pub struct Entity {
    /// Unique identifier.
    pub id: String,
    /// Four-word identity (if network-linked).
    pub four_words: Option<String>,
    /// Entity type.
    pub entity_type: EntityType,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Parent organization ID (for projects/channels/groups).
    pub parent_org_id: Option<String>,
    /// Current user's role in this entity.
    pub role: MemberRole,
    /// Member count.
    pub member_count: usize,
    /// Whether this is a local-only entity.
    pub is_local_only: bool,
    /// Whether this is a personal entity (only for the current user).
    pub is_personal: bool,
    /// Created timestamp.
    pub created_at: i64,
}

impl Entity {
    /// Create a new entity (user is owner by default).
    #[must_use]
    pub fn new(id: String, entity_type: EntityType, name: String) -> Self {
        Self {
            id,
            four_words: None,
            entity_type,
            name,
            description: None,
            parent_org_id: None,
            role: MemberRole::Owner, // Creator is always owner
            member_count: 1,
            is_local_only: true,
            is_personal: false,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a personal entity (for Personal section).
    #[must_use]
    pub fn new_personal(id: String, entity_type: EntityType, name: String) -> Self {
        let mut entity = Self::new(id, entity_type, name);
        entity.is_personal = true;
        entity
    }

    /// Check if this entity has a Kanban board.
    #[must_use]
    pub fn has_kanban(&self) -> bool {
        self.entity_type == EntityType::Project
    }

    /// Check if this entity belongs in "My Organizations" section.
    #[must_use]
    pub fn is_my_organization(&self) -> bool {
        self.entity_type == EntityType::Organisation && self.role.is_owner()
    }

    /// Check if this entity belongs in "My Communities" section.
    #[must_use]
    pub fn is_my_community(&self) -> bool {
        self.entity_type == EntityType::Organisation && !self.role.is_owner()
    }

    /// Check if user can edit this entity.
    #[must_use]
    pub fn can_edit(&self) -> bool {
        self.role.can_edit()
    }

    /// Check if user can delete this entity.
    #[must_use]
    pub fn can_delete(&self) -> bool {
        self.role.is_owner()
    }

    /// Check if user can manage members (invite, remove, change roles).
    #[must_use]
    pub fn can_manage_members(&self) -> bool {
        self.role.can_admin()
    }

    /// Check if user can create child entities.
    #[must_use]
    pub fn can_create_children(&self) -> bool {
        // Only owners and admins can create children in organizations
        if self.entity_type == EntityType::Organisation {
            self.role.can_admin()
        } else {
            // For other entity types, only owner can create children
            self.role.is_owner()
        }
    }

    /// Check if user has read-only access.
    #[must_use]
    pub fn is_read_only(&self) -> bool {
        !self.role.can_edit()
    }
}
