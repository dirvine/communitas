// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kanban board backed by x0x task list API.
//!
//! Three columns: To Do, In Progress, Done. Tasks are loaded via
//! `GET /task-lists/:id/tasks` and actions (`add`, `claim`, `complete`)
//! are mapped to the corresponding task-list API endpoints.
//!
//! The board-to-task-list mapping is persisted in a KV store keyed by
//! group prefix, matching the Swift BoardView pattern.

use dioxus::prelude::*;
use tracing::warn;

use communitas_x0x_client::{Task, X0xClient};

use crate::tokens::{colors, spacing, typography};
use crate::x0x_contract;

/// Polling interval for task list refresh (seconds).
const POLL_SECS: u64 = 5;

/// Board view props.
#[derive(Props, Clone, PartialEq)]
pub struct BoardViewProps {
    /// The group/space ID that owns this board.
    pub group_id: String,
}

/// Board view — 3-column kanban backed by x0x task lists.
#[component]
pub fn BoardView(props: BoardViewProps) -> Element {
    let group_id = props.group_id.clone();
    let prefix = x0x_contract::group_prefix(&group_id).to_owned();

    let mut tasks = use_signal(Vec::<Task>::new);
    let mut list_id = use_signal(|| None::<String>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);
    let mut new_task_title = use_signal(String::new);
    let mut refresh_key = use_signal(|| 0u64);

    // ── Initialization: find or create the task list for this board ──
    let init_prefix = prefix.clone();
    use_future(move || {
        let prefix = init_prefix.clone();
        async move {
            let client = X0xClient::new();
            let board_store = format!("x0x-board-{prefix}");
            let kv_key = format!("board.{prefix}.listId");

            // Ensure board store exists
            if let Err(e) = client.create_store(&board_store, &board_store).await {
                let msg = format!("{e}");
                if !msg.contains("409") && !msg.contains("already") && !msg.contains("exists") {
                    warn!(target: "ui.board", "failed to create board store: {e}");
                }
            }

            // Look up existing task list ID from KV
            let resolved_id = match client.get(&board_store, &kv_key).await {
                Ok(value) => {
                    let stored = String::from_utf8_lossy(
                        &base64::Engine::decode(
                            &base64::engine::general_purpose::STANDARD,
                            &value.value,
                        )
                        .unwrap_or_default(),
                    )
                    .trim()
                    .to_owned();
                    if stored.is_empty() {
                        None
                    } else {
                        Some(stored)
                    }
                }
                Err(_) => None,
            };

            let id = match resolved_id {
                Some(id) => id,
                None => {
                    // Create new task list and persist its ID
                    let topic = format!("x0x.group.{prefix}.board/tasks");
                    match client.create_task_list("Board", &topic).await {
                        Ok(created) => {
                            let new_id = created.id.clone();
                            if let Err(e) = client
                                .put(&board_store, &kv_key, new_id.as_bytes(), Some("text/plain"))
                                .await
                            {
                                warn!(target: "ui.board", "failed to persist board list ID: {e}");
                            }
                            new_id
                        }
                        Err(e) => {
                            error.set(Some(format!("Failed to create board: {e}")));
                            loading.set(false);
                            return;
                        }
                    }
                }
            };

            list_id.set(Some(id.clone()));

            // Initial task load
            match client.list_tasks(&id).await {
                Ok(t) => tasks.set(t),
                Err(e) => warn!(target: "ui.board", "initial task load failed: {e}"),
            }
            loading.set(false);
        }
    });

    // ── Polling: refresh tasks every POLL_SECS ──
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(POLL_SECS)).await;
            let _key = *refresh_key.read();
            if let Some(id) = list_id.read().clone() {
                let client = X0xClient::new();
                if let Ok(t) = client.list_tasks(&id).await {
                    tasks.set(t);
                }
            }
        }
    });

    // ── Render ──
    let current_tasks = tasks.read().clone();
    let is_loading = loading();
    let current_error = error.read().clone();

    if is_loading {
        return rsx! {
            div {
                style: "display: flex; align-items: center; justify-content: center; height: 200px; color: {colors::TEXT_MUTED};",
                "Loading board..."
            }
        };
    }

    if let Some(err) = current_error {
        return rsx! {
            div {
                style: "padding: {spacing::LG}; color: {colors::DANGER};",
                "Error: {err}"
            }
        };
    }

    let todo: Vec<_> = current_tasks
        .iter()
        .filter(|t| t.state.as_deref() == Some("todo") || t.state.is_none())
        .cloned()
        .collect();
    let in_progress: Vec<_> = current_tasks
        .iter()
        .filter(|t| t.state.as_deref() == Some("in_progress"))
        .cloned()
        .collect();
    let done: Vec<_> = current_tasks
        .iter()
        .filter(|t| t.state.as_deref() == Some("done"))
        .cloned()
        .collect();

    rsx! {
        div {
            style: "display: flex; flex-direction: column; height: 100%; padding: {spacing::MD};",

            // ── Add task bar ──
            div {
                style: "display: flex; gap: {spacing::SM}; margin-bottom: {spacing::MD};",
                input {
                    style: "flex: 1; padding: {spacing::SM} {spacing::MD}; border: 1px solid {colors::BORDER_DEFAULT}; border-radius: 6px; background: {colors::SURFACE_CARD}; color: {colors::TEXT_PRIMARY}; font-size: {typography::TEXT_SM};",
                    r#type: "text",
                    placeholder: "New task...",
                    value: "{new_task_title}",
                    oninput: move |evt: FormEvent| new_task_title.set(evt.value()),
                    onkeypress: move |evt: KeyboardEvent| {
                        if evt.key() == Key::Enter {
                            let title = new_task_title().trim().to_string();
                            if !title.is_empty() {
                                new_task_title.set(String::new());
                                spawn(async move {
                                    if let Some(id) = list_id.read().clone() {
                                        let client = X0xClient::new();
                                        if let Err(e) = client.add_task(&id, &title, None).await {
                                            warn!(target: "ui.board", "add task failed: {e}");
                                        }
                                        refresh_key.set(refresh_key() + 1);
                                    }
                                });
                            }
                        }
                    },
                }
                button {
                    style: "padding: {spacing::SM} {spacing::LG}; background: {colors::PRIMARY}; color: white; border: none; border-radius: 6px; cursor: pointer; font-size: {typography::TEXT_SM};",
                    onclick: move |_| {
                        let title = new_task_title().trim().to_string();
                        if !title.is_empty() {
                            new_task_title.set(String::new());
                            spawn(async move {
                                if let Some(id) = list_id.read().clone() {
                                    let client = X0xClient::new();
                                    if let Err(e) = client.add_task(&id, &title, None).await {
                                        warn!(target: "ui.board", "add task failed: {e}");
                                    }
                                    refresh_key.set(refresh_key() + 1);
                                }
                            });
                        }
                    },
                    "Add"
                }
            }

            // ── Three-column board ──
            div {
                style: "display: flex; gap: {spacing::MD}; flex: 1; overflow: auto;",

                // To Do
                {board_column("To Do", colors::TEXT_MUTED, &todo, list_id, refresh_key, "claim")}

                // In Progress
                {board_column("In Progress", "#e67e22", &in_progress, list_id, refresh_key, "complete")}

                // Done
                {board_column("Done", colors::SUCCESS, &done, list_id, refresh_key, "")}
            }
        }
    }
}

