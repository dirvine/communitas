//! Identity model diagram component.
//!
//! Visual diagram explaining the WHO/WHERE/SHOWN identity model:
//! - WHO (Identity): pubkey_hex - cryptographic identity
//! - WHERE (Connection): connection words - network location
//! - SHOWN (Display): display_name - user-friendly label

use dioxus::prelude::*;

/// Visual diagram explaining the Communitas identity model.
#[component]
pub fn IdentityModelDiagram() -> Element {
    rsx! {
        div { class: "identity-model-diagram",
            // WHO Row
            div { class: "diagram-row",
                div { class: "diagram-label", "WHO" }
                div { class: "diagram-box who-box",
                    div { class: "diagram-title", "Identity" }
                    div { class: "diagram-content",
                        span { class: "diagram-key", "pubkey_hex" }
                        span { class: "diagram-desc", "(ML-DSA-65 public key)" }
                    }
                    div { class: "diagram-example", "a1b2c3d4...9f0e (3904 hex chars)" }
                }
            }
            // Arrow
            div { class: "diagram-arrow", "↓" }
            // WHERE Row
            div { class: "diagram-row",
                div { class: "diagram-label", "WHERE" }
                div { class: "diagram-box where-box",
                    div { class: "diagram-title", "Connection" }
                    div { class: "diagram-content",
                        span { class: "diagram-key", "connection_words" }
                        span { class: "diagram-desc", "(network address)" }
                    }
                    div { class: "diagram-example", "ocean-forest-moon-star" }
                }
            }
            // Arrow
            div { class: "diagram-arrow", "↓" }
            // SHOWN Row
            div { class: "diagram-row",
                div { class: "diagram-label", "SHOWN" }
                div { class: "diagram-box shown-box",
                    div { class: "diagram-title", "Display Name" }
                    div { class: "diagram-content",
                        span { class: "diagram-key", "display_name" }
                        span { class: "diagram-desc", "(user-chosen)" }
                    }
                    div { class: "diagram-example", "Alice" }
                }
            }
        }
    }
}

/// Simplified inline identity model reference for use in tooltips and help text.
#[component]
pub fn IdentityModelReference() -> Element {
    rsx! {
        span { class: "identity-model-ref",
            code { "pubkey" },
            " = identity (cryptographic) | ",
            code { "connection" },
            " = network location | ",
            code { "display" },
            " = shown name"
        }
    }
}
