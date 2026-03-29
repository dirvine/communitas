//! First-run onboarding gate for the Communitas desktop app.
//!
//! This component wraps the entire app and blocks the UI until the x0xd
//! daemon is installed and running. It is designed to be shown on first
//! launch (or whenever x0xd is unavailable) with no sidebars or empty
//! dashboards behind it.
//!
//! # States
//!
//! ```text
//! Checking → Running     → (show app)
//!          → NotInstalled → [Install x0x] → Installing → Starting → Running
//!                         → [Cancel]      → Cancelled  → [Try Again] | [Quit]
//!          → NotRunning  → [Start x0x]   → Starting   → Running
//! ```
//!
//! # Example
//!
//! ```ignore
//! rsx! {
//!     OnboardingGate {
//!         // your app children here
//!         Router::<Route> {}
//!     }
//! }
//! ```

use communitas_x0x_client::{DaemonManager, DaemonState};
use dioxus::prelude::*;
use tracing::warn;

use crate::tokens::{colors, radius, spacing, typography};

/// How long to poll for the daemon to become healthy after starting/installing
/// (in seconds). Generous timeout to account for first-run key generation.
const HEALTH_TIMEOUT_SECS: u64 = 30;

/// Terminate the application cleanly.
///
/// Wraps `std::process::exit` so event handler closures return `()`.
fn quit_app() {
    std::process::exit(0);
}

/// How often to poll health when waiting for startup (ms).
const POLL_INTERVAL_MS: u64 = 500;

/// Internal state machine for the onboarding gate.
#[derive(Clone, PartialEq, Eq, Debug)]
enum GateState {
    /// Initial state: checking whether x0xd is installed and running.
    Checking,
    /// x0x is not installed on this machine.
    NotInstalled,
    /// x0x is installed but the daemon is not running.
    NotRunning,
    /// Install in progress.
    Installing,
    /// Start in progress (after install or from NotRunning).
    Starting,
    /// An operation failed; the string is the human-readable error message.
    Failed(String),
    /// The user clicked Cancel from the NotInstalled screen.
    Cancelled,
    /// Daemon is healthy — the app content can be shown.
    Ready,
}

