//! Filter panel for Kanban board swimlane and card filtering.

use communitas_ui_api::kanban::{CardState, PriorityView, SwimlaneMode};
use dioxus::prelude::*;

/// Represents the active filters for a board.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BoardFilters {
    /// Filter by assignee IDs.
    pub assignees: Vec<String>,
    /// Filter by tag IDs.
    pub tags: Vec<String>,
    /// Filter by due date status.
    pub due_date: Option<DueDateFilter>,
    /// Filter by card state.
    pub states: Vec<CardState>,
    /// Filter by priority levels.
    pub priorities: Vec<PriorityView>,
    /// Search text filter.
    pub search: Option<String>,
}

/// Due date filter options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DueDateFilter {
    Overdue,
    DueSoon,
    NoDate,
}

impl BoardFilters {
    /// Returns true if any filters are active.
    pub fn has_active_filters(&self) -> bool {
        !self.assignees.is_empty()
            || !self.tags.is_empty()
            || self.due_date.is_some()
            || !self.states.is_empty()
            || !self.priorities.is_empty()
            || self.search.is_some()
    }

    /// Count of active filter categories.
    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if !self.assignees.is_empty() {
            count += 1;
        }
        if !self.tags.is_empty() {
            count += 1;
        }
        if self.due_date.is_some() {
            count += 1;
        }
        if !self.states.is_empty() {
            count += 1;
        }
        if !self.priorities.is_empty() {
            count += 1;
        }
        if self.search.is_some() {
            count += 1;
        }
        count
    }
}

/// Filter panel component for board views.
#[derive(Props, Clone, PartialEq)]
pub struct FilterPanelProps {
    /// Callback when filters change.
    #[props(optional)]
    pub on_filter_change: Option<EventHandler<BoardFilters>>,
    /// Callback when swimlane mode changes.
    #[props(optional)]
    pub on_swimlane_change: Option<EventHandler<SwimlaneMode>>,
    /// Current swimlane mode.
    #[props(optional, default = SwimlaneMode::None)]
    pub current_swimlane: SwimlaneMode,
}

