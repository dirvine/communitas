//! Style builder utilities for the enhanced design system.
//!
//! Provides composable style functions for Dioxus components.

use crate::design_tokens::{gradients, motion, palette, radius, semantic, shadow, spacing, typography};

/// Glass card style - warm-tinted frosted glass effect.
pub fn glass_card() -> String {
    format!(
        "background: {}; \
         backdrop-filter: blur(20px) saturate(180%); \
         -webkit-backdrop-filter: blur(20px) saturate(180%); \
         border: 1px solid {}; \
         border-radius: {}; \
         box-shadow: {}, {}; \
         position: relative; \
         overflow: hidden;",
        semantic::GLASS_BG,
        semantic::BORDER_DEFAULT,
        radius::XL,
        shadow::LG,
        shadow::GLOW_SM
    )
}

/// Glass card with glow effect for emphasis.
pub fn glass_card_glow() -> String {
    format!(
        "background: {}; \
         backdrop-filter: blur(24px) saturate(200%); \
         -webkit-backdrop-filter: blur(24px) saturate(200%); \
         border: 1px solid {}; \
         border-radius: {}; \
         box-shadow: {}, {};",
        semantic::GLASS_BG,
        semantic::BORDER_STRONG,
        radius::XL,
        shadow::XL,
        shadow::GLOW_MD
    )
}

/// Elevated surface style.
pub fn surface_elevated() -> String {
    format!(
        "background-color: {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         box-shadow: {};",
        semantic::BG_TERTIARY,
        semantic::BORDER_SUBTLE,
        radius::LG,
        shadow::MD
    )
}

/// Primary button style.
pub fn button_primary() -> String {
    format!(
        "background: {}; \
         color: {}; \
         font-family: {}; \
         font-size: {}; \
         font-weight: {}; \
         padding: {} {}; \
         border: none; \
         border-radius: {}; \
         cursor: pointer; \
         transition: {}; \
         box-shadow: {}, {};",
        gradients::BUTTON_PRIMARY,
        "#ffffff",
        typography::FONT_BODY,
        typography::SIZE_BASE,
        typography::WEIGHT_SEMIBOLD,
        spacing::MD,
        spacing::XL,
        radius::LG,
        motion::transition("all"),
        shadow::MD,
        shadow::GLOW_SM
    )
}

/// Primary button hover state.
pub fn button_primary_hover() -> String {
    format!(
        "background: {}; \
         transform: translateY(-1px); \
         box-shadow: {}, {};",
        "linear-gradient(135deg, #34d399 0%, #10b981 100%)",
        shadow::LG,
        shadow::GLOW_MD
    )
}

/// Secondary/ghost button style.
pub fn button_secondary() -> String {
    format!(
        "background: transparent; \
         color: {}; \
         font-family: {}; \
         font-size: {}; \
         font-weight: {}; \
         padding: {} {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         cursor: pointer; \
         transition: {};",
        semantic::TEXT_PRIMARY,
        typography::FONT_BODY,
        typography::SIZE_BASE,
        typography::WEIGHT_MEDIUM,
        spacing::MD,
        spacing::XL,
        semantic::BORDER_DEFAULT,
        radius::LG,
        motion::transition("all")
    )
}

/// Ghost button hover.
pub fn button_secondary_hover() -> String {
    format!(
        "background: {}; \
         border-color: {}; \
         color: {};",
        semantic::BG_HOVER,
        semantic::BORDER_STRONG,
        palette::JADE_400
    )
}

/// Icon button style.
pub fn button_icon() -> String {
    format!(
        "display: flex; \
         align-items: center; \
         justify-content: center; \
         width: {}; \
         height: {}; \
         background: transparent; \
         color: {}; \
         border: none; \
         border-radius: {}; \
         cursor: pointer; \
         transition: {};",
        spacing::XXL,
        spacing::XXL,
        semantic::TEXT_SECONDARY,
        radius::MD,
        motion::transition("all")
    )
}

/// Text input style.
pub fn input_text() -> String {
    format!(
        "width: 100%; \
         background: {}; \
         color: {}; \
         font-family: {}; \
         font-size: {}; \
         padding: {} {}; \
         border: 1px solid {}; \
         border-radius: {}; \
         outline: none; \
         transition: {};",
        semantic::BG_TERTIARY,
        semantic::TEXT_PRIMARY,
        typography::FONT_BODY,
        typography::SIZE_BASE,
        spacing::MD,
        spacing::BASE,
        semantic::BORDER_SUBTLE,
        radius::LG,
        motion::transition("all")
    )
}

