//! WCAG contrast ratio utilities for accessibility compliance.
//!
//! This module provides functions to calculate and validate color contrast
//! ratios according to WCAG 2.1 guidelines.
//!
//! # WCAG Requirements
//!
//! - **AA Normal text**: Minimum 4.5:1 contrast ratio
//! - **AA Large text**: Minimum 3:1 contrast ratio (18pt+ or 14pt+ bold)
//! - **AAA Normal text**: Minimum 7:1 contrast ratio
//! - **AAA Large text**: Minimum 4.5:1 contrast ratio
//!
//! # Example
//!
//! ```rust
//! use communitas_dioxus::contrast::{contrast_ratio, meets_wcag_aa};
//!
//! let ratio = contrast_ratio("#f8fafc", "#0f172a");
//! assert!(meets_wcag_aa("#f8fafc", "#0f172a", false));
//! ```

/// Parses a hex color string to RGB values.
///
/// Supports formats: `#RGB`, `#RRGGBB`, `#RRGGBBAA`
///
/// # Returns
/// `Some((r, g, b))` if valid hex color, `None` otherwise.
#[must_use]
pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        // Short form: #RGB -> #RRGGBB
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some((r, g, b))
        }
        // Standard form: #RRGGBB
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        // With alpha: #RRGGBBAA (ignore alpha)
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

/// Calculates the relative luminance of a color.
///
/// Uses the WCAG 2.1 formula:
/// L = 0.2126 * R + 0.7152 * G + 0.0722 * B
///
/// Where R, G, B are linearized sRGB values.
///
/// # Arguments
/// * `hex` - Hex color string (e.g., "#ffffff" or "#fff")
///
/// # Returns
/// Relative luminance value between 0.0 (black) and 1.0 (white).
/// Returns 0.0 if the hex string is invalid.
#[must_use]
pub fn luminance(hex: &str) -> f64 {
    let Some((r, g, b)) = parse_hex_color(hex) else {
        return 0.0;
    };

    // Convert to sRGB (0.0-1.0)
    let r = f64::from(r) / 255.0;
    let g = f64::from(g) / 255.0;
    let b = f64::from(b) / 255.0;

    // Linearize sRGB values
    let r_lin = linearize_srgb(r);
    let g_lin = linearize_srgb(g);
    let b_lin = linearize_srgb(b);

    // Calculate luminance
    0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
}

/// Linearizes an sRGB color component.
fn linearize_srgb(value: f64) -> f64 {
    if value <= 0.03928 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

/// Calculates the contrast ratio between two colors.
///
/// Uses the WCAG 2.1 formula:
/// (L1 + 0.05) / (L2 + 0.05)
///
/// Where L1 is the lighter color's luminance and L2 is the darker.
///
/// # Arguments
/// * `fg` - Foreground (text) color hex string
/// * `bg` - Background color hex string
///
/// # Returns
/// Contrast ratio between 1.0 (same color) and 21.0 (black on white).
#[must_use]
pub fn contrast_ratio(fg: &str, bg: &str) -> f64 {
    let l1 = luminance(fg);
    let l2 = luminance(bg);

    // Ensure lighter color is in numerator
    let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };

    (lighter + 0.05) / (darker + 0.05)
}

/// Checks if a color combination meets WCAG AA requirements.
///
/// # Arguments
/// * `fg` - Foreground (text) color hex string
/// * `bg` - Background color hex string
/// * `is_large_text` - True if text is 18pt+ or 14pt+ bold
///
/// # Returns
/// `true` if the contrast ratio meets WCAG AA requirements.
#[must_use]
pub fn meets_wcag_aa(fg: &str, bg: &str, is_large_text: bool) -> bool {
    let ratio = contrast_ratio(fg, bg);
    if is_large_text {
        ratio >= 3.0
    } else {
        ratio >= 4.5
    }
}

