// SPDX-License-Identifier: MIT OR Apache-2.0

//! Confirmation dialog for destructive actions.

use crate::design_tokens::{palette, radius, semantic, spacing, typography};
use dioxus::prelude::*;

/// Props for the ConfirmDialog component.
#[derive(Props, Clone, PartialEq)]
pub struct ConfirmDialogProps {
    /// Dialog title.
    pub title: String,
    /// Description of what will happen.
    pub message: String,
    /// Text for the confirm button.
    #[props(default = "Confirm".to_string())]
    pub confirm_text: String,
    /// Text for the cancel button.
    #[props(default = "Cancel".to_string())]
    pub cancel_text: String,
    /// Whether this is a destructive action (shows red confirm button).
    #[props(default = false)]
    pub destructive: bool,
    /// Callback when confirmed.
    pub on_confirm: EventHandler<()>,
    /// Callback when cancelled.
    pub on_cancel: EventHandler<()>,
}

/// Modal confirmation dialog for actions that need user verification.
///
/// For destructive actions, set `destructive: true` to show a red confirm button.
///
/// # Example
///
/// ```rust
/// use communitas_dioxus::components::ConfirmDialog;
///
/// rsx! {
///     ConfirmDialog {
///         title: "Delete Organization",
///         message: "Are you sure you want to permanently delete \"Acme Corp\"? This action cannot be undone.",
///         confirm_text: "Delete",
///         cancel_text: "Cancel",
///         destructive: true,
///         on_confirm: move |_| {
///             // Handle deletion
///         },
///         on_cancel: move |_| {
///             // Hide dialog
///         },
///     }
/// }
/// ```
#[component]
pub fn ConfirmDialog(props: ConfirmDialogProps) -> Element {
    let confirm_bg = if props.destructive {
        palette::ROSE_500 // Red for destructive
    } else {
        semantic::ACCENT // Normal accent for non-destructive
    };

    rsx! {
        // Full-screen backdrop
        div {
            style: "position: fixed; inset: 0; z-index: 1000; \
                    display: flex; align-items: center; justify-content: center; \
                    background: rgba(0, 0, 0, 0.5); backdrop-filter: blur(4px);",
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "confirm-title",
            // Click backdrop to cancel
            onclick: move |_| props.on_cancel.call(()),

            // Dialog box (stop propagation so clicking box doesn't close)
            div {
                style: format!(
                    "background: {}; border: 1px solid {}; border-radius: {}; \
                     padding: {}; max-width: 420px; width: 90%; \
                     box-shadow: 0 20px 60px rgba(0,0,0,0.3);",
                    semantic::BG_PRIMARY, semantic::BORDER_DEFAULT,
                    radius::XL, spacing::XL
                ),
                onclick: move |evt| evt.stop_propagation(),

                // Title
                h2 {
                    id: "confirm-title",
                    style: format!(
                        "font-size: {}; font-weight: {}; color: {}; margin: 0 0 {} 0;",
                        typography::SIZE_LG, typography::WEIGHT_SEMIBOLD,
                        semantic::TEXT_PRIMARY, spacing::SM
                    ),
                    "{props.title}"
                }

                // Message
                p {
                    style: format!(
                        "font-size: {}; color: {}; margin: 0 0 {} 0; line-height: 1.5;",
                        typography::SIZE_SM, semantic::TEXT_SECONDARY, spacing::XL
                    ),
                    "{props.message}"
                }

                // Buttons row
                div {
                    style: format!(
                        "display: flex; justify-content: flex-end; gap: {};",
                        spacing::SM
                    ),

                    // Cancel button
                    button {
                        style: format!(
                            "padding: {} {}; border-radius: {}; border: 1px solid {}; \
                             background: transparent; color: {}; cursor: pointer; \
                             font-size: {}; font-weight: {};",
                            spacing::SM, spacing::BASE,
                            radius::MD, semantic::BORDER_DEFAULT,
                            semantic::TEXT_SECONDARY,
                            typography::SIZE_SM, typography::WEIGHT_MEDIUM
                        ),
                        onclick: move |_| props.on_cancel.call(()),
                        "{props.cancel_text}"
                    }

                    // Confirm button
                    button {
                        style: format!(
                            "padding: {} {}; border-radius: {}; border: none; \
                             background: {}; color: white; cursor: pointer; \
                             font-size: {}; font-weight: {};",
                            spacing::SM, spacing::BASE,
                            radius::MD, confirm_bg,
                            typography::SIZE_SM, typography::WEIGHT_MEDIUM
                        ),
                        onclick: move |_| props.on_confirm.call(()),
                        "{props.confirm_text}"
                    }
                }
            }
        }
    }
}
