// SPDX-License-Identifier: MIT OR Apache-2.0

//! Default permission mappings for standard roles.

use std::collections::HashMap;

use super::access_level::AccessLevel;
use super::resource_type::ResourceType;

/// Standard role names used in the system.
pub mod roles {
    /// Entity owner with full administrative access.
    pub const OWNER: &str = "owner";

    /// Administrator with full access (alias for owner).
    pub const ADMIN: &str = "admin";

    /// Regular member with contribution access.
    pub const MEMBER: &str = "member";

    /// Viewer with read-only access.
    pub const VIEWER: &str = "viewer";

    /// Guest with minimal read-only access.
    pub const GUEST: &str = "guest";
}

/// Get default permissions for a standard role.
///
/// Returns a map from [`ResourceType`] to [`AccessLevel`] representing
/// the default permissions for members with the given role.
///
/// # Supported Roles
///
/// - `"owner"` / `"admin"`: Full Edit access to all resources
/// - `"member"`: Edit access to content, ReadOnly for management
/// - `"viewer"` / `"guest"`: ReadOnly access to content, limited visibility
/// - Unknown roles: Minimal ReadOnly access to messages only
///
/// # Examples
///
/// ```
/// use communitas_core::permissions::{role_defaults, AccessLevel, ResourceType};
///
/// let member_perms = role_defaults("member");
/// assert_eq!(member_perms.get(&ResourceType::Messages), Some(&AccessLevel::Edit));
/// assert_eq!(member_perms.get(&ResourceType::Settings), Some(&AccessLevel::ReadOnly));
///
/// let guest_perms = role_defaults("guest");
/// assert_eq!(guest_perms.get(&ResourceType::Settings), Some(&AccessLevel::NotVisible));
/// ```
pub fn role_defaults(role: &str) -> HashMap<ResourceType, AccessLevel> {
    let mut perms = HashMap::new();

    match role.to_lowercase().as_str() {
        "owner" | "admin" => {
            // Full access to everything
            perms.insert(ResourceType::Messages, AccessLevel::Edit);
            perms.insert(ResourceType::Documents, AccessLevel::Edit);
            perms.insert(ResourceType::KanbanBoards, AccessLevel::Edit);
            perms.insert(ResourceType::Files, AccessLevel::Edit);
            perms.insert(ResourceType::Members, AccessLevel::Edit);
            perms.insert(ResourceType::Settings, AccessLevel::Edit);
        }
        "member" => {
            // Can contribute content but not manage entity
            perms.insert(ResourceType::Messages, AccessLevel::Edit);
            perms.insert(ResourceType::Documents, AccessLevel::Edit);
            perms.insert(ResourceType::KanbanBoards, AccessLevel::Edit);
            perms.insert(ResourceType::Files, AccessLevel::Edit);
            perms.insert(ResourceType::Members, AccessLevel::ReadOnly);
            perms.insert(ResourceType::Settings, AccessLevel::ReadOnly);
        }
        "viewer" | "guest" => {
            // Read-only access to content
            perms.insert(ResourceType::Messages, AccessLevel::ReadOnly);
            perms.insert(ResourceType::Documents, AccessLevel::ReadOnly);
            perms.insert(ResourceType::KanbanBoards, AccessLevel::ReadOnly);
            perms.insert(ResourceType::Files, AccessLevel::ReadOnly);
            perms.insert(ResourceType::Members, AccessLevel::ReadOnly);
            perms.insert(ResourceType::Settings, AccessLevel::NotVisible);
        }
        _ => {
            // Unknown role - minimal access (secure by default)
            perms.insert(ResourceType::Messages, AccessLevel::ReadOnly);
            perms.insert(ResourceType::Documents, AccessLevel::NotVisible);
            perms.insert(ResourceType::KanbanBoards, AccessLevel::NotVisible);
            perms.insert(ResourceType::Files, AccessLevel::NotVisible);
            perms.insert(ResourceType::Members, AccessLevel::NotVisible);
            perms.insert(ResourceType::Settings, AccessLevel::NotVisible);
        }
    }

    perms
}

/// Get the most permissive default role.
///
/// Returns "owner" which has full Edit access to all resources.
pub fn most_permissive_role() -> &'static str {
    roles::OWNER
}

/// Get the least permissive named role.
///
/// Returns "guest" which has minimal read-only access.
pub fn least_permissive_role() -> &'static str {
    roles::GUEST
}

/// Check if a role name is a known standard role.
///
/// Returns `true` for: owner, admin, member, viewer, guest
pub fn is_standard_role(role: &str) -> bool {
    matches!(
        role.to_lowercase().as_str(),
        "owner" | "admin" | "member" | "viewer" | "guest"
    )
}

