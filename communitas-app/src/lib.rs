// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Communitas App Library
//!
//! Re-exports all public types and components for the Dioxus application.

// Security: Enforce no-panic policy in production code
// Note: Using deny instead of forbid because Dioxus Props macro uses panic internally
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod app;
pub mod hooks;
pub mod platform;
pub mod screens;
pub mod services;
pub mod state;

// Re-export commonly used types
pub use app::App;
pub use services::CoreService;
pub use state::AppState;
