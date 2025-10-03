//! Content-addressed blobs for identity system
//!
//! This module defines larger data structures stored as content-addressed blobs:
//! - IdentityDescriptorBlob: Authoritative signed identity with PQC keys
//! - ConnectionBlob: NAT traversal and networking information
//! - SiteManifestBlob: Website content manifest

use crate::dht_identity::{ContentId, MAX_WEB_CONTENT_SIZE, PROTOCOL_VERSION};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Re-export PQC types from saorsa-core
use saorsa_core::quantum_crypto::ant_quic_integration::{
    MlDsaPublicKey, MlDsaSecretKey, MlDsaSignature,
};

/// Transport key entry for ant-quic compatibility
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportKeyEntry {
    /// SubjectPublicKeyInfo bytes
    #[serde(with = "serde_bytes")]
    pub spki: Vec<u8>,

    /// Algorithm identifier (e.g., "Ed25519", "ML-DSA")
    pub algorithm: String,

    /// Peer ID = blake3(spki)
    pub peer_id: [u8; 32],
}

/// Continuity information for key rotation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuityProof {
    /// Hash of previous ML-DSA public key
    pub previous_key_hash: [u8; 32],

    /// Rotation signature by previous key over (prev_hash, new_hash, four_words, seq, ts)
    #[serde(with = "serde_bytes")]
    pub rotation_signature: Vec<u8>,
}

/// Media asset references
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaAssets {
    /// Avatar image content ID
    pub avatar_cid: Option<ContentId>,

    /// Banner image content ID  
    pub banner_cid: Option<ContentId>,
}

/// Authoritative identity descriptor with PQC signatures
/// This is the main signed blob that proves identity ownership
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityDescriptorBlob {
    /// Blob format version
    #[serde(rename = "v")]
    pub version: u8,

    /// Normalized four-word identity
    #[serde(rename = "four")]
    pub four_words: String,

    /// Hash of IdentityRootRecord core content (excluding descriptor_cid) this descriptor binds to
    #[serde(rename = "root_digest")]
    pub root_digest: [u8; 32],

    /// ML-DSA-65 public key (1952 bytes)
    #[serde(rename = "ml_dsa_pub", with = "serde_bytes")]
    pub ml_dsa_public_key: Vec<u8>,

    /// ML-KEM-768 public key (1184 bytes)
    #[serde(rename = "ml_kem_pub", with = "serde_bytes")]
    pub ml_kem_public_key: Vec<u8>,

    /// Transport keys for ant-quic integration
    #[serde(rename = "transport_keys")]
    pub transport_keys: Vec<TransportKeyEntry>,

    /// Preferred display name (max 128 bytes)
    #[serde(rename = "preferred_display_name")]
    pub preferred_display_name: String,

    /// Optional continuity proof for key rotation
    #[serde(rename = "continuity", skip_serializing_if = "Option::is_none")]
    pub continuity: Option<ContinuityProof>,

    /// Optional site manifest CID
    #[serde(rename = "site_manifest_cid", skip_serializing_if = "Option::is_none")]
    pub site_manifest_cid: Option<ContentId>,

    /// Optional media assets
    #[serde(rename = "media", skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaAssets>,

    /// Optional capability flags and policies
    #[serde(rename = "capabilities", skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<BTreeMap<String, serde_cbor::Value>>,

    /// ML-DSA signature over canonical CBOR of all fields except signature
    #[serde(rename = "sig", with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// Networking and NAT traversal information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionBlob {
    /// Blob format version  
    #[serde(rename = "v")]
    pub version: u8,

    /// Transport ID for consistency check
    #[serde(rename = "peer")]
    pub peer_transport_id: [u8; 32],

    /// Bootstrap/rendezvous nodes for NAT traversal coordination
    #[serde(rename = "rendezvous")]
    pub rendezvous_nodes: Vec<RendezvousEntry>,

    /// Optional relay hints (not mandatory for ant-quic)
    #[serde(rename = "relays", skip_serializing_if = "Option::is_none")]
    pub relay_hints: Option<Vec<RelayEntry>>,

    /// Connection policy settings
    #[serde(rename = "policy")]
    pub policy: ConnectionPolicy,

    /// Freshness information
    #[serde(rename = "freshness")]
    pub freshness: FreshnessInfo,
}

