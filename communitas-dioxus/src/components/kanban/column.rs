//! Kanban column component with cards and drop zone.

use communitas_ui_api::kanban::ColumnView;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::info;

use super::card::KanbanCard;

/// Kanban column showing a list of cards.
#[derive(Props, Clone, PartialEq)]
pub struct KanbanColumnProps {
    /// The column data to display.
    pub column: ColumnView,
    /// The board ID this column belongs to.
    pub board_id: String,
    /// Callback to announce card moves for screen readers.
    pub on_move_announce: EventHandler<String>,
}

#[component]
pub fn KanbanColumn(props: KanbanColumnProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let column = &props.column;

    // Column menu state
    let mut show_menu = use_signal(|| false);

    // Add card state
    let mut show_add_card = use_signal(|| false);
    let mut new_card_title = use_signal(String::new);
    let mut adding_card = use_signal(|| false);

    // Drag state for drop zone
    let mut is_drag_over = use_signal(|| false);

    let column_id = column.id.clone();
    let column_id_for_drop = column_id.clone();
    let column_id_for_add = column_id.clone();
    let board_id = props.board_id.clone();
    let board_id_for_add = board_id.clone();
    let card_count = column.cards.len();

    // WIP limit indicator
    let wip_exceeded = column
        .wip_limit
        .map(|limit| card_count > limit as usize)
        .unwrap_or(false);

    rsx! {
        div {
            class: format!(
                "kanban-column w-72 flex-shrink-0 rounded-lg border bg-slate-900/50 flex flex-col max-h-full {}",
                if is_drag_over() { "border-emerald-400 bg-emerald-400/5" }
                else if wip_exceeded { "border-amber-500/50" }
                else { "border-slate-800" }
            ),
            role: "listitem",
            aria_label: format!("Column: {}", column.name),
            aria_dropeffect: if is_drag_over() { "move" } else { "none" },
            // Drop zone handlers
            ondragover: move |evt| {
                evt.prevent_default();
                is_drag_over.set(true);
            },
            ondragleave: move |_| {
                is_drag_over.set(false);
            },
            ondrop: move |_evt| {
                is_drag_over.set(false);
                // TODO: Handle card drop - get card ID from drag data
                info!(target = "ui.kanban", event = "card_dropped", column_id = %column_id_for_drop);
            },
            // Column header
            div {
                class: "flex items-center justify-between p-3 border-b border-slate-800",
                div {
                    class: "flex items-center gap-2",
                    h3 {
                        class: "font-semibold text-white truncate",
                        role: "heading",
                        aria_level: "3",
                        aria_label: format!(
                            "{} column, {} cards",
                            column.name, card_count
                        ),
                        "{column.name}"
                    }
                    span {
                        class: format!(
                            "text-xs px-1.5 py-0.5 rounded {}",
                            if wip_exceeded { "bg-amber-500/20 text-amber-400" }
                            else { "bg-slate-700 text-slate-400" }
                        ),
                        "{card_count}"
                        if let Some(limit) = column.wip_limit {
                            " / {limit}"
                        }
                    }
                }
                // Column menu button
                div {
                    class: "relative",
                    button {
                        class: "p-1 text-slate-400 hover:text-white rounded hover:bg-slate-700 transition",
                        onclick: move |_| show_menu.set(!show_menu()),
                        "⋮"
                    }
                    if show_menu() {
                        ColumnMenu {
                            on_rename: move |_| {
                                info!(target = "ui.kanban", event = "column_rename_clicked");
                                show_menu.set(false);
                            },
                            on_set_wip: move |_| {
                                info!(target = "ui.kanban", event = "column_wip_clicked");
                                show_menu.set(false);
                            },
                            on_delete: move |_| {
                                info!(target = "ui.kanban", event = "column_delete_clicked");
                                show_menu.set(false);
                            },
                            on_close: move |_| show_menu.set(false),
                        }
                    }
                }
            }
            // Cards list
            div {
                class: "flex-1 overflow-y-auto p-2 space-y-2",
                role: "list",
                aria_label: "Cards in column",
                {column.cards.iter().map(|card| {
                    let card_id = card.id.clone();
                    rsx! {
                        KanbanCard {
                            key: "{card_id}",
                            card: card.clone(),
                            board_id: board_id.clone(),
                            column_id: column_id.clone(),
                            on_move_announce: props.on_move_announce,
                        }
                    }
                })}
                // Drop placeholder when dragging over empty column
                if is_drag_over() && column.cards.is_empty() {
                    div {
                        class: "h-24 rounded-lg border-2 border-dashed border-emerald-400/50 bg-emerald-400/5 flex items-center justify-center",
                        span { class: "text-emerald-400/70 text-sm", "Drop card here" }
                    }
                }
            }
            // Add card section
            div {
                class: "p-2 border-t border-slate-800",
                if show_add_card() {
                    AddCardForm {
                        title: new_card_title(),
                        adding: adding_card(),
                        on_title_change: move |title: String| new_card_title.set(title),
                        on_submit: move |_| {
                            let services = services.clone();
                            let title = new_card_title();
                            let board_id = board_id_for_add.clone();
                            let column_id = column_id_for_add.clone();
                            adding_card.set(true);
                            spawn(async move {
                                match services.kanban().create_card(&board_id, &column_id, &title).await {
                                    Ok(card) => {
                                        info!(target = "ui.kanban", event = "card_created", card_id = %card.id);
                                        show_add_card.set(false);
                                        new_card_title.set(String::new());
                                    }
                                    Err(err) => {
                                        tracing::error!(target = "ui.kanban", "failed to create card: {err}");
                                    }
                                }
                                adding_card.set(false);
                            });
                        },
                        on_cancel: move |_| {
                            show_add_card.set(false);
                            new_card_title.set(String::new());
                        },
                    }
                } else {
                    button {
                        class: "w-full py-2 text-sm text-slate-400 hover:text-emerald-400 hover:bg-slate-800/50 rounded transition flex items-center justify-center gap-1",
                        onclick: move |_| show_add_card.set(true),
                        span { "+" }
                        "Add card"
                    }
                }
            }
        }
    }
}

