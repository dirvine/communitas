// SPDX-License-Identifier: MIT OR Apache-2.0

//! Member permissions structure combining role defaults with overrides.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::access_level::AccessLevel;
use super::resource_type::ResourceType;
use super::role_defaults::role_defaults;

/// Granular permissions for a member within an entity.
///
/// Combines role-based defaults with individual permission overrides.
/// When checking access, overrides take precedence over role defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberPermissions {
    /// Four-word identity of the member.
    pub member_id: String,

    /// Role label (e.g., "owner", "member", "viewer").
    ///
    /// This determines the default permissions when no override exists.
    pub role: String,

    /// Permission overrides per resource type.
    ///
    /// If a resource type is present here, it overrides the role default.
    /// If absent, the role default is used.
    #[serde(default)]
    pub overrides: HashMap<ResourceType, AccessLevel>,

    /// When permissions were last updated.
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,

    /// Four-word identity of who last updated the permissions.
    ///
    /// `None` if permissions were auto-generated from role defaults.
    pub updated_by: Option<String>,
}

impl MemberPermissions {
    /// Create new permissions for a member with a role.
    ///
    /// Initializes with no overrides (all permissions come from role defaults).
    pub fn new(member_id: String, role: String) -> Self {
        Self {
            member_id,
            role,
            overrides: HashMap::new(),
            updated_at: Utc::now(),
            updated_by: None,
        }
    }

    /// Create permissions with the owner role (full access).
    pub fn owner(member_id: String) -> Self {
        Self::new(member_id, "owner".to_string())
    }

    /// Create permissions with the member role (contributor access).
    pub fn member(member_id: String) -> Self {
        Self::new(member_id, "member".to_string())
    }

    /// Create permissions with the viewer role (read-only access).
    pub fn viewer(member_id: String) -> Self {
        Self::new(member_id, "viewer".to_string())
    }

    /// Get the effective access level for a resource type.
    ///
    /// Returns the override if one exists, otherwise the role default.
    pub fn get_access(&self, resource: ResourceType) -> AccessLevel {
        // Check for override first
        if let Some(&level) = self.overrides.get(&resource) {
            return level;
        }

        // Fall back to role defaults
        let defaults = role_defaults(&self.role);
        defaults
            .get(&resource)
            .copied()
            .unwrap_or(AccessLevel::NotVisible)
    }

    /// Check if the member can access a resource at the required level.
    ///
    /// Returns `true` if the effective access level is at least `required`.
    pub fn can_access(&self, resource: ResourceType, required: AccessLevel) -> bool {
        self.get_access(resource).allows(required)
    }

    /// Check if the member can view a resource.
    pub fn can_view(&self, resource: ResourceType) -> bool {
        self.get_access(resource).can_view()
    }

    /// Check if the member can edit a resource.
    pub fn can_edit(&self, resource: ResourceType) -> bool {
        self.get_access(resource).can_edit()
    }

    /// Set a permission override for a resource type.
    ///
    /// The override will take precedence over the role default.
    pub fn set_override(&mut self, resource: ResourceType, level: AccessLevel, updated_by: String) {
        self.overrides.insert(resource, level);
        self.updated_at = Utc::now();
        self.updated_by = Some(updated_by);
    }

    /// Remove a permission override, reverting to role default.
    pub fn remove_override(&mut self, resource: ResourceType, updated_by: String) {
        self.overrides.remove(&resource);
        self.updated_at = Utc::now();
        self.updated_by = Some(updated_by);
    }

    /// Clear all overrides, reverting to pure role defaults.
    pub fn clear_overrides(&mut self, updated_by: String) {
        self.overrides.clear();
        self.updated_at = Utc::now();
        self.updated_by = Some(updated_by);
    }

    /// Change the member's role.
    ///
    /// Note: This does not clear overrides - they continue to take precedence.
    pub fn set_role(&mut self, role: String, updated_by: String) {
        self.role = role;
        self.updated_at = Utc::now();
        self.updated_by = Some(updated_by);
    }

    /// Get all effective permissions as a map.
    ///
    /// Combines role defaults with overrides for all resource types.
    pub fn get_all_permissions(&self) -> HashMap<ResourceType, AccessLevel> {
        let mut perms = role_defaults(&self.role);

        // Apply overrides
        for (resource, &level) in &self.overrides {
            perms.insert(*resource, level);
        }

        perms
    }

    /// Check if there are any permission overrides.
    pub fn has_overrides(&self) -> bool {
        !self.overrides.is_empty()
    }

