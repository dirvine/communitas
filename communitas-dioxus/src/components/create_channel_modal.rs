// SPDX-License-Identifier: MIT OR Apache-2.0

//! Modal for creating a channel in the canonical x0x channel store.

use crate::design_tokens::{motion, radius, semantic, shadow, spacing, typography};
use crate::x0x_contract;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CreateChannelModalProps {
    pub channel_name: String,
    pub channel_description: String,
    pub submitting: bool,
    pub error: Option<String>,
    pub on_name_change: EventHandler<String>,
    pub on_description_change: EventHandler<String>,
    pub on_create: EventHandler<()>,
    pub on_cancel: EventHandler<()>,
}

#[component]
pub fn CreateChannelModal(props: CreateChannelModalProps) -> Element {
    let normalized_name = x0x_contract::normalize_channel_name(&props.channel_name);
    let slug_preview = if normalized_name.is_empty() {
        "Use lowercase letters, numbers, or dashes.".to_string()
    } else {
        format!("Topic slug: #{}", normalized_name)
    };

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
                motion::EASE_OUT
            ),
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "create-channel-modal-title",
            onclick: move |_| props.on_cancel.call(()),

            div {
                style: format!(
                    "width: 100%; \
                     max-width: 480px; \
                     background: {}; \
                     border: 1px solid {}; \
                     border-radius: {}; \
                     box-shadow: {}; \
                     overflow: hidden; \
                     animation: slideInUp {} {};",
                    semantic::BG_SECONDARY,
                    semantic::BORDER_DEFAULT,
                    radius::XL,
                    shadow::XL,
                    motion::SLOW,
                    motion::EASE_OUT
                ),
                onclick: move |evt| evt.stop_propagation(),

                div {
                    style: format!(
                        "padding: {} {}; \
                         border-bottom: 1px solid {};",
                        spacing::BASE,
                        spacing::XL,
                        semantic::BORDER_DEFAULT
                    ),

                    h2 {
                        id: "create-channel-modal-title",
                        style: format!(
                            "margin: 0; \
                             font-size: {}; \
                             font-weight: {}; \
                             color: {};",
                            typography::SIZE_LG,
                            typography::WEIGHT_SEMIBOLD,
                            semantic::TEXT_PRIMARY
                        ),
                        "Create Channel"
                    }

                    p {
                        style: format!(
                            "margin: {} 0 0 0; \
                             font-size: {}; \
                             color: {};",
                            spacing::XS,
                            typography::SIZE_SM,
                            semantic::TEXT_MUTED
                        ),
                        "This writes the frozen x0x-compatible `channels_index` array schema."
                    }
                }

                form {
                    style: format!(
                        "display: flex; \
                         flex-direction: column; \
                         gap: {}; \
                         padding: {} {};",
                        spacing::BASE,
                        spacing::BASE,
                        spacing::XL
                    ),
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        props.on_create.call(());
                    },

                    label {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::XS
                        ),

                        span {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_PRIMARY
                            ),
                            "Channel name"
                        }

                        input {
                            r#type: "text",
                            value: "{props.channel_name}",
                            placeholder: "general-chat",
                            autofocus: true,
                            disabled: props.submitting,
                            style: format!(
                                "width: 100%; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 background: {}; \
                                 color: {}; \
                                 padding: {} {}; \
                                 font-size: {};",
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::BG_TERTIARY,
                                semantic::TEXT_PRIMARY,
                                spacing::SM,
                                spacing::BASE,
                                typography::SIZE_SM
                            ),
                            oninput: move |evt: Event<FormData>| props.on_name_change.call(evt.value().to_string()),
                        }
                    }

                    div {
                        style: format!(
                            "font-size: {}; \
                             color: {};",
                            typography::SIZE_XS,
                            semantic::TEXT_MUTED
                        ),
                        "{slug_preview}"
                    }

                    label {
                        style: format!(
                            "display: flex; \
                             flex-direction: column; \
                             gap: {};",
                            spacing::XS
                        ),

                        span {
                            style: format!(
                                "font-size: {}; \
                                 font-weight: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                typography::WEIGHT_MEDIUM,
                                semantic::TEXT_PRIMARY
                            ),
                            "Description"
                        }

                        textarea {
                            value: "{props.channel_description}",
                            rows: "3",
                            placeholder: "What belongs in this channel?",
                            disabled: props.submitting,
                            style: format!(
                                "width: 100%; \
                                 resize: vertical; \
                                 border: 1px solid {}; \
                                 border-radius: {}; \
                                 background: {}; \
                                 color: {}; \
                                 padding: {} {}; \
                                 font-size: {}; \
                                 line-height: {};",
                                semantic::BORDER_DEFAULT,
                                radius::LG,
                                semantic::BG_TERTIARY,
                                semantic::TEXT_PRIMARY,
                                spacing::SM,
                                spacing::BASE,
                                typography::SIZE_SM,
                                typography::LEADING_NORMAL
                            ),
                            oninput: move |evt: Event<FormData>| props.on_description_change.call(evt.value().to_string()),
                        }
                    }

                    if let Some(ref error) = props.error {
                        div {
                            style: format!(
                                "font-size: {}; \
                                 color: {};",
                                typography::SIZE_SM,
                                semantic::ERROR
                            ),
                            "{error}"
                        }
                    }

                    div {
                        style: format!(
                            "display: flex; \
                             justify-content: flex-end; \
                             gap: {};",
                            spacing::SM
                        ),

                        button {
                            r#type: "button",
                            disabled: props.submitting,
                            style: format!(
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
                                if props.submitting { "not-allowed" } else { "pointer" }
                            ),
                            onclick: move |_| props.on_cancel.call(()),
                            "Cancel"
                        }

                        button {
                            r#type: "submit",
                            disabled: props.submitting || normalized_name.is_empty(),
                            style: format!(
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
                                if props.submitting || normalized_name.is_empty() {
                                    "not-allowed"
                                } else {
                                    "pointer"
                                },
                                if props.submitting || normalized_name.is_empty() {
                                    "0.6"
                                } else {
                                    "1"
                                }
                            ),
                            if props.submitting { "Creating..." } else { "Create Channel" }
                        }
                    }
                }
            }
        }
    }
}
