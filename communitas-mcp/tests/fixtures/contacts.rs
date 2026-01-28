// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Contact test fixtures

#![allow(dead_code)]

use serde_json::{Value, json};

/// Generate a test contact fixture
pub fn contact_fixture(name: &str) -> Value {
    json!({
        "name": name,
        "email": format!("{}@test.example.com", name.to_lowercase().replace(' ', ".")),
        "phone": "+1-555-0100"
    })
}

/// Generate multiple contact fixtures
pub fn contact_fixtures(count: usize) -> Vec<Value> {
    (1..=count)
        .map(|i| contact_fixture(&format!("Test Contact {}", i)))
        .collect()
}

/// Test contact for Alice
pub fn alice_contact() -> Value {
    json!({
        "name": "Alice Anderson",
        "email": "alice@test.example.com",
        "phone": "+1-555-0101",
        "notes": "Primary test contact"
    })
}

/// Test contact for Bob
pub fn bob_contact() -> Value {
    json!({
        "name": "Bob Baker",
        "email": "bob@test.example.com",
        "phone": "+1-555-0102",
        "notes": "Secondary test contact"
    })
}

/// Test contact for Charlie
pub fn charlie_contact() -> Value {
    json!({
        "name": "Charlie Chen",
        "email": "charlie@test.example.com",
        "phone": "+1-555-0103",
        "notes": "Third test contact"
    })
}

/// Contact update fixture
pub fn contact_update(id: &str) -> Value {
    json!({
        "id": id,
        "name": "Updated Name",
        "email": "updated@test.example.com"
    })
}

/// Contact search fixture
pub fn contact_search(query: &str) -> Value {
    json!({
        "query": query,
        "limit": 10
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_fixture() {
        let contact = contact_fixture("Test User");
        assert_eq!(contact["name"], "Test User");
        assert!(contact["email"].as_str().unwrap().contains("test.user"));
    }

    #[test]
    fn test_contact_fixtures() {
        let contacts = contact_fixtures(5);
        assert_eq!(contacts.len(), 5);
    }
}
