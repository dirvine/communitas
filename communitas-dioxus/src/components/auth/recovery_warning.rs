//! Recovery backup warning components.
//!
//! Displays warnings for users who have enabled passkey authentication
//! but haven't set up a recovery backup method (mnemonic phrase).

use dioxus::prelude::*;

/// Props for the RecoveryWarningBanner component.
#[derive(Props, Clone, PartialEq)]
pub struct RecoveryWarningBannerProps {
    /// Four-word identity this warning applies to
    pub four_words: String,
    /// Display name of the identity
    pub display_name: String,
    /// Callback when user wants to set up recovery
    pub on_setup_recovery: EventHandler<()>,
    /// Callback when user dismisses the warning
    pub on_dismiss: EventHandler<()>,
}

/// Warning banner shown when passkey is enabled but no recovery is set up.
///
/// Non-intrusive banner that appears at the top of the app, reminding users
/// to set up a backup recovery method for their passkey-protected identity.
#[component]
pub fn RecoveryWarningBanner(props: RecoveryWarningBannerProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between gap-4 bg-amber-900/30 border border-amber-700/50 rounded-lg px-4 py-3",
            role: "alert",
            aria_live: "polite",
            // Warning icon and message
            div { class: "flex items-center gap-3",
                WarningIcon { class: "h-5 w-5 text-amber-400 flex-shrink-0" }
                div { class: "flex flex-col",
                    p { class: "text-sm font-medium text-amber-200",
                        "Recovery backup recommended"
                    }
                    p { class: "text-xs text-amber-300/80",
                        "Your identity \"{props.display_name}\" uses passkey auth. Without a recovery phrase, you may lose access if this device is lost."
                    }
                }
            }
            // Action buttons
            div { class: "flex items-center gap-2 flex-shrink-0",
                button {
                    class: "rounded-lg bg-amber-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-amber-500 transition",
                    onclick: move |_| props.on_setup_recovery.call(()),
                    "Set Up Recovery"
                }
                button {
                    class: "rounded-lg border border-amber-600/50 px-2 py-1.5 text-xs text-amber-300 hover:bg-amber-900/30 transition",
                    onclick: move |_| props.on_dismiss.call(()),
                    title: "Dismiss for now",
                    // X icon
                    svg {
                        class: "h-4 w-4",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        view_box: "0 0 24 24",
                        path {
                            d: "M6 18L18 6M6 6l12 12",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                        }
                    }
                }
            }
        }
    }
}

/// Props for the RecoveryWarningBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct RecoveryWarningBadgeProps {
    /// Optional extra CSS classes
    #[props(default = String::new())]
    pub class: String,
}

/// Small warning badge shown next to identity in switcher.
///
/// Indicates that this identity has passkey enabled but no recovery backup.
#[component]
pub fn RecoveryWarningBadge(props: RecoveryWarningBadgeProps) -> Element {
    rsx! {
        span {
            class: "inline-flex items-center rounded bg-amber-900/30 px-1.5 py-0.5 {props.class}",
            title: "No recovery backup - consider adding a recovery phrase",
            WarningIcon { class: "h-3 w-3 text-amber-400" }
        }
    }
}

/// Props for the RecoverySetupModal component.
#[derive(Props, Clone, PartialEq)]
pub struct RecoverySetupModalProps {
    /// Four-word identity this applies to
    pub four_words: String,
    /// Display name of the identity
    pub display_name: String,
    /// Callback when user completes or cancels setup
    pub on_close: EventHandler<()>,
    /// Callback when user wants to view recovery phrase
    pub on_view_phrase: EventHandler<()>,
}

/// Modal explaining why recovery backup is important and how to set it up.
#[component]
pub fn RecoverySetupModal(props: RecoverySetupModalProps) -> Element {
    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm",
            onclick: move |_| props.on_close.call(()),
            // Modal content
            div {
                class: "mx-4 w-full max-w-md rounded-2xl border border-slate-800 bg-slate-950 p-6 shadow-2xl",
                onclick: move |evt| evt.stop_propagation(),
                // Header with warning icon
                div { class: "flex items-center gap-3 mb-4",
                    div { class: "flex items-center justify-center w-12 h-12 rounded-full bg-amber-900/30",
                        WarningIcon { class: "h-6 w-6 text-amber-400" }
                    }
                    div {
                        h2 { class: "text-lg font-semibold text-slate-100",
                            "Protect Your Identity"
                        }
                        p { class: "text-xs text-slate-400",
                            "{props.display_name}"
                        }
                    }
                }
                // Explanation
                div { class: "mb-6 space-y-3",
                    p { class: "text-sm text-slate-300",
                        "Your identity uses passkey (biometric) authentication, which is tied to this device only."
                    }
                    div { class: "rounded-lg bg-slate-900 p-3 border border-slate-800",
                        h3 { class: "text-sm font-medium text-amber-300 mb-2",
                            "What happens if you lose this device?"
                        }
                        ul { class: "space-y-1 text-xs text-slate-400",
                            li { class: "flex items-start gap-2",
                                span { class: "text-red-400 mt-0.5", "\u{2022}" }
                                span { "Without a recovery phrase, your identity and all associated data will be permanently inaccessible" }
                            }
                            li { class: "flex items-start gap-2",
                                span { class: "text-red-400 mt-0.5", "\u{2022}" }
                                span { "No one, including us, can recover your identity without the phrase" }
                            }
                        }
                    }
                    div { class: "rounded-lg bg-emerald-900/20 p-3 border border-emerald-800/30",
                        h3 { class: "text-sm font-medium text-emerald-300 mb-2",
                            "Solution: Backup Recovery Phrase"
                        }
                        ul { class: "space-y-1 text-xs text-slate-400",
                            li { class: "flex items-start gap-2",
                                span { class: "text-emerald-400 mt-0.5", "\u{2713}" }
                                span { "Write down your 24-word recovery phrase" }
                            }
                            li { class: "flex items-start gap-2",
                                span { class: "text-emerald-400 mt-0.5", "\u{2713}" }
                                span { "Store it safely offline (not on this device)" }
                            }
                            li { class: "flex items-start gap-2",
                                span { class: "text-emerald-400 mt-0.5", "\u{2713}" }
                                span { "Use it to restore access on any device" }
                            }
                        }
                    }
                }
                // Action buttons
                div { class: "flex flex-col gap-2",
                    button {
                        class: "w-full rounded-xl bg-emerald-600 px-6 py-3 text-sm font-semibold text-white shadow-lg hover:bg-emerald-500 transition",
                        onclick: move |_| props.on_view_phrase.call(()),
                        "View Recovery Phrase"
                    }
                    button {
                        class: "w-full rounded-xl border border-slate-700 px-6 py-2 text-sm font-medium text-slate-400 hover:bg-slate-900 transition",
                        onclick: move |_| props.on_close.call(()),
                        "I'll do this later"
                    }
                }
            }
        }
    }
}

/// Warning triangle icon.
#[component]
fn WarningIcon(#[props(default = "h-6 w-6".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            view_box: "0 0 24 24",
            path {
                stroke_linecap: "round",
                stroke_linejoin: "round",
                d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_warning_badge_class_default() {
        let props = RecoveryWarningBadgeProps {
            class: String::new(),
        };
        assert!(props.class.is_empty());
    }

    #[test]
    fn recovery_warning_badge_with_custom_class() {
        let props = RecoveryWarningBadgeProps {
            class: "ml-2".to_string(),
        };
        assert_eq!(props.class, "ml-2");
    }
}
