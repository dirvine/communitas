//! Typed Rust client for the x0xd daemon REST API and WebSocket.
//!
//! This crate provides [`X0xClient`] for HTTP-based access to the full
//! x0xd REST API, [`X0xWebSocket`] for real-time bidirectional messaging,
//! and [`DaemonManager`] for installing, starting, and monitoring the daemon.
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
//! // Use the REST API.
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
pub mod daemon;
pub mod error;
pub mod types;
pub mod websocket;

pub use client::X0xClient;
pub use daemon::{DaemonManager, DaemonState};
pub use error::{Result, X0xError};
pub use types::*;
pub use websocket::X0xWebSocket;
