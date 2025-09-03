// Isolated test for secure storage functionality
use anyhow::{Context, Result};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
        Self { user_id }
    }

    /// Store encryption keys securely
    pub async fn store_encryption_keys(&self, master_key: &str, key_pair: &str) -> Result<()> {
        // Store master key
        let master_key_name = format!("{}_master_key", self.user_id);
        println!("Debug: Storing master key with name: {}", master_key_name);

        let master_key_entry = Entry::new(SERVICE_NAME, &master_key_name)
            .context("Failed to create master key entry")?;

        master_key_entry
            .set_password(master_key)
            .context("Failed to store master key")?;

        println!("Debug: Successfully stored master key");

        // Immediately verify the key was stored
        match master_key_entry.get_password() {
            Ok(retrieved) => println!("Debug: Immediate verification successful: {}", retrieved),
            Err(e) => println!("Debug: Immediate verification failed: {:?}", e),
        }

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

        Ok(())
    }

    /// Retrieve encryption keys securely
    pub async fn get_encryption_keys(&self) -> Result<serde_json::Value> {
        let master_key = self.get_master_key().await?;
        let key_pair = self.get_key_pair().await?;

        let keys = serde_json::json!({
            "masterKey": master_key,
            "keyPair": key_pair,
            "retrievedAt": chrono::Utc::now().timestamp()
        });

        Ok(keys)
    }

    /// Get master key from secure storage
    async fn get_master_key(&self) -> Result<String> {
        let key_name = format!("{}_master_key", self.user_id);
        println!("Debug: Getting master key with name: {}", key_name);

        let entry =
            Entry::new(SERVICE_NAME, &key_name).context("Failed to create master key entry")?;

        println!("Debug: Created entry, attempting to get password...");
        match entry.get_password() {
            Ok(password) => {
                println!("Debug: Successfully got password: {}", password);
                Ok(password)
            }
            Err(e) => {
                println!("Debug: keyring error: {:?}", e);
                Err(anyhow::anyhow!("Failed to retrieve master key: {:?}", e))
            }
        }
    }

    /// Get key pair from secure storage
    async fn get_key_pair(&self) -> Result<String> {
        let entry = Entry::new(SERVICE_NAME, &format!("{}_key_pair", self.user_id))
            .context("Failed to create key pair entry")?;

        entry.get_password().context("Failed to retrieve key pair")
    }

    /// Delete all keys for this user (cleanup)
    pub async fn delete_all_keys(&self) -> Result<()> {
        // Delete master key
        let master_key_entry = Entry::new(SERVICE_NAME, &format!("{}_master_key", self.user_id))
            .context("Failed to create master key entry")?;
        let _ = master_key_entry.delete_credential();

        // Delete key pair
        let key_pair_entry = Entry::new(SERVICE_NAME, &format!("{}_key_pair", self.user_id))
            .context("Failed to create key pair entry")?;
        let _ = key_pair_entry.delete_credential();

        // Delete metadata
        let metadata_entry = Entry::new(SERVICE_NAME, &format!("{}_metadata", self.user_id))
            .context("Failed to create metadata entry")?;
        let _ = metadata_entry.delete_credential();

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
                        let _ = entry.delete_credential(); // Clean up
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
}

#[tokio::main]
async fn main() {
    println!("=== Secure Storage Test ===");

    // Test availability
    println!("Testing secure storage availability...");
    let available = SecureStorageManager::is_available();
    println!("Secure storage available: {}", available);
    println!(
        "Storage backend: {}",
        SecureStorageManager::get_storage_info()
    );

    if !available {
        println!("❌ Secure storage not available, skipping tests");
        return;
    }

    // Test store and retrieve
    let test_user = format!("test_user_{}", Uuid::new_v4());
    let manager = SecureStorageManager::new(test_user.clone());

    println!("\nTesting with user: {}", test_user);

    // Clean up any existing test data
    println!("Cleaning up existing data...");
    let cleanup_result = manager.delete_all_keys().await;
    println!("Cleanup result: {:?}", cleanup_result);

    // Store test keys
    println!("Storing encryption keys...");
    let result = manager
        .store_encryption_keys("test_master_key", "test_key_pair")
        .await;
    match &result {
        Ok(_) => println!("✓ Successfully stored keys"),
        Err(e) => println!("❌ Store error: {}", e),
    }

    if result.is_err() {
        println!("❌ Failed to store keys, aborting test");
        return;
    }

    // Add a small delay to ensure keyring has processed the write
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Retrieve keys
    println!("Retrieving encryption keys...");
    let keys_result = manager.get_encryption_keys().await;
    match &keys_result {
        Ok(keys) => {
            println!("✓ Successfully retrieved keys: {}", keys);

            // Verify the keys match
            if keys.get("masterKey").and_then(|v| v.as_str()) == Some("test_master_key")
                && keys.get("keyPair").and_then(|v| v.as_str()) == Some("test_key_pair")
            {
                println!("✅ Key verification passed!");
            } else {
                println!("❌ Key verification failed");
                println!("  Expected masterKey: test_master_key");
                println!(
                    "  Actual masterKey: {:?}",
                    keys.get("masterKey").and_then(|v| v.as_str())
                );
                println!("  Expected keyPair: test_key_pair");
                println!(
                    "  Actual keyPair: {:?}",
                    keys.get("keyPair").and_then(|v| v.as_str())
                );
            }
        }
        Err(e) => println!("❌ Retrieve error: {}", e),
    }

    // Clean up
    println!("Final cleanup...");
    let cleanup_result = manager.delete_all_keys().await;
    println!("Final cleanup result: {:?}", cleanup_result);

    println!("=== Test Complete ===");
}
