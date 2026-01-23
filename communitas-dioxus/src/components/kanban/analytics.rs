//! Analytics dashboard UI components for Kanban boards.
//!
//! Provides visual charts and metrics for board performance tracking.

use dioxus::prelude::*;

use communitas_ui_service::UiServices;
use communitas_ui_service::kanban::{
    BoardAnalytics, BurndownChart, CycleTimeDistribution, TimeRange, VelocityMetric,
};

/// Props for the AnalyticsDashboard component.
#[derive(Props, Clone, PartialEq)]
pub struct AnalyticsDashboardProps {
    /// Board ID to show analytics for.
    board_id: String,
    /// Board name for display.
    board_name: String,
    /// Handler to close the dashboard.
    on_close: EventHandler<()>,
}

/// Analytics dashboard showing board metrics and charts.
#[component]
pub fn AnalyticsDashboard(props: AnalyticsDashboardProps) -> Element {
    let services = use_context::<UiServices>();

    // Selected time range for charts
    let mut time_range = use_signal(|| TimeRange::Weeks(4));

    // Analytics data states
    let mut analytics = use_signal(|| Option::<BoardAnalytics>::None);
    let mut velocity = use_signal(|| Option::<VelocityMetric>::None);
    let mut cycle_time = use_signal(|| Option::<CycleTimeDistribution>::None);
    let mut burndown = use_signal(|| Option::<BurndownChart>::None);

    // Loading state
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| Option::<String>::None);

    // Load analytics data on mount and when time range changes
    let board_id = props.board_id.clone();
    let board_id_for_effect = board_id.clone();
    let services_for_effect = services.clone();

    use_effect(move || {
        let board_id = board_id_for_effect.clone();
        let services = services_for_effect.clone();
        let current_range = *time_range.read();

        spawn(async move {
            loading.set(true);
            error_msg.set(None);

            // Fetch board analytics
            match services.kanban().get_board_analytics(&board_id).await {
                Ok(data) => analytics.set(Some(data)),
                Err(e) => {
                    error_msg.set(Some(format!("Failed to load analytics: {}", e)));
                    loading.set(false);
                    return;
                }
            }

            // Fetch velocity
            match services
                .kanban()
                .calculate_velocity(&board_id, current_range)
                .await
            {
                Ok(data) => velocity.set(Some(data)),
                Err(e) => tracing::warn!("Failed to load velocity: {}", e),
            }

            // Fetch cycle time distribution
            match services
                .kanban()
                .calculate_cycle_time_distribution(&board_id, current_range)
                .await
            {
                Ok(data) => cycle_time.set(Some(data)),
                Err(e) => tracing::warn!("Failed to load cycle time: {}", e),
            }

            // Fetch burndown (use time range timestamps)
            let now = chrono::Utc::now().timestamp();
            let (start, end) = current_range.to_timestamps(now);
            match services
                .kanban()
                .calculate_burndown(&board_id, start, end)
                .await
            {
                Ok(data) => burndown.set(Some(data)),
                Err(e) => tracing::warn!("Failed to load burndown: {}", e),
            }

            loading.set(false);
        });
    });

    // Time range change handler
    let on_range_change = move |range: TimeRange| {
        time_range.set(range);
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| props.on_close.call(()),

            // Dashboard modal
            div {
                class: "bg-gray-900 rounded-lg shadow-xl max-w-6xl w-full mx-4 max-h-[90vh] overflow-hidden flex flex-col",
                onclick: move |e| e.stop_propagation(),

                // Header
                header {
                    class: "flex items-center justify-between px-6 py-4 border-b border-gray-700",

                    h2 {
                        class: "text-xl font-semibold text-white",
                        "Analytics: {props.board_name}"
                    }

                    div {
                        class: "flex items-center gap-4",

                        // Time range selector
                        TimeRangeSelector {
                            current: *time_range.read(),
                            on_change: on_range_change,
                        }

                        // Close button
                        button {
                            class: "p-2 text-gray-400 hover:text-white hover:bg-gray-700 rounded transition-colors",
                            onclick: move |_| props.on_close.call(()),
                            title: "Close",
                            "✕"
                        }
                    }
                }

                // Content
                main {
                    class: "flex-1 overflow-y-auto p-6",

                    if *loading.read() {
                        div {
                            class: "flex items-center justify-center h-64",
                            div {
                                class: "text-gray-400",
                                "Loading analytics..."
                            }
                        }
                    } else if let Some(err) = error_msg.read().as_ref() {
                        div {
                            class: "bg-red-900/30 border border-red-500 rounded p-4 text-red-400",
                            "{err}"
                        }
                    } else {
                        // Metrics overview
                        if let Some(ref data) = *analytics.read() {
                            MetricsOverview { analytics: data.clone() }
                        }

                        // Charts grid
                        div {
                            class: "grid grid-cols-1 lg:grid-cols-2 gap-6 mt-6",

                            // Velocity chart
                            if let Some(ref data) = *velocity.read() {
                                VelocityChart { velocity: data.clone() }
                            }

                            // Burndown chart
                            if let Some(ref data) = *burndown.read() {
                                BurndownChartView { burndown: data.clone() }
                            }

                            // Cycle time histogram
                            if let Some(ref data) = *cycle_time.read() {
                                CycleTimeChart { distribution: data.clone() }
                            }

                            // Column distribution
                            if let Some(ref data) = *analytics.read() {
                                ColumnDistribution { analytics: data.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Time range selector component.
#[derive(Props, Clone, PartialEq)]
struct TimeRangeSelectorProps {
    /// Currently selected range.
    current: TimeRange,
    /// Handler for range changes.
    on_change: EventHandler<TimeRange>,
}

#[component]
fn TimeRangeSelector(props: TimeRangeSelectorProps) -> Element {
    let options = [
        (TimeRange::Days(7), "7 Days"),
        (TimeRange::Weeks(2), "2 Weeks"),
        (TimeRange::Weeks(4), "4 Weeks"),
        (TimeRange::Months(3), "3 Months"),
    ];

    rsx! {
        div {
            class: "flex items-center gap-2",

            span {
                class: "text-sm text-gray-400",
                "Period:"
            }

            div {
                class: "flex rounded bg-gray-800",

                for (range, label) in options {
                    button {
                        class: if props.current == range {
                            "px-3 py-1.5 text-sm rounded bg-blue-600 text-white"
                        } else {
                            "px-3 py-1.5 text-sm text-gray-400 hover:text-white hover:bg-gray-700"
                        },
                        onclick: move |_| props.on_change.call(range),
                        "{label}"
                    }
                }
            }
        }
    }
}

/// Metrics overview cards.
#[derive(Props, Clone, PartialEq)]
struct MetricsOverviewProps {
    /// Board analytics data.
    analytics: BoardAnalytics,
}

#[component]
fn MetricsOverview(props: MetricsOverviewProps) -> Element {
    let data = &props.analytics;

    // Format cycle time
    let cycle_time_str = data
        .avg_cycle_time_seconds
        .map(format_duration)
        .unwrap_or_else(|| "—".to_string());

    let overdue_color = if data.overdue_count > 0 {
        "red".to_string()
    } else {
        "gray".to_string()
    };

    rsx! {
        div {
            class: "grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4",

            MetricCard {
                label: "Total Cards".to_string(),
                value: data.total_active_cards.to_string(),
                color: "blue".to_string(),
            }

            MetricCard {
                label: "Completed This Week".to_string(),
                value: data.completed_this_week.to_string(),
                color: "green".to_string(),
            }

            MetricCard {
                label: "Completed This Month".to_string(),
                value: data.completed_this_month.to_string(),
                color: "green".to_string(),
            }

            MetricCard {
                label: "Work In Progress".to_string(),
                value: data.wip_count.to_string(),
                color: "yellow".to_string(),
            }

            MetricCard {
                label: "Overdue".to_string(),
                value: data.overdue_count.to_string(),
                color: overdue_color,
            }

            MetricCard {
                label: "Avg Cycle Time".to_string(),
                value: cycle_time_str,
                color: "purple".to_string(),
            }
        }
    }
}

/// Single metric card.
#[derive(Props, Clone, PartialEq)]
struct MetricCardProps {
    /// Label text.
    label: String,
    /// Value to display.
    value: String,
    /// Color theme: blue, green, yellow, red, purple, gray.
    color: String,
}

#[component]
fn MetricCard(props: MetricCardProps) -> Element {
    let (bg_class, text_class) = match props.color.as_str() {
        "blue" => ("bg-blue-900/30 border-blue-500/30", "text-blue-400"),
        "green" => ("bg-green-900/30 border-green-500/30", "text-green-400"),
        "yellow" => ("bg-yellow-900/30 border-yellow-500/30", "text-yellow-400"),
        "red" => ("bg-red-900/30 border-red-500/30", "text-red-400"),
        "purple" => ("bg-purple-900/30 border-purple-500/30", "text-purple-400"),
        _ => ("bg-gray-800/50 border-gray-600/30", "text-gray-400"),
    };

    rsx! {
        div {
            class: format!("rounded-lg border p-4 {bg_class}"),

            div {
                class: "text-xs text-gray-400 mb-1",
                "{props.label}"
            }

            div {
                class: format!("text-2xl font-bold {text_class}"),
                "{props.value}"
            }
        }
    }
}

/// Velocity bar chart.
#[derive(Props, Clone, PartialEq)]
struct VelocityChartProps {
    /// Velocity data.
    velocity: VelocityMetric,
}

#[component]
fn VelocityChart(props: VelocityChartProps) -> Element {
    let data = &props.velocity;

    // Find max for scaling
    let max_completed = data
        .data_points
        .iter()
        .map(|p| p.cards_completed)
        .max()
        .unwrap_or(1)
        .max(1);

    rsx! {
        div {
            class: "bg-gray-800 rounded-lg p-4",

            h3 {
                class: "text-lg font-medium text-white mb-4",
                "Velocity"
            }

            div {
                class: "text-sm text-gray-400 mb-4",
                "Avg: {data.average:.1} cards/period | Std Dev: {data.std_deviation:.1}"
            }

            // Bar chart
            div {
                class: "flex items-end gap-2 h-32",

                for point in &data.data_points {
                    div {
                        class: "flex-1 flex flex-col items-center",

                        // Bar
                        div {
                            class: "w-full bg-blue-500 rounded-t",
                            style: "height: {(point.cards_completed as f64 / max_completed as f64 * 100.0).max(4.0)}%",
                        }

                        // Label
                        div {
                            class: "text-xs text-gray-500 mt-1",
                            "{point.cards_completed}"
                        }
                    }
                }
            }
        }
    }
}

/// Burndown chart view.
#[derive(Props, Clone, PartialEq)]
struct BurndownChartViewProps {
    /// Burndown data.
    burndown: BurndownChart,
}

#[component]
fn BurndownChartView(props: BurndownChartViewProps) -> Element {
    let data = &props.burndown;

    // Calculate max for scaling
    let max_remaining = data
        .actual_line
        .iter()
        .map(|p| p.remaining)
        .max()
        .unwrap_or(data.initial_card_count)
        .max(1);

    rsx! {
        div {
            class: "bg-gray-800 rounded-lg p-4",

            h3 {
                class: "text-lg font-medium text-white mb-4",
                "Burndown"
            }

            div {
                class: "text-sm text-gray-400 mb-4",
                "Started: {data.initial_card_count} | Current scope: {data.current_scope} | Change: {data.scope_change:+}"
            }

            // Simple burndown visualization
            div {
                class: "flex items-end gap-1 h-32",

                for point in &data.actual_line {
                    div {
                        class: "flex-1 flex flex-col items-center",

                        // Remaining bar
                        div {
                            class: "w-full bg-orange-500 rounded-t",
                            style: "height: {(point.remaining as f64 / max_remaining as f64 * 100.0).max(4.0)}%",
                            title: "Remaining: {point.remaining}",
                        }
                    }
                }
            }

            // Legend
            div {
                class: "flex items-center gap-4 mt-4 text-xs text-gray-400",

                div {
                    class: "flex items-center gap-1",
                    span { class: "w-3 h-3 bg-orange-500 rounded" }
                    "Remaining"
                }
            }
        }
    }
}

/// Cycle time histogram.
#[derive(Props, Clone, PartialEq)]
struct CycleTimeChartProps {
    /// Cycle time distribution data.
    distribution: CycleTimeDistribution,
}

#[component]
fn CycleTimeChart(props: CycleTimeChartProps) -> Element {
    let data = &props.distribution;

    // Find max for scaling
    let max_count = data
        .histogram
        .iter()
        .map(|b| b.count)
        .max()
        .unwrap_or(1)
        .max(1);

    // Pre-compute formatted strings
    let median_str = format_duration(data.median_seconds);
    let p85_str = format_duration(data.p85_seconds);

    rsx! {
        div {
            class: "bg-gray-800 rounded-lg p-4",

            h3 {
                class: "text-lg font-medium text-white mb-4",
                "Cycle Time Distribution"
            }

            div {
                class: "text-sm text-gray-400 mb-4",
                "Median: {median_str} | 85th: {p85_str}"
            }

            // Histogram bars
            div {
                class: "flex items-end gap-2 h-32",

                for bucket in &data.histogram {
                    div {
                        class: "flex-1 flex flex-col items-center",

                        // Bar
                        div {
                            class: "w-full bg-purple-500 rounded-t",
                            style: "height: {(bucket.count as f64 / max_count as f64 * 100.0).max(4.0)}%",
                            title: "{bucket.label}: {bucket.count} cards",
                        }

                        // Label
                        div {
                            class: "text-xs text-gray-500 mt-1 truncate w-full text-center",
                            title: "{bucket.label}",
                            "{bucket.label}"
                        }
                    }
                }
            }
        }
    }
}

/// Column distribution pie chart (simplified as bar).
#[derive(Props, Clone, PartialEq)]
struct ColumnDistributionProps {
    /// Board analytics data.
    analytics: BoardAnalytics,
}

#[component]
fn ColumnDistribution(props: ColumnDistributionProps) -> Element {
    let data = &props.analytics;

    // Sort columns by card count
    let mut columns: Vec<_> = data.column_card_counts.iter().collect();
    columns.sort_by(|a, b| b.1.cmp(a.1));

    let total = data.total_active_cards.max(1);

    rsx! {
        div {
            class: "bg-gray-800 rounded-lg p-4",

            h3 {
                class: "text-lg font-medium text-white mb-4",
                "Cards by Column"
            }

            div {
                class: "space-y-2",

                for (col_id, count) in columns.iter().take(8) {
                    div {
                        class: "flex items-center gap-3",

                        // Column name (truncated)
                        div {
                            class: "w-24 text-sm text-gray-400 truncate",
                            title: "{col_id}",
                            "{col_id}"
                        }

                        // Progress bar
                        div {
                            class: "flex-1 h-4 bg-gray-700 rounded overflow-hidden",

                            div {
                                class: "h-full bg-emerald-500",
                                style: "width: {(**count as f64 / total as f64 * 100.0).min(100.0)}%",
                            }
                        }

                        // Count
                        div {
                            class: "w-8 text-sm text-gray-400 text-right",
                            "{count}"
                        }
                    }
                }
            }
        }
    }
}

/// Format duration from seconds to human-readable string.
fn format_duration(seconds: i64) -> String {
    const HOUR: i64 = 3600;
    const DAY: i64 = 86400;

    if seconds < HOUR {
        format!("{}m", seconds / 60)
    } else if seconds < DAY {
        format!("{}h", seconds / HOUR)
    } else {
        let days = seconds / DAY;
        if days == 1 {
            "1 day".to_string()
        } else {
            format!("{} days", days)
        }
    }
}
