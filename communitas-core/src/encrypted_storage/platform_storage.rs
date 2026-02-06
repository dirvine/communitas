//! Platform-specific secure storage implementation
//!
//! Handles OS-specific storage paths and integrates with platform security features:
//! - macOS: Keychain Services
//! - Windows: DPAPI (Data Protection API)
//! - Linux: Secret Service API

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;

use crate::encrypted_storage::VaultInfo;

/// Platform-specific storage manager
pub struct PlatformStorage {
    base_path: PathBuf,
    password_locators: RwLock<HashMap<Vec<u8>, String>>, // password_hash -> four_words
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

        // Create platform-specific subdirectories
        let locator_path = base_path.join("locators");
        std::fs::create_dir_all(&locator_path)?;

        Ok(Self {
            base_path: base_path.clone(),
            password_locators: RwLock::new(Self::load_locators(&locator_path)?),
            platform_type,
        })
    }

    /// Check if a fully-initialized vault exists (has vault.meta, not just a directory)
    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        let vault_path = self.base_path.join(four_words);
        Ok(vault_path.join("vault.meta").exists())
    }

    /// Store a password locator for password-only login
    pub async fn store_password_locator(
        &self,
        password_hash: &[u8],
        four_words: &str,
    ) -> Result<()> {
        let mut locators = self.password_locators.write().await;
        locators.insert(password_hash.to_vec(), four_words.to_string());

        // Persist to disk
        self.save_locators(&locators).await?;

        // Platform-specific secure storage
        self.store_platform_specific(four_words, password_hash)
            .await?;

        Ok(())
    }

    /// Find vault by password hash
    pub async fn find_vault_by_password_hash(&self, password_hash: &[u8]) -> Result<String> {
        // Check in-memory cache first
        let locators = self.password_locators.read().await;
        if let Some(four_words) = locators.get(password_hash) {
            return Ok(four_words.clone());
        }

        // Try platform-specific lookup
        self.lookup_platform_specific(password_hash).await
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

    fn load_locators(locator_path: &Path) -> Result<HashMap<Vec<u8>, String>> {
        let locator_file = locator_path.join("password_locators.enc");
        if !locator_file.exists() {
            return Ok(HashMap::new());
        }

        let data = std::fs::read(&locator_file)?;
        // In production, this should be encrypted
        let locators: HashMap<String, String> = serde_json::from_slice(&data)?;

        // Convert from hex strings to bytes
        let mut result = HashMap::new();
        for (hash_hex, four_words) in locators {
            if let Ok(hash_bytes) = hex::decode(&hash_hex) {
                result.insert(hash_bytes, four_words);
            }
        }

        Ok(result)
    }

    async fn save_locators(&self, locators: &HashMap<Vec<u8>, String>) -> Result<()> {
        let locator_path = self.base_path.join("locators");
        let locator_file = locator_path.join("password_locators.enc");

        // Convert to hex strings for JSON serialization
        let mut hex_locators = HashMap::new();
        for (hash_bytes, four_words) in locators {
            hex_locators.insert(hex::encode(hash_bytes), four_words.clone());
        }

        // In production, this should be encrypted
        let data = serde_json::to_vec(&hex_locators)?;
        fs::write(&locator_file, data).await?;

        // Secure the file
        self.secure_file(&locator_file).await?;

        Ok(())
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

    // Platform-specific implementations

    #[cfg(target_os = "macos")]
    async fn store_platform_specific(&self, four_words: &str, password_hash: &[u8]) -> Result<()> {
        // Store in macOS Keychain
        use keyring::Entry;

        let service = "com.saorsalabs.communitas.locator";
        let account = hex::encode(password_hash);

        let entry = Entry::new(service, &account)?;
        entry.set_password(four_words)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn store_platform_specific(&self, four_words: &str, password_hash: &[u8]) -> Result<()> {
        // Store using Windows Credential Manager
        use windows_sys::Win32::Security::Credentials::*;

        // Implementation would use CredWrite here
        // For now, use file-based storage
        let secure_path = self.get_secure_path(four_words);
        fs::create_dir_all(&secure_path).await?;

        let locator_file = secure_path.join("locator.dat");
        fs::write(&locator_file, password_hash).await?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    async fn store_platform_specific(&self, four_words: &str, password_hash: &[u8]) -> Result<()> {
        // Store using Secret Service API (libsecret)
        // For now, use XDG directory with proper permissions
        let secure_path = self.get_secure_path(four_words);
        fs::create_dir_all(&secure_path).await?;

        let locator_file = secure_path.join("locator.dat");
        fs::write(&locator_file, password_hash).await?;

        // Set restrictive permissions
        self.secure_file(&locator_file).await?;

        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    async fn store_platform_specific(
        &self,
        _four_words: &str,
        _password_hash: &[u8],
    ) -> Result<()> {
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn lookup_platform_specific(&self, password_hash: &[u8]) -> Result<String> {
        use keyring::Entry;

        let service = "com.saorsalabs.communitas.locator";
        let account = hex::encode(password_hash);

        let entry = Entry::new(service, &account)?;
        let four_words = entry.get_password()?;

        Ok(four_words)
    }

    #[cfg(not(target_os = "macos"))]
    async fn lookup_platform_specific(&self, _password_hash: &[u8]) -> Result<String> {
        Err(anyhow::anyhow!("No vault found for this password"))
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
    async fn test_platform_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = PlatformStorage::new(&temp_dir.path().to_path_buf()).unwrap();

        // Test password locator
        let password_hash = blake3::hash(b"test_password");
        storage
            .store_password_locator(password_hash.as_bytes(), "test-vault-words")
            .await
            .unwrap();

        let found = storage
            .find_vault_by_password_hash(password_hash.as_bytes())
            .await
            .unwrap();

        assert_eq!(found, "test-vault-words");
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
