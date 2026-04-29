// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for upgrade/self-update cells.

mod harness;

use anyhow::{Result, ensure};
use serde_json::json;

use harness::{ParityHarness, ensure_ok, u64_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_upgrade_check_updates() -> Result<()> {
    let mut harness = ParityHarness::start("upgrade-check").await?;
    let response = harness.app.command(json!({ "op": "upgrade.check" }))?;
    ensure_ok(&response)?;
    let status = u64_field(&response, "http_status")?;
    ensure!(status != 404, "GET /upgrade must be routed by x0xd");
    ensure!(
        response.get("body").is_some(),
        "upgrade check should return a structured response body"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_upgrade_apply_update_endpoint() -> Result<()> {
    let mut harness = ParityHarness::start("upgrade-apply").await?;
    let response = harness.app.command(json!({ "op": "upgrade.apply" }))?;
    ensure_ok(&response)?;
    let status = u64_field(&response, "http_status")?;
    ensure!(status != 404, "POST /upgrade/apply must be routed by x0xd");
    ensure!(
        response.get("body").is_some(),
        "upgrade apply should return a structured response body"
    );
    Ok(())
}
