/// HTTP Control API for TUI automation and testing
///
/// This module provides a REST API for controlling the TUI application
/// via HTTP requests. This enables MCP-driven testing and automation.
///
/// # Example Usage
///
/// ```bash
/// # Start TUI with control API
/// communitas-tui --control-port 3040
///
/// # Check health
/// curl http://localhost:3040/health
///
/// # Get current identity
/// curl http://localhost:3040/api/identity/current
///
/// # Send message
/// curl -X POST http://localhost:3040/api/messages/send \
///   -H "Content-Type: application/json" \
///   -d '{"entity_id": "...", "entity_type": "Channel", "text": "Hello!"}'
/// ```
pub mod handlers;
pub mod routes;
pub mod server;
pub mod types;

pub use server::ControlServer;
