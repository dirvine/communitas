// SPDX-License-Identifier: MIT OR Apache-2.0

//! Enhanced design tokens for the "Digital Forest Sanctuary" theme.
//!
//! A warm, organic aesthetic with bioluminescent jade accents.

/// Core color palette - forest-inspired with warm undertones.
pub mod palette {
    // Primary jade/emerald spectrum
    pub const JADE_50: &str = "#ecfdf5";
    pub const JADE_100: &str = "#d1fae5";
    pub const JADE_200: &str = "#a7f3d0";
    pub const JADE_300: &str = "#6ee7b7";
    pub const JADE_400: &str = "#34d399";
    pub const JADE_500: &str = "#10b981";
    pub const JADE_600: &str = "#059669";
    pub const JADE_700: &str = "#047857";
    pub const JADE_800: &str = "#065f46";
    pub const JADE_900: &str = "#064e3b";

    // Deep forest backgrounds (warmer than pure slate)
    pub const FOREST_950: &str = "#0a0f14";
    pub const FOREST_900: &str = "#0f1720";
    pub const FOREST_850: &str = "#141d28";
    pub const FOREST_800: &str = "#1a2634";
    pub const FOREST_700: &str = "#243344";
    pub const FOREST_600: &str = "#334155";
    pub const FOREST_500: &str = "#475569";
    pub const FOREST_400: &str = "#64748b";
    pub const FOREST_300: &str = "#94a3b8";
    pub const FOREST_200: &str = "#cbd5e1";
    pub const FOREST_100: &str = "#e2e8f0";
    pub const FOREST_50: &str = "#f1f5f9";

    // Accent colors for entities
    pub const AMBER_500: &str = "#f59e0b";
    pub const AMBER_400: &str = "#fbbf24";
    pub const VIOLET_500: &str = "#8b5cf6";
    pub const VIOLET_400: &str = "#a78bfa";
    pub const CORAL_500: &str = "#f97316";
    pub const CORAL_400: &str = "#fb923c";
    pub const ROSE_500: &str = "#f43f5e";
    pub const ROSE_400: &str = "#fb7185";
    pub const SKY_500: &str = "#0ea5e9";
    pub const SKY_400: &str = "#38bdf8";

    // Semantic status colors
    pub const SUCCESS: &str = "#22c55e";
    pub const WARNING: &str = "#eab308";
    pub const ERROR: &str = "#ef4444";
    pub const INFO: &str = "#3b82f6";
}

/// Semantic colors for UI elements.
pub mod semantic {
    use super::palette;

    // Surfaces
    pub const BG_BASE: &str = palette::FOREST_950;
    pub const BG_PRIMARY: &str = palette::FOREST_900;
    pub const BG_SECONDARY: &str = palette::FOREST_850;
    pub const BG_TERTIARY: &str = palette::FOREST_800;
    pub const BG_ELEVATED: &str = palette::FOREST_700;
    pub const BG_HOVER: &str = palette::FOREST_700;

    // Glass effect backgrounds
    pub const GLASS_BG: &str = "rgba(20, 29, 40, 0.7)";
    pub const GLASS_BORDER: &str = "rgba(52, 211, 153, 0.15)";
    pub const GLASS_GLOW: &str = "rgba(16, 185, 129, 0.1)";

    // Text
    pub const TEXT_PRIMARY: &str = palette::FOREST_50;
    pub const TEXT_SECONDARY: &str = palette::FOREST_300;
    pub const TEXT_MUTED: &str = palette::FOREST_400;
    pub const TEXT_INVERSE: &str = palette::FOREST_900;

    // Borders
    pub const BORDER_SUBTLE: &str = "rgba(52, 211, 153, 0.08)";
    pub const BORDER_DEFAULT: &str = "rgba(52, 211, 153, 0.15)";
    pub const BORDER_STRONG: &str = "rgba(52, 211, 153, 0.25)";
    pub const BORDER_FOCUS: &str = palette::JADE_500;

    // Interactive
    pub const PRIMARY: &str = palette::JADE_500;
    pub const PRIMARY_HOVER: &str = palette::JADE_400;
    pub const PRIMARY_ACTIVE: &str = palette::JADE_600;
    pub const ACCENT: &str = palette::JADE_400;

    // Entity type colors
    pub const ENTITY_ORG: &str = palette::JADE_500;
    pub const ENTITY_PROJECT: &str = palette::AMBER_500;
    pub const ENTITY_CHANNEL: &str = palette::SKY_500;
    pub const ENTITY_GROUP: &str = palette::VIOLET_500;
    pub const ENTITY_PERSON: &str = palette::CORAL_500;

