// SPDX-License-Identifier: MIT OR Apache-2.0

//! Verifies Rust↔Swift client method parity.
//!
//! Every Rust client method should have a corresponding Swift method.
//! This test reads the Swift source and checks for each Rust method.
//!
//! Run: cargo nextest run -p communitas-x0x-client --test swift_parity

/// Maps Rust method names to their expected Swift equivalents.
///
/// Rust uses snake_case, Swift uses camelCase. Some names differ
/// beyond case convention — those are mapped explicitly.
const RUST_TO_SWIFT: &[(&str, &str)] = &[
    // System & Identity
    ("health", "health"),
    ("status", "status"),
    ("shutdown", "shutdown"),
    ("agent", "agent"),
    ("agent_user_id", "agentUserId"),
    ("agent_card", "agentCard"),
    ("import_agent_card", "importAgentCard"),
    ("ws_sessions", "wsSessions"),
    // Announcements
    ("announce", "announce"),
    ("announce_with_options", "announceWithOptions"),
    // Discovery & Network
    ("peers", "peers"),
    ("discovered_agents", "discoveredAgents"),
    ("discovered_agent", "discoveredAgent"),
    ("presence", "presence"),
    ("network_status", "networkStatus"),
    ("bootstrap_cache", "bootstrapCache"),
    // Gossip
    ("publish", "publish"),
    ("subscribe", "subscribe"),
    ("unsubscribe", "unsubscribe"),
    // Direct Messaging
    ("connect_agent", "connectAgent"),
    ("send_direct", "sendDirect"),
    ("direct_connections", "directConnections"),
    // Contacts
    ("list_contacts", "listContacts"),
    ("add_contact", "addContact"),
    ("set_trust", "setTrust"),
    ("update_contact", "updateContact"),
    ("remove_contact", "removeContact"),
    ("revoke_contact", "revokeContact"),
    ("revocations", "revocations"),
    // Machines
    ("list_machines", "listMachines"),
    ("add_machine", "addMachine"),
    ("remove_machine", "removeMachine"),
    ("pin_machine", "pinMachine"),
    ("unpin_machine", "unpinMachine"),
    // Trust
    ("evaluate_trust", "evaluateTrust"),
    // MLS Groups
    ("create_mls_group", "createMlsGroup"),
    ("list_mls_groups", "listMlsGroups"),
    ("get_mls_group", "getMlsGroup"),
    ("add_mls_member", "addMlsMember"),
    ("remove_mls_member", "removeMlsMember"),
    ("encrypt", "encrypt"),
    ("decrypt", "decrypt"),
    ("create_mls_welcome", "createMlsWelcome"),
    // Named Groups
    ("create_group", "createGroup"),
    ("list_groups", "listGroups"),
    ("get_group", "groupInfo"),
    ("invite", "invite"),
    ("join_group", "joinGroup"),
    ("set_group_display_name", "setGroupDisplayName"),
    ("leave_group", "leaveGroup"),
    // Task Lists
    ("create_task_list", "createTaskList"),
    ("list_task_lists", "listTaskLists"),
    ("list_tasks", "listTasks"),
    ("add_task", "addTask"),
    ("claim_task", "claimTask"),
    ("complete_task", "completeTask"),
    // KV Stores
    ("create_store", "createStore"),
    ("join_store", "joinStore"),
    ("list_stores", "listStores"),
    ("list_keys", "storeKeys"),
    ("put", "storePut"),
    ("get", "storeGet"),
    ("delete_key", "storeDelete"),
    // Files
    ("send_file", "sendFile"),
    ("transfers", "listTransfers"),
    ("transfer_status", "transferStatus"),
    ("accept_file", "acceptFile"),
    ("reject_file", "rejectFile"),
    // Constitution
    ("constitution", "constitutionJSON"),
    ("constitution_json", "constitutionJSON"),
    // Upgrades
    ("check_upgrade", "checkUpgrade"),
];

/// Verify that each Rust method has a Swift equivalent.
#[test]
fn swift_client_has_all_rust_methods() {
    let swift_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../communitas-apple/Sources/X0xClient/X0xClient.swift"
    );

    let swift_src = match std::fs::read_to_string(swift_path) {
        Ok(src) => src,
        Err(_) => {
            eprintln!(
                "WARNING: Swift client not found at {swift_path} — \
                 skipping parity check. This is expected in CI without \
                 the Apple source checked out."
            );
            return;
        }
    };

    let mut missing = Vec::new();
    for (rust_name, swift_name) in RUST_TO_SWIFT {
        // Look for `func <swift_name>(` in the Swift source
        let pattern = format!("func {swift_name}(");
        if !swift_src.contains(&pattern) {
            // Also try without parens (property accessors)
            let pattern2 = format!("func {swift_name} ");
            if !swift_src.contains(&pattern2) {
                missing.push(format!("{rust_name} -> {swift_name}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Rust methods without Swift equivalents ({} missing):\n  {}\n\n\
         Either add the method to X0xClient.swift or update the mapping in RUST_TO_SWIFT.",
        missing.len(),
        missing.join("\n  ")
    );
}

/// Verify that the mapping covers all Rust methods from client_coverage.
#[test]
fn parity_map_covers_all_rust_methods() {
    let rust_src = include_str!("../src/client.rs");

    // Extract all `pub async fn` and `pub fn` names from client.rs
    let mut client_methods: Vec<&str> = Vec::new();
    for line in rust_src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub async fn ") {
            if let Some(name) = rest.split('(').next() {
                client_methods.push(name.trim());
            }
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            if let Some(name) = rest.split('(').next() {
                // Skip constructors and non-API methods
                let name = name.trim();
                if name != "new"
                    && name != "from_config"
                    && name != "with_base_url"
                    && name != "with_base_url_and_token"
                    && name != "discover"
                    && name != "base_url"
                {
                    client_methods.push(name);
                }
            }
        }
    }

    let mapped: std::collections::HashSet<&str> = RUST_TO_SWIFT.iter().map(|(r, _)| *r).collect();

    let mut unmapped = Vec::new();
    for method in &client_methods {
        if !mapped.contains(method) {
            unmapped.push(*method);
        }
    }

    assert!(
        unmapped.is_empty(),
        "Rust client methods without Swift parity mapping:\n  {}\n\n\
         Add entries to RUST_TO_SWIFT in swift_parity.rs.",
        unmapped.join("\n  ")
    );
}