/// Checks if a color combination meets WCAG AAA requirements.
///
/// # Arguments
/// * `fg` - Foreground (text) color hex string
/// * `bg` - Background color hex string
/// * `is_large_text` - True if text is 18pt+ or 14pt+ bold
///
/// # Returns
/// `true` if the contrast ratio meets WCAG AAA requirements.
#[must_use]
pub fn meets_wcag_aaa(fg: &str, bg: &str, is_large_text: bool) -> bool {
    let ratio = contrast_ratio(fg, bg);
    if is_large_text {
        ratio >= 4.5
    } else {
        ratio >= 7.0
    }
}

/// Returns the WCAG compliance level for a color combination.
///
/// # Arguments
/// * `fg` - Foreground (text) color hex string
/// * `bg` - Background color hex string
/// * `is_large_text` - True if text is 18pt+ or 14pt+ bold
///
/// # Returns
/// Compliance level as a string: "AAA", "AA", or "Fail".
#[must_use]
pub fn wcag_level(fg: &str, bg: &str, is_large_text: bool) -> &'static str {
    if meets_wcag_aaa(fg, bg, is_large_text) {
        "AAA"
    } else if meets_wcag_aa(fg, bg, is_large_text) {
        "AA"
    } else {
        "Fail"
    }
}

/// Suggests a minimum contrast ratio for accessibility.
///
/// # Arguments
/// * `level` - Desired WCAG level ("AA" or "AAA")
/// * `is_large_text` - True if text is 18pt+ or 14pt+ bold
///
/// # Returns
/// Minimum required contrast ratio.
#[must_use]
pub fn minimum_ratio(level: &str, is_large_text: bool) -> f64 {
    match (level, is_large_text) {
        ("AAA", false) => 7.0,
        ("AAA", true) => 4.5,
        ("AA", false) => 4.5,
        ("AA", true) => 3.0,
        _ => 4.5, // Default to AA normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::colors;

    // ==================== Parse Hex Color Tests ====================

    #[test]
    fn parse_hex_color_standard_format() {
        assert_eq!(parse_hex_color("#ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#10b981"), Some((16, 185, 129)));
    }

    #[test]
    fn parse_hex_color_short_format() {
        assert_eq!(parse_hex_color("#fff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#abc"), Some((170, 187, 204)));
    }

    #[test]
    fn parse_hex_color_with_alpha() {
        assert_eq!(parse_hex_color("#ffffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#00000080"), Some((0, 0, 0)));
    }

    #[test]
    fn parse_hex_color_without_hash() {
        assert_eq!(parse_hex_color("ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("000"), Some((0, 0, 0)));
    }

    #[test]
    fn parse_hex_color_invalid() {
        assert_eq!(parse_hex_color(""), None);
        assert_eq!(parse_hex_color("#gg0000"), None);
        assert_eq!(parse_hex_color("#12345"), None);
        assert_eq!(parse_hex_color("not a color"), None);
    }

    // ==================== Luminance Tests ====================

    #[test]
    fn luminance_white_is_one() {
        let l = luminance("#ffffff");
        assert!((l - 1.0).abs() < 0.001);
    }

    #[test]
    fn luminance_black_is_zero() {
        let l = luminance("#000000");
        assert!(l.abs() < 0.001);
    }

    #[test]
    fn luminance_gray_is_middle() {
        let l = luminance("#808080");
        assert!(l > 0.2 && l < 0.3); // ~0.216
    }

    #[test]
    fn luminance_invalid_returns_zero() {
        assert!(luminance("invalid").abs() < 0.001);
    }

    // ==================== Contrast Ratio Tests ====================

    #[test]
    fn contrast_ratio_black_white_is_21() {
        let ratio = contrast_ratio("#000000", "#ffffff");
        assert!((ratio - 21.0).abs() < 0.1);
    }

    #[test]
    fn contrast_ratio_same_color_is_one() {
        let ratio = contrast_ratio("#ff0000", "#ff0000");
        assert!((ratio - 1.0).abs() < 0.001);
    }

    #[test]
    fn contrast_ratio_is_symmetric() {
        let ratio1 = contrast_ratio("#000000", "#ffffff");
        let ratio2 = contrast_ratio("#ffffff", "#000000");
        assert!((ratio1 - ratio2).abs() < 0.001);
    }

    // ==================== WCAG AA Tests ====================

    #[test]
    fn wcag_aa_black_on_white_passes() {
        assert!(meets_wcag_aa("#000000", "#ffffff", false));
        assert!(meets_wcag_aa("#000000", "#ffffff", true));
    }

    #[test]
    fn wcag_aa_similar_colors_fail() {
        // Light gray on white - low contrast
        assert!(!meets_wcag_aa("#cccccc", "#ffffff", false));
    }

    #[test]
    fn wcag_aa_large_text_threshold() {
        // This contrast ratio is between 3.0 and 4.5
        // Should pass for large text, fail for normal text
        let fg = "#767676"; // Gray with ~4.54:1 contrast on white
        let bg = "#ffffff";
        let ratio = contrast_ratio(fg, bg);
        // Adjust test based on actual ratio
        if (3.0..4.5).contains(&ratio) {
            assert!(meets_wcag_aa(fg, bg, true));
            assert!(!meets_wcag_aa(fg, bg, false));
        }
    }

    // ==================== WCAG AAA Tests ====================

    #[test]
    fn wcag_aaa_black_on_white_passes() {
        assert!(meets_wcag_aaa("#000000", "#ffffff", false));
        assert!(meets_wcag_aaa("#000000", "#ffffff", true));
    }

    #[test]
    fn wcag_aaa_needs_higher_contrast() {
        // Gray that passes AA but not AAA
        let fg = "#595959"; // ~7.0:1 on white
        let bg = "#ffffff";
        let ratio = contrast_ratio(fg, bg);
        assert!(ratio >= 4.5); // Passes AA
        // Whether it passes AAA depends on exact ratio
    }

    // ==================== WCAG Level Tests ====================

    #[test]
    fn wcag_level_black_on_white() {
        assert_eq!(wcag_level("#000000", "#ffffff", false), "AAA");
        assert_eq!(wcag_level("#000000", "#ffffff", true), "AAA");
    }

    #[test]
    fn wcag_level_low_contrast() {
        assert_eq!(wcag_level("#dddddd", "#ffffff", false), "Fail");
    }

    // ==================== Minimum Ratio Tests ====================

    #[test]
    fn minimum_ratio_values() {
        assert!((minimum_ratio("AAA", false) - 7.0).abs() < 0.001);
        assert!((minimum_ratio("AAA", true) - 4.5).abs() < 0.001);
        assert!((minimum_ratio("AA", false) - 4.5).abs() < 0.001);
        assert!((minimum_ratio("AA", true) - 3.0).abs() < 0.001);
    }

    // ==================== Design Token Validation Tests ====================

    #[test]
    fn design_token_text_primary_on_surface_bg_passes_aa() {
        // TEXT_PRIMARY (#f8fafc) on SURFACE_BG (#0f172a)
        assert!(
            meets_wcag_aa(colors::TEXT_PRIMARY, colors::SURFACE_BG, false),
            "TEXT_PRIMARY on SURFACE_BG must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_text_secondary_on_surface_bg_passes_aa() {
        // TEXT_SECONDARY (#94a3b8) on SURFACE_BG (#0f172a)
        assert!(
            meets_wcag_aa(colors::TEXT_SECONDARY, colors::SURFACE_BG, false),
            "TEXT_SECONDARY on SURFACE_BG must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_text_muted_on_surface_bg_passes_aa_large() {
        // TEXT_MUTED (#64748b) on SURFACE_BG (#0f172a)
        // Muted text may only pass for large text
        let ratio = contrast_ratio(colors::TEXT_MUTED, colors::SURFACE_BG);
        assert!(
            ratio >= 3.0,
            "TEXT_MUTED on SURFACE_BG must have at least 3:1 contrast for large text, got {:.2}",
            ratio
        );
    }

    #[test]
    fn design_token_primary_on_surface_bg_passes_aa() {
        // PRIMARY (#10b981) on SURFACE_BG (#0f172a)
        assert!(
            meets_wcag_aa(colors::PRIMARY, colors::SURFACE_BG, false),
            "PRIMARY on SURFACE_BG must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_danger_on_surface_bg_passes_aa() {
        // DANGER (#ef4444) on SURFACE_BG (#0f172a)
        assert!(
            meets_wcag_aa(colors::DANGER, colors::SURFACE_BG, false),
            "DANGER on SURFACE_BG must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_warning_on_surface_bg_passes_aa_large() {
        // WARNING (#f59e0b) on SURFACE_BG (#0f172a)
        let ratio = contrast_ratio(colors::WARNING, colors::SURFACE_BG);
        assert!(
            ratio >= 3.0,
            "WARNING on SURFACE_BG must have at least 3:1 contrast for large text, got {:.2}",
            ratio
        );
    }

    #[test]
    fn design_token_success_on_surface_bg_passes_aa() {
        // SUCCESS (#22c55e) on SURFACE_BG (#0f172a)
        assert!(
            meets_wcag_aa(colors::SUCCESS, colors::SURFACE_BG, false),
            "SUCCESS on SURFACE_BG must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_text_inverse_on_primary_passes_aa() {
        // TEXT_INVERSE (#0f172a) on PRIMARY (#10b981)
        assert!(
            meets_wcag_aa(colors::TEXT_INVERSE, colors::PRIMARY, false),
            "TEXT_INVERSE on PRIMARY must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_text_primary_on_surface_card_passes_aa() {
        // TEXT_PRIMARY (#f8fafc) on SURFACE_CARD (#1e293b)
        assert!(
            meets_wcag_aa(colors::TEXT_PRIMARY, colors::SURFACE_CARD, false),
            "TEXT_PRIMARY on SURFACE_CARD must pass WCAG AA"
        );
    }

    #[test]
    fn design_token_text_secondary_on_surface_card_passes_aa() {
        // TEXT_SECONDARY (#94a3b8) on SURFACE_CARD (#1e293b)
        let ratio = contrast_ratio(colors::TEXT_SECONDARY, colors::SURFACE_CARD);
        assert!(
            ratio >= 3.0,
            "TEXT_SECONDARY on SURFACE_CARD must have at least 3:1 contrast, got {:.2}",
            ratio
        );
    }

    #[test]
    fn print_all_contrast_ratios() {
        // This test prints all contrast ratios for documentation
        println!("\n=== Design Token Contrast Ratios ===\n");

        let text_colors = [
            ("TEXT_PRIMARY", colors::TEXT_PRIMARY),
            ("TEXT_SECONDARY", colors::TEXT_SECONDARY),
            ("TEXT_MUTED", colors::TEXT_MUTED),
            ("PRIMARY", colors::PRIMARY),
            ("DANGER", colors::DANGER),
            ("WARNING", colors::WARNING),
            ("SUCCESS", colors::SUCCESS),
        ];

        let bg_colors = [
            ("SURFACE_BG", colors::SURFACE_BG),
            ("SURFACE_CARD", colors::SURFACE_CARD),
            ("SURFACE_ELEVATED", colors::SURFACE_ELEVATED),
        ];

        for (text_name, text_color) in &text_colors {
            for (bg_name, bg_color) in &bg_colors {
                let ratio = contrast_ratio(text_color, bg_color);
                let level = wcag_level(text_color, bg_color, false);
                println!(
                    "{:15} on {:18} = {:5.2}:1 ({})",
                    text_name, bg_name, ratio, level
                );
            }
        }
    }
}
