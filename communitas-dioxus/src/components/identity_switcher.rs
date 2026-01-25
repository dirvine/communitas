//! Identity switcher component for quick identity switching.
//!
//! Provides a dropdown menu displaying the current identity and recent identities,
//! with passkey indicators and management capabilities.
//!
//! ## Features
//! - Quick-switch between identities with biometric auth for passkey-protected identities
//! - Passkey management (register/delete)
//! - Recovery backup warnings
//! - Last-used timestamps
//! - Keyboard shortcut: Cmd+Shift+I (macOS) / Ctrl+Shift+I (Windows/Linux)

use crate::components::auth::{
    BiometricState, PasskeyPrompt, RecoverySetupModal, RecoveryWarningBadge,
};
use crate::{AuthPhase, use_auth};
use communitas_ui_service::UiServices;
use communitas_ui_service::auth::{AuthService, RecentIdentity};
use dioxus::prelude::*;
use futures::StreamExt;
use std::sync::Arc;
use tracing::{error, info};

/// Keyboard shortcut hint text for the identity switcher.
pub const KEYBOARD_SHORTCUT_HINT: &str = "\u{2318}\u{21E7}I"; // ⌘⇧I for macOS

/// Message types for identity switcher coroutine.
#[derive(Debug, Clone)]
enum SwitcherAction {
    /// Switch to a different identity (without passkey)
    SwitchIdentity(String),
    /// Start biometric authentication for identity switch (with passkey)
    StartBiometricAuth(String, String), // (four_words, display_name)
    /// Register passkey for current session
    RegisterPasskey,
    /// Delete passkey for an identity
    DeletePasskey(String),
    /// Remove identity from recent list
    RemoveRecent(String),
    /// Refresh the list of recent identities
    RefreshList,
}

/// Props for the IdentitySwitcher component.
#[derive(Props, Clone, PartialEq)]
pub struct IdentitySwitcherProps {
    /// Callback when logout is requested
    pub on_logout: EventHandler<()>,
}

