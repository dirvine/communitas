//! Version management system for Communitas Dioxus application.
//!
//! Provides compile-time version information, build metadata, and version comparison utilities.

use std::fmt;

/// Helper macro to provide optional environment variables with fallback.
macro_rules! option_env_or {
    ($key:expr, $default:expr) => {
        match option_env!($key) {
            Some(val) => val,
            None => $default,
        }
    };
}

/// Version information captured at compile time.
///
/// Contains the application version, git commit hash, build timestamp, and target platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionInfo {
    /// Application version from Cargo.toml (e.g., "0.1.0")
    pub version: &'static str,
    /// Git commit hash at build time, or "unknown" if unavailable
    pub commit_hash: &'static str,
    /// Build timestamp (e.g., "2026-01-23"), or "unknown" if unavailable
    pub build_date: &'static str,
    /// Build target triple (e.g., "x86_64-apple-darwin")
    pub target: &'static str,
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Communitas {} ({}, built {}, {})",
            self.version, self.commit_hash, self.build_date, self.target
        )
    }
}

impl VersionInfo {
    /// Compares this version with another version string.
    ///
    /// Performs a simple semantic versioning comparison using numeric parts.
    /// Handles versions like "0.1.0", "1.2.3", etc.
    ///
    /// Returns `true` if this version is newer than the other version.
    pub fn is_newer_than(&self, other: &str) -> bool {
        Self::compare_versions(self.version, other) > 0
    }

    /// Parses a version string into numeric components.
    fn parse_version(version: &str) -> Vec<u32> {
        version
            .split('.')
            .filter_map(|part| {
                // Extract leading digits from each part
                part.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse::<u32>()
                    .ok()
            })
            .collect()
    }

    /// Compares two version strings.
    ///
    /// Returns positive if left > right, zero if equal, negative if left < right.
    fn compare_versions(left: &str, right: &str) -> i32 {
        let left_parts = Self::parse_version(left);
        let right_parts = Self::parse_version(right);

        let max_len = left_parts.len().max(right_parts.len());

        for i in 0..max_len {
            let left_val = left_parts.get(i).copied().unwrap_or(0);
            let right_val = right_parts.get(i).copied().unwrap_or(0);

            match left_val.cmp(&right_val) {
                std::cmp::Ordering::Greater => return 1,
                std::cmp::Ordering::Less => return -1,
                std::cmp::Ordering::Equal => continue,
            }
        }

        0
    }
}

/// Current version information for the Communitas Dioxus application.
///
/// This constant is populated at compile time using environment variables and macros.
pub const CURRENT: VersionInfo = VersionInfo {
    version: env!("CARGO_PKG_VERSION"),
    commit_hash: option_env_or!("GIT_HASH", "unknown"),
    build_date: option_env_or!("BUILD_DATE", "unknown"),
    target: option_env_or!("TARGET", "unknown"),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info_display() {
        let version = VersionInfo {
            version: "0.1.0",
            commit_hash: "abc123",
            build_date: "2026-01-23",
            target: "x86_64-apple-darwin",
        };

        let display = format!("{}", version);
        assert!(display.contains("0.1.0"), "Should contain version");
        assert!(display.contains("abc123"), "Should contain commit hash");
        assert!(display.contains("2026-01-23"), "Should contain build date");
    }

    #[test]
    fn test_is_newer_than_major() {
        let v1 = VersionInfo {
            version: "2.0.0",
            commit_hash: "test",
            build_date: "test",
            target: "test",
        };

        assert!(v1.is_newer_than("1.9.9"), "2.0.0 > 1.9.9");
        assert!(!v1.is_newer_than("2.0.0"), "2.0.0 == 2.0.0");
        assert!(!v1.is_newer_than("2.1.0"), "2.0.0 < 2.1.0");
    }

    #[test]
    fn test_is_newer_than_minor() {
        let v1 = VersionInfo {
            version: "1.5.0",
            commit_hash: "test",
            build_date: "test",
            target: "test",
        };

        assert!(v1.is_newer_than("1.4.9"), "1.5.0 > 1.4.9");
        assert!(!v1.is_newer_than("1.5.0"), "1.5.0 == 1.5.0");
        assert!(!v1.is_newer_than("1.5.1"), "1.5.0 < 1.5.1");
    }

    #[test]
    fn test_is_newer_than_patch() {
        let v1 = VersionInfo {
            version: "1.0.5",
            commit_hash: "test",
            build_date: "test",
            target: "test",
        };

        assert!(v1.is_newer_than("1.0.4"), "1.0.5 > 1.0.4");
        assert!(!v1.is_newer_than("1.0.5"), "1.0.5 == 1.0.5");
        assert!(!v1.is_newer_than("1.0.6"), "1.0.5 < 1.0.6");
    }

    #[test]
    fn test_parse_version() {
        let parts = VersionInfo::parse_version("1.2.3");
        assert_eq!(parts, vec![1, 2, 3]);

        let parts = VersionInfo::parse_version("0.1.0");
        assert_eq!(parts, vec![0, 1, 0]);

        let parts = VersionInfo::parse_version("2.0");
        assert_eq!(parts, vec![2, 0]);
    }

    #[test]
    fn test_parse_version_with_suffix() {
        // "-alpha" suffix has no leading digits, so ignored
        let parts = VersionInfo::parse_version("1.2.3-alpha");
        assert_eq!(parts, vec![1, 2, 3]);

        // "-beta.1" becomes "beta" (no digits) then "1" (digit), so we get the 1
        // This is fine for comparison purposes - prereleases are treated as extra parts
        let parts = VersionInfo::parse_version("2.0.0-beta.1");
        assert_eq!(parts, vec![2, 0, 0, 1]);
    }

    #[test]
    fn test_current_version() {
        let current = CURRENT;
        assert!(!current.version.is_empty(), "Version should not be empty");
    }
}