#[component]
pub fn FilterPanel(props: FilterPanelProps) -> Element {
    // Filter state
    let mut filters = use_signal(BoardFilters::default);

    // Available options (would come from board data in real implementation)
    let available_assignees = [
        ("user-1".to_string(), "Alice".to_string()),
        ("user-2".to_string(), "Bob".to_string()),
        ("user-3".to_string(), "Charlie".to_string()),
    ];
    let available_tags = [
        ("tag-1".to_string(), "Bug", "#ef4444"),
        ("tag-2".to_string(), "Feature", "#22c55e"),
        ("tag-3".to_string(), "Urgent", "#f59e0b"),
    ];

    // Search input state
    let mut search_input = use_signal(String::new);

    let notify_change = {
        let on_filter_change = props.on_filter_change;
        move |new_filters: BoardFilters| {
            if let Some(handler) = &on_filter_change {
                handler.call(new_filters);
            }
        }
    };

    let on_swimlane_change = props.on_swimlane_change;

    rsx! {
        div {
            class: "filter-panel border-b border-slate-800 bg-slate-900/50 p-4",
            role: "region",
            aria_label: "Board filters",
            // Search row
            div {
                class: "flex items-center gap-4 mb-4",
                // Search input
                div {
                    class: "flex-1 max-w-xs",
                    input {
                        r#type: "search",
                        class: "w-full rounded-lg border border-slate-700 bg-slate-800 px-3 py-2 text-sm text-slate-200 placeholder-slate-500 focus:border-emerald-400 focus:outline-none",
                        placeholder: "Search cards...",
                        value: "{search_input}",
                        oninput: move |evt| {
                            let value = evt.value();
                            search_input.set(value.clone());
                            filters.with_mut(|f| {
                                f.search = if value.is_empty() { None } else { Some(value) };
                            });
                            notify_change(filters());
                        },
                    }
                }
                // Swimlane toggle
                div {
                    class: "flex items-center gap-2",
                    span { class: "text-sm text-slate-400", "View:" }
                    SwimlaneSelector {
                        current: props.current_swimlane,
                        on_change: move |mode| {
                            if let Some(handler) = &on_swimlane_change {
                                handler.call(mode);
                            }
                        },
                    }
                }
                // Clear all button
                if filters().has_active_filters() {
                    button {
                        class: "text-sm text-slate-400 hover:text-emerald-400",
                        onclick: move |_| {
                            filters.set(BoardFilters::default());
                            search_input.set(String::new());
                            notify_change(BoardFilters::default());
                        },
                        "Clear filters ({filters().active_count()})"
                    }
                }
            }
            // Filter chips row
            div {
                class: "flex flex-wrap gap-4",
                // Assignee filter
                FilterGroup {
                    label: "Assignees",
                    options: available_assignees.iter().map(|(id, name)| {
                        FilterOption {
                            id: id.clone(),
                            label: name.clone(),
                            color: None,
                        }
                    }).collect(),
                    selected: filters().assignees.clone(),
                    on_toggle: move |id: String| {
                        filters.with_mut(|f| {
                            if f.assignees.contains(&id) {
                                f.assignees.retain(|a| a != &id);
                            } else {
                                f.assignees.push(id);
                            }
                        });
                        notify_change(filters());
                    },
                }
                // Tag filter
                FilterGroup {
                    label: "Tags",
                    options: available_tags.iter().map(|(id, name, color)| {
                        FilterOption {
                            id: id.clone(),
                            label: name.to_string(),
                            color: Some(color.to_string()),
                        }
                    }).collect(),
                    selected: filters().tags.clone(),
                    on_toggle: move |id: String| {
                        filters.with_mut(|f| {
                            if f.tags.contains(&id) {
                                f.tags.retain(|t| t != &id);
                            } else {
                                f.tags.push(id);
                            }
                        });
                        notify_change(filters());
                    },
                }
                // Due date filter
                DueDateFilterGroup {
                    selected: filters().due_date,
                    on_change: move |filter| {
                        filters.with_mut(|f| {
                            f.due_date = filter;
                        });
                        notify_change(filters());
                    },
                }
                // State filter
                StateFilterGroup {
                    selected: filters().states.clone(),
                    on_toggle: move |state: CardState| {
                        filters.with_mut(|f| {
                            if f.states.contains(&state) {
                                f.states.retain(|s| s != &state);
                            } else {
                                f.states.push(state);
                            }
                        });
                        notify_change(filters());
                    },
                }
                // Priority filter
                PriorityFilterGroup {
                    selected: filters().priorities.clone(),
                    on_toggle: move |priority: PriorityView| {
                        filters.with_mut(|f| {
                            if f.priorities.contains(&priority) {
                                f.priorities.retain(|p| p != &priority);
                            } else {
                                f.priorities.push(priority);
                            }
                        });
                        notify_change(filters());
                    },
                }
            }
        }
    }
}

/// Swimlane view selector.
#[derive(Props, Clone, PartialEq)]
struct SwimlaneSelectorProps {
    current: SwimlaneMode,
    on_change: EventHandler<SwimlaneMode>,
}

#[component]
fn SwimlaneSelector(props: SwimlaneSelectorProps) -> Element {
    let modes = [
        (SwimlaneMode::None, "Standard"),
        (SwimlaneMode::ByAssignee, "By Assignee"),
        (SwimlaneMode::ByTag, "By Tag"),
        (SwimlaneMode::ByState, "By State"),
    ];

    rsx! {
        div {
            class: "flex rounded-lg border border-slate-700 overflow-hidden",
            {modes.iter().map(|(mode, label)| {
                let is_selected = *mode == props.current;
                let mode = *mode;
                let on_change = props.on_change;
                rsx! {
                    button {
                        key: "{label}",
                        class: format!(
                            "px-3 py-1.5 text-xs transition {}",
                            if is_selected {
                                "bg-emerald-500/20 text-emerald-400"
                            } else {
                                "bg-slate-800 text-slate-400 hover:bg-slate-700"
                            }
                        ),
                        onclick: move |_| on_change.call(mode),
                        "{label}"
                    }
                }
            })}
        }
    }
}

