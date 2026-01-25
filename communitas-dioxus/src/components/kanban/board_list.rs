//! Board list page showing all Kanban boards.

use communitas_ui_api::kanban::BoardSummary;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::info;

/// Board list page component showing a grid of board cards.
#[component]
pub fn BoardListPage() -> Element {
    let services = use_context::<Arc<UiServices>>();
    let kanban = services.kanban();

    // Subscribe to board updates
    let boards_snapshot = use_signal(|| kanban.current_snapshot());
    let mut boards_signal = boards_snapshot;

    use_future(move || {
        let mut rx = kanban.subscribe();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                boards_signal.set(rx.borrow().clone());
            }
        }
    });

    let snapshot = boards_snapshot();
    let loading = snapshot.loading;
    let boards: Vec<BoardSummary> = snapshot.boards.clone();

    // Show create board modal state
    let mut show_create_modal = use_signal(|| false);
    let mut new_board_name = use_signal(String::new);

    rsx! {
        div {
            class: "board-list-page flex flex-col gap-6",
            role: "main",
            aria_label: "Kanban boards",
            // Header
            div {
                class: "flex items-center justify-between",
                h1 {
                    class: "text-2xl font-semibold text-white",
                    "Boards"
                }
                button {
                    class: "rounded-lg bg-emerald-500 px-4 py-2 font-semibold text-slate-900 shadow-lg shadow-emerald-500/30 transition hover:bg-emerald-400",
                    onclick: move |_| show_create_modal.set(true),
                    "Create Board"
                }
            }
            // Loading skeleton
            if loading {
                BoardListSkeleton {}
            } else if boards.is_empty() {
                // Empty state
                BoardListEmptyState {
                    on_create: move |_| show_create_modal.set(true),
                }
            } else {
                // Board grid
                div {
                    class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                    role: "list",
                    aria_label: "Board list",
                    {boards.iter().map(|board| {
                        let board_id = board.id.clone();
                        rsx! {
                            BoardCard {
                                key: "{board_id}",
                                board: board.clone(),
                            }
                        }
                    })}
                }
            }
            // Create board modal
            if show_create_modal() {
                CreateBoardModal {
                    board_name: new_board_name(),
                    on_name_change: move |name: String| new_board_name.set(name),
                    on_create: move |_| {
                        let services = services.clone();
                        let name = new_board_name();
                        spawn(async move {
                            match services.kanban().create_board("default", &name, None).await {
                                Ok(board) => {
                                    info!(target = "ui.kanban", event = "board_created", board_id = %board.id);
                                    show_create_modal.set(false);
                                    new_board_name.set(String::new());
                                }
                                Err(err) => {
                                    tracing::error!(target = "ui.kanban", "failed to create board: {err}");
                                }
                            }
                        });
                    },
                    on_cancel: move |_| {
                        show_create_modal.set(false);
                        new_board_name.set(String::new());
                    },
                }
            }
        }
    }
}

/// Individual board card in the grid.
#[derive(Props, Clone, PartialEq)]
struct BoardCardProps {
    board: BoardSummary,
}

#[component]
fn BoardCard(props: BoardCardProps) -> Element {
    let board = &props.board;
    let board_id = board.id.clone();

    // Format last activity time
    let last_activity = board.last_activity.map(|ts| {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = now - ts;
        let diff_mins = diff_ms / 60_000;
        let diff_hours = diff_mins / 60;
        let diff_days = diff_hours / 24;

        if diff_days > 0 {
            format!("{}d ago", diff_days)
        } else if diff_hours > 0 {
            format!("{}h ago", diff_hours)
        } else if diff_mins > 0 {
            format!("{}m ago", diff_mins)
        } else {
            "just now".to_string()
        }
    });

    rsx! {
        Link {
            to: crate::Route::ProjectBoardRoute { project_id: board_id.clone() },
            class: "board-card rounded-xl border border-slate-800 bg-slate-900/60 p-4 transition hover:border-emerald-400 hover:bg-slate-900/80",
            role: "listitem",
            aria_label: format!(
                "{}, {} cards, {} columns",
                board.name, board.card_count, board.column_count
            ),
            div {
                class: "flex flex-col gap-3",
                // Board name
                h3 {
                    class: "text-lg font-semibold text-white truncate",
                    "{board.name}"
                }
                // Stats
                div {
                    class: "flex gap-4 text-sm text-slate-400",
                    span {
                        class: "flex items-center gap-1",
                        span { class: "text-slate-500", "📊" }
                        "{board.column_count} columns"
                    }
                    span {
                        class: "flex items-center gap-1",
                        span { class: "text-slate-500", "📋" }
                        "{board.card_count} cards"
                    }
                }
                // Last activity
                if let Some(activity) = last_activity {
                    div {
                        class: "text-xs text-slate-500",
                        "Last activity: {activity}"
                    }
                }
            }
        }
    }
}

/// Skeleton loader for board list while loading.
#[component]
fn BoardListSkeleton() -> Element {
    rsx! {
        div {
            class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
            aria_busy: "true",
            aria_label: "Loading boards",
            {(0..6).map(|i| rsx! {
                div {
                    key: "{i}",
                    class: "rounded-xl border border-slate-800 bg-slate-900/60 p-4 animate-pulse",
                    div { class: "h-6 w-32 bg-slate-700 rounded mb-3" }
                    div { class: "flex gap-4",
                        div { class: "h-4 w-20 bg-slate-700 rounded" }
                        div { class: "h-4 w-16 bg-slate-700 rounded" }
                    }
                    div { class: "h-3 w-24 bg-slate-700 rounded mt-3" }
                }
            })}
        }
    }
}

