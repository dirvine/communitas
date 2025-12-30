// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Theme and styling for the Communitas application.
//!
//! Design Direction: "Warm Digital Commons"
//! A rich, organic aesthetic that embodies decentralization, trust, and community.
//! Forest greens and warm earth tones create an inviting, secure feeling.

use iced::widget::{button, container, scrollable, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

/// Core color palette for the application.
///
/// Uses a "Warm Digital Commons" aesthetic with forest greens and earth tones.
pub struct Palette;

impl Palette {
    // ═══════════════════════════════════════════════════════════════════════
    // FOUNDATION COLORS - The base of our visual language
    // ═══════════════════════════════════════════════════════════════════════

    /// Deep forest - primary dark background (rich green-black)
    pub const DEEP_FOREST: Color = Color::from_rgb(0.102, 0.141, 0.125); // #1a241f

    /// Moss - secondary surface color (rich dark green)
    pub const MOSS: Color = Color::from_rgb(0.176, 0.239, 0.212); // #2d3d36

    /// Fern - elevated surface color
    pub const FERN: Color = Color::from_rgb(0.224, 0.298, 0.263); // #394c43

    /// Lichen - subtle background variation
    pub const LICHEN: Color = Color::from_rgb(0.278, 0.361, 0.317); // #475c51

    /// Sage - muted accent for secondary elements
    pub const SAGE: Color = Color::from_rgb(0.502, 0.596, 0.541); // #80988a

    /// Jade - primary accent color (vibrant, action-oriented)
    pub const JADE: Color = Color::from_rgb(0.298, 0.686, 0.514); // #4caf83

    /// Emerald - hover/active state for jade
    pub const EMERALD: Color = Color::from_rgb(0.231, 0.784, 0.541); // #3bc88a

    /// Amber - notification/highlight accent
    pub const AMBER: Color = Color::from_rgb(0.878, 0.698, 0.396); // #e0b265

    /// Cream - primary text on dark backgrounds
    pub const CREAM: Color = Color::from_rgb(0.949, 0.933, 0.906); // #f2eee7

    /// Warm white - brightest surface (for cards, inputs)
    pub const WARM_WHITE: Color = Color::from_rgb(0.976, 0.965, 0.945); // #f9f7f1

    /// Stone - muted text color
    pub const STONE: Color = Color::from_rgb(0.631, 0.604, 0.565); // #a19a90

    /// Charcoal - text on light backgrounds
    pub const CHARCOAL: Color = Color::from_rgb(0.176, 0.192, 0.180); // #2d312e

    // ═══════════════════════════════════════════════════════════════════════
    // ENTITY TYPE COLORS - Distinctive colors for different entity types
    // ═══════════════════════════════════════════════════════════════════════

    /// Organisation entity - deep teal
    pub const ORGANISATION: Color = Color::from_rgb(0.180, 0.545, 0.545); // #2e8b8b

    /// Project entity - warm gold
    pub const PROJECT: Color = Color::from_rgb(0.847, 0.647, 0.278); // #d8a547

    /// Channel entity - vibrant jade (matches our accent)
    pub const CHANNEL: Color = Color::from_rgb(0.298, 0.686, 0.514); // #4caf83

    /// Group entity - soft violet
    pub const GROUP: Color = Color::from_rgb(0.557, 0.408, 0.647); // #8e68a5

    /// Person/contact - warm coral
    pub const PERSON: Color = Color::from_rgb(0.835, 0.475, 0.357); // #d5795b

    // ═══════════════════════════════════════════════════════════════════════
    // STATUS COLORS - Communication states
    // ═══════════════════════════════════════════════════════════════════════

    /// Online status - bright jade
    pub const ONLINE: Color = Color::from_rgb(0.298, 0.784, 0.514); // #4cc883

    /// Away status - soft amber
    pub const AWAY: Color = Color::from_rgb(0.878, 0.698, 0.396); // #e0b265

    /// Offline status - muted sage
    pub const OFFLINE: Color = Color::from_rgb(0.502, 0.545, 0.514); // #808b83

    /// Error/danger - warm red (not harsh)
    pub const ERROR: Color = Color::from_rgb(0.820, 0.341, 0.341); // #d15757

    /// Success - matches our jade theme
    pub const SUCCESS: Color = Color::from_rgb(0.298, 0.784, 0.514); // #4cc883

    /// Warning - amber
    pub const WARNING: Color = Color::from_rgb(0.878, 0.698, 0.396); // #e0b265

    // ═══════════════════════════════════════════════════════════════════════
    // LAYOUT COLORS - Structural elements
    // ═══════════════════════════════════════════════════════════════════════

    /// Sidebar background - deep forest
    pub const SIDEBAR_BG: Color = Self::DEEP_FOREST;

    /// Detail pane background - warm white
    pub const DETAIL_BG: Color = Self::WARM_WHITE;

    /// Selected item highlight (jade with transparency)
    pub const SELECTED: Color = Color::from_rgba(0.298, 0.686, 0.514, 0.20);

    /// Hover state (subtle jade glow)
    pub const HOVER: Color = Color::from_rgba(0.298, 0.686, 0.514, 0.12);

    /// Border color - subtle sage
    pub const BORDER: Color = Color::from_rgba(0.502, 0.596, 0.541, 0.3);

    /// Muted/secondary text (for dark backgrounds)
    pub const TEXT_MUTED: Color = Self::SAGE;

    /// Primary text (for light backgrounds)
    pub const TEXT_PRIMARY: Color = Self::CHARCOAL;

    /// Light text (for dark backgrounds)
    pub const TEXT_LIGHT: Color = Self::CREAM;

    // ═══════════════════════════════════════════════════════════════════════
    // CALL UI COLORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Call UI background - deep forest
    pub const CALL_BG: Color = Self::DEEP_FOREST;

    /// Call controls background - moss
    pub const CALL_CONTROLS_BG: Color = Self::MOSS;

    // ═══════════════════════════════════════════════════════════════════════
    // PRIORITY COLORS
    // ═══════════════════════════════════════════════════════════════════════

    /// Low priority - muted sage
    pub const PRIORITY_LOW: Color = Self::SAGE;

    /// Normal priority - jade
    pub const PRIORITY_NORMAL: Color = Self::JADE;

    /// High priority - amber
    pub const PRIORITY_HIGH: Color = Self::AMBER;

    /// Urgent priority - warm red
    pub const PRIORITY_URGENT: Color = Self::ERROR;
}

/// Get the entity type color.
#[must_use]
pub fn entity_color(entity_type: crate::state::EntityType) -> Color {
    match entity_type {
        crate::state::EntityType::Organisation => Palette::ORGANISATION,
        crate::state::EntityType::Project => Palette::PROJECT,
        crate::state::EntityType::Channel => Palette::CHANNEL,
        crate::state::EntityType::Group => Palette::GROUP,
    }
}

/// Parse a hex color string to Color.
#[must_use]
pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Palette::TEXT_MUTED;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);

    Color::from_rgb8(r, g, b)
}

