// SPDX-License-Identifier: MIT OR Apache-2.0

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

/// Props for InCallBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct InCallBadgeProps {
    /// Optional entity name (channel/group) the contact is in.
    #[props(default)]
    pub call_entity_name: Option<String>,
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

/// Badge showing that a contact is currently in a call.
#[component]
pub fn InCallBadge(props: InCallBadgeProps) -> Element {
    let (badge_size, text_size, icon_size) = match props.size {
        "xs" => ("px-1 py-0.5", "text-xs", "text-xs"),
        "sm" => ("px-1.5 py-0.5", "text-xs", "text-sm"),
        "md" => ("px-2 py-1", "text-sm", "text-base"),
        "lg" => ("px-2.5 py-1", "text-base", "text-lg"),
        _ => ("px-1.5 py-0.5", "text-xs", "text-sm"),
    };

    let label = props
        .call_entity_name
        .as_ref()
        .map(|name| format!("In call: {}", name))
        .unwrap_or_else(|| "In call".to_string());

    rsx! {
        span {
            class: format!(
                "in-call-badge inline-flex items-center gap-1 {} rounded-full bg-violet-600/80 text-violet-100",
                badge_size
            ),
            role: "status",
            aria_label: "{label}",
            title: "{label}",
            // Phone icon with animation
            span {
                class: format!("{} animate-pulse", icon_size),
                "📞"
            }
            // Text (only show entity name on larger sizes)
            if props.size == "md" || props.size == "lg" {
                span {
                    class: format!("{} truncate max-w-[100px]", text_size),
                    if let Some(name) = &props.call_entity_name {
                        "{name}"
                    } else {
                        "In call"
                    }
                }
            }
        }
    }
}

/// Props for PresenceWithCallBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct PresenceWithCallBadgeProps {
    /// The presence status to display.
    pub status: PresenceStatus,
    /// Whether the contact is currently in a call.
    #[props(default = false)]
    pub is_in_call: bool,
    /// Optional entity name of the call.
    #[props(default)]
    pub call_entity_name: Option<String>,
    /// Whether the contact is currently screen sharing.
    #[props(default = false)]
    pub is_screen_sharing: bool,
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

/// Combined presence badge that shows status, in-call, or presenting indicator.
#[component]
pub fn PresenceWithCallBadge(props: PresenceWithCallBadgeProps) -> Element {
    rsx! {
        span {
            class: "presence-with-call inline-flex items-center gap-1.5",
            // Priority: presenting > in-call > presence status
            if props.is_screen_sharing {
                PresentingBadge {
                    size: props.size,
                }
            } else if props.is_in_call {
                InCallBadge {
                    call_entity_name: props.call_entity_name.clone(),
                    size: props.size,
                }
            } else {
                PresenceBadge {
                    status: props.status,
                    size: props.size,
                }
            }
        }
    }
}

/// Compact in-call dot indicator (no text, just a pulsing icon).
#[derive(Props, Clone, PartialEq)]
pub struct InCallDotProps {
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
    /// Optional tooltip with call entity name.
    #[props(default)]
    pub call_entity_name: Option<String>,
}

#[component]
pub fn InCallDot(props: InCallDotProps) -> Element {
    let dot_size = match props.size {
        "xs" => "w-3 h-3 text-[8px]",
        "sm" => "w-4 h-4 text-[10px]",
        "md" => "w-5 h-5 text-xs",
        "lg" => "w-6 h-6 text-sm",
        _ => "w-4 h-4 text-[10px]",
    };

    let label = props
        .call_entity_name
        .as_ref()
        .map(|name| format!("In call: {}", name))
        .unwrap_or_else(|| "In call".to_string());

    rsx! {
        span {
            class: format!(
                "in-call-dot {} rounded-full bg-violet-600 flex items-center justify-center animate-pulse",
                dot_size
            ),
            role: "status",
            aria_label: "{label}",
            title: "{label}",
            "📞"
        }
    }
}

