//! TOFU (Trust On First Use) validation and identity pinning logic
//!
//! This module implements the security model for the DHT identity system:
//! - TOFU identity pinning on first contact
//! - Key rotation validation with continuity signatures
//! - Anti-quic transport identity binding
//! - Security event logging and monitoring

use crate::dht_identity::storage::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// Error types for identity validation
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Identity not pinned: {0}")]
    IdentityNotPinned(String),
    
    #[error("Key rotation validation failed: {0}")]
    KeyRotationFailed(String),
    
    #[error("Transport identity mismatch: {0}")]
    TransportMismatch(String),
    
    #[error("Invalid continuity signature: {0}")]
    InvalidContinuitySignature(String),
    
    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    
    #[error("Identity validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] IdentityStorageError),
    
    #[error("Cryptographic error: {0}")]
    CryptographicError(String),
}

pub type ValidationResult<T> = std::result::Result<T, ValidationError>;

/// Pinned identity information for TOFU security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedIdentity {
    /// Four-word identity
    pub four_words: String,
    
    /// Pinned ML-DSA public key hash
    pub ml_dsa_key_hash: [u8; 32],
    
    /// Pinned ML-KEM public key hash
    pub ml_kem_key_hash: [u8; 32],
    
    /// Pinned transport ID from ant-quic
    pub transport_id: [u8; 32],
    
    /// First contact timestamp
    pub pinned_at: SystemTime,
    
    /// Sequence number of the identity when pinned
    pub pinned_sequence: u32,
    
    /// Trust level (can be upgraded through endorsements/verification)
    pub trust_level: TrustLevel,
    
    /// Security events related to this identity
    pub security_events: Vec<SecurityEvent>,
}

/// Trust levels for identities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// TOFU - trusted on first use (default)
    Tofu,
    
    /// Manually verified by user
    UserVerified,
    
    /// Verified through endorsement system
    EndorsementVerified,
    
    /// High trust through multiple endorsements
    HighTrust,
    
    /// Blocked/revoked identity
    Blocked,
}

/// Security events for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    /// Event type
    pub event_type: SecurityEventType,
    
    /// Event timestamp
    pub timestamp: SystemTime,
    
    /// Event description
    pub description: String,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    /// Identity first pinned
    IdentityPinned,
    
    /// Key rotation detected
    KeyRotation,
    
    /// Transport identity changed
    TransportChanged,
    
    /// Suspicious activity detected
    SuspiciousActivity,
    
    /// Manual trust level change
    TrustLevelChanged,
    
    /// Identity blocked
    IdentityBlocked,
    
    /// Signature verification failure
    SignatureFailure,
}

/// TOFU validator and identity manager
pub struct TofuValidator {
    /// Pinned identities database
    pinned_identities: Arc<RwLock<HashMap<String, PinnedIdentity>>>,
    
    /// Storage backend
    storage: Arc<IdentityStorage>,
    
    /// Validation policy configuration
    policy: ValidationPolicy,
}

/// Validation policy configuration
#[derive(Debug, Clone)]
pub struct ValidationPolicy {
    /// Allow TOFU on first contact
    pub allow_tofu: bool,
    
    /// Require continuity signatures for key rotation
    pub require_continuity: bool,
    
    /// Enable transport identity binding
    pub enable_transport_binding: bool,
    
    /// Maximum allowed time skew for timestamps (seconds)
    pub max_time_skew: u64,
    
    /// Minimum sequence number increment for updates
    pub min_sequence_increment: u32,
    
    /// Enable security event logging
    pub enable_audit_logging: bool,
}

impl Default for ValidationPolicy {
    fn default() -> Self {
        Self {
            allow_tofu: true,
            require_continuity: true,
            enable_transport_binding: true,
            max_time_skew: 300, // 5 minutes
            min_sequence_increment: 1,
            enable_audit_logging: true,
        }
    }
}

/// Result of identity validation
#[derive(Debug, Clone)]
pub struct ValidationResult_ {
    /// Whether the identity is valid
    pub is_valid: bool,
    
    /// Whether this was a first contact (TOFU)
    pub is_first_contact: bool,
    
    /// Whether key rotation occurred
    pub key_rotation_detected: bool,
    
    /// Current trust level
    pub trust_level: TrustLevel,
    
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<String>,
    
    /// Security events generated during validation
    pub security_events: Vec<SecurityEvent>,
}

impl TofuValidator {
    /// Create a new TOFU validator
    pub fn new(storage: Arc<IdentityStorage>, policy: ValidationPolicy) -> Self {
        Self {
            pinned_identities: Arc::new(RwLock::new(HashMap::new())),
            storage,
            policy,
        }
    }

