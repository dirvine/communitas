# Saorsa Storage System API Documentation

## Overview

The Saorsa Storage System provides a secure, policy-driven storage solution with four distinct storage policies, advanced encryption, content addressing, and performance optimization. This document covers the complete API for developers integrating with the storage system.

## Storage Policies

### 1. PrivateMax
**Maximum Security Policy** - Local-only storage with random encryption keys
- **Use Case**: Highly sensitive data that should never leave the device
- **Encryption**: Random 256-bit ChaCha20-Poly1305 keys
- **Storage Location**: Local filesystem only
- **Access Control**: Single user only
- **Performance**: Fastest (no network overhead)
- **Size Limit**: 100MB per item

### 2. PrivateScoped
**Namespace-Isolated Private Storage** - DHT storage with namespace-derived keys
- **Use Case**: Private data with controlled backup/sync capability
- **Encryption**: HKDF-SHA256 derived keys from namespace
- **Storage Location**: DHT with 3-replica redundancy
- **Access Control**: Namespace-scoped access
- **Performance**: <500ms remote operations
- **Size Limit**: 50MB per item

### 3. GroupScoped
**Shared Group Storage** - Multi-user access with group encryption
- **Use Case**: Team collaboration and shared resources
- **Encryption**: Group shared keys with member access control
- **Storage Location**: DHT distributed across group members
- **Access Control**: Group membership required
- **Performance**: <500ms remote operations
- **Size Limit**: 10MB per item

### 4. PublicMarkdown
**Public Content** - Convergent encryption for public markdown documents
- **Use Case**: Public documentation, blogs, shared knowledge
- **Encryption**: Convergent encryption (content-derived keys)
- **Storage Location**: Public DHT
- **Access Control**: Public read access
- **Performance**: <300ms cached operations
- **Size Limit**: 1MB per item, markdown only

## Core API Components

### Storage Engine

#### Initialization
```rust
// Initialize storage engine
let engine = StorageEngine::new(dht, master_key, config_manager).await?;
```

#### Store Content
```rust
let request = StorageRequest {
    content: content_bytes,
    content_type: "text/markdown".to_string(),
    policy: StoragePolicy::PrivateScoped { 
        namespace: "user:documents".to_string() 
    },
    metadata: StorageMetadata {
        content_type: "text/markdown".to_string(),
        author: "user123".to_string(),
        tags: vec!["documentation".to_string()],
        created_at: Utc::now(),
        size: content_bytes.len() as u64,
        checksum: blake3::hash(&content_bytes).to_hex().to_string(),
        modified_at: None,
    },
    user_id: "user123".to_string(),
    group_id: None,
    namespace: Some("user:documents".to_string()),
};

let response = engine.store_content(request).await?;
```

#### Retrieve Content
```rust
let request = RetrievalRequest {
    address: storage_address,
    user_id: "user123".to_string(),
    decryption_key: None, // Auto-derived from policy
};

let response = engine.retrieve_content(request).await?;
```

### Tauri Commands

#### Initialize Storage
```typescript
import { invoke } from '@tauri-apps/api/core';

const initRequest = {
    master_key_hex: "a".repeat(64), // 32 bytes in hex
    config_path: null // Use default config
};

const success = await invoke('init_storage_engine', { request: initRequest });
```

#### Store Content
```typescript
const storeRequest = {
    content: new TextEncoder().encode("# My Document\nContent here"),
    content_type: "text/markdown",
    policy: { 
        PrivateScoped: { namespace: "user:documents" }
    },
    author: "user123",
    tags: ["documentation"],
    user_id: "user123",
    group_id: null,
    namespace: "user:documents"
};

const response = await invoke('store_content', { request: storeRequest });
```

#### Retrieve Content
```typescript
const retrieveRequest = {
    address: {
        content_id: "content_hash_here",
        policy: { PrivateScoped: { namespace: "user:documents" } }
    },
    user_id: "user123",
    decryption_key_hex: null
};

const response = await invoke('retrieve_content', { request: retrieveRequest });
```

