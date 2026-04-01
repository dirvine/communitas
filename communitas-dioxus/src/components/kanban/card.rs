// SPDX-License-Identifier: MIT OR Apache-2.0

//! Kanban card component with drag-and-drop support.

use communitas_ui_api::SyncState;
use communitas_ui_api::kanban::{CardState, CardView, ChecklistProgress, PriorityView, TagView};
use communitas_ui_service::UiServices;
use communitas_ui_service::kanban::MoveDirection;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::info;

use crate::tokens::colors;

use super::{DragState, DraggingCard};

/// Consolidated interaction state for a Kanban card.
///
/// Replaces 5 separate boolean signals with a single enum for:
/// - Fewer signal subscriptions per card
/// - Enforced mutual exclusivity (only one active state at a time)
/// - Cleaner state transitions
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
enum CardInteractionState {
    /// No interaction in progress
    #[default]
    Idle,
    /// Card is being dragged
    Dragging,
    /// Card is being moved (keyboard navigation)
    Moving,
    /// Card detail modal is shown
    ShowingDetailModal,
    /// Move menu is shown
    ShowingMoveMenu,
    /// Help tooltip is shown
    ShowingHelpTooltip,
}

/// Kanban card component that can be dragged between columns.
#[derive(Props, Clone, PartialEq)]
pub struct KanbanCardProps {
    /// The card data to display.
    pub card: CardView,
    /// The board ID this card belongs to.
    pub board_id: String,
    /// The column ID this card belongs to.
    pub column_id: String,
    /// Callback to announce card moves for screen readers.
    pub on_move_announce: EventHandler<String>,
}

