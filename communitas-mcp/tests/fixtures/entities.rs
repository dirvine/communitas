// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Entity test fixtures

#![allow(dead_code)]

use serde_json::{Value, json};

/// Entity types
pub enum EntityType {
    Group,
    Channel,
    Project,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Group => "group",
            EntityType::Channel => "channel",
            EntityType::Project => "project",
        }
    }
}

/// Generate an entity fixture
pub fn entity_fixture(name: &str, entity_type: EntityType) -> Value {
    json!({
        "name": name,
        "type": entity_type.as_str(),
        "description": format!("Test {} entity", entity_type.as_str())
    })
}

/// Test group entity
pub fn test_group() -> Value {
    json!({
        "name": "Test Group",
        "type": "group",
        "description": "A test group for unit testing"
    })
}

/// Test channel entity
pub fn test_channel() -> Value {
    json!({
        "name": "test-channel",
        "type": "channel",
        "description": "A test channel for unit testing"
    })
}

/// Test project entity
pub fn test_project() -> Value {
    json!({
        "name": "Test Project",
        "type": "project",
        "description": "A test project for unit testing"
    })
}

/// Entity update fixture
pub fn entity_update(id: &str) -> Value {
    json!({
        "id": id,
        "name": "Updated Entity Name",
        "description": "Updated description"
    })
}

/// Member addition fixture
pub fn add_member_fixture(entity_id: &str, user_id: &str) -> Value {
    json!({
        "entity_id": entity_id,
        "user_id": user_id,
        "role": "member"
    })
}

/// Invitation fixture
pub fn invitation_fixture(entity_id: &str) -> Value {
    json!({
        "entity_id": entity_id,
        "message": "You're invited to join this entity",
        "expires_in_days": 7
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_fixture() {
        let entity = entity_fixture("My Group", EntityType::Group);
        assert_eq!(entity["name"], "My Group");
        assert_eq!(entity["type"], "group");
    }

    #[test]
    fn test_predefined_entities() {
        assert_eq!(test_group()["type"], "group");
        assert_eq!(test_channel()["type"], "channel");
        assert_eq!(test_project()["type"], "project");
    }
}
