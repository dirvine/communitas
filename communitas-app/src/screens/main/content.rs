// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Content Screen
//!
//! Main content browser with channels and entity list.

use crate::app::Route;
use crate::services::{Entity, EntityType};
use crate::state::use_app_state;
use dioxus::prelude::*;

/// Content screen component - main dashboard
#[component]
pub fn ContentScreen() -> Element {
    let navigator = use_navigator();
    let app_state = use_app_state();

    // Check authentication
    if !*app_state.is_authenticated.read() {
        let _ = navigator.push(Route::WelcomeScreen {});
        return rsx! { div { "Redirecting..." } };
    }

    let four_words = app_state.four_words.read().clone().unwrap_or_default();
    let display_name = app_state
        .display_name
        .read()
        .clone()
        .unwrap_or_else(|| "User".to_string());

    // Entity list state
    let mut entities = use_signal(Vec::<Entity>::new);
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    // Load entities on mount
    let core = app_state.core.read().clone();
    use_effect(move || {
        let core_clone = core.clone();
        spawn(async move {
            is_loading.set(true);
            match core_clone.list_entities().await {
                Ok(list) => {
                    entities.set(list);
                    error_message.set(None);
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed to load entities: {}", e)));
                }
            }
            is_loading.set(false);
        });
    });

    // Group entities by type
    let entities_read = entities.read();
    let channels: Vec<&Entity> = entities_read
        .iter()
        .filter(|e| e.entity_type == EntityType::Channel)
        .collect();
    let groups: Vec<&Entity> = entities_read
        .iter()
        .filter(|e| e.entity_type == EntityType::Group)
        .collect();
    let organizations: Vec<&Entity> = entities_read
        .iter()
        .filter(|e| e.entity_type == EntityType::Organisation)
        .collect();
    let projects: Vec<&Entity> = entities_read
        .iter()
        .filter(|e| e.entity_type == EntityType::Project)
        .collect();

    rsx! {
        div {
            class: "content-screen",
            style: "display: flex; height: 100vh; background: #f5f5f7;",

            // Sidebar
            div {
                style: "width: 280px; background: #1d1d1f; color: white; display: flex; flex-direction: column;",

                // User info header
                div {
                    style: "padding: 20px; border-bottom: 1px solid #333;",

                    p {
                        style: "font-size: 16px; font-weight: 600; margin-bottom: 4px;",
                        "{display_name}"
                    }
                    p {
                        style: "font-size: 12px; color: #86868b;",
                        "{four_words}"
                    }
                }

                // Navigation items
                nav {
                    style: "flex: 1; padding: 16px; overflow-y: auto;",

                    // Channels section
                    div {
                        style: "margin-bottom: 24px;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                            h3 {
                                style: "font-size: 12px; color: #86868b; text-transform: uppercase; letter-spacing: 0.5px;",
                                "Channels"
                            }
                            button {
                                style: "background: none; border: none; color: #86868b; cursor: pointer; font-size: 16px; padding: 0;",
                                onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "channel".to_string() }); },
                                "+"
                            }
                        }

                        // Default general channel
                        button {
                            style: "width: 100%; text-align: left; padding: 8px 12px; background: #333; border: none; border-radius: 8px; color: white; cursor: pointer; font-size: 14px; margin-bottom: 4px;",
                            onclick: move |_| { navigator.push(Route::ChatScreen { entity_id: "general".to_string() }); },
                            "# general"
                        }

                        // Dynamic channels
                        for channel in channels.iter() {
                            button {
                                key: "{channel.id}",
                                style: "width: 100%; text-align: left; padding: 8px 12px; background: transparent; border: none; border-radius: 8px; color: #ccc; cursor: pointer; font-size: 14px; margin-bottom: 2px;",
                                onclick: {
                                    let id = channel.id.clone();
                                    move |_| { navigator.push(Route::ChatScreen { entity_id: id.clone() }); }
                                },
                                "# {channel.name}"
                            }
                        }
                    }

                    // Groups section
                    div {
                        style: "margin-bottom: 24px;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                            h3 {
                                style: "font-size: 12px; color: #86868b; text-transform: uppercase; letter-spacing: 0.5px;",
                                "Groups"
                            }
                            button {
                                style: "background: none; border: none; color: #86868b; cursor: pointer; font-size: 16px; padding: 0;",
                                onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "group".to_string() }); },
                                "+"
                            }
                        }

                        if groups.is_empty() {
                            p {
                                style: "font-size: 12px; color: #666; padding: 8px;",
                                "No groups yet"
                            }
                        }

                        for group in groups.iter() {
                            button {
                                key: "{group.id}",
                                style: "width: 100%; text-align: left; padding: 8px 12px; background: transparent; border: none; border-radius: 8px; color: #ccc; cursor: pointer; font-size: 14px; margin-bottom: 2px;",
                                onclick: {
                                    let id = group.id.clone();
                                    move |_| { navigator.push(Route::ChatScreen { entity_id: id.clone() }); }
                                },
                                "👥 {group.name}"
                            }
                        }
                    }

                    // Organizations section
                    div {
                        style: "margin-bottom: 24px;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                            h3 {
                                style: "font-size: 12px; color: #86868b; text-transform: uppercase; letter-spacing: 0.5px;",
                                "Organizations"
                            }
                            button {
                                style: "background: none; border: none; color: #86868b; cursor: pointer; font-size: 16px; padding: 0;",
                                onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "organisation".to_string() }); },
                                "+"
                            }
                        }

                        if organizations.is_empty() {
                            p {
                                style: "font-size: 12px; color: #666; padding: 8px;",
                                "No organizations yet"
                            }
                        }

                        for org in organizations.iter() {
                            button {
                                key: "{org.id}",
                                style: "width: 100%; text-align: left; padding: 8px 12px; background: transparent; border: none; border-radius: 8px; color: #ccc; cursor: pointer; font-size: 14px; margin-bottom: 2px;",
                                "🏢 {org.name}"
                            }
                        }
                    }

                    // Projects section
                    div {
                        style: "margin-bottom: 24px;",

                        div {
                            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;",
                            h3 {
                                style: "font-size: 12px; color: #86868b; text-transform: uppercase; letter-spacing: 0.5px;",
                                "Projects"
                            }
                            button {
                                style: "background: none; border: none; color: #86868b; cursor: pointer; font-size: 16px; padding: 0;",
                                onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "project".to_string() }); },
                                "+"
                            }
                        }

                        if projects.is_empty() {
                            p {
                                style: "font-size: 12px; color: #666; padding: 8px;",
                                "No projects yet"
                            }
                        }

                        for project in projects.iter() {
                            button {
                                key: "{project.id}",
                                style: "width: 100%; text-align: left; padding: 8px 12px; background: transparent; border: none; border-radius: 8px; color: #ccc; cursor: pointer; font-size: 14px; margin-bottom: 2px;",
                                "📁 {project.name}"
                            }
                        }
                    }
                }

                // Settings button at bottom
                div {
                    style: "padding: 16px; border-top: 1px solid #333;",

                    button {
                        style: "width: 100%; padding: 12px; background: #333; border: none; border-radius: 8px; color: white; cursor: pointer; font-size: 14px;",
                        onclick: move |_| { navigator.push(Route::SettingsScreen {}); },
                        "⚙️ Settings"
                    }
                }
            }

            // Main content area
            div {
                style: "flex: 1; display: flex; flex-direction: column;",

                // Header
                div {
                    style: "padding: 16px 24px; border-bottom: 1px solid #e5e5ea; background: white;",

                    h1 {
                        style: "font-size: 20px; color: #1d1d1f;",
                        "Welcome to Communitas"
                    }
                }

                // Error message
                if let Some(error) = error_message.read().as_ref() {
                    div {
                        style: "margin: 16px 24px; background: #ffebee; color: #c62828; padding: 12px 16px; border-radius: 8px; font-size: 14px;",
                        "{error}"
                    }
                }

                // Content
                div {
                    style: "flex: 1; padding: 24px; overflow-y: auto;",

                    // Loading indicator
                    if *is_loading.read() {
                        div {
                            style: "text-align: center; padding: 40px;",
                            p { style: "color: #86868b;", "Loading entities..." }
                        }
                    }

                    // Quick create buttons
                    div {
                        style: "display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 32px;",

                        // Create Group button
                        button {
                            style: "padding: 20px; background: white; border: 2px solid #e5e5ea; border-radius: 12px; cursor: pointer; text-align: left;",
                            onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "group".to_string() }); },
                            div {
                                style: "font-size: 24px; margin-bottom: 8px;",
                                "👥"
                            }
                            h3 {
                                style: "font-size: 16px; color: #1d1d1f; margin-bottom: 4px;",
                                "Create Group"
                            }
                            p {
                                style: "font-size: 12px; color: #86868b;",
                                "Team collaboration space"
                            }
                        }

                        // Create Organization button
                        button {
                            style: "padding: 20px; background: white; border: 2px solid #e5e5ea; border-radius: 12px; cursor: pointer; text-align: left;",
                            onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "organisation".to_string() }); },
                            div {
                                style: "font-size: 24px; margin-bottom: 8px;",
                                "🏢"
                            }
                            h3 {
                                style: "font-size: 16px; color: #1d1d1f; margin-bottom: 4px;",
                                "Create Organization"
                            }
                            p {
                                style: "font-size: 12px; color: #86868b;",
                                "Manage teams and projects"
                            }
                        }

                        // Create Project button
                        button {
                            style: "padding: 20px; background: white; border: 2px solid #e5e5ea; border-radius: 12px; cursor: pointer; text-align: left;",
                            onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "project".to_string() }); },
                            div {
                                style: "font-size: 24px; margin-bottom: 8px;",
                                "📁"
                            }
                            h3 {
                                style: "font-size: 16px; color: #1d1d1f; margin-bottom: 4px;",
                                "Create Project"
                            }
                            p {
                                style: "font-size: 12px; color: #86868b;",
                                "Organize work and tasks"
                            }
                        }

                        // Create Channel button
                        button {
                            style: "padding: 20px; background: white; border: 2px solid #e5e5ea; border-radius: 12px; cursor: pointer; text-align: left;",
                            onclick: move |_| { navigator.push(Route::CreateEntityScreen { entity_type: "channel".to_string() }); },
                            div {
                                style: "font-size: 24px; margin-bottom: 8px;",
                                "#"
                            }
                            h3 {
                                style: "font-size: 16px; color: #1d1d1f; margin-bottom: 4px;",
                                "Create Channel"
                            }
                            p {
                                style: "font-size: 12px; color: #86868b;",
                                "Topic-based discussions"
                            }
                        }
                    }

                    // Getting started section
                    div {
                        style: "text-align: center; max-width: 500px; margin: 0 auto; padding: 40px 0;",

                        div {
                            style: "width: 80px; height: 80px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border-radius: 20px; margin: 0 auto 24px; display: flex; align-items: center; justify-content: center;",
                            span { style: "font-size: 36px; color: white;", "C" }
                        }

                        h2 {
                            style: "font-size: 24px; color: #1d1d1f; margin-bottom: 8px;",
                            "Get Started"
                        }

                        p {
                            style: "font-size: 14px; color: #86868b; margin-bottom: 24px;",
                            "Create groups, channels, and projects to collaborate with your team. All data is encrypted and synced peer-to-peer."
                        }

                        button {
                            style: "padding: 12px 24px; background: #007AFF; color: white; border: none; border-radius: 8px; font-size: 14px; cursor: pointer;",
                            onclick: move |_| { navigator.push(Route::ChatScreen { entity_id: "general".to_string() }); },
                            "Open General Channel"
                        }
                    }
                }
            }
        }
    }
}