/// Rendezvous/bootstrap node entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendezvousEntry {
    /// Address (host:port or four-word endpoint with explicit port)
    pub address: String,

    /// Priority for selection (lower = higher priority)
    pub priority: u8,
}

/// Relay node hint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayEntry {
    /// Relay identifier or address
    pub id: String,

    /// Priority for selection
    pub priority: u8,
}

/// Connection policy configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionPolicy {
    /// Allow relay-assisted connections if direct fails
    pub allow_relays: bool,

    /// Support path migration/multi-path
    pub path_migration: bool,
}

/// Blob freshness information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessInfo {
    /// Creation/update timestamp (milliseconds since epoch)
    pub timestamp: u64,

    /// Suggested TTL in seconds
    pub ttl_seconds: u32,
}

/// Website content manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteManifestBlob {
    /// Blob format version
    #[serde(rename = "v")]
    pub version: u8,

    /// Index file path (e.g., "/index.html")
    #[serde(rename = "index")]
    pub index_file: String,

    /// Page/file entries
    #[serde(rename = "pages")]
    pub pages: Vec<PageEntry>,

    /// Overall integrity hash (blake3 of concatenated page hashes)
    #[serde(rename = "integrity")]
    pub integrity_hash: [u8; 32],

    /// Optional compression method for content chunks
    #[serde(rename = "compression", skip_serializing_if = "Option::is_none")]
    pub compression: Option<String>,
}

/// Individual page/file entry in site manifest
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageEntry {
    /// URL path (e.g., "/about.html", "/images/logo.png")
    pub path: String,

    /// Content ID of the page data
    pub content_id: ContentId,

    /// Size in bytes
    pub size_bytes: u32,

    /// MIME type
    pub mime_type: String,

    /// Content hash for integrity check
    pub content_hash: [u8; 32],
}

impl IdentityDescriptorBlob {
    /// Create a new identity descriptor blob (without signature)
    pub fn new(
        four_words: String,
        root_digest: [u8; 32],
        ml_dsa_public_key: Vec<u8>,
        ml_kem_public_key: Vec<u8>,
        transport_keys: Vec<TransportKeyEntry>,
        preferred_display_name: String,
    ) -> Self {
        // Validate display name length
        let display_name = if preferred_display_name.len() > 128 {
            preferred_display_name[..128].to_string()
        } else {
            preferred_display_name
        };

        Self {
            version: PROTOCOL_VERSION,
            four_words,
            root_digest,
            ml_dsa_public_key,
            ml_kem_public_key,
            transport_keys,
            preferred_display_name: display_name,
            continuity: None,
            site_manifest_cid: None,
            media: None,
            capabilities: None,
            signature: Vec::new(), // Will be filled by sign()
        }
    }

    /// Calculate the canonical CBOR bytes for signing
    /// This excludes the signature field itself
    pub fn signing_bytes(&self) -> Result<Vec<u8>, serde_cbor::Error> {
        // Create a copy without signature for signing
        let mut signing_copy = self.clone();
        signing_copy.signature = Vec::new();

        serde_cbor::to_vec(&signing_copy)
    }

    /// Sign the descriptor with ML-DSA private key
    pub fn sign(&mut self, secret_key: &MlDsaSecretKey) -> Result<(), String> {
        let signing_bytes = self
            .signing_bytes()
            .map_err(|e| format!("CBOR serialization error: {}", e))?;

        // Use saorsa-core's ML-DSA signing
        let signature = saorsa_core::quantum_crypto::ml_dsa_sign(secret_key, &signing_bytes)
            .map_err(|e| format!("Signing error: {}", e))?;

        self.signature = signature.as_bytes().to_vec();
        Ok(())
    }