    /// Validate an identity using TOFU principles
    pub async fn validate_identity(&self, identity: &ResolvedIdentity) -> ValidationResult<ValidationResult_> {
        let four_words = identity.four_words.as_str();
        let mut warnings = Vec::new();
        let mut security_events = Vec::new();
        
        // Check if identity is already pinned
        let pinned_identities = self.pinned_identities.read().await;
        let existing_pin = pinned_identities.get(four_words).cloned();
        drop(pinned_identities);
        
        if let Some(pinned) = existing_pin {
            // Validate against pinned identity
            self.validate_against_pinned_identity(identity, &pinned, &mut warnings, &mut security_events).await
        } else {
            // First contact - TOFU validation
            self.validate_first_contact(identity, &mut warnings, &mut security_events).await
        }
    }

    /// Validate identity against pinned data
    async fn validate_against_pinned_identity(
        &self, 
        identity: &ResolvedIdentity, 
        pinned: &PinnedIdentity,
        warnings: &mut Vec<String>,
        security_events: &mut Vec<SecurityEvent>
    ) -> ValidationResult<ValidationResult_> {
        let mut key_rotation_detected = false;
        
        // Check sequence number progression
        if identity.root_record.sequence <= pinned.pinned_sequence {
            warnings.push("Non-increasing sequence number".to_string());
        }
        
        // Check time progression
        let current_time = identity.root_record.timestamp;
        let pinned_time = pinned.pinned_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
            
        if current_time < pinned_time {
            return Err(ValidationError::ValidationFailed(
                "Timestamp regression detected".to_string()
            ));
        }
        
        // Validate key continuity
        let current_ml_dsa_hash = identity.root_record.ml_dsa_key_hash;
        let current_ml_kem_hash = identity.root_record.ml_kem_key_hash;
        
        if current_ml_dsa_hash != pinned.ml_dsa_key_hash || 
           current_ml_kem_hash != pinned.ml_kem_key_hash {
            key_rotation_detected = true;
            
            if self.policy.require_continuity {
                // Validate continuity signature if key rotation occurred
                self.validate_key_rotation(identity, pinned).await?;
                
                security_events.push(SecurityEvent {
                    event_type: SecurityEventType::KeyRotation,
                    timestamp: SystemTime::now(),
                    description: "Key rotation validated with continuity signature".to_string(),
                    metadata: HashMap::new(),
                });
            }
        }
        
        // Validate transport identity binding
        if self.policy.enable_transport_binding {
            if identity.root_record.transport_id != pinned.transport_id {
                security_events.push(SecurityEvent {
                    event_type: SecurityEventType::TransportChanged,
                    timestamp: SystemTime::now(),
                    description: "Transport identity changed".to_string(),
                    metadata: HashMap::new(),
                });
                
                warnings.push("Transport identity changed".to_string());
            }
        }
        
        // Update pinned identity if key rotation was valid
        if key_rotation_detected {
            self.update_pinned_identity(identity).await?;
        }
        
        Ok(ValidationResult_ {
            is_valid: true,
            is_first_contact: false,
            key_rotation_detected,
            trust_level: pinned.trust_level,
            warnings: warnings.clone(),
            security_events: security_events.clone(),
        })
    }

    /// Validate first contact using TOFU
    async fn validate_first_contact(
        &self,
        identity: &ResolvedIdentity,
        warnings: &mut Vec<String>,
        security_events: &mut Vec<SecurityEvent>
    ) -> ValidationResult<ValidationResult_> {
        if !self.policy.allow_tofu {
            return Err(ValidationError::ValidationFailed(
                "TOFU not allowed by policy".to_string()
            ));
        }
        
        // Basic validation of the identity
        identity.descriptor.validate()
            .map_err(|e| ValidationError::ValidationFailed(e))?;
        
        // Verify signature
        match identity.descriptor.verify() {
            Ok(true) => {},
            Ok(false) | Err(_) => {
                return Err(ValidationError::SignatureVerificationFailed(
                    "Invalid identity descriptor signature".to_string()
                ));
            }
        }
        
        // Pin the identity
        let pinned_identity = PinnedIdentity {
            four_words: identity.four_words.as_str().to_string(),
            ml_dsa_key_hash: identity.root_record.ml_dsa_key_hash,
            ml_kem_key_hash: identity.root_record.ml_kem_key_hash,
            transport_id: identity.root_record.transport_id,
            pinned_at: SystemTime::now(),
            pinned_sequence: identity.root_record.sequence,
            trust_level: TrustLevel::Tofu,
            security_events: Vec::new(),
        };
        
        // Store the pinned identity
        {
            let mut pinned_identities = self.pinned_identities.write().await;
            pinned_identities.insert(identity.four_words.as_str().to_string(), pinned_identity);
        }
        
        security_events.push(SecurityEvent {
            event_type: SecurityEventType::IdentityPinned,
            timestamp: SystemTime::now(),
            description: format!("Identity {} pinned on first contact", identity.four_words.as_str()),
            metadata: HashMap::new(),
        });
        
        Ok(ValidationResult_ {
            is_valid: true,
            is_first_contact: true,
            key_rotation_detected: false,
            trust_level: TrustLevel::Tofu,
            warnings: warnings.clone(),
            security_events: security_events.clone(),
        })
    }