/// Get all standard role names.
pub fn standard_roles() -> &'static [&'static str] {
    &[
        roles::OWNER,
        roles::ADMIN,
        roles::MEMBER,
        roles::VIEWER,
        roles::GUEST,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner_has_full_access() {
        let perms = role_defaults("owner");

        for resource in ResourceType::all() {
            assert_eq!(
                perms.get(resource),
                Some(&AccessLevel::Edit),
                "Owner should have Edit access to {:?}",
                resource
            );
        }
    }

    #[test]
    fn test_admin_same_as_owner() {
        let owner_perms = role_defaults("owner");
        let admin_perms = role_defaults("admin");

        for resource in ResourceType::all() {
            assert_eq!(
                owner_perms.get(resource),
                admin_perms.get(resource),
                "Admin should have same permissions as owner for {:?}",
                resource
            );
        }
    }

    #[test]
    fn test_member_permissions() {
        let perms = role_defaults("member");

        // Members can edit content
        assert_eq!(perms.get(&ResourceType::Messages), Some(&AccessLevel::Edit));
        assert_eq!(
            perms.get(&ResourceType::Documents),
            Some(&AccessLevel::Edit)
        );
        assert_eq!(
            perms.get(&ResourceType::KanbanBoards),
            Some(&AccessLevel::Edit)
        );
        assert_eq!(perms.get(&ResourceType::Files), Some(&AccessLevel::Edit));

        // Members can only view management areas
        assert_eq!(
            perms.get(&ResourceType::Members),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Settings),
            Some(&AccessLevel::ReadOnly)
        );
    }

    #[test]
    fn test_viewer_permissions() {
        let perms = role_defaults("viewer");

        // Viewers can view content
        assert_eq!(
            perms.get(&ResourceType::Messages),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Documents),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::KanbanBoards),
            Some(&AccessLevel::ReadOnly)
        );

        // Viewers cannot see settings
        assert_eq!(
            perms.get(&ResourceType::Settings),
            Some(&AccessLevel::NotVisible)
        );
    }

    #[test]
    fn test_unknown_role_minimal_access() {
        let perms = role_defaults("unknown_role");

        // Unknown roles get minimal access
        assert_eq!(
            perms.get(&ResourceType::Messages),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Documents),
            Some(&AccessLevel::NotVisible)
        );
        assert_eq!(
            perms.get(&ResourceType::Settings),
            Some(&AccessLevel::NotVisible)
        );
    }

    #[test]
    fn test_case_insensitivity() {
        let lower = role_defaults("member");
        let upper = role_defaults("MEMBER");
        let mixed = role_defaults("Member");

        for resource in ResourceType::all() {
            assert_eq!(lower.get(resource), upper.get(resource));
            assert_eq!(lower.get(resource), mixed.get(resource));
        }
    }

    #[test]
    fn test_is_standard_role() {
        assert!(is_standard_role("owner"));
        assert!(is_standard_role("ADMIN"));
        assert!(is_standard_role("member"));
        assert!(!is_standard_role("custom_role"));
        assert!(!is_standard_role(""));
    }

    #[test]
    fn test_standard_roles() {
        let roles = standard_roles();
        assert!(roles.contains(&"owner"));
        assert!(roles.contains(&"admin"));
        assert!(roles.contains(&"member"));
        assert!(roles.contains(&"viewer"));
        assert!(roles.contains(&"guest"));
    }

    #[test]
    fn test_most_permissive_role() {
        let role = most_permissive_role();
        assert_eq!(role, roles::OWNER);

        // Verify it has full Edit access to all resources
        let perms = role_defaults(role);
        for resource in ResourceType::all() {
            assert_eq!(
                perms.get(resource),
                Some(&AccessLevel::Edit),
                "Most permissive role should have Edit on {:?}",
                resource
            );
        }
    }

    #[test]
    fn test_least_permissive_role() {
        let role = least_permissive_role();
        assert_eq!(role, roles::GUEST);

        // Verify it has restricted access
        let perms = role_defaults(role);
        assert_eq!(
            perms.get(&ResourceType::Messages),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Settings),
            Some(&AccessLevel::NotVisible)
        );
    }

    #[test]
    fn test_guest_permissions() {
        let perms = role_defaults("guest");

        // Guests can view content
        assert_eq!(
            perms.get(&ResourceType::Messages),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Documents),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::KanbanBoards),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Files),
            Some(&AccessLevel::ReadOnly)
        );
        assert_eq!(
            perms.get(&ResourceType::Members),
            Some(&AccessLevel::ReadOnly)
        );

        // Guests cannot see settings
        assert_eq!(
            perms.get(&ResourceType::Settings),
            Some(&AccessLevel::NotVisible)
        );
    }

    #[test]
    fn test_roles_constants() {
        assert_eq!(roles::OWNER, "owner");
        assert_eq!(roles::ADMIN, "admin");
        assert_eq!(roles::MEMBER, "member");
        assert_eq!(roles::VIEWER, "viewer");
        assert_eq!(roles::GUEST, "guest");
    }
}
