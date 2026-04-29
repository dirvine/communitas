// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for identity cells.

mod harness;

use anyhow::{Result, ensure};
use serde_json::json;

use harness::{ParityHarness, ensure_ok, string_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_identity_get_agent_id_card() -> Result<()> {
    let mut harness = ParityHarness::start("identity-card").await?;
    let response = harness
        .app
        .command(json!({ "op": "identity.agent_card" }))?;
    ensure_ok(&response)?;
    let agent_id = string_field(&response, "agent_id")?;
    ensure!(!agent_id.is_empty(), "agent id must not be empty");
    ensure!(
        agent_id == string_field(&response, "card_agent_id")?,
        "agent card must describe the active daemon"
    );
    ensure!(
        string_field(&response, "link")?.starts_with("x0x://agent/"),
        "agent card link must use x0x://agent/"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_identity_import_agent_card() -> Result<()> {
    let mut harness = ParityHarness::start("identity-import").await?;
    let secondary = harness.secondary_client();
    let secondary_card = secondary.agent_card(Some("Dioxus B"), Some(false)).await?;
    let secondary_identity = secondary.agent().await?;

    let response = harness.app.command(json!({
        "op": "identity.import_card",
        "card": secondary_card.link,
    }))?;
    ensure_ok(&response)?;
    ensure!(
        string_field(&response, "agent_id")? == secondary_identity.agent_id,
        "imported card should create the secondary agent contact"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_identity_export_keypairs_gap_recorded() -> Result<()> {
    let mut harness = ParityHarness::start("identity-export-gap").await?;
    let response = harness
        .app
        .command(json!({ "op": "identity.export_keypairs" }))?;
    ensure_ok(&response)?;
    ensure!(
        string_field(&response, "status")? == "unsupported",
        "Dioxus must record the missing client keypair-export method rather than fake coverage"
    );
    ensure!(
        string_field(&response, "reason")?.contains("communitas-x0x-client"),
        "gap reason should name the missing client surface"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_identity_user_identity_opt_in_read() -> Result<()> {
    let mut harness = ParityHarness::start("identity-user").await?;
    let response = harness.app.command(json!({ "op": "identity.user_id" }))?;
    ensure_ok(&response)?;
    ensure!(
        response
            .get("opt_in_state_read")
            .and_then(serde_json::Value::as_bool)
            == Some(true),
        "user identity opt-in state should be readable"
    );
    Ok(())
}