/// Full-screen onboarding gate component.
///
/// Renders the onboarding UI when x0xd is not available, or passes through
/// to `children` when the daemon is healthy.
#[component]
pub fn OnboardingGate(children: Element) -> Element {
    let mut gate = use_signal(|| GateState::Checking);

    // On mount, check daemon state once.
    use_future(move || async move {
        let manager = DaemonManager::new();
        let state = manager.state().await;
        match state {
            DaemonState::Running | DaemonState::Degraded => gate.set(GateState::Ready),
            DaemonState::NotRunning => gate.set(GateState::NotRunning),
            DaemonState::NotInstalled => gate.set(GateState::NotInstalled),
        }
    });

    match gate() {
        GateState::Ready => rsx! { {children} },

        GateState::Checking => rsx! {
            OnboardingScreen {
                CheckingView {}
            }
        },

        GateState::NotInstalled => rsx! {
            OnboardingScreen {
                NotInstalledView {
                    on_install: move |_| {
                        gate.set(GateState::Installing);
                        spawn(async move {
                            match DaemonManager::install().await {
                                Ok(()) => {
                                    gate.set(GateState::Starting);
                                    match DaemonManager::start().await {
                                        Ok(()) => {
                                            if poll_until_healthy(HEALTH_TIMEOUT_SECS).await {
                                                gate.set(GateState::Ready);
                                            } else {
                                                gate.set(GateState::Failed(
                                                    "x0x started but did not become healthy within 15 seconds. Try restarting Communitas.".to_string(),
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            warn!(target: "ui.onboarding", "failed to start x0x after install: {e}");
                                            gate.set(GateState::Failed(format!("x0x installed but failed to start: {e}")));
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!(target: "ui.onboarding", "x0x install failed: {e}");
                                    gate.set(GateState::Failed(format!("Installation failed: {e}")));
                                }
                            }
                        });
                    },
                    on_cancel: move |_| gate.set(GateState::Cancelled),
                }
            }
        },

        GateState::NotRunning => rsx! {
            OnboardingScreen {
                NotRunningView {
                    on_start: move |_| {
                        gate.set(GateState::Starting);
                        spawn(async move {
                            match DaemonManager::start().await {
                                Ok(()) => {
                                    if poll_until_healthy(HEALTH_TIMEOUT_SECS).await {
                                        gate.set(GateState::Ready);
                                    } else {
                                        gate.set(GateState::Failed(
                                            "x0x did not become healthy within 15 seconds. Try restarting Communitas.".to_string(),
                                        ));
                                    }
                                }
                                Err(e) => {
                                    warn!(target: "ui.onboarding", "failed to start x0x daemon: {e}");
                                    gate.set(GateState::Failed(format!("Failed to start x0x: {e}")));
                                }
                            }
                        });
                    },
                    on_quit: move |_| quit_app(),
                }
            }
        },

        GateState::Installing => rsx! {
            OnboardingScreen {
                ProgressView {
                    message: "Installing x0x...",
                    detail: "Downloading and installing via curl -sfL https://x0x.md | sh",
                }
            }
        },

        GateState::Starting => rsx! {
            OnboardingScreen {
                ProgressView {
                    message: "Starting x0x...",
                    detail: "Waiting for the daemon to become healthy.",
                }
            }
        },

        GateState::Failed(ref err) => {
            let err_msg = err.clone();
            rsx! {
                OnboardingScreen {
                    FailedView {
                        error: err_msg,
                        on_retry: move |_| gate.set(GateState::Checking),
                    }
                }
            }
        }

        GateState::Cancelled => rsx! {
            OnboardingScreen {
                CancelledView {
                    on_try_again: move |_| gate.set(GateState::Checking),
                    on_quit: move |_| quit_app(),
                }
            }
        },
    }
}

// ── Helper ───────────────────────────────────────────────────────────────────

/// Poll the x0xd health endpoint until it responds successfully or the
/// `timeout_secs` deadline elapses. Returns `true` if healthy.
///
/// Re-discovers the daemon config on every poll iteration to handle the case
/// where x0xd hasn't finished writing `api.port` / `api-token` yet.
async fn poll_until_healthy(timeout_secs: u64) -> bool {
    use communitas_x0x_client::X0xClient;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        // Re-discover on every iteration — the daemon may have just started
        // and written its config files since the last attempt.
        let client = X0xClient::new();
        if client.health().await.is_ok() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
    }
}

// ── Layout wrapper ────────────────────────────────────────────────────────────

/// Full-viewport centered wrapper shared by all onboarding screens.
#[component]
fn OnboardingScreen(children: Element) -> Element {
    let backdrop_style = format!(
        "position: fixed; inset: 0; z-index: 1000; \
         display: flex; align-items: center; justify-content: center; \
         background: {}; \
         background-image: \
           radial-gradient(ellipse at 20% 80%, rgba(0, 212, 255, 0.04) 0%, transparent 50%), \
           radial-gradient(ellipse at 80% 20%, rgba(0, 212, 255, 0.06) 0%, transparent 50%);",
        colors::SURFACE_BG,
    );

    let card_style = format!(
        "background: {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         box-shadow: 0 20px 60px rgba(0, 0, 0, 0.6), 0 0 40px rgba(0, 212, 255, 0.06); \
         padding: {} {}; \
         width: 100%; \
         max-width: 480px; \
         display: flex; \
         flex-direction: column; \
         align-items: center; \
         gap: {};",
        colors::SURFACE_CARD,
        colors::BORDER_DEFAULT,
        radius::XL,
        spacing::XXL,
        spacing::XXL,
        spacing::LG,
    );

    rsx! {
        div {
            style: "{backdrop_style}",
            role: "dialog",
            "aria-modal": "true",
            "aria-label": "x0x setup required",

            div {
                style: "{card_style}",
                {children}
            }
        }
    }
}

// ── Logo / wordmark ───────────────────────────────────────────────────────────

/// Communitas logo area shown at the top of every onboarding card.
#[component]
fn AppLogo() -> Element {
    let icon_style = format!(
        "width: 64px; height: 64px; border-radius: {}; \
         background: linear-gradient(135deg, {} 0%, {} 100%); \
         display: flex; align-items: center; justify-content: center; \
         font-size: 2rem; \
         box-shadow: 0 0 24px rgba(0, 212, 255, 0.3);",
        radius::XL,
        colors::PRIMARY,
        "#0080ff",
    );

    let title_style = format!(
        "font-family: {}; font-size: {}; font-weight: 700; \
         color: {}; letter-spacing: -0.02em; margin: 0;",
        typography::FONT_SANS,
        typography::TEXT_2XL,
        colors::TEXT_PRIMARY,
    );

    rsx! {
        div {
            style: "display: flex; flex-direction: column; align-items: center; gap: 0.75rem;",
            div {
                style: "{icon_style}",
                "aria-hidden": "true",
                "🌐"
            }
            h1 {
                style: "{title_style}",
                "Communitas"
            }
        }
    }
}

// ── Sub-components ────────────────────────────────────────────────────────────

/// Shown while performing the initial daemon state check.
#[component]
fn CheckingView() -> Element {
    let msg_style = format!(
        "color: {}; font-family: {}; font-size: {}; text-align: center;",
        colors::TEXT_SECONDARY,
        typography::FONT_SANS,
        typography::TEXT_SM,
    );

    rsx! {
        AppLogo {}
        p { style: "{msg_style}", "Checking x0x daemon status…" }
        SpinnerIcon {}
    }
}

/// Shown when x0x is not installed.
#[component]
fn NotInstalledView(
    on_install: EventHandler<MouseEvent>,
    on_cancel: EventHandler<MouseEvent>,
) -> Element {
    let body_style = format!(
        "color: {}; font-family: {}; font-size: {}; \
         text-align: center; line-height: 1.6; max-width: 360px;",
        colors::TEXT_SECONDARY,
        typography::FONT_SANS,
        typography::TEXT_SM,
    );

    rsx! {
        AppLogo {}

        p { style: "{body_style}",
            "Communitas needs "
            strong { style: "color: {colors::TEXT_PRIMARY};", "x0x" }
            " to connect to the decentralized network."
        }

        ButtonRow {
            PrimaryButton {
                label: "Install x0x",
                onclick: move |e| on_install.call(e),
            }
            SecondaryButton {
                label: "Cancel",
                onclick: move |e| on_cancel.call(e),
            }
        }
    }
}

/// Shown when x0x is installed but not running.
#[component]
fn NotRunningView(
    on_start: EventHandler<MouseEvent>,
    on_quit: EventHandler<MouseEvent>,
) -> Element {
    let heading_style = format!(
        "font-family: {}; font-size: {}; font-weight: 600; \
         color: {}; text-align: center; margin: 0;",
        typography::FONT_SANS,
        typography::TEXT_LG,
        colors::TEXT_PRIMARY,
    );

    let body_style = format!(
        "color: {}; font-family: {}; font-size: {}; \
         text-align: center; line-height: 1.6; max-width: 360px;",
        colors::TEXT_SECONDARY,
        typography::FONT_SANS,
        typography::TEXT_SM,
    );

    rsx! {
        AppLogo {}

        h2 { style: "{heading_style}", "Starting x0x…" }

        p { style: "{body_style}",
            "x0x is installed but not currently running. \
             Click "
            strong { style: "color: {colors::TEXT_PRIMARY};", "Start x0x" }
            " to launch the daemon."
        }

        ButtonRow {
            PrimaryButton {
                label: "Start x0x",
                onclick: move |e| on_start.call(e),
            }
            DestructiveButton {
                label: "Quit",
                onclick: move |e| on_quit.call(e),
            }
        }
    }
}

/// Shown while an install or start operation is in progress.
#[component]
fn ProgressView(message: &'static str, detail: &'static str) -> Element {
    let heading_style = format!(
        "font-family: {}; font-size: {}; font-weight: 600; \
         color: {}; text-align: center; margin: 0;",
        typography::FONT_SANS,
        typography::TEXT_LG,
        colors::TEXT_PRIMARY,
    );

    let detail_style = format!(
        "color: {}; font-family: {}; font-size: {}; \
         text-align: center; line-height: 1.6; max-width: 360px;",
        colors::TEXT_MUTED,
        typography::FONT_SANS,
        typography::TEXT_SM,
    );

    rsx! {
        AppLogo {}
        h2 { style: "{heading_style}", "{message}" }
        SpinnerIcon {}
        p { style: "{detail_style}", "{detail}" }
    }
}

/// Shown after an operation failed, with an error message and retry button.
#[component]
fn FailedView(error: String, on_retry: EventHandler<MouseEvent>) -> Element {
    let heading_style = format!(
        "font-family: {}; font-size: {}; font-weight: 600; \
         color: {}; text-align: center; margin: 0;",
        typography::FONT_SANS,
        typography::TEXT_LG,
        colors::ERROR,
    );

    let error_style = format!(
        "color: {}; font-family: {}; font-size: {}; \
         text-align: center; line-height: 1.6; max-width: 360px; \
         background: rgba(255, 68, 102, 0.08); \
         border: 1px solid rgba(255, 68, 102, 0.25); \
         border-radius: {}; \
         padding: {} {};",
        colors::DANGER,
        typography::FONT_SANS,
        typography::TEXT_SM,
        radius::LG,
        spacing::SM,
        spacing::MD,
    );

    rsx! {
        AppLogo {}
        h2 { style: "{heading_style}", "Something went wrong" }
        p { style: "{error_style}", "{error}" }
        ButtonRow {
            PrimaryButton {
                label: "Try Again",
                onclick: move |e| on_retry.call(e),
            }
        }
    }
}

/// Shown after the user clicks Cancel from the NotInstalled screen.
#[component]
fn CancelledView(
    on_try_again: EventHandler<MouseEvent>,
    on_quit: EventHandler<MouseEvent>,
) -> Element {
    let heading_style = format!(
        "font-family: {}; font-size: {}; font-weight: 600; \
         color: {}; text-align: center; margin: 0;",
        typography::FONT_SANS,
        typography::TEXT_LG,
        colors::TEXT_PRIMARY,
    );

    let body_style = format!(
        "color: {}; font-family: {}; font-size: {}; \
         text-align: center; line-height: 1.6; max-width: 360px;",
        colors::TEXT_SECONDARY,
        typography::FONT_SANS,
        typography::TEXT_SM,
    );

    let code_style = format!(
        "display: inline-block; \
         font-family: {}; font-size: {}; \
         color: {}; \
         background: {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         padding: 0.125rem {}; \
         user-select: all; \
         cursor: text;",
        typography::FONT_MONO,
        typography::TEXT_SM,
        colors::PRIMARY,
        colors::SURFACE_ELEVATED,
        colors::BORDER_DEFAULT,
        radius::MD,
        spacing::SM,
    );

    rsx! {
        AppLogo {}

        h2 { style: "{heading_style}", "x0x is required" }

        p { style: "{body_style}",
            "Communitas requires x0x to function. Please install it \
             and restart Communitas when you're ready."
        }

        p { style: "{body_style}", "Install manually:" }

        code { style: "{code_style}", "curl -sfL https://x0x.md | sh" }

        ButtonRow {
            PrimaryButton {
                label: "Try Again",
                onclick: move |e| on_try_again.call(e),
            }
            DestructiveButton {
                label: "Quit",
                onclick: move |e| on_quit.call(e),
            }
        }
    }
}

// ── Shared button / layout primitives ────────────────────────────────────────

/// Row wrapper for action buttons.
#[component]
fn ButtonRow(children: Element) -> Element {
    rsx! {
        div {
            style: "display: flex; flex-direction: row; gap: 0.75rem; justify-content: center; flex-wrap: wrap; width: 100%;",
            {children}
        }
    }
}

/// Primary (accent) action button.
#[component]
fn PrimaryButton(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    let style = format!(
        "background: linear-gradient(135deg, {} 0%, #0080ff 100%); \
         color: {}; \
         font-family: {}; font-size: {}; font-weight: 600; \
         padding: {} {}; \
         border: none; border-radius: {}; \
         cursor: pointer; \
         box-shadow: 0 0 16px rgba(0, 212, 255, 0.3); \
         transition: box-shadow 150ms ease, transform 150ms ease; \
         white-space: nowrap;",
        colors::PRIMARY,
        "#0a0c14",
        typography::FONT_SANS,
        typography::TEXT_SM,
        spacing::SM,
        spacing::LG,
        radius::LG,
    );

    rsx! {
        button {
            r#type: "button",
            style: "{style}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

/// Secondary (muted) action button.
#[component]
fn SecondaryButton(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    let style = format!(
        "background: transparent; \
         color: {}; \
         font-family: {}; font-size: {}; font-weight: 500; \
         padding: {} {}; \
         border: 1px solid {}; border-radius: {}; \
         cursor: pointer; \
         transition: border-color 150ms ease, color 150ms ease; \
         white-space: nowrap;",
        colors::TEXT_MUTED,
        typography::FONT_SANS,
        typography::TEXT_SM,
        spacing::SM,
        spacing::LG,
        colors::BORDER_DEFAULT,
        radius::LG,
    );

    rsx! {
        button {
            r#type: "button",
            style: "{style}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

/// Destructive action button (Quit).
#[component]
fn DestructiveButton(label: &'static str, onclick: EventHandler<MouseEvent>) -> Element {
    let style = format!(
        "background: transparent; \
         color: {}; \
         font-family: {}; font-size: {}; font-weight: 500; \
         padding: {} {}; \
         border: 1px solid rgba(255, 68, 102, 0.4); border-radius: {}; \
         cursor: pointer; \
         transition: border-color 150ms ease, background 150ms ease; \
         white-space: nowrap;",
        colors::DANGER,
        typography::FONT_SANS,
        typography::TEXT_SM,
        spacing::SM,
        spacing::LG,
        radius::LG,
    );

    rsx! {
        button {
            r#type: "button",
            style: "{style}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

/// Animated spinner icon for in-progress states.
#[component]
fn SpinnerIcon() -> Element {
    let style = format!(
        "width: 32px; height: 32px; \
         border: 3px solid {}; \
         border-top-color: {}; \
         border-radius: 50%; \
         animation: communitas-spin 700ms linear infinite;",
        colors::BORDER_DEFAULT,
        colors::PRIMARY,
    );

    // Keyframes injected inline — Dioxus desktop uses WebView CSS so a
    // `<style>` element works the same as global CSS.
    rsx! {
        style {
            "@keyframes communitas-spin {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}"
        }
        div {
            style: "{style}",
            role: "status",
            "aria-label": "Loading",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_state_checking_is_initial() {
        let s = GateState::Checking;
        assert_eq!(s, GateState::Checking);
    }

    #[test]
    fn gate_state_failed_carries_message() {
        let msg = "something went wrong".to_string();
        let s = GateState::Failed(msg.clone());
        assert_eq!(s, GateState::Failed(msg));
    }

    #[test]
    fn gate_state_ready_not_equal_checking() {
        assert_ne!(GateState::Ready, GateState::Checking);
    }
}
