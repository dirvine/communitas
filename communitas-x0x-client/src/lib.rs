//! Thin typed mirror of the x0xd REST and WebSocket APIs.
//!
//! This crate provides [`X0xClient`] for HTTP access to daemon routes,
//! [`X0xWebSocket`] for `/ws` and `/ws/direct` sessions, and
//! [`DaemonManager`] for CLI-faithful install/start/stop helpers.
//!
//! Scope freeze: this crate intentionally covers **REST + WebSocket only**.
//! Server-sent-event surfaces such as `/events` and `/direct/events` are part
//! of x0x, but are intentionally out of scope for this crate.
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

pub mod client;
pub mod config;
pub mod daemon;
pub mod error;
pub mod types;
pub mod websocket;

pub use client::X0xClient;
pub use config::{X0xConfig, discover as discover_x0x_config};
pub use daemon::{DaemonManager, DaemonState};
pub use error::{Result, X0xError};
pub use types::*;
pub use websocket::X0xWebSocket;