## TypeScript Types

```typescript
export interface StoragePolicy {
    PrivateMax?: null;
    PrivateScoped?: { namespace: string };
    GroupScoped?: { group_id: string };
    PublicMarkdown?: null;
}

export interface StorageMetadata {
    content_type: string;
    author: string;
    tags: string[];
    created_at: string; // ISO 8601
    modified_at?: string; // ISO 8601
    size: number;
    checksum: string;
}

export interface StorageAddress {
    content_id: string;
    policy: StoragePolicy;
}

export interface StorageResponse {
    address: StorageAddress;
    chunks_stored: number;
    total_size: number;
    encrypted_size: number;
    operation_time_ms: number;
    storage_location: StorageLocation;
}

export interface RetrievalResponse {
    content: Uint8Array;
    metadata: StorageMetadata;
    source: RetrievalSource;
    operation_time_ms: number;
}

export type StorageLocation = 
    | "Local"
    | { Dht: { replicas: number } }
    | { Group: { members: string[] } }
    | "Public";

export type RetrievalSource = 
    | "Cache"
    | "Local" 
    | "Dht"
    | "Group"
    | { Reconstructed: { from_chunks: number } };
```

## React Hooks

### useStorage Hook
```typescript
import { useStorage } from '@/hooks/useStorage';

function DocumentEditor() {
    const { storeContent, retrieveContent, isLoading, error } = useStorage();

    const saveDocument = async (content: string) => {
        const result = await storeContent({
            content: new TextEncoder().encode(content),
            content_type: "text/markdown",
            policy: { PrivateScoped: { namespace: "user:documents" } },
            author: "current_user",
            tags: ["document"],
            user_id: "current_user"
        });
        
        if (result.success) {
            console.log('Saved with address:', result.data.address);
        }
    };

    const loadDocument = async (address: StorageAddress) => {
        const result = await retrieveContent({
            address,
            user_id: "current_user"
        });
        
        if (result.success) {
            const content = new TextDecoder().decode(result.data.content);
            return content;
        }
    };

    return (
        <div>
            {isLoading && <div>Processing...</div>}
            {error && <div>Error: {error.message}</div>}
            {/* Your UI here */}
        </div>
    );
}
```

## Performance Characteristics

### Benchmarks
- **Local Operations**: <100ms (PrivateMax policy)
- **Remote Operations**: <500ms (DHT-based policies)
- **Cache Hit Operations**: <50ms
- **Content Chunking**: 256KB optimal chunk size
- **Compression Ratio**: ~30% reduction with gzip compression

### Optimization Features
- **LRU Cache**: 100MB default capacity with automatic eviction
- **Geographic Routing**: Automatic peer selection based on latency
- **Compression**: Automatic compression for content >4KB
- **Integrity Verification**: BLAKE3 checksums for all cached content
- **Performance Profiling**: Built-in timing and metadata tracking

## Security Features

### Encryption Standards
- **Algorithm**: ChaCha20-Poly1305 AEAD
- **Key Derivation**: HKDF-SHA256 for namespace isolation
- **Randomness**: Cryptographically secure random number generation
- **Key Storage**: Platform-specific secure storage integration

### Access Control
- **Namespace Isolation**: HKDF prevents cross-namespace key derivation
- **Group Access Control**: Membership verification required
- **Content Addressing**: BLAKE3 prevents content tampering
- **Zero-Knowledge**: No plaintext content exposed in transport

### Audit Compliance
- **Memory Safety**: Rust prevents buffer overflows and use-after-free
- **Error Handling**: No panic/unwrap in production code
- **Logging**: Comprehensive audit trails without sensitive data exposure
- **Validation**: Input sanitization and size limit enforcement

## Error Handling

