// SPDX-License-Identifier: MIT OR Apache-2.0

//! Update UI components for displaying and managing application updates.
//!
//! Provides components for showing update status, available updates, and
//! triggering update downloads/installations.

use dioxus::prelude::*;

/// Props for the UpdateCard component.
#[derive(Props, Clone, PartialEq)]
pub struct UpdateCardProps {
    /// Current application version.
    pub current_version: String,
    /// Whether auto-update is enabled.
    pub auto_update_enabled: bool,
    /// Callback when auto-update toggle is changed.
    pub on_toggle_auto_update: EventHandler<bool>,
    /// Callback when check for updates is clicked.
    pub on_check_updates: EventHandler<()>,
}

/// A card displaying update settings and status.
///
/// Shows the current version, auto-update toggle, and a button to
/// manually check for updates.
#[component]
pub fn UpdateCard(props: UpdateCardProps) -> Element {
    rsx! {
        div {
            class: "update-card",
            role: "region",
            "aria-labelledby": "update-card-title",

            h3 {
                id: "update-card-title",
                class: "update-card-title",
                "Software Updates"
            }

            div {
                class: "update-card-content",

                div {
                    class: "update-version-row",
                    span {
                        class: "update-label",
                        "Current Version:"
                    }
                    span {
                        class: "update-value",
                        "{props.current_version}"
                    }
                }

                div {
                    class: "update-toggle-row",
                    label {
                        class: "update-toggle-label",
                        r#for: "auto-update-toggle",
                        "Automatic Updates"
                    }
                    input {
                        r#type: "checkbox",
                        id: "auto-update-toggle",
                        class: "update-toggle",
                        checked: props.auto_update_enabled,
                        "aria-describedby": "auto-update-description",
                        onchange: move |evt| {
                            props.on_toggle_auto_update.call(evt.checked());
                        }
                    }
                }

                p {
                    id: "auto-update-description",
                    class: "update-description",
                    "When enabled, Communitas will automatically download and install updates in the background."
                }

                button {
                    class: "update-check-button",
                    r#type: "button",
                    onclick: move |_| props.on_check_updates.call(()),
                    "Check for Updates"
                }
            }
        }
    }
}

/// Props for the UpdateAvailableModal component.
#[derive(Props, Clone, PartialEq)]
pub struct UpdateAvailableModalProps {
    /// Whether the modal is visible.
    pub visible: bool,
    /// The new version available.
    pub version: String,
    /// Release notes for the update.
    pub release_notes: String,
    /// When the update was published.
    pub published_at: String,
    /// Callback when "Update Now" is clicked.
    pub on_update: EventHandler<()>,
    /// Callback when "Later" is clicked.
    pub on_later: EventHandler<()>,
    /// Callback when "Skip This Version" is clicked.
    pub on_skip: EventHandler<()>,
}

/// Modal dialog showing an available update.
///
/// Displays version information, release notes, and options to update,
/// defer, or skip the update.
#[component]
pub fn UpdateAvailableModal(props: UpdateAvailableModalProps) -> Element {
    if !props.visible {
        return rsx! {};
    }

    rsx! {
        div {
            class: "update-modal-overlay",
            role: "dialog",
            "aria-modal": "true",
            "aria-labelledby": "update-modal-title",

            div {
                class: "update-modal",

                h2 {
                    id: "update-modal-title",
                    class: "update-modal-title",
                    "Update Available"
                }

                div {
                    class: "update-modal-content",

                    p {
                        class: "update-modal-version",
                        "Version {props.version} is now available."
                    }

                    if !props.published_at.is_empty() {
                        p {
                            class: "update-modal-date",
                            "Published: {props.published_at}"
                        }
                    }

                    if !props.release_notes.is_empty() {
                        div {
                            class: "update-modal-notes",

                            h4 {
                                class: "update-modal-notes-title",
                                "Release Notes"
                            }

                            p {
                                class: "update-modal-notes-content",
                                "{props.release_notes}"
                            }
                        }
                    }
                }

                div {
                    class: "update-modal-actions",

                    button {
                        class: "update-modal-button update-modal-button-primary",
                        r#type: "button",
                        onclick: move |_| props.on_update.call(()),
                        "Update Now"
                    }

                    button {
                        class: "update-modal-button update-modal-button-secondary",
                        r#type: "button",
                        onclick: move |_| props.on_later.call(()),
                        "Later"
                    }

                    button {
                        class: "update-modal-button update-modal-button-tertiary",
                        r#type: "button",
                        onclick: move |_| props.on_skip.call(()),
                        "Skip This Version"
                    }
                }
            }
        }
    }
}

/// Props for the UpdateProgressBar component.
#[derive(Props, Clone, PartialEq)]
pub struct UpdateProgressBarProps {
    /// Download progress from 0.0 to 1.0.
    pub progress: f32,
    /// Optional label to display.
    #[props(default)]
    pub label: Option<String>,
}

/// A progress bar for displaying update download progress.
#[component]
pub fn UpdateProgressBar(props: UpdateProgressBarProps) -> Element {
    let percentage = (props.progress * 100.0).round() as i32;
    let label = props
        .label
        .clone()
        .unwrap_or_else(|| format!("Downloading update... {percentage}%"));

    rsx! {
        div {
            class: "update-progress",
            role: "progressbar",
            "aria-valuenow": "{percentage}",
            "aria-valuemin": "0",
            "aria-valuemax": "100",
            "aria-label": "{label}",

            div {
                class: "update-progress-label",
                "{label}"
            }

            div {
                class: "update-progress-track",

                div {
                    class: "update-progress-fill",
                    style: "width: {percentage}%;"
                }
            }
        }
    }
}

/// Props for the UpdateStatusBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct UpdateStatusBadgeProps {
    /// The status to display.
    pub status: UpdateBadgeStatus,
}

/// Status variants for the update badge.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Will be used when settings UI is integrated
pub enum UpdateBadgeStatus {
    /// Currently checking for updates.
    Checking,
    /// Application is up to date.
    UpToDate,
    /// An update is available.
    UpdateAvailable,
    /// Update is being downloaded.
    Downloading,
    /// Update is ready to install.
    ReadyToInstall,
    /// An error occurred.
    Error,
}

/// A small badge showing update status.
#[component]
pub fn UpdateStatusBadge(props: UpdateStatusBadgeProps) -> Element {
    let (class, text) = match &props.status {
        UpdateBadgeStatus::Checking => ("update-badge-checking", "Checking..."),
        UpdateBadgeStatus::UpToDate => ("update-badge-up-to-date", "Up to date"),
        UpdateBadgeStatus::UpdateAvailable => ("update-badge-available", "Update available"),
        UpdateBadgeStatus::Downloading => ("update-badge-downloading", "Downloading..."),
        UpdateBadgeStatus::ReadyToInstall => ("update-badge-ready", "Ready to install"),
        UpdateBadgeStatus::Error => ("update-badge-error", "Update error"),
    };

    rsx! {
        span {
            class: "update-badge {class}",
            role: "status",
            "{text}"
        }
    }
}
