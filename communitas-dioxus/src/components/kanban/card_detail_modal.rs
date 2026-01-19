//! Card detail modal for viewing and editing card contents.

use communitas_ui_api::kanban::{
    ActivityEntry, CardDetail, CardState, CommentView, StepView, TagView,
};
use communitas_ui_service::UiServices;
use communitas_ui_service::kanban::CardUpdate;
use dioxus::prelude::*;
use std::sync::Arc;
use tracing::info;

/// Modal for viewing and editing card details.
#[derive(Props, Clone, PartialEq)]
pub struct CardDetailModalProps {
    /// The board ID containing the card.
    pub board_id: String,
    /// The card ID to display.
    pub card_id: String,
    /// Handler for closing the modal.
    pub on_close: EventHandler<()>,
}

#[component]
pub fn CardDetailModal(props: CardDetailModalProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let kanban = services.kanban();

    // Card data state
    let mut card_data = use_signal(|| Option::<CardDetail>::None);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    // Edit states
    let mut editing_title = use_signal(|| false);
    let mut title_draft = use_signal(String::new);
    let mut editing_description = use_signal(|| false);
    let mut description_draft = use_signal(String::new);

    // New step input
    let mut new_step_title = use_signal(String::new);
    let mut adding_step = use_signal(|| false);

    // New comment input
    let mut new_comment_text = use_signal(String::new);
    let mut adding_comment = use_signal(|| false);

    // Activity section collapsed
    let mut show_activity = use_signal(|| false);

    // Fetch card data
    let board_id = props.board_id.clone();
    let card_id = props.card_id.clone();
    use_future(move || {
        let kanban = kanban.clone();
        let board_id = board_id.clone();
        let card_id = card_id.clone();
        async move {
            match kanban.get_card(&board_id, &card_id).await {
                Ok(card) => {
                    title_draft.set(card.title.clone());
                    description_draft.set(card.description.clone().unwrap_or_default());
                    card_data.set(Some(card));
                    loading.set(false);
                }
                Err(err) => {
                    error.set(Some(err.to_string()));
                    loading.set(false);
                }
            }
        }
    });

    // Clone values for use in multiple closures
    let services_for_title = services.clone();
    let services_for_desc = services.clone();
    let services_for_add_step = services.clone();
    let services_for_toggle_step = services.clone();
    let services_for_comments = services.clone();

    let board_id_for_title = props.board_id.clone();
    let board_id_for_desc = props.board_id.clone();
    let board_id_for_add_step = props.board_id.clone();
    let board_id_for_toggle_step = props.board_id.clone();
    let board_id_for_comments = props.board_id.clone();

    let card_id_for_title = props.card_id.clone();
    let card_id_for_desc = props.card_id.clone();
    let card_id_for_add_step = props.card_id.clone();
    let card_id_for_toggle_step = props.card_id.clone();
    let card_id_for_comments = props.card_id.clone();

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/70",
            role: "dialog",
            aria_modal: "true",
            aria_label: "Card details",
            onclick: move |_| props.on_close.call(()),
            // Modal content
            div {
                class: "w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-xl border border-slate-700 bg-slate-900 shadow-2xl",
                onclick: move |evt| evt.stop_propagation(),
                // Loading state
                if loading() {
                    CardDetailSkeleton {}
                } else if let Some(err) = error() {
                    div {
                        class: "p-6 text-center",
                        p { class: "text-red-400 mb-4", "Failed to load card: {err}" }
                        button {
                            class: "rounded-lg bg-slate-700 px-4 py-2 text-slate-200",
                            onclick: move |_| props.on_close.call(()),
                            "Close"
                        }
                    }
                } else if let Some(card) = card_data() {
                    // Header with title and actions
                    div {
                        class: "flex items-start justify-between p-6 border-b border-slate-800",
                        div {
                            class: "flex-1",
                            if editing_title() {
                                TitleEditor {
                                    value: title_draft(),
                                    on_change: move |v: String| title_draft.set(v),
                                    on_save: move |_| {
                                        let services = services_for_title.clone();
                                        let board_id = board_id_for_title.clone();
                                        let card_id = card_id_for_title.clone();
                                        let new_title = title_draft();
                                        spawn(async move {
                                            let update = CardUpdate {
                                                title: Some(new_title),
                                                ..Default::default()
                                            };
                                            if let Err(err) = services.kanban().update_card(&board_id, &card_id, update).await {
                                                tracing::error!(target = "ui.kanban", "failed to update title: {err}");
                                            } else {
                                                editing_title.set(false);
                                            }
                                        });
                                    },
                                    on_cancel: move |_| {
                                        title_draft.set(card.title.clone());
                                        editing_title.set(false);
                                    },
                                }
                            } else {
                                h2 {
                                    class: "text-xl font-semibold text-white cursor-pointer hover:text-emerald-400",
                                    onclick: move |_| editing_title.set(true),
                                    "{card.title}"
                                }
                            }
                            // State selector
                            div {
                                class: "mt-2",
                                StateSelector {
                                    current: card.state,
                                    on_change: move |new_state: CardState| {
                                        info!(target = "ui.kanban", event = "state_changed", state = ?new_state);
                                        // TODO: Update state via service
                                    },
                                }
                            }
                        }
                        // Close button
                        button {
                            class: "p-2 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800",
                            onclick: move |_| props.on_close.call(()),
                            "✕"
                        }
                    }
                    // Content sections
                    div {
                        class: "p-6 space-y-6",
                        // Description section
                        DescriptionSection {
                            description: card.description.clone(),
                            editing: editing_description(),
                            draft: description_draft(),
                            on_edit: move |_| editing_description.set(true),
                            on_change: move |v: String| description_draft.set(v),
                            on_save: move |_| {
                                let services = services_for_desc.clone();
                                let board_id = board_id_for_desc.clone();
                                let card_id = card_id_for_desc.clone();
                                let new_desc = description_draft();
                                spawn(async move {
                                    let update = CardUpdate {
                                        description: Some(new_desc),
                                        ..Default::default()
                                    };
                                    if let Err(err) = services.kanban().update_card(&board_id, &card_id, update).await {
                                        tracing::error!(target = "ui.kanban", "failed to update description: {err}");
                                    } else {
                                        editing_description.set(false);
                                    }
                                });
                            },
                            on_cancel: move |_| {
                                description_draft.set(card.description.clone().unwrap_or_default());
                                editing_description.set(false);
                            },
                        }
                        // Assignees section
                        AssigneesSection {
                            assignees: card.assignees.clone(),
                            on_add: move |_| {
                                info!(target = "ui.kanban", event = "add_assignee_clicked");
                            },
                        }
                        // Tags section
                        TagsSection {
                            tags: card.tags.clone(),
                            on_add: move |_| {
                                info!(target = "ui.kanban", event = "add_tag_clicked");
                            },
                        }
                        // Due date section
                        DueDateSection {
                            due_date: card.due_date,
                            on_change: move |_date| {
                                info!(target = "ui.kanban", event = "due_date_changed");
                            },
                        }
                        // Checklist section
                        ChecklistSection {
                            steps: card.steps.clone(),
                            new_step_title: new_step_title(),
                            adding: adding_step(),
                            on_new_step_change: move |v: String| new_step_title.set(v),
                            on_add_step: move |_| {
                                let services = services_for_add_step.clone();
                                let board_id = board_id_for_add_step.clone();
                                let card_id = card_id_for_add_step.clone();
                                let title = new_step_title();
                                adding_step.set(true);
                                spawn(async move {
                                    match services.kanban().add_step(&board_id, &card_id, &title).await {
                                        Ok(_step) => {
                                            new_step_title.set(String::new());
                                            // Refresh card data
                                            if let Ok(updated) = services.kanban().get_card(&board_id, &card_id).await {
                                                card_data.set(Some(updated));
                                            }
                                        }
                                        Err(err) => {
                                            tracing::error!(target = "ui.kanban", "failed to add step: {err}");
                                        }
                                    }
                                    adding_step.set(false);
                                });
                            },
                            on_toggle_step: move |step_id: String| {
                                let services = services_for_toggle_step.clone();
                                let board_id = board_id_for_toggle_step.clone();
                                let card_id = card_id_for_toggle_step.clone();
                                spawn(async move {
                                    match services.kanban().toggle_step(&board_id, &card_id, &step_id).await {
                                        Ok(_) => {
                                            // Refresh card data
                                            if let Ok(updated) = services.kanban().get_card(&board_id, &card_id).await {
                                                card_data.set(Some(updated));
                                            }
                                        }
                                        Err(err) => {
                                            tracing::error!(target = "ui.kanban", "failed to toggle step: {err}");
                                        }
                                    }
                                });
                            },
                        }
                        // Comments section
                        CommentsSection {
                            comments: card.comments.clone(),
                            new_comment: new_comment_text(),
                            adding: adding_comment(),
                            on_new_comment_change: move |v: String| new_comment_text.set(v),
                            on_add_comment: move |_| {
                                let services = services_for_comments.clone();
                                let board_id = board_id_for_comments.clone();
                                let card_id = card_id_for_comments.clone();
                                let text = new_comment_text();
                                adding_comment.set(true);
                                spawn(async move {
                                    match services.kanban().add_comment(&board_id, &card_id, &text).await {
                                        Ok(_comment) => {
                                            new_comment_text.set(String::new());
                                            // Refresh card data
                                            if let Ok(updated) = services.kanban().get_card(&board_id, &card_id).await {
                                                card_data.set(Some(updated));
                                            }
                                        }
                                        Err(err) => {
                                            tracing::error!(target = "ui.kanban", "failed to add comment: {err}");
                                        }
                                    }
                                    adding_comment.set(false);
                                });
                            },
                        }
                        // Activity section (collapsible)
                        ActivitySection {
                            activity: card.activity.clone(),
                            expanded: show_activity(),
                            on_toggle: move |_| show_activity.set(!show_activity()),
                        }
                    }
                }
            }
        }
    }
}