/// Render a single board column with task cards and action buttons.
fn board_column(
    title: &str,
    accent: &str,
    tasks: &[Task],
    list_id: Signal<Option<String>>,
    refresh_key: Signal<u64>,
    action: &str,
) -> Element {
    let action_label = match action {
        "claim" => Some("Start"),
        "complete" => Some("Done"),
        _ => None,
    };
    let action_str = action.to_owned();

    rsx! {
        div {
            style: "flex: 1; min-width: 220px; background: {colors::SURFACE_CARD}; border-radius: 8px; padding: {spacing::MD}; display: flex; flex-direction: column;",

            // Column header
            div {
                style: "display: flex; align-items: center; gap: {spacing::SM}; margin-bottom: {spacing::MD};",
                div {
                    style: "width: 10px; height: 10px; border-radius: 50%; background: {accent};",
                }
                span {
                    style: "font-weight: 600; font-size: {typography::TEXT_SM}; color: {colors::TEXT_PRIMARY};",
                    "{title}"
                }
                span {
                    style: "color: {colors::TEXT_MUTED}; font-size: {typography::TEXT_XS}; margin-left: auto;",
                    "{tasks.len()}"
                }
            }

            // Task cards
            div {
                style: "display: flex; flex-direction: column; gap: {spacing::SM}; overflow-y: auto; flex: 1;",
                if tasks.is_empty() {
                    div {
                        style: "text-align: center; color: {colors::TEXT_MUTED}; font-size: {typography::TEXT_XS}; padding: {spacing::LG} 0;",
                        "No tasks"
                    }
                }
                for task in tasks.iter() {
                    {task_card(task, list_id, refresh_key, &action_str, action_label)}
                }
            }
        }
    }
}

/// Render a single task card.
fn task_card(
    task: &Task,
    list_id: Signal<Option<String>>,
    mut refresh_key: Signal<u64>,
    action: &str,
    action_label: Option<&str>,
) -> Element {
    let task_id = task.id.clone();
    let task_title = task.title.clone();
    let task_desc = task.description.clone();
    let id_prefix = if task_id.len() > 8 {
        &task_id[..8]
    } else {
        &task_id
    };
    let action_owned = action.to_owned();

    rsx! {
        div {
            style: "background: {colors::SURFACE_BG}; border: 1px solid {colors::BORDER_DEFAULT}; border-radius: 6px; padding: {spacing::SM} {spacing::MD};",

            div {
                style: "font-size: {typography::TEXT_SM}; color: {colors::TEXT_PRIMARY}; font-weight: 500; margin-bottom: 2px;",
                "{task_title}"
            }

            if let Some(desc) = &task_desc {
                div {
                    style: "font-size: {typography::TEXT_XS}; color: {colors::TEXT_MUTED}; margin-bottom: {spacing::XS};",
                    "{desc}"
                }
            }

            div {
                style: "display: flex; align-items: center; justify-content: space-between;",
                span {
                    style: "font-size: 10px; color: {colors::TEXT_MUTED}; font-family: monospace;",
                    "{id_prefix}"
                }
                if let Some(label) = action_label {
                    button {
                        style: "font-size: {typography::TEXT_XS}; padding: 2px {spacing::SM}; background: {colors::SURFACE_CARD}; border: 1px solid {colors::BORDER_DEFAULT}; border-radius: 4px; cursor: pointer; color: {colors::TEXT_PRIMARY};",
                        onclick: {
                            let tid = task_id.clone();
                            let act = action_owned.clone();
                            move |_| {
                                let tid = tid.clone();
                                let act = act.clone();
                                spawn(async move {
                                    if let Some(lid) = list_id.read().clone() {
                                        let client = X0xClient::new();
                                        let result = if act == "claim" {
                                            client.claim_task(&lid, &tid).await
                                        } else {
                                            client.complete_task(&lid, &tid).await
                                        };
                                        if let Err(e) = result {
                                            warn!(target: "ui.board", "{act} task failed: {e}");
                                        }
                                        refresh_key.set(refresh_key() + 1);
                                    }
                                });
                            }
                        },
                        "{label}"
                    }
                }
            }
        }
    }
}
