//! Board view component with columns, cards, and swimlane support.

use communitas_ui_api::kanban::{BoardView as BoardViewData, CardState, CardView, SwimlaneMode};
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use super::card::KanbanCard;
use super::column::KanbanColumn;

/// Board view showing columns with cards or swimlane groupings.
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

    // Swimlane mode state (synced with service)
    let mut swimlane_mode = use_signal(|| SwimlaneMode::None);

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

    // Aria-live announcement for screen readers
    let mut announcement = use_signal(String::new);

    let board_id_for_add = props.board_id.clone();
    let board_id_for_swimlane = props.board_id.clone();

    rsx! {
        div {
            class: "board-view flex flex-col h-full",
            role: "main",
            // Aria-live region for screen reader announcements
            div {
                role: "status",
                aria_live: "polite",
                aria_atomic: "true",
                class: "sr-only",
                "{announcement}"
            }
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
                            info!(target = "ui.kanban", event = "filters_changed");
                        },
                        on_swimlane_change: {
                            let services = services.clone();
                            move |mode: SwimlaneMode| {
                                swimlane_mode.set(mode);
                                services.kanban().set_swimlane_mode(mode);
                                info!(target = "ui.kanban", event = "swimlane_mode_changed", mode = ?mode);
                            }
                        },
                        current_swimlane: swimlane_mode(),
                    }
                }
                // Render either swimlane view or standard column view
                if swimlane_mode() == SwimlaneMode::None {
                    // Standard column view
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
                                        on_move_announce: move |msg: String| {
                                            announcement.set(msg);
                                        },
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
                } else {
                    // Swimlane view
                    SwimlaneView {
                        board: board.clone(),
                        board_id: board_id_for_swimlane.clone(),
                        mode: swimlane_mode(),
                        on_announce: move |msg: String| {
                            announcement.set(msg);
                        },
                    }
                }
            }
        }
    }
}

/// A single swimlane containing grouped cards.
#[derive(Debug)]
struct Swimlane {
    /// Unique key for this swimlane.
    key: String,
    /// Display label for the swimlane header.
    label: String,
    /// Optional color for visual distinction.
    color: Option<String>,
    /// Cards in this swimlane.
    cards: Vec<CardView>,
}

/// Group cards into swimlanes based on the specified mode.
fn group_cards_into_swimlanes(board: &BoardViewData, mode: SwimlaneMode) -> Vec<Swimlane> {
    // Collect all cards from all columns
    let all_cards: Vec<CardView> = board
        .columns
        .iter()
        .flat_map(|col| col.cards.clone())
        .collect();

    match mode {
        SwimlaneMode::None => vec![], // Should not happen, handled by caller
        SwimlaneMode::ByAssignee => group_by_assignee(&all_cards),
        SwimlaneMode::ByTag => group_by_tag(&all_cards),
        SwimlaneMode::ByState => group_by_state(&all_cards),
    }
}

/// Group cards by assignee.
fn group_by_assignee(cards: &[CardView]) -> Vec<Swimlane> {
    let mut groups: HashMap<String, Vec<CardView>> = HashMap::new();
    let mut unassigned: Vec<CardView> = Vec::new();

    for card in cards {
        if card.assignees.is_empty() {
            unassigned.push(card.clone());
        } else {
            // Card appears in each assignee's swimlane
            for assignee in &card.assignees {
                groups
                    .entry(assignee.clone())
                    .or_default()
                    .push(card.clone());
            }
        }
    }

    let mut swimlanes: Vec<Swimlane> = groups
        .into_iter()
        .map(|(assignee, cards)| Swimlane {
            key: assignee.clone(),
            label: assignee,
            color: None,
            cards,
        })
        .collect();

    // Sort by label
    swimlanes.sort_by(|a, b| a.label.cmp(&b.label));

    // Add unassigned at the end
    if !unassigned.is_empty() {
        swimlanes.push(Swimlane {
            key: "_unassigned".to_string(),
            label: "Unassigned".to_string(),
            color: Some("#64748b".to_string()), // slate-500
            cards: unassigned,
        });
    }

    swimlanes
}

/// Group cards by tag.
fn group_by_tag(cards: &[CardView]) -> Vec<Swimlane> {
    let mut groups: HashMap<String, (String, Option<String>, Vec<CardView>)> = HashMap::new();
    let mut no_tag: Vec<CardView> = Vec::new();

    for card in cards {
        if card.tags.is_empty() {
            no_tag.push(card.clone());
        } else {
            // Card appears in each tag's swimlane
            for tag in &card.tags {
                let entry = groups
                    .entry(tag.id.clone())
                    .or_insert_with(|| (tag.name.clone(), Some(tag.color.clone()), Vec::new()));
                entry.2.push(card.clone());
            }
        }
    }

    let mut swimlanes: Vec<Swimlane> = groups
        .into_iter()
        .map(|(id, (name, color, cards))| Swimlane {
            key: id,
            label: name,
            color,
            cards,
        })
        .collect();

    // Sort by label
    swimlanes.sort_by(|a, b| a.label.cmp(&b.label));

    // Add "No Tag" at the end
    if !no_tag.is_empty() {
        swimlanes.push(Swimlane {
            key: "_no_tag".to_string(),
            label: "No Tag".to_string(),
            color: Some("#64748b".to_string()), // slate-500
            cards: no_tag,
        });
    }

    swimlanes
}