    /// Verify the descriptor signature
    pub fn verify(&self) -> Result<bool, String> {
        if self.signature.is_empty() {
            return Err("No signature present".to_string());
        }

        let signing_bytes = self
            .signing_bytes()
            .map_err(|e| format!("CBOR serialization error: {}", e))?;

        // Parse the public key and signature
        let public_key = MlDsaPublicKey::from_bytes(&self.ml_dsa_public_key)
            .map_err(|e| format!("Invalid public key: {}", e))?;

        let signature = MlDsaSignature::from_bytes(&self.signature)
            .map_err(|e| format!("Invalid signature: {}", e))?;

        // Use saorsa-core's ML-DSA verification
        saorsa_core::quantum_crypto::ml_dsa_verify(&public_key, &signing_bytes, &signature)
            .map_err(|e| format!("Verification error: {}", e))
    }

    /// Serialize to canonical CBOR bytes
    pub fn to_cbor(&self) -> Result<Vec<u8>, serde_cbor::Error> {
        serde_cbor::to_vec(self)
    }

    /// Deserialize from CBOR bytes
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, serde_cbor::Error> {
        serde_cbor::from_slice(bytes)
    }

    /// Calculate content address (CID) of this blob
    pub fn content_address(&self) -> Result<ContentId, serde_cbor::Error> {
        let cbor_bytes = self.to_cbor()?;
        Ok(crate::dht_identity::key_derivation::derive_content_address(
            &cbor_bytes,
        ))
    }

    /// Validate the descriptor for correctness
    pub fn validate(&self) -> Result<(), String> {
        // Validate four words format
        crate::dht_identity::key_derivation::normalize_four_words(&self.four_words)?;

        // Validate display name length
        if self.preferred_display_name.len() > 128 {
            return Err("Display name too long (max 128 bytes)".to_string());
        }

        // Validate key sizes (approximate - exact sizes may vary)
        if self.ml_dsa_public_key.len() < 1800 || self.ml_dsa_public_key.len() > 2000 {
            return Err("Invalid ML-DSA public key size".to_string());
        }

        if self.ml_kem_public_key.len() < 1100 || self.ml_kem_public_key.len() > 1300 {
            return Err("Invalid ML-KEM public key size".to_string());
        }

        // Validate transport keys
        for key_entry in &self.transport_keys {
            if key_entry.spki.is_empty() {
                return Err("Empty SPKI in transport key".to_string());
            }

            let computed_peer_id = *blake3::hash(&key_entry.spki).as_bytes();
            if computed_peer_id != key_entry.peer_id {
                return Err("Invalid peer_id in transport key".to_string());
            }
        }

        // If continuity proof exists, basic validation
        if let Some(continuity) = &self.continuity {
            if continuity.rotation_signature.is_empty() {
                return Err("Empty rotation signature in continuity proof".to_string());
            }
        }

        Ok(())
    }
}

impl ConnectionBlob {
    /// Create a new connection blob
    pub fn new(
        peer_transport_id: [u8; 32],
        rendezvous_nodes: Vec<RendezvousEntry>,
        timestamp: u64,
        ttl_seconds: u32,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            peer_transport_id,
            rendezvous_nodes,
            relay_hints: None,
            policy: ConnectionPolicy {
                allow_relays: true,
                path_migration: true,
            },
            freshness: FreshnessInfo {
                timestamp,
                ttl_seconds,
            },
        }
    }

    /// Serialize to canonical CBOR bytes
    pub fn to_cbor(&self) -> Result<Vec<u8>, serde_cbor::Error> {
        serde_cbor::to_vec(self)
    }

    /// Deserialize from CBOR bytes
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, serde_cbor::Error> {
        serde_cbor::from_slice(bytes)
    }

    /// Calculate content address (CID) of this blob
    pub fn content_address(&self) -> Result<ContentId, serde_cbor::Error> {
        let cbor_bytes = self.to_cbor()?;
        Ok(crate::dht_identity::key_derivation::derive_content_address(
            &cbor_bytes,
        ))
    }
}

