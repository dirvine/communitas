// SPDX-License-Identifier: MIT OR Apache-2.0

//! Granular permission system for entity resources.
//!
//! This module provides per-member, per-resource access control with three levels:
//! - [`AccessLevel::NotVisible`]: Resource is hidden from the member
//! - [`AccessLevel::ReadOnly`]: Member can view but not modify
//! - [`AccessLevel::Edit`]: Member has full access
//!
//! # Architecture
//!
//! Permissions are stored per-member within each entity using CRDT maps.
//! Each member has a role (e.g., "owner", "member", "viewer") that provides
//! default permissions, which can be overridden on a per-resource basis.
//!
//! # Example
//!
//! ```ignore
//! use communitas_core::permissions::{AccessLevel, ResourceType, role_defaults};
//!
//! // Get default permissions for a "member" role
//! let defaults = role_defaults("member");
//!
//! // Members can edit messages by default
//! assert_eq!(defaults.get(&ResourceType::Messages), Some(&AccessLevel::Edit));
//!
//! // But can only view settings
//! assert_eq!(defaults.get(&ResourceType::Settings), Some(&AccessLevel::ReadOnly));
//! ```

mod access_level;
mod member_permissions;
mod resource_type;
mod role_defaults;

pub use access_level::AccessLevel;
pub use member_permissions::MemberPermissions;
pub use resource_type::ResourceType;
pub use role_defaults::{
    is_standard_role, least_permissive_role, most_permissive_role, role_defaults, roles,
    standard_roles,
};
