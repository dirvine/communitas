// SPDX-License-Identifier: MIT OR Apache-2.0

//! Space view with tab bar (Chat, Board, Files, Swarm, Feed).
//!
//! Each space is a group. The Chat tab shows channel sidebar + channel chat.
//! Board reuses the kanban system. Files uses the files view. Swarm provides
//! agent task delegation. Feed is a social post stream per space.

use dioxus::prelude::*;
use tracing::info;

use crate::components;
use crate::models;
use crate::tokens::{colors, spacing, typography};

/// Active tab within a space.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SpaceTab {
    #[default]
    Chat,
    Board,
    Files,
    Swarm,
    Feed,
    Wiki,
    Web,
    /// Owner/admin panel for policy, roster, and join requests.
    Manage,
}

/// Props for the space view.
#[derive(Props, Clone, PartialEq)]
pub struct SpaceViewProps {
    /// The group/space ID.
    pub space_id: String,
    /// Optional initial tab (defaults to Chat).
    #[props(default)]
    pub initial_tab: Option<String>,
}

/// Space view component with tab bar.
#[component]
pub fn SpaceView(props: SpaceViewProps) -> Element {
    let mut active_tab = use_signal(|| match props.initial_tab.as_deref() {
        Some("board") => SpaceTab::Board,
        Some("files") => SpaceTab::Files,
        Some("swarm") => SpaceTab::Swarm,
        Some("feed") => SpaceTab::Feed,
        Some("wiki") => SpaceTab::Wiki,
        Some("web") => SpaceTab::Web,
        Some("manage") => SpaceTab::Manage,
        _ => SpaceTab::Chat,
    });

    let mut selected_channel =
        use_signal(|| Option::<components::channel_sidebar::SelectedChannel>::None);
    let mut thread_message = use_signal(|| Option::<models::channel::ChatMessage>::None);
    let reply_count_bump = use_signal(|| Option::<(String, u64)>::None);
    let mut channel_sidebar_refresh = use_signal(|| 0u64);
    let mut create_channel_group_id = use_signal(|| Option::<String>::None);
    let mut create_channel_name = use_signal(String::new);
    let mut create_channel_description = use_signal(String::new);
    let mut create_channel_submitting = use_signal(|| false);
    let mut create_channel_error = use_signal(|| None::<String>);

    let current_tab = *active_tab.read();
    let space_id = props.space_id.clone();

    let container_style = format!(
        "display: flex; flex-direction: column; height: 100%; overflow: hidden; \
         background-color: {};",
        colors::SURFACE_BG,
    );

    let tab_bar_style = format!(
        "display: flex; gap: 0; border-bottom: 1px solid {}; flex-shrink: 0; \
         padding: 0 {};",
        colors::BORDER_DEFAULT,
        spacing::MD,
    );

    let tab_btn = |tab: SpaceTab, _label: &str| -> String {
        let active = current_tab == tab;
        format!(
            "padding: {} {}; font-size: {}; font-weight: {}; \
             color: {}; background: none; border: none; \
             border-bottom: 2px solid {}; cursor: pointer; \
             transition: color 150ms ease, border-color 150ms ease;",
            spacing::SM,
            spacing::MD,
            typography::TEXT_SM,
            if active { "600" } else { "400" },
            if active {
                colors::PRIMARY
            } else {
                colors::TEXT_MUTED
            },
            if active {
                colors::PRIMARY
            } else {
                "transparent"
            },
        )
    };

    let content_style = "flex: 1; overflow: hidden; display: flex;";

    rsx! {
        div {
            style: "{container_style}",

            // Tab bar
            div {
                style: "{tab_bar_style}",
                role: "tablist",

                button {
                    style: tab_btn(SpaceTab::Chat, "Chat"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Chat,
                    onclick: move |_| active_tab.set(SpaceTab::Chat),
                    "Chat"
                }
                button {
                    style: tab_btn(SpaceTab::Board, "Board"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Board,
                    onclick: move |_| active_tab.set(SpaceTab::Board),
                    "Board"
                }
                button {
                    style: tab_btn(SpaceTab::Files, "Files"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Files,
                    onclick: move |_| active_tab.set(SpaceTab::Files),
                    "Files"
                }
                button {
                    style: tab_btn(SpaceTab::Swarm, "Swarm"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Swarm,
                    onclick: move |_| active_tab.set(SpaceTab::Swarm),
                    "Swarm"
                }
                button {
                    style: tab_btn(SpaceTab::Feed, "Feed"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Feed,
                    onclick: move |_| active_tab.set(SpaceTab::Feed),
                    "Feed"
                }
                button {
                    style: tab_btn(SpaceTab::Wiki, "Wiki"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Wiki,
                    onclick: move |_| active_tab.set(SpaceTab::Wiki),
                    "Wiki"
                }
                button {
                    style: tab_btn(SpaceTab::Web, "Web"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Web,
                    onclick: move |_| active_tab.set(SpaceTab::Web),
                    "Web"
                }
                button {
                    style: tab_btn(SpaceTab::Manage, "Manage"),
                    role: "tab",
                    "aria-selected": current_tab == SpaceTab::Manage,
                    onclick: move |_| active_tab.set(SpaceTab::Manage),
                    "Manage"
                }
            }

            // Tab content
            div {
                style: "{content_style}",
                role: "tabpanel",

                match current_tab {
                    SpaceTab::Chat => rsx! {
                        // Channel sidebar + chat
                        div {
                            style: "width: 220px; flex-shrink: 0; border-right: 1px solid rgba(37, 41, 64, 0.8);",
                            components::ChannelSidebar {
                                refresh_key: channel_sidebar_refresh(),
                                selected: selected_channel(),
                                on_select: move |ch: components::channel_sidebar::SelectedChannel| {
                                    selected_channel.set(Some(ch));
                                    thread_message.set(None);
                                },
                                on_create_channel: move |group_id: String| {
                                    create_channel_group_id.set(Some(group_id));
                                    create_channel_name.set(String::new());
                                    create_channel_description.set(String::new());
                                    create_channel_error.set(None);
                                },
                                on_open_identity: move |_| {},
                            }
                        }

                        div {
                            style: "flex: 1; display: flex; overflow: hidden;",

                            if let Some(channel) = selected_channel() {
                                div {
                                    style: "flex: 1; display: flex; flex-direction: column; overflow: hidden;",
                                    components::ChannelChatView {
                                        key: "{channel.group_id}:{channel.channel_name}",
                                        channel: channel.clone(),
                                        on_open_thread: move |msg: models::channel::ChatMessage| {
                                            thread_message.set(Some(msg));
                                        },
                                        reply_count_bump: reply_count_bump,
                                    }
                                }

                                if let Some(ref parent_msg) = thread_message() {
                                    if let Some(ref ch) = selected_channel() {
                                        components::ThreadPanel {
                                            key: "{ch.group_id}:{ch.channel_name}:{parent_msg.id}",
                                            parent_message: parent_msg.clone(),
                                            channel: ch.clone(),
                                            on_close: move |_| {
                                                thread_message.set(None);
                                            },
                                            reply_count_bump: reply_count_bump,
                                        }
                                    }
                                }
                            } else {
                                div {
                                    style: format!(
                                        "flex: 1; display: flex; align-items: center; justify-content: center; \
                                         color: {};",
                                        colors::TEXT_MUTED,
                                    ),
                                    "Select a channel to start chatting"
                                }
                            }
                        }

                        // Create channel modal
                        if let Some(group_id) = create_channel_group_id() {
                            components::CreateChannelModal {
                                channel_name: create_channel_name(),
                                channel_description: create_channel_description(),
                                submitting: create_channel_submitting(),
                                error: create_channel_error(),
                                on_name_change: move |value: String| create_channel_name.set(value),
                                on_description_change: move |value: String| create_channel_description.set(value),
                                on_cancel: move |_| {
                                    if create_channel_submitting() { return; }
                                    create_channel_group_id.set(None);
                                    create_channel_error.set(None);
                                },
                                on_create: {
                                    let _space_id = space_id.clone();
                                    move |_| {
                                        if create_channel_submitting() { return; }

                                        let group_id = group_id.clone();
                                        let raw_name = create_channel_name();
                                        let raw_description = create_channel_description();

                                        create_channel_submitting.set(true);
                                        create_channel_error.set(None);

                                        spawn(async move {
                                            match crate::create_channel(&group_id, &raw_name, &raw_description).await {
                                                Ok(channel) => {
                                                    info!(
                                                        target: "ui.space",
                                                        "Created channel {} in group {}",
                                                        channel.name, group_id
                                                    );
                                                    selected_channel.set(Some(
                                                        components::channel_sidebar::SelectedChannel {
                                                            group_id: group_id.clone(),
                                                            channel_name: channel.name.clone(),
                                                            topic: channel.topic.clone(),
                                                            meta: Some(channel),
                                                        },
                                                    ));
                                                    thread_message.set(None);
                                                    channel_sidebar_refresh.set(channel_sidebar_refresh() + 1);
                                                    create_channel_group_id.set(None);
                                                }
                                                Err(err) => create_channel_error.set(Some(err)),
                                            }
                                            create_channel_submitting.set(false);
                                        });
                                    }
                                },
                            }
                        }
                    },

                    SpaceTab::Board => {
                        let gid = space_id.clone();
                        rsx! {
                            div {
                                style: "flex: 1; overflow: auto;",
                                components::board_view::BoardView { group_id: gid }
                            }
                        }
                    },

                    SpaceTab::Files => {
                        let sid = space_id.clone();
                        rsx! {
                            div {
                                style: "flex: 1; overflow: auto;",
                                crate::components::files_view::FilesView {
                                    space_id: sid,
                                }
                            }
                        }
                    }

                    SpaceTab::Swarm => {
                        let sid = space_id.clone();
                        rsx! {
                            crate::components::swarm_view::SwarmView {
                                space_id: sid,
                            }
                        }
                    }

                    SpaceTab::Feed => {
                        let sid = space_id.clone();
                        rsx! {
                            crate::components::feed_view::FeedView {
                                space_id: sid,
                            }
                        }
                    }

                    SpaceTab::Wiki => {
                        let sid = space_id.clone();
                        rsx! {
                            crate::components::WikiView {
                                group_id: sid,
                            }
                        }
                    }

                    SpaceTab::Web => {
                        let sid = space_id.clone();
                        rsx! {
                            crate::components::WebView {
                                group_id: sid,
                            }
                        }
                    }

                    SpaceTab::Manage => {
                        let sid = space_id.clone();
                        rsx! {
                            div {
                                style: "flex: 1; overflow: auto;",
                                components::SpaceAdminPanel {
                                    space_id: sid,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