/// Skeleton loader for card detail.
#[component]
fn CardDetailSkeleton() -> Element {
    rsx! {
        div {
            class: "p-6 animate-pulse",
            aria_busy: "true",
            div { class: "h-8 w-64 bg-slate-700 rounded mb-4" }
            div { class: "h-4 w-32 bg-slate-700 rounded mb-6" }
            div { class: "h-24 bg-slate-700 rounded mb-4" }
            div { class: "h-16 bg-slate-700 rounded mb-4" }
            div { class: "h-32 bg-slate-700 rounded" }
        }
    }
}

/// Inline title editor.
#[derive(Props, Clone, PartialEq)]
struct TitleEditorProps {
    value: String,
    on_change: EventHandler<String>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn TitleEditor(props: TitleEditorProps) -> Element {
    rsx! {
        form {
            class: "flex gap-2",
            onsubmit: move |evt| {
                evt.prevent_default();
                props.on_save.call(());
            },
            input {
                r#type: "text",
                class: "flex-1 rounded border border-slate-600 bg-slate-800 px-3 py-2 text-lg text-white focus:border-emerald-400 focus:outline-none",
                value: "{props.value}",
                autofocus: true,
                oninput: move |evt| props.on_change.call(evt.value()),
            }
            button {
                r#type: "submit",
                class: "rounded bg-emerald-500 px-3 py-2 text-sm font-semibold text-slate-900",
                "Save"
            }
            button {
                r#type: "button",
                class: "rounded px-3 py-2 text-sm text-slate-400 hover:text-white",
                onclick: move |_| props.on_cancel.call(()),
                "Cancel"
            }
        }
    }
}

/// State selector dropdown.
#[derive(Props, Clone, PartialEq)]
struct StateSelectorProps {
    current: CardState,
    on_change: EventHandler<CardState>,
}

#[component]
fn StateSelector(props: StateSelectorProps) -> Element {
    let states = [
        (CardState::Todo, "To Do", "bg-slate-600"),
        (CardState::InProgress, "In Progress", "bg-blue-600"),
        (CardState::InReview, "In Review", "bg-amber-600"),
        (CardState::Done, "Done", "bg-emerald-600"),
        (CardState::Archived, "Archived", "bg-slate-700"),
    ];

    let current_label = states
        .iter()
        .find(|(s, _, _)| *s == props.current)
        .map(|(_, l, _)| *l)
        .unwrap_or("Unknown");
    let current_color = states
        .iter()
        .find(|(s, _, _)| *s == props.current)
        .map(|(_, _, c)| *c)
        .unwrap_or("bg-slate-600");

    let mut show_dropdown = use_signal(|| false);

    rsx! {
        div {
            class: "relative inline-block",
            button {
                class: format!("rounded px-2 py-1 text-xs font-medium text-white {current_color}"),
                onclick: move |_| show_dropdown.set(!show_dropdown()),
                "{current_label}"
            }
            if show_dropdown() {
                div {
                    class: "absolute top-full left-0 mt-1 rounded border border-slate-700 bg-slate-800 shadow-lg z-10",
                    {states.iter().map(|(state, label, color)| {
                        let state = *state;
                        let on_change = props.on_change;
                        rsx! {
                            button {
                                key: "{label}",
                                class: format!("w-full px-3 py-1.5 text-left text-sm text-white hover:bg-slate-700 flex items-center gap-2"),
                                onclick: move |_| {
                                    on_change.call(state);
                                    show_dropdown.set(false);
                                },
                                span {
                                    class: format!("w-2 h-2 rounded-full {color}"),
                                }
                                "{label}"
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// Description section with edit mode.
#[derive(Props, Clone, PartialEq)]
struct DescriptionSectionProps {
    description: Option<String>,
    editing: bool,
    draft: String,
    on_edit: EventHandler<()>,
    on_change: EventHandler<String>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
}

#[component]
fn DescriptionSection(props: DescriptionSectionProps) -> Element {
    rsx! {
        div {
            class: "space-y-2",
            h3 {
                class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                "Description"
            }
            if props.editing {
                form {
                    class: "space-y-2",
                    onsubmit: move |evt| {
                        evt.prevent_default();
                        props.on_save.call(());
                    },
                    textarea {
                        class: "w-full h-32 rounded border border-slate-600 bg-slate-800 px-3 py-2 text-sm text-slate-200 resize-none focus:border-emerald-400 focus:outline-none",
                        value: "{props.draft}",
                        autofocus: true,
                        oninput: move |evt| props.on_change.call(evt.value()),
                    }
                    div {
                        class: "flex gap-2",
                        button {
                            r#type: "submit",
                            class: "rounded bg-emerald-500 px-3 py-1.5 text-sm font-semibold text-slate-900",
                            "Save"
                        }
                        button {
                            r#type: "button",
                            class: "rounded px-3 py-1.5 text-sm text-slate-400 hover:text-white",
                            onclick: move |_| props.on_cancel.call(()),
                            "Cancel"
                        }
                    }
                }
            } else {
                div {
                    class: "rounded border border-slate-700 bg-slate-800/50 px-3 py-2 cursor-pointer hover:bg-slate-800",
                    onclick: move |_| props.on_edit.call(()),
                    if let Some(desc) = &props.description {
                        if !desc.is_empty() {
                            p { class: "text-sm text-slate-200 whitespace-pre-wrap", "{desc}" }
                        } else {
                            p { class: "text-sm text-slate-500 italic", "Add a description..." }
                        }
                    } else {
                        p { class: "text-sm text-slate-500 italic", "Add a description..." }
                    }
                }
            }
        }
    }
}

/// Assignees section.
#[derive(Props, Clone, PartialEq)]
struct AssigneesSectionProps {
    assignees: Vec<String>,
    on_add: EventHandler<()>,
}

#[component]
fn AssigneesSection(props: AssigneesSectionProps) -> Element {
    rsx! {
        div {
            class: "space-y-2",
            h3 {
                class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                "Assignees"
            }
            div {
                class: "flex flex-wrap gap-2",
                {props.assignees.iter().map(|assignee| {
                    let initials: String = assignee.chars().take(2).collect();
                    rsx! {
                        div {
                            key: "{assignee}",
                            class: "flex items-center gap-2 rounded-full bg-slate-700 pl-1 pr-3 py-1",
                            div {
                                class: "w-6 h-6 rounded-full bg-slate-600 flex items-center justify-center",
                                span {
                                    class: "text-[10px] text-slate-300 uppercase",
                                    "{initials}"
                                }
                            }
                            span { class: "text-sm text-slate-200", "{assignee}" }
                        }
                    }
                })}
                button {
                    class: "rounded-full border border-dashed border-slate-600 px-3 py-1 text-sm text-slate-400 hover:border-emerald-400 hover:text-emerald-400",
                    onclick: move |_| props.on_add.call(()),
                    "+ Add"
                }
            }
        }
    }
}

/// Tags section.
#[derive(Props, Clone, PartialEq)]
struct TagsSectionProps {
    tags: Vec<TagView>,
    on_add: EventHandler<()>,
}

#[component]
fn TagsSection(props: TagsSectionProps) -> Element {
    rsx! {
        div {
            class: "space-y-2",
            h3 {
                class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                "Tags"
            }
            div {
                class: "flex flex-wrap gap-2",
                {props.tags.iter().map(|tag| {
                    let bg_style = format!("background-color: {}", tag.color);
                    rsx! {
                        span {
                            key: "{tag.id}",
                            class: "px-2 py-1 rounded text-xs font-medium text-white",
                            style: "{bg_style}",
                            "{tag.name}"
                        }
                    }
                })}
                button {
                    class: "rounded border border-dashed border-slate-600 px-2 py-1 text-xs text-slate-400 hover:border-emerald-400 hover:text-emerald-400",
                    onclick: move |_| props.on_add.call(()),
                    "+ Add tag"
                }
            }
        }
    }
}

/// Due date section.
#[derive(Props, Clone, PartialEq)]
struct DueDateSectionProps {
    due_date: Option<i64>,
    on_change: EventHandler<Option<i64>>,
}

#[component]
fn DueDateSection(props: DueDateSectionProps) -> Element {
    let formatted = props.due_date.map(|ts| {
        let dt = chrono::DateTime::from_timestamp_millis(ts);
        dt.map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Invalid date".to_string())
    });

    rsx! {
        div {
            class: "space-y-2",
            h3 {
                class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                "Due Date"
            }
            div {
                class: "flex items-center gap-2",
                if let Some(date) = formatted {
                    span {
                        class: "rounded bg-slate-700 px-3 py-1 text-sm text-slate-200",
                        "{date}"
                    }
                    button {
                        class: "text-sm text-slate-400 hover:text-red-400",
                        onclick: move |_| props.on_change.call(None),
                        "Clear"
                    }
                } else {
                    button {
                        class: "rounded border border-dashed border-slate-600 px-3 py-1 text-sm text-slate-400 hover:border-emerald-400 hover:text-emerald-400",
                        onclick: move |_| {
                            // TODO: Open date picker
                            props.on_change.call(Some(chrono::Utc::now().timestamp_millis()));
                        },
                        "Set due date"
                    }
                }
            }
        }
    }
}

/// Checklist section.
#[derive(Props, Clone, PartialEq)]
struct ChecklistSectionProps {
    steps: Vec<StepView>,
    new_step_title: String,
    adding: bool,
    on_new_step_change: EventHandler<String>,
    on_add_step: EventHandler<()>,
    on_toggle_step: EventHandler<String>,
}

#[component]
fn ChecklistSection(props: ChecklistSectionProps) -> Element {
    let completed = props.steps.iter().filter(|s| s.completed).count();
    let total = props.steps.len();
    let percentage = if total > 0 {
        (completed as f32 / total as f32 * 100.0) as u32
    } else {
        0
    };

    // Clone values needed in closures
    let new_step_title = props.new_step_title.clone();
    let new_step_title_for_submit = new_step_title.clone();
    let new_step_title_for_disabled = new_step_title.clone();
    let on_add_step = props.on_add_step;
    let on_new_step_change = props.on_new_step_change;
    let adding = props.adding;

    rsx! {
        div {
            class: "space-y-2",
            div {
                class: "flex items-center justify-between",
                h3 {
                    class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                    "Checklist"
                }
                if total > 0 {
                    span {
                        class: "text-xs text-slate-500",
                        "{completed}/{total}"
                    }
                }
            }
            // Progress bar
            if total > 0 {
                div {
                    class: "h-1.5 bg-slate-700 rounded-full overflow-hidden",
                    div {
                        class: "h-full bg-emerald-500 rounded-full transition-all",
                        style: "width: {percentage}%",
                    }
                }
            }
            // Steps list
            div {
                class: "space-y-1",
                {props.steps.iter().map(|step| {
                    let step_id = step.id.clone();
                    let on_toggle = props.on_toggle_step;
                    rsx! {
                        div {
                            key: "{step_id}",
                            class: "flex items-center gap-2 py-1",
                            input {
                                r#type: "checkbox",
                                class: "w-4 h-4 rounded border-slate-600 bg-slate-800 text-emerald-500 focus:ring-emerald-500",
                                checked: step.completed,
                                onchange: move |_| on_toggle.call(step_id.clone()),
                            }
                            span {
                                class: format!(
                                    "text-sm {}",
                                    if step.completed { "text-slate-500 line-through" } else { "text-slate-200" }
                                ),
                                "{step.title}"
                            }
                        }
                    }
                })}
            }
            // Add step form
            form {
                class: "flex gap-2 mt-2",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    if !new_step_title_for_submit.trim().is_empty() {
                        on_add_step.call(());
                    }
                },
                input {
                    r#type: "text",
                    class: "flex-1 rounded border border-slate-700 bg-slate-800 px-3 py-1.5 text-sm text-slate-200 focus:border-emerald-400 focus:outline-none",
                    placeholder: "Add a step...",
                    disabled: adding,
                    value: "{new_step_title}",
                    oninput: move |evt| on_new_step_change.call(evt.value()),
                }
                button {
                    r#type: "submit",
                    class: "rounded bg-slate-700 px-3 py-1.5 text-sm text-slate-200 hover:bg-slate-600 disabled:opacity-50",
                    disabled: adding || new_step_title_for_disabled.trim().is_empty(),
                    if adding { "..." } else { "Add" }
                }
            }
        }
    }
}

/// Comments section.
#[derive(Props, Clone, PartialEq)]
struct CommentsSectionProps {
    comments: Vec<CommentView>,
    new_comment: String,
    adding: bool,
    on_new_comment_change: EventHandler<String>,
    on_add_comment: EventHandler<()>,
}

#[component]
fn CommentsSection(props: CommentsSectionProps) -> Element {
    // Clone values needed in closures
    let comment_count = props.comments.len();
    let new_comment = props.new_comment.clone();
    let new_comment_for_submit = new_comment.clone();
    let new_comment_for_disabled = new_comment.clone();
    let on_add_comment = props.on_add_comment;
    let on_new_comment_change = props.on_new_comment_change;
    let adding = props.adding;

    rsx! {
        div {
            class: "space-y-3",
            h3 {
                class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                "Comments ({comment_count})"
            }
            // Comments list
            div {
                class: "space-y-3",
                {props.comments.iter().map(|comment| {
                    let initials: String = comment.author_name.chars().take(2).collect();
                    let time = format_timestamp(comment.created_at);
                    rsx! {
                        div {
                            key: "{comment.id}",
                            class: "flex gap-3",
                            div {
                                class: "w-8 h-8 rounded-full bg-slate-700 flex items-center justify-center flex-shrink-0",
                                span {
                                    class: "text-xs text-slate-300 uppercase",
                                    "{initials}"
                                }
                            }
                            div {
                                class: "flex-1",
                                div {
                                    class: "flex items-center gap-2",
                                    span { class: "text-sm font-medium text-white", "{comment.author_name}" }
                                    span { class: "text-xs text-slate-500", "{time}" }
                                }
                                p { class: "text-sm text-slate-300 mt-1 whitespace-pre-wrap", "{comment.text}" }
                            }
                        }
                    }
                })}
            }
            // Add comment form
            form {
                class: "flex gap-2",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    if !new_comment_for_submit.trim().is_empty() {
                        on_add_comment.call(());
                    }
                },
                textarea {
                    class: "flex-1 rounded border border-slate-700 bg-slate-800 px-3 py-2 text-sm text-slate-200 resize-none focus:border-emerald-400 focus:outline-none",
                    placeholder: "Write a comment...",
                    rows: "2",
                    disabled: adding,
                    value: "{new_comment}",
                    oninput: move |evt| on_new_comment_change.call(evt.value()),
                }
                button {
                    r#type: "submit",
                    class: "self-end rounded bg-emerald-500 px-3 py-2 text-sm font-semibold text-slate-900 disabled:opacity-50",
                    disabled: adding || new_comment_for_disabled.trim().is_empty(),
                    if adding { "..." } else { "Post" }
                }
            }
        }
    }
}

/// Activity section (collapsible).
#[derive(Props, Clone, PartialEq)]
struct ActivitySectionProps {
    activity: Vec<ActivityEntry>,
    expanded: bool,
    on_toggle: EventHandler<()>,
}

#[component]
fn ActivitySection(props: ActivitySectionProps) -> Element {
    rsx! {
        div {
            class: "space-y-2",
            button {
                class: "flex items-center justify-between w-full text-left",
                onclick: move |_| props.on_toggle.call(()),
                h3 {
                    class: "text-sm font-medium text-slate-400 uppercase tracking-wide",
                    "Activity ({props.activity.len()})"
                }
                span {
                    class: "text-slate-500",
                    if props.expanded { "▼" } else { "▶" }
                }
            }
            if props.expanded {
                div {
                    class: "space-y-2 pl-4 border-l border-slate-700",
                    {props.activity.iter().map(|entry| {
                        let time = format_timestamp(entry.timestamp);
                        rsx! {
                            div {
                                key: "{entry.id}",
                                class: "text-sm",
                                span { class: "text-slate-400", "{entry.actor_name} " }
                                span { class: "text-slate-500", "{entry.description}" }
                                span { class: "text-xs text-slate-600 ml-2", "{time}" }
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// Format timestamp as relative time.
fn format_timestamp(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let diff_ms = now - ts;
    let diff_mins = diff_ms / 60_000;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_days > 7 {
        chrono::DateTime::from_timestamp_millis(ts)
            .map(|d| d.format("%b %d").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    } else if diff_days > 0 {
        format!("{}d ago", diff_days)
    } else if diff_hours > 0 {
        format!("{}h ago", diff_hours)
    } else if diff_mins > 0 {
        format!("{}m ago", diff_mins)
    } else {
        "just now".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_timestamp_just_now() {
        let now = chrono::Utc::now().timestamp_millis();
        let result = format_timestamp(now);
        assert_eq!(result, "just now");
    }

    #[test]
    fn format_timestamp_minutes_ago() {
        let five_mins_ago = chrono::Utc::now().timestamp_millis() - 5 * 60_000;
        let result = format_timestamp(five_mins_ago);
        assert_eq!(result, "5m ago");
    }

    #[test]
    fn format_timestamp_hours_ago() {
        let two_hours_ago = chrono::Utc::now().timestamp_millis() - 2 * 60 * 60_000;
        let result = format_timestamp(two_hours_ago);
        assert_eq!(result, "2h ago");
    }

    #[test]
    fn format_timestamp_days_ago() {
        let three_days_ago = chrono::Utc::now().timestamp_millis() - 3 * 24 * 60 * 60_000;
        let result = format_timestamp(three_days_ago);
        assert_eq!(result, "3d ago");
    }
}
