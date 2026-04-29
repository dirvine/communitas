// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for connectivity/discovery cells.

mod harness;

use anyhow::Result;
use serde_json::json;

use harness::{ParityHarness, ensure_ok, u64_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_connectivity_discover_agents_cache_foaf() -> Result<()> {
    let mut harness = ParityHarness::start("connectivity-discover").await?;
    let response = harness
        .app
        .command(json!({ "op": "connectivity.discover_agents" }))?;
    ensure_ok(&response)?;
    let _ = u64_field(&response, "discovered_agents")?;
    let _ = u64_field(&response, "foaf_agents")?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_connectivity_four_word_network_bootstrap() -> Result<()> {
    let mut harness = ParityHarness::start("connectivity-bootstrap").await?;
    let response = harness
        .app
        .command(json!({ "op": "connectivity.four_word_bootstrap" }))?;
    ensure_ok(&response)?;
    let _ = u64_field(&response, "bootstrap_connection_count")?;
    let _ = u64_field(&response, "bootstrap_peers")?;
    Ok(())
}