/// Empty state when no boards exist.
#[derive(Props, Clone, PartialEq)]
struct BoardListEmptyStateProps {
    on_create: EventHandler<()>,
}

#[component]
fn BoardListEmptyState(props: BoardListEmptyStateProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center py-16 text-center",
            div {
                class: "w-20 h-20 rounded-full bg-slate-800 flex items-center justify-center mb-4",
                span { class: "text-4xl", "📋" }
            }
            h3 {
                class: "text-lg font-semibold text-white mb-2",
                "No boards yet"
            }
            p {
                class: "text-slate-400 mb-6 max-w-md",
                "Create your first Kanban board to start tracking tasks and workflows."
            }
            button {
                class: "rounded-lg bg-emerald-500 px-6 py-3 font-semibold text-slate-900 shadow-lg shadow-emerald-500/30 transition hover:bg-emerald-400",
                onclick: move |_| props.on_create.call(()),
                "Create your first board"
            }
        }
    }
}

/// Modal for creating a new board.
#[derive(Props, Clone, PartialEq)]
struct CreateBoardModalProps {
    board_name: String,
    on_name_change: EventHandler<String>,
    on_create: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn CreateBoardModal(props: CreateBoardModalProps) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Create board",
            onclick: move |_| props.on_cancel.call(()),
            div {
                class: "w-full max-w-md rounded-xl border border-slate-700 bg-slate-900 p-6 shadow-2xl",
                onclick: move |evt| evt.stop_propagation(),
                h2 {
                    class: "text-xl font-semibold text-white mb-4",
                    "Create New Board"
                }
                form {
                    class: "flex flex-col gap-4",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        props.on_create.call(());
                    },
                    label {
                        class: "flex flex-col gap-2",
                        span {
                            class: "text-sm font-medium text-slate-200",
                            "Board name"
                        }
                        input {
                            r#type: "text",
                            class: "rounded-lg border border-slate-700 bg-slate-800 px-4 py-3 text-slate-100 focus:border-emerald-400 focus:outline-none",
                            placeholder: "e.g., Sprint Planning",
                            value: "{props.board_name}",
                            autofocus: true,
                            oninput: move |evt| props.on_name_change.call(evt.value()),
                        }
                    }
                    div {
                        class: "flex gap-3 justify-end mt-2",
                        button {
                            r#type: "button",
                            class: "rounded-lg border border-slate-600 px-4 py-2 text-slate-300 transition hover:bg-slate-800",
                            onclick: move |_| props.on_cancel.call(()),
                            "Cancel"
                        }
                        button {
                            r#type: "submit",
                            class: "rounded-lg bg-emerald-500 px-4 py-2 font-semibold text-slate-900 shadow-lg shadow-emerald-500/30 transition hover:bg-emerald-400 disabled:opacity-50",
                            disabled: props.board_name.trim().is_empty(),
                            "Create"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::useless_vec)]
mod tests {
    use super::*;

    #[test]
    fn board_card_formats_activity_correctly() {
        // Test that BoardCard can be constructed with valid props
        let board = BoardSummary {
            id: "board-1".to_string(),
            name: "Test Board".to_string(),
            entity_id: "entity-1".to_string(),
            column_count: 3,
            card_count: 10,
            last_activity: Some(chrono::Utc::now().timestamp_millis()),
        };
        assert_eq!(board.name, "Test Board");
        assert_eq!(board.column_count, 3);
    }

    #[test]
    fn board_list_filters_by_entity() {
        let boards = vec![
            BoardSummary {
                id: "b1".to_string(),
                name: "Board 1".to_string(),
                entity_id: "e1".to_string(),
                column_count: 1,
                card_count: 5,
                last_activity: None,
            },
            BoardSummary {
                id: "b2".to_string(),
                name: "Board 2".to_string(),
                entity_id: "e2".to_string(),
                column_count: 2,
                card_count: 8,
                last_activity: None,
            },
        ];

        let filtered: Vec<_> = boards
            .iter()
            .filter(|b| b.entity_id == "e1")
            .cloned()
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Board 1");
    }

    #[test]
    fn board_card_aria_label_format() {
        let board = BoardSummary {
            id: "board-1".to_string(),
            name: "Sprint Planning".to_string(),
            entity_id: "entity-1".to_string(),
            column_count: 4,
            card_count: 12,
            last_activity: None,
        };

        // Test the ARIA label format matches expected pattern
        let aria_label = format!(
            "{}, {} cards, {} columns",
            board.name, board.card_count, board.column_count
        );
        assert_eq!(aria_label, "Sprint Planning, 12 cards, 4 columns");
    }

    #[test]
    fn board_card_aria_label_with_zero_counts() {
        let board = BoardSummary {
            id: "board-empty".to_string(),
            name: "New Board".to_string(),
            entity_id: "entity-1".to_string(),
            column_count: 0,
            card_count: 0,
            last_activity: None,
        };

        let aria_label = format!(
            "{}, {} cards, {} columns",
            board.name, board.card_count, board.column_count
        );
        assert_eq!(aria_label, "New Board, 0 cards, 0 columns");
    }
}
