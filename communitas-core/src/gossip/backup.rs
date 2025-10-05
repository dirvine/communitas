// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Backup Management via Favourite Contacts
//!
//! Implements SPEC2.md §4: Data and backup
//!
//! - Local-first state
//! - Mark Favourite contacts to store encrypted replicas (ChaCha20Poly1305)
//! - Replicas include: contact list, device list, minimal account metadata
//! - Recovery: connect to any favourite, run delta-CRDT anti-entropy, rejoin MLS groups

use anyhow::{Context, Result};
use saorsa_pqc::symmetric::{ChaCha20Poly1305Cipher, SymmetricKey};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Backup data stored with favourite contacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    /// Contact list (four-word addresses)
    pub contacts: HashSet<String>,

    /// Device list for this identity
    pub devices: Vec<DeviceInfo>,

    /// Minimal account metadata
    pub account_metadata: AccountMetadata,

    /// Last backup timestamp
    pub last_backup: chrono::DateTime<chrono::Utc>,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_name: String,
    pub device_type: String, // "Desktop", "Mobile", etc.
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

/// Minimal account metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadata {
    pub four_words: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Backup manager for favourite contacts replication
pub struct BackupManager {
    favourite_contacts: HashSet<String>,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new() -> Self {
        Self {
            favourite_contacts: HashSet::new(),
        }
    }

    /// Add a favourite contact for backup replication
    pub fn add_favourite(&mut self, four_words: String) {
        self.favourite_contacts.insert(four_words);
    }

    /// Remove a favourite contact
    pub fn remove_favourite(&mut self, four_words: &str) {
        self.favourite_contacts.remove(four_words);
    }

    /// Get list of favourite contacts
    pub fn get_favourites(&self) -> Vec<String> {
        self.favourite_contacts.iter().cloned().collect()
    }

    /// Create backup data for replication
    pub fn create_backup(
        &self,
        contacts: HashSet<String>,
        devices: Vec<DeviceInfo>,
        account_metadata: AccountMetadata,
    ) -> BackupData {
        BackupData {
            contacts,
            devices,
            account_metadata,
            last_backup: chrono::Utc::now(),
        }
    }

    /// Encrypt backup data for storage with a favourite contact
    ///
    /// Uses ChaCha20Poly1305 AEAD encryption per SPEC2.md §10
    ///
    /// Package format: [nonce (12 bytes) || ciphertext || key (32 bytes)]
    pub async fn encrypt_backup(&self, backup: &BackupData, _recipient: &str) -> Result<Vec<u8>> {
        // 1. Serialize backup data
        let plaintext = bincode::serialize(backup).context("Failed to serialize backup data")?;

        // 2. Generate per-backup encryption key (ChaCha20Poly1305)
        let key = SymmetricKey::generate();
        let cipher = ChaCha20Poly1305Cipher::new(&key);

        // 3. Encrypt with ChaCha20Poly1305 AEAD
        let (ciphertext, nonce) = cipher
            .encrypt(&plaintext, None)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // 4. Package: [nonce (12 bytes) || ciphertext || key (32 bytes)]
        let mut package = Vec::with_capacity(12 + ciphertext.len() + 32);
        package.extend_from_slice(nonce.as_slice());
        package.extend_from_slice(&ciphertext);
        package.extend_from_slice(key.as_bytes());

        Ok(package)
    }

    /// Decrypt backup data received from a favourite contact
    ///
    /// Uses ChaCha20Poly1305 AEAD decryption per SPEC2.md §10
    ///
    /// Package format: [nonce (12 bytes) || ciphertext || key (32 bytes)]
    pub async fn decrypt_backup(&self, encrypted: &[u8]) -> Result<BackupData> {
        // 1. Validate package length
        if encrypted.len() < 44 {
            anyhow::bail!(
                "Invalid encrypted package: too short (minimum 44 bytes for nonce + key)"
            );
        }

        // 2. Unpack components
        let nonce_bytes = &encrypted[0..12];
        let key_bytes = &encrypted[encrypted.len() - 32..];
        let ciphertext = &encrypted[12..encrypted.len() - 32];

        // 3. Reconstruct key and cipher
        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
        let key = SymmetricKey::from_bytes(key_array);
        let cipher = ChaCha20Poly1305Cipher::new(&key);

        // 4. Decrypt with ChaCha20Poly1305 AEAD
        let nonce: [u8; 12] = nonce_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
        let plaintext = cipher.decrypt(ciphertext, &nonce, None).map_err(|e| {
            anyhow::anyhow!("Decryption failed (possible tampering or wrong key): {}", e)
        })?;

        // 5. Deserialize backup data
        let backup: BackupData =
            bincode::deserialize(&plaintext).context("Failed to deserialize backup data")?;

        Ok(backup)
    }

