/**
 * Saorsa Storage System - Main Module
 * Implements the complete storage system from the Developer Guide specification
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// Re-export all public types - being specific to avoid conflicts
pub use errors::*;
pub use policy::{PolicyManager, PolicyTransition};
pub use namespace::{NamespaceManager};
pub use group::{GroupManager, GroupInfo, GroupMember, GroupRole, GroupKey};
pub use engine::{StorageEngine, StorageRequest, StorageResponse, RetrievalRequest, RetrievalResponse, RetrievalSource, StorageEngineStats, StorageLocation};
pub use content::{ContentAddressing, ContentAddress, ContentChunk as StorageContentChunk};
pub use network::{NetworkManager, NetworkConfig as NetConfig, PeerInfo, NetworkStats};
pub use cache::{StorageCache, CacheEntry, CacheStats, CacheConfig as CacheCfg};
pub use config::{ConfigManager, StorageConfig};
pub use benchmarks::{BenchmarkConfig, StorageBenchmark, BenchmarkReport, BenchmarkResult};
pub use profiler::{Profiler, ProfileGuard, OptimizationRecommendations};
pub use performance_test::{PerformanceTestRunner, run_performance_smoke_test, run_comprehensive_performance_test};
pub use pqc_crypto::{PqcCryptoManager, PqcEncryptedContent, PqcEncryptionMode, MlKemKeypair, MlKemEncapsulation};

// Sub-modules
pub mod errors;
pub mod policy;
pub mod namespace;
pub mod group;
pub mod engine;
pub mod content;
pub mod network;
pub mod cache;
pub mod config;
pub mod benchmarks;
pub mod profiler;
pub mod performance_test;
pub mod pqc_crypto;

#[cfg(test)]
pub mod pqc_integration_tests;

/// Storage policy definitions matching the Developer Guide specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StoragePolicy {
    /// Maximum security - local keys only, no deduplication
    PrivateMax,
    /// User-scoped encryption with namespace isolation
    PrivateScoped { namespace: String },
    /// Group-shared encryption with collaborative access
    GroupScoped { group_id: String },
    /// Public content with convergent encryption
    PublicMarkdown,
}

impl StoragePolicy {
    /// Get the encryption mode for this policy
    pub fn encryption_mode(&self) -> EncryptionMode {
        match self {
            StoragePolicy::PrivateMax => EncryptionMode::ChaCha20Poly1305Local,
            StoragePolicy::PrivateScoped { .. } => EncryptionMode::ChaCha20Poly1305Derived,
            StoragePolicy::GroupScoped { .. } => EncryptionMode::ChaCha20Poly1305Shared,
            StoragePolicy::PublicMarkdown => EncryptionMode::Convergent,
        }
    }

    /// Get the deduplication scope for this policy
    pub fn deduplication_scope(&self) -> DeduplicationScope {
        match self {
            StoragePolicy::PrivateMax => DeduplicationScope::None,
            StoragePolicy::PrivateScoped { namespace } => DeduplicationScope::User(namespace.clone()),
            StoragePolicy::GroupScoped { group_id } => DeduplicationScope::Group(group_id.clone()),
            StoragePolicy::PublicMarkdown => DeduplicationScope::Global,
        }
    }

    /// Check if this policy allows content sharing
    pub fn allows_sharing(&self) -> bool {
        matches!(self, StoragePolicy::GroupScoped { .. } | StoragePolicy::PublicMarkdown)
    }

    /// Check if this policy requires namespace key derivation
    pub fn requires_namespace_key(&self) -> bool {
        matches!(self, StoragePolicy::PrivateScoped { .. })
    }

    /// Check if this policy requires group key management
    pub fn requires_group_key(&self) -> bool {
        matches!(self, StoragePolicy::GroupScoped { .. })
    }

    /// Check if this policy requires audit trail
    pub fn requires_audit(&self) -> bool {
        matches!(self, StoragePolicy::PublicMarkdown)
    }

    /// Check if this policy allows binary content
    pub fn allows_binary_content(&self) -> bool {
        !matches!(self, StoragePolicy::PublicMarkdown)
    }

    /// Get maximum content size for this policy
    pub fn max_content_size(&self) -> Option<u64> {
        match self {
            StoragePolicy::PrivateMax => Some(100 * 1024 * 1024), // 100MB
            StoragePolicy::PrivateScoped { .. } => Some(1024 * 1024 * 1024), // 1GB
            StoragePolicy::GroupScoped { .. } => Some(5 * 1024 * 1024 * 1024), // 5GB
            StoragePolicy::PublicMarkdown => Some(10 * 1024 * 1024), // 10MB
        }
    }
}

/// Encryption modes supported by the storage system
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum EncryptionMode {
    /// ChaCha20-Poly1305 with local random keys (PrivateMax)
    ChaCha20Poly1305Local,
    /// ChaCha20-Poly1305 with namespace-derived keys (PrivateScoped)
    ChaCha20Poly1305Derived,
    /// ChaCha20-Poly1305 with shared group keys (GroupScoped)
    ChaCha20Poly1305Shared,
    /// Convergent encryption for public content (PublicMarkdown)
    Convergent,
}

/// Deduplication scope definitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeduplicationScope {
    /// No deduplication
    None,
    /// Deduplication within user namespace
    User(String),
    /// Deduplication within group
    Group(String),
    /// Global deduplication
    Global,
}

/// Storage address for content identification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageAddress {
    pub content_id: String,
    pub policy: StoragePolicy,
    pub namespace: Option<String>,
    pub group_id: Option<String>,
}

impl StorageAddress {
    pub fn new(content_id: String, policy: StoragePolicy) -> Self {
        let namespace = match &policy {
            StoragePolicy::PrivateScoped { namespace } => Some(namespace.clone()),
            _ => None,
        };

        let group_id = match &policy {
            StoragePolicy::GroupScoped { group_id } => Some(group_id.clone()),
            _ => None,
        };

        Self {
            content_id,
            policy,
            namespace,
            group_id,
        }
    }
}

/// Metadata associated with stored content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetadata {
    pub content_type: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub size: u64,
    pub checksum: String,
}

impl Default for StorageMetadata {
    fn default() -> Self {
        Self {
            content_type: "application/octet-stream".to_string(),
            author: "unknown".to_string(),
            tags: Vec::new(),
            created_at: Utc::now(),
            modified_at: None,
            size: 0,
            checksum: String::new(),
        }
    }
}

/// Encrypted content structure
#[derive(Debug, Clone)]
pub struct EncryptedContent {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12], // ChaCha20-Poly1305 nonce size
    pub content_address: String,
    pub algorithm: String,
}

/// Content chunk for distributed storage
#[derive(Debug, Clone)]
pub struct ContentChunk {
    pub data: Vec<u8>,
    pub index: u32,
    pub total_chunks: u32,
    pub checksum: String,
    pub address: String,
}

/// Policy transition result
#[derive(Debug)]
pub struct PolicyTransitionResult {
    pub requires_re_encryption: bool,
    pub new_content_address: Option<String>,
    pub migration_tasks: Vec<String>,
}

/// Group key information
#[derive(Debug, Clone)]
pub struct GroupKeyInfo {
    pub key_id: String,
    pub encrypted_key: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub version: u32,
}

/// Namespace configuration
#[derive(Debug)]
pub struct NamespaceConfig {
    pub master_key: [u8; 32],
    pub derivation_info: String,
    pub version: u32,
}

/// Storage quota information
#[derive(Debug)]
pub struct StorageQuota {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub policy_limits: HashMap<String, u64>,
}

/// Routing statistics for performance analysis
#[derive(Debug)]
pub struct RoutingStatistics {
    pub avg_node_distance_km: f64,
    pub total_hops: u32,
    pub cache_hit_rate: f64,
    pub avg_latency_ms: u64,
}

/// Chunk information for content analysis
#[derive(Debug)]
pub struct ChunkInfo {
    pub chunk_count: u32,
    pub chunk_size: u32,
    pub total_size: u64,
    pub redundancy_factor: f64,
}

/// Content statistics for usage tracking
#[derive(Debug)]
pub struct ContentStatistics {
    pub view_count: u64,
    pub download_count: u64,
    pub last_accessed: DateTime<Utc>,
    pub size_bytes: u64,
}

/// Search result for Markdown Web content
#[derive(Debug)]
pub struct SearchResult {
    pub address: StorageAddress,
    pub title: String,
    pub snippet: String,
    pub relevance_score: f64,
}

/// Quota information for namespace management
#[derive(Debug)]
pub struct QuotaInfo {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub file_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_policy_encryption_modes() {
        assert_eq!(
            StoragePolicy::PrivateMax.encryption_mode(),
            EncryptionMode::ChaCha20Poly1305Local
        );
        assert_eq!(
            StoragePolicy::PrivateScoped { namespace: "test".to_string() }.encryption_mode(),
            EncryptionMode::ChaCha20Poly1305Derived
        );
        assert_eq!(
            StoragePolicy::GroupScoped { group_id: "test".to_string() }.encryption_mode(),
            EncryptionMode::ChaCha20Poly1305Shared
        );
        assert_eq!(
            StoragePolicy::PublicMarkdown.encryption_mode(),
            EncryptionMode::Convergent
        );
    }

    #[test]
    fn test_storage_policy_deduplication_scopes() {
        assert_eq!(
            StoragePolicy::PrivateMax.deduplication_scope(),
            DeduplicationScope::None
        );
        assert_eq!(
            StoragePolicy::PrivateScoped { namespace: "alice".to_string() }.deduplication_scope(),
            DeduplicationScope::User("alice".to_string())
        );
        assert_eq!(
            StoragePolicy::GroupScoped { group_id: "team".to_string() }.deduplication_scope(),
            DeduplicationScope::Group("team".to_string())
        );
        assert_eq!(
            StoragePolicy::PublicMarkdown.deduplication_scope(),
            DeduplicationScope::Global
        );
    }

    #[test]
    fn test_storage_policy_capabilities() {
        let private_max = StoragePolicy::PrivateMax;
        assert!(!private_max.allows_sharing());
        assert!(!private_max.requires_namespace_key());
        assert!(!private_max.requires_group_key());
        assert!(!private_max.requires_audit());
        assert!(private_max.allows_binary_content());

        let private_scoped = StoragePolicy::PrivateScoped { namespace: "test".to_string() };
        assert!(!private_scoped.allows_sharing());
        assert!(private_scoped.requires_namespace_key());
        assert!(!private_scoped.requires_group_key());

        let group_scoped = StoragePolicy::GroupScoped { group_id: "test".to_string() };
        assert!(group_scoped.allows_sharing());
        assert!(!group_scoped.requires_namespace_key());
        assert!(group_scoped.requires_group_key());

        let public_markdown = StoragePolicy::PublicMarkdown;
        assert!(public_markdown.allows_sharing());
        assert!(public_markdown.requires_audit());
        assert!(!public_markdown.allows_binary_content());
    }

    #[test]
    fn test_storage_address_creation() {
        let policy = StoragePolicy::PrivateScoped { namespace: "test_ns".to_string() };
        let address = StorageAddress::new("content_123".to_string(), policy.clone());
        
        assert_eq!(address.content_id, "content_123");
        assert_eq!(address.policy, policy);
        assert_eq!(address.namespace, Some("test_ns".to_string()));
        assert_eq!(address.group_id, None);
    }

    #[test]
    fn test_storage_metadata_default() {
        let metadata = StorageMetadata::default();
        assert_eq!(metadata.content_type, "application/octet-stream");
        assert_eq!(metadata.author, "unknown");
        assert!(metadata.tags.is_empty());
        assert_eq!(metadata.size, 0);
    }
}