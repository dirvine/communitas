// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Application identifier for secure storage
const APP_NAME: &str = "Communitas";
const SERVICE_NAME: &str = "communitas-p2p";

/// Encryption key metadata stored with keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureKeyMetadata {
    pub key_id: String,
    pub key_type: String,
    pub scope: Option<String>,
    pub created_at: u64,
    pub last_used: u64,
    pub version: u32,
}

/// Secure storage manager using platform-specific secure storage
#[derive(Debug)]
pub struct SecureStorageManager {
    user_id: String,
}

impl SecureStorageManager {
    /// Create a new secure storage manager for a user
    pub fn new(user_id: String) -> Self {
        debug!("Initializing secure storage for user: {}", user_id);
        Self { user_id }
    }

    /// Store encryption keys securely
    pub async fn store_encryption_keys(
        &self,
        master_key: &str,
        key_pair: &str,
    ) -> Result<()> {
        debug!("Storing encryption keys for user: {}", self.user_id);

        // Store master key
        let master_key_entry = Entry::new(SERVICE_NAME, &format!("{}_master_key", self.user_id))
            .context("Failed to create master key entry")?;
        
        master_key_entry
            .set_password(master_key)
            .context("Failed to store master key")?;

        // Store key pair
        let key_pair_entry = Entry::new(SERVICE_NAME, &format!("{}_key_pair", self.user_id))
            .context("Failed to create key pair entry")?;
            
        key_pair_entry
            .set_password(key_pair)
            .context("Failed to store key pair")?;

        // Store metadata
        let metadata = serde_json::json!({
            "user_id": self.user_id,
            "stored_at": chrono::Utc::now().timestamp(),
            "version": 1,
            "app": APP_NAME
        });

        let metadata_entry = Entry::new(SERVICE_NAME, &format!("{}_metadata", self.user_id))
            .context("Failed to create metadata entry")?;
            
        metadata_entry
            .set_password(&metadata.to_string())
            .context("Failed to store metadata")?;

        debug!("Successfully stored encryption keys");
        Ok(())
    }

    /// Retrieve encryption keys securely
    pub async fn get_encryption_keys(&self) -> Result<serde_json::Value> {
        debug!("Retrieving encryption keys for user: {}", self.user_id);

        let master_key = self.get_master_key().await?;
        let key_pair = self.get_key_pair().await?;

        let keys = serde_json::json!({
            "masterKey": master_key,
            "keyPair": key_pair,
            "retrievedAt": chrono::Utc::now().timestamp()
        });

        debug!("Successfully retrieved encryption keys");
        Ok(keys)
    }