/// Column context menu.
#[derive(Props, Clone, PartialEq)]
struct ColumnMenuProps {
    on_rename: EventHandler<()>,
    on_set_wip: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_close: EventHandler<()>,
}

#[component]
fn ColumnMenu(props: ColumnMenuProps) -> Element {
    rsx! {
        div {
            class: "absolute right-0 top-full mt-1 w-40 rounded-lg border border-slate-700 bg-slate-800 shadow-lg z-10",
            role: "menu",
            div {
                class: "py-1",
                button {
                    class: "w-full px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700",
                    role: "menuitem",
                    onclick: move |_| props.on_rename.call(()),
                    "Rename"
                }
                button {
                    class: "w-full px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700",
                    role: "menuitem",
                    onclick: move |_| props.on_set_wip.call(()),
                    "Set WIP limit"
                }
                hr { class: "my-1 border-slate-700" }
                button {
                    class: "w-full px-3 py-2 text-left text-sm text-red-400 hover:bg-slate-700",
                    role: "menuitem",
                    onclick: move |_| props.on_delete.call(()),
                    "Delete column"
                }
            }
        }
    }
}

/// Form for adding a new card.
#[derive(Props, Clone, PartialEq)]
struct AddCardFormProps {
    title: String,
    adding: bool,
    on_title_change: EventHandler<String>,
    on_submit: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn AddCardForm(props: AddCardFormProps) -> Element {
    rsx! {
        form {
            class: "flex flex-col gap-2",
            onsubmit: move |evt| {
                evt.prevent_default();
                props.on_submit.call(());
            },
            textarea {
                class: "w-full rounded border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-slate-100 resize-none focus:border-emerald-400 focus:outline-none",
                placeholder: "Card title",
                rows: "2",
                disabled: props.adding,
                value: "{props.title}",
                autofocus: true,
                oninput: move |evt| props.on_title_change.call(evt.value()),
            }
            div {
                class: "flex gap-2",
                button {
                    r#type: "submit",
                    class: "flex-1 rounded bg-emerald-500 px-3 py-1.5 text-sm font-semibold text-slate-900 disabled:opacity-50",
                    disabled: props.adding || props.title.trim().is_empty(),
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

#[cfg(test)]
mod tests {
    #[test]
    fn column_wip_exceeded_detection() {
        let wip_limit = Some(3u32);
        let card_count = 5usize;
        let wip_exceeded = wip_limit
            .map(|limit| card_count > limit as usize)
            .unwrap_or(false);
        assert!(wip_exceeded);

        let card_count = 2;
        let wip_exceeded = wip_limit
            .map(|limit| card_count > limit as usize)
            .unwrap_or(false);
        assert!(!wip_exceeded);
    }

    #[test]
    fn column_header_aria_label_format() {
        let column_name = "To Do";
        let card_count = 5usize;

        let aria_label = format!("{} column, {} cards", column_name, card_count);
        assert_eq!(aria_label, "To Do column, 5 cards");
    }

    #[test]
    fn column_header_aria_label_empty_column() {
        let column_name = "Done";
        let card_count = 0usize;

        let aria_label = format!("{} column, {} cards", column_name, card_count);
        assert_eq!(aria_label, "Done column, 0 cards");
    }

    #[test]
    fn column_dropeffect_state() {
        // When dragging over, aria-dropeffect should be "move"
        let is_drag_over = true;
        let dropeffect = if is_drag_over { "move" } else { "none" };
        assert_eq!(dropeffect, "move");

        // When not dragging, aria-dropeffect should be "none"
        let is_drag_over = false;
        let dropeffect = if is_drag_over { "move" } else { "none" };
        assert_eq!(dropeffect, "none");
    }
}
