// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Settings Screen
//!
//! Application settings and preferences management.

use crate::app::Route;
use crate::state::use_app_state;
use dioxus::prelude::*;

/// Settings screen component
#[component]
pub fn SettingsScreen() -> Element {
    let navigator = use_navigator();
    let mut app_state = use_app_state();

    // Check authentication
    if !*app_state.is_authenticated.read() {
        let _ = navigator.push(Route::WelcomeScreen {});
        return rsx! { div { "Redirecting..." } };
    }

    let four_words = app_state.four_words.read().clone().unwrap_or_default();
    let display_name = app_state
        .display_name
        .read()
        .clone()
        .unwrap_or_else(|| "User".to_string());

    let handle_logout = move |_| {
        app_state.is_authenticated.set(false);
        app_state.four_words.set(None);
        app_state.display_name.set(None);
        navigator.push(Route::WelcomeScreen {});
    };

    let toggle_dark_mode = move |_| {
        let current = *app_state.is_dark_mode.read();
        app_state.is_dark_mode.set(!current);
    };

    rsx! {
        div {
            class: "settings-screen",
            style: "min-height: 100vh; background: #f5f5f7;",

            // Header
            div {
                style: "background: white; padding: 16px 24px; border-bottom: 1px solid #e5e5ea; display: flex; align-items: center; gap: 16px;",

                button {
                    style: "background: none; border: none; color: #007AFF; cursor: pointer; font-size: 14px;",
                    onclick: move |_| { navigator.push(Route::ContentScreen {}); },
                    "← Back"
                }

                h1 {
                    style: "font-size: 20px; color: #1d1d1f; margin: 0;",
                    "Settings"
                }
            }

            // Content
            div {
                style: "max-width: 600px; margin: 0 auto; padding: 24px;",

                // Profile section
                section {
                    style: "background: white; border-radius: 12px; padding: 20px; margin-bottom: 16px;",

                    h2 {
                        style: "font-size: 16px; color: #1d1d1f; margin-bottom: 16px;",
                        "Profile"
                    }

                    div {
                        style: "display: flex; align-items: center; gap: 16px; margin-bottom: 16px;",

                        // Avatar placeholder
                        div {
                            style: "width: 64px; height: 64px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border-radius: 16px; display: flex; align-items: center; justify-content: center;",
                            span { style: "font-size: 24px; color: white; font-weight: 600;", "{display_name.chars().next().unwrap_or('U')}" }
                        }

                        div {
                            p {
                                style: "font-size: 18px; font-weight: 600; color: #1d1d1f; margin-bottom: 4px;",
                                "{display_name}"
                            }
                            p {
                                style: "font-size: 14px; color: #86868b;",
                                "{four_words}"
                            }
                        }
                    }

                    // Copy identity button
                    button {
                        style: "width: 100%; padding: 12px; background: #f5f5f7; border: none; border-radius: 8px; color: #007AFF; cursor: pointer; font-size: 14px;",
                        onclick: move |_| {
                            // TODO: Copy to clipboard
                        },
                        "Copy Four-Word Identity"
                    }
                }

                // Appearance section
                section {
                    style: "background: white; border-radius: 12px; padding: 20px; margin-bottom: 16px;",

                    h2 {
                        style: "font-size: 16px; color: #1d1d1f; margin-bottom: 16px;",
                        "Appearance"
                    }

                    // Dark mode toggle
                    div {
                        style: "display: flex; justify-content: space-between; align-items: center; padding: 12px 0;",

                        div {
                            p {
                                style: "font-size: 14px; color: #1d1d1f;",
                                "Dark Mode"
                            }
                            p {
                                style: "font-size: 12px; color: #86868b;",
                                "Use dark theme throughout the app"
                            }
                        }

                        button {
                            style: "width: 50px; height: 28px; border-radius: 14px; border: none; cursor: pointer; transition: background 0.2s;",
                            style: if *app_state.is_dark_mode.read() { "background: #007AFF;" } else { "background: #e5e5ea;" },
                            onclick: toggle_dark_mode,

                            div {
                                style: "width: 24px; height: 24px; background: white; border-radius: 12px; margin: 2px; transition: transform 0.2s;",
                                style: if *app_state.is_dark_mode.read() { "transform: translateX(22px);" } else { "transform: translateX(0);" },
                            }
                        }
                    }
                }

                // Network section
                section {
                    style: "background: white; border-radius: 12px; padding: 20px; margin-bottom: 16px;",

                    h2 {
                        style: "font-size: 16px; color: #1d1d1f; margin-bottom: 16px;",
                        "Network"
                    }

                    div {
                        style: "padding: 12px 0; border-bottom: 1px solid #f0f0f0;",

                        p {
                            style: "font-size: 14px; color: #1d1d1f; margin-bottom: 4px;",
                            "Connection Status"
                        }
                        p {
                            style: "font-size: 12px; color: #34C759;",
                            "Connected"
                        }
                    }

                    div {
                        style: "padding: 12px 0;",

                        p {
                            style: "font-size: 14px; color: #1d1d1f; margin-bottom: 4px;",
                            "Peers"
                        }
                        p {
                            style: "font-size: 12px; color: #86868b;",
                            "0 connected"
                        }
                    }
                }

                // Security section
                section {
                    style: "background: white; border-radius: 12px; padding: 20px; margin-bottom: 16px;",

                    h2 {
                        style: "font-size: 16px; color: #1d1d1f; margin-bottom: 16px;",
                        "Security"
                    }

                    div {
                        style: "padding: 12px 0;",

                        p {
                            style: "font-size: 14px; color: #1d1d1f; margin-bottom: 4px;",
                            "Cryptography"
                        }
                        p {
                            style: "font-size: 12px; color: #86868b;",
                            "Post-Quantum (ML-DSA-87)"
                        }
                    }
                }

                // About section
                section {
                    style: "background: white; border-radius: 12px; padding: 20px; margin-bottom: 16px;",

                    h2 {
                        style: "font-size: 16px; color: #1d1d1f; margin-bottom: 16px;",
                        "About"
                    }

                    div {
                        style: "padding: 12px 0; border-bottom: 1px solid #f0f0f0;",

                        p {
                            style: "font-size: 14px; color: #1d1d1f; margin-bottom: 4px;",
                            "Version"
                        }
                        p {
                            style: "font-size: 12px; color: #86868b;",
                            "0.1.0 (Dioxus Edition)"
                        }
                    }

                    div {
                        style: "padding: 12px 0;",

                        p {
                            style: "font-size: 14px; color: #1d1d1f; margin-bottom: 4px;",
                            "License"
                        }
                        p {
                            style: "font-size: 12px; color: #86868b;",
                            "AGPL-3.0-or-later OR Commercial"
                        }
                    }
                }

                // Logout button
                button {
                    style: "width: 100%; padding: 16px; background: #FF3B30; color: white; border: none; border-radius: 12px; font-size: 16px; font-weight: 600; cursor: pointer;",
                    onclick: handle_logout,
                    "Sign Out"
                }
            }
        }
    }
}
