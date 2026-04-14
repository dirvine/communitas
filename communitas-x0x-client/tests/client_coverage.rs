// SPDX-License-Identifier: MIT OR Apache-2.0

//! API coverage guardian for the communitas x0x client.
//!
//! Every public method on X0xClient and X0xWebSocket MUST have an entry
//! in the COVERED list below. When a new method is added, this test fails
//! until it is listed here AND a contract test is written.
//!
//! Run: cargo nextest run -p communitas-x0x-client --test client_coverage

use std::collections::HashSet;

// ── Every public method on X0xClient that wraps an x0x API endpoint ────
//
// Format: (method_name, http_method, path)
// When you add a method to X0xClient, add it here too.
const COVERED_REST: &[(&str, &str, &str)] = &[
    // System & Identity
    ("health", "GET", "/health"),
    ("status", "GET", "/status"),
    ("shutdown", "POST", "/shutdown"),
    ("agent", "GET", "/agent"),
    ("agent_user_id", "GET", "/agent/user-id"),
    ("agent_card", "GET", "/agent/card"),
    ("import_agent_card", "POST", "/agent/card/import"),
    ("introduction", "GET", "/introduction"),
    ("ws_sessions", "GET", "/ws/sessions"),
    // Announcements
    ("announce", "POST", "/announce"),
    ("announce_with_options", "POST", "/announce"),
    // Discovery & Network
    ("peers", "GET", "/peers"),
    ("discovered_agents", "GET", "/agents/discovered"),
    ("discovered_agent", "GET", "/agents/discovered/:agent_id"),
    ("presence", "GET", "/presence"),
    ("network_status", "GET", "/network/status"),
    ("bootstrap_cache", "GET", "/network/bootstrap-cache"),
    // Gossip Pub/Sub
    ("publish", "POST", "/publish"),
    ("subscribe", "POST", "/subscribe"),
    ("unsubscribe", "DELETE", "/subscribe/:id"),
    // Direct Messaging
    ("connect_agent", "POST", "/agents/connect"),
    ("send_direct", "POST", "/direct/send"),
    ("direct_connections", "GET", "/direct/connections"),
    // Contacts
    ("list_contacts", "GET", "/contacts"),
    ("add_contact", "POST", "/contacts"),
    ("set_trust", "POST", "/contacts/trust"),
    ("update_contact", "PATCH", "/contacts/:agent_id"),
    ("remove_contact", "DELETE", "/contacts/:agent_id"),
    ("revoke_contact", "POST", "/contacts/:agent_id/revoke"),
    ("revocations", "GET", "/contacts/:agent_id/revocations"),
    // Machines
    ("list_machines", "GET", "/contacts/:agent_id/machines"),
    ("add_machine", "POST", "/contacts/:agent_id/machines"),
    (
        "remove_machine",
        "DELETE",
        "/contacts/:agent_id/machines/:machine_id",
    ),
    (
        "pin_machine",
        "POST",
        "/contacts/:agent_id/machines/:machine_id/pin",
    ),
    (
        "unpin_machine",
        "DELETE",
        "/contacts/:agent_id/machines/:machine_id/pin",
    ),
    // Trust
    ("evaluate_trust", "POST", "/trust/evaluate"),
    // MLS Groups
    ("create_mls_group", "POST", "/mls/groups"),
    ("list_mls_groups", "GET", "/mls/groups"),
    ("get_mls_group", "GET", "/mls/groups/:id"),
    ("add_mls_member", "POST", "/mls/groups/:id/members"),
    (
        "remove_mls_member",
        "DELETE",
        "/mls/groups/:id/members/:agent_id",
    ),
    ("encrypt", "POST", "/mls/groups/:id/encrypt"),
    ("decrypt", "POST", "/mls/groups/:id/decrypt"),
    ("create_mls_welcome", "POST", "/mls/groups/:id/welcome"),
    // Named Groups — core
    ("create_group", "POST", "/groups"),
    ("create_group_with_preset", "POST", "/groups"),
    ("list_groups", "GET", "/groups"),
    ("get_group", "GET", "/groups/:id"),
    ("update_named_group", "PATCH", "/groups/:id"),
    ("invite", "POST", "/groups/:id/invite"),
    ("join_group", "POST", "/groups/join"),
    ("set_group_display_name", "PUT", "/groups/:id/display-name"),
    ("leave_group", "DELETE", "/groups/:id"),
    // Named Groups — policy, roles, bans
    ("update_group_policy", "PATCH", "/groups/:id/policy"),
    ("list_named_group_members", "GET", "/groups/:id/members"),
    ("add_named_group_member", "POST", "/groups/:id/members"),
    ("remove_named_group_member", "DELETE", "/groups/:id/members/:agent_id"),
    ("set_named_group_member_role", "PATCH", "/groups/:id/members/:agent_id/role"),
    ("ban_group_member", "POST", "/groups/:id/ban/:agent_id"),
    ("unban_group_member", "DELETE", "/groups/:id/ban/:agent_id"),
    // Named Groups — join requests
    ("list_join_requests", "GET", "/groups/:id/requests"),
    ("create_join_request", "POST", "/groups/:id/requests"),
    ("approve_join_request", "POST", "/groups/:id/requests/:request_id/approve"),
    ("reject_join_request", "POST", "/groups/:id/requests/:request_id/reject"),
    ("cancel_join_request", "DELETE", "/groups/:id/requests/:request_id"),
    // Named Groups — discovery (C + C.2)
    ("discover_groups", "GET", "/groups/discover"),
    ("discover_groups_nearby", "GET", "/groups/discover/nearby"),
    ("list_shard_subscriptions", "GET", "/groups/discover/subscriptions"),
    ("subscribe_directory_shard", "POST", "/groups/discover/subscribe"),
    (
        "unsubscribe_directory_shard",
        "DELETE",
        "/groups/discover/subscribe/:kind/:shard",
    ),
    ("get_group_card", "GET", "/groups/cards/:id"),
    ("import_group_card", "POST", "/groups/cards/import"),
    // Named Groups — public messaging (Phase E)
    ("send_group_public_message", "POST", "/groups/:id/send"),
    ("get_group_public_messages", "GET", "/groups/:id/messages"),
    // Named Groups — state chain (Phase D.3)
    ("get_group_state", "GET", "/groups/:id/state"),
    ("seal_group_state", "POST", "/groups/:id/state/seal"),
    ("withdraw_group_state", "POST", "/groups/:id/state/withdraw"),
    // Named Groups — secure plane (Phase D.2)
    ("secure_group_encrypt", "POST", "/groups/:id/secure/encrypt"),
    ("secure_group_decrypt", "POST", "/groups/:id/secure/decrypt"),
    ("secure_group_reseal", "POST", "/groups/:id/secure/reseal"),
    ("secure_open_envelope", "POST", "/groups/secure/open-envelope"),
    // Task Lists
    ("create_task_list", "POST", "/task-lists"),
    ("list_task_lists", "GET", "/task-lists"),
    ("list_tasks", "GET", "/task-lists/:id/tasks"),
    ("add_task", "POST", "/task-lists/:id/tasks"),
    ("claim_task", "PATCH", "/task-lists/:id/tasks/:tid"),
    ("complete_task", "PATCH", "/task-lists/:id/tasks/:tid"),
    // Key-Value Stores
    ("create_store", "POST", "/stores"),
    ("join_store", "POST", "/stores/:id/join"),
    ("list_stores", "GET", "/stores"),
    ("list_keys", "GET", "/stores/:id/keys"),
    ("put", "PUT", "/stores/:id/:key"),
    ("get", "GET", "/stores/:id/:key"),
    ("delete_key", "DELETE", "/stores/:id/:key"),
    // Files
    ("send_file", "POST", "/files/send"),
    ("transfers", "GET", "/files/transfers"),
    ("transfer_status", "GET", "/files/transfers/:id"),
    ("accept_file", "POST", "/files/accept/:id"),
    ("reject_file", "POST", "/files/reject/:id"),
    // Constitution & GUI
    ("constitution", "GET", "/constitution"),
    ("constitution_json", "GET", "/constitution/json"),
    ("gui_html", "GET", "/gui"),
    // Upgrades
    ("check_upgrade", "GET", "/upgrade"),
    // Presence (extended)
    ("presence_online", "GET", "/presence/online"),
    ("presence_foaf", "GET", "/presence/foaf"),
    ("presence_find", "GET", "/presence/find/:id"),
    ("presence_status", "GET", "/presence/status/:id"),
    // Agent discovery (extended)
    (
        "agent_reachability",
        "GET",
        "/agents/reachability/:agent_id",
    ),
    ("find_agent", "POST", "/agents/find/:agent_id"),
    // User agents
    ("user_agents", "GET", "/users/:user_id/agents"),
];

