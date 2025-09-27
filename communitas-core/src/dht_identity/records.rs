//! DHT record structures for identity system
//!
//! This module defines the small (≤512B) records stored directly in the DHT:
//! - IdentityRootRecord: Main identity pointer with hashes and metadata
//! - ConnectionRecord: Fast-updating connectivity information
//! - SiteManifestRecord: Website content manifest pointer

use crate::dht_identity::{DhtKey, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Content Identifier - 32-byte BLAKE3 hash
pub type ContentId = [u8; 32];

/// ML-DSA public key hash (32 bytes)
pub type MlDsaKeyHash = [u8; 32];

/// ML-KEM public key hash (32 bytes) 
pub type MlKemKeyHash = [u8; 32];

/// Transport ID from ant-quic (32 bytes)
pub type TransportId = [u8; 32];

/// Identity hash binding (32 bytes)
pub type IdentityHash = [u8; 32];

/// Display name hash (32 bytes)
pub type DisplayNameHash = [u8; 32];

/// Main DHT record that binds four-word identity to descriptor blob
/// Size: ~280 bytes (well under 512B limit)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityRootRecord {
    /// Record format version
    #[serde(rename = "v")]
    pub version: u8,
    
    /// Monotonic sequence number for updates
    #[serde(rename = "seq")]
    pub sequence: u32,
    
    /// Created/updated timestamp (milliseconds since epoch)
    #[serde(rename = "ts")]
    pub timestamp: u64,
    
    /// Identity hash = blake3(normalized_four_words) for binding
    #[serde(rename = "idh")]
    pub identity_hash: IdentityHash,
    
    /// Hash of ML-DSA public key
    #[serde(rename = "dpkh")]
    pub ml_dsa_key_hash: MlDsaKeyHash,
    
    /// Hash of ML-KEM public key
    #[serde(rename = "kpkh")]
    pub ml_kem_key_hash: MlKemKeyHash,
    
    /// Transport ID from ant-quic for channel binding
    #[serde(rename = "tpid")]
    pub transport_id: TransportId,
    
    /// Content ID of IdentityDescriptorBlob
    #[serde(rename = "desc")]
    pub descriptor_cid: ContentId,
    
    /// Optional: Content ID of current ConnectionBlob
    #[serde(rename = "conn", skip_serializing_if = "Option::is_none")]
    pub connection_cid: Option<ContentId>,
    
    /// Optional: Content ID of current SiteManifestBlob
    #[serde(rename = "site", skip_serializing_if = "Option::is_none")]
    pub site_cid: Option<ContentId>,
    
    /// Optional: Hash of preferred display name (hint only)
    #[serde(rename = "dnh", skip_serializing_if = "Option::is_none")]
    pub display_name_hash: Option<DisplayNameHash>,
    
    /// Optional: Time-to-live hint in seconds
    #[serde(rename = "ttl", skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u32>,
    
    /// Optional: Additional metadata (reserved for future use)
    #[serde(rename = "meta", skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, serde_cbor::Value>>,
}

/// Fast-updating connection information pointer
/// Size: ~160 bytes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionRecord {
    /// Record format version
    #[serde(rename = "v")]
    pub version: u8,
    
    /// Independent sequence number (frequent updates)
    #[serde(rename = "seq")]
    pub sequence: u32,
    
    /// Updated timestamp
    #[serde(rename = "ts")]
    pub timestamp: u64,
    
    /// Transport ID for consistency check
    #[serde(rename = "tpid")]
    pub transport_id: TransportId,
    
    /// Content ID of ConnectionBlob
    #[serde(rename = "conn")]
    pub connection_cid: ContentId,
    
    /// Time-to-live hint in seconds
    #[serde(rename = "ttl")]
    pub ttl_seconds: u32,
}

/// Website content manifest pointer
/// Size: ~120 bytes  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteManifestRecord {
    /// Record format version
    #[serde(rename = "v")]
    pub version: u8,
    
    /// Monotonic sequence number
    #[serde(rename = "seq")]
    pub sequence: u32,
    
    /// Updated timestamp
    #[serde(rename = "ts")]
    pub timestamp: u64,
    
    /// Content ID of SiteManifestBlob
    #[serde(rename = "site")]
    pub site_cid: ContentId,
    
    /// Time-to-live hint in seconds
    #[serde(rename = "ttl")]
    pub ttl_seconds: u32,
}