impl SiteManifestBlob {
    /// Create a new site manifest blob
    pub fn new(index_file: String, pages: Vec<PageEntry>) -> Result<Self, String> {
        // Calculate total size
        let total_size: u64 = pages.iter().map(|p| p.size_bytes as u64).sum();

        if total_size > MAX_WEB_CONTENT_SIZE as u64 {
            return Err(format!(
                "Total content size {} exceeds limit {}",
                total_size, MAX_WEB_CONTENT_SIZE
            ));
        }

        // Calculate integrity hash
        let mut hasher = Hasher::new();
        for page in &pages {
            hasher.update(&page.content_hash);
        }
        let integrity_hash = hasher.finalize().into();

        Ok(Self {
            version: PROTOCOL_VERSION,
            index_file,
            pages,
            integrity_hash,
            compression: None,
        })
    }

    /// Serialize to canonical CBOR bytes
    pub fn to_cbor(&self) -> Result<Vec<u8>, serde_cbor::Error> {
        serde_cbor::to_vec(self)
    }

    /// Deserialize from CBOR bytes
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, serde_cbor::Error> {
        serde_cbor::from_slice(bytes)
    }

    /// Calculate content address (CID) of this blob
    pub fn content_address(&self) -> Result<ContentId, serde_cbor::Error> {
        let cbor_bytes = self.to_cbor()?;
        Ok(crate::dht_identity::key_derivation::derive_content_address(
            &cbor_bytes,
        ))
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), String> {
        if self.index_file.is_empty() {
            return Err("Index file path cannot be empty".to_string());
        }

        if self.pages.is_empty() {
            return Err("Site must have at least one page".to_string());
        }

        // Check total size
        let total_size: u64 = self.pages.iter().map(|p| p.size_bytes as u64).sum();
        if total_size > MAX_WEB_CONTENT_SIZE as u64 {
            return Err(format!(
                "Total content size {} exceeds limit {}",
                total_size, MAX_WEB_CONTENT_SIZE
            ));
        }

        // Validate integrity hash
        let mut hasher = Hasher::new();
        for page in &self.pages {
            hasher.update(&page.content_hash);
        }
        let expected_integrity: [u8; 32] = hasher.finalize().into();

        if expected_integrity != self.integrity_hash {
            return Err("Integrity hash mismatch".to_string());
        }

        // Check for duplicate paths
        let mut paths = std::collections::HashSet::new();
        for page in &self.pages {
            if !paths.insert(&page.path) {
                return Err(format!("Duplicate page path: {}", page.path));
            }
        }

        // Ensure index file exists
        if !self.pages.iter().any(|p| p.path == self.index_file) {
            return Err(format!("Index file {} not found in pages", self.index_file));
        }

        Ok(())
    }

