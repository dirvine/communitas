//! Modal for creating a new space (group) or joining an existing one via invite link.

use crate::design_tokens::{motion, radius, semantic, shadow, spacing, typography};
use dioxus::prelude::*;

/// Which tab is active in the modal.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpaceModalTab {
    /// Create a new space.
    Create,
    /// Join an existing space via invite link.
    Join,
}

/// Props for the create/join space modal.
#[derive(Props, Clone, PartialEq)]
pub struct CreateSpaceModalProps {
    /// Currently active tab.
    pub active_tab: SpaceModalTab,
    /// Space name input (Create tab).
    pub space_name: String,
    /// Space description input (Create tab).
    pub space_description: String,
    /// Invite link input (Join tab).
    pub invite_link: String,
    /// Display name input (used for both create and join).
    pub display_name: String,
    /// Whether a submission is in progress.
    pub submitting: bool,
    /// Error message to display.
    pub error: Option<String>,
    /// Called when the active tab changes.
    pub on_tab_change: EventHandler<SpaceModalTab>,
    /// Called when space name changes.
    pub on_name_change: EventHandler<String>,
    /// Called when space description changes.
    pub on_description_change: EventHandler<String>,
    /// Called when invite link changes.
    pub on_invite_change: EventHandler<String>,
    /// Called when display name changes.
    pub on_display_name_change: EventHandler<String>,
    /// Called when Create is clicked.
    pub on_create: EventHandler<()>,
    /// Called when Join is clicked.
    pub on_join: EventHandler<()>,
    /// Called when the modal should close.
    pub on_cancel: EventHandler<()>,
}