    /// Get the number of permission overrides.
    pub fn override_count(&self) -> usize {
        self.overrides.len()
    }
}

impl Default for MemberPermissions {
    /// Default permissions: viewer role with no overrides.
    fn default() -> Self {
        Self::new(String::new(), "viewer".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let perms =
            MemberPermissions::new("alice-bob-carol-dave".to_string(), "member".to_string());

        assert_eq!(perms.member_id, "alice-bob-carol-dave");
        assert_eq!(perms.role, "member");
        assert!(perms.overrides.is_empty());
        assert!(perms.updated_by.is_none());
    }

    #[test]
    fn test_owner_has_full_access() {
        let perms = MemberPermissions::owner("owner-identity-four-words".to_string());

        for resource in ResourceType::all() {
            assert!(
                perms.can_edit(*resource),
                "Owner should be able to edit {:?}",
                resource
            );
        }
    }

    #[test]
    fn test_viewer_read_only() {
        let perms = MemberPermissions::viewer("viewer-identity-four-words".to_string());

        // Can view content
        assert!(perms.can_view(ResourceType::Messages));
        assert!(perms.can_view(ResourceType::Documents));

        // Cannot edit
        assert!(!perms.can_edit(ResourceType::Messages));
        assert!(!perms.can_edit(ResourceType::Documents));

        // Cannot see settings
        assert!(!perms.can_view(ResourceType::Settings));
    }

    #[test]
    fn test_override_takes_precedence() {
        let mut perms = MemberPermissions::viewer("viewer-four-words-here".to_string());

        // Viewers cannot edit messages by default
        assert!(!perms.can_edit(ResourceType::Messages));

        // Add override to allow editing messages
        perms.set_override(
            ResourceType::Messages,
            AccessLevel::Edit,
            "admin-four-words-here".to_string(),
        );

        // Now can edit messages
        assert!(perms.can_edit(ResourceType::Messages));

        // But still cannot edit documents (no override)
        assert!(!perms.can_edit(ResourceType::Documents));
    }

    #[test]
    fn test_remove_override() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());

        // Members can edit by default
        assert!(perms.can_edit(ResourceType::Messages));

        // Add restrictive override
        perms.set_override(
            ResourceType::Messages,
            AccessLevel::ReadOnly,
            "admin-four-words-here".to_string(),
        );

        // Now cannot edit
        assert!(!perms.can_edit(ResourceType::Messages));

        // Remove override
        perms.remove_override(ResourceType::Messages, "admin-four-words-here".to_string());

