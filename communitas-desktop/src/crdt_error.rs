use thiserror::Error;

/// Errors that can occur during CRDT operations
/// Some variants are infrastructure for future use
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum CrdtError {
    /// Database connection or query failed
    #[error("Database error: {0}")]
    Database(#[from] libsql::Error),

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

    /// Materialization to SQL failed
    #[error("SQL materialization failed for {entity_type}/{entity_id}: {reason}")]
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

    /// Schema initialization failed
    #[error("Schema initialization failed: {0}")]
    SchemaInit(String),

    /// Generic operation error
    #[error("Operation error: {0}")]
    Operation(String),
}

/// Result type for CRDT operations
pub type CrdtResult<T> = Result<T, CrdtError>;

impl CrdtError {
    /// Create a new encoding error
    #[allow(dead_code)]
    pub fn encoding_error(msg: impl Into<String>) -> Self {
        Self::Encoding(msg.into())
    }

    /// Create a new materialization error
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn map_operation(key: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::MapOperation {
            key: key.into(),
            reason: reason.into(),
        }
    }

    /// Create a new type mismatch error
    #[allow(dead_code)]
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
        let err = CrdtError::materialization_failed("channel", "ch-1", "SQL constraint violation");
        assert_eq!(
            err.to_string(),
            "SQL materialization failed for channel/ch-1: SQL constraint violation"
        );
    }
}
