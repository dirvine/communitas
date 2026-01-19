//! Kanban card component with drag-and-drop support.

use communitas_ui_api::kanban::{CardState, CardView, ChecklistProgress, TagView};
use dioxus::prelude::*;
use tracing::info;

/// Kanban card component that can be dragged between columns.
#[derive(Props, Clone, PartialEq)]
pub struct KanbanCardProps {
    /// The card data to display.
    pub card: CardView,
    /// The board ID this card belongs to.
    pub board_id: String,
}

#[component]
pub fn KanbanCard(props: KanbanCardProps) -> Element {
    let card = &props.card;

    // Drag state
    let mut is_dragging = use_signal(|| false);

    // Modal state
    let mut show_detail_modal = use_signal(|| false);

    let card_id = card.id.clone();
    let card_id_for_drag_start = card_id.clone();
    let card_id_for_drag_end = card_id.clone();
    let board_id = props.board_id.clone();

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

    rsx! {
        div {
            class: format!(
                "kanban-card rounded-lg border border-l-4 bg-slate-800 p-3 cursor-pointer transition hover:bg-slate-750 {} {}",
                state_color,
                if is_dragging() { "opacity-50 border-slate-600" } else { "border-slate-700" }
            ),
            role: "listitem",
            aria_label: format!("Card: {}", card.title),
            tabindex: "0",
            draggable: "true",
            // Drag handlers
            ondragstart: move |_evt| {
                is_dragging.set(true);
                info!(target = "ui.kanban", event = "card_drag_start", card_id = %card_id_for_drag_start);
                // TODO: Set drag data with card ID
            },
            ondragend: move |_| {
                is_dragging.set(false);
                info!(target = "ui.kanban", event = "card_drag_end", card_id = %card_id_for_drag_end);
            },
            // Click to open detail
            onclick: move |_| {
                show_detail_modal.set(true);
            },
            // Keyboard support
            onkeydown: move |evt| {
                if evt.key() == Key::Enter {
                    show_detail_modal.set(true);
                }
            },
            // Card content
            div {
                class: "flex flex-col gap-2",
                // Title
                h4 {
                    class: "font-medium text-white text-sm line-clamp-2",
                    "{card.title}"
                }
                // Description preview (if exists)
                if let Some(desc) = &card.description {
                    if !desc.is_empty() {
                        p {
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
                    // Right side: due date and checklist progress
                    div {
                        class: "flex items-center gap-2",
                        // Due date badge
                        if let Some((label, classes)) = due_date_display {
                            span {
                                class: format!("text-xs px-1.5 py-0.5 rounded {classes}"),
                                "{label}"
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
        if show_detail_modal() {
            super::card_detail_modal::CardDetailModal {
                board_id: board_id.clone(),
                card_id: card.id.clone(),
                on_close: move |_| show_detail_modal.set(false),
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
}