/// Input focus state.
pub fn input_focus() -> String {
    format!(
        "border-color: {}; \
         box-shadow: 0 0 0 3px {};",
        semantic::PRIMARY,
        "rgba(16, 185, 129, 0.15)"
    )
}

/// Textarea style.
pub fn input_textarea() -> String {
    format!(
        "{}; \
         min-height: 100px; \
         resize: vertical;",
        input_text()
    )
}

/// Label style.
pub fn label() -> String {
    format!(
        "display: block; \
         color: {}; \
         font-family: {}; \
         font-size: {}; \
         font-weight: {}; \
         margin-bottom: {};",
        semantic::TEXT_SECONDARY,
        typography::FONT_BODY,
        typography::SIZE_SM,
        typography::WEIGHT_MEDIUM,
        spacing::SM
    )
}

/// Heading styles.
pub mod heading {
    use super::*;

    pub fn h1() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             font-weight: {}; \
             line-height: {}; \
             letter-spacing: {}; \
             margin: 0;",
            semantic::TEXT_PRIMARY,
            typography::FONT_DISPLAY,
            typography::SIZE_4XL,
            typography::WEIGHT_BOLD,
            typography::LEADING_TIGHT,
            typography::TRACKING_TIGHT
        )
    }

    pub fn h2() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             font-weight: {}; \
             line-height: {}; \
             letter-spacing: {}; \
             margin: 0;",
            semantic::TEXT_PRIMARY,
            typography::FONT_DISPLAY,
            typography::SIZE_3XL,
            typography::WEIGHT_BOLD,
            typography::LEADING_TIGHT,
            typography::TRACKING_TIGHT
        )
    }

    pub fn h3() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             font-weight: {}; \
             line-height: {}; \
             margin: 0;",
            semantic::TEXT_PRIMARY,
            typography::FONT_DISPLAY,
            typography::SIZE_2XL,
            typography::WEIGHT_SEMIBOLD,
            typography::LEADING_TIGHT
        )
    }

    pub fn h4() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             font-weight: {}; \
             line-height: {}; \
             margin: 0;",
            semantic::TEXT_PRIMARY,
            typography::FONT_BODY,
            typography::SIZE_XL,
            typography::WEIGHT_SEMIBOLD,
            typography::LEADING_NORMAL
        )
    }

    /// Section header with accent line.
    pub fn section() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             font-weight: {}; \
             text-transform: uppercase; \
             letter-spacing: {}; \
             margin: 0;",
            semantic::TEXT_MUTED,
            typography::FONT_BODY,
            typography::SIZE_XS,
            typography::WEIGHT_SEMIBOLD,
            typography::TRACKING_WIDER
        )
    }
}

/// Text styles.
pub mod text {
    use super::*;

    pub fn body() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             line-height: {};",
            semantic::TEXT_PRIMARY,
            typography::FONT_BODY,
            typography::SIZE_BASE,
            typography::LEADING_NORMAL
        )
    }

    pub fn secondary() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             line-height: {};",
            semantic::TEXT_SECONDARY,
            typography::FONT_BODY,
            typography::SIZE_BASE,
            typography::LEADING_NORMAL
        )
    }

    pub fn muted() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             line-height: {};",
            semantic::TEXT_MUTED,
            typography::FONT_BODY,
            typography::SIZE_SM,
            typography::LEADING_NORMAL
        )
    }

    pub fn mono() -> String {
        format!(
            "color: {}; \
             font-family: {}; \
             font-size: {}; \
             line-height: {};",
            semantic::ACCENT,
            typography::FONT_MONO,
            typography::SIZE_SM,
            typography::LEADING_NORMAL
        )
    }

    pub fn link() -> String {
        format!(
            "color: {}; \
             text-decoration: none; \
             transition: {}; \
             cursor: pointer;",
            semantic::PRIMARY,
            motion::transition("color")
        )
    }

    pub fn link_hover() -> String {
        format!("color: {};", semantic::PRIMARY_HOVER)
    }
}

/// Badge/chip styles.
pub mod badge {
    use super::*;