// ═══════════════════════════════════════════════════════════════════════════
// CONTAINER STYLES
// ═══════════════════════════════════════════════════════════════════════════

/// Sidebar container style - deep forest background with subtle depth.
#[must_use]
pub fn sidebar_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::SIDEBAR_BG.into()),
        text_color: Some(Palette::TEXT_LIGHT),
        ..Default::default()
    }
}

/// Detail pane container style - warm white with subtle shadow.
#[must_use]
pub fn detail_container(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::DETAIL_BG.into()),
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Selected row style - jade highlight with rounded corners.
#[must_use]
pub fn selected_row(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::SELECTED.into()),
        border: Border::default().rounded(8),
        ..Default::default()
    }
}

/// Card style - elevated surface with warm shadow.
#[must_use]
pub fn card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::WARM_WHITE.into()),
        border: Border {
            color: Palette::BORDER,
            width: 1.0,
            radius: 12.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.1, 0.12, 0.1, 0.15),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Elevated card style - for important elements.
#[must_use]
pub fn elevated_card_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::WARM_WHITE.into()),
        border: Border {
            color: Palette::JADE.scale_alpha(0.2),
            width: 1.0,
            radius: 16.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.1, 0.15, 0.1, 0.2),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Kanban column style with entity-specific color accent.
