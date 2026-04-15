// SPDX-License-Identifier: MIT OR Apache-2.0

//! Platform-specific secure storage implementation
//!
//! Handles OS-specific storage paths and integrates with platform security features:
//! - macOS: Keychain Services
//! - Windows: DPAPI (Data Protection API)
//! - Linux: Secret Service API
//!
//! SECURITY: Vault authentication uses four-word address + password for key
//! derivation only. Password hashes are never stored or used as lookup keys.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::encrypted_storage::VaultInfo;

/// Platform-specific storage manager
pub struct PlatformStorage {
    base_path: PathBuf,
    platform_type: PlatformType,
}

/// Platform type detection for OS-specific storage backends
/// Windows and Linux variants reserved for future cross-platform implementation
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum PlatformType {
    MacOS,
    Windows,
    Linux,
}

impl PlatformStorage {
    /// Create a new platform storage manager
    pub fn new(base_path: &PathBuf) -> Result<Self> {
        let platform_type = Self::detect_platform();

        // Ensure base directory exists
        std::fs::create_dir_all(base_path)?;

        Ok(Self {
            base_path: base_path.clone(),
            platform_type,
        })
    }

    /// Check if a fully-initialized vault exists (has vault.meta, not just a directory)
    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        let vault_path = self.base_path.join(four_words);
        Ok(vault_path.join("vault.meta").exists())
    }

    /// List all available vaults
    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>> {
        let mut vaults = Vec::new();

        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();

                    // Skip system directories
                    if name_str.starts_with('.') || name_str == "locators" || name_str == "temp" {
                        continue;
                    }

                    // Try to load vault metadata
                    if let Ok(info) = self.load_vault_info(&path).await {
                        vaults.push(info);
                    }
                }
            }
        }

        Ok(vaults)
    }

    /// Get secure storage path for sensitive data
    pub fn get_secure_path(&self, four_words: &str) -> PathBuf {
        match self.platform_type {
            PlatformType::MacOS => {
                // Use Application Support for secure data
                let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                path.push("Library");
                path.push("Application Support");
                path.push("com.saorsalabs.communitas");
                path.push("secure");
                path.push(four_words);
                path
            }
            PlatformType::Windows => {
                // Use AppData\Local for secure data
                let mut path =
                    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData"));
                path.push("communitas");
                path.push("secure");
                path.push(four_words);
                path
            }
            PlatformType::Linux => {
                // Use XDG_DATA_HOME for secure data
                let mut path = dirs::data_dir().unwrap_or_else(|| {
                    let mut home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
                    home.push(".local");
                    home.push("share");
                    home
                });
                path.push("communitas");
                path.push("secure");
                path.push(four_words);
                path
            }
        }
    }

    /// Apply platform-specific file permissions for security
    pub async fn secure_file(&self, path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path).await?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600); // Owner read/write only
            fs::set_permissions(path, permissions).await?;
        }

        #[cfg(windows)]
        {
            // Windows file permissions are handled differently
            // Could use Windows ACLs here if needed
            self.windows_secure_file(path).await?;
        }

        Ok(())
    }

    // Private helper methods

    fn detect_platform() -> PlatformType {
        #[cfg(target_os = "macos")]
        return PlatformType::MacOS;

        #[cfg(target_os = "windows")]
        return PlatformType::Windows;

        #[cfg(target_os = "linux")]
        return PlatformType::Linux;

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return PlatformType::Linux; // Default to Linux for unknown platforms
    }

    async fn load_vault_info(&self, vault_path: &PathBuf) -> Result<VaultInfo> {
        let metadata_path = vault_path.join("vault.meta");
        let metadata_json = fs::read(&metadata_path).await?;
        let metadata: VaultMetadata = serde_json::from_slice(&metadata_json)?;

        let four_words = vault_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Get display name from metadata, fallback to four_words if not stored
        let display_name = if metadata.display_name.is_empty() {
            four_words.clone()
        } else {
            metadata.display_name.clone()
        };

        // Calculate vault size
        let mut size_bytes = 0u64;
        let mut entries = fs::read_dir(vault_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(meta) = entry.metadata().await {
                size_bytes += meta.len();
            }
        }

        Ok(VaultInfo {
            four_words,
            display_name,
            created_at: metadata.created_at,
            last_accessed: metadata.last_accessed,
            size_bytes,
        })
    }

    #[cfg(windows)]
    /// Windows-specific file security (reserved for future Windows DPAPI integration)
    #[allow(dead_code)]
    async fn windows_secure_file(&self, _path: &PathBuf) -> Result<()> {
        // Windows-specific file security
        // Would use SetFileSecurity API here
        Ok(())
    }

    #[cfg(not(windows))]
    #[allow(dead_code)]
    async fn windows_secure_file(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VaultMetadata {
    pub created_at: u64,
    pub last_accessed: u64,
    /// Display name stored unencrypted for vault listing
    #[serde(default)]
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_platform_storage_vault_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = PlatformStorage::new(&temp_dir.path().to_path_buf()).unwrap();

        // Non-existent vault should return false
        let exists = storage.vault_exists("no-such-vault").await.unwrap();
        assert!(!exists);
    }

    #[test]
    fn test_secure_path_generation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = PlatformStorage::new(&temp_dir.path().to_path_buf()).unwrap();

        let path = storage.get_secure_path("test-four-words");

        // Path should contain the four-words
        assert!(path.to_string_lossy().contains("test-four-words"));

        // Path should be platform-specific
        #[cfg(target_os = "macos")]
        assert!(path.to_string_lossy().contains("Application Support"));

        #[cfg(target_os = "windows")]
        assert!(path.to_string_lossy().contains("communitas"));

        #[cfg(target_os = "linux")]
        assert!(path.to_string_lossy().contains("communitas"));
    }
}
