// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Login Screen
//!
//! Four-word identity login with vault selection.

use crate::app::Route;
use crate::services::CoreService;
use crate::state::use_app_state;
use dioxus::prelude::*;

/// Login screen component
#[component]
pub fn LoginScreen() -> Element {
    let navigator = use_navigator();
    let mut app_state = use_app_state();

    let mut four_words = use_signal(String::new);
    let mut error_message = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    let handle_login = move |_| {
        let words = four_words.read().clone();
        let nav = navigator;

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            // Validate four-word format using the dictionary
            if !CoreService::validate_four_words(&words) {
                let word_count = words.split('-').count();
                let msg = if word_count != 4 {
                    format!(
                        "Please enter exactly 4 words separated by dashes (got {})",
                        word_count
                    )
                } else {
                    "One or more words are not in the dictionary".to_string()
                };
                error_message.set(Some(msg));
                is_loading.set(false);
                return;
            }

            // Try to initialize with the four-word identity
            let core = app_state.core.read().clone();
            match core
                .initialize(
                    words.clone(),
                    "User".to_string(), // Will be updated from profile
                    "Desktop".to_string(),
                )
                .await
            {
                Ok(()) => {
                    app_state.four_words.set(Some(words));
                    app_state.is_authenticated.set(true);
                    nav.push(Route::ContentScreen {});
                }
                Err(e) => {
                    error_message.set(Some(format!("Login failed: {}", e)));
                }
            }

            is_loading.set(false);
        });
    };

    rsx! {
        div {
            class: "login-screen",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #f5f5f7; padding: 24px;",

            // Back button
            button {
                style: "position: absolute; top: 24px; left: 24px; background: none; border: none; font-size: 16px; color: #007AFF; cursor: pointer;",
                onclick: move |_| { navigator.push(Route::WelcomeScreen {}); },
                "← Back"
            }

            h1 {
                style: "font-size: 28px; margin-bottom: 8px; color: #1d1d1f;",
                "Welcome Back"
            }

            p {
                style: "font-size: 16px; color: #86868b; margin-bottom: 32px;",
                "Enter your four-word identity"
            }

            // Four-word input
            input {
                style: "width: 320px; padding: 16px; font-size: 18px; border: 2px solid #e5e5ea; border-radius: 12px; text-align: center; margin-bottom: 16px;",
                r#type: "text",
                placeholder: "ocean-forest-moon-star",
                value: "{four_words}",
                oninput: move |evt| four_words.set(evt.value().to_lowercase().replace(' ', "-")),
            }

            // Error message
            if let Some(error) = error_message.read().as_ref() {
                p {
                    style: "color: #FF3B30; font-size: 14px; margin-bottom: 16px;",
                    "{error}"
                }
            }

            // Login button
            button {
                style: "width: 320px; padding: 16px 32px; font-size: 16px; font-weight: 600; background: #007AFF; color: white; border: none; border-radius: 12px; cursor: pointer;",
                disabled: *is_loading.read(),
                onclick: handle_login,
                if *is_loading.read() { "Logging in..." } else { "Login" }
            }
        }
    }
}
