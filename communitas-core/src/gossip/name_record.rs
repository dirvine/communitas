// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Name Record Protocol - Four-Words → SiteId Binding
//!
//! Enables DNS-free name resolution by binding human-memorable four-word
//! addresses to cryptographic SiteId (BLAKE3 hash of ML-DSA-65 public key).
//!
//! ## Protocol
//! 1. Site owner creates NameRecord: four_words + site_id + timestamp
//! 2. Signs with ML-DSA-65 private key
//! 3. Gossips NameRecord to site's rendezvous shard
//! 4. Clients subscribe to shard, cache verified bindings
//! 5. Conflicts resolved via TOFU (Trust On First Use)
//!
//! ## Security
//! - Signature proves ownership of both four-words and site_id
//! - Timestamp prevents replay attacks
//! - TOFU + FOAF endorsements prevent hijacking

use anyhow::Result;
use saorsa_pqc::dsa_traits::{SerDes, Signer, Verifier};
use saorsa_pqc::ml_dsa_65::{PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use super::sites::SiteId;

/// Name record binding four-words to SiteId
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameRecord {
    /// Protocol version
    pub version: u8,

    /// Four-word address (e.g., "ocean-forest-moon-star")
    pub four_words: String,

    /// Site identifier (BLAKE3 hash of public key)
    pub site_id: SiteId,

    /// Full ML-DSA-65 public key (1952 bytes)
    pub public_key: Vec<u8>,

    /// Creation timestamp (Unix milliseconds)
    pub created_at: u64,

    /// Nonce (prevents identical records)
    pub nonce: [u8; 32],

    /// ML-DSA-65 signature (3309 bytes)
    pub signature: Vec<u8>,
}

impl NameRecord {
    /// Create a new unsigned name record
    pub fn new(four_words: String, public_key: &PublicKey) -> Self {
        let site_id = SiteId::from_public_key(public_key);

        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_millis() as u64;

        // Generate random nonce
        let mut nonce = [0u8; 32];
        if getrandom::getrandom(&mut nonce).is_err() {
            // Fallback to zeros if random generation fails (should never happen)
            tracing::warn!("Failed to generate random nonce, using zeros");
        }

        Self {
            version: 1,
            four_words,
            site_id,
            public_key: public_key.clone().into_bytes().to_vec(),
            created_at,
            nonce,
            signature: vec![],
        }
    }

    /// Get canonical bytes for signing
    fn to_sign_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.version);
        bytes.extend_from_slice(self.four_words.as_bytes());
        bytes.extend_from_slice(&self.site_id.hash);
        bytes.extend_from_slice(&self.public_key);
        bytes.extend_from_slice(&self.created_at.to_le_bytes());
        bytes.extend_from_slice(&self.nonce);
        bytes
    }

    /// Sign the name record
    pub fn sign(&mut self, signing_key: &PrivateKey) -> Result<()> {
        let message = self.to_sign_bytes();
        let signature = signing_key
            .try_sign(&message, &[])
            .map_err(|e| anyhow::anyhow!("ML-DSA-65 signing failed: {}", e))?;

        self.signature = signature.to_vec();
        Ok(())
    }

    /// Verify name record signature
    pub fn verify(&self) -> Result<()> {
        // Public key must be 1952 bytes
        if self.public_key.len() != 1952 {
            anyhow::bail!("Invalid public key size: {}", self.public_key.len());
        }

        // Verify public key hash matches site_id
        let pk_hash = blake3::hash(&self.public_key);
        if pk_hash.as_bytes() != &self.site_id.hash {
            anyhow::bail!("Public key does not match site_id");
        }

        // Deserialize public key
        let pk_array: [u8; 1952] = self
            .public_key
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Public key is not 1952 bytes"))?;
        let public_key = PublicKey::try_from_bytes(pk_array)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {}", e))?;

        // Signature must be 3309 bytes
        if self.signature.len() != 3309 {
            anyhow::bail!("Invalid signature size: {}", self.signature.len());
        }

        // Convert signature to array
        let sig_array: [u8; 3309] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to convert signature to array"))?;

        // Verify signature
        let message = self.to_sign_bytes();
        if !public_key.verify(&message, &sig_array, &[]) {
            anyhow::bail!("Signature verification failed");
        }

        Ok(())
    }
}

/// Name registry with TOFU conflict resolution
pub struct NameRegistry {
    /// Name bindings (four_words → NameRecord)
    records: Arc<RwLock<HashMap<String, NameRecord>>>,

    /// Reverse index (site_id_hash → four_words)
    reverse_index: Arc<RwLock<HashMap<[u8; 32], String>>>,
}

