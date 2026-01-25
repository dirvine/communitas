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
    /// Info color (blue-500)
    pub const INFO: &str = "#3b82f6";
    /// Error color (red-500)
    pub const ERROR: &str = "#ef4444";

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

/// Responsive breakpoint tokens following Tailwind's naming convention.
///
/// These breakpoints define the screen widths at which layouts change.
/// Use with CSS media queries: `@media (min-width: {breakpoint})`.
pub mod breakpoints {
    /// Small screens (640px) - Mobile landscape
    pub const SM: &str = "640px";
    /// Small screens value in pixels
    pub const SM_PX: u32 = 640;

    /// Medium screens (768px) - Tablets
    pub const MD: &str = "768px";
    /// Medium screens value in pixels
    pub const MD_PX: u32 = 768;

    /// Large screens (1024px) - Small laptops
    pub const LG: &str = "1024px";
    /// Large screens value in pixels
    pub const LG_PX: u32 = 1024;

    /// Extra large screens (1280px) - Desktops
    pub const XL: &str = "1280px";
    /// Extra large screens value in pixels
    pub const XL_PX: u32 = 1280;

    /// 2x Extra large screens (1536px) - Large desktops
    pub const XXL: &str = "1536px";
    /// 2x Extra large screens value in pixels
    pub const XXL_PX: u32 = 1536;

    /// Container max-widths for each breakpoint.
    pub mod container {
        /// Max-width at SM breakpoint
        pub const SM: &str = "640px";
        /// Max-width at MD breakpoint
        pub const MD: &str = "768px";
        /// Max-width at LG breakpoint
        pub const LG: &str = "1024px";
        /// Max-width at XL breakpoint
        pub const XL: &str = "1280px";
        /// Max-width at XXL breakpoint
        pub const XXL: &str = "1536px";
    }

    /// Generate a min-width media query for a breakpoint.
    #[must_use]
    pub fn media_min(breakpoint: &str) -> String {
        format!("@media (min-width: {})", breakpoint)
    }

    /// Generate a max-width media query for a breakpoint.
    #[must_use]
    pub fn media_max(breakpoint: &str) -> String {
        format!("@media (max-width: {})", breakpoint)
    }

    /// Check if a width (in pixels) is at or above a breakpoint.
    #[must_use]
    pub fn is_above(width: u32, breakpoint_px: u32) -> bool {
        width >= breakpoint_px
    }

    /// Check if a width (in pixels) is below a breakpoint.
    #[must_use]
    pub fn is_below(width: u32, breakpoint_px: u32) -> bool {
        width < breakpoint_px
    }
}

/// Motion preference utilities for accessibility.
///
/// These utilities help respect the user's `prefers-reduced-motion` setting.
/// Use `motion_safe_class()` to conditionally apply animation classes,
/// and `transition_safe()` to conditionally apply transitions.
///
/// # Example
/// ```rust
/// use communitas_dioxus::tokens::motion;
///
/// // Returns "animate-pulse" if motion allowed, "" otherwise
/// let class = motion::animation_class("animate-pulse");
///
/// // Returns full transition CSS or "transition: none" if reduced motion
/// let style = motion::transition_safe(transition::DEFAULT);
/// ```
pub mod motion {
    use super::transition;

    /// CSS media query for prefers-reduced-motion.
    pub const MEDIA_REDUCE_MOTION: &str = "@media (prefers-reduced-motion: reduce)";

    /// CSS custom property for transition duration (use in stylesheets).
    pub const CSS_VAR_TRANSITION_DURATION: &str = "--transition-duration";

    /// CSS custom property value for normal motion.
    pub const CSS_VAR_DURATION_NORMAL: &str = "200ms";

    /// CSS custom property value for reduced motion.
    pub const CSS_VAR_DURATION_REDUCED: &str = "0ms";

    /// Returns a CSS transition property that respects reduce-motion.
    ///
    /// This generates CSS using a CSS custom property that can be overridden
    /// via media query for users who prefer reduced motion.
    ///
    /// # Arguments
    /// * `property` - The CSS property to transition (e.g., "all", "opacity", "transform")
    /// * `duration` - The transition duration from `transition` module (e.g., `transition::DEFAULT`)
    ///
    /// # Returns
    /// CSS string with transition using custom property.
    #[must_use]
    pub fn transition_safe(property: &str, duration: &str) -> String {
        format!(
            "transition: {} {}; transition: {} var({}, {});",
            property, duration, property, CSS_VAR_TRANSITION_DURATION, duration
        )
    }