/// Every public method on X0xWebSocket.
const COVERED_WS: &[&str] = &[
    "connect",
    "connect_direct",
    "connect_to",
    "connect_with_token",
    "connect_direct_with_token",
    "subscribe",
    "unsubscribe",
    "publish",
    "send_direct",
    "ping",
];

/// Every public method on X0xSseStream.
const COVERED_SSE: &[&str] = &[
    "connect",
    "connect_direct",
    "connect_presence",
    "connect_to",
    "connect_with_token",
    "recv",
];

/// Verifies no duplicate method names in COVERED_REST.
#[test]
fn no_duplicate_method_names() {
    let mut seen = HashSet::new();
    let mut dupes = Vec::new();
    for (name, _, _) in COVERED_REST {
        if !seen.insert(*name) {
            dupes.push(*name);
        }
    }
    assert!(
        dupes.is_empty(),
        "Duplicate method names in COVERED_REST: {:?}",
        dupes
    );
}

/// Verifies no duplicate paths (same method+path).
#[test]
fn no_duplicate_endpoints() {
    let mut seen = HashSet::new();
    let mut dupes = Vec::new();
    for (name, method, path) in COVERED_REST {
        let key = format!("{method} {path}");
        // allow intentional duplicates (e.g., announce and announce_with_options)
        if !seen.insert(key.clone()) && *name != "announce_with_options" {
            dupes.push(format!("{name}: {key}"));
        }
    }
    // Allow intentional shared endpoints:
    // - announce / announce_with_options share POST /announce
    // - claim_task / complete_task share PATCH /task-lists/:id/tasks/:tid (different action param)
    // - create_group / create_group_with_preset share POST /groups (preset-aware overload)
    let real_dupes: Vec<_> = dupes
        .iter()
        .filter(|d| {
            !d.contains("announce")
                && !d.contains("task")
                && !d.contains("create_group_with_preset")
        })
        .collect();
    assert!(
        real_dupes.is_empty(),
        "Duplicate endpoints in COVERED_REST: {:?}",
        real_dupes
    );
}