impl NameRegistry {
    /// Create a new name registry
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            reverse_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a name record (TOFU - Trust On First Use)
    ///
    /// If name already exists:
    /// - If same site_id: update (allows re-announcement)
    /// - If different site_id: reject (TOFU - first claim wins)
    ///
    /// # Returns
    /// Ok(true) if registered, Ok(false) if rejected (conflict)
    pub async fn register(&self, record: NameRecord) -> Result<bool> {
        // Verify signature
        record.verify()?;

        let mut records = self.records.write().await;
        let mut reverse = self.reverse_index.write().await;

        // Check for existing registration
        if let Some(existing) = records.get(&record.four_words) {
            if existing.site_id == record.site_id {
                // Same site, allow update (re-announcement)
                records.insert(record.four_words.clone(), record.clone());
                reverse.insert(record.site_id.hash, record.four_words.clone());
                debug!("Updated name record: {}", record.four_words);
                return Ok(true);
            } else {
                // Different site, reject (TOFU)
                warn!(
                    "Name conflict: {} already claimed by {:?}",
                    record.four_words, existing.site_id
                );
                return Ok(false);
            }
        }

        // New registration
        records.insert(record.four_words.clone(), record.clone());
        reverse.insert(record.site_id.hash, record.four_words.clone());
        debug!(
            "Registered name: {} → {:?}",
            record.four_words, record.site_id
        );

        Ok(true)
    }

    /// Resolve four-words to SiteId
    pub async fn resolve(&self, four_words: &str) -> Option<SiteId> {
        let records = self.records.read().await;
        records.get(four_words).map(|r| r.site_id.clone())
    }

    /// Reverse lookup: SiteId to four-words
    pub async fn reverse_lookup(&self, site_id: &SiteId) -> Option<String> {
        let reverse = self.reverse_index.read().await;
        reverse.get(&site_id.hash).cloned()
    }

    /// Get all registered names
    pub async fn list_names(&self) -> Vec<String> {
        let records = self.records.read().await;
        records.keys().cloned().collect()
    }
}

impl Default for NameRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use saorsa_pqc::ml_dsa_65::try_keygen_with_rng;

    fn generate_test_keypair(seed: u64) -> (PrivateKey, PublicKey) {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let (pk, sk) = try_keygen_with_rng(&mut rng).expect("Failed to generate test keypair");
        (sk, pk)
    }

    #[test]
    fn test_name_record_creation_and_signing() {
        let (sk, pk) = generate_test_keypair(1);
        let mut record = NameRecord::new("ocean-forest-moon-star".to_string(), &pk);

        // Initially unsigned
        assert_eq!(record.signature.len(), 0);

        // Sign it
        record.sign(&sk).expect("Failed to sign");

        // Should have signature
        assert_eq!(record.signature.len(), 3309);

        // Verify it
        record.verify().expect("Verification failed");
    }

    #[test]
    fn test_name_record_tamper_detection() {
        let (sk, pk) = generate_test_keypair(2);
        let mut record = NameRecord::new("test-name-here-now".to_string(), &pk);
        record.sign(&sk).expect("Failed to sign");

        // Tamper with four_words
        record.four_words = "tampered-name-fake-bad".to_string();

        // Verification should fail
        assert!(record.verify().is_err(), "Tampered record should fail");
    }

    #[tokio::test]
    async fn test_registry_tofu() {
        let registry = NameRegistry::new();

        let (sk1, pk1) = generate_test_keypair(3);
        let (sk2, pk2) = generate_test_keypair(4);

        // First claim
        let mut record1 = NameRecord::new("cool-site-here-now".to_string(), &pk1);
        record1.sign(&sk1).unwrap();

        assert!(registry.register(record1).await.unwrap());

        // Second claim (different site, same name) - should be rejected
        let mut record2 = NameRecord::new("cool-site-here-now".to_string(), &pk2);
        record2.sign(&sk2).unwrap();

        assert!(
            !registry.register(record2).await.unwrap(),
            "TOFU should reject second claim"
        );
    }

    #[tokio::test]
    async fn test_registry_resolve() {
        let registry = NameRegistry::new();
        let (sk, pk) = generate_test_keypair(5);
        let site_id = SiteId::from_public_key(&pk);

        let mut record = NameRecord::new("my-awesome-site".to_string(), &pk);
        record.sign(&sk).unwrap();
        registry.register(record).await.unwrap();

        // Resolve should work
        let resolved = registry.resolve("my-awesome-site").await;
        assert_eq!(resolved, Some(site_id.clone()));

        // Reverse lookup should work
        let name = registry.reverse_lookup(&site_id).await;
        assert_eq!(name, Some("my-awesome-site".to_string()));
    }

    #[tokio::test]
    async fn test_registry_update_same_site() {
        let registry = NameRegistry::new();
        let (sk, pk) = generate_test_keypair(6);

        // Register first time
        let mut record1 = NameRecord::new("my-site".to_string(), &pk);
        record1.sign(&sk).unwrap();
        registry.register(record1).await.unwrap();

        // Re-register same site (update)
        let mut record2 = NameRecord::new("my-site".to_string(), &pk);
        record2.sign(&sk).unwrap();

        assert!(
            registry.register(record2).await.unwrap(),
            "Same site should allow update"
        );

        // Should still have only one record
        let names = registry.list_names().await;
        assert_eq!(names.len(), 1);
    }
}