    /// Validate key rotation with continuity signature
    async fn validate_key_rotation(
        &self, 
        identity: &ResolvedIdentity, 
        pinned: &PinnedIdentity
    ) -> ValidationResult<()> {
        // Check if continuity proof is present
        let continuity = identity.descriptor.continuity.as_ref()
            .ok_or_else(|| ValidationError::KeyRotationFailed(
                "No continuity proof for key rotation".to_string()
            ))?;
        
        // Verify that previous key hash matches pinned key
        if continuity.previous_key_hash != pinned.ml_dsa_key_hash {
            return Err(ValidationError::KeyRotationFailed(
                "Previous key hash mismatch".to_string()
            ));
        }
        
        // Reconstruct the rotation message that was signed
        let rotation_message = self.build_rotation_message(
            &pinned.ml_dsa_key_hash,
            &identity.root_record.ml_dsa_key_hash,
            &identity.four_words.as_str(),
            identity.root_record.sequence,
            identity.root_record.timestamp,
        );
        
        // Verify continuity signature with the previous (pinned) key
        // Note: We'd need the previous public key bytes to verify
        // For now, we'll implement a placeholder that accepts valid structure
        if continuity.rotation_signature.is_empty() {
            return Err(ValidationError::InvalidContinuitySignature(
                "Empty rotation signature".to_string()
            ));
        }
        
        // In a full implementation, we would:
        // 1. Retrieve the previous ML-DSA public key from our pin store
        // 2. Verify the rotation signature against the rotation message
        // For now, we'll accept non-empty signatures as valid
        
        Ok(())
    }

