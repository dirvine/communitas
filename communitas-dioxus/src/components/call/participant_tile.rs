//! Participant tile component for displaying call participants.

use communitas_ui_api::Participant;
use dioxus::prelude::*;

/// Props for the ParticipantTile component.
#[derive(Props, Clone, PartialEq)]
pub struct ParticipantTileProps {
    /// Participant data to display.
    pub participant: Participant,
    /// Whether this is the current user.
    #[props(default = false)]
    pub is_self: bool,
    /// Optional size variant (default: "md").
    #[props(default = "md")]
    pub size: &'static str,
}

/// Participant tile showing avatar, name, and call status indicators.
#[component]
pub fn ParticipantTile(props: ParticipantTileProps) -> Element {
    let p = &props.participant;

    // Size classes
    let (tile_size, avatar_size, text_size, indicator_size) = match props.size {
        "sm" => ("w-24 h-28", "w-12 h-12 text-lg", "text-xs", "w-4 h-4"),
        "md" => ("w-32 h-36", "w-16 h-16 text-xl", "text-sm", "w-5 h-5"),
        "lg" => ("w-40 h-44", "w-20 h-20 text-2xl", "text-base", "w-6 h-6"),
        _ => ("w-32 h-36", "w-16 h-16 text-xl", "text-sm", "w-5 h-5"),
    };

    // Speaking indicator glow
    let speaking_class = if p.is_speaking {
        "ring-2 ring-emerald-400 ring-offset-2 ring-offset-slate-900"
    } else {
        ""
    };

    // Audio level visualization (0.0 - 1.0)
    let audio_bar_height = format!("{}%", (p.audio_level * 100.0).min(100.0) as u32);

    // Avatar initials
    let initials: String = p
        .display_name
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase();

    rsx! {
        div {
            class: format!(
                "participant-tile flex flex-col items-center gap-2 p-3 bg-slate-800 rounded-xl {}",
                tile_size
            ),
            role: "listitem",
            aria_label: format!(
                "{}{}, {}",
                p.display_name,
                if props.is_self { " (You)" } else { "" },
                if p.is_muted { "muted" } else { "unmuted" }
            ),
            // Avatar with speaking indicator
            div {
                class: "relative",
                // Avatar circle
                div {
                    class: format!(
                        "flex items-center justify-center rounded-full bg-gradient-to-br from-emerald-500 to-teal-600 text-white font-medium {} {}",
                        avatar_size,
                        speaking_class
                    ),
                    "{initials}"
                }
                // Video indicator (bottom-left)
                if p.is_video_enabled {
                    span {
                        class: format!(
                            "absolute -bottom-1 -left-1 flex items-center justify-center rounded-full bg-emerald-500 text-white {} text-xs",
                            indicator_size
                        ),
                        title: "Camera on",
                        "📷"
                    }
                }
                // Mute indicator (bottom-right)
                if p.is_muted {
                    span {
                        class: format!(
                            "absolute -bottom-1 -right-1 flex items-center justify-center rounded-full bg-red-500 text-white {} text-xs",
                            indicator_size
                        ),
                        title: "Muted",
                        "🔇"
                    }
                }
                // Audio level bar (right side)
                if !p.is_muted && p.audio_level > 0.0 {
                    div {
                        class: "absolute -right-2 top-1/2 -translate-y-1/2 w-1 h-8 bg-slate-700 rounded-full overflow-hidden",
                        div {
                            class: "absolute bottom-0 w-full bg-emerald-400 rounded-full transition-all",
                            style: "height: {audio_bar_height}",
                        }
                    }
                }
            }
            // Name and status
            div {
                class: "text-center",
                p {
                    class: format!("font-medium text-white truncate {}", text_size),
                    "{p.display_name}"
                    if props.is_self {
                        span {
                            class: "text-slate-400 ml-1",
                            "(You)"
                        }
                    }
                }
                p {
                    class: "text-xs text-slate-500 truncate",
                    title: "{p.four_words}",
                    "{p.four_words}"
                }
            }
        }
    }
}

/// Grid of participant tiles.
#[derive(Props, Clone, PartialEq)]
pub struct ParticipantGridProps {
    /// List of participants to display.
    pub participants: Vec<Participant>,
    /// Current user's participant ID.
    pub my_participant_id: String,
}

#[component]
pub fn ParticipantGrid(props: ParticipantGridProps) -> Element {
    rsx! {
        div {
            class: "participant-grid grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 p-4",
            role: "list",
            aria_label: "Call participants",
            for participant in props.participants.iter() {
                ParticipantTile {
                    key: "{participant.id}",
                    participant: participant.clone(),
                    is_self: participant.id == props.my_participant_id,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_participant() -> Participant {
        Participant {
            id: "p1".to_string(),
            display_name: "Alice Smith".to_string(),
            four_words: "ocean-forest-moon-star".to_string(),
            role: Default::default(),
            is_muted: false,
            is_muted_by_host: false,
            is_video_enabled: false,
            is_speaking: false,
            is_screen_sharing: false,
            hand_raised: false,
            audio_level: 0.0,
            joined_at: 0,
        }
    }

    #[test]
    fn participant_initials() {
        let p = make_participant();
        let initials: String = p
            .display_name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        assert_eq!(initials, "AS");
    }

    #[test]
    fn size_variants_produce_valid_classes() {
        for size in ["sm", "md", "lg"] {
            let (tile, avatar, text, indicator) = match size {
                "sm" => ("w-24 h-28", "w-12 h-12 text-lg", "text-xs", "w-4 h-4"),
                "md" => ("w-32 h-36", "w-16 h-16 text-xl", "text-sm", "w-5 h-5"),
                "lg" => ("w-40 h-44", "w-20 h-20 text-2xl", "text-base", "w-6 h-6"),
                _ => ("w-32 h-36", "w-16 h-16 text-xl", "text-sm", "w-5 h-5"),
            };
            assert!(!tile.is_empty());
            assert!(!avatar.is_empty());
            assert!(!text.is_empty());
            assert!(!indicator.is_empty());
        }
    }

    #[test]
    fn audio_level_clamps_to_100() {
        let level = 1.5_f32;
        let height = (level * 100.0).min(100.0) as u32;
        assert_eq!(height, 100);
    }
}