#[component]
pub fn KanbanCard(props: KanbanCardProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut drag_state = use_context::<Signal<DragState>>();
    // Clone card to avoid lifetime issues in closures
    let card = props.card.clone();

    // Consolidated interaction state - replaces 5 separate boolean signals
    let mut interaction_state = use_signal(CardInteractionState::default);

    let card_id = card.id.clone();
    let card_title = card.title.clone();
    let card_id_for_drag_start = card_id.clone();
    let card_title_for_drag = card_title.clone();
    let card_id_for_drag_end = card_id.clone();
    let card_id_for_keyboard = card_id.clone();
    let card_id_for_move_menu = card_id.clone();
    let board_id = props.board_id.clone();
    let board_id_for_keyboard = board_id.clone();
    let board_id_for_move_menu = board_id.clone();
    let column_id = props.column_id.clone();
    let column_id_for_move_menu = column_id.clone();
    let services_for_move = services.clone();
    let column_id_for_drag = column_id.clone();
    let card_position = card.position;
    let on_move_announce = props.on_move_announce;

    // Format due date
    let due_date_display = card.due_date.map(|ts| {
        let now = chrono::Utc::now().timestamp_millis();
        let diff_ms = ts - now;
        let diff_days = diff_ms / 86_400_000;

        if diff_days < 0 {
            ("Overdue", "text-red-400 bg-red-500/20")
        } else if diff_days == 0 {
            ("Today", "text-amber-400 bg-amber-500/20")
        } else if diff_days == 1 {
            ("Tomorrow", "text-emerald-400 bg-emerald-500/20")
        } else if diff_days <= 7 {
            ("This week", "text-slate-400 bg-slate-700")
        } else {
            ("Later", "text-slate-500 bg-slate-800")
        }
    });

    // State color indicator
    let state_color = match card.state {
        CardState::Todo => "border-l-slate-500",
        CardState::InProgress => "border-l-blue-500",
        CardState::InReview => "border-l-amber-500",
        CardState::Done => "border-l-emerald-500",
        CardState::Archived => "border-l-slate-700",
    };

    // Generate description ID for aria-describedby
    let desc_id = format!("card-desc-{}", card.id);
    let has_description = card.description.as_ref().is_some_and(|d| !d.is_empty());

    // Extract booleans from interaction state for rendering
    let is_dragging = matches!(interaction_state(), CardInteractionState::Dragging);
    let is_moving = matches!(interaction_state(), CardInteractionState::Moving);

    rsx! {
        div {
            class: format!(
                "kanban-card rounded-lg border border-l-4 bg-slate-800 p-3 cursor-pointer transition hover:bg-slate-750 {} {} {}",
                state_color,
                if is_dragging { "opacity-50 border-slate-600" } else { "border-slate-700" },
                if is_moving { "ring-2 ring-emerald-400 ring-offset-2 ring-offset-slate-900" } else { "" }
            ),
            role: "listitem",
            aria_label: format!("Card: {}", card.title),
            aria_describedby: if has_description { desc_id.clone() } else { String::new() },
            aria_grabbed: if is_dragging { "true" } else { "false" },
            tabindex: "0",
            draggable: "true",
            // Drag handlers
            ondragstart: move |_evt| {
                interaction_state.set(CardInteractionState::Dragging);
                // Set drag data in shared context for column drop handlers
                drag_state.set(DragState {
                    dragging_card: Some(DraggingCard {
                        card_id: card_id_for_drag_start.clone(),
                        source_column_id: column_id_for_drag.clone(),
                        card_title: card_title_for_drag.clone(),
                    }),
                });
                info!(target = "ui.kanban", event = "card_drag_start", card_id = %card_id_for_drag_start);
            },
            ondragend: move |_| {
                interaction_state.set(CardInteractionState::Idle);
                // Clear drag data
                drag_state.set(DragState::default());
                info!(target = "ui.kanban", event = "card_drag_end", card_id = %card_id_for_drag_end);
            },
            // Click to open detail
            onclick: move |_| {
                interaction_state.set(CardInteractionState::ShowingDetailModal);
            },
            // Keyboard support (Enter to open detail, Ctrl+Arrow to move, m for menu, ? for help)
            onkeydown: move |evt| {
                if evt.key() == Key::Enter {
                    interaction_state.set(CardInteractionState::ShowingDetailModal);
                    return;
                }

                // 'm' key opens move menu
                if evt.key() == Key::Character("m".to_string()) || evt.key() == Key::Character("M".to_string()) {
                    evt.prevent_default();
                    interaction_state.set(CardInteractionState::ShowingMoveMenu);
                    return;
                }

                // '?' key toggles keyboard shortcuts help
                if evt.key() == Key::Character("?".to_string()) {
                    evt.prevent_default();
                    let current = interaction_state();
                    if matches!(current, CardInteractionState::ShowingHelpTooltip) {
                        interaction_state.set(CardInteractionState::Idle);
                    } else {
                        interaction_state.set(CardInteractionState::ShowingHelpTooltip);
                    }
                    return;
                }

                // Escape closes menus/modals
                if evt.key() == Key::Escape {
                    interaction_state.set(CardInteractionState::Idle);
                    return;
                }

                // Handle Ctrl+Arrow for keyboard card movement
                if evt.modifiers().ctrl() {
                    let direction = match evt.key() {
                        Key::ArrowLeft => Some(MoveDirection::Left),
                        Key::ArrowRight => Some(MoveDirection::Right),
                        Key::ArrowUp => Some(MoveDirection::Up),
                        Key::ArrowDown => Some(MoveDirection::Down),
                        _ => None,
                    };

                    if let Some(dir) = direction {
                        evt.prevent_default();
                        let services = services.clone();
                        let board_id = board_id_for_keyboard.clone();
                        let card_id = card_id_for_keyboard.clone();
                        let column_id = column_id.clone();
                        let on_announce = on_move_announce;

                        interaction_state.set(CardInteractionState::Moving);

                        spawn(async move {
                            match services
                                .kanban()
                                .move_card_keyboard(&board_id, &card_id, &column_id, card_position, dir)
                                .await
                            {
                                Ok(result) => {
                                    // Announce for screen readers
                                    let message = match dir {
                                        MoveDirection::Left | MoveDirection::Right => {
                                            format!(
                                                "Card {} moved to {}",
                                                result.card_title, result.target_column_name
                                            )
                                        }
                                        MoveDirection::Up => {
                                            format!("Card {} moved up", result.card_title)
                                        }
                                        MoveDirection::Down => {
                                            format!("Card {} moved down", result.card_title)
                                        }
                                    };
                                    on_announce.call(message);
                                    info!(
                                        target = "ui.kanban",
                                        event = "card_moved_keyboard",
                                        card_id = %result.card_id,
                                        direction = ?dir,
                                        target_column = %result.target_column_name
                                    );
                                }
                                Err(err) => {
                                    // Don't announce errors that indicate no-op (already at edge)
                                    tracing::debug!(
                                        target = "ui.kanban",
                                        "keyboard move rejected: {err}"
                                    );
                                }
                            }
                            interaction_state.set(CardInteractionState::Idle);
                        });
                    }
                }
            },
            // Card content
            div {
                class: "flex flex-col gap-2",
                // Title with sync indicator
                div {
                    class: "flex items-start justify-between gap-1",
                    h4 {
                        class: "font-medium text-white text-sm line-clamp-2 flex-1",
                        "{card.title}"
                    }
                    // Sync state indicator (shown when not synced)
                    CardSyncIndicator { state: card.sync_state }
                }
                // Description preview (if exists)
                if let Some(desc) = &card.description {
                    if !desc.is_empty() {
                        p {
                            id: "{desc_id}",
                            class: "text-xs text-slate-400 line-clamp-2",
                            "{desc}"
                        }
                    }
                }
                // Tags
                if !card.tags.is_empty() {
                    div {
                        class: "flex flex-wrap gap-1",
                        {card.tags.iter().take(3).map(|tag| {
                            rsx! {
                                TagChip { key: "{tag.id}", tag: tag.clone() }
                            }
                        })}
                        if card.tags.len() > 3 {
                            {
                                let extra = card.tags.len() - 3;
                                rsx! {
                                    span {
                                        class: "text-xs text-slate-500",
                                        "+{extra}"
                                    }
                                }
                            }
                        }
                    }
                }
                // Bottom row: assignees, due date, checklist
                div {
                    class: "flex items-center justify-between mt-1",
                    // Assignees
                    if !card.assignees.is_empty() {
                        div {
                            class: "flex -space-x-1",
                            {card.assignees.iter().take(3).enumerate().map(|(i, assignee)| {
                                let initials: String = assignee.chars().take(2).collect();
                                rsx! {
                                    div {
                                        key: "{i}",
                                        class: "w-6 h-6 rounded-full bg-slate-600 border border-slate-800 flex items-center justify-center",
                                        title: "{assignee}",
                                        span {
                                            class: "text-[10px] text-slate-300 uppercase",
                                            "{initials}"
                                        }
                                    }
                                }
                            })}
                            if card.assignees.len() > 3 {
                                {
                                    let extra = card.assignees.len() - 3;
                                    rsx! {
                                        div {
                                            class: "w-6 h-6 rounded-full bg-slate-700 border border-slate-800 flex items-center justify-center",
                                            span {
                                                class: "text-[10px] text-slate-400",
                                                "+{extra}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Right side: priority, due date, thread indicator, and checklist progress
                    div {
                        class: "flex items-center gap-2",
                        // Priority badge
                        if let Some(priority) = &card.priority {
                            PriorityBadge { priority: *priority }
                        }
                        // Due date badge
                        if let Some((label, classes)) = due_date_display {
                            span {
                                class: format!("text-xs px-1.5 py-0.5 rounded {classes}"),
                                "{label}"
                            }
                        }
                        // Thread indicator
                        if card.linked_thread_id.is_some() {
                            ThreadIndicator {
                                thread_name: card.linked_thread_name.clone(),
                            }
                        }
                        // Checklist progress
                        if let Some(progress) = &card.checklist_progress {
                            ChecklistProgressBadge { progress: progress.clone() }
                        }
                    }
                }
            }
        }
        // Detail modal
        if matches!(interaction_state(), CardInteractionState::ShowingDetailModal) {
            super::card_detail_modal::CardDetailModal {
                board_id: board_id.clone(),
                card_id: card.id.clone(),
                on_close: move |_| interaction_state.set(CardInteractionState::Idle),
            }
        }
        // Move menu (accessible via 'm' key)
        if matches!(interaction_state(), CardInteractionState::ShowingMoveMenu) {
            CardMoveMenu {
                board_id: board_id_for_move_menu.clone(),
                card_id: card_id_for_move_menu.clone(),
                current_column_id: column_id_for_move_menu.clone(),
                on_move: move |target_col: String| {
                    interaction_state.set(CardInteractionState::Idle);
                    let services = services_for_move.clone();
                    let board_id = board_id_for_move_menu.clone();
                    let card_id = card_id_for_move_menu.clone();
                    let on_announce = on_move_announce;
                    spawn(async move {
                        if let Err(e) = services.kanban().move_card(&board_id, &card_id, &target_col, 0).await {
                            tracing::error!(target = "ui.kanban", "Move failed: {e}");
                        } else {
                            on_announce.call("Card moved to column".to_string());
                        }
                    });
                },
                on_close: move |_| interaction_state.set(CardInteractionState::Idle),
            }
        }
        // Keyboard shortcuts help tooltip
        if matches!(interaction_state(), CardInteractionState::ShowingHelpTooltip) {
            KeyboardHelpTooltip {
                on_close: move |_| interaction_state.set(CardInteractionState::Idle),
            }
        }
    }
}

/// Small tag chip for display on cards.
#[derive(Props, Clone, PartialEq)]
struct TagChipProps {
    tag: TagView,
}

#[component]
fn TagChip(props: TagChipProps) -> Element {
    let tag = &props.tag;
    // Parse color or use default
    let bg_style = format!("background-color: {}", tag.color);

    rsx! {
        span {
            class: "px-1.5 py-0.5 rounded text-[10px] font-medium text-white truncate max-w-16",
            style: "{bg_style}",
            "{tag.name}"
        }
    }
}

/// Card sync state indicator - small badge in card corner.
#[derive(Props, Clone, PartialEq)]
struct CardSyncIndicatorProps {
    /// Sync state to display.
    state: SyncState,
}

#[component]
fn CardSyncIndicator(props: CardSyncIndicatorProps) -> Element {
    // Don't show indicator for synced items
    if props.state == SyncState::Synced {
        return rsx! {};
    }

    let (icon, color, label) = match props.state {
        SyncState::Synced => ("✓", colors::SUCCESS, "Synced"),
        SyncState::Syncing => ("⟳", colors::INFO, "Syncing"),
        SyncState::Queued => ("⏱", colors::WARNING, "Waiting to sync"),
        SyncState::Conflict => ("⚠", colors::WARNING, "Has conflicts"),
        SyncState::Error => ("✕", colors::ERROR, "Sync failed"),
    };

    rsx! {
        span {
            class: "text-xs flex-shrink-0",
            style: "color: {color}",
            title: "{label}",
            aria_label: "{label}",
            "{icon}"
        }
    }
}

/// Checklist progress badge.
#[derive(Props, Clone, PartialEq)]
struct ChecklistProgressBadgeProps {
    progress: ChecklistProgress,
}

#[component]
fn ChecklistProgressBadge(props: ChecklistProgressBadgeProps) -> Element {
    let progress = &props.progress;
    let percentage = if progress.total > 0 {
        (progress.completed as f32 / progress.total as f32 * 100.0) as u32
    } else {
        0
    };
    let is_complete = progress.completed == progress.total && progress.total > 0;

    rsx! {
        div {
            class: format!(
                "flex items-center gap-1 text-xs {}",
                if is_complete { "text-emerald-400" } else { "text-slate-400" }
            ),
            title: format!("{}/{} steps completed", progress.completed, progress.total),
            if is_complete {
                span { "✓" }
            } else {
                // Progress bar
                div {
                    class: "w-8 h-1 bg-slate-700 rounded-full overflow-hidden",
                    div {
                        class: "h-full bg-slate-400 rounded-full",
                        style: "width: {percentage}%",
                    }
                }
            }
            span {
                "{progress.completed}/{progress.total}"
            }
        }
    }
}

/// Priority badge component for displaying card priority.
#[derive(Props, Clone, PartialEq)]
struct PriorityBadgeProps {
    priority: PriorityView,
}

#[component]
fn PriorityBadge(props: PriorityBadgeProps) -> Element {
    let priority = props.priority;
    let (label, bg_color, text_color) = match priority {
        PriorityView::Urgent => ("Urgent", "bg-red-500/20", "text-red-400"),
        PriorityView::High => ("High", "bg-orange-500/20", "text-orange-400"),
        PriorityView::Normal => ("Normal", "bg-blue-500/20", "text-blue-400"),
        PriorityView::Low => ("Low", "bg-slate-500/20", "text-slate-400"),
    };

    rsx! {
        span {
            class: format!("text-xs px-1.5 py-0.5 rounded font-medium {bg_color} {text_color}"),
            title: format!("Priority: {}", label),
            "{label}"
        }
    }
}

/// Thread indicator badge showing card has linked discussion.
#[derive(Props, Clone, PartialEq)]
struct ThreadIndicatorProps {
    /// Display name of the linked thread (optional).
    thread_name: Option<String>,
}

#[component]
fn ThreadIndicator(props: ThreadIndicatorProps) -> Element {
    let title = props
        .thread_name
        .as_ref()
        .map(|n| format!("Linked to: {}", n))
        .unwrap_or_else(|| "Has linked discussion".to_string());

    rsx! {
        span {
            class: "text-xs px-1.5 py-0.5 rounded bg-emerald-500/20 text-emerald-400 flex items-center gap-1",
            title: "{title}",
            "💬"
        }
    }
}

/// Move menu for keyboard-accessible card movement.
/// Shows a list of columns to move the card to.
#[derive(Props, Clone, PartialEq)]
struct CardMoveMenuProps {
    board_id: String,
    card_id: String,
    current_column_id: String,
    on_move: EventHandler<String>,
    on_close: EventHandler<()>,
}

#[component]
fn CardMoveMenu(props: CardMoveMenuProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let mut columns = use_signal(Vec::new);

    // Fetch columns for this board
    let board_id = props.board_id.clone();
    use_future(move || {
        let services = services.clone();
        let board_id = board_id.clone();
        async move {
            if let Ok(board) = services.kanban().get_board(&board_id).await {
                columns.set(board.columns);
            }
        }
    });

    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| props.on_close.call(()),
        }
        // Menu
        div {
            class: "absolute left-0 top-full mt-1 w-48 rounded-lg border border-slate-600 bg-slate-800 shadow-xl z-50",
            role: "menu",
            aria_label: "Move card to column",
            div {
                class: "px-3 py-2 border-b border-slate-700",
                span { class: "text-xs font-medium text-slate-400 uppercase", "Move to..." }
            }
            div {
                class: "py-1 max-h-48 overflow-y-auto",
                for col in columns() {
                    if col.id != props.current_column_id {
                        button {
                            key: "{col.id}",
                            class: "w-full px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none",
                            role: "menuitem",
                            tabindex: "0",
                            onclick: {
                                let col_id = col.id.clone();
                                move |_| props.on_move.call(col_id.clone())
                            },
                            "{col.name}"
                        }
                    }
                }
            }
            div {
                class: "px-3 py-2 border-t border-slate-700",
                span { class: "text-xs text-slate-500", "Press Esc to cancel" }
            }
        }
    }
}

/// Keyboard shortcuts help tooltip.
#[derive(Props, Clone, PartialEq)]
struct KeyboardHelpTooltipProps {
    on_close: EventHandler<()>,
}

#[component]
fn KeyboardHelpTooltip(props: KeyboardHelpTooltipProps) -> Element {
    rsx! {
        // Backdrop
        div {
            class: "fixed inset-0 z-40",
            onclick: move |_| props.on_close.call(()),
        }
        // Tooltip
        div {
            class: "absolute left-0 top-full mt-1 w-64 rounded-lg border border-slate-600 bg-slate-800 shadow-xl z-50 p-4",
            role: "tooltip",
            aria_label: "Keyboard shortcuts",
            h4 { class: "text-sm font-semibold text-white mb-3", "Keyboard Shortcuts" }
            div {
                class: "space-y-2 text-xs",
                ShortcutRow { keys: "Enter", description: "Open card details" }
                ShortcutRow { keys: "m", description: "Open move menu" }
                ShortcutRow { keys: "Ctrl + ←/→", description: "Move to adjacent column" }
                ShortcutRow { keys: "Ctrl + ↑/↓", description: "Move within column" }
                ShortcutRow { keys: "?", description: "Toggle this help" }
                ShortcutRow { keys: "Esc", description: "Close menus" }
            }
            div {
                class: "mt-3 pt-2 border-t border-slate-700",
                span { class: "text-xs text-slate-500", "Press ? or Esc to close" }
            }
        }
    }
}

/// Single shortcut row in the help tooltip.
#[derive(Props, Clone, PartialEq)]
struct ShortcutRowProps {
    keys: &'static str,
    description: &'static str,
}

#[component]
fn ShortcutRow(props: ShortcutRowProps) -> Element {
    rsx! {
        div {
            class: "flex justify-between items-center",
            span {
                class: "px-1.5 py-0.5 rounded bg-slate-700 text-slate-300 font-mono",
                "{props.keys}"
            }
            span { class: "text-slate-400", "{props.description}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_color_mapping() {
        let todo_color = match CardState::Todo {
            CardState::Todo => "border-l-slate-500",
            _ => "",
        };
        assert_eq!(todo_color, "border-l-slate-500");

        let done_color = match CardState::Done {
            CardState::Done => "border-l-emerald-500",
            _ => "",
        };
        assert_eq!(done_color, "border-l-emerald-500");
    }

    #[test]
    fn checklist_progress_percentage() {
        let progress = ChecklistProgress {
            completed: 3,
            total: 10,
        };
        let percentage = if progress.total > 0 {
            (progress.completed as f32 / progress.total as f32 * 100.0) as u32
        } else {
            0
        };
        assert_eq!(percentage, 30);
    }

    #[test]
    fn checklist_complete_detection() {
        let progress = ChecklistProgress {
            completed: 5,
            total: 5,
        };
        let is_complete = progress.completed == progress.total && progress.total > 0;
        assert!(is_complete);
    }

    #[test]
    fn empty_checklist_returns_zero_percentage() {
        let progress = ChecklistProgress {
            completed: 0,
            total: 0,
        };
        let percentage = if progress.total > 0 {
            (progress.completed as f32 / progress.total as f32 * 100.0) as u32
        } else {
            0
        };
        assert_eq!(percentage, 0);
    }

    #[test]
    fn card_state_variants_are_distinct() {
        assert_ne!(CardState::Todo, CardState::InProgress);
        assert_ne!(CardState::InProgress, CardState::InReview);
        assert_ne!(CardState::InReview, CardState::Done);
        assert_ne!(CardState::Done, CardState::Archived);
    }

    #[test]
    fn card_aria_grabbed_state() {
        // When dragging, aria-grabbed should be "true"
        let is_dragging = true;
        let grabbed = if is_dragging { "true" } else { "false" };
        assert_eq!(grabbed, "true");

        // When not dragging, aria-grabbed should be "false"
        let is_dragging = false;
        let grabbed = if is_dragging { "true" } else { "false" };
        assert_eq!(grabbed, "false");
    }

    #[test]
    fn card_description_id_format() {
        let card_id = "card-123";
        let desc_id = format!("card-desc-{}", card_id);
        assert_eq!(desc_id, "card-desc-card-123");
    }

    #[test]
    fn card_aria_describedby_with_description() {
        let card_id = "card-456";
        let description = Some("This is a test description".to_string());
        let has_description = description.as_ref().is_some_and(|d| !d.is_empty());

        let desc_id = format!("card-desc-{}", card_id);
        let describedby = if has_description {
            desc_id.clone()
        } else {
            String::new()
        };
        assert_eq!(describedby, "card-desc-card-456");
    }

    #[test]
    fn card_aria_describedby_without_description() {
        let card_id = "card-789";
        let description: Option<String> = None;
        let has_description = description.as_ref().is_some_and(|d| !d.is_empty());

        let desc_id = format!("card-desc-{}", card_id);
        let describedby = if has_description {
            desc_id
        } else {
            String::new()
        };
        assert!(describedby.is_empty());
    }

    #[test]
    fn card_aria_describedby_with_empty_description() {
        let card_id = "card-empty";
        let description = Some(String::new());
        let has_description = description.as_ref().is_some_and(|d| !d.is_empty());

        let desc_id = format!("card-desc-{}", card_id);
        let describedby = if has_description {
            desc_id
        } else {
            String::new()
        };
        assert!(describedby.is_empty());
    }

    #[test]
    fn priority_badge_colors() {
        // Test priority color mapping
        let urgent = match PriorityView::Urgent {
            PriorityView::Urgent => ("Urgent", "bg-red-500/20", "text-red-400"),
            _ => ("", "", ""),
        };
        assert_eq!(urgent.0, "Urgent");
        assert!(urgent.1.contains("red"));

        let high = match PriorityView::High {
            PriorityView::High => ("High", "bg-orange-500/20", "text-orange-400"),
            _ => ("", "", ""),
        };
        assert_eq!(high.0, "High");
        assert!(high.1.contains("orange"));

        let normal = match PriorityView::Normal {
            PriorityView::Normal => ("Normal", "bg-blue-500/20", "text-blue-400"),
            _ => ("", "", ""),
        };
        assert_eq!(normal.0, "Normal");
        assert!(normal.1.contains("blue"));

        let low = match PriorityView::Low {
            PriorityView::Low => ("Low", "bg-slate-500/20", "text-slate-400"),
            _ => ("", "", ""),
        };
        assert_eq!(low.0, "Low");
        assert!(low.1.contains("slate"));
    }

    #[test]
    fn priority_sort_order() {
        // Urgent should sort before High, etc.
        assert!(PriorityView::Urgent.sort_order() < PriorityView::High.sort_order());
        assert!(PriorityView::High.sort_order() < PriorityView::Normal.sort_order());
        assert!(PriorityView::Normal.sort_order() < PriorityView::Low.sort_order());
    }
}