pub fn kanban_column_style(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_theme| container::Style {
        background: Some(Color::from_rgba(color.r, color.g, color.b, 0.08).into()),
        border: Border {
            color: Color::from_rgba(color.r, color.g, color.b, 0.25),
            width: 1.0,
            radius: 16.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(color.r, color.g, color.b, 0.1),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Own message bubble style - jade-tinted.
#[must_use]
pub fn own_message_bubble(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.224, 0.376, 0.314).into()), // Jade-tinted dark
        border: Border::default().rounded(16),
        text_color: Some(Palette::CREAM),
        ..Default::default()
    }
}

/// Other's message bubble style - moss surface.
#[must_use]
pub fn other_message_bubble(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::FERN.into()),
        border: Border::default().rounded(16),
        text_color: Some(Palette::CREAM),
        ..Default::default()
    }
}

/// Light message bubble for own messages (on light backgrounds).
#[must_use]
pub fn own_message_bubble_light(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.878, 0.941, 0.906).into()), // Soft jade tint
        border: Border::default().rounded(16),
        text_color: Some(Palette::CHARCOAL),
        ..Default::default()
    }
}

/// Light message bubble for others (on light backgrounds).
#[must_use]
pub fn other_message_bubble_light(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgb(0.933, 0.925, 0.910).into()), // Warm cream
        border: Border::default().rounded(16),
        text_color: Some(Palette::CHARCOAL),
        ..Default::default()
    }
}

/// Thread panel style - elevated surface.
#[must_use]
pub fn thread_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::WARM_WHITE.into()),
        border: Border {
            color: Palette::BORDER,
            width: 1.0,
            radius: 0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.1, 0.12, 0.1, 0.1),
            offset: Vector::new(-4.0, 0.0),
            blur_radius: 16.0,
        },
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Modal overlay style - deep forest with transparency.
#[must_use]
pub fn modal_overlay(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(0.102, 0.141, 0.125, 0.85).into()),
        ..Default::default()
    }
}

/// Modal content style - elevated card.
#[must_use]
pub fn modal_content(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::WARM_WHITE.into()),
        border: Border {
            color: Palette::JADE.scale_alpha(0.3),
            width: 1.0,
            radius: 20.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.05, 0.1, 0.07, 0.4),
            offset: Vector::new(0.0, 16.0),
            blur_radius: 48.0,
        },
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Header bar style - subtle gradient effect.
#[must_use]
pub fn header_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::WARM_WHITE.into()),
        border: Border {
            color: Palette::BORDER,
            width: 0.0,
            radius: 0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.1, 0.12, 0.1, 0.08),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        text_color: Some(Palette::TEXT_PRIMARY),
        ..Default::default()
    }
}

/// Section header style for sidebar.
#[must_use]
pub fn section_header_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.15).into()),
        border: Border::default().rounded(6),
        ..Default::default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BUTTON STYLES
// ═══════════════════════════════════════════════════════════════════════════

/// Primary button style - jade with hover effects.
#[must_use]
pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Palette::JADE.into()),
        text_color: Palette::DEEP_FOREST,
        border: Border::default().rounded(10),
        shadow: Shadow {
            color: Color::from_rgba(0.298, 0.686, 0.514, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Palette::EMERALD.into()),
            shadow: Shadow {
                color: Color::from_rgba(0.298, 0.686, 0.514, 0.4),
                offset: Vector::new(0.0, 1.0),
                blur_radius: 4.0,
            },
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(Palette::EMERALD.into()),
            shadow: Shadow {
                color: Color::from_rgba(0.298, 0.686, 0.514, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Palette::SAGE.scale_alpha(0.5).into()),
            text_color: Palette::STONE,
            shadow: Shadow::default(),
            ..base
        },
    }
}

/// Secondary button style - outlined jade.
#[must_use]
pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: Palette::JADE,
        border: Border {
            color: Palette::JADE,
            width: 1.5,
            radius: 10.into(),
        },
        shadow: Shadow::default(),
        ..Default::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Palette::JADE.scale_alpha(0.15).into()),
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(Palette::JADE.scale_alpha(0.1).into()),
            border: Border {
                color: Palette::EMERALD,
                width: 1.5,
                radius: 10.into(),
            },
            text_color: Palette::EMERALD,
            ..base
        },
        button::Status::Disabled => button::Style {
            border: Border {
                color: Palette::SAGE.scale_alpha(0.5),
                width: 1.0,
                radius: 10.into(),
            },
            text_color: Palette::STONE,
            ..base
        },
    }
}