impl IdentityRootRecord {
    /// Create a new identity root record
    pub fn new(
        sequence: u32,
        timestamp: u64,
        identity_hash: IdentityHash,
        ml_dsa_key_hash: MlDsaKeyHash,
        ml_kem_key_hash: MlKemKeyHash,
        transport_id: TransportId,
        descriptor_cid: ContentId,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            sequence,
            timestamp,
            identity_hash,
            ml_dsa_key_hash,
            ml_kem_key_hash,
            transport_id,
            descriptor_cid,
            connection_cid: None,
            site_cid: None,
            display_name_hash: None,
            ttl_seconds: None,
            metadata: None,
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
    
    /// Calculate hash of the core identity fields (excluding descriptor_cid and optional fields)
    /// This is used for binding the descriptor blob to the root record without circular dependency
    pub fn core_hash(&self) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        
        // Domain separation for root core digest
        hasher.update(b"root-core-v1");
        
        // Hash the core identifying fields in deterministic order
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.sequence.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&self.identity_hash);
        hasher.update(&self.ml_dsa_key_hash);
        hasher.update(&self.ml_kem_key_hash);
        hasher.update(&self.transport_id);
        
        hasher.finalize().into()
    }
    
    /// Calculate full content hash binding core content with descriptor CID
    /// This provides the complete binding of root record to descriptor
    pub fn content_hash(&self) -> [u8; 32] {
        use blake3::Hasher;
        let mut hasher = Hasher::new();
        
        // Domain separation for root binding
        hasher.update(b"root-bind-v1");
        
        let core_hash = self.core_hash();
        hasher.update(&core_hash);
        hasher.update(&self.descriptor_cid);
        
        hasher.finalize().into()
    }
    
    /// Check if record size is within DHT limits
    pub fn validate_size(&self) -> Result<(), String> {
        let cbor_size = self.to_cbor()
            .map_err(|e| format!("CBOR serialization error: {}", e))?
            .len();
            
        if cbor_size > super::DHT_RECORD_MAX_SIZE {
            return Err(format!(
                "Record too large: {} bytes (max: {})", 
                cbor_size, 
                super::DHT_RECORD_MAX_SIZE
            ));
        }
        
        Ok(())
    }
}

impl ConnectionRecord {
    /// Create a new connection record
    pub fn new(
        sequence: u32,
        timestamp: u64,
        transport_id: TransportId,
        connection_cid: ContentId,
        ttl_seconds: u32,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            sequence,
            timestamp,
            transport_id,
            connection_cid,
            ttl_seconds,
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
    
    /// Check if record size is within DHT limits
    pub fn validate_size(&self) -> Result<(), String> {
        let cbor_size = self.to_cbor()
            .map_err(|e| format!("CBOR serialization error: {}", e))?
            .len();
            
        if cbor_size > super::DHT_RECORD_MAX_SIZE {
            return Err(format!(
                "Record too large: {} bytes (max: {})", 
                cbor_size, 
                super::DHT_RECORD_MAX_SIZE
            ));
        }
        
        Ok(())
    }
}

impl SiteManifestRecord {
    /// Create a new site manifest record
    pub fn new(
        sequence: u32,
        timestamp: u64,
        site_cid: ContentId,
        ttl_seconds: u32,
    ) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            sequence,
            timestamp,
            site_cid,
            ttl_seconds,
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
    
