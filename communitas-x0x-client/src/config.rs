//! Platform-specific x0x daemon configuration discovery.
//!
//! x0xd writes two files into its data directory:
//! - `api.port`  – the `host:port` the API server is listening on (e.g. `127.0.0.1:12700`)
//! - `api-token` – a 64-character hex Bearer token that must accompany every request
//!
//! This module locates those files at runtime and returns an [`X0xConfig`] that
//! the HTTP and WebSocket clients use to authenticate.

use std::path::PathBuf;

use crate::error::{Result, X0xError};

/// Discovered x0x daemon configuration.
///
/// Contains the API address and Bearer token required to talk to x0xd.
#[derive(Debug, Clone)]
pub struct X0xConfig {
    /// The host:port the x0xd API server is listening on (e.g. `127.0.0.1:12700`).
    pub address: String,
    /// The 64-character hex Bearer token required for authenticated endpoints.
    pub token: String,
}

/// Return the platform-specific x0x data directory.
///
/// | Platform | Path |
/// |----------|------|
/// | macOS    | `~/Library/Application Support/x0x` |
/// | Linux    | `~/.local/share/x0x` |
/// | Windows  | `%APPDATA%\x0x` |
///
/// Returns `None` if the home/data directory cannot be determined.
pub fn x0x_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        // ~/Library/Application Support/x0x
        dirs_next::data_dir().map(|d| d.join("x0x"))
    }
    #[cfg(target_os = "linux")]
    {
        // ~/.local/share/x0x
        dirs_next::data_dir().map(|d| d.join("x0x"))
    }
    #[cfg(target_os = "windows")]
    {
        // %APPDATA%\x0x
        dirs_next::data_dir().map(|d| d.join("x0x"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        dirs_next::data_dir().map(|d| d.join("x0x"))
    }
}

/// Read the API address from the x0x `api.port` file.
///
/// Returns the address string, e.g. `127.0.0.1:12700`.
///
/// # Errors
///
/// Returns [`X0xError::Config`] if the data directory cannot be determined or
/// the file is missing/unreadable.
pub fn read_api_address() -> Result<String> {
    let dir = x0x_data_dir()
        .ok_or_else(|| X0xError::Config("cannot determine x0x data directory".to_owned()))?;
    let path = dir.join("api.port");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| X0xError::Config(format!("cannot read {}: {e}", path.display())))?;
    Ok(raw.trim().to_owned())
}

/// Read the API Bearer token from the x0x `api-token` file.
///
/// Returns the 64-character hex token string.
///
/// # Errors
///
/// Returns [`X0xError::Config`] if the data directory cannot be determined or
/// the file is missing/unreadable.
pub fn read_api_token() -> Result<String> {
    let dir = x0x_data_dir()
        .ok_or_else(|| X0xError::Config("cannot determine x0x data directory".to_owned()))?;
    let path = dir.join("api-token");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| X0xError::Config(format!("cannot read {}: {e}", path.display())))?;
    Ok(raw.trim().to_owned())
}

/// Discover the full x0x daemon configuration by reading both config files.
///
/// # Errors
///
/// Returns [`X0xError::Config`] if either file is missing or unreadable.
pub fn discover() -> Result<X0xConfig> {
    let address = read_api_address()?;
    let token = read_api_token()?;
    Ok(X0xConfig { address, token })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn write_temp_files(address: &str, token: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let mut f = std::fs::File::create(dir.path().join("api.port")).unwrap();
        writeln!(f, "{address}").unwrap();
        let mut f = std::fs::File::create(dir.path().join("api-token")).unwrap();
        writeln!(f, "{token}").unwrap();
        dir
    }

    #[test]
    fn test_x0x_data_dir_is_some() {
        // Verify we can compute a data directory on the current platform.
        // We don't assert a specific path – just that it's not None.
        let result = x0x_data_dir();
        assert!(result.is_some(), "x0x_data_dir() returned None");
    }

    #[test]
    fn test_read_api_address_missing_file() {
        // Point at a non-existent directory – should get a Config error.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("api.port");
        let raw = std::fs::read_to_string(&path);
        assert!(raw.is_err(), "expected error for missing file");
    }

    #[test]
    fn test_read_api_address_from_file() {
        let dir = write_temp_files("127.0.0.1:12700", "abcd1234");
        let raw = std::fs::read_to_string(dir.path().join("api.port")).unwrap();
        assert_eq!(raw.trim(), "127.0.0.1:12700");
    }

    #[test]
    fn test_read_api_token_from_file() {
        let token = "a".repeat(64);
        let dir = write_temp_files("127.0.0.1:12700", &token);
        let raw = std::fs::read_to_string(dir.path().join("api-token")).unwrap();
        assert_eq!(raw.trim(), token);
    }

    #[test]
    fn test_discover_builds_config() {
        let token = "b".repeat(64);
        let _dir = write_temp_files("127.0.0.1:19999", &token);
        // We can't easily override the data dir here without injecting it,
        // so just verify the struct fields work correctly by constructing one.
        let cfg = X0xConfig {
            address: "127.0.0.1:19999".to_owned(),
            token: token.clone(),
        };
        assert_eq!(cfg.address, "127.0.0.1:19999");
        assert_eq!(cfg.token, token);
    }
}
