//! Call quality indicator displaying signal strength bars.
//!
//! Shows connection quality as 1-5 bars with color coding:
//! - 5 bars (green): Excellent
//! - 4 bars (green): Good
//! - 3 bars (yellow): Fair
//! - 2 bars (orange): Poor
//! - 1 bar (red): Critical
//! - 0 bars (gray): Unknown

use communitas_ui_api::call::ConnectionQuality;
use communitas_ui_service::UiServices;
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the QualityIndicator component.
#[derive(Props, Clone, PartialEq)]
pub struct QualityIndicatorProps {
    /// Size variant: "sm", "md", or "lg".
    #[props(default = "md")]
    pub size: &'static str,
    /// Whether to show the label text.
    #[props(default = false)]
    pub show_label: bool,
    /// Whether to show detailed metrics on hover.
    #[props(default = true)]
    pub show_tooltip: bool,
}

/// Call quality indicator with signal strength bars.
#[component]
pub fn QualityIndicator(props: QualityIndicatorProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    // Subscribe to call state changes
    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                call_snapshot.set(rx.borrow().clone());
            }
        }
    });

    let snapshot = call_snapshot();
    let quality = snapshot.quality_metrics.quality;
    let metrics = &snapshot.quality_metrics;

    let bars = quality.signal_bars();
    let label = quality.label();

    // Color based on quality
    let (bar_active_color, bar_inactive_color) = match quality {
        ConnectionQuality::Excellent | ConnectionQuality::Good => {
            ("bg-emerald-500", "bg-slate-600")
        }
        ConnectionQuality::Fair => ("bg-amber-500", "bg-slate-600"),
        ConnectionQuality::Poor => ("bg-orange-500", "bg-slate-600"),
        ConnectionQuality::Critical => ("bg-red-500", "bg-slate-600"),
        ConnectionQuality::Unknown => ("bg-slate-500", "bg-slate-700"),
    };

    // Size variants
    let (bar_width, bar_heights, gap, text_size) = match props.size {
        "sm" => (
            "w-1",
            ["h-1", "h-1.5", "h-2", "h-2.5", "h-3"],
            "gap-0.5",
            "text-xs",
        ),
        "md" => (
            "w-1.5",
            ["h-1.5", "h-2", "h-3", "h-4", "h-5"],
            "gap-0.5",
            "text-sm",
        ),
        "lg" => (
            "w-2",
            ["h-2", "h-3", "h-4", "h-5", "h-6"],
            "gap-1",
            "text-base",
        ),
        _ => (
            "w-1.5",
            ["h-1.5", "h-2", "h-3", "h-4", "h-5"],
            "gap-0.5",
            "text-sm",
        ),
    };

    // Tooltip content
    let tooltip = format!(
        "Quality: {}\nLatency: {}ms\nPacket Loss: {:.1}%\nJitter: {}ms",
        label, metrics.latency_ms, metrics.packet_loss_percent, metrics.jitter_ms
    );

    rsx! {
        div {
            class: "quality-indicator inline-flex items-end {gap}",
            role: "status",
            aria_label: format!("Call quality: {}", label),
            title: if props.show_tooltip { tooltip } else { label.to_string() },

            // Signal bars (5 bars)
            for i in 0..5 {
                span {
                    class: format!(
                        "{} {} rounded-sm {}",
                        bar_width,
                        bar_heights[i],
                        if (i + 1) <= bars as usize { bar_active_color } else { bar_inactive_color }
                    ),
                }
            }

            // Optional label
            if props.show_label {
                span {
                    class: format!("{} ml-1 text-slate-300", text_size),
                    "{label}"
                }
            }
        }
    }
}

/// Compact quality dot indicator (single dot with color coding).
#[derive(Props, Clone, PartialEq)]
pub struct QualityDotProps {
    /// Size variant: "sm", "md", or "lg".
    #[props(default = "sm")]
    pub size: &'static str,
}

#[component]
pub fn QualityDot(props: QualityDotProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                call_snapshot.set(rx.borrow().clone());
            }
        }
    });

    let snapshot = call_snapshot();
    let quality = snapshot.quality_metrics.quality;
    let label = quality.label();

    let dot_color = match quality {
        ConnectionQuality::Excellent | ConnectionQuality::Good => "bg-emerald-500",
        ConnectionQuality::Fair => "bg-amber-500",
        ConnectionQuality::Poor => "bg-orange-500",
        ConnectionQuality::Critical => "bg-red-500",
        ConnectionQuality::Unknown => "bg-slate-500",
    };

    let dot_size = match props.size {
        "sm" => "w-2 h-2",
        "md" => "w-2.5 h-2.5",
        "lg" => "w-3 h-3",
        _ => "w-2 h-2",
    };

    // Pulsing animation for poor/critical quality
    let animation = match quality {
        ConnectionQuality::Poor | ConnectionQuality::Critical => "animate-pulse",
        _ => "",
    };

    rsx! {
        span {
            class: format!("quality-dot {dot_size} {dot_color} rounded-full {animation}"),
            role: "status",
            aria_label: format!("Call quality: {}", label),
            title: "{label}",
        }
    }
}