/// Danger button style (for destructive actions).
#[must_use]
pub fn danger_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Palette::ERROR.into()),
        text_color: Palette::CREAM,
        border: Border::default().rounded(10),
        shadow: Shadow {
            color: Color::from_rgba(0.820, 0.341, 0.341, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..Default::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Color::from_rgb(0.75, 0.28, 0.28).into()),
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgb(0.85, 0.38, 0.38).into()),
            shadow: Shadow {
                color: Color::from_rgba(0.820, 0.341, 0.341, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Palette::SAGE.scale_alpha(0.5).into()),
            text_color: Palette::STONE,
            shadow: Shadow::default(),
            ..base
        },
    }
}

/// Ghost/text button style - minimal, for navigation.
#[must_use]
pub fn ghost_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: Palette::JADE,
        border: Border::default().rounded(8),
        shadow: Shadow::default(),
        ..Default::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Palette::HOVER.into()),
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(Palette::HOVER.into()),
            text_color: Palette::EMERALD,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Palette::STONE,
            ..base
        },
    }
}

/// Ghost button for dark backgrounds (sidebar).
#[must_use]
pub fn ghost_button_dark(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: None,
        text_color: Palette::CREAM,
        border: Border::default().rounded(8),
        shadow: Shadow::default(),
        ..Default::default()
    };

    match status {
        button::Status::Active | button::Status::Pressed => button::Style {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.1).into()),
            ..base
        },
        button::Status::Hovered => button::Style {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
            text_color: Palette::JADE,
            ..base
        },
        button::Status::Disabled => button::Style {
            text_color: Palette::SAGE,
            ..base
        },
    }
}

/// Sidebar item button style.
#[must_use]
pub fn sidebar_item_button(
    _theme: &Theme,
    status: button::Status,
    is_selected: bool,
) -> button::Style {
    if is_selected {
        button::Style {
            background: Some(Color::from_rgba(0.298, 0.686, 0.514, 0.2).into()),
            text_color: Palette::JADE,
            border: Border {
                color: Palette::JADE.scale_alpha(0.4),
                width: 0.0,
                radius: 10.into(),
            },
            shadow: Shadow::default(),
            ..Default::default()
        }
    } else {
        match status {
            button::Status::Active | button::Status::Pressed => button::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.08).into()),
                text_color: Palette::CREAM,
                border: Border::default().rounded(10),
                shadow: Shadow::default(),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.06).into()),
                text_color: Palette::CREAM,
                border: Border::default().rounded(10),
                shadow: Shadow::default(),
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: None,
                text_color: Palette::SAGE,
                border: Border::default().rounded(10),
                shadow: Shadow::default(),
                ..Default::default()
            },
        }
    }
}

/// Tab button style.
#[must_use]
pub fn tab_button(_theme: &Theme, status: button::Status, is_selected: bool) -> button::Style {
    if is_selected {
        button::Style {
            background: Some(Palette::JADE.into()),
            text_color: Palette::DEEP_FOREST,
            border: Border::default().rounded(8),
            shadow: Shadow {
                color: Color::from_rgba(0.298, 0.686, 0.514, 0.25),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0,
            },
            ..Default::default()
        }
    } else {
        match status {
            button::Status::Hovered => button::Style {
                background: Some(Palette::HOVER.into()),
                text_color: Palette::JADE,
                border: Border::default().rounded(8),
                shadow: Shadow::default(),
                ..Default::default()
            },
            _ => button::Style {
                background: None,
                text_color: Palette::STONE,
                border: Border::default().rounded(8),
                shadow: Shadow::default(),
                ..Default::default()
            },
        }
    }
}

