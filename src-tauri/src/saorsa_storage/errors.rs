/**
 * Saorsa Storage System - Error Types
 * Comprehensive error handling for all storage operations
 */
use std::time::Duration;
use thiserror::Error;

/// Main storage error type with comprehensive error classification
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Invalid storage policy: {reason}")]
    InvalidPolicy { reason: String },

    #[error("Key derivation failed: {source}")]
    KeyDerivation {
        #[from]
        source: KeyDerivationError,
    },

    #[error("Encryption failed: {source}")]
    Encryption {
        #[from]
        source: EncryptionError,
    },

    #[error("Network operation failed: {source}")]
    Network {
        #[from]
        source: NetworkError,
    },

    #[error("Content integrity check failed for address {address}")]
    IntegrityFailure { address: String },

    #[error("Storage quota exceeded: {current}/{limit} bytes")]
    QuotaExceeded { current: u64, limit: u64 },

    #[error("Namespace access denied: {namespace}")]
    AccessDenied { namespace: String },

    #[error("Group access denied for user {user_id} in group {group_id}")]
    GroupAccessDenied { user_id: String, group_id: String },

    #[error("Content not found at address {address}")]
    NotFound { address: String },

    #[error("Operation timeout after {duration:?}")]
    Timeout { duration: Duration },

    #[error("Insufficient redundancy: {available}/{required} replicas")]
    InsufficientRedundancy { available: u32, required: u32 },

    #[error("Content too large: {size} bytes exceeds limit {limit}")]
    ContentTooLarge { size: u64, limit: u64 },

    #[error("Invalid policy transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Cache operation failed: {reason}")]
    CacheError { reason: String },

    #[error("DHT operation failed: {reason}")]
    DhtError { reason: String },

    #[error("FEC operation failed: {reason}")]
    FecError { reason: String },

    #[error("IO operation failed: {reason}")]
    IoError { reason: String },

    #[error("Serialization failed: {reason}")]
    SerializationError { reason: String },

    #[error("Configuration error: {reason}")]
    ConfigError { reason: String },

    #[error("Internal error: {reason}")]
    Internal { reason: String },
}

/// Key derivation specific errors
#[derive(Debug, Error)]
pub enum KeyDerivationError {
    #[error("HKDF expansion failed")]
    HkdfExpansion,

    #[error("Invalid master key length: {length}, expected 32")]
    InvalidKeyLength { length: usize },

    #[error("Namespace not found: {namespace}")]
    NamespaceNotFound { namespace: String },

    #[error("Secure storage unavailable")]
    SecureStorageUnavailable,

    #[error("Key corruption detected")]
    KeyCorruption,

    #[error("HMAC generation failed")]
    HmacGenerationFailed,

    #[error("Invalid key version: {version}")]
    InvalidKeyVersion { version: u32 },

    #[error("Key rotation in progress")]
    KeyRotationInProgress,

    #[error("ML-KEM key generation failed: {details}")]
    MlKemKeyGeneration { details: String },
}

/// Encryption specific errors
#[derive(Debug, Error)]
pub enum EncryptionError {
    #[error("Encryption operation failed")]
    EncryptionFailed,

    #[error("Decryption operation failed")]
    DecryptionFailed,

    #[error("Invalid nonce length: {length}, expected 12")]
    InvalidNonce { length: usize },

    #[error("Chunk integrity verification failed")]
    IntegrityCheckFailed,

    #[error("Cipher initialization failed")]
    CipherInitializationFailed,

    #[error("Authentication tag verification failed")]
    AuthenticationFailed,

    #[error("Unsupported encryption mode: {mode}")]
    UnsupportedMode { mode: String },

    #[error("Key material insufficient")]
    InsufficientKeyMaterial,
}

/// Network operation specific errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed to {address}: {reason}")]
    ConnectionFailed { address: String, reason: String },

    #[error("Request timeout after {timeout:?}")]
    RequestTimeout { timeout: Duration },

    #[error("Peer not found: {peer_id}")]
    PeerNotFound { peer_id: String },

    #[error("Network partition detected")]
    NetworkPartition,

    #[error("Bandwidth limit exceeded")]
    BandwidthExceeded,

    #[error("Protocol error: {reason}")]
    ProtocolError { reason: String },

    #[error("Routing failed: {reason}")]
    RoutingFailed { reason: String },

    #[error("Peer rejected request: {reason}")]
    PeerRejected { reason: String },

    #[error("Geographic routing failed: no suitable peers")]
    GeographicRoutingFailed,
}

