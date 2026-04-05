// SPDX-License-Identifier: MIT OR Apache-2.0

//! Entity type discriminator.
//!
//! Defines the kinds of entities managed by Communitas: persons, groups,
//! projects, channels, and organisations. Previously embedded in
//! `legacy_crdt.rs`; extracted to its own module for clarity.

use serde::{Deserialize, Serialize};

/// The kind of entity in the Communitas system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// An individual user / agent.
    Person,
    /// A named group (space).
    Group,
    /// A project within a group.
    Project,
    /// A channel within a group.
    Channel,
    /// A top-level organisation.
    Organisation,
}

impl EntityType {
    /// String representation for document IDs and serialization.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Group => "group",
            Self::Project => "project",
            Self::Channel => "channel",
            Self::Organisation => "organisation",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