/// Modal overlay for creating a new space or joining via invite link.
#[component]
pub fn CreateSpaceModal(props: CreateSpaceModalProps) -> Element {
    let tab = props.active_tab;

    rsx! {
        div {
            style: format!(
                "position: fixed; \
                 inset: 0; \
                 z-index: 1000; \
                 display: flex; \
                 align-items: center; \
                 justify-content: center; \
                 background: rgba(0, 0, 0, 0.7); \
                 backdrop-filter: blur(4px); \
                 animation: fadeIn {} {};",
                motion::NORMAL,
                motion::EASE_OUT,
            ),
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "create-space-modal-title",
            onclick: move |_| props.on_cancel.call(()),

            div {
                style: format!(
                    "width: 100%; \
                     max-width: 480px; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     box-shadow: {}; \
                     overflow: hidden;",
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    shadow::XL,
                ),
                onclick: move |evt| evt.stop_propagation(),

                // Header with title
                div {
                    style: format!(
                        "padding: {} {}; \
                         border-bottom: 1px solid {};",
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT,
                    ),

                    h2 {
                        id: "create-space-modal-title",
                        style: format!(
                            "margin: 0; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_LG,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY,
                        ),
                        if tab == SpaceModalTab::Create { "Create Space" } else { "Join Space" }
                    }
                }

                // Tab bar
                div {
                    style: format!(
                        "display: flex; \
                         border-bottom: 1px solid {};",
                        semantic::BORDER_DEFAULT,
                    ),

                    for (t, label) in [(SpaceModalTab::Create, "Create"), (SpaceModalTab::Join, "Join")] {
                        {
                            let is_active = tab == t;
                            rsx! {
                                button {
                                    key: "{label}",
                                    style: format!(
                                        "flex: 1; \
                                         padding: {} {}; \
                                         border: none; \
                                         background: none; \
                                         color: {}; \
                                         font-size: {}; \
                                         font-weight: {}; \
                                         border-bottom: 2px solid {}; \
                                         cursor: pointer; \
                                         transition: {};",
                                        spacing::SM,
                                        spacing::BASE,
                                        if is_active { semantic::PRIMARY } else { semantic::TEXT_MUTED },
                                        typography::SIZE_SM,
                                        if is_active { typography::WEIGHT_SEMIBOLD } else { typography::WEIGHT_NORMAL },
                                        if is_active { semantic::PRIMARY } else { "transparent" },
                                        motion::transition("color, border-color"),
                                    ),
                                    onclick: move |_| props.on_tab_change.call(t),
                                    "{label}"
                                }
                            }
                        }
                    }
                }

                // Body
                form {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {}; \
                         padding: {} {};",
                        spacing::BASE,
                        spacing::BASE,
                        spacing::XL,
                    ),
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        if tab == SpaceModalTab::Create {
                            props.on_create.call(());
                        } else {
                            props.on_join.call(());
                        }
                    },

                    if tab == SpaceModalTab::Create {
                        // Space name
                        label {
                            style: format!(
                                "display: flex; \
                                 flex-direction: column; \
                                 gap: {};",
                                spacing::XS,
                            ),

                            span {
                                style: format!(
                                    "font-size: {}; \
                                     font-weight: {}; \
                                     color: {};",
                                    typography::SIZE_SM,
                                    typography::WEIGHT_MEDIUM,
                                    semantic::TEXT_PRIMARY,
                                ),
                                "Space name"
                            }

                            input {
                                r#type: "text",
                                value: "{props.space_name}",
                                placeholder: "My team space",
                                autofocus: true,
                                disabled: props.submitting,
                                style: input_style(),
                                oninput: move |evt: Event<FormData>| props.on_name_change.call(evt.value().to_string()),
                            }
                        }

                        // Description
                        label {
                            style: format!(
                                "display: flex; \
                                 flex-direction: column; \
                                 gap: {};",
                                spacing::XS,
                            ),

                            span {
                                style: format!(
                                    "font-size: {}; \
                                     font-weight: {}; \
                                     color: {};",
                                    typography::SIZE_SM,
                                    typography::WEIGHT_MEDIUM,
                                    semantic::TEXT_PRIMARY,
                                ),
                                "Description (optional)"
                            }

                            textarea {
                                value: "{props.space_description}",
                                rows: "3",
                                placeholder: "What is this space for?",
                                disabled: props.submitting,
                                style: textarea_style(),
                                oninput: move |evt: Event<FormData>| props.on_description_change.call(evt.value().to_string()),
                            }
                        }
                    } else {
                        // Join tab - invite link input
                        label {
                            style: format!(
                                "display: flex; \
                                 flex-direction: column; \
                                 gap: {};",
                                spacing::XS,
                            ),

                            span {
                                style: format!(
                                    "font-size: {}; \
                                     font-weight: {}; \
                                     color: {};",
                                    typography::SIZE_SM,
                                    typography::WEIGHT_MEDIUM,
                                    semantic::TEXT_PRIMARY,
                                ),
                                "Invite link"
                            }

                            input {
                                r#type: "text",
                                value: "{props.invite_link}",
                                placeholder: "Paste invite link here",
                                autofocus: true,
                                disabled: props.submitting,
                                style: input_style(),
                                oninput: move |evt: Event<FormData>| props.on_invite_change.call(evt.value().to_string()),
                            }
                        }
                    }

                    // Display name (shared between tabs)
                    label {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::XS,
                        ),

                        span {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_PRIMARY,
                            ),
                            "Your display name (optional)"
                        }

                        input {
                            r#type: "text",
                            value: "{props.display_name}",
                            placeholder: "How others see you",
                            disabled: props.submitting,
                            style: input_style(),
                            oninput: move |evt: Event<FormData>| props.on_display_name_change.call(evt.value().to_string()),
                        }
                    }

                    // Error message
                    if let Some(ref error) = props.error {
                        div {
                            style: format!(
                                "font-size: {}; color: {};",
                                typography::SIZE_SM,
                                semantic::ERROR,
                            ),
                            "{error}"
                        }
                    }

                    // Actions
                    div {
                        style: format!(
                            "display: flex; \
                             justify-content: flex-end; \
                             gap: {};",
                            spacing::SM,
                        ),

                        button {
                            r#type: "button",
                            disabled: props.submitting,
                            style: cancel_button_style(props.submitting),
                            onclick: move |_| props.on_cancel.call(()),
                            "Cancel"
                        }

                        button {
                            r#type: "submit",
                            disabled: props.submitting || !can_submit(tab, &props.space_name, &props.invite_link),
                            style: submit_button_style(
                                props.submitting || !can_submit(tab, &props.space_name, &props.invite_link),
                            ),
                            if props.submitting {
                                if tab == SpaceModalTab::Create { "Creating..." } else { "Joining..." }
                            } else if tab == SpaceModalTab::Create {
                                "Create Space"
                            } else {
                                "Join Space"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Check if the form can be submitted.
fn can_submit(tab: SpaceModalTab, space_name: &str, invite_link: &str) -> bool {
    match tab {
        SpaceModalTab::Create => !space_name.trim().is_empty(),
        SpaceModalTab::Join => !invite_link.trim().is_empty(),
    }
}

/// Shared input field style.
fn input_style() -> String {
    format!(
        "width: 100%; \
         border: 1px solid {}; \
         border-radius: {}; \
         background: {}; \
         color: {}; \
         padding: {} {}; \
         font-size: {}; \
         outline: none;",
        semantic::BORDER_DEFAULT,
        radius::LG,
        semantic::BG_TERTIARY,
        semantic::TEXT_PRIMARY,
        spacing::SM,
        spacing::BASE,
        typography::SIZE_SM,
    )
}

/// Shared textarea style.
fn textarea_style() -> String {
    format!(
        "width: 100%; \
         resize: vertical; \
         border: 1px solid {}; \
         border-radius: {}; \
         background: {}; \
         color: {}; \
         padding: {} {}; \
         font-size: {}; \
         line-height: {}; \
         outline: none;",
        semantic::BORDER_DEFAULT,
        radius::LG,
        semantic::BG_TERTIARY,
        semantic::TEXT_PRIMARY,
        spacing::SM,
        spacing::BASE,
        typography::SIZE_SM,
        typography::LEADING_NORMAL,
    )
}

/// Cancel button style.
fn cancel_button_style(disabled: bool) -> String {
    format!(
        "border: 1px solid {}; \
         border-radius: {}; \
         background: transparent; \
         color: {}; \
         padding: {} {}; \
         font-size: {}; \
         cursor: {};",
        semantic::BORDER_DEFAULT,
        radius::LG,
        semantic::TEXT_SECONDARY,
        spacing::SM,
        spacing::BASE,
        typography::SIZE_SM,
        if disabled { "not-allowed" } else { "pointer" },
    )
}

/// Submit button style.
fn submit_button_style(disabled: bool) -> String {
    format!(
        "border: none; \
         border-radius: {}; \
         background: {}; \
         color: {}; \
         padding: {} {}; \
         font-size: {}; \
         font-weight: {}; \
         cursor: {}; \
         opacity: {};",
        radius::LG,
        semantic::PRIMARY,
        semantic::TEXT_INVERSE,
        spacing::SM,
        spacing::BASE,
        typography::SIZE_SM,
        typography::WEIGHT_SEMIBOLD,
        if disabled { "not-allowed" } else { "pointer" },
        if disabled { "0.6" } else { "1" },
    )
}
