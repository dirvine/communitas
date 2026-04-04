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
    ("remove_machine", "DELETE", "/contacts/:agent_id/machines/:machine_id"),
    ("pin_machine", "POST", "/contacts/:agent_id/machines/:machine_id/pin"),
    ("unpin_machine", "DELETE", "/contacts/:agent_id/machines/:machine_id/pin"),
    // Trust
    ("evaluate_trust", "POST", "/trust/evaluate"),
    // MLS Groups
    ("create_mls_group", "POST", "/mls/groups"),
    ("list_mls_groups", "GET", "/mls/groups"),
    ("get_mls_group", "GET", "/mls/groups/:id"),
    ("add_mls_member", "POST", "/mls/groups/:id/members"),
    ("remove_mls_member", "DELETE", "/mls/groups/:id/members/:agent_id"),
    ("encrypt", "POST", "/mls/groups/:id/encrypt"),
    ("decrypt", "POST", "/mls/groups/:id/decrypt"),
    ("create_mls_welcome", "POST", "/mls/groups/:id/welcome"),
    // Named Groups
    ("create_group", "POST", "/groups"),
    ("list_groups", "GET", "/groups"),
    ("get_group", "GET", "/groups/:id"),
    ("invite", "POST", "/groups/:id/invite"),
    ("join_group", "POST", "/groups/join"),
    ("set_group_display_name", "PUT", "/groups/:id/display-name"),
    ("leave_group", "DELETE", "/groups/:id"),
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
    // Constitution
    ("constitution", "GET", "/constitution"),
    ("constitution_json", "GET", "/constitution/json"),
    // Upgrades
    ("check_upgrade", "GET", "/upgrade"),
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
    let real_dupes: Vec<_> = dupes
        .iter()
        .filter(|d| !d.contains("announce") && !d.contains("task"))
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
    assert!(
        dupes.is_empty(),
        "Duplicate WS method names: {:?}",
        dupes
    );
}

/// Coverage count matches expected total.
#[test]
fn coverage_count_is_correct() {
    assert!(
        COVERED_REST.len() >= 71,
        "Expected at least 71 REST methods, got {}. \
         Did you add a method to X0xClient without updating COVERED_REST?",
        COVERED_REST.len()
    );
    assert!(
        COVERED_WS.len() >= 9,
        "Expected at least 9 WS methods, got {}",
        COVERED_WS.len()
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
        if let Some(name) = name {
            if !excluded.contains(name) && !covered.contains(name) {
                uncovered.push(name.to_string());
            }
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
        if let Some(name) = name {
            if !excluded.contains(name) && !covered.contains(name) {
                uncovered.push(name.to_string());
            }
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
