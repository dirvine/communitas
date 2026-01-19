//! Design tokens for consistent styling across Communitas UI.
//!
//! These tokens define the visual language of the application:
//! - Colors: semantic color palette (primary, danger, surfaces, text)
//! - Spacing: consistent margin/padding values
//! - Typography: font families and sizes
//! - Radius: border radius values
//! - Shadow: box shadow presets
//! - Transition: animation timing

/// Semantic color tokens following Tailwind's naming convention.
pub mod colors {
    // Primary brand colors
    /// Primary action color (emerald-500)
    pub const PRIMARY: &str = "#10b981";
    /// Primary hover state (emerald-600)
    pub const PRIMARY_HOVER: &str = "#059669";
    /// Secondary accent color (blue-500)
    pub const SECONDARY: &str = "#3b82f6";
    /// Danger/error color (red-500)
    pub const DANGER: &str = "#ef4444";
    /// Warning color (amber-500)
    pub const WARNING: &str = "#f59e0b";
    /// Success color (green-500)
    pub const SUCCESS: &str = "#22c55e";

    // Surface colors (dark theme)
    /// Main background (slate-900)
    pub const SURFACE_BG: &str = "#0f172a";
    /// Card background (slate-800)
    pub const SURFACE_CARD: &str = "#1e293b";
    /// Elevated surface (slate-700)
    pub const SURFACE_ELEVATED: &str = "#334155";
    /// Overlay with transparency
    pub const SURFACE_OVERLAY: &str = "rgba(15, 23, 42, 0.8)";

    // Text colors
    /// Primary text (slate-50)
    pub const TEXT_PRIMARY: &str = "#f8fafc";
    /// Secondary text (slate-400)
    pub const TEXT_SECONDARY: &str = "#94a3b8";
    /// Muted text (slate-500)
    pub const TEXT_MUTED: &str = "#64748b";
    /// Text on primary/light backgrounds (slate-900)
    pub const TEXT_INVERSE: &str = "#0f172a";

    // Border colors
    /// Default border (slate-700)
    pub const BORDER_DEFAULT: &str = "#334155";
    /// Focus ring color (emerald-500)
    pub const BORDER_FOCUS: &str = "#10b981";
}

/// Spacing tokens using rem units for responsive scaling.
pub mod spacing {
    /// Extra small: 4px
    pub const XS: &str = "0.25rem";
    /// Small: 8px
    pub const SM: &str = "0.5rem";
    /// Medium: 16px
    pub const MD: &str = "1rem";
    /// Large: 24px
    pub const LG: &str = "1.5rem";
    /// Extra large: 32px
    pub const XL: &str = "2rem";
    /// 2x Extra large: 48px
    pub const XXL: &str = "3rem";
}

/// Border radius tokens.
pub mod radius {
    /// Small: 4px
    pub const SM: &str = "0.25rem";
    /// Medium: 6px
    pub const MD: &str = "0.375rem";
    /// Large: 8px
    pub const LG: &str = "0.5rem";
    /// Extra large: 12px
    pub const XL: &str = "0.75rem";
    /// Full/pill: 9999px
    pub const FULL: &str = "9999px";
}

/// Typography tokens.
pub mod typography {
    /// System sans-serif font stack
    pub const FONT_SANS: &str = "system-ui, -apple-system, sans-serif";
    /// Monospace font stack
    pub const FONT_MONO: &str = "ui-monospace, monospace";

    /// Extra small text: 12px
    pub const TEXT_XS: &str = "0.75rem";
    /// Small text: 14px
    pub const TEXT_SM: &str = "0.875rem";
    /// Base text: 16px
    pub const TEXT_BASE: &str = "1rem";
    /// Large text: 18px
    pub const TEXT_LG: &str = "1.125rem";
    /// Extra large text: 20px
    pub const TEXT_XL: &str = "1.25rem";
    /// 2x Extra large text: 24px
    pub const TEXT_2XL: &str = "1.5rem";
}

/// Box shadow presets.
pub mod shadow {
    /// Small shadow
    pub const SM: &str = "0 1px 2px rgba(0, 0, 0, 0.3)";
    /// Medium shadow
    pub const MD: &str = "0 4px 6px rgba(0, 0, 0, 0.3)";
    /// Large shadow
    pub const LG: &str = "0 10px 15px rgba(0, 0, 0, 0.3)";
}

/// Transition timing presets.
pub mod transition {
    /// Fast: 150ms
    pub const FAST: &str = "150ms ease-in-out";
    /// Default: 200ms
    pub const DEFAULT: &str = "200ms ease-in-out";
    /// Slow: 300ms
    pub const SLOW: &str = "300ms ease-in-out";
}

// Style builder helpers

/// Generate background-color CSS property.
pub fn style_bg(color: &str) -> String {
    format!("background-color: {};", color)
}

