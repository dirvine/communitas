//! Component style presets using design tokens.
//!
//! Provides ready-to-use style strings for common UI patterns.
//! These presets ensure consistency across components while allowing
//! customization through token values.

use crate::tokens::{colors, radius, shadow, spacing, transition};

/// Button style presets.
pub mod button {
    use super::*;

    /// Primary action button (emerald background).
    pub fn primary() -> String {
        format!(
            "background-color: {}; color: {}; padding: {} {}; border-radius: {}; transition: all {}; border: none; cursor: pointer;",
            colors::PRIMARY,
            colors::TEXT_INVERSE,
            spacing::SM,
            spacing::MD,
            radius::MD,
            transition::DEFAULT
        )
    }

    /// Primary button hover state.
    pub fn primary_hover() -> String {
        format!("background-color: {};", colors::PRIMARY_HOVER)
    }

    /// Secondary/outline button.
    pub fn secondary() -> String {
        format!(
            "background-color: transparent; color: {}; border: 1px solid {}; padding: {} {}; border-radius: {}; cursor: pointer; transition: all {};",
            colors::TEXT_PRIMARY,
            colors::BORDER_DEFAULT,
            spacing::SM,
            spacing::MD,
            radius::MD,
            transition::DEFAULT
        )
    }

    /// Secondary button hover state.
    pub fn secondary_hover() -> String {
        format!(
            "background-color: {}; border-color: {};",
            colors::SURFACE_ELEVATED,
            colors::TEXT_SECONDARY
        )
    }

    /// Danger/destructive button.
    pub fn danger() -> String {
        format!(
            "background-color: {}; color: {}; padding: {} {}; border-radius: {}; border: none; cursor: pointer; transition: all {};",
            colors::DANGER,
            colors::TEXT_PRIMARY,
            spacing::SM,
            spacing::MD,
            radius::MD,
            transition::DEFAULT
        )
    }

    /// Ghost button (minimal styling).
    pub fn ghost() -> String {
        format!(
            "background-color: transparent; color: {}; padding: {} {}; border-radius: {}; border: none; cursor: pointer; transition: all {};",
            colors::TEXT_SECONDARY,
            spacing::SM,
            spacing::MD,
            radius::MD,
            transition::FAST
        )
    }

    /// Ghost button hover state.
    pub fn ghost_hover() -> String {
        format!(
            "background-color: {}; color: {};",
            colors::SURFACE_ELEVATED,
            colors::TEXT_PRIMARY
        )
    }
}

/// Card style presets.
pub mod card {
    use super::*;

    /// Default card styling.
    pub fn default() -> String {
        format!(
            "background-color: {}; border-radius: {}; padding: {}; box-shadow: {};",
            colors::SURFACE_CARD,
            radius::LG,
            spacing::MD,
            shadow::MD
        )
    }

    /// Elevated card with stronger shadow.
    pub fn elevated() -> String {
        format!(
            "background-color: {}; border-radius: {}; padding: {}; box-shadow: {};",
            colors::SURFACE_ELEVATED,
            radius::LG,
            spacing::MD,
            shadow::LG
        )
    }

    /// Interactive card with hover state support.
    pub fn interactive() -> String {
        format!(
            "background-color: {}; border-radius: {}; padding: {}; box-shadow: {}; cursor: pointer; transition: all {};",
            colors::SURFACE_CARD,
            radius::LG,
            spacing::MD,
            shadow::SM,
            transition::DEFAULT
        )
    }

    /// Interactive card hover state.
    pub fn interactive_hover() -> String {
        format!(
            "background-color: {}; box-shadow: {}; transform: translateY(-2px);",
            colors::SURFACE_ELEVATED,
            shadow::MD
        )
    }

    /// Card with border instead of shadow.
    pub fn bordered() -> String {
        format!(
            "background-color: {}; border-radius: {}; padding: {}; border: 1px solid {};",
            colors::SURFACE_CARD,
            radius::LG,
            spacing::MD,
            colors::BORDER_DEFAULT
        )
    }
}

