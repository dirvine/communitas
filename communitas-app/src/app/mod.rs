// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Application Root Module
//!
//! Contains the root App component, router configuration, and theme system.

mod router;
mod theme;

use crate::services::CoreService;
use crate::state::AppState;
use dioxus::prelude::*;

pub use router::Route;

/// Root application component
///
/// Sets up the application context, initializes services,
/// and renders the router.
#[component]
pub fn App() -> Element {
    // Initialize CoreService
    let core_service = use_signal(CoreService::new);

    // Initialize AppState
    let app_state = AppState {
        core: core_service,
        four_words: use_signal(|| None),
        display_name: use_signal(|| None),
        is_authenticated: use_signal(|| false),
        is_dark_mode: use_signal(|| false),
    };

    // Provide AppState to all children
    use_context_provider(|| app_state.clone());

    rsx! {
        div {
            class: "communitas-app",
            style: "font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif; height: 100vh;",

            Router::<Route> {}
        }
    }
}
