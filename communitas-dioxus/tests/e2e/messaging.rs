// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus parity tests for live messaging cells.

mod harness;

use anyhow::Result;
use serde_json::json;

use harness::{ParityHarness, ensure_ok, string_field};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "live x0xd + Dioxus binary E2E harness"]
async fn dioxus_parity_messaging_publish_receives_ws_payload() -> Result<()> {
    let mut harness = ParityHarness::start("messaging-pubsub").await?;
    let response = harness
        .app
        .command(json!({ "op": "messaging.pubsub_roundtrip" }))?;
    ensure_ok(&response)?;
    let stream = string_field(&response, "stream")?;
    let topic = string_field(&response, "topic")?;
    let payload = string_field(&response, "payload")?;
    assert_eq!(stream, "ws");
    assert!(topic.starts_with("dioxus-pubsub-"));
    assert_eq!(payload, format!("hello-{topic}"));
    Ok(())
}
