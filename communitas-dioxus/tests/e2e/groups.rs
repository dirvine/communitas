// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for named-group cells.

mod harness;

use anyhow::{Result, ensure};
use serde_json::json;

use harness::{ParityHarness, bool_field, ensure_ok, string_field, u64_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_groups_policy_roles_bans() -> Result<()> {
    let mut harness = ParityHarness::start("groups-policy").await?;
    let secondary_identity = harness.secondary_client().agent().await?;
    let response = harness.app.command(json!({
        "op": "groups.policy",
        "member_agent_id": secondary_identity.agent_id,
    }))?;
    ensure_ok(&response)?;
    ensure!(
        !string_field(&response, "group_id")?.is_empty(),
        "policy test should create a named group"
    );
    ensure!(
        bool_field(&response, "policy_present")?,
        "created group should expose its policy"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_groups_discover_tag_nearby() -> Result<()> {
    let mut harness = ParityHarness::start("groups-discover").await?;
    let response = harness.app.command(json!({ "op": "groups.discover" }))?;
    ensure_ok(&response)?;
    ensure!(
        !string_field(&response, "group_id")?.is_empty(),
        "discover test should create a discoverable group"
    );
    let _ = u64_field(&response, "query_count")?;
    let _ = u64_field(&response, "nearby_count")?;
    Ok(())
}
