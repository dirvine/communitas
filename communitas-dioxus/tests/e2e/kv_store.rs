// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for KV-store cells.

mod harness;

use anyhow::{Result, ensure};
use serde_json::json;

use harness::{ParityHarness, bool_field, ensure_ok, string_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_kv_create_list_stores() -> Result<()> {
    let mut harness = ParityHarness::start("kv-list").await?;
    let response = harness.app.command(json!({ "op": "kv.create_list" }))?;
    ensure_ok(&response)?;
    ensure!(
        bool_field(&response, "listed")?,
        "created store should be present in the list"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_kv_put_get_delete_key() -> Result<()> {
    let mut harness = ParityHarness::start("kv-crud").await?;
    let response = harness.app.command(json!({ "op": "kv.put_get_delete" }))?;
    ensure_ok(&response)?;
    ensure!(
        bool_field(&response, "missing_after_delete")?,
        "deleted key should no longer be readable"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_kv_access_policy_enforcement() -> Result<()> {
    let mut harness = ParityHarness::start("kv-policy").await?;
    let response = harness
        .app
        .command(json!({ "op": "kv.access_policy_setup" }))?;
    ensure_ok(&response)?;
    let store_id = string_field(&response, "store_id")?;
    let key = string_field(&response, "key")?;
    let foreign_read = harness.secondary_client().get(&store_id, &key).await;
    ensure!(
        foreign_read.is_err(),
        "secondary daemon must not read the primary daemon's private store"
    );
    Ok(())
}