/// Namespace specific errors
#[derive(Debug, Error)]
pub enum NamespaceError {
    #[error("Invalid namespace format: {namespace}")]
    InvalidFormat { namespace: String },

    #[error("Namespace exceeds maximum length of {max_length} characters")]
    ExceedsMaxLength { max_length: usize },

    #[error("Namespace '{namespace}' is reserved")]
    Reserved { namespace: String },

    #[error("Namespace contains invalid characters")]
    InvalidCharacters,

    #[error("Namespace already exists: {namespace}")]
    AlreadyExists { namespace: String },

    #[error("Namespace key derivation failed")]
    KeyDerivationFailed,

    #[error("Namespace isolation violation")]
    IsolationViolation,

    #[error("Concurrent modification detected")]
    ConcurrentModification,
}

/// Group management specific errors
#[derive(Debug, Error)]
pub enum GroupError {
    #[error("Group not found: {group_id}")]
    GroupNotFound { group_id: String },

    #[error("User {user_id} is not a member of group {group_id}")]
    UserNotMember { user_id: String, group_id: String },

    #[error("Group key not found for group {group_id}")]
    GroupKeyNotFound { group_id: String },

    #[error("Key wrapping failed")]
    KeyWrappingFailed,

    #[error("Key unwrapping failed")]
    KeyUnwrappingFailed,

    #[error("Group already exists: {group_id}")]
    GroupAlreadyExists { group_id: String },

    #[error("Insufficient permissions for group operation")]
    InsufficientPermissions,

    #[error("Group key rotation failed")]
    KeyRotationFailed,

    #[error("Maximum group size exceeded")]
    MaxGroupSizeExceeded,
}

/// Cache specific errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache miss for key: {key}")]
    CacheMiss { key: String },

    #[error("Cache full, eviction failed")]
    CacheFull,

    #[error("Cache corruption detected")]
    CacheCorruption,

    #[error("Cache write failed: {reason}")]
    WriteFailed { reason: String },

    #[error("Cache read failed: {reason}")]
    ReadFailed { reason: String },

    #[error("Invalid cache entry")]
    InvalidEntry,

    #[error("Cache lock contention")]
    LockContention,

    #[error("Cache eviction failed")]
    EvictionFailed,
}

/// Content addressing and chunking errors
#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Invalid content address: {address}")]
    InvalidAddress { address: String },

    #[error("Chunking failed: {reason}")]
    ChunkingFailed { reason: String },

    #[error("Chunk reconstruction failed")]
    ReconstructionFailed,

    #[error("Content checksum mismatch")]
    ChecksumMismatch,

    #[error("Invalid chunk index: {index}")]
    InvalidChunkIndex { index: u32 },

    #[error("Missing chunks: {missing_count}/{total_count}")]
    MissingChunks {
        missing_count: u32,
        total_count: u32,
    },

    #[error("Content type validation failed")]
    ContentTypeValidationFailed,

    #[error("Content size validation failed")]
    ContentSizeValidationFailed,
}

/// Policy management errors
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Unknown policy type: {policy_type}")]
    UnknownPolicyType { policy_type: String },

    #[error("Policy validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Policy enforcement failed: {reason}")]
    EnforcementFailed { reason: String },

    #[error("Policy transition not allowed from {from} to {to}")]
    TransitionNotAllowed { from: String, to: String },

    #[error("Missing required policy parameter: {parameter}")]
    MissingParameter { parameter: String },

    #[error("Invalid policy configuration")]
    InvalidConfiguration,

    #[error("Policy conflict detected")]
    PolicyConflict,

    #[error("Policy not found: {policy_id}")]
    PolicyNotFound { policy_id: String },
}

// Conversion implementations for better error propagation

impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::IoError {
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::SerializationError {
            reason: err.to_string(),
        }
    }
}

