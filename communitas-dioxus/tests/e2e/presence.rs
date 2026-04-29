// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for presence cells.

mod harness;

use anyhow::Result;
use serde_json::json;

use harness::{ParityHarness, ensure_ok, u64_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_presence_foaf_walk() -> Result<()> {
    let mut harness = ParityHarness::start("presence-foaf").await?;
    let response = harness.app.command(json!({ "op": "presence.foaf" }))?;
    ensure_ok(&response)?;
    let _ = u64_field(&response, "agents")?;
    Ok(())
}