    /// Replicate backup to all favourite contacts
    ///
    /// Note: Actual network transmission is handled by GossipContext.
    /// This method prepares encrypted packages for transmission.
    pub async fn replicate_to_favourites(
        &self,
        backup: &BackupData,
    ) -> Result<Vec<(String, Vec<u8>)>> {
        let mut packages = Vec::new();

        for favourite in &self.favourite_contacts {
            let encrypted = self.encrypt_backup(backup, favourite).await?;
            tracing::debug!(
                "Prepared encrypted backup for {}: {} bytes",
                favourite,
                encrypted.len()
            );
            packages.push((favourite.clone(), encrypted));
        }

        Ok(packages)
    }

    /// Recover from a favourite contact
    ///
    /// Note: This prepares the recovery request. Actual network operations
    /// are handled by GossipContext (CRDT anti-entropy, MLS group rejoining).
    pub async fn prepare_recovery_request(&self, favourite: &str) -> Result<String> {
        // Create recovery request message
        let request = serde_json::json!({
            "type": "backup_recovery_request",
            "favourite": favourite,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(request.to_string())
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_manager_favourites() {
        let mut mgr = BackupManager::new();

        mgr.add_favourite("river-mountain-cloud-light".to_string());
        mgr.add_favourite("ocean-forest-moon-star".to_string());

        let favourites = mgr.get_favourites();
        assert_eq!(favourites.len(), 2);

        mgr.remove_favourite("ocean-forest-moon-star");
        let favourites = mgr.get_favourites();
        assert_eq!(favourites.len(), 1);
    }

    #[tokio::test]
    async fn test_chacha20poly1305_encryption_roundtrip() {
        // Test complete encryption -> decryption cycle
        let mgr = BackupManager::new();

        // Create test backup data
        let mut contacts = HashSet::new();
        contacts.insert("ocean-forest-moon-star".to_string());
        contacts.insert("river-mountain-cloud-light".to_string());

        let backup = BackupData {
            contacts,
            devices: vec![DeviceInfo {
                device_name: "TestDevice".to_string(),
                device_type: "Desktop".to_string(),
                last_seen: chrono::Utc::now(),
            }],
            account_metadata: AccountMetadata {
                four_words: "test-identity-four-words".to_string(),
                display_name: "Test User".to_string(),
                created_at: chrono::Utc::now(),
            },
            last_backup: chrono::Utc::now(),
        };

        // Encrypt
        let encrypted = mgr
            .encrypt_backup(&backup, "recipient")
            .await
            .expect("Encryption should succeed");

        // Verify package structure: [nonce (12) || ciphertext || key (32)]
        assert!(encrypted.len() >= 44, "Package must be at least 44 bytes");

        // Decrypt
        let decrypted = mgr
            .decrypt_backup(&encrypted)
            .await
            .expect("Decryption should succeed");

        // Verify data integrity
        assert_eq!(decrypted.contacts.len(), 2);
        assert_eq!(decrypted.devices.len(), 1);
        assert_eq!(decrypted.account_metadata.display_name, "Test User");
    }

    #[tokio::test]
    async fn test_encryption_produces_different_ciphertexts() {
        // Ensure encryption is non-deterministic (different nonces)
        let mgr = BackupManager::new();

        let backup = BackupData {
            contacts: HashSet::new(),
            devices: vec![],
            account_metadata: AccountMetadata {
                four_words: "test-four-words".to_string(),
                display_name: "Test".to_string(),
                created_at: chrono::Utc::now(),
            },
            last_backup: chrono::Utc::now(),
        };

        let encrypted1 = mgr
            .encrypt_backup(&backup, "recipient")
            .await
            .expect("Encryption 1 should succeed");
        let encrypted2 = mgr
            .encrypt_backup(&backup, "recipient")
            .await
            .expect("Encryption 2 should succeed");

        // Ciphertexts should differ due to random nonces
        assert_ne!(
            encrypted1, encrypted2,
            "Encryption should use random nonces"
        );

        // But both should decrypt to same plaintext
        let decrypted1 = mgr
            .decrypt_backup(&encrypted1)
            .await
            .expect("Decryption 1 should succeed");
        let decrypted2 = mgr
            .decrypt_backup(&encrypted2)
            .await
            .expect("Decryption 2 should succeed");

        assert_eq!(
            decrypted1.account_metadata.four_words,
            decrypted2.account_metadata.four_words
        );
    }

    #[tokio::test]
    async fn test_decryption_fails_on_tampered_data() {
        // Verify AEAD protection against tampering
        let mgr = BackupManager::new();

        let backup = BackupData {
            contacts: HashSet::new(),
            devices: vec![],
            account_metadata: AccountMetadata {
                four_words: "test-four-words".to_string(),
                display_name: "Test".to_string(),
                created_at: chrono::Utc::now(),
            },
            last_backup: chrono::Utc::now(),
        };

        let mut encrypted = mgr
            .encrypt_backup(&backup, "recipient")
            .await
            .expect("Encryption should succeed");

        // Tamper with ciphertext (flip a bit in the middle)
        let mid = encrypted.len() / 2;
        encrypted[mid] ^= 0xFF;

        // Decryption should fail due to authentication tag mismatch
        let result = mgr.decrypt_backup(&encrypted).await;
        assert!(result.is_err(), "Decryption should fail on tampered data");
    }

    #[tokio::test]
    async fn test_decryption_fails_on_invalid_package() {
        let mgr = BackupManager::new();

        // Too short package (less than 44 bytes)
        let short_package = vec![0u8; 43];
        let result = mgr.decrypt_backup(&short_package).await;
        assert!(
            result.is_err(),
            "Should reject package shorter than 44 bytes"
        );

        // Empty package
        let empty_package = vec![];
        let result = mgr.decrypt_backup(&empty_package).await;
        assert!(result.is_err(), "Should reject empty package");
    }

    #[tokio::test]
    async fn test_replicate_to_favourites() {
        let mut mgr = BackupManager::new();
        mgr.add_favourite("alice-bob-charlie-david".to_string());
        mgr.add_favourite("eve-frank-grace-henry".to_string());

        let backup = BackupData {
            contacts: HashSet::new(),
            devices: vec![],
            account_metadata: AccountMetadata {
                four_words: "test-four-words".to_string(),
                display_name: "Test".to_string(),
                created_at: chrono::Utc::now(),
            },
            last_backup: chrono::Utc::now(),
        };

        let packages = mgr
            .replicate_to_favourites(&backup)
            .await
            .expect("Replication should succeed");

        // Should create one encrypted package per favourite
        assert_eq!(packages.len(), 2);

        // Each package should be valid encrypted backup
        for (_recipient, encrypted) in packages {
            assert!(encrypted.len() >= 44, "Each package should be valid");
            let decrypted = mgr
                .decrypt_backup(&encrypted)
                .await
                .expect("Package should decrypt successfully");
            assert_eq!(decrypted.account_metadata.display_name, "Test");
        }
    }

    #[tokio::test]
    async fn test_large_backup_encryption() {
        // Test encryption with larger data sets
        let mgr = BackupManager::new();

        let mut contacts = HashSet::new();
        for i in 0..100 {
            contacts.insert(format!("contact-{}-four-words", i));
        }

        let mut devices = vec![];
        for i in 0..10 {
            devices.push(DeviceInfo {
                device_name: format!("Device {}", i),
                device_type: "Desktop".to_string(),
                last_seen: chrono::Utc::now(),
            });
        }

        let backup = BackupData {
            contacts,
            devices,
            account_metadata: AccountMetadata {
                four_words: "large-test-four-words".to_string(),
                display_name: "Large Test".to_string(),
                created_at: chrono::Utc::now(),
            },
            last_backup: chrono::Utc::now(),
        };

        let encrypted = mgr
            .encrypt_backup(&backup, "recipient")
            .await
            .expect("Should encrypt large backup");

        let decrypted = mgr
            .decrypt_backup(&encrypted)
            .await
            .expect("Should decrypt large backup");

        assert_eq!(decrypted.contacts.len(), 100);
        assert_eq!(decrypted.devices.len(), 10);
    }

    #[tokio::test]
    async fn test_prepare_recovery_request() {
        let mgr = BackupManager::new();

        let request = mgr
            .prepare_recovery_request("alice-bob-charlie-david")
            .await
            .expect("Should create recovery request");

        // Parse JSON
        let parsed: serde_json::Value =
            serde_json::from_str(&request).expect("Request should be valid JSON");

        assert_eq!(parsed["type"], "backup_recovery_request");
        assert_eq!(parsed["favourite"], "alice-bob-charlie-david");
        assert!(parsed["timestamp"].is_string());
    }
}
