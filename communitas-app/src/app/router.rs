// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Router Configuration
//!
//! Defines all application routes and their corresponding screens.

use crate::screens::{
    auth::{CreateIdentityScreen, LoginScreen, WelcomeScreen},
    main::{ChatScreen, ContentScreen, CreateEntityScreen},
    settings::SettingsScreen,
};
use dioxus::prelude::*;

/// Application routes
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
#[allow(clippy::enum_variant_names)]
pub enum Route {
    // Auth routes
    #[route("/")]
    WelcomeScreen {},

    #[route("/login")]
    LoginScreen {},

    #[route("/create-identity")]
    CreateIdentityScreen {},

    // Main routes
    #[route("/content")]
    ContentScreen {},

    #[route("/chat/:entity_id")]
    ChatScreen { entity_id: String },

    #[route("/create/:entity_type")]
    CreateEntityScreen { entity_type: String },

    // Settings routes
    #[route("/settings")]
    SettingsScreen {},
}
