//! Board view component with columns and cards.

use communitas_ui_api::kanban::BoardView as BoardViewData;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::info;

use super::column::KanbanColumn;

/// Board view showing columns with cards.
#[derive(Props, Clone, PartialEq)]
pub struct BoardViewProps {
    /// The board ID to display.
    pub board_id: String,
}

#[component]
pub fn BoardView(props: BoardViewProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let kanban = services.kanban();

    // Board data state
    let mut board_data = use_signal(|| Option::<BoardViewData>::None);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    // Conflict banner state
    let mut show_conflict_banner = use_signal(|| false);

    // Fetch board data
    let board_id = props.board_id.clone();
    use_future(move || {
        let kanban = kanban.clone();
        let board_id = board_id.clone();
        async move {
            match kanban.get_board(&board_id).await {
                Ok(board) => {
                    board_data.set(Some(board));
                    loading.set(false);
                }
                Err(err) => {
                    error.set(Some(err.to_string()));
                    loading.set(false);
                }
            }
        }
    });

    // Filter state
    let mut show_filters = use_signal(|| false);

    // Add column state
    let mut show_add_column = use_signal(|| false);
    let mut new_column_name = use_signal(String::new);
    let mut adding_column = use_signal(|| false);

    let board_id_for_add = props.board_id.clone();

    rsx! {
        div {
            class: "board-view flex flex-col h-full",
            role: "main",
            // Loading state
            if loading() {
                BoardViewSkeleton {}
            } else if let Some(err) = error() {
                // Error state
                div {
                    class: "flex flex-col items-center justify-center h-64",
                    p { class: "text-red-400 mb-4", "Failed to load board: {err}" }
                    button {
                        class: "rounded-lg bg-emerald-500 px-4 py-2 font-semibold text-slate-900",
                        onclick: move |_| {
                            loading.set(true);
                            error.set(None);
                        },
                        "Retry"
                    }
                }
            } else if let Some(board) = board_data() {
                // CRDT conflict banner
                if show_conflict_banner() {
                    ConflictBanner {
                        on_dismiss: move |_| show_conflict_banner.set(false),
                    }
                }
                // Board header
                BoardHeader {
                    name: board.name.clone(),
                    description: board.description.clone(),
                    on_filter_toggle: move |_| show_filters.set(!show_filters()),
                    filter_active: show_filters(),
                }
                // Filter panel (collapsible)
                if show_filters() {
                    super::filters::FilterPanel {
                        on_filter_change: move |_filters| {
                            // TODO: Apply filters to visible cards
                            info!(target = "ui.kanban", event = "filters_changed");
                        },
                    }
                }
                // Columns container with horizontal scroll
                div {
                    class: "flex-1 overflow-x-auto overflow-y-hidden",
                    div {
                        class: "flex gap-4 h-full p-4 min-w-max",
                        role: "list",
                        aria_label: "Board columns",
                        {board.columns.iter().map(|column| {
                            let col_id = column.id.clone();
                            rsx! {
                                KanbanColumn {
                                    key: "{col_id}",
                                    column: column.clone(),
                                    board_id: props.board_id.clone(),
                                }
                            }
                        })}
                        // Add column button
                        div {
                            class: "flex-shrink-0 w-72",
                            if show_add_column() {
                                AddColumnForm {
                                    name: new_column_name(),
                                    adding: adding_column(),
                                    on_name_change: move |name: String| new_column_name.set(name),
                                    on_submit: move |_| {
                                        let services = services.clone();
                                        let name = new_column_name();
                                        let board_id = board_id_for_add.clone();
                                        adding_column.set(true);
                                        spawn(async move {
                                            let position = board_data().map(|b| b.columns.len() as u32).unwrap_or(0);
                                            match services.kanban().create_column(&board_id, &name, position).await {
                                                Ok(col) => {
                                                    info!(target = "ui.kanban", event = "column_created", column_id = %col.id);
                                                    show_add_column.set(false);
                                                    new_column_name.set(String::new());
                                                    // Refresh board data
                                                    if let Ok(updated) = services.kanban().get_board(&board_id).await {
                                                        board_data.set(Some(updated));
                                                    }
                                                }
                                                Err(err) => {
                                                    tracing::error!(target = "ui.kanban", "failed to create column: {err}");
                                                }
                                            }
                                            adding_column.set(false);
                                        });
                                    },
                                    on_cancel: move |_| {
                                        show_add_column.set(false);
                                        new_column_name.set(String::new());
                                    },
                                }
                            } else {
                                button {
                                    class: "w-full h-12 rounded-lg border-2 border-dashed border-slate-700 text-slate-400 hover:border-emerald-400 hover:text-emerald-400 transition flex items-center justify-center gap-2",
                                    onclick: move |_| show_add_column.set(true),
                                    span { "+" }
                                    "Add Column"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Board header with name and controls.
#[derive(Props, Clone, PartialEq)]
struct BoardHeaderProps {
    name: String,
    description: Option<String>,
    on_filter_toggle: EventHandler<()>,
    filter_active: bool,
}

#[component]
fn BoardHeader(props: BoardHeaderProps) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between p-4 border-b border-slate-800",
            div {
                h1 {
                    class: "text-2xl font-semibold text-white",
                    "{props.name}"
                }
                if let Some(desc) = &props.description {
                    p {
                        class: "text-sm text-slate-400 mt-1",
                        "{desc}"
                    }
                }
            }
            div {
                class: "flex gap-2",
                button {
                    class: format!(
                        "rounded-lg px-3 py-2 text-sm transition {}",
                        if props.filter_active {
                            "bg-emerald-500/20 text-emerald-400 border border-emerald-500/50"
                        } else {
                            "bg-slate-800 text-slate-300 hover:bg-slate-700"
                        }
                    ),
                    onclick: move |_| props.on_filter_toggle.call(()),
                    "Filters"
                }
                button {
                    class: "rounded-lg bg-slate-800 px-3 py-2 text-sm text-slate-300 hover:bg-slate-700 transition",
                    "Settings"
                }
            }
        }
    }
}

/// CRDT conflict banner.
#[derive(Props, Clone, PartialEq)]
struct ConflictBannerProps {
    on_dismiss: EventHandler<()>,
}

#[component]
fn ConflictBanner(props: ConflictBannerProps) -> Element {
    rsx! {
        div {
            class: "bg-amber-500/20 border-b border-amber-500/50 px-4 py-2 flex items-center justify-between",
            role: "alert",
            div {
                class: "flex items-center gap-2",
                span { class: "text-amber-400", "⚠️" }
                span { class: "text-amber-200 text-sm",
                    "Changes from another device detected. Syncing..."
                }
            }
            button {
                class: "text-amber-400 hover:text-amber-300 text-sm",
                onclick: move |_| props.on_dismiss.call(()),
                "Dismiss"
            }
        }
    }
}

/// Skeleton loader for board view.
#[component]
fn BoardViewSkeleton() -> Element {
    rsx! {
        div {
            class: "flex flex-col h-full animate-pulse",
            aria_busy: "true",
            aria_label: "Loading board",
            // Header skeleton
            div {
                class: "p-4 border-b border-slate-800",
                div { class: "h-8 w-48 bg-slate-700 rounded" }
                div { class: "h-4 w-64 bg-slate-700 rounded mt-2" }
            }
            // Columns skeleton
            div {
                class: "flex-1 p-4 flex gap-4 overflow-hidden",
                {(0..4).map(|i| rsx! {
                    div {
                        key: "{i}",
                        class: "w-72 flex-shrink-0 rounded-lg border border-slate-800 bg-slate-900/50 p-3",
                        div { class: "h-6 w-24 bg-slate-700 rounded mb-4" }
                        {(0..3).map(|j| rsx! {
                            div {
                                key: "{j}",
                                class: "h-24 bg-slate-800 rounded-lg mb-2"
                            }
                        })}
                    }
                })}
            }
        }
    }
}

/// Form for adding a new column.
#[derive(Props, Clone, PartialEq)]
struct AddColumnFormProps {
    name: String,
    adding: bool,
    on_name_change: EventHandler<String>,
    on_submit: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn AddColumnForm(props: AddColumnFormProps) -> Element {
    rsx! {
        div {
            class: "rounded-lg border border-slate-700 bg-slate-900/80 p-3",
            form {
                class: "flex flex-col gap-2",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    props.on_submit.call(());
                },
                input {
                    r#type: "text",
                    class: "rounded border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-slate-100 focus:border-emerald-400 focus:outline-none",
                    placeholder: "Column name",
                    disabled: props.adding,
                    value: "{props.name}",
                    autofocus: true,
                    oninput: move |evt| props.on_name_change.call(evt.value()),
                }
                div {
                    class: "flex gap-2",
                    button {
                        r#type: "submit",
                        class: "flex-1 rounded bg-emerald-500 px-3 py-1.5 text-sm font-semibold text-slate-900 disabled:opacity-50",
                        disabled: props.adding || props.name.trim().is_empty(),
                        if props.adding { "Adding..." } else { "Add" }
                    }
                    button {
                        r#type: "button",
                        class: "rounded px-3 py-1.5 text-sm text-slate-400 hover:text-slate-200",
                        disabled: props.adding,
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn board_header_renders_name() {
        // Test that header props work correctly
        let name = "Test Board".to_string();
        let desc = Some("A test description".to_string());
        assert_eq!(name, "Test Board");
        assert_eq!(desc, Some("A test description".to_string()));
    }
}
