// SPDX-License-Identifier: MIT OR Apache-2.0

//! CLI-faithful lifecycle helpers for the local x0x daemon.
//!
//! This module wraps documented `x0x` CLI behavior for checking, installing,
//! starting, stopping, and optionally auto-starting the daemon. It does not
//! assume undocumented direct `x0xd` process management semantics.

use std::process::Command;
use std::time::Duration;

use crate::client::X0xClient;
use crate::error::{Result, X0xError};

/// Maximum time to wait for `x0x start` / `x0x stop` / `x0x autostart` (seconds).
const SUBPROCESS_TIMEOUT_SECS: u64 = 30;

/// Maximum time to wait for the installer (`curl -sfL https://x0x.md | sh`) (seconds).
const INSTALL_TIMEOUT_SECS: u64 = 120;

/// The current state of the x0xd daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonState {
    /// x0x binary is not installed on this machine.
    NotInstalled,
    /// x0x binary is installed but the daemon is not running.
    NotRunning,
    /// The daemon is running and healthy.
    Running,
    /// The daemon is running but reporting issues (degraded connectivity, etc.).
    Degraded,
}

/// Manages the x0xd daemon lifecycle.
///
/// Can check status, install, start, and configure autostart.
#[derive(Debug, Clone)]
pub struct DaemonManager {
    client: X0xClient,
}

impl DaemonManager {
    /// Create a daemon manager using the default client address.
    pub fn new() -> Self {
        Self {
            client: X0xClient::new(),
        }
    }

    /// Create a daemon manager using a custom client.
    pub fn with_client(client: X0xClient) -> Self {
        Self { client }
    }

    /// Check the current state of the daemon.
    pub async fn state(&self) -> DaemonState {
        // First check if we can reach the daemon.
        match self.client.health().await {
            Ok(health) => {
                if health.status == "healthy" || health.status == "running" {
                    DaemonState::Running
                } else {
                    DaemonState::Degraded
                }
            }
            Err(_) => {
                // Daemon not reachable. Check if binary is installed.
                if Self::is_installed() {
                    DaemonState::NotRunning
                } else {
                    DaemonState::NotInstalled
                }
            }
        }
    }

    /// Check if the `x0x` binary is installed.
    ///
    /// Checks `which x0x` first (PATH lookup), then falls back to well-known
    /// install locations matching the Swift DaemonManager behavior.
    pub fn is_installed() -> bool {
        // Fast path: check PATH
        if Command::new("which")
            .arg("x0x")
            .output()
            .is_ok_and(|o| o.status.success())
        {
            return true;
        }

        // Check well-known install locations
        let home = dirs_next::home_dir().unwrap_or_default();
        let candidates = [
            std::path::PathBuf::from("/usr/local/bin/x0x"),
            std::path::PathBuf::from("/opt/homebrew/bin/x0x"),
            std::path::PathBuf::from("/opt/zerobrew/bin/x0x"),
            home.join(".cargo/bin/x0x"),
            home.join(".x0x/bin/x0x"),
        ];

        candidates.iter().any(|p| p.exists())
    }

    /// Install x0x using the official installer script.
    ///
    /// Runs: `curl -sfL https://x0x.md | sh`
    ///
    /// Times out after [`INSTALL_TIMEOUT_SECS`] seconds.
    pub async fn install() -> Result<()> {
        tracing::info!("installing x0x via curl -sfL https://x0x.md | sh");
        let output = run_with_timeout(
            tokio::process::Command::new("sh")
                .arg("-c")
                .arg("curl -sfL https://x0x.md | sh"),
            Duration::from_secs(INSTALL_TIMEOUT_SECS),
            "install",
        )
        .await?;

        if output.status.success() {
            tracing::info!("x0x installed successfully");
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(X0xError::Daemon(format!("install failed: {stderr}")))
        }
    }

    /// Configure x0x to start automatically on boot.
    ///
    /// Runs: `x0x autostart`
    ///
    /// Times out after [`SUBPROCESS_TIMEOUT_SECS`] seconds.
    pub async fn autostart() -> Result<()> {
        tracing::info!("configuring x0x autostart");
        let output = run_with_timeout(
            tokio::process::Command::new("x0x").arg("autostart"),
            Duration::from_secs(SUBPROCESS_TIMEOUT_SECS),
            "autostart",
        )
        .await?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(X0xError::Daemon(format!("autostart failed: {stderr}")))
        }
    }

    /// Start the x0x daemon.
    ///
    /// Runs: `x0x start`
    ///
    /// Times out after [`SUBPROCESS_TIMEOUT_SECS`] seconds.
    pub async fn start() -> Result<()> {
        tracing::info!("starting x0x daemon");
        let output = run_with_timeout(
            tokio::process::Command::new("x0x").arg("start"),
            Duration::from_secs(SUBPROCESS_TIMEOUT_SECS),
            "start",
        )
        .await?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(X0xError::Daemon(format!("start failed: {stderr}")))
        }
    }

    /// Stop the x0x daemon.
    ///
    /// Runs: `x0x stop`
    ///
    /// Times out after [`SUBPROCESS_TIMEOUT_SECS`] seconds.
    pub async fn stop() -> Result<()> {
        tracing::info!("stopping x0x daemon");
        let output = run_with_timeout(
            tokio::process::Command::new("x0x").arg("stop"),
            Duration::from_secs(SUBPROCESS_TIMEOUT_SECS),
            "stop",
        )
        .await?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(X0xError::Daemon(format!("stop failed: {stderr}")))
        }
    }

    async fn wait_until_healthy(&self, timeout_secs: u64) -> Result<()> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
        loop {
            match self.client.health().await {
                Ok(_) => return Ok(()),
                Err(e) if std::time::Instant::now() >= deadline => {
                    return Err(X0xError::Daemon(format!(
                        "daemon did not become healthy before timeout: {e}"
                    )));
                }
                Err(_) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
                }
            }
        }
    }

    /// Ensure the daemon is installed and running.
    ///
    /// This helper mirrors documented x0x CLI behavior but does not implicitly
    /// enable autostart; callers that want autostart should opt into
    /// [`Self::autostart`] separately.
    pub async fn ensure_running(&self) -> Result<()> {
        match self.state().await {
            DaemonState::Running => {
                tracing::debug!("x0x daemon already running");
                Ok(())
            }
            DaemonState::Degraded => {
                tracing::warn!("x0x daemon is degraded but reachable");
                Ok(())
            }
            DaemonState::NotRunning => {
                Self::start().await?;
                self.wait_until_healthy(10).await
            }
            DaemonState::NotInstalled => {
                Self::install().await?;
                Self::start().await?;
                self.wait_until_healthy(15).await
            }
        }
    }
}

impl Default for DaemonManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a subprocess with a timeout. Kills the child if the deadline elapses.
async fn run_with_timeout(
    cmd: &mut tokio::process::Command,
    timeout: Duration,
    label: &str,
) -> Result<std::process::Output> {
    let child = cmd.kill_on_drop(true).spawn()?;

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(result) => Ok(result?),
        Err(_) => {
            // Timeout expired — the child is killed by `kill_on_drop(true)`.
            tracing::warn!("x0x {label} timed out after {}s", timeout.as_secs());
            Err(X0xError::Daemon(format!(
                "{label} timed out after {}s",
                timeout.as_secs()
            )))
        }
    }
}
