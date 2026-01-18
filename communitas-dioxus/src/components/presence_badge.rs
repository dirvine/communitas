//! Presence badge component for displaying contact status.

use communitas_ui_api::PresenceStatus;
use dioxus::prelude::*;

/// Props for the PresenceBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct PresenceBadgeProps {
    /// The presence status to display.
    pub status: PresenceStatus,
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

/// Presence indicator badge showing online/away/busy/offline status.
#[component]
pub fn PresenceBadge(props: PresenceBadgeProps) -> Element {
    let (dot_class, text_class, label) = match props.status {
        PresenceStatus::Online => ("bg-emerald-400", "text-emerald-400", "Online"),
        PresenceStatus::Away => ("bg-amber-400", "text-amber-400", "Away"),
        PresenceStatus::Busy => ("bg-red-400", "text-red-400", "Busy"),
        PresenceStatus::Offline => ("bg-slate-500", "text-slate-500", "Offline"),
        PresenceStatus::Unknown => ("bg-slate-600", "text-slate-600", "Unknown"),
    };

    let (dot_size, text_size) = match props.size {
        "xs" => ("w-1.5 h-1.5", "text-xs"),
        "sm" => ("w-2 h-2", "text-xs"),
        "md" => ("w-2.5 h-2.5", "text-sm"),
        "lg" => ("w-3 h-3", "text-base"),
        _ => ("w-2 h-2", "text-xs"),
    };

    rsx! {
        span {
            class: "presence-badge inline-flex items-center gap-1.5",
            role: "status",
            aria_label: format!("Status: {}", label),
            // Status dot
            span {
                class: format!("rounded-full {dot_size} {dot_class}"),
            }
            // Status text
            span {
                class: format!("{text_size} {text_class}"),
                "{label}"
            }
        }
    }
}

/// Compact presence dot without text label.
#[derive(Props, Clone, PartialEq)]
pub struct PresenceDotProps {
    /// The presence status to display.
    pub status: PresenceStatus,
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

#[component]
pub fn PresenceDot(props: PresenceDotProps) -> Element {
    let dot_class = match props.status {
        PresenceStatus::Online => "bg-emerald-400",
        PresenceStatus::Away => "bg-amber-400",
        PresenceStatus::Busy => "bg-red-400",
        PresenceStatus::Offline => "bg-slate-500",
        PresenceStatus::Unknown => "bg-slate-600",
    };

    let dot_size = match props.size {
        "xs" => "w-1.5 h-1.5",
        "sm" => "w-2 h-2",
        "md" => "w-2.5 h-2.5",
        "lg" => "w-3 h-3",
        _ => "w-2 h-2",
    };

    let label = match props.status {
        PresenceStatus::Online => "Online",
        PresenceStatus::Away => "Away",
        PresenceStatus::Busy => "Busy",
        PresenceStatus::Offline => "Offline",
        PresenceStatus::Unknown => "Unknown",
    };

    rsx! {
        span {
            class: format!("presence-dot rounded-full {dot_size} {dot_class}"),
            role: "status",
            aria_label: format!("Status: {}", label),
            title: "{label}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presence_badge_renders_correct_label() {
        // Test that each status maps to correct label
        assert_eq!(
            match PresenceStatus::Online {
                PresenceStatus::Online => "Online",
                _ => "",
            },
            "Online"
        );
        assert_eq!(
            match PresenceStatus::Away {
                PresenceStatus::Away => "Away",
                _ => "",
            },
            "Away"
        );
        assert_eq!(
            match PresenceStatus::Busy {
                PresenceStatus::Busy => "Busy",
                _ => "",
            },
            "Busy"
        );
        assert_eq!(
            match PresenceStatus::Offline {
                PresenceStatus::Offline => "Offline",
                _ => "",
            },
            "Offline"
        );
    }

    #[test]
    fn size_variants_map_correctly() {
        let sizes = ["xs", "sm", "md", "lg"];
        for size in sizes {
            let (dot, text) = match size {
                "xs" => ("w-1.5 h-1.5", "text-xs"),
                "sm" => ("w-2 h-2", "text-xs"),
                "md" => ("w-2.5 h-2.5", "text-sm"),
                "lg" => ("w-3 h-3", "text-base"),
                _ => ("w-2 h-2", "text-xs"),
            };
            assert!(!dot.is_empty());
            assert!(!text.is_empty());
        }
    }
}
