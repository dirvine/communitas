// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use thiserror::Error;

/// Errors that can occur during CRDT operations
#[derive(Error, Debug)]
pub enum CrdtError {
    /// Filesystem operation failed
    #[error("Filesystem error: {0}")]
    FileSystem(String),

    /// Failed to serialize data
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Failed to deserialize data
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Failed to encode or decode Yrs state
    #[error("CRDT encoding/decoding error: {0}")]
    Encoding(String),

    /// Document not found
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Invalid entity type
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    /// Invalid document ID format
    #[error("Invalid document ID: {0}")]
    InvalidDocumentId(String),

    /// Conflict resolution failed
    #[error("Conflict resolution failed: {0}")]
    ConflictResolution(String),

    /// State vector mismatch during sync
    #[error("State vector mismatch: expected {expected}, got {actual}")]
    StateVectorMismatch { expected: String, actual: String },

    /// Materialization failed (for backwards compatibility, though less relevant now)
    #[error("Materialization failed for {entity_type}/{entity_id}: {reason}")]
    MaterializationFailed {
        entity_type: String,
        entity_id: String,
        reason: String,
    },

    /// Map operation failed (get/set/delete)
    #[error("Map operation failed on key '{key}': {reason}")]
    MapOperation { key: String, reason: String },

    /// Type mismatch when reading from Map
    #[error("Type mismatch for key '{key}': expected {expected}, got {actual}")]
    TypeMismatch {
        key: String,
        expected: String,
        actual: String,
    },

    /// Generic operation error
    #[error("Operation error: {0}")]
    Operation(String),
}

/// Result type for CRDT operations
pub type CrdtResult<T> = Result<T, CrdtError>;

impl CrdtError {
    /// Create a new encoding error
    pub fn encoding_error(msg: impl Into<String>) -> Self {
        Self::Encoding(msg.into())
    }

    /// Create a new materialization error
    pub fn materialization_failed(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::MaterializationFailed {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            reason: reason.into(),
        }
    }

    /// Create a new map operation error
    pub fn map_operation(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::MapOperation {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Create a new type mismatch error
    pub fn type_mismatch(
        key: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::TypeMismatch {
            key: key.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = CrdtError::DocumentNotFound("doc-123".to_string());
        assert_eq!(err.to_string(), "Document not found: doc-123");

        let err = CrdtError::type_mismatch("status", "string", "number");
        assert_eq!(
            err.to_string(),
            "Type mismatch for key 'status': expected string, got number"
        );
    }

    #[test]
    fn test_materialization_error() {
        let err = CrdtError::materialization_failed("channel", "ch-1", "Validation failed");
        assert_eq!(
            err.to_string(),
            "Materialization failed for channel/ch-1: Validation failed"
        );
    }

    #[test]
    fn test_filesystem_error() {
        let err = CrdtError::FileSystem("Failed to write file".to_string());
        assert_eq!(err.to_string(), "Filesystem error: Failed to write file");
    }
}