    /// Check if record size is within DHT limits
    pub fn validate_size(&self) -> Result<(), String> {
        let cbor_size = self.to_cbor()
            .map_err(|e| format!("CBOR serialization error: {}", e))?
            .len();
            
        if cbor_size > super::DHT_RECORD_MAX_SIZE {
            return Err(format!(
                "Record too large: {} bytes (max: {})", 
                cbor_size, 
                super::DHT_RECORD_MAX_SIZE
            ));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dht_identity::key_derivation::*;

    fn create_test_identity_root() -> IdentityRootRecord {
        IdentityRootRecord::new(
            1,                                    // sequence
            1640995200000,                        // timestamp (Jan 1, 2022)
            [1u8; 32],                           // identity_hash
            [2u8; 32],                           // ml_dsa_key_hash
            [3u8; 32],                           // ml_kem_key_hash
            [4u8; 32],                           // transport_id
            [5u8; 32],                           // descriptor_cid
        )
    }

    fn create_test_connection_record() -> ConnectionRecord {
        ConnectionRecord::new(
            1,                                    // sequence
            1640995200000,                        // timestamp
            [4u8; 32],                           // transport_id
            [6u8; 32],                           // connection_cid
            3600,                                // ttl_seconds
        )
    }

    fn create_test_site_record() -> SiteManifestRecord {
        SiteManifestRecord::new(
            1,                                    // sequence
            1640995200000,                        // timestamp
            [7u8; 32],                           // site_cid
            7200,                                // ttl_seconds
        )
    }

    #[test]
    fn test_identity_root_record_creation() {
        let record = create_test_identity_root();
        
        assert_eq!(record.version, PROTOCOL_VERSION);
        assert_eq!(record.sequence, 1);
        assert_eq!(record.timestamp, 1640995200000);
        assert_eq!(record.identity_hash, [1u8; 32]);
        assert_eq!(record.ml_dsa_key_hash, [2u8; 32]);
        assert_eq!(record.ml_kem_key_hash, [3u8; 32]);
        assert_eq!(record.transport_id, [4u8; 32]);
        assert_eq!(record.descriptor_cid, [5u8; 32]);
        assert_eq!(record.connection_cid, None);
        assert_eq!(record.site_cid, None);
        assert_eq!(record.display_name_hash, None);
        assert_eq!(record.ttl_seconds, None);
        assert_eq!(record.metadata, None);
    }

    #[test]
    fn test_identity_root_record_cbor_serialization() {
        let record = create_test_identity_root();
        
        // Test serialization
        let cbor_bytes = record.to_cbor().expect("Should serialize");
        assert!(!cbor_bytes.is_empty());
        
        // Test deserialization
        let deserialized = IdentityRootRecord::from_cbor(&cbor_bytes)
            .expect("Should deserialize");
        assert_eq!(record, deserialized);
    }

    #[test]
    fn test_identity_root_record_size_validation() {
        let record = create_test_identity_root();
        
        // Should be within size limits
        record.validate_size().expect("Should be within limits");
        
        // Check actual size
        let cbor_size = record.to_cbor().unwrap().len();
        println!("IdentityRootRecord CBOR size: {} bytes", cbor_size);
        assert!(cbor_size <= super::super::DHT_RECORD_MAX_SIZE);
        assert!(cbor_size < 300); // Should be well under limit
    }

    #[test] 
    fn test_identity_root_record_content_hash() {
        let record1 = create_test_identity_root();
        let mut record2 = record1.clone();
        
        // Same content should produce same hash
        assert_eq!(record1.content_hash(), record2.content_hash());
        assert_eq!(record1.core_hash(), record2.core_hash());
        
        // Different sequence should produce different hash
        record2.sequence = 2;
        assert_ne!(record1.content_hash(), record2.content_hash());
        assert_ne!(record1.core_hash(), record2.core_hash());
        
        // Optional fields should not affect core hash
        let mut record3 = record1.clone();
        record3.connection_cid = Some([99u8; 32]);
        record3.ttl_seconds = Some(3600);
        assert_eq!(record1.core_hash(), record3.core_hash());
        
        // But different descriptor_cid should affect content hash
        let mut record4 = record1.clone();
        record4.descriptor_cid = [99u8; 32];
        assert_ne!(record1.content_hash(), record4.content_hash());
        assert_eq!(record1.core_hash(), record4.core_hash()); // Core hash unchanged
    }

    #[test]
    fn test_connection_record_operations() {
        let record = create_test_connection_record();
        
        // Test properties
        assert_eq!(record.version, PROTOCOL_VERSION);
        assert_eq!(record.sequence, 1);
        assert_eq!(record.transport_id, [4u8; 32]);
        assert_eq!(record.connection_cid, [6u8; 32]);
        assert_eq!(record.ttl_seconds, 3600);
        
        // Test serialization round-trip
        let cbor_bytes = record.to_cbor().expect("Should serialize");
        let deserialized = ConnectionRecord::from_cbor(&cbor_bytes)
            .expect("Should deserialize");
        assert_eq!(record, deserialized);
        
        // Test size validation
        record.validate_size().expect("Should be within limits");
        
        let cbor_size = cbor_bytes.len();
        println!("ConnectionRecord CBOR size: {} bytes", cbor_size);
        assert!(cbor_size <= super::super::DHT_RECORD_MAX_SIZE);
        assert!(cbor_size < 200); // Should be quite small
    }

    #[test]
    fn test_site_manifest_record_operations() {
        let record = create_test_site_record();
        
        // Test properties
        assert_eq!(record.version, PROTOCOL_VERSION);
        assert_eq!(record.sequence, 1);
        assert_eq!(record.site_cid, [7u8; 32]);
        assert_eq!(record.ttl_seconds, 7200);
        
        // Test serialization round-trip
        let cbor_bytes = record.to_cbor().expect("Should serialize");
        let deserialized = SiteManifestRecord::from_cbor(&cbor_bytes)
            .expect("Should deserialize");
        assert_eq!(record, deserialized);
        
        // Test size validation
        record.validate_size().expect("Should be within limits");
        
        let cbor_size = cbor_bytes.len();
        println!("SiteManifestRecord CBOR size: {} bytes", cbor_size);
        assert!(cbor_size <= super::super::DHT_RECORD_MAX_SIZE);
        assert!(cbor_size < 150); // Should be the smallest record
    }

    #[test] 
    fn test_records_with_optional_fields() {
        let mut record = create_test_identity_root();
        
        // Add optional fields
        record.connection_cid = Some([6u8; 32]);
        record.site_cid = Some([7u8; 32]);
        record.display_name_hash = Some([8u8; 32]);
        record.ttl_seconds = Some(3600);
        
        // Create metadata
        let mut metadata = BTreeMap::new();
        metadata.insert("test_key".to_string(), serde_cbor::Value::Text("test_value".to_string()));
        record.metadata = Some(metadata);
        
        // Should still be within size limits
        record.validate_size().expect("Should be within limits even with optional fields");
        
        // Test serialization
        let cbor_bytes = record.to_cbor().expect("Should serialize");
        let deserialized = IdentityRootRecord::from_cbor(&cbor_bytes)
            .expect("Should deserialize");
        assert_eq!(record, deserialized);
        
        let cbor_size = cbor_bytes.len();
        println!("IdentityRootRecord with optional fields CBOR size: {} bytes", cbor_size);
        assert!(cbor_size <= super::super::DHT_RECORD_MAX_SIZE);
    }

    #[test]
    fn test_cbor_deterministic_serialization() {
        let record = create_test_identity_root();
        
        // Multiple serializations should produce identical bytes
        let bytes1 = record.to_cbor().unwrap();
        let bytes2 = record.to_cbor().unwrap();
        
        assert_eq!(bytes1, bytes2, "CBOR serialization should be deterministic");
    }

    #[test]
    fn test_records_with_real_four_word_keys() {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();
        
        let id_key = derive_identity_key(&four_words);
        let conn_key = derive_connection_key(&four_words);
        let site_key = derive_site_key(&four_words);
        
        // Create records with real derived keys as test data
        let mut root_record = create_test_identity_root();
        root_record.identity_hash = *blake3::hash(four_words.as_str().as_bytes()).as_bytes();
        
        let conn_record = ConnectionRecord::new(1, 1640995200000, [4u8; 32], [6u8; 32], 3600);
        let site_record = SiteManifestRecord::new(1, 1640995200000, [7u8; 32], 7200);
        
        // All should serialize properly
        let _root_cbor = root_record.to_cbor().unwrap();
        let _conn_cbor = conn_record.to_cbor().unwrap();
        let _site_cbor = site_record.to_cbor().unwrap();
        
        println!("DHT keys for 'ocean-forest-moon-star':");
        println!("  ID:   {:02x?}", id_key);
        println!("  Conn: {:02x?}", conn_key);
        println!("  Site: {:02x?}", site_key);
    }
}
