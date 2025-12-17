// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Theme System
//!
//! Defines color schemes, typography, and visual constants for the application.

/// Application theme configuration
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    /// Primary brand color
    pub primary: &'static str,
    /// Secondary accent color
    pub secondary: &'static str,
    /// Background color
    pub background: &'static str,
    /// Surface color (cards, dialogs)
    pub surface: &'static str,
    /// Text primary color
    pub text_primary: &'static str,
    /// Text secondary color
    pub text_secondary: &'static str,
    /// Error color
    pub error: &'static str,
    /// Success color
    pub success: &'static str,
    /// Border color
    pub border: &'static str,
}

impl Theme {
    /// Light theme colors
    pub const LIGHT: Theme = Theme {
        primary: "#007AFF",
        secondary: "#5856D6",
        background: "#FFFFFF",
        surface: "#F2F2F7",
        text_primary: "#000000",
        text_secondary: "#8E8E93",
        error: "#FF3B30",
        success: "#34C759",
        border: "#E5E5EA",
    };

    /// Dark theme colors
    #[allow(dead_code)]
    pub const DARK: Theme = Theme {
        primary: "#0A84FF",
        secondary: "#5E5CE6",
        background: "#000000",
        surface: "#1C1C1E",
        text_primary: "#FFFFFF",
        text_secondary: "#8E8E93",
        error: "#FF453A",
        success: "#30D158",
        border: "#38383A",
    };
}

impl Default for Theme {
    fn default() -> Self {
        Theme::LIGHT
    }
}
