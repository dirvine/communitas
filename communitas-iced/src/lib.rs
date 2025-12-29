// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Communitas Iced GUI - Cross-platform desktop application.
//!
//! This crate provides a native, cross-platform GUI for Communitas using
//! the Iced framework (Elm-inspired MVU architecture).
//!
//! # Features
//!
//! - Full feature parity with the Swift macOS app
//! - Cross-platform: macOS, Linux, Windows
//! - Static binary distribution
//! - Direct Rust integration with `communitas-core`
//! - Post-quantum cryptography ready
//!
//! # Architecture
//!
//! The application follows the Model-View-Update (MVU) pattern:
//!
//! - **Model**: `CommunitasApp` holds all application state
//! - **View**: Views render the UI based on current state
//! - **Update**: Messages trigger state transitions
//!
//! ```text
//! User Action -> Message -> update() -> State Change -> view() -> UI
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

pub mod app;
pub mod error;
pub mod message;
pub mod state;
pub mod theme;
pub mod views;
pub mod webrtc;

pub use app::{AppState, CommunitasApp, PaneType};
pub use error::AppError;
pub use message::Message;
pub use state::*;
