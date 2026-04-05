// SPDX-License-Identifier: MIT OR Apache-2.0

//! Access level definitions for granular permissions.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::str::FromStr;

/// Granular access level for a resource type.
///
/// Access levels are ordered from most restrictive to least restrictive:
/// `NotVisible` < `ReadOnly` < `Edit`
///
/// This ordering allows permission checks to verify if a user has
/// "at least" a certain level of access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessLevel {
    /// Resource is hidden - member cannot see it exists.
    ///
    /// Use this for sensitive resources that should be completely
    /// invisible to certain members (e.g., admin settings for guests).
    NotVisible,

    /// Read-only access - can view but not modify.
    ///
    /// Members with this level can:
    /// - View resource content
    /// - List resources
    /// - Read metadata
    ///
    /// Members cannot:
    /// - Create new resources
    /// - Modify existing resources
    /// - Delete resources
    ReadOnly,

    /// Full access - can view, create, modify, and delete.
    ///
    /// Members with this level have complete control over the resource type,
    /// subject to other constraints (e.g., cannot delete resources created
    /// by others without additional permissions).
    Edit,
}

impl Default for AccessLevel {
    /// Default access level is `NotVisible` (secure by default).
    fn default() -> Self {
        AccessLevel::NotVisible
    }
}

impl AccessLevel {
    /// Check if this access level allows the required level.
    ///
    /// Returns `true` if `self` is at least as permissive as `required`.
    ///
    /// # Examples
    ///
    /// ```
    /// use communitas_bindings::permissions::AccessLevel;
    ///
    /// assert!(AccessLevel::Edit.allows(AccessLevel::ReadOnly));
    /// assert!(AccessLevel::Edit.allows(AccessLevel::Edit));
    /// assert!(!AccessLevel::ReadOnly.allows(AccessLevel::Edit));
    /// assert!(!AccessLevel::NotVisible.allows(AccessLevel::ReadOnly));
    /// ```
    pub fn allows(self, required: AccessLevel) -> bool {
        self >= required
    }

    /// Check if this access level can view the resource.
    ///
    /// Returns `true` for `ReadOnly` and `Edit` levels.
    pub fn can_view(self) -> bool {
        matches!(self, AccessLevel::ReadOnly | AccessLevel::Edit)
    }

    /// Check if this access level can modify the resource.
    ///
    /// Returns `true` only for `Edit` level.
    pub fn can_edit(self) -> bool {
        matches!(self, AccessLevel::Edit)
    }

    /// Get the numeric rank for ordering (0 = NotVisible, 1 = ReadOnly, 2 = Edit).
    fn rank(self) -> u8 {
        match self {
            AccessLevel::NotVisible => 0,
            AccessLevel::ReadOnly => 1,
            AccessLevel::Edit => 2,
        }
    }

    /// Convert to string representation.
    pub fn as_str(self) -> &'static str {
        match self {
            AccessLevel::NotVisible => "not_visible",
            AccessLevel::ReadOnly => "read_only",
            AccessLevel::Edit => "edit",
        }
    }
}

/// Error when parsing an invalid access level string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseAccessLevelError {
    /// The invalid input string.
    pub invalid_value: String,
}

impl std::fmt::Display for ParseAccessLevelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid access level '{}': expected 'not_visible', 'read_only', or 'edit'",
            self.invalid_value
        )
    }
}

impl std::error::Error for ParseAccessLevelError {}

impl FromStr for AccessLevel {
    type Err = ParseAccessLevelError;