/// Group cards by state.
fn group_by_state(cards: &[CardView]) -> Vec<Swimlane> {
    let state_order = [
        (CardState::Todo, "To Do", "#64748b"),             // slate-500
        (CardState::InProgress, "In Progress", "#3b82f6"), // blue-500
        (CardState::InReview, "In Review", "#f59e0b"),     // amber-500
        (CardState::Done, "Done", "#22c55e"),              // emerald-500
        (CardState::Archived, "Archived", "#475569"),      // slate-600
    ];

    state_order
        .iter()
        .map(|(state, label, color)| {
            let state_cards: Vec<CardView> = cards
                .iter()
                .filter(|c| c.state == *state)
                .cloned()
                .collect();

            Swimlane {
                key: format!("{state:?}"),
                label: (*label).to_string(),
                color: Some((*color).to_string()),
                cards: state_cards,
            }
        })
        .filter(|s| !s.cards.is_empty()) // Only show swimlanes with cards
        .collect()
}

/// Swimlane view component showing cards grouped horizontally.
#[derive(Props, Clone, PartialEq)]
struct SwimlaneViewProps {
    board: BoardViewData,
    board_id: String,
    mode: SwimlaneMode,
    on_announce: EventHandler<String>,
}

#[component]
fn SwimlaneView(props: SwimlaneViewProps) -> Element {
    let swimlanes = group_cards_into_swimlanes(&props.board, props.mode);

    if swimlanes.is_empty() {
        return rsx! {
            div {
                class: "flex-1 flex items-center justify-center text-slate-400",
                p { "No cards to display in swimlane view" }
            }
        };
    }

    rsx! {
        div {
            class: "flex-1 overflow-y-auto p-4",
            div {
                class: "flex flex-col gap-4",
                role: "list",
                aria_label: "Swimlanes",
                {swimlanes.iter().map(|swimlane| {
                    let key = swimlane.key.clone();
                    rsx! {
                        SwimlaneRow {
                            key: "{key}",
                            swimlane: swimlane.clone(),
                            board_id: props.board_id.clone(),
                            columns: props.board.columns.clone(),
                            on_announce: props.on_announce,
                        }
                    }
                })}
            }
        }
    }
}

/// A single swimlane row with its cards displayed horizontally.
#[derive(Props, Clone, PartialEq)]
struct SwimlaneRowProps {
    swimlane: Swimlane,
    board_id: String,
    columns: Vec<communitas_ui_api::kanban::ColumnView>,
    on_announce: EventHandler<String>,
}

impl PartialEq for Swimlane {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.label == other.label
            && self.color == other.color
            && self.cards == other.cards
    }
}

impl Clone for Swimlane {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            label: self.label.clone(),
            color: self.color.clone(),
            cards: self.cards.clone(),
        }
    }
}

