//! Passkey/Biometric authentication prompt component.
//!
//! Displays a TouchID/FaceID prompt for biometric authentication,
//! with fallback to password authentication.

use dioxus::prelude::*;

/// State of the biometric authentication prompt.
#[derive(Debug, Clone, PartialEq)]
pub enum BiometricState {
    /// Waiting for user to initiate
    Idle,
    /// Authenticating (showing biometric prompt)
    Authenticating,
    /// Authentication succeeded
    Success,
    /// Authentication failed with error message
    Failed(String),
}

/// Props for the PasskeyPrompt component.
#[derive(Props, Clone, PartialEq)]
pub struct PasskeyPromptProps {
    /// Four-word identity being authenticated
    pub four_words: String,
    /// Display name of the identity
    pub display_name: String,
    /// Current authentication state
    #[props(default = BiometricState::Idle)]
    pub state: BiometricState,
    /// Callback when user initiates biometric auth
    pub on_authenticate: EventHandler<()>,
    /// Callback when user wants to use password instead
    pub on_use_password: EventHandler<()>,
    /// Callback to cancel/close the prompt
    pub on_cancel: EventHandler<()>,
}

/// Biometric authentication prompt modal.
///
/// Shows a centered modal with TouchID/FaceID icon and authentication button.
#[component]
pub fn PasskeyPrompt(props: PasskeyPromptProps) -> Element {
    let state_class = match &props.state {
        BiometricState::Idle => "",
        BiometricState::Authenticating => "animate-pulse",
        BiometricState::Success => "",
        BiometricState::Failed(_) => "",
    };

    let icon_color = match &props.state {
        BiometricState::Idle => "text-emerald-400",
        BiometricState::Authenticating => "text-emerald-300",
        BiometricState::Success => "text-emerald-500",
        BiometricState::Failed(_) => "text-red-400",
    };

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm",
            onclick: move |_| props.on_cancel.call(()),
            // Modal content
            div {
                class: "mx-4 w-full max-w-sm rounded-2xl border border-slate-800 bg-slate-950 p-8 shadow-2xl",
                onclick: move |evt| evt.stop_propagation(),
                // TouchID/Fingerprint icon
                div {
                    class: "flex justify-center mb-6 {state_class}",
                    FingerprintIcon { class: "h-20 w-20 {icon_color}" }
                }
                // Title
                h2 {
                    class: "text-center text-xl font-semibold text-slate-100 mb-2",
                    "Authenticate with Touch ID"
                }
                // Identity info
                div {
                    class: "text-center mb-6",
                    p { class: "text-sm text-slate-300", "{props.display_name}" }
                    p { class: "text-xs text-slate-500", "{props.four_words}" }
                }
                // State-specific content
                match &props.state {
                    BiometricState::Idle => rsx! {
                        // Authenticate button
                        button {
                            class: "w-full rounded-xl bg-emerald-600 px-6 py-3 text-sm font-semibold text-white shadow-lg hover:bg-emerald-500 transition",
                            onclick: move |_| props.on_authenticate.call(()),
                            "Use Touch ID"
                        }
                    },
                    BiometricState::Authenticating => rsx! {
                        // Loading state
                        div { class: "flex items-center justify-center gap-3 py-3",
                            div { class: "h-5 w-5 animate-spin rounded-full border-2 border-emerald-400 border-t-transparent" }
                            span { class: "text-sm text-slate-400", "Waiting for Touch ID..." }
                        }
                    },
                    BiometricState::Success => rsx! {
                        // Success state
                        div { class: "flex items-center justify-center gap-2 py-3 text-emerald-400",
                            CheckIcon { class: "h-6 w-6" }
                            span { class: "text-sm font-medium", "Authentication successful" }
                        }
                    },
                    BiometricState::Failed(message) => rsx! {
                        // Error state
                        div { class: "mb-4",
                            div { class: "flex items-center justify-center gap-2 py-2 text-red-400",
                                XCircleIcon { class: "h-6 w-6" }
                                span { class: "text-sm font-medium", "Authentication failed" }
                            }
                            p { class: "text-center text-xs text-slate-500 mt-1", "{message}" }
                        }
                        button {
                            class: "w-full rounded-xl bg-emerald-600 px-6 py-3 text-sm font-semibold text-white shadow-lg hover:bg-emerald-500 transition",
                            onclick: move |_| props.on_authenticate.call(()),
                            "Try Again"
                        }
                    },
                }
                // Fallback link
                if !matches!(props.state, BiometricState::Authenticating | BiometricState::Success) {
                    div { class: "mt-4 text-center",
                        button {
                            class: "text-sm text-slate-400 hover:text-slate-300 transition",
                            onclick: move |_| props.on_use_password.call(()),
                            "Use password instead"
                        }
                    }
                }
                // Cancel button
                if !matches!(props.state, BiometricState::Authenticating | BiometricState::Success) {
                    button {
                        class: "mt-4 w-full rounded-xl border border-slate-700 px-6 py-2 text-sm font-medium text-slate-400 hover:bg-slate-900 transition",
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

/// Props for the compact inline passkey button.
#[derive(Props, Clone, PartialEq)]
pub struct PasskeyButtonProps {
    /// Callback when button is clicked
    pub on_click: EventHandler<()>,
    /// Whether the button is loading
    #[props(default = false)]
    pub loading: bool,
    /// Whether to show full text or just icon
    #[props(default = false)]
    pub compact: bool,
}

/// Compact passkey authentication button for login forms.
#[component]
pub fn PasskeyButton(props: PasskeyButtonProps) -> Element {
    if props.compact {
        rsx! {
            button {
                class: "flex items-center justify-center rounded-lg border border-slate-700 p-3 text-slate-300 hover:border-emerald-400 hover:text-emerald-400 transition disabled:opacity-50",
                disabled: props.loading,
                onclick: move |_| props.on_click.call(()),
                title: "Sign in with Touch ID",
                if props.loading {
                    div { class: "h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" }
                } else {
                    FingerprintIcon { class: "h-5 w-5" }
                }
            }
        }
    } else {
        rsx! {
            button {
                class: "flex items-center justify-center gap-2 rounded-xl border border-slate-700 px-6 py-3 text-sm font-medium text-slate-300 hover:border-emerald-400 hover:text-emerald-400 transition disabled:opacity-50",
                disabled: props.loading,
                onclick: move |_| props.on_click.call(()),
                if props.loading {
                    div { class: "h-5 w-5 animate-spin rounded-full border-2 border-current border-t-transparent" }
                } else {
                    FingerprintIcon { class: "h-5 w-5" }
                }
                span { "Sign in with Touch ID" }
            }
        }
    }
}

/// Fingerprint/TouchID icon.
#[component]
fn FingerprintIcon(#[props(default = "h-6 w-6".to_string())] class: String) -> Element {
    rsx! {
        svg {
            class: "{class}",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "1.5",
            view_box: "0 0 24 24",
            // Fingerprint pattern
            path {
                stroke_linecap: "round",
                stroke_linejoin: "round",
                d: "M7.864 4.243A7.5 7.5 0 0119.5 10.5c0 2.92-.556 5.709-1.568 8.268M5.742 6.364A7.465 7.465 0 004.5 10.5a7.464 7.464 0 01-1.15 3.993m1.989 3.559A11.209 11.209 0 008.25 10.5a3.75 3.75 0 117.5 0c0 .527-.021 1.049-.064 1.565M12 10.5a14.94 14.94 0 01-3.6 9.75m6.633-4.596a18.666 18.666 0 01-2.485 5.33"
            }
        }
    }
}

/// Check icon for success state.
#[component]
fn CheckIcon(#[props(default = "h-6 w-6".to_string())] class: String) -> Element {
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
                d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            }
        }
    }
}

/// X circle icon for error state.
#[component]
fn XCircleIcon(#[props(default = "h-6 w-6".to_string())] class: String) -> Element {
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
                d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biometric_state_variants() {
        let idle = BiometricState::Idle;
        let auth = BiometricState::Authenticating;
        let success = BiometricState::Success;
        let failed = BiometricState::Failed("Test error".to_string());

        assert_eq!(idle, BiometricState::Idle);
        assert_eq!(auth, BiometricState::Authenticating);
        assert_eq!(success, BiometricState::Success);
        assert!(matches!(failed, BiometricState::Failed(_)));
    }
}
