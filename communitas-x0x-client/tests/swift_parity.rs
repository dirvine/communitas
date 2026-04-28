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
    ("introduction", "introduction"),
    ("ws_sessions", "wsSessions"),
    // Announcements
    ("announce", "announce"),
    ("announce_with_options", "announceWithOptions"),
    // Discovery & Network
    ("peers", "peers"),
    ("discovered_agents", "discoveredAgents"),
    ("discovered_agent", "discoveredAgent"),
    ("machine_for_agent", "machineForAgent"),
    ("discovered_machines", "discoveredMachines"),
    ("discovered_machine", "discoveredMachine"),
    ("machines_by_user", "machinesByUser"),
    ("presence", "presence"),
    ("network_status", "networkStatus"),
    ("bootstrap_cache", "bootstrapCache"),
    ("connectivity_diagnostics", "connectivityDiagnostics"),
    ("gossip_stats", "gossipStats"),
    ("probe_peer", "probePeer"),
    ("peer_health", "peerHealth"),
    // Gossip
    ("publish", "publish"),
    ("subscribe", "subscribe"),
    ("unsubscribe", "unsubscribe"),
    // Direct Messaging
    ("connect_agent", "connectAgent"),
    ("connect_machine", "connectMachine"),
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
    // Named Groups — core
    ("create_group", "createGroup"),
    ("create_group_with_preset", "createGroupWithPreset"),
    ("list_groups", "listGroups"),
    ("get_group", "groupInfo"),
    ("update_named_group", "updateNamedGroup"),
    ("invite", "invite"),
    ("join_group", "joinGroup"),
    ("set_group_display_name", "setGroupDisplayName"),
    ("leave_group", "leaveGroup"),
    // Named Groups — policy, roles, bans
    ("update_group_policy", "updateGroupPolicy"),
    ("list_named_group_members", "listNamedGroupMembers"),
    ("add_named_group_member", "addNamedGroupMember"),
    ("remove_named_group_member", "removeNamedGroupMember"),
    ("set_named_group_member_role", "setNamedGroupMemberRole"),
    ("ban_group_member", "banGroupMember"),
    ("unban_group_member", "unbanGroupMember"),
    // Named Groups — join requests
    ("list_join_requests", "listJoinRequests"),
    ("create_join_request", "createJoinRequest"),
    ("approve_join_request", "approveJoinRequest"),
    ("reject_join_request", "rejectJoinRequest"),
    ("cancel_join_request", "cancelJoinRequest"),
    // Named Groups — discovery (C + C.2)
    ("discover_groups", "discoverGroups"),
    ("discover_groups_nearby", "discoverGroupsNearby"),
    ("list_shard_subscriptions", "listShardSubscriptions"),
    ("subscribe_directory_shard", "subscribeDirectoryShard"),
    ("unsubscribe_directory_shard", "unsubscribeDirectoryShard"),
    ("get_group_card", "getGroupCard"),
    ("import_group_card", "importGroupCard"),
    // Named Groups — public messaging (Phase E)
    ("send_group_public_message", "sendGroupPublicMessage"),
    ("get_group_public_messages", "getGroupPublicMessages"),
    // Named Groups — state chain (Phase D.3)
    ("get_group_state", "getGroupState"),
    ("seal_group_state", "sealGroupState"),
    ("withdraw_group_state", "withdrawGroupState"),
    // Named Groups — secure plane (Phase D.2)
    ("secure_group_encrypt", "secureGroupEncrypt"),
    ("secure_group_decrypt", "secureGroupDecrypt"),
    ("secure_group_reseal", "secureGroupReseal"),
    ("secure_open_envelope", "secureOpenEnvelope"),
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
    // Constitution & GUI
    ("constitution", "constitution"),
    ("constitution_json", "constitutionJSON"),
    ("gui_html", "guiHTML"),
    // Upgrades
    ("check_upgrade", "checkUpgrade"),
    // Presence (extended)
    ("presence_online", "presenceOnline"),
    ("presence_foaf", "presenceFoaf"),
    ("presence_find", "presenceFind"),
    ("presence_status", "presenceStatus"),
    // Agent discovery (extended)
    ("agent_reachability", "agentReachability"),
    ("find_agent", "findAgent"),
    // User agents
    ("user_agents", "userAgents"),
];

/// Maps Rust `X0xSseStream` methods to their Swift counterparts on
/// `X0xSseStream`. The Rust side lives in `src/sse.rs`; the Swift side
/// lives in `Sources/X0xClient/X0xSseStream.swift`.
const RUST_SSE_TO_SWIFT: &[(&str, &str)] = &[
    ("connect", "connect"),
    ("connect_direct", "connectDirect"),
    ("connect_presence", "connectPresence"),
    ("connect_peer_events", "connectPeerEvents"),
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
        if let Some(rest) = trimmed.strip_prefix("pub async fn ")
            && let Some(name) = rest.split('(').next()
        {
            client_methods.push(name.trim());
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ")
            && let Some(name) = rest.split('(').next()
        {
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

/// Verify that each Rust `X0xSseStream` method has a Swift equivalent.
///
/// Mirrors `swift_client_has_all_rust_methods` but for the SSE consumer
/// (`src/sse.rs` ↔ `Sources/X0xClient/X0xSseStream.swift`).
#[test]
fn swift_sse_stream_has_all_rust_methods() {
    let swift_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../communitas-apple/Sources/X0xClient/X0xSseStream.swift"
    );

    let swift_src = match std::fs::read_to_string(swift_path) {
        Ok(src) => src,
        Err(_) => {
            eprintln!(
                "WARNING: Swift X0xSseStream not found at {swift_path} — \
                 skipping parity check. This is expected in CI without \
                 the Apple source checked out."
            );
            return;
        }
    };

    let mut missing = Vec::new();
    for (rust_name, swift_name) in RUST_SSE_TO_SWIFT {
        let pattern = format!("func {swift_name}(");
        if !swift_src.contains(&pattern) {
            missing.push(format!("{rust_name} -> {swift_name}"));
        }
    }

    assert!(
        missing.is_empty(),
        "Rust X0xSseStream methods without Swift equivalents ({} missing):\n  {}\n\n\
         Either add the method to X0xSseStream.swift or update the mapping in RUST_SSE_TO_SWIFT.",
        missing.len(),
        missing.join("\n  ")
    );
}