    // Presence indicators
    pub const PRESENCE_ONLINE: &str = palette::SUCCESS;
    pub const PRESENCE_AWAY: &str = palette::WARNING;
    pub const PRESENCE_BUSY: &str = palette::ERROR;
    pub const PRESENCE_OFFLINE: &str = palette::FOREST_500;

    // Status colors (re-exported from palette for convenience)
    pub const SUCCESS: &str = palette::SUCCESS;
    pub const WARNING: &str = palette::WARNING;
    pub const ERROR: &str = palette::ERROR;
    pub const INFO: &str = palette::INFO;
}

/// Gradients for backgrounds and accents.
pub mod gradients {
    /// Subtle mesh gradient for auth background
    pub const AUTH_BG: &str = "radial-gradient(ellipse at 20% 80%, rgba(16, 185, 129, 0.08) 0%, transparent 50%), \
                               radial-gradient(ellipse at 80% 20%, rgba(6, 95, 70, 0.12) 0%, transparent 50%), \
                               radial-gradient(ellipse at 50% 50%, rgba(20, 29, 40, 1) 0%, rgba(10, 15, 20, 1) 100%)";

    /// Glass card gradient overlay
    pub const GLASS_OVERLAY: &str =
        "linear-gradient(135deg, rgba(255,255,255,0.03) 0%, rgba(255,255,255,0) 100%)";

    /// Primary button gradient
    pub const BUTTON_PRIMARY: &str = "linear-gradient(135deg, #10b981 0%, #059669 100%)";

    /// Glow effect for focused elements
    pub const GLOW_JADE: &str =
        "0 0 20px rgba(16, 185, 129, 0.3), 0 0 40px rgba(16, 185, 129, 0.1)";

    /// Sidebar gradient
    pub const SIDEBAR_BG: &str =
        "linear-gradient(180deg, rgba(15, 23, 32, 0.98) 0%, rgba(10, 15, 20, 0.99) 100%)";
}

/// Spacing scale (rem-based).
pub mod spacing {
    pub const NONE: &str = "0";
    pub const PX: &str = "1px";
    pub const XXS: &str = "0.125rem"; // 2px
    pub const XS: &str = "0.25rem"; // 4px
    pub const SM: &str = "0.5rem"; // 8px
    pub const MD: &str = "0.75rem"; // 12px
    pub const BASE: &str = "1rem"; // 16px
    pub const LG: &str = "1.25rem"; // 20px
    pub const XL: &str = "1.5rem"; // 24px
    pub const XXL: &str = "2rem"; // 32px
    pub const XXXL: &str = "3rem"; // 48px
    pub const HUGE: &str = "4rem"; // 64px
}

/// Border radius values.
pub mod radius {
    pub const NONE: &str = "0";
    pub const SM: &str = "0.375rem"; // 6px
    pub const MD: &str = "0.5rem"; // 8px
    pub const LG: &str = "0.75rem"; // 12px
    pub const XL: &str = "1rem"; // 16px
    pub const XXL: &str = "1.5rem"; // 24px
    pub const FULL: &str = "9999px";
}

/// Typography tokens.
pub mod typography {
    // Font families
    pub const FONT_DISPLAY: &str =
        "'SF Pro Display', -apple-system, BlinkMacSystemFont, system-ui, sans-serif";
    pub const FONT_BODY: &str =
        "'SF Pro Text', -apple-system, BlinkMacSystemFont, system-ui, sans-serif";
    pub const FONT_MONO: &str = "'SF Mono', 'JetBrains Mono', 'Fira Code', ui-monospace, monospace";

    // Font sizes
    pub const SIZE_XXS: &str = "0.625rem"; // 10px
    pub const SIZE_XS: &str = "0.75rem"; // 12px
    pub const SIZE_SM: &str = "0.8125rem"; // 13px
    pub const SIZE_BASE: &str = "0.875rem"; // 14px
    pub const SIZE_MD: &str = "1rem"; // 16px
    pub const SIZE_LG: &str = "1.125rem"; // 18px
    pub const SIZE_XL: &str = "1.25rem"; // 20px
    pub const SIZE_2XL: &str = "1.5rem"; // 24px
    pub const SIZE_3XL: &str = "2rem"; // 32px
    pub const SIZE_4XL: &str = "2.5rem"; // 40px