    /// Get total content size in bytes
    pub fn total_size(&self) -> u64 {
        self.pages.iter().map(|p| p.size_bytes as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transport_key() -> TransportKeyEntry {
        let spki = b"test_spki_data".to_vec();
        let peer_id = *blake3::hash(&spki).as_bytes();

        TransportKeyEntry {
            spki,
            algorithm: "Ed25519".to_string(),
            peer_id,
        }
    }

    fn create_test_identity_descriptor() -> IdentityDescriptorBlob {
        IdentityDescriptorBlob::new(
            "ocean-forest-moon-star".to_string(),
            [1u8; 32],       // root_digest
            vec![2u8; 1952], // ml_dsa_public_key (approximate size)
            vec![3u8; 1184], // ml_kem_public_key (approximate size)
            vec![create_test_transport_key()],
            "Test User".to_string(),
        )
    }

    #[test]
    fn test_identity_descriptor_creation() {
        let descriptor = create_test_identity_descriptor();

        assert_eq!(descriptor.version, PROTOCOL_VERSION);
        assert_eq!(descriptor.four_words, "ocean-forest-moon-star");
        assert_eq!(descriptor.root_digest, [1u8; 32]);
        assert_eq!(descriptor.preferred_display_name, "Test User");
        assert_eq!(descriptor.transport_keys.len(), 1);
        assert!(descriptor.signature.is_empty()); // Not signed yet
    }

    #[test]
    fn test_identity_descriptor_validation() {
        let descriptor = create_test_identity_descriptor();
        descriptor.validate().expect("Should be valid");

        // Test invalid four words
        let mut invalid = descriptor.clone();
        invalid.four_words = "invalid".to_string();
        assert!(invalid.validate().is_err());

        // Test long display name - should fail validation when set directly
        let mut long_name = descriptor.clone();
        long_name.preferred_display_name = "x".repeat(129);
        assert!(
            long_name.validate().is_err(),
            "Should fail validation for long display name"
        );
    }

    #[test]
    fn test_identity_descriptor_cbor_serialization() {
        let descriptor = create_test_identity_descriptor();

        let cbor_bytes = descriptor.to_cbor().expect("Should serialize");
        assert!(!cbor_bytes.is_empty());

        let deserialized =
            IdentityDescriptorBlob::from_cbor(&cbor_bytes).expect("Should deserialize");
        assert_eq!(descriptor, deserialized);
    }

    #[test]
    fn test_identity_descriptor_content_address() {
        let descriptor = create_test_identity_descriptor();

        let cid1 = descriptor.content_address().expect("Should compute CID");
        let cid2 = descriptor.content_address().expect("Should compute CID");

        // Same content should produce same CID
        assert_eq!(cid1, cid2);

        // Different content should produce different CID
        let mut different = descriptor.clone();
        different.preferred_display_name = "Different User".to_string();
        let cid3 = different.content_address().expect("Should compute CID");
        assert_ne!(cid1, cid3);
    }

    #[test]
    fn test_identity_descriptor_signing_bytes() {
        let descriptor = create_test_identity_descriptor();

        let signing_bytes = descriptor
            .signing_bytes()
            .expect("Should get signing bytes");
        assert!(!signing_bytes.is_empty());

        // Signing bytes should be deterministic
        let signing_bytes2 = descriptor
            .signing_bytes()
            .expect("Should get signing bytes");
        assert_eq!(signing_bytes, signing_bytes2);

        // Adding signature should not affect signing bytes
        let mut signed = descriptor.clone();
        signed.signature = vec![99u8; 100];
        let signing_bytes3 = signed.signing_bytes().expect("Should get signing bytes");
        assert_eq!(signing_bytes, signing_bytes3);
    }

    #[test]
    fn test_connection_blob_creation() {
        let rendezvous = vec![
            RendezvousEntry {
                address: "bootstrap1.example.com:9000".to_string(),
                priority: 1,
            },
            RendezvousEntry {
                address: "bootstrap2.example.com:9000".to_string(),
                priority: 2,
            },
        ];

        let blob = ConnectionBlob::new(
            [4u8; 32], // transport_id
            rendezvous,
            1640995200000, // timestamp
            3600,          // ttl
        );

        assert_eq!(blob.version, PROTOCOL_VERSION);
        assert_eq!(blob.peer_transport_id, [4u8; 32]);
        assert_eq!(blob.rendezvous_nodes.len(), 2);
        assert_eq!(blob.freshness.timestamp, 1640995200000);
        assert_eq!(blob.freshness.ttl_seconds, 3600);
        assert!(blob.policy.allow_relays);
        assert!(blob.policy.path_migration);
    }

    #[test]
    fn test_connection_blob_serialization() {
        let blob = ConnectionBlob::new(
            [4u8; 32],
            vec![RendezvousEntry {
                address: "test.example.com:9000".to_string(),
                priority: 1,
            }],
            1640995200000,
            3600,
        );

        let cbor_bytes = blob.to_cbor().expect("Should serialize");
        let deserialized = ConnectionBlob::from_cbor(&cbor_bytes).expect("Should deserialize");
        assert_eq!(blob, deserialized);

        let cid = blob.content_address().expect("Should compute CID");
        assert_eq!(cid.len(), 32);
    }

    #[test]
    fn test_site_manifest_creation() {
        let pages = vec![
            PageEntry {
                path: "/index.html".to_string(),
                content_id: [1u8; 32],
                size_bytes: 1024,
                mime_type: "text/html".to_string(),
                content_hash: [10u8; 32],
            },
            PageEntry {
                path: "/about.html".to_string(),
                content_id: [2u8; 32],
                size_bytes: 512,
                mime_type: "text/html".to_string(),
                content_hash: [20u8; 32],
            },
        ];

        let manifest = SiteManifestBlob::new("/index.html".to_string(), pages)
            .expect("Should create manifest");

        assert_eq!(manifest.version, PROTOCOL_VERSION);
        assert_eq!(manifest.index_file, "/index.html");
        assert_eq!(manifest.pages.len(), 2);
        assert_eq!(manifest.total_size(), 1536);
    }

    #[test]
    fn test_site_manifest_validation() {
        let pages = vec![PageEntry {
            path: "/index.html".to_string(),
            content_id: [1u8; 32],
            size_bytes: 1024,
            mime_type: "text/html".to_string(),
            content_hash: [10u8; 32],
        }];

        let manifest = SiteManifestBlob::new("/index.html".to_string(), pages)
            .expect("Should create manifest");

        manifest.validate().expect("Should be valid");

        // Test missing index file
        let mut invalid = manifest.clone();
        invalid.index_file = "/missing.html".to_string();
        assert!(invalid.validate().is_err());

        // Test duplicate paths
        let duplicate_pages = vec![
            PageEntry {
                path: "/test.html".to_string(),
                content_id: [1u8; 32],
                size_bytes: 100,
                mime_type: "text/html".to_string(),
                content_hash: [10u8; 32],
            },
            PageEntry {
                path: "/test.html".to_string(), // Duplicate!
                content_id: [2u8; 32],
                size_bytes: 200,
                mime_type: "text/html".to_string(),
                content_hash: [20u8; 32],
            },
        ];

        let duplicate_manifest = SiteManifestBlob::new("/test.html".to_string(), duplicate_pages);
        assert!(duplicate_manifest.is_ok()); // Created successfully
        assert!(duplicate_manifest.unwrap().validate().is_err()); // But validation fails
    }

    #[test]
    fn test_site_manifest_size_limit() {
        let large_page = PageEntry {
            path: "/large.bin".to_string(),
            content_id: [1u8; 32],
            size_bytes: (MAX_WEB_CONTENT_SIZE + 1) as u32, // Exceed limit
            mime_type: "application/octet-stream".to_string(),
            content_hash: [10u8; 32],
        };

        let result = SiteManifestBlob::new("/large.bin".to_string(), vec![large_page]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds limit"));
    }

    #[test]
    fn test_transport_key_validation() {
        let spki = b"test_spki_data".to_vec();
        let correct_peer_id = *blake3::hash(&spki).as_bytes();

        // Valid transport key
        let valid_key = TransportKeyEntry {
            spki: spki.clone(),
            algorithm: "Ed25519".to_string(),
            peer_id: correct_peer_id,
        };

        let mut descriptor = create_test_identity_descriptor();
        descriptor.transport_keys = vec![valid_key];
        descriptor.validate().expect("Should be valid");

        // Invalid peer_id
        let invalid_key = TransportKeyEntry {
            spki,
            algorithm: "Ed25519".to_string(),
            peer_id: [99u8; 32], // Wrong peer_id
        };

        descriptor.transport_keys = vec![invalid_key];
        assert!(descriptor.validate().is_err());
    }

    #[test]
    fn test_display_name_truncation() {
        let long_name = "x".repeat(200);
        let descriptor = IdentityDescriptorBlob::new(
            "ocean-forest-moon-star".to_string(),
            [1u8; 32],
            vec![2u8; 1952],
            vec![3u8; 1184],
            vec![create_test_transport_key()],
            long_name,
        );

        assert_eq!(descriptor.preferred_display_name.len(), 128);
        descriptor
            .validate()
            .expect("Should be valid after truncation");
    }
}
