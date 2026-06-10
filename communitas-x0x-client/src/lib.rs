// SPDX-License-Identifier: MIT OR Apache-2.0

//! Thin typed mirror of the x0xd REST, WebSocket, and SSE APIs.
//!
//! This crate provides [`X0xClient`] for HTTP access to daemon routes,
//! [`X0xWebSocket`] for `/ws` and `/ws/direct` sessions,
//! [`X0xSseStream`] for `/events`, `/direct/events`, and `/presence/events`,
//! and [`DaemonManager`] for CLI-faithful install/start/stop helpers.
//!
//! It intentionally stays transport-focused and does not define Communitas
//! app-level semantics such as channel schemas, topic conventions, or UI state.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use communitas_x0x_client::{X0xClient, DaemonManager};
//!
//! # async fn example() -> communitas_x0x_client::Result<()> {
//! // Ensure the daemon is running (installs if needed).
//! let dm = DaemonManager::new();
//! dm.ensure_running().await?;
//!
//! // Auto-discover address + token from x0xd config files and connect.
//! let client = X0xClient::new();
//! let identity = client.agent().await?;
//! println!("I am agent {}", identity.agent_id);
//!
//! // Publish a message.
//! client.publish("my-topic", b"hello world").await?;
//! # Ok(())
//! # }
//! ```

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod client;
pub mod config;
pub mod daemon;
pub mod error;
pub mod sse;
pub mod types;
pub mod websocket;

pub use client::X0xClient;
pub use config::{X0xConfig, discover as discover_x0x_config};
pub use daemon::{DaemonManager, DaemonState, is_running_health_status};
pub use error::{Result, X0xError};
pub use sse::{SseFrame, X0xSseStream};
pub use types::*;
pub use websocket::X0xWebSocket;
