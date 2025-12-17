// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Platform-Specific Utilities
//!
//! Contains platform-specific code for desktop, mobile, and web targets.

use std::path::PathBuf;

/// Get the platform-appropriate data directory
#[allow(dead_code)]
pub fn get_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("communitas"))
}

/// Get the platform-appropriate config directory
#[allow(dead_code)]
pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("communitas"))
}

/// Get the platform-appropriate cache directory
#[allow(dead_code)]
pub fn get_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("communitas"))
}

/// Platform identification
#[allow(dead_code, clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Linux,
    Windows,
    IOS,
    Android,
    Web,
}

impl Platform {
    /// Detect the current platform
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "ios")]
        return Platform::IOS;

        #[cfg(target_os = "android")]
        return Platform::Android;

        #[cfg(target_arch = "wasm32")]
        return Platform::Web;

        #[cfg(not(any(
            target_os = "macos",
            target_os = "linux",
            target_os = "windows",
            target_os = "ios",
            target_os = "android",
            target_arch = "wasm32"
        )))]
        return Platform::Linux; // Default fallback
    }

    /// Check if this is a desktop platform
    #[allow(dead_code)]
    pub fn is_desktop(&self) -> bool {
        matches!(self, Platform::MacOS | Platform::Linux | Platform::Windows)
    }

    /// Check if this is a mobile platform
    #[allow(dead_code)]
    pub fn is_mobile(&self) -> bool {
        matches!(self, Platform::IOS | Platform::Android)
    }

    /// Check if this is a web platform
    #[allow(dead_code)]
    pub fn is_web(&self) -> bool {
        matches!(self, Platform::Web)
    }
}

/// Get platform display name
#[allow(dead_code)]
pub fn platform_name() -> &'static str {
    match Platform::current() {
        Platform::MacOS => "macOS",
        Platform::Linux => "Linux",
        Platform::Windows => "Windows",
        Platform::IOS => "iOS",
        Platform::Android => "Android",
        Platform::Web => "Web",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();
        assert!(platform.is_desktop() || platform.is_mobile() || platform.is_web());
    }

    #[test]
    fn test_data_dir() {
        // Should return Some on most platforms
        if Platform::current().is_desktop() {
            assert!(get_data_dir().is_some());
        }
    }
}