/// Generate color CSS property.
pub fn style_text(color: &str) -> String {
    format!("color: {};", color)
}

/// Generate border CSS property.
pub fn style_border(color: &str, width: &str) -> String {
    format!("border: {} solid {};", width, color)
}

/// Generate padding CSS property.
pub fn style_padding(value: &str) -> String {
    format!("padding: {};", value)
}

/// Generate margin CSS property.
pub fn style_margin(value: &str) -> String {
    format!("margin: {};", value)
}

/// Generate border-radius CSS property.
pub fn style_rounded(value: &str) -> String {
    format!("border-radius: {};", value)
}

/// Generate box-shadow CSS property.
pub fn style_shadow(value: &str) -> String {
    format!("box-shadow: {};", value)
}

/// Generate transition CSS property.
pub fn style_transition(value: &str) -> String {
    format!("transition: all {};", value)
}

/// Combine multiple style strings into one.
pub fn styles(parts: &[&str]) -> String {
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_tokens_are_valid_hex() {
        // All hex colors should start with #
        assert!(colors::PRIMARY.starts_with('#'));
        assert!(colors::PRIMARY_HOVER.starts_with('#'));
        assert!(colors::SECONDARY.starts_with('#'));
        assert!(colors::DANGER.starts_with('#'));
        assert!(colors::WARNING.starts_with('#'));
        assert!(colors::SUCCESS.starts_with('#'));
        assert!(colors::SURFACE_BG.starts_with('#'));
        assert!(colors::SURFACE_CARD.starts_with('#'));
        assert!(colors::SURFACE_ELEVATED.starts_with('#'));
        assert!(colors::TEXT_PRIMARY.starts_with('#'));
        assert!(colors::TEXT_SECONDARY.starts_with('#'));
        assert!(colors::TEXT_MUTED.starts_with('#'));
        assert!(colors::TEXT_INVERSE.starts_with('#'));
        assert!(colors::BORDER_DEFAULT.starts_with('#'));
        assert!(colors::BORDER_FOCUS.starts_with('#'));
    }

    #[test]
    fn color_overlay_is_rgba() {
        assert!(colors::SURFACE_OVERLAY.starts_with("rgba("));
    }

    #[test]
    fn spacing_tokens_are_rem() {
        assert!(spacing::XS.ends_with("rem"));
        assert!(spacing::SM.ends_with("rem"));
        assert!(spacing::MD.ends_with("rem"));
        assert!(spacing::LG.ends_with("rem"));
        assert!(spacing::XL.ends_with("rem"));
        assert!(spacing::XXL.ends_with("rem"));
    }

    #[test]
    fn radius_tokens_are_valid() {
        assert!(radius::SM.ends_with("rem"));
        assert!(radius::MD.ends_with("rem"));
        assert!(radius::LG.ends_with("rem"));
        assert!(radius::XL.ends_with("rem"));
        assert!(radius::FULL.ends_with("px"));
    }

    #[test]
    fn typography_sizes_are_rem() {
        assert!(typography::TEXT_XS.ends_with("rem"));
        assert!(typography::TEXT_SM.ends_with("rem"));
        assert!(typography::TEXT_BASE.ends_with("rem"));
        assert!(typography::TEXT_LG.ends_with("rem"));
        assert!(typography::TEXT_XL.ends_with("rem"));
        assert!(typography::TEXT_2XL.ends_with("rem"));
    }

    #[test]
    fn shadow_tokens_contain_rgba() {
        assert!(shadow::SM.contains("rgba"));
        assert!(shadow::MD.contains("rgba"));
        assert!(shadow::LG.contains("rgba"));
    }

    #[test]
    fn transition_tokens_contain_ms() {
        assert!(transition::FAST.contains("ms"));
        assert!(transition::DEFAULT.contains("ms"));
        assert!(transition::SLOW.contains("ms"));
    }

    #[test]
    fn style_bg_generates_css() {
        let css = style_bg(colors::PRIMARY);
        assert_eq!(css, "background-color: #10b981;");
    }

    #[test]
    fn style_text_generates_css() {
        let css = style_text(colors::TEXT_PRIMARY);
        assert_eq!(css, "color: #f8fafc;");
    }

    #[test]
    fn style_border_generates_css() {
        let css = style_border(colors::BORDER_DEFAULT, "1px");
        assert_eq!(css, "border: 1px solid #334155;");
    }

    #[test]
    fn style_padding_generates_css() {
        let css = style_padding(spacing::MD);
        assert_eq!(css, "padding: 1rem;");
    }

    #[test]
    fn style_rounded_generates_css() {
        let css = style_rounded(radius::LG);
        assert_eq!(css, "border-radius: 0.5rem;");
    }

    #[test]
    fn styles_combines_parts() {
        let css = styles(&["color: red;", "background: blue;"]);
        assert_eq!(css, "color: red; background: blue;");
    }
}