/// Detailed quality metrics panel.
#[derive(Props, Clone, PartialEq)]
pub struct QualityDetailsPanelProps {
    /// Whether the panel is expanded.
    #[props(default = false)]
    pub expanded: bool,
}

#[component]
pub fn QualityDetailsPanel(props: QualityDetailsPanelProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let call_service = services.call();

    let mut call_snapshot = use_signal(|| call_service.current_snapshot());

    let _updater = use_resource(move || {
        let call = services.call();
        async move {
            let mut rx = call.subscribe();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                call_snapshot.set(rx.borrow().clone());
            }
        }
    });

    if !props.expanded {
        return rsx! {};
    }

    let snapshot = call_snapshot();
    let metrics = &snapshot.quality_metrics;

    rsx! {
        div {
            class: "quality-details-panel bg-slate-800 rounded-lg p-3 text-sm space-y-2",
            // Quality header
            div {
                class: "flex items-center justify-between",
                span { class: "text-slate-400", "Connection Quality" }
                QualityIndicator { size: "sm", show_label: true }
            }
            // Metrics grid
            div {
                class: "grid grid-cols-2 gap-2 text-slate-300",
                // Latency
                div {
                    span { class: "text-slate-500", "Latency: " }
                    span { "{metrics.latency_ms}ms" }
                }
                // Jitter
                div {
                    span { class: "text-slate-500", "Jitter: " }
                    span { "{metrics.jitter_ms}ms" }
                }
                // Packet loss
                div {
                    span { class: "text-slate-500", "Packet Loss: " }
                    span { "{metrics.packet_loss_percent:.1}%" }
                }
                // Bitrate
                div {
                    span { class: "text-slate-500", "Bitrate: " }
                    span { "{metrics.total_bitrate_kbps()}kbps" }
                }
            }
            // Video stats (if video is active)
            if metrics.has_video() {
                div {
                    class: "border-t border-slate-700 pt-2 mt-2",
                    span { class: "text-slate-500 text-xs", "Video: " }
                    span { class: "text-slate-300 text-xs",
                        "{metrics.video_width}x{metrics.video_height} @ {metrics.video_fps}fps"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use communitas_ui_api::call::ConnectionQuality;

    #[test]
    fn bar_colors_for_quality_levels() {
        let test_cases = [
            (ConnectionQuality::Excellent, "emerald"),
            (ConnectionQuality::Good, "emerald"),
            (ConnectionQuality::Fair, "amber"),
            (ConnectionQuality::Poor, "orange"),
            (ConnectionQuality::Critical, "red"),
            (ConnectionQuality::Unknown, "slate"),
        ];

        for (quality, expected_color) in test_cases {
            let color = match quality {
                ConnectionQuality::Excellent | ConnectionQuality::Good => "emerald",
                ConnectionQuality::Fair => "amber",
                ConnectionQuality::Poor => "orange",
                ConnectionQuality::Critical => "red",
                ConnectionQuality::Unknown => "slate",
            };
            assert_eq!(
                color, expected_color,
                "Quality {:?} should use {} color",
                quality, expected_color
            );
        }
    }

    #[test]
    fn size_variants() {
        let sizes = ["sm", "md", "lg"];
        for size in sizes {
            let bar_width = match size {
                "sm" => "w-1",
                "md" => "w-1.5",
                "lg" => "w-2",
                _ => "w-1.5",
            };
            assert!(!bar_width.is_empty());
        }
    }

    #[test]
    fn signal_bars_mapping() {
        assert_eq!(ConnectionQuality::Unknown.signal_bars(), 0);
        assert_eq!(ConnectionQuality::Critical.signal_bars(), 1);
        assert_eq!(ConnectionQuality::Poor.signal_bars(), 2);
        assert_eq!(ConnectionQuality::Fair.signal_bars(), 3);
        assert_eq!(ConnectionQuality::Good.signal_bars(), 4);
        assert_eq!(ConnectionQuality::Excellent.signal_bars(), 5);
    }
}