/// Identity switcher dropdown component.
///
/// Shows the current identity with a dropdown menu of recent identities,
/// allowing quick switching and passkey management.
#[component]
pub fn IdentitySwitcher(props: IdentitySwitcherProps) -> Element {
    let mut auth = use_auth();
    let services = use_context::<Arc<UiServices>>();
    let auth_service = services.auth();

    let mut dropdown_open = use_signal(|| false);
    let mut modal_open = use_signal(|| false);
    let mut recent_identities = use_signal(Vec::<RecentIdentity>::new);
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    // Biometric auth state for passkey-protected identity switching
    let mut biometric_prompt_open = use_signal(|| false);
    let mut biometric_state = use_signal(|| BiometricState::Idle);
    let mut biometric_target = use_signal(|| Option::<(String, String)>::None); // (four_words, display_name)

    // Recovery setup modal state
    let mut recovery_modal_open = use_signal(|| false);
    let mut recovery_modal_identity = use_signal(|| Option::<(String, String)>::None); // (four_words, display_name)

    let session = auth.read().session.clone();

    // Global keyboard shortcut handler (Cmd+Shift+I / Ctrl+Shift+I)
    // This provides a quick way to access the identity switcher
    // Note: Full keyboard shortcut implementation requires registration at the App level
    // The shortcut hint is shown in the button tooltip for discoverability

    // Load recent identities when dropdown opens
    let load_action = {
        let auth_service = auth_service.clone();
        use_coroutine(move |mut rx: UnboundedReceiver<SwitcherAction>| {
            let auth_service = auth_service.clone();
            async move {
                while let Some(action) = rx.next().await {
                    match action {
                        SwitcherAction::RefreshList => {
                            is_loading.set(true);
                            error_message.set(None);
                            match auth_service.list_recent_identities().await {
                                Ok(identities) => {
                                    recent_identities.set(identities);
                                }
                                Err(err) => {
                                    error!(target: "ui.identity_switcher", "Failed to load recent identities: {err}");
                                    error_message
                                        .set(Some(format!("Failed to load identities: {err}")));
                                }
                            }
                            is_loading.set(false);
                        }
                        SwitcherAction::SwitchIdentity(four_words) => {
                            is_loading.set(true);
                            error_message.set(None);
                            match auth_service.switch_identity(&four_words).await {
                                Ok(new_session) => {
                                    info!(target: "ui.identity_switcher", "Switched to identity: {}", four_words);
                                    auth.with_mut(|state| {
                                        state.session = Some(new_session);
                                        state.phase = AuthPhase::Authenticated;
                                        state.error = None;
                                    });
                                    dropdown_open.set(false);
                                    biometric_prompt_open.set(false);
                                }
                                Err(err) => {
                                    error!(target: "ui.identity_switcher", "Failed to switch identity: {err}");
                                    error_message.set(Some(format!("Failed to switch: {err}")));
                                }
                            }
                            is_loading.set(false);
                        }
                        SwitcherAction::StartBiometricAuth(four_words, display_name) => {
                            // Show biometric prompt and start authentication
                            biometric_target.set(Some((four_words.clone(), display_name.clone())));
                            biometric_state.set(BiometricState::Authenticating);
                            biometric_prompt_open.set(true);
                            dropdown_open.set(false);

                            // Attempt biometric/passkey authentication via identity switch
                            // The switch_identity method handles passkey auth internally
                            match auth_service.switch_identity(&four_words).await {
                                Ok(new_session) => {
                                    info!(target: "ui.identity_switcher", "Biometric auth succeeded for: {}", four_words);
                                    biometric_state.set(BiometricState::Success);
                                    // Brief delay to show success state before closing
                                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                    auth.with_mut(|state| {
                                        state.session = Some(new_session);
                                        state.phase = AuthPhase::Authenticated;
                                        state.error = None;
                                    });
                                    biometric_prompt_open.set(false);
                                    biometric_state.set(BiometricState::Idle);
                                    biometric_target.set(None);
                                }
                                Err(err) => {
                                    error!(target: "ui.identity_switcher", "Biometric auth failed: {err}");
                                    biometric_state.set(BiometricState::Failed(err.to_string()));
                                }
                            }
                        }
                        SwitcherAction::RegisterPasskey => {
                            is_loading.set(true);
                            error_message.set(None);
                            match auth_service.register_passkey().await {
                                Ok(()) => {
                                    info!(target: "ui.identity_switcher", "Passkey registered successfully");
                                    // Refresh list to show updated passkey status
                                    if let Ok(identities) =
                                        auth_service.list_recent_identities().await
                                    {
                                        recent_identities.set(identities);
                                    }
                                }
                                Err(err) => {
                                    error!(target: "ui.identity_switcher", "Failed to register passkey: {err}");
                                    error_message
                                        .set(Some(format!("Failed to register passkey: {err}")));
                                }
                            }
                            is_loading.set(false);
                        }
                        SwitcherAction::DeletePasskey(four_words) => {
                            is_loading.set(true);
                            error_message.set(None);
                            match auth_service.delete_passkey(&four_words).await {
                                Ok(()) => {
                                    info!(target: "ui.identity_switcher", "Passkey deleted for: {}", four_words);
                                    // Refresh list to show updated passkey status
                                    if let Ok(identities) =
                                        auth_service.list_recent_identities().await
                                    {
                                        recent_identities.set(identities);
                                    }
                                }
                                Err(err) => {
                                    error!(target: "ui.identity_switcher", "Failed to delete passkey: {err}");
                                    error_message
                                        .set(Some(format!("Failed to delete passkey: {err}")));
                                }
                            }
                            is_loading.set(false);
                        }
                        SwitcherAction::RemoveRecent(four_words) => {
                            is_loading.set(true);
                            error_message.set(None);
                            match auth_service.remove_recent_identity(&four_words).await {
                                Ok(()) => {
                                    info!(target: "ui.identity_switcher", "Removed identity from recent: {}", four_words);
                                    // Refresh list
                                    if let Ok(identities) =
                                        auth_service.list_recent_identities().await
                                    {
                                        recent_identities.set(identities);
                                    }
                                }
                                Err(err) => {
                                    error!(target: "ui.identity_switcher", "Failed to remove identity: {err}");
                                    error_message.set(Some(format!("Failed to remove: {err}")));
                                }
                            }
                            is_loading.set(false);
                        }
                    }
                }
            }
        })
    };

    // Load identities when dropdown opens
    let load_action_clone = load_action;
    let on_dropdown_toggle = move |_| {
        let new_state = !dropdown_open();
        dropdown_open.set(new_state);
        if new_state {
            load_action_clone.send(SwitcherAction::RefreshList);
        }
    };

    let current_four_words = session.as_ref().map(|s| s.four_words.clone());
    let current_has_passkey = {
        let identities = recent_identities.read();
        current_four_words
            .as_ref()
            .and_then(|fw| {
                identities
                    .iter()
                    .find(|i| &i.four_words == fw)
                    .map(|i| i.has_passkey)
            })
            .unwrap_or(false)
    };

    // Capture identities for the dropdown render
    let identities_for_dropdown: Vec<RecentIdentity> = recent_identities.read().clone();
    let identities_for_modal: Vec<RecentIdentity> = recent_identities.read().clone();

    rsx! {
        div {
            class: "relative",
            // Current identity button
            if let Some(session) = session.clone() {
                button {
                    class: "flex items-center gap-3 rounded-lg border border-slate-700 px-4 py-2 text-left transition hover:border-emerald-400 hover:bg-slate-900/50",
                    onclick: on_dropdown_toggle,
                    aria_expanded: dropdown_open(),
                    aria_haspopup: "true",
                    title: "Switch identity ({KEYBOARD_SHORTCUT_HINT})",
                    // Identity info
                    div { class: "flex flex-col",
                        div { class: "flex items-center gap-2",
                            span { class: "font-semibold text-slate-100", "{session.display_name}" }
                            if current_has_passkey {
                                PasskeyBadge {}
                            }
                        }
                        p { class: "text-xs text-slate-500", "{session.four_words}" }
                    }
                    // Dropdown arrow
                    svg {
                        class: if dropdown_open() { "h-4 w-4 text-slate-400 rotate-180 transition-transform" } else { "h-4 w-4 text-slate-400 transition-transform" },
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        view_box: "0 0 24 24",
                        path {
                            d: "M19 9l-7 7-7-7",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                        }
                    }
                }
            }

            // Dropdown backdrop (closes on click outside)
            if dropdown_open() {
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| dropdown_open.set(false),
                }
            }

            // Dropdown menu
            if dropdown_open() {
                div {
                    class: "absolute right-0 top-full z-50 mt-2 w-72 rounded-xl border border-slate-800 bg-slate-950 shadow-xl",
                    role: "menu",
                    // Error message if any
                    if let Some(err) = error_message() {
                        div { class: "px-4 py-2 text-xs text-red-400 border-b border-slate-800",
                            "{err}"
                        }
                    }
                    // Loading indicator
                    if is_loading() {
                        div { class: "px-4 py-2 text-xs text-slate-500 border-b border-slate-800",
                            "Loading..."
                        }
                    }
                    // Recent identities section
                    div { class: "py-2",
                        p { class: "px-4 py-1 text-xs uppercase tracking-wider text-slate-600",
                            "Switch Identity"
                        }
                        {identities_for_dropdown.iter().map(|identity| {
                            let four_words = identity.four_words.clone();
                            let display_name = identity.display_name.clone();
                            let has_passkey = identity.has_passkey;
                            // Show recovery warning if passkey is enabled
                            // In production, this would also check !has_recovery
                            let needs_recovery = has_passkey;
                            let is_current = current_four_words.as_ref().is_some_and(|fw| fw == &four_words);
                            let action = load_action;
                            let fw_for_switch = four_words.clone();
                            let dn_for_switch = display_name.clone();
                            let fw_for_recovery = four_words.clone();
                            let dn_for_recovery = display_name.clone();
                            rsx! {
                                IdentityItem {
                                    key: "{four_words}",
                                    identity: identity.clone(),
                                    is_current: is_current,
                                    needs_recovery_warning: needs_recovery,
                                    on_select: move |_| {
                                        if !is_current {
                                            if has_passkey {
                                                // Use biometric auth for passkey-protected identities
                                                action.send(SwitcherAction::StartBiometricAuth(
                                                    fw_for_switch.clone(),
                                                    dn_for_switch.clone()
                                                ));
                                            } else {
                                                // Direct switch for non-passkey identities
                                                action.send(SwitcherAction::SwitchIdentity(fw_for_switch.clone()));
                                            }
                                        }
                                    },
                                    on_setup_recovery: Some(EventHandler::new(move |_| {
                                        recovery_modal_identity.set(Some((fw_for_recovery.clone(), dn_for_recovery.clone())));
                                        recovery_modal_open.set(true);
                                    })),
                                }
                            }
                        })}
                    }
                    // Divider
                    div { class: "border-t border-slate-800" }
                    // Actions
                    div { class: "py-2",
                        // Settings button
                        button {
                            class: "flex w-full items-center gap-3 px-4 py-2 text-sm text-slate-300 hover:bg-slate-900",
                            onclick: move |_| {
                                modal_open.set(true);
                                dropdown_open.set(false);
                            },
                            // Gear icon
                            svg {
                                class: "h-4 w-4 text-slate-500",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                }
                                circle {
                                    cx: "12",
                                    cy: "12",
                                    r: "3",
                                }
                            }
                            span { "Passkey Settings" }
                        }
                        // Logout button
                        button {
                            class: "flex w-full items-center gap-3 px-4 py-2 text-sm text-red-400 hover:bg-slate-900",
                            onclick: move |_| {
                                dropdown_open.set(false);
                                props.on_logout.call(());
                            },
                            // Logout icon
                            svg {
                                class: "h-4 w-4",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                }
                            }
                            span { "Logout" }
                        }
                    }
                }
            }

            // Passkey management modal
            if modal_open() {
                PasskeyManagementModal {
                    identities: identities_for_modal.clone(),
                    current_four_words: current_four_words.clone().unwrap_or_default(),
                    on_close: move |_| modal_open.set(false),
                    on_register: move |_| {
                        load_action.send(SwitcherAction::RegisterPasskey);
                    },
                    on_delete: move |four_words: String| {
                        load_action.send(SwitcherAction::DeletePasskey(four_words));
                    },
                    on_remove: move |four_words: String| {
                        load_action.send(SwitcherAction::RemoveRecent(four_words));
                    },
                }
            }

            // Biometric authentication prompt
            if biometric_prompt_open() {
                if let Some((target_four_words, target_display_name)) = biometric_target() {
                    PasskeyPrompt {
                        four_words: target_four_words.clone(),
                        display_name: target_display_name.clone(),
                        state: biometric_state(),
                        on_authenticate: move |_| {
                            // Retry authentication
                            if let Some((fw, dn)) = biometric_target() {
                                load_action.send(SwitcherAction::StartBiometricAuth(fw, dn));
                            }
                        },
                        on_use_password: move |_| {
                            // Fall back to password auth (just switch without passkey)
                            if let Some((fw, _)) = biometric_target() {
                                biometric_prompt_open.set(false);
                                biometric_state.set(BiometricState::Idle);
                                biometric_target.set(None);
                                load_action.send(SwitcherAction::SwitchIdentity(fw));
                            }
                        },
                        on_cancel: move |_| {
                            // Cancel and close prompt
                            biometric_prompt_open.set(false);
                            biometric_state.set(BiometricState::Idle);
                            biometric_target.set(None);
                        },
                    }
                }
            }

            // Recovery setup modal
            if recovery_modal_open() {
                if let Some((fw, dn)) = recovery_modal_identity() {
                    RecoverySetupModal {
                        four_words: fw.clone(),
                        display_name: dn.clone(),
                        on_close: move |_| {
                            recovery_modal_open.set(false);
                            recovery_modal_identity.set(None);
                        },
                        on_view_phrase: move |_| {
                            // TODO: Navigate to recovery phrase view
                            // For now, just close the modal
                            info!(target: "ui.identity_switcher", "User requested to view recovery phrase");
                            recovery_modal_open.set(false);
                            recovery_modal_identity.set(None);
                        },
                    }
                }
            }
        }
    }
}