    // Font weights
    pub const WEIGHT_NORMAL: &str = "400";
    pub const WEIGHT_MEDIUM: &str = "500";
    pub const WEIGHT_SEMIBOLD: &str = "600";
    pub const WEIGHT_BOLD: &str = "700";

    // Line heights
    pub const LEADING_TIGHT: &str = "1.25";
    pub const LEADING_NORMAL: &str = "1.5";
    pub const LEADING_RELAXED: &str = "1.625";

    // Letter spacing
    pub const TRACKING_TIGHT: &str = "-0.025em";
    pub const TRACKING_NORMAL: &str = "0";
    pub const TRACKING_WIDE: &str = "0.05em";
    pub const TRACKING_WIDER: &str = "0.1em";
}

/// Animation and transition tokens.
pub mod motion {
    // Durations
    pub const INSTANT: &str = "0ms";
    pub const FAST: &str = "100ms";
    pub const NORMAL: &str = "200ms";
    pub const SLOW: &str = "300ms";
    pub const SLOWER: &str = "500ms";

    // Easings
    pub const EASE_DEFAULT: &str = "cubic-bezier(0.4, 0, 0.2, 1)";
    pub const EASE_IN: &str = "cubic-bezier(0.4, 0, 1, 1)";
    pub const EASE_OUT: &str = "cubic-bezier(0, 0, 0.2, 1)";
    pub const EASE_IN_OUT: &str = "cubic-bezier(0.4, 0, 0.2, 1)";
    pub const EASE_BOUNCE: &str = "cubic-bezier(0.34, 1.56, 0.64, 1)";
    pub const EASE_SMOOTH: &str = "cubic-bezier(0.25, 0.1, 0.25, 1)";

    /// Standard transition for interactive elements
    pub fn transition(properties: &str) -> String {
        format!("{} {} {}", properties, NORMAL, EASE_DEFAULT)
    }

    /// Transition with custom duration
    pub fn transition_with(properties: &str, duration: &str, easing: &str) -> String {
        format!("{} {} {}", properties, duration, easing)
    }
}

/// Shadow tokens.
pub mod shadow {
    pub const NONE: &str = "none";
    pub const SM: &str = "0 1px 2px rgba(0, 0, 0, 0.2)";
    pub const MD: &str = "0 4px 6px -1px rgba(0, 0, 0, 0.2), 0 2px 4px -2px rgba(0, 0, 0, 0.1)";
    pub const LG: &str = "0 10px 15px -3px rgba(0, 0, 0, 0.25), 0 4px 6px -4px rgba(0, 0, 0, 0.1)";
    pub const XL: &str = "0 20px 25px -5px rgba(0, 0, 0, 0.3), 0 8px 10px -6px rgba(0, 0, 0, 0.15)";

    // Glow shadows for emphasis
    pub const GLOW_SM: &str = "0 0 10px rgba(16, 185, 129, 0.15)";
    pub const GLOW_MD: &str = "0 0 20px rgba(16, 185, 129, 0.2), 0 0 40px rgba(16, 185, 129, 0.05)";
    pub const GLOW_LG: &str = "0 0 30px rgba(16, 185, 129, 0.25), 0 0 60px rgba(16, 185, 129, 0.1)";

    // Inset shadows for depth
    pub const INSET_SM: &str = "inset 0 1px 2px rgba(0, 0, 0, 0.2)";
    pub const INSET_MD: &str = "inset 0 2px 4px rgba(0, 0, 0, 0.25)";
}

/// Z-index scale.
pub mod z_index {
    pub const BASE: &str = "0";
    pub const DROPDOWN: &str = "100";
    pub const STICKY: &str = "200";
    pub const FIXED: &str = "300";
    pub const MODAL_BACKDROP: &str = "400";
    pub const MODAL: &str = "500";
    pub const POPOVER: &str = "600";
    pub const TOOLTIP: &str = "700";
    pub const TOAST: &str = "800";
}

/// Layout constants.
pub mod layout {
    pub const SIDEBAR_WIDTH: &str = "280px";
    pub const SIDEBAR_COLLAPSED: &str = "72px";
    pub const THREAD_PANEL_WIDTH: &str = "360px";
    pub const HEADER_HEIGHT: &str = "56px";
    pub const TAB_BAR_HEIGHT: &str = "48px";
    pub const COMPOSER_MIN_HEIGHT: &str = "56px";
    pub const COMPOSER_MAX_HEIGHT: &str = "200px";
}