/// Verifies all WebSocket method names are unique.
#[test]
fn no_duplicate_ws_methods() {
    let mut seen = HashSet::new();
    let mut dupes = Vec::new();
    for name in COVERED_WS {
        if !seen.insert(*name) {
            dupes.push(*name);
        }
    }
    assert!(dupes.is_empty(), "Duplicate WS method names: {:?}", dupes);
}

/// Verifies all SSE method names are unique.
#[test]
fn no_duplicate_sse_methods() {
    let mut seen = HashSet::new();
    let mut dupes = Vec::new();
    for name in COVERED_SSE {
        if !seen.insert(*name) {
            dupes.push(*name);
        }
    }
    assert!(dupes.is_empty(), "Duplicate SSE method names: {:?}", dupes);
}

/// Coverage count matches expected total.
#[test]
fn coverage_count_is_correct() {
    assert!(
        COVERED_REST.len() >= 80,
        "Expected at least 80 REST methods, got {}. \
         Did you add a method to X0xClient without updating COVERED_REST?",
        COVERED_REST.len()
    );
    assert!(
        COVERED_WS.len() >= 9,
        "Expected at least 9 WS methods, got {}",
        COVERED_WS.len()
    );
    assert!(
        COVERED_SSE.len() >= 6,
        "Expected at least 6 SSE methods, got {}",
        COVERED_SSE.len()
    );
}

