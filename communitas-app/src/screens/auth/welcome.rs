// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Welcome Screen
//!
//! Initial landing page with options to login or create identity.

use crate::app::Route;
use dioxus::prelude::*;

/// Welcome screen component
#[component]
pub fn WelcomeScreen() -> Element {
    let navigator = use_navigator();

    rsx! {
        div {
            class: "welcome-screen",
            style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100vh; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white;",

            // Logo placeholder
            div {
                style: "width: 120px; height: 120px; background: rgba(255,255,255,0.2); border-radius: 24px; display: flex; align-items: center; justify-content: center; margin-bottom: 32px;",
                span { style: "font-size: 48px;", "C" }
            }

            h1 {
                style: "font-size: 36px; margin-bottom: 8px; font-weight: 600;",
                "Communitas"
            }

            p {
                style: "font-size: 16px; opacity: 0.9; margin-bottom: 48px;",
                "Decentralized Collaboration"
            }

            // Create Identity button
            button {
                style: "width: 280px; padding: 16px 32px; font-size: 16px; font-weight: 600; background: white; color: #667eea; border: none; border-radius: 12px; cursor: pointer; margin-bottom: 16px;",
                onclick: move |_| { navigator.push(Route::CreateIdentityScreen {}); },
                "Create Identity"
            }

            // Login button
            button {
                style: "width: 280px; padding: 16px 32px; font-size: 16px; font-weight: 600; background: transparent; color: white; border: 2px solid white; border-radius: 12px; cursor: pointer;",
                onclick: move |_| { navigator.push(Route::LoginScreen {}); },
                "I Have an Identity"
            }

            // Version info
            p {
                style: "position: absolute; bottom: 24px; font-size: 12px; opacity: 0.6;",
                "Version 0.1.0 - Dioxus Edition"
            }
        }
    }
}
