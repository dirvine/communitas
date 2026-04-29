// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared Dioxus E2E harness for parity-matrix cells.
#![allow(dead_code)]
//!
//! The harness mirrors the x0x daemon-fixture pattern: every test gets two
//! isolated `x0xd` daemons, then launches the Communitas Dioxus binary in its
//! feature-gated headless JSON driver mode pointed at daemon A.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use communitas_x0x_client::X0xClient;
use serde_json::{Value, json};
use x0x_test_harness::daemon::DaemonFixture;

const DRIVER_TIMEOUT: Duration = Duration::from_secs(45);

/// Two-daemon Dioxus parity fixture.
pub struct ParityHarness {
    /// Headless Dioxus app driver connected to `primary`.
    pub app: DioxusDriver,
    /// Primary daemon under test; the Dioxus app points at this daemon.
    pub primary: DaemonFixture,
    /// Secondary daemon used to supply remote cards/agent IDs and negative
    /// access-policy checks.
    pub secondary: DaemonFixture,
}

impl ParityHarness {
    /// Start two isolated daemons and the headless Dioxus test driver.
    pub async fn start(prefix: &str) -> Result<Self> {
        let primary = DaemonFixture::start(&format!("{prefix}-a")).await;
        let secondary = DaemonFixture::start(&format!("{prefix}-b")).await;
        let api_base = primary.url("");
        let token = primary.api_token().to_string();
        let mut app = DioxusDriver::start(&api_base, &token)?;
        let hello = app.command(json!({
            "op": "handshake",
            "api_base": api_base,
        }))?;
        ensure_ok(&hello)?;
        Ok(Self {
            app,
            primary,
            secondary,
        })
    }

    /// Typed client for the primary daemon.
    pub fn primary_client(&self) -> X0xClient {
        X0xClient::with_base_url_and_token(&self.primary.url(""), self.primary.api_token())
    }

    /// Typed client for the secondary daemon.
    pub fn secondary_client(&self) -> X0xClient {
        X0xClient::with_base_url_and_token(&self.secondary.url(""), self.secondary.api_token())
    }
}

/// Headless Dioxus binary driver.
pub struct DioxusDriver {
    child: Child,
    stdin: ChildStdin,
    lines: Receiver<String>,
}

impl DioxusDriver {
    /// Launch the Communitas Dioxus binary in JSON-driver mode.
    pub fn start(api_base: &str, token: &str) -> Result<Self> {
        let binary = dioxus_binary_path()?;
        let mut child = Command::new(&binary)
            .env("COMMUNITAS_TEST_MODE", "1")
            .env("X0X_API_BASE", api_base)
            .env("X0X_API_TOKEN", token)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("launch Communitas Dioxus binary at {}", binary.display()))?;

        let stdin = child
            .stdin
            .take()
            .context("Dioxus test driver stdin unavailable")?;
        let stdout = child
            .stdout
            .take()
            .context("Dioxus test driver stdout unavailable")?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line_result in reader.lines() {
                let Ok(line) = line_result else {
                    break;
                };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            lines: rx,
        })
    }

    /// Send one JSON command and wait for one JSON response.
    pub fn command(&mut self, command: Value) -> Result<Value> {
        writeln!(self.stdin, "{command}").context("write Dioxus test-driver command")?;
        self.stdin
            .flush()
            .context("flush Dioxus test-driver command")?;
        let line = self
            .lines
            .recv_timeout(DRIVER_TIMEOUT)
            .context("timed out waiting for Dioxus test-driver response")?;
        serde_json::from_str(&line).with_context(|| format!("parse Dioxus driver JSON: {line}"))
    }
}

impl Drop for DioxusDriver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Assert that a driver response carries `ok: true`.
pub fn ensure_ok(value: &Value) -> Result<()> {
    if value.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        bail!("Dioxus driver returned error response: {value}")
    }
}

/// Extract a response field as string.
pub fn string_field(value: &Value, field: &str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .with_context(|| format!("missing string field `{field}` in {value}"))
}

/// Extract a response field as bool.
pub fn bool_field(value: &Value, field: &str) -> Result<bool> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .with_context(|| format!("missing bool field `{field}` in {value}"))
}

/// Extract a response field as u64.
pub fn u64_field(value: &Value, field: &str) -> Result<u64> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .with_context(|| format!("missing u64 field `{field}` in {value}"))
}

fn dioxus_binary_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_communitas-dioxus") {
        return Ok(PathBuf::from(path));
    }
    if let Ok(path) = std::env::var("CI_DIOXUS_BIN") {
        return Ok(PathBuf::from(path));
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("target");
    path.push("debug");
    path.push(format!("communitas-dioxus{}", std::env::consts::EXE_SUFFIX));
    Ok(path)
}