    /// Returns a CSS transition for 'all' properties that respects reduce-motion.
    ///
    /// Convenience wrapper around `transition_safe("all", duration)`.
    #[must_use]
    pub fn transition_all_safe(duration: &str) -> String {
        transition_safe("all", duration)
    }

    /// Returns the default transition with reduce-motion support.
    #[must_use]
    pub fn transition_default_safe() -> String {
        transition_all_safe(transition::DEFAULT)
    }

    /// Returns CSS animation property that respects reduce-motion.
    ///
    /// Uses CSS custom property for animation duration that can be set to 0
    /// when user prefers reduced motion.
    ///
    /// # Arguments
    /// * `animation_name` - Name of the keyframe animation
    /// * `duration` - Animation duration (e.g., "1s", "200ms")
    /// * `timing` - Timing function (e.g., "ease-in-out", "linear")
    /// * `iteration` - Iteration count (e.g., "infinite", "1")
    #[must_use]
    pub fn animation_safe(
        animation_name: &str,
        duration: &str,
        timing: &str,
        iteration: &str,
    ) -> String {
        format!(
            "animation: {} {} {} {}; animation: {} var(--animation-duration, {}) {} {};",
            animation_name,
            duration,
            timing,
            iteration,
            animation_name,
            duration,
            timing,
            iteration
        )
    }

    /// Returns the Tailwind class for animation, or empty string if motion should be reduced.
    ///
    /// This is useful for conditionally applying Tailwind animation classes.
    /// The actual motion preference detection happens via CSS media queries,
    /// so this function returns the class and CSS handles the reduction.
    ///
    /// For Tailwind, use `motion-safe:` and `motion-reduce:` prefixes instead:
    /// - `motion-safe:animate-pulse` - Only animate when motion is allowed
    /// - `motion-reduce:animate-none` - Disable animation when motion is reduced
    ///
    /// # Arguments
    /// * `normal_class` - The Tailwind class to apply when motion is allowed
    ///
    /// # Returns
    /// CSS class string with motion-safe prefix.
    #[must_use]
    pub fn animation_class(normal_class: &str) -> String {
        format!("motion-safe:{}", normal_class)
    }

    /// Returns CSS classes for both motion states.
    ///
    /// # Arguments
    /// * `normal_class` - Class when motion is allowed
    /// * `reduced_class` - Class when motion is reduced (can be empty)
    ///
    /// # Returns
    /// Combined CSS class string with appropriate motion prefixes.
    #[must_use]
    pub fn motion_class(normal_class: &str, reduced_class: &str) -> String {
        if reduced_class.is_empty() {
            format!("motion-safe:{}", normal_class)
        } else {
            format!(
                "motion-safe:{} motion-reduce:{}",
                normal_class, reduced_class
            )
        }
    }