#[component]
fn SwimlaneRow(props: SwimlaneRowProps) -> Element {
    let mut collapsed = use_signal(|| false);
    let card_count = props.swimlane.cards.len();
    let header_color = props.swimlane.color.as_deref().unwrap_or("#64748b");

    // Find the column each card belongs to (for drag-drop context)
    let find_column_for_card = |card_id: &str| -> Option<String> {
        for col in &props.columns {
            if col.cards.iter().any(|c| c.id == card_id) {
                return Some(col.id.clone());
            }
        }
        None
    };

    rsx! {
        div {
            class: "swimlane-row rounded-lg border border-slate-800 bg-slate-900/50",
            role: "listitem",
            // Swimlane header
            button {
                class: "w-full flex items-center gap-3 p-3 hover:bg-slate-800/50 transition rounded-t-lg",
                onclick: move |_| collapsed.set(!collapsed()),
                aria_expanded: if collapsed() { "false" } else { "true" },
                // Color indicator
                span {
                    class: "w-3 h-3 rounded-full flex-shrink-0",
                    style: "background-color: {header_color}",
                }
                // Label
                span {
                    class: "font-medium text-slate-200 flex-1 text-left",
                    "{props.swimlane.label}"
                }
                // Card count badge
                span {
                    class: "text-xs px-2 py-0.5 rounded-full bg-slate-700 text-slate-400",
                    "{card_count}"
                }
                // Collapse indicator
                span {
                    class: "text-slate-500 text-sm",
                    if collapsed() { "+" } else { "-" }
                }
            }
            // Cards container (horizontal scroll)
            if !collapsed() {
                div {
                    class: "overflow-x-auto border-t border-slate-800",
                    div {
                        class: "flex gap-3 p-3 min-w-max",
                        role: "list",
                        aria_label: format!("Cards in {}", props.swimlane.label),
                        {props.swimlane.cards.iter().map(|card| {
                            let card_id = card.id.clone();
                            let column_id = find_column_for_card(&card.id)
                                .unwrap_or_default();
                            rsx! {
                                div {
                                    key: "{card_id}",
                                    class: "w-72 flex-shrink-0",
                                    KanbanCard {
                                        card: card.clone(),
                                        board_id: props.board_id.clone(),
                                        column_id: column_id,
                                        on_move_announce: props.on_announce,
                                    }
                                }
                            }
                        })}
                        if props.swimlane.cards.is_empty() {
                            div {
                                class: "text-slate-500 text-sm italic py-4 px-2",
                                "No cards"
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
                span { class: "text-amber-400", "!" }
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
    use super::*;
    use communitas_ui_api::kanban::{BoardSettings, ColumnView, TagView};

    fn make_test_card(id: &str, title: &str, state: CardState) -> CardView {
        CardView {
            id: id.to_string(),
            title: title.to_string(),
            description: None,
            state,
            assignees: vec![],
            tags: vec![],
            due_date: None,
            checklist_progress: None,
            position: 0,
        }
    }

    fn make_test_board() -> BoardViewData {
        BoardViewData {
            id: "board-1".to_string(),
            name: "Test Board".to_string(),
            entity_id: "entity-1".to_string(),
            description: None,
            columns: vec![
                ColumnView {
                    id: "col-1".to_string(),
                    name: "To Do".to_string(),
                    position: 0,
                    cards: vec![
                        make_test_card("card-1", "Card 1", CardState::Todo),
                        make_test_card("card-2", "Card 2", CardState::Todo),
                    ],
                    wip_limit: None,
                },
                ColumnView {
                    id: "col-2".to_string(),
                    name: "In Progress".to_string(),
                    position: 1,
                    cards: vec![make_test_card("card-3", "Card 3", CardState::InProgress)],
                    wip_limit: None,
                },
            ],
            settings: BoardSettings::default(),
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn group_by_state_creates_correct_swimlanes() {
        let board = make_test_board();
        let all_cards: Vec<CardView> = board.columns.iter().flat_map(|c| c.cards.clone()).collect();

        let swimlanes = group_by_state(&all_cards);

        assert_eq!(swimlanes.len(), 2); // Todo and InProgress
        assert_eq!(swimlanes[0].label, "To Do");
        assert_eq!(swimlanes[0].cards.len(), 2);
        assert_eq!(swimlanes[1].label, "In Progress");
        assert_eq!(swimlanes[1].cards.len(), 1);
    }

    #[test]
    fn group_by_assignee_handles_unassigned() {
        let cards = vec![
            make_test_card("card-1", "Unassigned Card", CardState::Todo),
            {
                let mut card = make_test_card("card-2", "Assigned Card", CardState::Todo);
                card.assignees = vec!["alice".to_string()];
                card
            },
        ];

        let swimlanes = group_by_assignee(&cards);

        assert_eq!(swimlanes.len(), 2);
        assert_eq!(swimlanes[0].label, "alice");
        assert_eq!(swimlanes[0].cards.len(), 1);
        assert_eq!(swimlanes[1].label, "Unassigned");
        assert_eq!(swimlanes[1].cards.len(), 1);
    }

    #[test]
    fn group_by_tag_handles_no_tag() {
        let cards = vec![make_test_card("card-1", "No Tag Card", CardState::Todo), {
            let mut card = make_test_card("card-2", "Tagged Card", CardState::Todo);
            card.tags = vec![TagView {
                id: "tag-1".to_string(),
                name: "Bug".to_string(),
                color: "#ff0000".to_string(),
            }];
            card
        }];

        let swimlanes = group_by_tag(&cards);

        assert_eq!(swimlanes.len(), 2);
        assert_eq!(swimlanes[0].label, "Bug");
        assert_eq!(swimlanes[0].cards.len(), 1);
        assert_eq!(swimlanes[1].label, "No Tag");
        assert_eq!(swimlanes[1].cards.len(), 1);
    }

    #[test]
    fn empty_cards_return_empty_swimlanes() {
        let cards: Vec<CardView> = vec![];

        let by_state = group_by_state(&cards);
        let by_assignee = group_by_assignee(&cards);
        let by_tag = group_by_tag(&cards);

        assert!(by_state.is_empty());
        assert!(by_assignee.is_empty());
        assert!(by_tag.is_empty());
    }

    #[test]
    fn board_header_renders_name() {
        let name = "Test Board".to_string();
        let desc = Some("A test description".to_string());
        assert_eq!(name, "Test Board");
        assert_eq!(desc, Some("A test description".to_string()));
    }
}