/// Single filter option.
#[derive(Clone, PartialEq)]
struct FilterOption {
    id: String,
    label: String,
    color: Option<String>,
}

/// Filter group component.
#[derive(Props, Clone, PartialEq)]
struct FilterGroupProps {
    label: &'static str,
    options: Vec<FilterOption>,
    selected: Vec<String>,
    on_toggle: EventHandler<String>,
}

#[component]
fn FilterGroup(props: FilterGroupProps) -> Element {
    let mut expanded = use_signal(|| false);
    let selected_count = props.selected.len();

    rsx! {
        div {
            class: "relative",
            // Trigger button
            button {
                class: format!(
                    "flex items-center gap-1 rounded-lg px-3 py-1.5 text-sm transition {}",
                    if selected_count > 0 {
                        "bg-emerald-500/20 text-emerald-400 border border-emerald-500/50"
                    } else {
                        "bg-slate-800 text-slate-400 hover:bg-slate-700"
                    }
                ),
                onclick: move |_| expanded.set(!expanded()),
                "{props.label}"
                if selected_count > 0 {
                    span {
                        class: "ml-1 text-xs",
                        "({selected_count})"
                    }
                }
                span {
                    class: "text-xs ml-1",
                    if expanded() { "^" } else { "v" }
                }
            }
            // Dropdown
            if expanded() {
                div {
                    class: "absolute top-full left-0 mt-1 min-w-32 rounded-lg border border-slate-700 bg-slate-800 shadow-lg z-20",
                    div {
                        class: "py-1 max-h-48 overflow-y-auto",
                        {props.options.iter().map(|opt| {
                            let is_selected = props.selected.contains(&opt.id);
                            let id = opt.id.clone();
                            let on_toggle = props.on_toggle;
                            rsx! {
                                button {
                                    key: "{id}",
                                    class: format!(
                                        "w-full px-3 py-1.5 text-left text-sm flex items-center gap-2 {}",
                                        if is_selected { "bg-slate-700 text-white" } else { "text-slate-300 hover:bg-slate-700" }
                                    ),
                                    onclick: move |_| on_toggle.call(id.clone()),
                                    // Checkbox indicator
                                    span {
                                        class: format!(
                                            "w-4 h-4 rounded border flex items-center justify-center {}",
                                            if is_selected { "bg-emerald-500 border-emerald-500" } else { "border-slate-600" }
                                        ),
                                        if is_selected {
                                            span { class: "text-xs text-white", "Y" }
                                        }
                                    }
                                    // Color dot if present
                                    if let Some(color) = &opt.color {
                                        span {
                                            class: "w-3 h-3 rounded-full",
                                            style: "background-color: {color}",
                                        }
                                    }
                                    "{opt.label}"
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}

/// Due date filter group.
#[derive(Props, Clone, PartialEq)]
struct DueDateFilterGroupProps {
    selected: Option<DueDateFilter>,
    on_change: EventHandler<Option<DueDateFilter>>,
}

#[component]
fn DueDateFilterGroup(props: DueDateFilterGroupProps) -> Element {
    let mut expanded = use_signal(|| false);

    let options = [
        (Some(DueDateFilter::Overdue), "Overdue", "text-red-400"),
        (Some(DueDateFilter::DueSoon), "Due soon", "text-amber-400"),
        (Some(DueDateFilter::NoDate), "No date", "text-slate-400"),
        (None, "Any", "text-slate-400"),
    ];

    let current_label = options
        .iter()
        .find(|(v, _, _)| *v == props.selected)
        .map(|(_, l, _)| *l)
        .unwrap_or("Due date");

    rsx! {
        div {
            class: "relative",
            button {
                class: format!(
                    "flex items-center gap-1 rounded-lg px-3 py-1.5 text-sm transition {}",
                    if props.selected.is_some() {
                        "bg-emerald-500/20 text-emerald-400 border border-emerald-500/50"
                    } else {
                        "bg-slate-800 text-slate-400 hover:bg-slate-700"
                    }
                ),
                onclick: move |_| expanded.set(!expanded()),
                "{current_label}"
                span {
                    class: "text-xs ml-1",
                    if expanded() { "^" } else { "v" }
                }
            }
            if expanded() {
                div {
                    class: "absolute top-full left-0 mt-1 min-w-32 rounded-lg border border-slate-700 bg-slate-800 shadow-lg z-20",
                    div {
                        class: "py-1",
                        {options.iter().map(|(value, label, color)| {
                            let is_selected = *value == props.selected;
                            let value = *value;
                            let on_change = props.on_change;
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: format!(
                                        "w-full px-3 py-1.5 text-left text-sm {} {}",
                                        color,
                                        if is_selected { "bg-slate-700" } else { "hover:bg-slate-700" }
                                    ),
                                    onclick: move |_| {
                                        on_change.call(value);
                                        expanded.set(false);
                                    },
                                    "{label}"
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}

/// State filter group.
#[derive(Props, Clone, PartialEq)]
struct StateFilterGroupProps {
    selected: Vec<CardState>,
    on_toggle: EventHandler<CardState>,
}

#[component]
fn StateFilterGroup(props: StateFilterGroupProps) -> Element {
    let mut expanded = use_signal(|| false);
    let selected_count = props.selected.len();

    let states = [
        (CardState::Todo, "To Do", "bg-slate-600"),
        (CardState::InProgress, "In Progress", "bg-blue-600"),
        (CardState::InReview, "In Review", "bg-amber-600"),
        (CardState::Done, "Done", "bg-emerald-600"),
    ];

    rsx! {
        div {
            class: "relative",
            button {
                class: format!(
                    "flex items-center gap-1 rounded-lg px-3 py-1.5 text-sm transition {}",
                    if selected_count > 0 {
                        "bg-emerald-500/20 text-emerald-400 border border-emerald-500/50"
                    } else {
                        "bg-slate-800 text-slate-400 hover:bg-slate-700"
                    }
                ),
                onclick: move |_| expanded.set(!expanded()),
                "State"
                if selected_count > 0 {
                    span {
                        class: "ml-1 text-xs",
                        "({selected_count})"
                    }
                }
                span {
                    class: "text-xs ml-1",
                    if expanded() { "^" } else { "v" }
                }
            }
            if expanded() {
                div {
                    class: "absolute top-full left-0 mt-1 min-w-32 rounded-lg border border-slate-700 bg-slate-800 shadow-lg z-20",
                    div {
                        class: "py-1",
                        {states.iter().map(|(state, label, color)| {
                            let is_selected = props.selected.contains(state);
                            let state = *state;
                            let on_toggle = props.on_toggle;
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: format!(
                                        "w-full px-3 py-1.5 text-left text-sm flex items-center gap-2 {}",
                                        if is_selected { "bg-slate-700 text-white" } else { "text-slate-300 hover:bg-slate-700" }
                                    ),
                                    onclick: move |_| on_toggle.call(state),
                                    span {
                                        class: format!(
                                            "w-4 h-4 rounded border flex items-center justify-center {}",
                                            if is_selected { "bg-emerald-500 border-emerald-500" } else { "border-slate-600" }
                                        ),
                                        if is_selected {
                                            span { class: "text-xs text-white", "Y" }
                                        }
                                    }
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
}

/// Priority filter group.
#[derive(Props, Clone, PartialEq)]
struct PriorityFilterGroupProps {
    selected: Vec<PriorityView>,
    on_toggle: EventHandler<PriorityView>,
}

#[component]
fn PriorityFilterGroup(props: PriorityFilterGroupProps) -> Element {
    let mut expanded = use_signal(|| false);
    let selected_count = props.selected.len();

    let priorities = [
        (PriorityView::Urgent, "Urgent", "bg-red-500"),
        (PriorityView::High, "High", "bg-orange-500"),
        (PriorityView::Normal, "Normal", "bg-blue-500"),
        (PriorityView::Low, "Low", "bg-slate-500"),
    ];

    rsx! {
        div {
            class: "relative",
            button {
                class: format!(
                    "flex items-center gap-1 rounded-lg px-3 py-1.5 text-sm transition {}",
                    if selected_count > 0 {
                        "bg-emerald-500/20 text-emerald-400 border border-emerald-500/50"
                    } else {
                        "bg-slate-800 text-slate-400 hover:bg-slate-700"
                    }
                ),
                onclick: move |_| expanded.set(!expanded()),
                "Priority"
                if selected_count > 0 {
                    span {
                        class: "ml-1 text-xs",
                        "({selected_count})"
                    }
                }
                span {
                    class: "text-xs ml-1",
                    if expanded() { "^" } else { "v" }
                }
            }
            if expanded() {
                div {
                    class: "absolute top-full left-0 mt-1 min-w-32 rounded-lg border border-slate-700 bg-slate-800 shadow-lg z-20",
                    div {
                        class: "py-1",
                        {priorities.iter().map(|(priority, label, color)| {
                            let is_selected = props.selected.contains(priority);
                            let priority = *priority;
                            let on_toggle = props.on_toggle;
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: format!(
                                        "w-full px-3 py-1.5 text-left text-sm flex items-center gap-2 {}",
                                        if is_selected { "bg-slate-700 text-white" } else { "text-slate-300 hover:bg-slate-700" }
                                    ),
                                    onclick: move |_| on_toggle.call(priority),
                                    span {
                                        class: format!(
                                            "w-4 h-4 rounded border flex items-center justify-center {}",
                                            if is_selected { "bg-emerald-500 border-emerald-500" } else { "border-slate-600" }
                                        ),
                                        if is_selected {
                                            span { class: "text-xs text-white", "Y" }
                                        }
                                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_filters_has_active_filters() {
        let empty = BoardFilters::default();
        assert!(!empty.has_active_filters());

        let with_assignee = BoardFilters {
            assignees: vec!["user-1".to_string()],
            ..Default::default()
        };
        assert!(with_assignee.has_active_filters());
    }

    #[test]
    fn board_filters_active_count() {
        let filters = BoardFilters {
            assignees: vec!["user-1".to_string()],
            tags: vec!["tag-1".to_string()],
            due_date: Some(DueDateFilter::Overdue),
            states: vec![],
            priorities: vec![],
            search: None,
        };
        assert_eq!(filters.active_count(), 3);
    }

    #[test]
    fn swimlane_mode_default() {
        let mode = SwimlaneMode::default();
        assert_eq!(mode, SwimlaneMode::None);
    }

    #[test]
    fn due_date_filter_variants() {
        assert_ne!(DueDateFilter::Overdue, DueDateFilter::DueSoon);
        assert_ne!(DueDateFilter::DueSoon, DueDateFilter::NoDate);
    }

    #[test]
    fn board_filters_default_has_no_active() {
        let filters = BoardFilters::default();
        assert_eq!(filters.active_count(), 0);
        assert!(!filters.has_active_filters());
        assert!(filters.assignees.is_empty());
        assert!(filters.tags.is_empty());
        assert!(filters.states.is_empty());
        assert!(filters.priorities.is_empty());
        assert!(filters.due_date.is_none());
        assert!(filters.search.is_none());
    }

    #[test]
    fn board_filters_with_priorities() {
        let filters = BoardFilters {
            priorities: vec![PriorityView::Urgent, PriorityView::High],
            ..Default::default()
        };
        assert!(filters.has_active_filters());
        assert_eq!(filters.active_count(), 1);
        assert_eq!(filters.priorities.len(), 2);
    }
}