    /// Generates CSS custom property declarations for motion preferences.
    ///
    /// Include this in your root stylesheet to enable motion-safe transitions
    /// throughout the application.
    ///
    /// # Returns
    /// CSS string with custom property declarations and media query.
    #[must_use]
    pub fn css_custom_properties() -> String {
        format!(
            r#":root {{
  {}: {};
  --animation-duration: 1s;
}}

{} {{
  :root {{
    {}: {};
    --animation-duration: 0.01ms;
  }}
}}"#,
            CSS_VAR_TRANSITION_DURATION,
            CSS_VAR_DURATION_NORMAL,
            MEDIA_REDUCE_MOTION,
            CSS_VAR_TRANSITION_DURATION,
            CSS_VAR_DURATION_REDUCED
        )
    }

    /// Check if a given Tailwind class is an animation class.
    #[must_use]
    pub fn is_animation_class(class: &str) -> bool {
        class.starts_with("animate-") || class.contains("transition")
    }

    /// Wrap animation classes with motion-safe prefix.
    ///
    /// Scans a class string and wraps any animation-related classes
    /// with the `motion-safe:` Tailwind prefix.
    #[must_use]
    pub fn wrap_animation_classes(classes: &str) -> String {
        classes
            .split_whitespace()
            .map(|class| {
                if is_animation_class(class) {
                    format!("motion-safe:{}", class)
                } else {
                    class.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
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
        assert!(colors::INFO.starts_with('#'));
        assert!(colors::ERROR.starts_with('#'));
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

    // Motion module tests
    #[test]
    fn motion_transition_safe_generates_css() {
        let css = motion::transition_safe("opacity", transition::DEFAULT);
        assert!(css.contains("transition: opacity 200ms ease-in-out"));
        assert!(css.contains("var(--transition-duration"));
    }

    #[test]
    fn motion_transition_all_safe_generates_css() {
        let css = motion::transition_all_safe(transition::FAST);
        assert!(css.contains("transition: all 150ms ease-in-out"));
    }

    #[test]
    fn motion_transition_default_safe_uses_default_duration() {
        let css = motion::transition_default_safe();
        assert!(css.contains("200ms"));
    }

    #[test]
    fn motion_animation_safe_generates_css() {
        let css = motion::animation_safe("pulse", "2s", "ease-in-out", "infinite");
        assert!(css.contains("animation: pulse 2s ease-in-out infinite"));
        assert!(css.contains("--animation-duration"));
    }

    #[test]
    fn motion_animation_class_adds_prefix() {
        let class = motion::animation_class("animate-pulse");
        assert_eq!(class, "motion-safe:animate-pulse");
    }

    #[test]
    fn motion_class_with_both_states() {
        let class = motion::motion_class("animate-pulse", "opacity-100");
        assert_eq!(class, "motion-safe:animate-pulse motion-reduce:opacity-100");
    }

    #[test]
    fn motion_class_with_empty_reduced() {
        let class = motion::motion_class("animate-spin", "");
        assert_eq!(class, "motion-safe:animate-spin");
    }

    #[test]
    fn motion_css_custom_properties_includes_media_query() {
        let css = motion::css_custom_properties();
        assert!(css.contains(":root"));
        assert!(css.contains("--transition-duration"));
        assert!(css.contains("@media (prefers-reduced-motion: reduce)"));
    }

    #[test]
    fn motion_is_animation_class_detects_animate() {
        assert!(motion::is_animation_class("animate-pulse"));
        assert!(motion::is_animation_class("animate-spin"));
        assert!(motion::is_animation_class("transition"));
        assert!(motion::is_animation_class("transition-colors"));
        assert!(!motion::is_animation_class("flex"));
        assert!(!motion::is_animation_class("p-4"));
    }

    #[test]
    fn motion_wrap_animation_classes_wraps_correctly() {
        let input = "flex animate-pulse p-4 transition-colors bg-slate-800";
        let output = motion::wrap_animation_classes(input);
        assert!(output.contains("motion-safe:animate-pulse"));
        assert!(output.contains("motion-safe:transition-colors"));
        assert!(output.contains("flex"));
        assert!(output.contains("p-4"));
        assert!(output.contains("bg-slate-800"));
        // Non-animation classes should NOT have prefix
        assert!(!output.contains("motion-safe:flex"));
        assert!(!output.contains("motion-safe:p-4"));
    }

    // Breakpoint tests
    #[test]
    fn breakpoints_are_valid_px_values() {
        assert!(breakpoints::SM.ends_with("px"));
        assert!(breakpoints::MD.ends_with("px"));
        assert!(breakpoints::LG.ends_with("px"));
        assert!(breakpoints::XL.ends_with("px"));
        assert!(breakpoints::XXL.ends_with("px"));
    }

    #[test]
    fn breakpoint_px_values_match() {
        assert_eq!(breakpoints::SM, format!("{}px", breakpoints::SM_PX));
        assert_eq!(breakpoints::MD, format!("{}px", breakpoints::MD_PX));
        assert_eq!(breakpoints::LG, format!("{}px", breakpoints::LG_PX));
        assert_eq!(breakpoints::XL, format!("{}px", breakpoints::XL_PX));
        assert_eq!(breakpoints::XXL, format!("{}px", breakpoints::XXL_PX));
    }

    #[test]
    fn breakpoint_media_query_generates_correctly() {
        let media = breakpoints::media_min(breakpoints::MD);
        assert_eq!(media, "@media (min-width: 768px)");

        let media_max = breakpoints::media_max(breakpoints::SM);
        assert_eq!(media_max, "@media (max-width: 640px)");
    }

    #[test]
    fn breakpoint_is_above_works() {
        assert!(breakpoints::is_above(800, breakpoints::SM_PX));
        assert!(breakpoints::is_above(768, breakpoints::MD_PX));
        assert!(!breakpoints::is_above(600, breakpoints::MD_PX));
    }

    #[test]
    fn breakpoint_is_below_works() {
        assert!(breakpoints::is_below(600, breakpoints::MD_PX));
        assert!(!breakpoints::is_below(800, breakpoints::MD_PX));
    }

    #[test]
    fn container_max_widths_are_valid() {
        assert!(breakpoints::container::SM.ends_with("px"));
        assert!(breakpoints::container::MD.ends_with("px"));
        assert!(breakpoints::container::LG.ends_with("px"));
        assert!(breakpoints::container::XL.ends_with("px"));
        assert!(breakpoints::container::XXL.ends_with("px"));
    }
}