    /// Parse from string representation.
    ///
    /// Accepts: "not_visible", "read_only", "edit" (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "not_visible" | "notvisible" | "hidden" => Ok(AccessLevel::NotVisible),
            "read_only" | "readonly" | "read" | "view" => Ok(AccessLevel::ReadOnly),
            "edit" | "write" | "full" => Ok(AccessLevel::Edit),
            _ => Err(ParseAccessLevelError {
                invalid_value: s.to_string(),
            }),
        }
    }
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccessLevel {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl std::fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_level_ordering() {
        assert!(AccessLevel::NotVisible < AccessLevel::ReadOnly);
        assert!(AccessLevel::ReadOnly < AccessLevel::Edit);
        assert!(AccessLevel::NotVisible < AccessLevel::Edit);
    }

    #[test]
    fn test_allows() {
        // Edit allows everything
        assert!(AccessLevel::Edit.allows(AccessLevel::Edit));
        assert!(AccessLevel::Edit.allows(AccessLevel::ReadOnly));
        assert!(AccessLevel::Edit.allows(AccessLevel::NotVisible));

        // ReadOnly allows itself and below
        assert!(AccessLevel::ReadOnly.allows(AccessLevel::ReadOnly));
        assert!(AccessLevel::ReadOnly.allows(AccessLevel::NotVisible));
        assert!(!AccessLevel::ReadOnly.allows(AccessLevel::Edit));

        // NotVisible only allows itself
        assert!(AccessLevel::NotVisible.allows(AccessLevel::NotVisible));
        assert!(!AccessLevel::NotVisible.allows(AccessLevel::ReadOnly));
        assert!(!AccessLevel::NotVisible.allows(AccessLevel::Edit));
    }

    #[test]
    fn test_can_view_and_edit() {
        assert!(!AccessLevel::NotVisible.can_view());
        assert!(!AccessLevel::NotVisible.can_edit());

        assert!(AccessLevel::ReadOnly.can_view());
        assert!(!AccessLevel::ReadOnly.can_edit());

        assert!(AccessLevel::Edit.can_view());
        assert!(AccessLevel::Edit.can_edit());
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "not_visible".parse::<AccessLevel>().unwrap(),
            AccessLevel::NotVisible
        );
        assert_eq!(
            "read_only".parse::<AccessLevel>().unwrap(),
            AccessLevel::ReadOnly
        );
        assert_eq!("edit".parse::<AccessLevel>().unwrap(), AccessLevel::Edit);
        assert_eq!("EDIT".parse::<AccessLevel>().unwrap(), AccessLevel::Edit);
        assert!("invalid".parse::<AccessLevel>().is_err());
    }

    #[test]
    fn test_default() {
        assert_eq!(AccessLevel::default(), AccessLevel::NotVisible);
    }

    #[test]
    fn test_serialization() {
        let level = AccessLevel::ReadOnly;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"read_only\"");

        let parsed: AccessLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, level);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", AccessLevel::NotVisible), "not_visible");
        assert_eq!(format!("{}", AccessLevel::ReadOnly), "read_only");
        assert_eq!(format!("{}", AccessLevel::Edit), "edit");
    }

    #[test]
    fn test_as_str() {
        assert_eq!(AccessLevel::NotVisible.as_str(), "not_visible");
        assert_eq!(AccessLevel::ReadOnly.as_str(), "read_only");
        assert_eq!(AccessLevel::Edit.as_str(), "edit");
    }

    #[test]
    fn test_parse_error_display() {
        let err: Result<AccessLevel, _> = "invalid_level".parse();
        assert!(err.is_err());
        let err = err.unwrap_err();
        assert!(err.to_string().contains("invalid_level"));
        assert!(err.to_string().contains("not_visible"));
        assert!(err.to_string().contains("read_only"));
        assert!(err.to_string().contains("edit"));
    }

    #[test]
    fn test_from_str_aliases() {
        // Test all alias forms
        assert_eq!(
            "hidden".parse::<AccessLevel>().unwrap(),
            AccessLevel::NotVisible
        );
        assert_eq!(
            "notvisible".parse::<AccessLevel>().unwrap(),
            AccessLevel::NotVisible
        );

        assert_eq!(
            "readonly".parse::<AccessLevel>().unwrap(),
            AccessLevel::ReadOnly
        );
        assert_eq!(
            "read".parse::<AccessLevel>().unwrap(),
            AccessLevel::ReadOnly
        );
        assert_eq!(
            "view".parse::<AccessLevel>().unwrap(),
            AccessLevel::ReadOnly
        );

        assert_eq!("write".parse::<AccessLevel>().unwrap(), AccessLevel::Edit);
        assert_eq!("full".parse::<AccessLevel>().unwrap(), AccessLevel::Edit);
    }

    #[test]
    fn test_rank() {
        // Test internal ordering via PartialOrd
        assert!(AccessLevel::NotVisible < AccessLevel::ReadOnly);
        assert!(AccessLevel::ReadOnly < AccessLevel::Edit);
        assert!(AccessLevel::NotVisible < AccessLevel::Edit);

        // Test equality
        assert_eq!(
            AccessLevel::Edit.cmp(&AccessLevel::Edit),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            AccessLevel::ReadOnly.cmp(&AccessLevel::NotVisible),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_partial_ord() {
        assert!(
            AccessLevel::NotVisible.partial_cmp(&AccessLevel::ReadOnly)
                == Some(std::cmp::Ordering::Less)
        );
        assert!(
            AccessLevel::Edit.partial_cmp(&AccessLevel::ReadOnly)
                == Some(std::cmp::Ordering::Greater)
        );
        assert!(
            AccessLevel::ReadOnly.partial_cmp(&AccessLevel::ReadOnly)
                == Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_allows_not_visible() {
        let level = AccessLevel::NotVisible;
        assert!(level.allows(AccessLevel::NotVisible));
        assert!(!level.allows(AccessLevel::ReadOnly));
        assert!(!level.allows(AccessLevel::Edit));
    }
}