/// Icon button style (circular, for actions).
#[must_use]
pub fn icon_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.05).into()),
        text_color: Palette::CHARCOAL,
        border: Border::default().rounded(999), // Circular
        shadow: Shadow::default(),
        ..Default::default()
    };

    match status {
        button::Status::Hovered => button::Style {
            background: Some(Palette::HOVER.into()),
            text_color: Palette::JADE,
            ..base
        },
        button::Status::Pressed | button::Status::Active => button::Style {
            background: Some(Palette::SELECTED.into()),
            text_color: Palette::JADE,
            ..base
        },
        button::Status::Disabled => button::Style {
            background: None,
            text_color: Palette::STONE,
            ..base
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// INPUT STYLES
// ═══════════════════════════════════════════════════════════════════════════

/// Input field style - warm and inviting.
#[must_use]
pub fn input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Palette::WARM_WHITE.into(),
        border: Border {
            color: Palette::BORDER,
            width: 1.5,
            radius: 12.into(),
        },
        icon: Palette::STONE,
        placeholder: Palette::STONE,
        value: Palette::CHARCOAL,
        selection: Palette::JADE.scale_alpha(0.3),
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: Palette::JADE,
                width: 2.0,
                radius: 12.into(),
            },
            ..base
        },
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: Palette::SAGE,
                width: 1.5,
                radius: 12.into(),
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Color::from_rgb(0.95, 0.94, 0.92).into(),
            value: Palette::STONE,
            ..base
        },
    }
}

/// Input field style for dark backgrounds.
#[must_use]
pub fn input_style_dark(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Palette::FERN.into(),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.15),
            width: 1.0,
            radius: 12.into(),
        },
        icon: Palette::SAGE,
        placeholder: Palette::SAGE,
        value: Palette::CREAM,
        selection: Palette::JADE.scale_alpha(0.4),
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Focused { .. } => text_input::Style {
            border: Border {
                color: Palette::JADE,
                width: 2.0,
                radius: 12.into(),
            },
            ..base
        },
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.25),
                width: 1.0,
                radius: 12.into(),
            },
            ..base
        },
        text_input::Status::Disabled => text_input::Style {
            background: Palette::MOSS.into(),
            value: Palette::SAGE,
            ..base
        },
    }
}

/// Search input style - pill-shaped.
#[must_use]
pub fn search_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let base = text_input::Style {
        background: Color::from_rgba(0.0, 0.0, 0.0, 0.04).into(),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 999.into(), // Pill shape
        },
        icon: Palette::STONE,
        placeholder: Palette::STONE,
        value: Palette::CHARCOAL,
        selection: Palette::JADE.scale_alpha(0.3),
    };

    match status {
        text_input::Status::Active => base,
        text_input::Status::Focused { .. } => text_input::Style {
            background: Palette::WARM_WHITE.into(),
            border: Border {
                color: Palette::JADE,
                width: 2.0,
                radius: 999.into(),
            },
            ..base
        },
        text_input::Status::Hovered => text_input::Style {
            background: Color::from_rgba(0.0, 0.0, 0.0, 0.06).into(),
            ..base
        },
        text_input::Status::Disabled => base,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCROLLABLE STYLES
// ═══════════════════════════════════════════════════════════════════════════

/// Scrollbar style for light backgrounds.
#[must_use]
pub fn scrollbar_style(_theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_color = Color::from_rgba(0.0, 0.0, 0.0, 0.15);
    let jade_scroller = Palette::JADE.scale_alpha(0.6);
    let active_jade = Palette::JADE;

    let auto_scroll_style = scrollable::AutoScroll {
        background: Color::TRANSPARENT.into(),
        border: Border::default(),
        shadow: Shadow::default(),
        icon: Palette::JADE,
    };

    let base = scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()),
            border: Border::default().rounded(4),
            scroller: scrollable::Scroller {
                background: scroller_color.into(),
                border: Border::default().rounded(4),
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()),
            border: Border::default().rounded(4),
            scroller: scrollable::Scroller {
                background: scroller_color.into(),
                border: Border::default().rounded(4),
            },
        },
        gap: None,
        auto_scroll: auto_scroll_style,
    };

    match status {
        scrollable::Status::Active { .. } => base,
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
            ..
        } => scrollable::Style {
            vertical_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_vertical_scrollbar_hovered {
                        jade_scroller.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.vertical_rail
            },
            horizontal_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_horizontal_scrollbar_hovered {
                        jade_scroller.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.horizontal_rail
            },
            ..base
        },
        scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
            ..
        } => scrollable::Style {
            vertical_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_vertical_scrollbar_dragged {
                        active_jade.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.vertical_rail
            },
            horizontal_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_horizontal_scrollbar_dragged {
                        active_jade.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.horizontal_rail
            },
            ..base
        },
    }
}