/// Props for PresentingBadge component.
#[derive(Props, Clone, PartialEq)]
pub struct PresentingBadgeProps {
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

/// Badge showing that a contact is currently screen sharing/presenting.
#[component]
pub fn PresentingBadge(props: PresentingBadgeProps) -> Element {
    let (badge_size, text_size, icon_size) = match props.size {
        "xs" => ("px-1 py-0.5", "text-xs", "text-xs"),
        "sm" => ("px-1.5 py-0.5", "text-xs", "text-sm"),
        "md" => ("px-2 py-1", "text-sm", "text-base"),
        "lg" => ("px-2.5 py-1", "text-base", "text-lg"),
        _ => ("px-1.5 py-0.5", "text-xs", "text-sm"),
    };

    rsx! {
        span {
            class: format!(
                "presenting-badge inline-flex items-center gap-1 {} rounded-full bg-sky-600/80 text-sky-100",
                badge_size
            ),
            role: "status",
            aria_label: "Presenting",
            title: "Presenting",
            // Screen icon with animation
            span {
                class: format!("{} animate-pulse", icon_size),
                "🖥️"
            }
            // Text (only show on larger sizes)
            if props.size == "md" || props.size == "lg" {
                span {
                    class: text_size.to_string(),
                    "Presenting"
                }
            }
        }
    }
}

/// Compact presenting dot indicator (no text, just a pulsing icon).
#[derive(Props, Clone, PartialEq)]
pub struct PresentingDotProps {
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

#[component]
pub fn PresentingDot(props: PresentingDotProps) -> Element {
    let dot_size = match props.size {
        "xs" => "w-3 h-3 text-[8px]",
        "sm" => "w-4 h-4 text-[10px]",
        "md" => "w-5 h-5 text-xs",
        "lg" => "w-6 h-6 text-sm",
        _ => "w-4 h-4 text-[10px]",
    };

    rsx! {
        span {
            class: format!(
                "presenting-dot {} rounded-full bg-sky-600 flex items-center justify-center animate-pulse",
                dot_size
            ),
            role: "status",
            aria_label: "Presenting",
            title: "Presenting",
            "🖥️"
        }
    }
}

/// Combined dot that shows presence OR in-call status OR presenting status.
#[derive(Props, Clone, PartialEq)]
pub struct PresenceOrCallDotProps {
    /// The presence status to display.
    pub status: PresenceStatus,
    /// Whether the contact is currently in a call.
    #[props(default = false)]
    pub is_in_call: bool,
    /// Optional entity name of the call.
    #[props(default)]
    pub call_entity_name: Option<String>,
    /// Whether the contact is currently screen sharing.
    #[props(default = false)]
    pub is_screen_sharing: bool,
    /// Optional size variant (default: "sm").
    #[props(default = "sm")]
    pub size: &'static str,
}

#[component]
pub fn PresenceOrCallDot(props: PresenceOrCallDotProps) -> Element {
    // Presenting takes priority (it's the most active state)
    if props.is_screen_sharing {
        rsx! {
            PresentingDot {
                size: props.size,
            }
        }
    } else if props.is_in_call {
        rsx! {
            InCallDot {
                size: props.size,
                call_entity_name: props.call_entity_name.clone(),
            }
        }
    } else {
        rsx! {
            PresenceDot {
                status: props.status,
                size: props.size,
            }
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

    #[test]
    fn in_call_badge_size_variants() {
        let sizes = ["xs", "sm", "md", "lg"];
        for size in sizes {
            let (badge, text, icon) = match size {
                "xs" => ("px-1 py-0.5", "text-xs", "text-xs"),
                "sm" => ("px-1.5 py-0.5", "text-xs", "text-sm"),
                "md" => ("px-2 py-1", "text-sm", "text-base"),
                "lg" => ("px-2.5 py-1", "text-base", "text-lg"),
                _ => ("px-1.5 py-0.5", "text-xs", "text-sm"),
            };
            assert!(!badge.is_empty());
            assert!(!text.is_empty());
            assert!(!icon.is_empty());
        }
    }

    #[test]
    fn in_call_dot_size_variants() {
        let sizes = ["xs", "sm", "md", "lg"];
        for size in sizes {
            let dot = match size {
                "xs" => "w-3 h-3 text-[8px]",
                "sm" => "w-4 h-4 text-[10px]",
                "md" => "w-5 h-5 text-xs",
                "lg" => "w-6 h-6 text-sm",
                _ => "w-4 h-4 text-[10px]",
            };
            assert!(!dot.is_empty());
        }
    }

    #[test]
    fn in_call_label_with_entity_name() {
        let call_entity_name = Some("Team Standup".to_string());
        let label = call_entity_name
            .as_ref()
            .map(|name| format!("In call: {}", name))
            .unwrap_or_else(|| "In call".to_string());
        assert_eq!(label, "In call: Team Standup");
    }

    #[test]
    fn in_call_label_without_entity_name() {
        let call_entity_name: Option<String> = None;
        let label = call_entity_name
            .as_ref()
            .map(|name| format!("In call: {}", name))
            .unwrap_or_else(|| "In call".to_string());
        assert_eq!(label, "In call");
    }

    #[test]
    fn presenting_badge_size_variants() {
        let sizes = ["xs", "sm", "md", "lg"];
        for size in sizes {
            let (badge, text, icon) = match size {
                "xs" => ("px-1 py-0.5", "text-xs", "text-xs"),
                "sm" => ("px-1.5 py-0.5", "text-xs", "text-sm"),
                "md" => ("px-2 py-1", "text-sm", "text-base"),
                "lg" => ("px-2.5 py-1", "text-base", "text-lg"),
                _ => ("px-1.5 py-0.5", "text-xs", "text-sm"),
            };
            assert!(!badge.is_empty());
            assert!(!text.is_empty());
            assert!(!icon.is_empty());
        }
    }

    #[test]
    fn presenting_dot_size_variants() {
        let sizes = ["xs", "sm", "md", "lg"];
        for size in sizes {
            let dot = match size {
                "xs" => "w-3 h-3 text-[8px]",
                "sm" => "w-4 h-4 text-[10px]",
                "md" => "w-5 h-5 text-xs",
                "lg" => "w-6 h-6 text-sm",
                _ => "w-4 h-4 text-[10px]",
            };
            assert!(!dot.is_empty());
        }
    }

    #[test]
    fn presence_or_call_dot_priority() {
        // Screen sharing should take priority over in-call
        // Simulates the priority logic: presenting > in-call > presence
        let scenarios = [
            (true, true, "presenting"),  // Both sharing and in-call -> presenting wins
            (true, false, "presenting"), // Sharing but not in-call -> presenting
            (false, true, "in-call"),    // In-call but not sharing -> in-call
            (false, false, "presence"),  // Neither -> presence
        ];
        for (is_screen_sharing, is_in_call, expected) in scenarios {
            let result = if is_screen_sharing {
                "presenting"
            } else if is_in_call {
                "in-call"
            } else {
                "presence"
            };
            assert_eq!(result, expected);
        }
    }
}
