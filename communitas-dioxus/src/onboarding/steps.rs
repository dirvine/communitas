//! Tour step content components for the onboarding experience.
//!
//! Provides individual step components with consistent structure and
//! a dispatcher component that renders the appropriate step by ID.

use dioxus::prelude::*;

/// Welcome step - Introduction to Communitas.
#[component]
pub fn WelcomeStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "🏠" }
            h4 { class: "tour-step-title", "Welcome to Communitas" }
            p { class: "tour-step-content",
                "A local-first collaboration platform that puts you in control. "
                "Connect with others using simple four-word identities like "
                "\"ocean-forest-moon-star\" - no servers, no tracking, just peer-to-peer communication."
            }
        }
    }
}

/// Messaging step - Overview of messaging features.
#[component]
pub fn MessagingStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "💬" }
            h4 { class: "tour-step-title", "Secure Messaging" }
            p { class: "tour-step-content",
                "Organize conversations in threads and channels. Every message is "
                "end-to-end encrypted by default. React to messages and create "
                "threaded replies to keep discussions organized."
            }
        }
    }
}

/// Drive step - File storage overview.
#[component]
pub fn DriveStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "📁" }
            h4 { class: "tour-step-title", "Virtual Drive" }
            p { class: "tour-step-content",
                "Each entity has a virtual disk with Private, Public, and Shared folders. "
                "Share files securely using connection words - no cloud storage required, "
                "just direct peer-to-peer transfer."
            }
        }
    }
}

/// Canvas step - Collaboration overview.
#[component]
pub fn CanvasStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "🎨" }
            h4 { class: "tour-step-title", "Collaborative Canvas" }
            p { class: "tour-step-content",
                "Real-time whiteboard for visual collaboration. Changes sync instantly "
                "using CRDT technology. Draw, add text, shapes, and sticky notes with "
                "your team in real-time."
            }
        }
    }
}

/// Kanban step - Task management overview.
#[component]
pub fn KanbanStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "📋" }
            h4 { class: "tour-step-title", "Task Management" }
            p { class: "tour-step-content",
                "Organize work with Kanban boards, columns, and cards. Use swimlanes "
                "to categorize tasks, set priorities, and track due dates. Perfect for "
                "agile teams and personal projects."
            }
        }
    }
}

/// Calls step - Communication overview.
#[component]
pub fn CallsStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "📞" }
            h4 { class: "tour-step-title", "Voice & Video Calls" }
            p { class: "tour-step-content",
                "Connect face-to-face with encrypted voice and video calls. Share your "
                "screen for presentations and collaboration. All communication stays "
                "peer-to-peer and private."
            }
        }
    }
}

/// Settings step - Customization overview.
#[component]
pub fn SettingsStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "⚙️" }
            h4 { class: "tour-step-title", "Customize Your Experience" }
            p { class: "tour-step-content",
                "Personalize Communitas with theme preferences, notification settings, "
                "and privacy controls. Manage your identity and connection settings "
                "from one convenient location."
            }
        }
    }
}

/// Help step - Getting help overview.
#[component]
pub fn HelpStep() -> Element {
    rsx! {
        div { class: "tour-step",
            div { class: "tour-step-icon", "❓" }
            h4 { class: "tour-step-title", "Need Help?" }
            p { class: "tour-step-content",
                "Access documentation and community resources anytime. Visit our "
                a { href: "#", "guides" }
                " for tutorials, or "
                a { href: "#", "join the community" }
                " for support. You can restart this tour from Settings anytime."
            }
        }
    }
}

/// Main component that renders the appropriate step based on step_id.
#[component]
pub fn StepContent(step_id: String) -> Element {
    match step_id.as_str() {
        "welcome" => rsx! { WelcomeStep {} },
        "messaging" => rsx! { MessagingStep {} },
        "drive" => rsx! { DriveStep {} },
        "canvas" => rsx! { CanvasStep {} },
        "kanban" => rsx! { KanbanStep {} },
        "calls" => rsx! { CallsStep {} },
        "settings" => rsx! { SettingsStep {} },
        "help" => rsx! { HelpStep {} },
        _ => rsx! { WelcomeStep {} }, // Default to welcome if unknown step
    }
}
