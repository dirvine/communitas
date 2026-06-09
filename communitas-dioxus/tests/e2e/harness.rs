// SPDX-License-Identifier: MIT OR Apache-2.0

//! Shared Dioxus E2E harness for parity-matrix cells.
#![allow(dead_code)]
//!
//! The harness mirrors the x0x daemon-fixture pattern: every test gets two
//! isolated `x0xd` daemons, then launches the Communitas Dioxus binary in its
//! feature-gated headless JSON driver mode pointed at daemon A.

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use communitas_x0x_client::X0xClient;
use serde_json::{Value, json};
use tempfile::TempDir;

const DRIVER_TIMEOUT: Duration = Duration::from_secs(45);
const DAEMON_STARTUP_TIMEOUT: Duration = Duration::from_secs(45);

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
        let primary = DaemonFixture::start(&format!("{prefix}-a")).await?;
        let secondary = DaemonFixture::start(&format!("{prefix}-b")).await?;
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

/// Isolated x0xd daemon fixture for Communitas Dioxus E2E tests.
pub struct DaemonFixture {
    process: Child,
    api_addr: String,
    api_token: String,
    tempdir: TempDir,
    stdout_log: PathBuf,
    stderr_log: PathBuf,
}

impl DaemonFixture {
    /// Start a fresh daemon, preferring `X0XD_BIN` when supplied by CI or scripts.
    pub async fn start(_prefix: &str) -> Result<Self> {
        let binary = find_x0xd_binary()?;
        let tempdir = TempDir::new().context("create daemon temp dir")?;
        let config_path = tempdir.path().join("config.toml");
        let config = format!(
            "bind_address = \"0.0.0.0:0\"\napi_address = \"127.0.0.1:0\"\ndata_dir = \"{}\"\nlog_level = \"warn\"\nbootstrap_peers = []\n",
            tempdir.path().display(),
        );
        fs::write(&config_path, config).context("write daemon config")?;

        let stdout_log = tempdir.path().join("daemon.stdout.log");
        let stderr_log = tempdir.path().join("daemon.stderr.log");
        let stdout = File::create(&stdout_log).context("create daemon stdout log")?;
        let stderr = File::create(&stderr_log).context("create daemon stderr log")?;
        let process = Command::new(&binary)
            .arg("--config")
            .arg(&config_path)
            .arg("--skip-update-check")
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .with_context(|| format!("start x0xd at {}", binary.display()))?;

        let mut fixture = Self {
            process,
            api_addr: String::new(),
            api_token: String::new(),
            tempdir,
            stdout_log,
            stderr_log,
        };
        fixture.wait_for_startup().await?;
        Ok(fixture)
    }

    async fn wait_for_startup(&mut self) -> Result<()> {
        let port_file = self.port_file();
        let deadline = tokio::time::Instant::now() + DAEMON_STARTUP_TIMEOUT;
        self.api_addr = loop {
            if let Some(status) = self
                .process
                .try_wait()
                .context("poll x0xd startup status")?
            {
                bail!(
                    "x0xd exited before writing api.port with status {status}\nstdout:\n{}\nstderr:\n{}",
                    read_log_excerpt(&self.stdout_log),
                    read_log_excerpt(&self.stderr_log)
                );
            }
            if tokio::time::Instant::now() > deadline {
                bail!(
                    "timeout waiting for x0xd api.port at {}\nstdout:\n{}\nstderr:\n{}",
                    port_file.display(),
                    read_log_excerpt(&self.stdout_log),
                    read_log_excerpt(&self.stderr_log)
                );
            }
            if let Ok(addr) = fs::read_to_string(&port_file) {
                let trimmed = addr.trim();
                if let Ok(addr) = trimmed.parse::<std::net::SocketAddr>() {
                    break addr.to_string();
                }
                if let Ok(port) = trimmed.parse::<u16>() {
                    break format!("127.0.0.1:{port}");
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        };

        let client = X0xClient::with_base_url(&self.url(""));
        let deadline = tokio::time::Instant::now() + DAEMON_STARTUP_TIMEOUT;
        loop {
            if tokio::time::Instant::now() > deadline {
                bail!(
                    "timeout waiting for x0xd health at {}\nstdout:\n{}\nstderr:\n{}",
                    self.url("/health"),
                    read_log_excerpt(&self.stdout_log),
                    read_log_excerpt(&self.stderr_log)
                );
            }
            if client.health().await.is_ok() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let token_file = self.token_file();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        self.api_token = loop {
            if let Ok(token) = fs::read_to_string(&token_file) {
                let token = token.trim().to_owned();
                if !token.is_empty() {
                    break token;
                }
            }
            if tokio::time::Instant::now() > deadline {
                bail!(
                    "timeout waiting for x0xd api-token at {}",
                    token_file.display()
                );
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        };

        Ok(())
    }

    /// Full HTTP URL for `path`.
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.api_addr, path)
    }

    /// Raw API token.
    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    fn port_file(&self) -> PathBuf {
        self.tempdir.path().join("api.port")
    }

    fn token_file(&self) -> PathBuf {
        self.tempdir.path().join("api-token")
    }
}

impl Drop for DaemonFixture {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
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

fn find_x0xd_binary() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(path) = std::env::var("X0XD_BIN") {
        candidates.push(PathBuf::from(path));
    }
    if let Ok(dir) = std::env::var("X0X_DIR") {
        let dir = PathBuf::from(dir);
        candidates.push(dir.join("target/release/x0xd"));
        candidates.push(dir.join("target/debug/x0xd"));
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .context("resolve Communitas workspace from E2E manifest dir")?;
    candidates.push(workspace_dir.join("../x0x/target/release/x0xd"));
    candidates.push(workspace_dir.join("../x0x/target/debug/x0xd"));
    candidates.push(std::env::current_dir()?.join("target/release/x0xd"));
    candidates.push(std::env::current_dir()?.join("target/debug/x0xd"));

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    bail!("could not locate x0xd; build x0x or set X0XD_BIN")
}

fn read_log_excerpt(path: &Path) -> String {
    let Ok(contents) = fs::read_to_string(path) else {
        return "(unreadable)".to_owned();
    };
    let mut lines: Vec<&str> = contents.lines().rev().take(40).collect();
    lines.reverse();
    lines.join("\n")
}