        // Back to default - can edit again
        assert!(perms.can_edit(ResourceType::Messages));
    }

    #[test]
    fn test_get_all_permissions() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());

        perms.set_override(
            ResourceType::Settings,
            AccessLevel::Edit,
            "admin-four-words-here".to_string(),
        );

        let all = perms.get_all_permissions();

        // Role default
        assert_eq!(all.get(&ResourceType::Messages), Some(&AccessLevel::Edit));

        // Override
        assert_eq!(all.get(&ResourceType::Settings), Some(&AccessLevel::Edit));
    }

    #[test]
    fn test_serialization() {
        let mut perms =
            MemberPermissions::new("alice-bob-carol-dave".to_string(), "member".to_string());
        perms.set_override(
            ResourceType::Files,
            AccessLevel::NotVisible,
            "admin-four-words-here".to_string(),
        );

        let json = serde_json::to_string(&perms).unwrap();
        let parsed: MemberPermissions = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.member_id, perms.member_id);
        assert_eq!(parsed.role, perms.role);
        assert_eq!(
            parsed.overrides.get(&ResourceType::Files),
            Some(&AccessLevel::NotVisible)
        );
    }

    #[test]
    fn test_can_access() {
        let perms = MemberPermissions::member("member-four-words-here".to_string());

        // Edit allows Edit
        assert!(perms.can_access(ResourceType::Messages, AccessLevel::Edit));

        // Edit also allows ReadOnly
        assert!(perms.can_access(ResourceType::Messages, AccessLevel::ReadOnly));

        // ReadOnly does not allow Edit
        assert!(perms.can_access(ResourceType::Settings, AccessLevel::ReadOnly));
        assert!(!perms.can_access(ResourceType::Settings, AccessLevel::Edit));
    }

    #[test]
    fn test_member_factory() {
        let perms = MemberPermissions::member("test-four-words-here".to_string());

        assert_eq!(perms.member_id, "test-four-words-here");
        assert_eq!(perms.role, "member");
        assert!(perms.overrides.is_empty());

        // Members can edit content but only view management
        assert!(perms.can_edit(ResourceType::Messages));
        assert!(perms.can_edit(ResourceType::Documents));
        assert!(!perms.can_edit(ResourceType::Members));
        assert!(!perms.can_edit(ResourceType::Settings));
    }

    #[test]
    fn test_clear_overrides() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());

        // Add multiple overrides
        perms.set_override(
            ResourceType::Messages,
            AccessLevel::ReadOnly,
            "admin-four-words-here".to_string(),
        );
        perms.set_override(
            ResourceType::Files,
            AccessLevel::NotVisible,
            "admin-four-words-here".to_string(),
        );

        assert_eq!(perms.override_count(), 2);
        assert!(perms.has_overrides());

        // Clear all overrides
        perms.clear_overrides("admin-four-words-here".to_string());

        assert!(!perms.has_overrides());
        assert_eq!(perms.override_count(), 0);
        assert!(perms.overrides.is_empty());

        // Should be back to member defaults
        assert!(perms.can_edit(ResourceType::Messages));
        assert!(perms.can_view(ResourceType::Files));
    }

    #[test]
    fn test_set_role() {
        let mut perms = MemberPermissions::viewer("user-four-words-here".to_string());

        // Viewers cannot edit
        assert!(!perms.can_edit(ResourceType::Messages));

        // Upgrade to member role
        perms.set_role("member".to_string(), "admin-four-words-here".to_string());

        assert_eq!(perms.role, "member");
        assert_eq!(perms.updated_by, Some("admin-four-words-here".to_string()));

        // Now can edit
        assert!(perms.can_edit(ResourceType::Messages));
    }

    #[test]
    fn test_set_role_preserves_overrides() {
        let mut perms = MemberPermissions::viewer("user-four-words-here".to_string());

        // Add an override that gives edit access to settings
        perms.set_override(
            ResourceType::Settings,
            AccessLevel::Edit,
            "admin-four-words-here".to_string(),
        );

        // Even as viewer, can edit settings due to override
        assert!(perms.can_edit(ResourceType::Settings));

        // Downgrade role to guest
        perms.set_role("guest".to_string(), "admin-four-words-here".to_string());

        // Override still takes precedence
        assert!(perms.can_edit(ResourceType::Settings));
        assert!(perms.has_overrides());
    }

    #[test]
    fn test_has_overrides() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());

        assert!(!perms.has_overrides());

        perms.set_override(
            ResourceType::Messages,
            AccessLevel::ReadOnly,
            "admin-four-words-here".to_string(),
        );

        assert!(perms.has_overrides());
    }

    #[test]
    fn test_override_count() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());

        assert_eq!(perms.override_count(), 0);

        perms.set_override(
            ResourceType::Messages,
            AccessLevel::ReadOnly,
            "admin-four-words-here".to_string(),
        );
        assert_eq!(perms.override_count(), 1);

        perms.set_override(
            ResourceType::Files,
            AccessLevel::NotVisible,
            "admin-four-words-here".to_string(),
        );
        assert_eq!(perms.override_count(), 2);

        perms.remove_override(ResourceType::Messages, "admin-four-words-here".to_string());
        assert_eq!(perms.override_count(), 1);
    }

    #[test]
    fn test_default_permissions() {
        let perms = MemberPermissions::default();

        // Default is viewer role
        assert_eq!(perms.role, "viewer");
        assert!(perms.member_id.is_empty());
        assert!(!perms.has_overrides());

        // Viewers can view but not edit
        assert!(perms.can_view(ResourceType::Messages));
        assert!(!perms.can_edit(ResourceType::Messages));
    }

    #[test]
    fn test_updated_at_changes() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());
        let initial_time = perms.updated_at;

        // Sleep briefly to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        perms.set_override(
            ResourceType::Messages,
            AccessLevel::ReadOnly,
            "admin-four-words-here".to_string(),
        );

        assert!(perms.updated_at > initial_time);
    }

    #[test]
    fn test_updated_by_tracking() {
        let mut perms = MemberPermissions::member("member-four-words-here".to_string());

        // Initial created without updated_by
        assert!(perms.updated_by.is_none());

        // After set_override, updated_by is set
        perms.set_override(
            ResourceType::Messages,
            AccessLevel::ReadOnly,
            "admin-one-two-three".to_string(),
        );
        assert_eq!(perms.updated_by, Some("admin-one-two-three".to_string()));

        // After remove_override, updated_by is updated
        perms.remove_override(ResourceType::Messages, "admin-four-five-six".to_string());
        assert_eq!(perms.updated_by, Some("admin-four-five-six".to_string()));
    }
}