### Error Types
```rust
pub enum StorageError {
    NotFound { address: String },
    AccessDenied { namespace: String },
    PolicyViolation { policy: String, reason: String },
    Encryption { source: EncryptionError },
    Network { source: NetworkError },
    CacheError { reason: String },
    IntegrityFailure { address: String },
    KeyDerivation { source: KeyDerivationError },
}
```

### Frontend Error Handling
```typescript
try {
    const result = await invoke('store_content', { request });
    // Handle success
} catch (error) {
    if (error.error_type === 'PolicyViolation') {
        // Handle policy violation
    } else if (error.error_type === 'AccessDenied') {
        // Handle access denied
    } else {
        // Handle other errors
    }
}
```

## Configuration

### Default Configuration
```json
{
    "cache": {
        "max_size_bytes": 104857600,
        "max_entries": 10000,
        "default_ttl_secs": 3600,
        "compress_threshold": 4096,
        "cleanup_interval_secs": 300,
        "enable_integrity_check": true
    },
    "network": {
        "operation_timeout_secs": 30,
        "retry_attempts": 3,
        "retry_backoff_ms": 1000,
        "max_concurrent_operations": 50,
        "enable_geographic_routing": true,
        "peer_discovery_interval_secs": 300
    },
    "content": {
        "optimal_chunk_size": 262144,
        "min_chunk_size": 1024,
        "max_chunk_size": 1048576,
        "enable_compression": true,
        "integrity_algorithm": "blake3"
    }
}
```

### Environment Variables
- `SAORSA_CONFIG_PATH`: Path to custom configuration file
- `SAORSA_LOG_LEVEL`: Logging level (debug, info, warn, error)
- `SAORSA_CACHE_SIZE`: Override default cache size in bytes
- `SAORSA_DISABLE_COMPRESSION`: Disable content compression

## Testing and Validation

### Unit Tests
```bash
# Run all storage tests
cargo test saorsa_storage

# Run specific test categories
cargo test storage_tests::test_content_chunking
cargo test storage_tests::test_namespace_isolation
cargo test storage_tests::test_policy_enforcement
```

### Performance Tests
```typescript
// Run performance validation
const result = await invoke('run_storage_performance_comprehensive_test');
console.log('Performance test passed:', result);

// Custom performance test
const config = {
    iterations: 100,
    warmup_iterations: 10,
    content_sizes: [1024, 10240, 102400],
    local_target_ms: 100,
    remote_target_ms: 500
};

const customResult = await invoke('run_storage_performance_test_custom', { config });
```

### Integration Testing
```rust
// Multi-policy integration test
#[tokio::test]
async fn test_multi_policy_workflow() {
    let engine = setup_storage_engine().await;
    
    // Test all four policies
    let policies = vec![
        StoragePolicy::PrivateMax,
        StoragePolicy::PrivateScoped { namespace: "test".to_string() },
        StoragePolicy::GroupScoped { group_id: "group1".to_string() },
        StoragePolicy::PublicMarkdown,
    ];
    
    for policy in policies {
        let result = test_store_retrieve_cycle(&engine, policy).await;
        assert!(result.is_ok());
    }
}
```

## Migration and Deployment

### Policy Transitions
```rust
// Transition content from private to group-scoped
let new_address = engine.transition_policy(
    &current_address,
    StoragePolicy::GroupScoped { group_id: "team-alpha".to_string() },
    "user123"
).await?;
```

### Maintenance Operations
```rust
// Perform regular maintenance
engine.maintenance().await?;

// Manual cache cleanup
let removed_items = engine.cache.cleanup().await?;

// Network peer discovery
engine.network_manager.discover_peers().await?;
```

### Production Deployment
1. **Initialize with secure master key** (32 random bytes)
2. **Configure appropriate cache sizes** for available memory
3. **Set network timeouts** based on deployment environment
4. **Enable monitoring** with performance profiling
5. **Regular maintenance** scheduled every 6 hours

This API provides a comprehensive, secure, and performant storage solution suitable for production deployment in decentralized applications.