/// Verifies coverage stays in lockstep with the sibling x0x endpoint registry.
///
/// We deliberately exempt `/gui/` because it is only an alias of `/gui`.
#[test]
fn coverage_matches_x0x_registry_when_available() {
    let registry_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../x0x/src/api/mod.rs");

    let Ok(src) = std::fs::read_to_string(&registry_path) else {
        eprintln!(
            "WARNING: x0x registry not found at {} — skipping live parity check",
            registry_path.display()
        );
        return;
    };

    let mut registry = HashSet::new();
    for block in src.split("EndpointDef {").skip(1) {
        let method = block
            .split("method: Method::")
            .nth(1)
            .and_then(|rest| rest.split(',').next())
            .map(str::trim);
        let path = block
            .split("path: \"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .map(str::trim);

        let (Some(method), Some(path)) = (method, path) else {
            continue;
        };

        let http_method = match method {
            "Get" => "GET",
            "Post" => "POST",
            "Put" => "PUT",
            "Patch" => "PATCH",
            "Delete" => "DELETE",
            _ => continue,
        };
        registry.insert((http_method.to_string(), path.to_string()));
    }

    let mut covered: HashSet<(String, String)> = COVERED_REST
        .iter()
        .map(|(_, method, path)| ((*method).to_string(), (*path).to_string()))
        .collect();
    covered.extend([
        ("GET".to_string(), "/events".to_string()),
        ("GET".to_string(), "/direct/events".to_string()),
        ("GET".to_string(), "/presence/events".to_string()),
        ("GET".to_string(), "/ws".to_string()),
        ("GET".to_string(), "/ws/direct".to_string()),
    ]);

    let exempt = HashSet::from([("GET".to_string(), "/gui/".to_string())]);

    let mut missing: Vec<_> = registry
        .difference(&covered)
        .filter(|endpoint| !exempt.contains(*endpoint))
        .cloned()
        .collect();
    missing.sort();

    assert!(
        missing.is_empty(),
        "x0x registry endpoints not covered by communitas-x0x-client: {:?}",
        missing
    );
}

/// Verifies the source file contains each method name.
///
/// This catches methods that are removed from the client but left
/// in COVERED_REST (stale entries).
#[test]
fn covered_methods_exist_in_source() {
    let client_src = include_str!("../src/client.rs");

    let mut missing = Vec::new();
    for (name, _, _) in COVERED_REST {
        // Look for `pub async fn <name>` or `pub fn <name>`
        let pattern_async = format!("pub async fn {name}");
        let pattern_sync = format!("pub fn {name}");
        if !client_src.contains(&pattern_async) && !client_src.contains(&pattern_sync) {
            missing.push(*name);
        }
    }
    assert!(
        missing.is_empty(),
        "Methods in COVERED_REST not found in client.rs: {:?}\n\
         Remove stale entries or check for typos.",
        missing
    );
}

/// Verifies the WebSocket source contains each method name.
#[test]
fn covered_ws_methods_exist_in_source() {
    let ws_src = include_str!("../src/websocket.rs");

    let mut missing = Vec::new();
    for name in COVERED_WS {
        let pattern_async = format!("pub async fn {name}");
        let pattern_sync = format!("pub fn {name}");
        if !ws_src.contains(&pattern_async) && !ws_src.contains(&pattern_sync) {
            missing.push(*name);
        }
    }
    assert!(
        missing.is_empty(),
        "Methods in COVERED_WS not found in websocket.rs: {:?}",
        missing
    );
}

/// Verifies the SSE source contains each method name.
#[test]
fn covered_sse_methods_exist_in_source() {
    let sse_src = include_str!("../src/sse.rs");

    let mut missing = Vec::new();
    for name in COVERED_SSE {
        let pattern_async = format!("pub async fn {name}");
        let pattern_sync = format!("pub fn {name}");
        if !sse_src.contains(&pattern_async) && !sse_src.contains(&pattern_sync) {
            missing.push(*name);
        }
    }
    assert!(
        missing.is_empty(),
        "Methods in COVERED_SSE not found in sse.rs: {:?}",
        missing
    );
}

/// Verifies every public method in client.rs is tracked in COVERED_REST.
///
/// This is the critical reverse check: `covered_methods_exist_in_source`
/// catches stale entries (method removed but still listed), while this test
/// catches new methods added to the client without updating the coverage list.
#[test]
fn all_client_methods_are_covered() {
    let client_src = include_str!("../src/client.rs");

    // Non-API methods that should be excluded from coverage tracking
    let excluded: HashSet<&str> = [
        "new",
        "from_config",
        "with_base_url",
        "with_base_url_and_token",
        "discover",
        "base_url",
    ]
    .into_iter()
    .collect();

    let covered: HashSet<&str> = COVERED_REST.iter().map(|(name, _, _)| *name).collect();

    let mut uncovered = Vec::new();
    for line in client_src.lines() {
        let trimmed = line.trim();
        let name = if let Some(rest) = trimmed.strip_prefix("pub async fn ") {
            rest.split('(').next().map(str::trim)
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            rest.split('(').next().map(str::trim)
        } else {
            None
        };
        if let Some(name) = name
            && !excluded.contains(name)
            && !covered.contains(name)
        {
            uncovered.push(name.to_string());
        }
    }

    assert!(
        uncovered.is_empty(),
        "Client methods NOT tracked in COVERED_REST ({} missing):\n  {}\n\n\
         To fix: add entries to COVERED_REST in client_coverage.rs.",
        uncovered.len(),
        uncovered.join("\n  ")
    );
}

/// Verifies every public method in websocket.rs is tracked in COVERED_WS.
#[test]
fn all_ws_methods_are_covered() {
    let ws_src = include_str!("../src/websocket.rs");

    let excluded: HashSet<&str> = ["new", "close", "recv", "is_connected"]
        .into_iter()
        .collect();

    let covered: HashSet<&str> = COVERED_WS.iter().copied().collect();

    let mut uncovered = Vec::new();
    for line in ws_src.lines() {
        let trimmed = line.trim();
        let name = if let Some(rest) = trimmed.strip_prefix("pub async fn ") {
            rest.split('(').next().map(str::trim)
        } else if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            rest.split('(').next().map(str::trim)
        } else {
            None
        };
        if let Some(name) = name
            && !excluded.contains(name)
            && !covered.contains(name)
        {
            uncovered.push(name.to_string());
        }
    }

    assert!(
        uncovered.is_empty(),
        "WebSocket methods NOT tracked in COVERED_WS ({} missing):\n  {}\n\n\
         To fix: add entries to COVERED_WS in client_coverage.rs.",
        uncovered.len(),
        uncovered.join("\n  ")
    );
}

/// Summary: prints a human-readable coverage report.
#[test]
fn print_coverage_summary() {
    let total_rest = COVERED_REST.len();
    let total_ws = COVERED_WS.len();
    eprintln!("\n=== communitas-x0x-client API Coverage ===");
    eprintln!("  REST methods tracked: {total_rest}");
    eprintln!("  WebSocket methods tracked: {total_ws}");
    eprintln!("  Total: {}", total_rest + total_ws);
    eprintln!("==========================================\n");
}