/// Props for a single identity item in the dropdown.
#[derive(Props, Clone, PartialEq)]
struct IdentityItemProps {
    identity: RecentIdentity,
    is_current: bool,
    /// Whether this identity needs a recovery backup warning
    #[props(default = false)]
    needs_recovery_warning: bool,
    on_select: EventHandler<()>,
    /// Callback when user clicks to set up recovery (optional)
    on_setup_recovery: Option<EventHandler<()>>,
}

/// Single identity row in the dropdown.
#[component]
fn IdentityItem(props: IdentityItemProps) -> Element {
    let bg_class = if props.is_current {
        "flex w-full items-center justify-between px-4 py-2 bg-slate-900/50 cursor-default"
    } else {
        "flex w-full items-center justify-between px-4 py-2 hover:bg-slate-900 cursor-pointer"
    };

    let last_used_text = format_relative_time(props.identity.last_used);

    rsx! {
        button {
            class: "{bg_class}",
            role: "menuitem",
            disabled: props.is_current,
            onclick: move |_| props.on_select.call(()),
            div { class: "flex flex-col text-left",
                div { class: "flex items-center gap-2",
                    span {
                        class: if props.is_current { "text-sm font-medium text-emerald-400" } else { "text-sm font-medium text-slate-200" },
                        "{props.identity.display_name}"
                    }
                    if props.identity.has_passkey {
                        PasskeyBadge {}
                    }
                    // Show warning badge if passkey is set but no recovery backup
                    if props.needs_recovery_warning {
                        RecoveryWarningBadge { class: "ml-1".to_string() }
                    }
                }
                div { class: "flex items-center gap-2",
                    p { class: "text-xs text-slate-500", "{props.identity.four_words}" }
                    span { class: "text-xs text-slate-600", "\u{2022}" }
                    p { class: "text-xs text-slate-600", "{last_used_text}" }
                }
            }
            if props.is_current {
                span { class: "text-xs text-emerald-400", "Current" }
            }
        }
    }
}

