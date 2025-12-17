// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Create Entity Screen
//!
//! Generic screen for creating groups, organizations, projects, and channels.

use crate::app::Route;
use crate::services::EntityType;
use crate::state::use_app_state;
use dioxus::prelude::*;

/// Create entity screen component
#[component]
pub fn CreateEntityScreen(entity_type: String) -> Element {
    let navigator = use_navigator();
    let app_state = use_app_state();

    // Check authentication
    if !*app_state.is_authenticated.read() {
        let _ = navigator.push(Route::WelcomeScreen {});
        return rsx! { div { "Redirecting..." } };
    }

    // Parse entity type
    let parsed_type = match entity_type.as_str() {
        "group" => EntityType::Group,
        "organisation" | "organization" => EntityType::Organisation,
        "project" => EntityType::Project,
        "channel" => EntityType::Channel,
        _ => EntityType::Group, // Default fallback
    };

    let type_display = match parsed_type {
        EntityType::Group => "Group",
        EntityType::Organisation => "Organization",
        EntityType::Project => "Project",
        EntityType::Channel => "Channel",
        EntityType::Person => "Person",
    };

    let mut name_input = use_signal(String::new);
    let mut description_input = use_signal(String::new);
    let mut is_creating = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);

    let handle_create = {
        let core = app_state.core.read().clone();
        move |_| {
            let name = name_input.read().clone();
            if name.trim().is_empty() {
                error_message.set(Some("Name is required".to_string()));
                return;
            }

            let description = {
                let desc = description_input.read().clone();
                if desc.trim().is_empty() {
                    None
                } else {
                    Some(desc)
                }
            };

            is_creating.set(true);
            error_message.set(None);
            success_message.set(None);

            let core_clone = core.clone();
            let entity_type_clone = parsed_type;
            let name_clone = name.clone();

            spawn(async move {
                match core_clone
                    .create_entity(name_clone.clone(), entity_type_clone, description)
                    .await
                {
                    Ok(entity) => {
                        success_message.set(Some(format!(
                            "Created '{}' successfully! (ID: {})",
                            entity.name,
                            &entity.id[..8]
                        )));
                        name_input.set(String::new());
                        description_input.set(String::new());
                    }
                    Err(e) => {
                        error_message.set(Some(format!("Failed to create: {}", e)));
                    }
                }
                is_creating.set(false);
            });
        }
    };

    rsx! {
        div {
            class: "create-entity-screen",
            style: "min-height: 100vh; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); display: flex; align-items: center; justify-content: center; padding: 20px;",

            div {
                style: "background: white; border-radius: 16px; padding: 40px; max-width: 500px; width: 100%; box-shadow: 0 20px 50px rgba(0,0,0,0.2);",

                // Back button
                button {
                    style: "background: none; border: none; color: #007AFF; cursor: pointer; font-size: 14px; margin-bottom: 20px;",
                    onclick: move |_| { navigator.push(Route::ContentScreen {}); },
                    "← Back to Home"
                }

                // Header
                div {
                    style: "text-align: center; margin-bottom: 32px;",

                    div {
                        style: "width: 64px; height: 64px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border-radius: 16px; margin: 0 auto 16px; display: flex; align-items: center; justify-content: center;",
                        span {
                            style: "font-size: 28px; color: white;",
                            match parsed_type {
                                EntityType::Group => "👥",
                                EntityType::Organisation => "🏢",
                                EntityType::Project => "📁",
                                EntityType::Channel => "#",
                                EntityType::Person => "👤",
                            }
                        }
                    }

                    h1 {
                        style: "font-size: 24px; color: #1d1d1f; margin-bottom: 8px;",
                        "Create {type_display}"
                    }

                    p {
                        style: "font-size: 14px; color: #86868b;",
                        match parsed_type {
                            EntityType::Group => "Create a group for team collaboration",
                            EntityType::Organisation => "Create an organization to manage teams",
                            EntityType::Project => "Create a project to organize work",
                            EntityType::Channel => "Create a channel for discussions",
                            EntityType::Person => "Add a person",
                        }
                    }
                }

                // Error message
                if let Some(error) = error_message.read().as_ref() {
                    div {
                        style: "background: #ffebee; color: #c62828; padding: 12px 16px; border-radius: 8px; margin-bottom: 16px; font-size: 14px;",
                        "{error}"
                    }
                }

                // Success message
                if let Some(success) = success_message.read().as_ref() {
                    div {
                        style: "background: #e8f5e9; color: #2e7d32; padding: 12px 16px; border-radius: 8px; margin-bottom: 16px; font-size: 14px;",
                        "{success}"
                    }
                }

                // Form
                div {
                    style: "display: flex; flex-direction: column; gap: 16px;",

                    // Name input
                    div {
                        label {
                            style: "display: block; font-size: 14px; font-weight: 500; color: #1d1d1f; margin-bottom: 8px;",
                            "{type_display} Name"
                        }
                        input {
                            style: "width: 100%; padding: 12px 16px; border: 2px solid #e5e5ea; border-radius: 8px; font-size: 16px; outline: none; box-sizing: border-box;",
                            r#type: "text",
                            placeholder: "Enter name...",
                            value: "{name_input}",
                            disabled: *is_creating.read(),
                            oninput: move |evt| name_input.set(evt.value()),
                        }
                    }

                    // Description input
                    div {
                        label {
                            style: "display: block; font-size: 14px; font-weight: 500; color: #1d1d1f; margin-bottom: 8px;",
                            "Description (optional)"
                        }
                        textarea {
                            style: "width: 100%; padding: 12px 16px; border: 2px solid #e5e5ea; border-radius: 8px; font-size: 16px; outline: none; min-height: 100px; resize: vertical; box-sizing: border-box; font-family: inherit;",
                            placeholder: "Enter description...",
                            value: "{description_input}",
                            disabled: *is_creating.read(),
                            oninput: move |evt| description_input.set(evt.value()),
                        }
                    }

                    // Create button
                    button {
                        style: if *is_creating.read() {
                            "width: 100%; padding: 14px; background: #ccc; color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: 600; cursor: not-allowed;"
                        } else {
                            "width: 100%; padding: 14px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; border-radius: 8px; font-size: 16px; font-weight: 600; cursor: pointer;"
                        },
                        disabled: *is_creating.read(),
                        onclick: handle_create,
                        if *is_creating.read() {
                            "Creating..."
                        } else {
                            "Create {type_display}"
                        }
                    }
                }
            }
        }
    }
}