/// Input style presets.
pub mod input {
    use super::*;

    /// Default input field styling.
    pub fn default() -> String {
        format!(
            "background-color: {}; color: {}; border: 1px solid {}; padding: {} {}; border-radius: {}; transition: all {};",
            colors::SURFACE_BG,
            colors::TEXT_PRIMARY,
            colors::BORDER_DEFAULT,
            spacing::SM,
            spacing::MD,
            radius::MD,
            transition::FAST
        )
    }

    /// Input focus state.
    pub fn focus() -> String {
        format!(
            "border-color: {}; outline: 2px solid {}; outline-offset: 2px;",
            colors::BORDER_FOCUS,
            colors::BORDER_FOCUS
        )
    }

    /// Input error state.
    pub fn error() -> String {
        format!(
            "border-color: {}; outline: 2px solid {}; outline-offset: 2px;",
            colors::DANGER,
            colors::DANGER
        )
    }

    /// Disabled input state.
    pub fn disabled() -> String {
        format!(
            "background-color: {}; color: {}; cursor: not-allowed; opacity: 0.6;",
            colors::SURFACE_ELEVATED,
            colors::TEXT_MUTED
        )
    }
}

/// Panel/container style presets.
pub mod panel {
    use super::*;

    /// Sidebar panel.
    pub fn sidebar() -> String {
        format!(
            "background-color: {}; width: 16rem; padding: {}; height: 100%;",
            colors::SURFACE_BG,
            spacing::MD
        )
    }

    /// Header bar.
    pub fn header() -> String {
        format!(
            "background-color: {}; padding: {} {}; border-bottom: 1px solid {};",
            colors::SURFACE_CARD,
            spacing::SM,
            spacing::MD,
            colors::BORDER_DEFAULT
        )
    }

    /// Content area.
    pub fn content() -> String {
        format!(
            "background-color: {}; padding: {}; flex: 1; overflow: auto;",
            colors::SURFACE_BG,
            spacing::MD
        )
    }

    /// Modal overlay.
    pub fn overlay() -> String {
        format!(
            "background-color: {}; position: fixed; inset: 0; display: flex; align-items: center; justify-content: center;",
            colors::SURFACE_OVERLAY
        )
    }

    /// Modal container.
    pub fn modal() -> String {
        format!(
            "background-color: {}; border-radius: {}; padding: {}; box-shadow: {}; max-width: 32rem; width: 90%;",
            colors::SURFACE_CARD,
            radius::XL,
            spacing::LG,
            shadow::LG
        )
    }
}

/// Text style presets.
pub mod text {
    use super::*;

    /// Page heading (h1).
    pub fn heading() -> String {
        format!(
            "color: {}; font-size: {}; font-weight: 600; margin-bottom: {};",
            colors::TEXT_PRIMARY,
            crate::tokens::typography::TEXT_2XL,
            spacing::MD
        )
    }

    /// Section heading (h2).
    pub fn subheading() -> String {
        format!(
            "color: {}; font-size: {}; font-weight: 500; margin-bottom: {};",
            colors::TEXT_PRIMARY,
            crate::tokens::typography::TEXT_XL,
            spacing::SM
        )
    }

    /// Body text.
    pub fn body() -> String {
        format!(
            "color: {}; font-size: {}; line-height: 1.5;",
            colors::TEXT_PRIMARY,
            crate::tokens::typography::TEXT_BASE
        )
    }

    /// Secondary/muted text.
    pub fn secondary() -> String {
        format!(
            "color: {}; font-size: {};",
            colors::TEXT_SECONDARY,
            crate::tokens::typography::TEXT_SM
        )
    }

    /// Small/caption text.
    pub fn caption() -> String {
        format!(
            "color: {}; font-size: {};",
            colors::TEXT_MUTED,
            crate::tokens::typography::TEXT_XS
        )
    }

