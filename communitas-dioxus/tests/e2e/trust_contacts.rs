// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for trust and contacts cells.

mod harness;

use anyhow::{Result, ensure};
use serde_json::json;

use harness::{ParityHarness, bool_field, ensure_ok, string_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_trust_add_block_trust_contact() -> Result<()> {
    let mut harness = ParityHarness::start("trust-contact").await?;
    let secondary_identity = harness.secondary_client().agent().await?;
    let response = harness.app.command(json!({
        "op": "trust.add_block_trust",
        "agent_id": secondary_identity.agent_id,
        "label": "Dioxus Trust Fixture",
    }))?;
    ensure_ok(&response)?;
    ensure!(
        string_field(&response, "final_trust")? == "Blocked",
        "trust flow should end with the contact blocked"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_trust_machine_pinning() -> Result<()> {
    let mut harness = ParityHarness::start("trust-pin").await?;
    let secondary_identity = harness.secondary_client().agent().await?;
    let response = harness.app.command(json!({
        "op": "trust.machine_pin",
        "agent_id": secondary_identity.agent_id,
        "machine_id": secondary_identity.machine_id,
        "label": "Dioxus Machine Fixture",
    }))?;
    ensure_ok(&response)?;
    ensure!(bool_field(&response, "pinned")?, "machine must be pinned");
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_trust_evaluator_decision_read() -> Result<()> {
    let mut harness = ParityHarness::start("trust-evaluate").await?;
    let secondary_identity = harness.secondary_client().agent().await?;
    let response = harness.app.command(json!({
        "op": "trust.evaluate",
        "agent_id": secondary_identity.agent_id,
        "machine_id": secondary_identity.machine_id,
    }))?;
    ensure_ok(&response)?;
    ensure!(
        !string_field(&response, "decision")?.is_empty(),
        "trust evaluator decision must be populated"
    );
    Ok(())
}