    /// Get master key from secure storage
    async fn get_master_key(&self) -> Result<String> {
        let key_name = format!("{}_master_key", self.user_id);
        let entry = Entry::new(SERVICE_NAME, &key_name)
            .context("Failed to create master key entry")?;
        
        // Try multiple approaches to handle macOS Keychain behavior
        for attempt in 1..=3 {
            match entry.get_password() {
                Ok(password) => return Ok(password),
                Err(keyring::Error::NoEntry) => {
                    debug!("Master key not found on attempt {}", attempt);
                    
                    if attempt < 3 {
                        // Progressive delay: 50ms, then 200ms
                        let delay_ms = if attempt == 1 { 50 } else { 200 };
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to retrieve master key: {:?}", e));
                }
            }
        }
        
        Err(anyhow::anyhow!("Master key not found after {} attempts", 3))
    }

    /// Get key pair from secure storage
    async fn get_key_pair(&self) -> Result<String> {
        let key_name = format!("{}_key_pair", self.user_id);
        let entry = Entry::new(SERVICE_NAME, &key_name)
            .context("Failed to create key pair entry")?;
        
        // Try multiple approaches to handle macOS Keychain behavior
        for attempt in 1..=3 {
            match entry.get_password() {
                Ok(password) => return Ok(password),
                Err(keyring::Error::NoEntry) => {
                    debug!("Key pair not found on attempt {}", attempt);
                    
                    if attempt < 3 {
                        // Progressive delay: 50ms, then 200ms
                        let delay_ms = if attempt == 1 { 50 } else { 200 };
                        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to retrieve key pair: {:?}", e));
                }
            }
        }
        
        Err(anyhow::anyhow!("Key pair not found after {} attempts", 3))
    }

    /// Store a derived or session key securely
    pub async fn store_derived_key(
        &self,
        key_id: &str,
        key_data: &str,
        metadata: &SecureKeyMetadata,
    ) -> Result<()> {
        debug!("Storing derived key: {} for user: {}", key_id, self.user_id);

        // Store the key data
        let key_entry = Entry::new(SERVICE_NAME, &format!("{}_{}_key", self.user_id, key_id))
            .context("Failed to create derived key entry")?;
            
        key_entry
            .set_password(key_data)
            .context("Failed to store derived key")?;

        // Store the metadata
        let metadata_json = serde_json::to_string(metadata)
            .context("Failed to serialize key metadata")?;
            
        let metadata_entry = Entry::new(SERVICE_NAME, &format!("{}_{}_meta", self.user_id, key_id))
            .context("Failed to create key metadata entry")?;
            
        metadata_entry
            .set_password(&metadata_json)
            .context("Failed to store key metadata")?;

        debug!("Successfully stored derived key: {}", key_id);
        Ok(())
    }

    /// Retrieve a derived or session key
    pub async fn get_derived_key(&self, key_id: &str) -> Result<(String, SecureKeyMetadata)> {
        debug!("Retrieving derived key: {} for user: {}", key_id, self.user_id);

        // Get key data
        let key_entry = Entry::new(SERVICE_NAME, &format!("{}_{}_key", self.user_id, key_id))
            .context("Failed to create derived key entry")?;
            
        let key_data = key_entry
            .get_password()
            .context("Failed to retrieve derived key")?;

        // Get metadata
        let metadata_entry = Entry::new(SERVICE_NAME, &format!("{}_{}_meta", self.user_id, key_id))
            .context("Failed to create key metadata entry")?;
            
        let metadata_json = metadata_entry
            .get_password()
            .context("Failed to retrieve key metadata")?;
            
        let metadata: SecureKeyMetadata = serde_json::from_str(&metadata_json)
            .context("Failed to deserialize key metadata")?;

        debug!("Successfully retrieved derived key: {}", key_id);
        Ok((key_data, metadata))
    }

    /// List all stored keys for this user
    pub async fn list_stored_keys(&self) -> Result<Vec<SecureKeyMetadata>> {
        debug!("Listing stored keys for user: {}", self.user_id);
        
        // Note: keyring doesn't provide a way to list all entries
        // In a production system, we'd maintain an index of key IDs
        // For now, we'll return the keys we know about from metadata
        
        warn!("Key listing not fully implemented - keyring crate limitation");
        Ok(vec![])
    }

    /// Delete a specific key
    pub async fn delete_key(&self, key_id: &str) -> Result<()> {
        debug!("Deleting key: {} for user: {}", key_id, self.user_id);

        // Delete key data
        let key_entry = Entry::new(SERVICE_NAME, &format!("{}_{}_key", self.user_id, key_id))
            .context("Failed to create derived key entry")?;
            
        if let Err(e) = key_entry.delete_password() {
            warn!("Failed to delete key data for {}: {}", key_id, e);
        }

        // Delete metadata
        let metadata_entry = Entry::new(SERVICE_NAME, &format!("{}_{}_meta", self.user_id, key_id))
            .context("Failed to create key metadata entry")?;
            
        if let Err(e) = metadata_entry.delete_password() {
            warn!("Failed to delete key metadata for {}: {}", key_id, e);
        }

        debug!("Successfully deleted key: {}", key_id);
        Ok(())
    }

    /// Delete all keys for this user (cleanup)
    pub async fn delete_all_keys(&self) -> Result<()> {
        debug!("Deleting all keys for user: {}", self.user_id);

        // Delete master key
        let master_key_entry = Entry::new(SERVICE_NAME, &format!("{}_master_key", self.user_id))
            .context("Failed to create master key entry")?;
        if let Err(e) = master_key_entry.delete_password() {
            warn!("Failed to delete master key: {}", e);
        }

        // Delete key pair
        let key_pair_entry = Entry::new(SERVICE_NAME, &format!("{}_key_pair", self.user_id))
            .context("Failed to create key pair entry")?;
        if let Err(e) = key_pair_entry.delete_password() {
            warn!("Failed to delete key pair: {}", e);
        }

        // Delete metadata
        let metadata_entry = Entry::new(SERVICE_NAME, &format!("{}_metadata", self.user_id))
            .context("Failed to create metadata entry")?;
        if let Err(e) = metadata_entry.delete_password() {
            warn!("Failed to delete metadata: {}", e);
        }

        debug!("Successfully deleted all keys for user: {}", self.user_id);
        Ok(())
    }

    /// Check if secure storage is available on this platform
    pub fn is_available() -> bool {
        // Try to create a test entry to check if secure storage is available
        match Entry::new(SERVICE_NAME, "test_availability") {
            Ok(entry) => {
                // Try to set and delete a test credential
                match entry.set_password("test") {
                    Ok(_) => {
                        let _ = entry.delete_password(); // Clean up
                        true
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Get storage backend information
    pub fn get_storage_info() -> String {
        #[cfg(target_os = "macos")]
        return "macOS Keychain".to_string();
        
        #[cfg(target_os = "windows")]
        return "Windows Credential Manager".to_string();
        
        #[cfg(target_os = "linux")]
        return "Linux Secret Service".to_string();
        
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        return "Unknown Platform".to_string();
    }

    /// Migrate from file-based storage to secure storage
    pub async fn migrate_from_file_storage(&self, app_data_dir: &std::path::Path) -> Result<bool> {
        debug!("Attempting to migrate from file storage for user: {}", self.user_id);

        let keys_dir = app_data_dir.join("encryption_keys");
        let user_keys_file = keys_dir.join(format!("{}_keys.json", self.user_id));

        if !user_keys_file.exists() {
            debug!("No file storage to migrate from");
            return Ok(false);
        }

        // Read existing file-based keys
        let content = tokio::fs::read_to_string(&user_keys_file).await
            .context("Failed to read existing key file")?;
            
        let keys: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse existing key file")?;

        // Extract keys
        let master_key = keys.get("masterKey")
            .and_then(|v| v.as_str())
            .context("Master key not found in file")?;
            
        let key_pair = keys.get("keyPair")
            .and_then(|v| v.as_str())
            .context("Key pair not found in file")?;

        // Store in secure storage
        self.store_encryption_keys(master_key, key_pair).await
            .context("Failed to store keys in secure storage during migration")?;

        // Backup and remove old file
        let backup_file = user_keys_file.with_extension("json.backup");
        tokio::fs::rename(&user_keys_file, &backup_file).await
            .context("Failed to backup old key file")?;

        debug!("Successfully migrated keys from file storage to secure storage");
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_secure_storage_availability() {
        // This test just checks if secure storage is available
        let available = SecureStorageManager::is_available();
        println!("Secure storage available: {}", available);
        println!("Storage backend: {}", SecureStorageManager::get_storage_info());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_keys() {
        if !SecureStorageManager::is_available() {
            println!("Skipping test - secure storage not available");
            return;
        }

        let test_user = format!("test_user_{}", Uuid::new_v4());
        let manager = SecureStorageManager::new(test_user.clone());
        
        println!("Testing with user: {}", test_user);
        
        // Clean up any existing test data
        let cleanup_result = manager.delete_all_keys().await;
        println!("Cleanup result: {:?}", cleanup_result);

        // Store test keys
        let result = manager.store_encryption_keys("test_master_key", "test_key_pair").await;
        if let Err(ref e) = result {
            println!("Store error: {}", e);
        }
        assert!(result.is_ok(), "Failed to store keys: {:?}", result.err());
        println!("Successfully stored keys");

        // Add a small delay to ensure keyring has processed the write
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Retrieve keys
        let keys_result = manager.get_encryption_keys().await;
        if let Err(ref e) = keys_result {
            println!("Retrieve error: {}", e);
        }
        assert!(keys_result.is_ok(), "Failed to retrieve keys: {:?}", keys_result.err());

        let keys = keys_result.unwrap();
        println!("Retrieved keys: {}", keys);
        assert_eq!(keys.get("masterKey").unwrap().as_str().unwrap(), "test_master_key");
        assert_eq!(keys.get("keyPair").unwrap().as_str().unwrap(), "test_key_pair");

        // Clean up
        let cleanup_result = manager.delete_all_keys().await;
        println!("Final cleanup result: {:?}", cleanup_result);
    }
}