/// Scrollbar style for dark backgrounds.
#[must_use]
pub fn scrollbar_style_dark(_theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let scroller_color = Color::from_rgba(1.0, 1.0, 1.0, 0.2);
    let jade_scroller = Palette::JADE.scale_alpha(0.7);
    let active_jade = Palette::JADE;

    let auto_scroll_style = scrollable::AutoScroll {
        background: Color::TRANSPARENT.into(),
        border: Border::default(),
        shadow: Shadow::default(),
        icon: Palette::SAGE,
    };

    let base = scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
            border: Border::default().rounded(4),
            scroller: scrollable::Scroller {
                background: scroller_color.into(),
                border: Border::default().rounded(4),
            },
        },
        horizontal_rail: scrollable::Rail {
            background: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.03).into()),
            border: Border::default().rounded(4),
            scroller: scrollable::Scroller {
                background: scroller_color.into(),
                border: Border::default().rounded(4),
            },
        },
        gap: None,
        auto_scroll: auto_scroll_style,
    };

    match status {
        scrollable::Status::Active { .. } => base,
        scrollable::Status::Hovered {
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
            ..
        } => scrollable::Style {
            vertical_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_vertical_scrollbar_hovered {
                        jade_scroller.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.vertical_rail
            },
            horizontal_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_horizontal_scrollbar_hovered {
                        jade_scroller.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.horizontal_rail
            },
            ..base
        },
        scrollable::Status::Dragged {
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
            ..
        } => scrollable::Style {
            vertical_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_vertical_scrollbar_dragged {
                        active_jade.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.vertical_rail
            },
            horizontal_rail: scrollable::Rail {
                scroller: scrollable::Scroller {
                    background: if is_horizontal_scrollbar_dragged {
                        active_jade.into()
                    } else {
                        scroller_color.into()
                    },
                    border: Border::default().rounded(4),
                },
                ..base.horizontal_rail
            },
            ..base
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STATUS INDICATOR STYLES
// ═══════════════════════════════════════════════════════════════════════════

/// Status indicator style.
#[must_use]
pub fn status_indicator(status: crate::state::ContactStatus) -> container::Style {
    container::Style {
        background: Some(status.color().into()),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.2),
            width: 1.0,
            radius: 999.into(), // Circular
        },
        shadow: Shadow {
            color: Color::from_rgba(status.color().r, status.color().g, status.color().b, 0.4),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 4.0,
        },
        ..Default::default()
    }
}

/// Badge style for notifications/counts.
#[must_use]
pub fn badge_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::ERROR.into()),
        border: Border::default().rounded(999),
        text_color: Some(Palette::CREAM),
        ..Default::default()
    }
}

/// Subtle badge style.
#[must_use]
pub fn subtle_badge_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Palette::JADE.scale_alpha(0.15).into()),
        border: Border::default().rounded(6),
        text_color: Some(Palette::JADE),
        ..Default::default()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DIVIDER STYLES
// ═══════════════════════════════════════════════════════════════════════════

/// Divider color for light backgrounds.
pub const DIVIDER_LIGHT: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);

/// Divider color for dark backgrounds.
pub const DIVIDER_DARK: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.1);

// ═══════════════════════════════════════════════════════════════════════════
// ANIMATION TIMING (for reference - Iced uses subscriptions for animation)
// ═══════════════════════════════════════════════════════════════════════════

/// Standard animation duration in milliseconds.
pub const ANIMATION_DURATION_MS: u64 = 200;

/// Fast animation duration in milliseconds.
pub const ANIMATION_FAST_MS: u64 = 100;

/// Slow animation duration in milliseconds.
pub const ANIMATION_SLOW_MS: u64 = 400;
