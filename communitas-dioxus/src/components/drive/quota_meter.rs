// SPDX-License-Identifier: MIT OR Apache-2.0

//! Quota meter component showing disk usage.

use communitas_ui_api::QuotaInfo;
use dioxus::prelude::*;

/// Quota meter props.
#[derive(Props, Clone, PartialEq)]
pub struct QuotaMeterProps {
    /// Quota information.
    pub quota: QuotaInfo,
    /// Whether to show detailed breakdown.
    #[props(default = false)]
    pub show_details: bool,
}

/// Quota meter showing disk usage with visual indicator.
#[component]
pub fn QuotaMeter(props: QuotaMeterProps) -> Element {
    let percent = props.quota.percent_used;
    let is_warning = props.quota.is_warning();
    let is_exceeded = props.quota.is_exceeded();

    let bar_color = if is_exceeded {
        "bg-red-500"
    } else if is_warning {
        "bg-amber-500"
    } else {
        "bg-emerald-500"
    };

    let text_color = if is_exceeded {
        "text-red-400"
    } else if is_warning {
        "text-amber-400"
    } else {
        "text-slate-400"
    };

    rsx! {
        div {
            class: "quota-meter",
            role: "meter",
            aria_valuenow: "{percent}",
            aria_valuemin: "0",
            aria_valuemax: "100",
            aria_label: "Storage usage",
            // Compact view
            div {
                class: "flex items-center gap-3",
                // Progress bar
                div {
                    class: "flex-1 h-2 bg-slate-800 rounded-full overflow-hidden",
                    div {
                        class: format!("h-full {} transition-all duration-300", bar_color),
                        style: "width: {percent.min(100.0)}%",
                    }
                }
                // Usage text
                span {
                    class: format!("text-xs whitespace-nowrap {}", text_color),
                    "{format_size(props.quota.used_bytes)} / {format_size(props.quota.quota_bytes)}"
                }
            }
            // Warning message
            if is_exceeded {
                div {
                    class: "mt-2 p-2 rounded bg-red-500/10 border border-red-500/30 text-xs text-red-400",
                    "⚠️ Storage quota exceeded. Delete files to continue uploading."
                }
            } else if is_warning {
                div {
                    class: "mt-2 p-2 rounded bg-amber-500/10 border border-amber-500/30 text-xs text-amber-400",
                    "⚠️ Storage is nearly full ({percent:.0}% used)"
                }
            }
            // Detailed breakdown
            if props.show_details {
                QuotaDetails { quota: props.quota.clone() }
            }
        }
    }
}

/// Detailed quota breakdown.
#[derive(Props, Clone, PartialEq)]
struct QuotaDetailsProps {
    quota: QuotaInfo,
}

#[component]
fn QuotaDetails(props: QuotaDetailsProps) -> Element {
    let available = props
        .quota
        .quota_bytes
        .saturating_sub(props.quota.used_bytes);

    rsx! {
        dl {
            class: "mt-3 space-y-1 text-xs",
            div {
                class: "flex justify-between",
                dt { class: "text-slate-500", "Used" }
                dd { class: "text-slate-300", {format_size(props.quota.used_bytes)} }
            }
            div {
                class: "flex justify-between",
                dt { class: "text-slate-500", "Available" }
                dd { class: "text-slate-300", {format_size(available)} }
            }
            div {
                class: "flex justify-between",
                dt { class: "text-slate-500", "Total" }
                dd { class: "text-slate-300", {format_size(props.quota.quota_bytes)} }
            }
        }
    }
}

/// Compact quota indicator for toolbar.
#[derive(Props, Clone, PartialEq)]
pub struct QuotaIndicatorProps {
    /// Quota information.
    pub quota: QuotaInfo,
    /// Click handler to show details.
    pub on_click: EventHandler<()>,
}

#[component]
pub fn QuotaIndicator(props: QuotaIndicatorProps) -> Element {
    let percent = props.quota.percent_used;
    let is_warning = props.quota.is_warning();
    let is_exceeded = props.quota.is_exceeded();

    let (bg_color, text_color, icon) = if is_exceeded {
        ("bg-red-500/20", "text-red-400", "⚠️")
    } else if is_warning {
        ("bg-amber-500/20", "text-amber-400", "⚡")
    } else {
        ("bg-slate-800", "text-slate-400", "💾")
    };

    rsx! {
        button {
            class: format!("flex items-center gap-2 px-3 py-1.5 rounded-lg {} {} text-xs hover:opacity-80 transition", bg_color, text_color),
            onclick: move |_| props.on_click.call(()),
            title: "Storage: {format_size(props.quota.used_bytes)} / {format_size(props.quota.quota_bytes)}",
            span { "{icon}" }
            span { "{percent:.0}% used" }
        }
    }
}

/// Format size for display.
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_api::DiskType;

    #[test]
    fn format_size_units() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 1024), "1.0 TB");
    }

    #[test]
    fn quota_percent_used() {
        let quota = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 50,
            quota_bytes: 100,
            percent_used: 50.0,
        };
        assert!((quota.percent_used - 50.0).abs() < 0.01);
    }

    #[test]
    fn quota_warning_threshold() {
        let normal = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 70,
            quota_bytes: 100,
            percent_used: 70.0,
        };
        assert!(!normal.is_warning());

        let warning = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 90,
            quota_bytes: 100,
            percent_used: 90.0,
        };
        assert!(warning.is_warning());
        assert!(!warning.is_exceeded());
    }

    #[test]
    fn quota_exceeded_threshold() {
        let exceeded = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 100,
            quota_bytes: 100,
            percent_used: 100.0,
        };
        assert!(exceeded.is_exceeded());
    }
}
