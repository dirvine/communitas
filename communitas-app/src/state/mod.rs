// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Application State Management
//!
//! Provides reactive state using Dioxus signals for the application.

use crate::services::CoreService;
use dioxus::prelude::*;

/// Global application state
///
/// Uses Dioxus signals for reactive state management.
/// Access via `use_context::<AppState>()` in components.
#[derive(Clone)]
pub struct AppState {
    /// Core service wrapper for communitas-core
    pub core: Signal<CoreService>,

    /// Current user's four-word identity
    pub four_words: Signal<Option<String>>,

    /// Current user's display name
    pub display_name: Signal<Option<String>>,

    /// Whether the user is authenticated
    pub is_authenticated: Signal<bool>,

    /// Whether dark mode is enabled
    pub is_dark_mode: Signal<bool>,
}

impl AppState {
    /// Create a new app state (for testing)
    pub fn new() -> Self {
        Self {
            core: Signal::new(CoreService::new()),
            four_words: Signal::new(None),
            display_name: Signal::new(None),
            is_authenticated: Signal::new(false),
            is_dark_mode: Signal::new(false),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook to access global app state
pub fn use_app_state() -> AppState {
    use_context::<AppState>()
}