impl From<anyhow::Error> for StorageError {
    fn from(err: anyhow::Error) -> Self {
        StorageError::Internal {
            reason: err.to_string(),
        }
    }
}

impl From<KeyDerivationError> for NamespaceError {
    fn from(err: KeyDerivationError) -> Self {
        match err {
            KeyDerivationError::HkdfExpansion => NamespaceError::KeyDerivationFailed,
            KeyDerivationError::InvalidKeyLength { .. } => NamespaceError::KeyDerivationFailed,
            KeyDerivationError::KeyCorruption => NamespaceError::KeyDerivationFailed,
            KeyDerivationError::HmacGenerationFailed => NamespaceError::KeyDerivationFailed,
            _ => NamespaceError::KeyDerivationFailed,
        }
    }
}

impl From<NamespaceError> for StorageError {
    fn from(err: NamespaceError) -> Self {
        StorageError::AccessDenied {
            namespace: match &err {
                NamespaceError::InvalidFormat { namespace } => namespace.clone(),
                NamespaceError::Reserved { namespace } => namespace.clone(),
                NamespaceError::AlreadyExists { namespace } => namespace.clone(),
                _ => "unknown".to_string(),
            },
        }
    }
}

impl From<GroupError> for StorageError {
    fn from(err: GroupError) -> Self {
        match err {
            GroupError::UserNotMember { user_id, group_id } => {
                StorageError::GroupAccessDenied { user_id, group_id }
            }
            GroupError::GroupNotFound { group_id } => StorageError::NotFound {
                address: format!("group:{}", group_id),
            },
            _ => StorageError::Internal {
                reason: err.to_string(),
            },
        }
    }
}

impl From<CacheError> for StorageError {
    fn from(err: CacheError) -> Self {
        StorageError::CacheError {
            reason: err.to_string(),
        }
    }
}

impl From<ContentError> for StorageError {
    fn from(err: ContentError) -> Self {
        match err {
            ContentError::InvalidAddress { address } => StorageError::NotFound { address },
            ContentError::ChecksumMismatch => StorageError::IntegrityFailure {
                address: "unknown".to_string(),
            },
            _ => StorageError::Internal {
                reason: err.to_string(),
            },
        }
    }
}

impl From<PolicyError> for StorageError {
    fn from(err: PolicyError) -> Self {
        StorageError::InvalidPolicy {
            reason: err.to_string(),
        }
    }
}

// Result type aliases for convenience
pub type StorageResult<T> = Result<T, StorageError>;
pub type KeyDerivationResult<T> = Result<T, KeyDerivationError>;
pub type EncryptionResult<T> = Result<T, EncryptionError>;
pub type NetworkResult<T> = Result<T, NetworkError>;
pub type NamespaceResult<T> = Result<T, NamespaceError>;
pub type GroupResult<T> = Result<T, GroupError>;
pub type CacheResult<T> = Result<T, CacheError>;
pub type ContentResult<T> = Result<T, ContentError>;
pub type PolicyResult<T> = Result<T, PolicyError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = StorageError::QuotaExceeded {
            current: 1000,
            limit: 500,
        };
        assert_eq!(error.to_string(), "Storage quota exceeded: 1000/500 bytes");
    }

    #[test]
    fn test_error_from_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let storage_error = StorageError::from(io_error);

        match storage_error {
            StorageError::IoError { reason } => {
                assert!(reason.contains("file not found"));
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_namespace_error_conversion() {
        let namespace_error = NamespaceError::Reserved {
            namespace: "system".to_string(),
        };
        let storage_error = StorageError::from(namespace_error);

        match storage_error {
            StorageError::AccessDenied { namespace } => {
                assert_eq!(namespace, "system");
            }
            _ => panic!("Expected AccessDenied variant"),
        }
    }

    #[test]
    fn test_group_error_conversion() {
        let group_error = GroupError::UserNotMember {
            user_id: "alice".to_string(),
            group_id: "team".to_string(),
        };
        let storage_error = StorageError::from(group_error);

        match storage_error {
            StorageError::GroupAccessDenied { user_id, group_id } => {
                assert_eq!(user_id, "alice");
                assert_eq!(group_id, "team");
            }
            _ => panic!("Expected GroupAccessDenied variant"),
        }
    }
}