    /// Link text.
    pub fn link() -> String {
        format!(
            "color: {}; text-decoration: none; cursor: pointer; transition: color {};",
            colors::PRIMARY,
            transition::FAST
        )
    }

    /// Link hover state.
    pub fn link_hover() -> String {
        format!("color: {};", colors::PRIMARY_HOVER)
    }
}

/// Badge/tag style presets.
pub mod badge {
    use super::*;

    /// Default badge.
    pub fn default() -> String {
        format!(
            "background-color: {}; color: {}; padding: 0.125rem {}; border-radius: {}; font-size: {};",
            colors::SURFACE_ELEVATED,
            colors::TEXT_SECONDARY,
            spacing::SM,
            radius::FULL,
            crate::tokens::typography::TEXT_XS
        )
    }

    /// Success badge.
    pub fn success() -> String {
        format!(
            "background-color: {}; color: {}; padding: 0.125rem {}; border-radius: {}; font-size: {};",
            colors::SUCCESS,
            colors::TEXT_INVERSE,
            spacing::SM,
            radius::FULL,
            crate::tokens::typography::TEXT_XS
        )
    }

    /// Warning badge.
    pub fn warning() -> String {
        format!(
            "background-color: {}; color: {}; padding: 0.125rem {}; border-radius: {}; font-size: {};",
            colors::WARNING,
            colors::TEXT_INVERSE,
            spacing::SM,
            radius::FULL,
            crate::tokens::typography::TEXT_XS
        )
    }

    /// Danger badge.
    pub fn danger() -> String {
        format!(
            "background-color: {}; color: {}; padding: 0.125rem {}; border-radius: {}; font-size: {};",
            colors::DANGER,
            colors::TEXT_PRIMARY,
            spacing::SM,
            radius::FULL,
            crate::tokens::typography::TEXT_XS
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_primary_contains_background() {
        let style = button::primary();
        assert!(style.contains("background-color:"));
        assert!(style.contains(colors::PRIMARY));
    }

    #[test]
    fn button_secondary_has_border() {
        let style = button::secondary();
        assert!(style.contains("border:"));
        assert!(style.contains("transparent"));
    }

    #[test]
    fn card_default_has_shadow() {
        let style = card::default();
        assert!(style.contains("box-shadow:"));
        assert!(style.contains(colors::SURFACE_CARD));
    }

    #[test]
    fn card_elevated_uses_elevated_surface() {
        let style = card::elevated();
        assert!(style.contains(colors::SURFACE_ELEVATED));
    }

    #[test]
    fn input_default_has_border() {
        let style = input::default();
        assert!(style.contains("border:"));
        assert!(style.contains(colors::BORDER_DEFAULT));
    }

    #[test]
    fn input_focus_has_outline() {
        let style = input::focus();
        assert!(style.contains("outline:"));
        assert!(style.contains(colors::BORDER_FOCUS));
    }

    #[test]
    fn panel_sidebar_has_width() {
        let style = panel::sidebar();
        assert!(style.contains("width: 16rem"));
        assert!(style.contains(colors::SURFACE_BG));
    }

    #[test]
    fn text_heading_has_font_size() {
        let style = text::heading();
        assert!(style.contains("font-size:"));
        assert!(style.contains("font-weight: 600"));
    }

    #[test]
    fn badge_success_uses_success_color() {
        let style = badge::success();
        assert!(style.contains(colors::SUCCESS));
    }

    #[test]
    fn all_presets_end_with_semicolon() {
        // Verify CSS is well-formed (ends with semicolon)
        assert!(button::primary().ends_with(';'));
        assert!(button::secondary().ends_with(';'));
        assert!(card::default().ends_with(';'));
        assert!(input::default().ends_with(';'));
        assert!(panel::sidebar().ends_with(';'));
        assert!(text::body().ends_with(';'));
        assert!(badge::default().ends_with(';'));
    }
}
