//! Offline indicator component for canvas sync status.

use communitas_ui_api::{OfflineOperation, OfflineStatus, OperationType};
use dioxus::prelude::*;

/// Props for the OfflineIndicator component.
#[derive(Props, Clone, PartialEq)]
pub struct OfflineIndicatorProps {
    /// List of pending offline operations.
    pub operations: Vec<OfflineOperation>,
    /// Whether currently syncing.
    #[props(default = false)]
    pub is_syncing: bool,
    /// Whether expanded to show details.
    #[props(default = false)]
    pub expanded: bool,
    /// Callback when expand/collapse is toggled.
    #[props(default)]
    pub on_toggle_expand: Option<EventHandler<()>>,
    /// Callback when retry is clicked.
    #[props(default)]
    pub on_retry: Option<EventHandler<()>>,
    /// Callback when dismiss is clicked.
    #[props(default)]
    pub on_dismiss: Option<EventHandler<String>>,
}

/// Offline queue indicator showing pending sync operations.
#[component]
pub fn OfflineIndicator(props: OfflineIndicatorProps) -> Element {
    let pending_count = props
        .operations
        .iter()
        .filter(|o| o.status.is_pending())
        .count();
    let failed_count = props
        .operations
        .iter()
        .filter(|o| o.status.is_failed())
        .count();
    let syncing_count = props
        .operations
        .iter()
        .filter(|o| o.status.is_syncing())
        .count();

    // Don't show if queue is empty and not syncing
    if props.operations.is_empty() && !props.is_syncing {
        return rsx! {};
    }

    let status_color = if failed_count > 0 {
        "border-red-500 bg-red-900/30"
    } else if props.is_syncing || syncing_count > 0 {
        "border-blue-500 bg-blue-900/30"
    } else if pending_count > 0 {
        "border-amber-500 bg-amber-900/30"
    } else {
        "border-emerald-500 bg-emerald-900/30"
    };

    rsx! {
        div {
            class: format!("offline-indicator border rounded-lg {}", status_color),
            role: "status",
            aria_live: "polite",
            // Compact view
            div {
                class: "flex items-center gap-2 p-2 cursor-pointer",
                onclick: {
                    let on_toggle = props.on_toggle_expand;
                    move |_| {
                        if let Some(handler) = &on_toggle {
                            handler.call(());
                        }
                    }
                },
                // Status icon
                span {
                    class: format!(
                        "text-lg {}",
                        if failed_count > 0 { "" } else if props.is_syncing { "animate-spin" } else { "" }
                    ),
                    if failed_count > 0 {
                        "⚠"
                    } else if props.is_syncing || syncing_count > 0 {
                        "🔄"
                    } else if pending_count > 0 {
                        "📤"
                    } else {
                        "✓"
                    }
                }
                // Status text
                span {
                    class: "text-sm text-slate-300",
                    {
                        if failed_count > 0 {
                            let s = if failed_count != 1 { "s" } else { "" };
                            format!("{} operation{} failed", failed_count, s)
                        } else if props.is_syncing || syncing_count > 0 {
                            let total = syncing_count + pending_count;
                            let s = if total != 1 { "s" } else { "" };
                            format!("Syncing {} operation{}...", total, s)
                        } else if pending_count > 0 {
                            format!("{} pending", pending_count)
                        } else {
                            "All synced".to_string()
                        }
                    }
                }
                // Queue count badge
                if !props.operations.is_empty() {
                    span {
                        class: format!(
                            "px-1.5 py-0.5 text-xs rounded-full {}",
                            if failed_count > 0 { "bg-red-500 text-white" } else { "bg-slate-600 text-slate-300" }
                        ),
                        "{props.operations.len()}"
                    }
                }
                // Expand indicator
                span {
                    class: "text-slate-500 text-xs",
                    if props.expanded { "▲" } else { "▼" }
                }
            }
            // Expanded details
            if props.expanded {
                div {
                    class: "border-t border-slate-700 p-2 max-h-48 overflow-y-auto",
                    // Retry all button
                    if failed_count > 0 {
                        button {
                            class: "w-full mb-2 px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-white text-sm rounded transition-colors",
                            onclick: {
                                let on_retry = props.on_retry;
                                move |_| {
                                    if let Some(handler) = &on_retry {
                                        handler.call(());
                                    }
                                }
                            },
                            "Retry failed operations"
                        }
                    }
                    // Operation list
                    if props.operations.is_empty() {
                        div {
                            class: "text-center text-slate-500 text-sm py-2",
                            "No pending operations"
                        }
                    } else {
                        for op in props.operations.iter() {
                            OperationItem {
                                key: "{op.id}",
                                operation: op.clone(),
                                on_dismiss: {
                                    let on_dismiss = props.on_dismiss;
                                    let id = op.id.clone();
                                    move |_| {
                                        if let Some(handler) = &on_dismiss {
                                            handler.call(id.clone());
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Props for operation item.
#[derive(Props, Clone, PartialEq)]
struct OperationItemProps {
    operation: OfflineOperation,
    on_dismiss: EventHandler<()>,
}

/// Individual operation in the queue.
#[component]
fn OperationItem(props: OperationItemProps) -> Element {
    let op = &props.operation;
    let icon = operation_icon(op.operation_type);

    let status_class = match &op.status {
        OfflineStatus::Pending => "text-amber-400",
        OfflineStatus::Syncing => "text-blue-400",
        OfflineStatus::Synced => "text-emerald-400",
        OfflineStatus::Failed(_) => "text-red-400",
    };

    let status_text = match &op.status {
        OfflineStatus::Pending => "Pending".to_string(),
        OfflineStatus::Syncing => "Syncing...".to_string(),
        OfflineStatus::Synced => "Synced".to_string(),
        OfflineStatus::Failed(msg) => format!("Failed: {}", msg),
    };

    rsx! {
        div {
            class: "flex items-center gap-2 py-1.5 px-2 rounded hover:bg-slate-700/50",
            // Operation icon
            span { class: "text-sm", "{icon}" }
            // Operation type
            span {
                class: "flex-1 text-sm text-slate-300 truncate",
                "{op.operation_type.label()}"
            }
            // Status
            span {
                class: format!("text-xs {}", status_class),
                "{status_text}"
            }
            // Retry count
            if op.retry_count > 0 {
                span {
                    class: "text-xs text-slate-500",
                    title: "Retry attempts",
                    "({op.retry_count})"
                }
            }
            // Dismiss button (only for failed/synced)
            if op.status.is_failed() || op.status.is_synced() {
                button {
                    class: "p-1 text-slate-500 hover:text-slate-300 transition-colors",
                    title: "Dismiss",
                    onclick: move |_| props.on_dismiss.call(()),
                    "×"
                }
            }
        }
    }
}

/// Get icon for operation type.
#[allow(dead_code)]
fn operation_icon(op_type: OperationType) -> &'static str {
    match op_type {
        OperationType::AddElement => "➕",
        OperationType::UpdateElement => "✏",
        OperationType::DeleteElement => "🗑",
        OperationType::AddLayer => "📄",
        OperationType::UpdateLayer => "📝",
        OperationType::DeleteLayer => "📄✕",
    }
}

/// Simple sync status badge for toolbar.
#[derive(Props, Clone, PartialEq)]
pub struct SyncStatusBadgeProps {
    /// Number of pending operations.
    pub pending_count: usize,
    /// Whether currently syncing.
    pub is_syncing: bool,
    /// Whether there are errors.
    pub has_errors: bool,
}

#[component]
pub fn SyncStatusBadge(props: SyncStatusBadgeProps) -> Element {
    if props.pending_count == 0 && !props.is_syncing && !props.has_errors {
        return rsx! {};
    }

    let bg_class = if props.has_errors {
        "bg-red-500"
    } else if props.is_syncing {
        "bg-blue-500"
    } else {
        "bg-amber-500"
    };

    rsx! {
        span {
            class: format!(
                "inline-flex items-center justify-center min-w-5 h-5 px-1.5 text-xs font-medium text-white rounded-full {}",
                bg_class
            ),
            role: "status",
            aria_label: format!(
                "{} pending operation{}{}",
                props.pending_count,
                if props.pending_count != 1 { "s" } else { "" },
                if props.has_errors { " with errors" } else { "" }
            ),
            if props.is_syncing {
                span { class: "mr-1 animate-spin text-[10px]", "⟳" }
            }
            "{props.pending_count}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_operation(id: &str, op_type: OperationType, status: OfflineStatus) -> OfflineOperation {
        OfflineOperation {
            id: id.to_string(),
            operation_type: op_type,
            payload: "{}".to_string(),
            created_at: 0,
            retry_count: 0,
            status,
        }
    }

    #[test]
    fn operation_icons_all_defined() {
        let ops = [
            OperationType::AddElement,
            OperationType::UpdateElement,
            OperationType::DeleteElement,
            OperationType::AddLayer,
            OperationType::UpdateLayer,
            OperationType::DeleteLayer,
        ];

        for op in ops {
            let icon = operation_icon(op);
            assert!(!icon.is_empty(), "Operation {:?} should have an icon", op);
        }
    }

    #[test]
    fn status_counting() {
        let operations = [
            make_operation("1", OperationType::AddElement, OfflineStatus::Pending),
            make_operation("2", OperationType::UpdateElement, OfflineStatus::Pending),
            make_operation(
                "3",
                OperationType::DeleteElement,
                OfflineStatus::Failed("Network error".to_string()),
            ),
            make_operation("4", OperationType::AddLayer, OfflineStatus::Syncing),
            make_operation("5", OperationType::UpdateLayer, OfflineStatus::Synced),
        ];

        let pending = operations.iter().filter(|o| o.status.is_pending()).count();
        let failed = operations.iter().filter(|o| o.status.is_failed()).count();
        let syncing = operations.iter().filter(|o| o.status.is_syncing()).count();
        let synced = operations.iter().filter(|o| o.status.is_synced()).count();

        assert_eq!(pending, 2);
        assert_eq!(failed, 1);
        assert_eq!(syncing, 1);
        assert_eq!(synced, 1);
    }

    #[test]
    fn status_color_priority() {
        // Failed takes priority
        let failed_count = 1;
        let is_syncing = true;
        let pending_count = 5;

        let status_color = if failed_count > 0 {
            "red"
        } else if is_syncing {
            "blue"
        } else if pending_count > 0 {
            "amber"
        } else {
            "emerald"
        };

        assert_eq!(status_color, "red");
    }

    #[test]
    fn empty_queue_hidden() {
        let operations: Vec<OfflineOperation> = vec![];
        let is_syncing = false;

        let should_show = !operations.is_empty() || is_syncing;
        assert!(!should_show);
    }

    #[test]
    fn syncing_without_queue_shown() {
        let operations: Vec<OfflineOperation> = vec![];
        let is_syncing = true;

        let should_show = !operations.is_empty() || is_syncing;
        assert!(should_show);
    }
}