    pub fn base() -> String {
        format!(
            "display: inline-flex; \
             align-items: center; \
             padding: {} {}; \
             font-family: {}; \
             font-size: {}; \
             font-weight: {}; \
             border-radius: {}; \
             white-space: nowrap;",
            spacing::XXS,
            spacing::SM,
            typography::FONT_BODY,
            typography::SIZE_XS,
            typography::WEIGHT_MEDIUM,
            radius::FULL
        )
    }

    pub fn primary() -> String {
        format!(
            "{} \
             background: rgba(16, 185, 129, 0.15); \
             color: {};",
            base(),
            palette::JADE_400
        )
    }

    pub fn secondary() -> String {
        format!(
            "{} \
             background: {}; \
             color: {};",
            base(),
            semantic::BG_ELEVATED,
            semantic::TEXT_SECONDARY
        )
    }

    pub fn with_color(color: &str) -> String {
        format!(
            "{} \
             background: {}15; \
             color: {};",
            base(),
            color,
            color
        )
    }
}

/// Presence dot styles.
pub mod presence {
    use super::*;

    fn dot_base() -> String {
        format!(
            "width: 10px; \
             height: 10px; \
             border-radius: {}; \
             border: 2px solid {};",
            radius::FULL,
            semantic::BG_PRIMARY
        )
    }

    pub fn online() -> String {
        format!(
            "{} \
             background: {}; \
             box-shadow: 0 0 6px {};",
            dot_base(),
            semantic::PRESENCE_ONLINE,
            semantic::PRESENCE_ONLINE
        )
    }

    pub fn away() -> String {
        format!(
            "{} \
             background: {};",
            dot_base(),
            semantic::PRESENCE_AWAY
        )
    }

    pub fn busy() -> String {
        format!(
            "{} \
             background: {};",
            dot_base(),
            semantic::PRESENCE_BUSY
        )
    }

    pub fn offline() -> String {
        format!(
            "{} \
             background: {};",
            dot_base(),
            semantic::PRESENCE_OFFLINE
        )
    }
}

/// Entity icon color.
pub fn entity_color(entity_type: &str) -> &'static str {
    match entity_type {
        "organization" | "Organisation" => semantic::ENTITY_ORG,
        "project" | "Project" => semantic::ENTITY_PROJECT,
        "channel" | "Channel" => semantic::ENTITY_CHANNEL,
        "group" | "Group" => semantic::ENTITY_GROUP,
        "person" | "Person" | "contact" => semantic::ENTITY_PERSON,
        _ => semantic::TEXT_MUTED,
    }
}

/// Avatar styles.
pub mod avatar {
    use super::*;

    pub fn base(size: &str) -> String {
        format!(
            "width: {}; \
             height: {}; \
             border-radius: {}; \
             overflow: hidden; \
             display: flex; \
             align-items: center; \
             justify-content: center; \
             font-family: {}; \
             font-weight: {}; \
             flex-shrink: 0;",
            size,
            size,
            radius::FULL,
            typography::FONT_DISPLAY,
            typography::WEIGHT_SEMIBOLD
        )
    }

    pub fn sm() -> String {
        format!("{} font-size: {};", base("28px"), typography::SIZE_XS)
    }

    pub fn md() -> String {
        format!("{} font-size: {};", base("36px"), typography::SIZE_SM)
    }

    pub fn lg() -> String {
        format!("{} font-size: {};", base("48px"), typography::SIZE_BASE)
    }

    pub fn xl() -> String {
        format!("{} font-size: {};", base("64px"), typography::SIZE_LG)
    }

    pub fn with_bg(bg_color: &str, text_color: &str) -> String {
        format!(
            "background-color: {}; color: {};",
            bg_color, text_color
        )
    }
}

/// Scrollbar styling.
pub fn custom_scrollbar() -> String {
    // Note: This should be applied via CSS class in index.html for webkit browsers
    "scrollbar-width: thin; scrollbar-color: rgba(52, 211, 153, 0.2) transparent;".to_string()
}

/// Flex utilities.
pub mod flex {
    pub fn row() -> &'static str {
        "display: flex; flex-direction: row;"
    }

    pub fn col() -> &'static str {
        "display: flex; flex-direction: column;"
    }

    pub fn center() -> &'static str {
        "display: flex; align-items: center; justify-content: center;"
    }

    pub fn between() -> &'static str {
        "display: flex; align-items: center; justify-content: space-between;"
    }

    pub fn start() -> &'static str {
        "display: flex; align-items: center; justify-content: flex-start;"
    }

    pub fn gap(size: &str) -> String {
        format!("gap: {};", size)
    }
}
