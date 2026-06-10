// SPDX-License-Identifier: MIT OR Apache-2.0

//! Benchmark crate for Communitas.
//!
//! This library exists to expose dependencies to benchmark targets.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

// Re-export dependencies for benchmarks
// Note: communitas-core has lib name = "communitas_bindings"
pub use communitas_bindings as communitas_core;
pub use communitas_ui_api;
pub use communitas_ui_service;
pub use tempfile;
pub use tokio;
