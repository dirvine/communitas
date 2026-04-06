// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(dead_code)]

use communitas_x0x_client::{X0xClient, X0xSseStream, X0xWebSocket, discover_x0x_config};
use serde::Deserialize;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Deserialize)]
pub struct TestTarget {
    pub name: String,
    pub address: String,
    pub token: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TargetMatrix {
    targets: Vec<TestTarget>,
}

impl TestTarget {
    pub fn base_url(&self) -> String {
        if self.address.starts_with("http://") || self.address.starts_with("https://") {
            self.address.trim_end_matches('/').to_string()
        } else {
            format!("http://{}", self.address.trim_end_matches('/'))
        }
    }

    pub fn ws_address(&self) -> String {
        self.address
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .trim_start_matches("ws://")
            .trim_start_matches("wss://")
            .trim_end_matches('/')
            .to_string()
    }

    pub fn client(&self) -> X0xClient {
        X0xClient::with_base_url_and_token(&self.base_url(), &self.token)
    }

    pub async fn ws(&self) -> communitas_x0x_client::Result<X0xWebSocket> {
        X0xWebSocket::connect_with_token(&self.ws_address(), &self.token).await
    }

    pub async fn ws_direct(&self) -> communitas_x0x_client::Result<X0xWebSocket> {
        X0xWebSocket::connect_direct_with_token(&self.ws_address(), &self.token).await
    }

    pub async fn sse(&self) -> communitas_x0x_client::Result<X0xSseStream> {
        X0xSseStream::connect_with_token(&self.ws_address(), &self.token, "/events").await
    }

    pub async fn sse_direct(&self) -> communitas_x0x_client::Result<X0xSseStream> {
        X0xSseStream::connect_with_token(&self.ws_address(), &self.token, "/direct/events").await
    }

    pub async fn sse_presence(&self) -> communitas_x0x_client::Result<X0xSseStream> {
        X0xSseStream::connect_with_token(&self.ws_address(), &self.token, "/presence/events").await
    }

    pub fn summary(&self) -> String {
        match (&self.region, &self.role) {
            (Some(region), Some(role)) => format!("{} ({region}, {role})", self.name),
            (Some(region), None) => format!("{} ({region})", self.name),
            _ => self.name.clone(),
        }
    }
}

pub fn load_targets() -> Vec<TestTarget> {
    if let Ok(path) = std::env::var("X0X_TEST_MATRIX_FILE") {
        return load_targets_from_file(Path::new(&path));
    }

    if let Ok(cfg) = discover_x0x_config() {
        return vec![TestTarget {
            name: "local-default".to_string(),
            address: cfg.address,
            token: cfg.token,
            role: Some("default".to_string()),
            region: Some("local".to_string()),
            kind: Some("local".to_string()),
        }];
    }

    panic!(
        "No live x0x target configuration found. Set X0X_TEST_MATRIX_FILE or start a local x0xd."
    );
}

fn load_targets_from_file(path: &Path) -> Vec<TestTarget> {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
    let parsed: TargetMatrix = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("failed to parse {}: {err}", path.display()));
    assert!(
        !parsed.targets.is_empty(),
        "{} did not contain any test targets",
        path.display()
    );
    parsed.targets
}

pub fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos()
}

pub fn mutations_enabled() -> bool {
    matches!(
        std::env::var("X0X_TEST_ALLOW_MUTATION").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

pub fn direct_file_enabled() -> bool {
    matches!(
        std::env::var("X0X_TEST_ENABLE_DIRECT_FILE").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

pub fn cross_node_crdt_enabled() -> bool {
    matches!(
        std::env::var("X0X_TEST_ENABLE_CROSS_NODE_CRDT").as_deref(),
        Ok("1" | "true" | "yes")
    )
}

pub async fn wait_until<F, Fut>(timeout: Duration, interval: Duration, mut check: F) -> bool
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();
    loop {
        if check().await {
            return true;
        }
        if start.elapsed() >= timeout {
            return false;
        }
        tokio::time::sleep(interval).await;
    }
}
