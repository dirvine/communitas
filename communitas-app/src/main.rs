// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Communitas App - Cross-Platform Dioxus Application
//!
//! Entry point for the Communitas P2P collaboration platform.
//! Supports desktop (macOS, Linux, Windows) and mobile (iOS, Android).

// Security: Enforce no-panic policy in production code
// Note: Using deny instead of forbid because Dioxus Props macro uses panic internally
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
// Hide console window on Windows in release builds
#![cfg_attr(all(windows, feature = "bundle"), windows_subsystem = "windows")]

use tracing::info;

mod app;
mod hooks;
mod platform;
mod screens;
mod services;
mod state;

use app::App;

fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Communitas App");
    info!("Platform: {}", std::env::consts::OS);
    info!("Architecture: {}", std::env::consts::ARCH);

    // Launch the Dioxus app with desktop configuration
    dioxus::launch(App);
}