/// Format a Unix timestamp as a relative time string.
fn format_relative_time(timestamp: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let diff = now.saturating_sub(timestamp);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        if mins == 1 {
            "1 min ago".to_string()
        } else {
            format!("{mins} mins ago")
        }
    } else if diff < 86400 {
        let hours = diff / 3600;
        if hours == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{hours} hours ago")
        }
    } else if diff < 604800 {
        let days = diff / 86400;
        if days == 1 {
            "yesterday".to_string()
        } else {
            format!("{days} days ago")
        }
    } else {
        let weeks = diff / 604800;
        if weeks == 1 {
            "1 week ago".to_string()
        } else {
            format!("{weeks} weeks ago")
        }
    }
}

/// Small passkey indicator badge (key icon).
#[component]
fn PasskeyBadge() -> Element {
    rsx! {
        span {
            class: "inline-flex items-center rounded bg-emerald-900/30 px-1.5 py-0.5",
            title: "Passkey enabled",
            // Key icon
            svg {
                class: "h-3 w-3 text-emerald-400",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                view_box: "0 0 24 24",
                path {
                    d: "M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                }
            }
        }
    }
}

/// Props for the passkey management modal.
#[derive(Props, Clone, PartialEq)]
struct PasskeyManagementModalProps {
    identities: Vec<RecentIdentity>,
    current_four_words: String,
    on_close: EventHandler<()>,
    on_register: EventHandler<()>,
    on_delete: EventHandler<String>,
    on_remove: EventHandler<String>,
}

/// Modal for managing passkeys and recent identities.
#[component]
fn PasskeyManagementModal(props: PasskeyManagementModalProps) -> Element {
    let current_has_passkey = props
        .identities
        .iter()
        .find(|i| i.four_words == props.current_four_words)
        .is_some_and(|i| i.has_passkey);

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm",
            onclick: move |_| props.on_close.call(()),
            // Modal content
            div {
                class: "mx-4 w-full max-w-md rounded-2xl border border-slate-800 bg-slate-950 shadow-2xl",
                onclick: move |evt| evt.stop_propagation(),
                // Header
                div { class: "flex items-center justify-between border-b border-slate-800 px-6 py-4",
                    h2 { class: "text-lg font-semibold text-slate-100", "Passkey Settings" }
                    button {
                        class: "rounded-lg p-1 text-slate-400 hover:bg-slate-800 hover:text-slate-200",
                        onclick: move |_| props.on_close.call(()),
                        // X icon
                        svg {
                            class: "h-5 w-5",
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
                // Current identity section
                div { class: "border-b border-slate-800 px-6 py-4",
                    p { class: "mb-3 text-xs uppercase tracking-wider text-slate-600",
                        "Current Identity"
                    }
                    div { class: "flex items-center justify-between",
                        span { class: "text-sm text-slate-300", "{props.current_four_words}" }
                        if current_has_passkey {
                            button {
                                class: "rounded-lg border border-red-900 px-3 py-1.5 text-xs font-medium text-red-400 hover:bg-red-900/20",
                                onclick: {
                                    let fw = props.current_four_words.clone();
                                    move |_| props.on_delete.call(fw.clone())
                                },
                                "Remove Passkey"
                            }
                        } else {
                            button {
                                class: "rounded-lg border border-emerald-900 px-3 py-1.5 text-xs font-medium text-emerald-400 hover:bg-emerald-900/20",
                                onclick: move |_| props.on_register.call(()),
                                "Enable Passkey"
                            }
                        }
                    }
                    p { class: "mt-2 text-xs text-slate-500",
                        "Passkeys enable biometric authentication for quick identity switching."
                    }
                }
                // Other identities section
                div { class: "max-h-64 overflow-y-auto px-6 py-4",
                    p { class: "mb-3 text-xs uppercase tracking-wider text-slate-600",
                        "Recent Identities"
                    }
                    {props.identities.iter().filter(|i| i.four_words != props.current_four_words).map(|identity| {
                        let fw = identity.four_words.clone();
                        let fw_for_delete = fw.clone();
                        let fw_for_remove = fw.clone();
                        let has_passkey = identity.has_passkey;
                        let display_name = identity.display_name.clone();
                        rsx! {
                            div {
                                key: "{fw}",
                                class: "flex items-center justify-between py-2",
                                div { class: "flex items-center gap-2",
                                    span { class: "text-sm text-slate-300", "{display_name}" }
                                    if has_passkey {
                                        PasskeyBadge {}
                                    }
                                }
                                div { class: "flex items-center gap-2",
                                    if has_passkey {
                                        button {
                                            class: "rounded px-2 py-1 text-xs text-red-400 hover:bg-red-900/20",
                                            title: "Remove passkey",
                                            onclick: move |_| props.on_delete.call(fw_for_delete.clone()),
                                            "Remove Key"
                                        }
                                    }
                                    button {
                                        class: "rounded px-2 py-1 text-xs text-slate-400 hover:bg-slate-800",
                                        title: "Remove from recent list",
                                        onclick: move |_| props.on_remove.call(fw_for_remove.clone()),
                                        // Trash icon
                                        svg {
                                            class: "h-4 w-4",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            view_box: "0 0 24 24",
                                            path {
                                                d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })}
                    if props.identities.iter().filter(|i| i.four_words != props.current_four_words).count() == 0 {
                        p { class: "py-4 text-center text-sm text-slate-500",
                            "No other recent identities"
                        }
                    }
                }
                // Footer
                div { class: "border-t border-slate-800 px-6 py-4",
                    button {
                        class: "w-full rounded-lg border border-slate-700 px-4 py-2 text-sm font-medium text-slate-300 hover:border-slate-600 hover:bg-slate-900",
                        onclick: move |_| props.on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switcher_action_variants() {
        // Test that all variants can be created
        let _ = SwitcherAction::SwitchIdentity("alpha-beta-gamma-delta".to_string());
        let _ = SwitcherAction::RegisterPasskey;
        let _ = SwitcherAction::DeletePasskey("alpha-beta-gamma-delta".to_string());
        let _ = SwitcherAction::RemoveRecent("alpha-beta-gamma-delta".to_string());
        let _ = SwitcherAction::RefreshList;
    }

    #[test]
    fn format_relative_time_just_now() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time(now), "just now");
        assert_eq!(format_relative_time(now - 30), "just now");
    }

    #[test]
    fn format_relative_time_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time(now - 60), "1 min ago");
        assert_eq!(format_relative_time(now - 120), "2 mins ago");
        assert_eq!(format_relative_time(now - 3000), "50 mins ago");
    }

    #[test]
    fn format_relative_time_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time(now - 3600), "1 hour ago");
        assert_eq!(format_relative_time(now - 7200), "2 hours ago");
    }

    #[test]
    fn format_relative_time_days() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time(now - 86400), "yesterday");
        assert_eq!(format_relative_time(now - 172800), "2 days ago");
    }

    #[test]
    fn format_relative_time_weeks() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_relative_time(now - 604800), "1 week ago");
        assert_eq!(format_relative_time(now - 1209600), "2 weeks ago");
    }

    #[test]
    fn keyboard_shortcut_hint_defined() {
        assert!(!KEYBOARD_SHORTCUT_HINT.is_empty());
    }
}