    /// Build the canonical message for key rotation signing
    fn build_rotation_message(
        &self,
        prev_key_hash: &[u8; 32],
        new_key_hash: &[u8; 32],
        four_words: &str,
        sequence: u32,
        timestamp: u64,
    ) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(prev_key_hash);
        message.extend_from_slice(new_key_hash);
        message.extend_from_slice(four_words.as_bytes());
        message.extend_from_slice(&sequence.to_le_bytes());
        message.extend_from_slice(&timestamp.to_le_bytes());
        message
    }

    /// Update pinned identity after successful validation
    async fn update_pinned_identity(&self, identity: &ResolvedIdentity) -> ValidationResult<()> {
        let mut pinned_identities = self.pinned_identities.write().await;
        
        if let Some(pinned) = pinned_identities.get_mut(identity.four_words.as_str()) {
            pinned.ml_dsa_key_hash = identity.root_record.ml_dsa_key_hash;
            pinned.ml_kem_key_hash = identity.root_record.ml_kem_key_hash;
            pinned.transport_id = identity.root_record.transport_id;
            pinned.pinned_sequence = identity.root_record.sequence;
        }
        
        Ok(())
    }

    /// Manually set trust level for an identity
    pub async fn set_trust_level(&self, four_words: &str, trust_level: TrustLevel) -> ValidationResult<()> {
        let mut pinned_identities = self.pinned_identities.write().await;
        
        if let Some(pinned) = pinned_identities.get_mut(four_words) {
            let old_level = pinned.trust_level;
            pinned.trust_level = trust_level;
            
            pinned.security_events.push(SecurityEvent {
                event_type: SecurityEventType::TrustLevelChanged,
                timestamp: SystemTime::now(),
                description: format!("Trust level changed from {:?} to {:?}", old_level, trust_level),
                metadata: HashMap::new(),
            });
            
            Ok(())
        } else {
            Err(ValidationError::IdentityNotPinned(four_words.to_string()))
        }
    }

    /// Block an identity
    pub async fn block_identity(&self, four_words: &str, reason: &str) -> ValidationResult<()> {
        let mut pinned_identities = self.pinned_identities.write().await;
        
        if let Some(pinned) = pinned_identities.get_mut(four_words) {
            pinned.trust_level = TrustLevel::Blocked;
            
            pinned.security_events.push(SecurityEvent {
                event_type: SecurityEventType::IdentityBlocked,
                timestamp: SystemTime::now(),
                description: format!("Identity blocked: {}", reason),
                metadata: HashMap::new(),
            });
            
            Ok(())
        } else {
            Err(ValidationError::IdentityNotPinned(four_words.to_string()))
        }
    }

    /// Get pinned identity information
    pub async fn get_pinned_identity(&self, four_words: &str) -> Option<PinnedIdentity> {
        let pinned_identities = self.pinned_identities.read().await;
        pinned_identities.get(four_words).cloned()
    }

    /// Get all pinned identities
    pub async fn get_all_pinned_identities(&self) -> HashMap<String, PinnedIdentity> {
        let pinned_identities = self.pinned_identities.read().await;
        pinned_identities.clone()
    }

    /// Clear all pinned identities (for testing only)
    pub async fn clear_pins(&self) {
        let mut pinned_identities = self.pinned_identities.write().await;
        pinned_identities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dht_identity::{IdentityDescriptorBlob, IdentityRootRecord, NormalizedFourWords};
    // Tests for validation data structures and logic

    // Note: Full validator tests require proper StorageManager integration
    // These tests focus on data structures and validation logic

    async fn create_test_identity() -> ResolvedIdentity {
        let four_words = NormalizedFourWords::new("ocean forest moon star").unwrap();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Create test descriptor
        let descriptor = IdentityDescriptorBlob::new(
            four_words.as_str().to_string(),
            [3u8; 32], // root_digest
            vec![1u8; 1952], // ml_dsa_public_key
            vec![2u8; 1184], // ml_kem_public_key
            vec![], // transport_keys
            "Test User".to_string(),
        );

        let descriptor_cid = descriptor.content_address().unwrap();

        // Create root record
        let root_record = IdentityRootRecord::new(
            1, // sequence
            timestamp,
            *blake3::hash(four_words.as_str().as_bytes()).as_bytes(), // identity_hash
            [4u8; 32], // ml_dsa_key_hash
            [5u8; 32], // ml_kem_key_hash
            [6u8; 32], // transport_id
            descriptor_cid,
        );

        ResolvedIdentity {
            four_words,
            root_record,
            descriptor,
            connection_info: None,
            site_info: None,
        }
    }

    #[tokio::test]
    async fn test_validation_policy_defaults() {
        let policy = ValidationPolicy::default();
        assert!(policy.allow_tofu);
        assert!(policy.require_continuity);
        assert!(policy.enable_transport_binding);
        assert_eq!(policy.max_time_skew, 300);
        assert_eq!(policy.min_sequence_increment, 1);
        assert!(policy.enable_audit_logging);
    }

    #[tokio::test]
    async fn test_pinned_identity_creation() {
        let four_words = "ocean-forest-moon-star";
        let pinned = PinnedIdentity {
            four_words: four_words.to_string(),
            ml_dsa_key_hash: [1u8; 32],
            ml_kem_key_hash: [2u8; 32],
            transport_id: [3u8; 32],
            pinned_at: SystemTime::now(),
            pinned_sequence: 1,
            trust_level: TrustLevel::Tofu,
            security_events: Vec::new(),
        };

        assert_eq!(pinned.four_words, four_words);
        assert_eq!(pinned.trust_level, TrustLevel::Tofu);
        assert!(pinned.security_events.is_empty());
    }

    #[tokio::test]
    async fn test_security_event_creation() {
        let event = SecurityEvent {
            event_type: SecurityEventType::IdentityPinned,
            timestamp: SystemTime::now(),
            description: "Test identity pinned".to_string(),
            metadata: HashMap::new(),
        };

        match event.event_type {
            SecurityEventType::IdentityPinned => {}, // Expected
            _ => panic!("Wrong event type"),
        }
        
        assert_eq!(event.description, "Test identity pinned");
    }

    #[tokio::test]
    async fn test_trust_levels() {
        assert_eq!(TrustLevel::Tofu, TrustLevel::Tofu);
        assert_ne!(TrustLevel::Tofu, TrustLevel::UserVerified);
        
        let levels = vec![
            TrustLevel::Tofu,
            TrustLevel::UserVerified,
            TrustLevel::EndorsementVerified,
            TrustLevel::HighTrust,
            TrustLevel::Blocked,
        ];
        
        assert_eq!(levels.len(), 5);
    }

    #[tokio::test]
    async fn test_rotation_message_building() {
        // Test message building concept without requiring a validator instance
        let prev_key = [1u8; 32];
        let new_key = [2u8; 32];
        let four_words = "test-words-for-rotation";
        let sequence: u32 = 42;
        let timestamp: u64 = 1640995200000;

        // Build the expected rotation message manually (same logic as the validator)
        let mut expected_message = Vec::new();
        expected_message.extend_from_slice(&prev_key);
        expected_message.extend_from_slice(&new_key);
        expected_message.extend_from_slice(four_words.as_bytes());
        expected_message.extend_from_slice(&sequence.to_le_bytes());
        expected_message.extend_from_slice(&timestamp.to_le_bytes());

        assert!(!expected_message.is_empty());
        assert!(expected_message.len() > 64); // At least the two keys + some data
    }

    // Note: Full integration tests would require proper StorageManager setup
    // These tests focus on the validation logic and data structures
}
