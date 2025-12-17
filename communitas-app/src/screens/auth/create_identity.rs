// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Create Identity Screen
//!
//! New identity creation with four-word generation.

use crate::app::Route;
use crate::services::CoreService;
use crate::state::use_app_state;
use dioxus::prelude::*;

/// Create Identity screen component
#[component]
pub fn CreateIdentityScreen() -> Element {
    let navigator = use_navigator();
    let mut app_state = use_app_state();

    let mut display_name = use_signal(String::new);
    let mut generated_words = use_signal(|| None::<String>);
    let mut error_message = use_signal(|| None::<String>);
    let mut is_loading = use_signal(|| false);

    // Generate random four words on mount
    use_effect(move || {
        if generated_words.read().is_none() {
            match CoreService::generate_four_words() {
                Ok(words) => generated_words.set(Some(words)),
                Err(e) => error_message.set(Some(format!("Failed to generate identity: {}", e))),
            }
        }
    });

    let handle_create = move |_| {
        let name = display_name.read().clone();
        let words = generated_words.read().clone();
        let nav = navigator;

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            // Validate inputs
            if name.trim().is_empty() {
                error_message.set(Some("Please enter a display name".to_string()));
                is_loading.set(false);
                return;
            }

            let Some(four_words) = words else {
                error_message.set(Some("Identity not generated yet".to_string()));
                is_loading.set(false);
                return;
            };

            // Initialize the core service
            let core = app_state.core.read().clone();
            match core
                .initialize(four_words.clone(), name.clone(), "Desktop".to_string())
                .await
            {
                Ok(()) => {
                    app_state.four_words.set(Some(four_words));
                    app_state.display_name.set(Some(name));
                    app_state.is_authenticated.set(true);
                    nav.push(Route::ContentScreen {});
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to create identity: {}", e)));
                }
            }

            is_loading.set(false);
        });
    };

    let regenerate = move |_| match CoreService::generate_four_words() {
        Ok(words) => {
            generated_words.set(Some(words));
            error_message.set(None);
        }
        Err(e) => error_message.set(Some(format!("Failed to generate identity: {}", e))),
    };

    rsx! {
        div {
            class: "create-identity-screen",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: #f5f5f7; padding: 24px;",

            // Back button
            button {
                style: "position: absolute; top: 24px; left: 24px; background: none; border: none; font-size: 16px; color: #007AFF; cursor: pointer;",
                onclick: move |_| { navigator.push(Route::WelcomeScreen {}); },
                "← Back"
            }

            h1 {
                style: "font-size: 28px; margin-bottom: 8px; color: #1d1d1f;",
                "Create Your Identity"
            }

            p {
                style: "font-size: 16px; color: #86868b; margin-bottom: 32px; text-align: center;",
                "Your unique four-word identity is your address on the network"
            }

            // Generated four-word identity
            div {
                style: "background: white; padding: 24px 32px; border-radius: 16px; margin-bottom: 24px; box-shadow: 0 2px 8px rgba(0,0,0,0.1);",

                if let Some(words) = generated_words.read().as_ref() {
                    p {
                        style: "font-size: 24px; font-weight: 600; color: #1d1d1f; letter-spacing: 0.5px;",
                        "{words}"
                    }
                } else {
                    p {
                        style: "font-size: 18px; color: #86868b;",
                        "Generating..."
                    }
                }
            }

            // Regenerate button
            button {
                style: "background: none; border: none; color: #007AFF; font-size: 14px; cursor: pointer; margin-bottom: 32px;",
                onclick: regenerate,
                "🔄 Generate New Identity"
            }

            // Display name input
            div {
                style: "width: 320px; margin-bottom: 24px;",

                label {
                    style: "display: block; font-size: 14px; color: #86868b; margin-bottom: 8px;",
                    "Display Name"
                }

                input {
                    style: "width: 100%; padding: 16px; font-size: 16px; border: 2px solid #e5e5ea; border-radius: 12px;",
                    r#type: "text",
                    placeholder: "Your name",
                    value: "{display_name}",
                    oninput: move |evt| display_name.set(evt.value()),
                }
            }

            // Error message
            if let Some(error) = error_message.read().as_ref() {
                p {
                    style: "color: #FF3B30; font-size: 14px; margin-bottom: 16px;",
                    "{error}"
                }
            }

            // Create button
            button {
                style: "width: 320px; padding: 16px 32px; font-size: 16px; font-weight: 600; background: #007AFF; color: white; border: none; border-radius: 12px; cursor: pointer;",
                disabled: *is_loading.read() || generated_words.read().is_none(),
                onclick: handle_create,
                if *is_loading.read() { "Creating..." } else { "Create Identity" }
            }

            // Security notice
            p {
                style: "max-width: 320px; text-align: center; font-size: 12px; color: #86868b; margin-top: 24px;",
                "Your identity is protected by post-quantum cryptography (ML-DSA-87). Save your four words securely - they cannot be recovered if lost."
            }
        }
    }
}
