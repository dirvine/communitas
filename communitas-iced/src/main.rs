// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Communitas Iced GUI - Entry Point
//!
//! Cross-platform desktop application for Communitas using Iced.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use communitas_iced::app::CommunitasApp;
use iced::{Size, window};

/// Application entry point.
fn main() -> iced::Result {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("communitas_iced=info".parse().unwrap_or_default()),
        )
        .init();

    tracing::info!("Starting Communitas Iced GUI");

    // Configure window settings explicitly to ensure visibility
    let window_settings = window::Settings {
        size: Size::new(1200.0, 800.0),
        position: window::Position::Centered,
        visible: true,
        resizable: true,
        decorations: true,
        ..window::Settings::default()
    };

    iced::application(
        CommunitasApp::new,
        CommunitasApp::update,
        CommunitasApp::view,
    )
    .title(CommunitasApp::title)
    .subscription(CommunitasApp::subscription)
    .theme(CommunitasApp::theme)
    .antialiasing(true)
    .window(window_settings)
    .run()
}
