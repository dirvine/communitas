# Saorsa Storage System - Deployment Guide

## Quick Start

### Prerequisites
- Rust 1.85+ with 2024 edition support
- Node.js 18+ for frontend development
- Tauri CLI for desktop application builds
- Git for version control

### Initial Setup
```bash
# Clone the repository
git clone <repository-url>
cd communitas

# Install frontend dependencies
npm install

# Build and test the storage system
cd src-tauri
cargo build --release
cargo test saorsa_storage

# Run the complete application
cd ..
npm run tauri dev
```

## Configuration Management

### Default Configuration
The system ships with sensible defaults suitable for development and small-scale deployments:

```json
{
  "cache": {
    "max_size_bytes": 104857600,        // 100MB cache
    "max_entries": 10000,               // 10k entries max
    "default_ttl_secs": 3600,           // 1 hour TTL
    "compress_threshold": 4096,         // Compress >4KB
    "cleanup_interval_secs": 300,       // 5 minute cleanup
    "enable_integrity_check": true      // Enable checksums
  },
  "network": {
    "operation_timeout_secs": 30,       // 30 second timeout
    "retry_attempts": 3,                // 3 retry attempts
    "retry_backoff_ms": 1000,           // 1 second backoff
    "max_concurrent_operations": 50,    // 50 concurrent ops
    "enable_geographic_routing": true,  // Enable geo routing
    "peer_discovery_interval_secs": 300 // 5 minute discovery
  },
  "content": {
    "optimal_chunk_size": 262144,       // 256KB chunks
    "min_chunk_size": 1024,             // 1KB minimum
    "max_chunk_size": 1048576,          // 1MB maximum
    "enable_compression": true,         // Enable compression
    "integrity_algorithm": "blake3"     // Use BLAKE3
  },
  "policies": {
    "private_max_size_mb": 100,         // 100MB limit
    "private_scoped_size_mb": 50,       // 50MB limit
    "group_scoped_size_mb": 10,         // 10MB limit
    "public_markdown_size_mb": 1        // 1MB limit
  }
}
```

### Production Configuration
For production deployments, create a custom configuration file:

```json
{
  "cache": {
    "max_size_bytes": 1073741824,       // 1GB cache
    "max_entries": 100000,              // 100k entries
    "default_ttl_secs": 7200,           // 2 hour TTL
    "compress_threshold": 2048,         // Compress >2KB
    "cleanup_interval_secs": 900,       // 15 minute cleanup
    "enable_integrity_check": true
  },
  "network": {
    "operation_timeout_secs": 60,       // 60 second timeout
    "retry_attempts": 5,                // 5 retry attempts
    "retry_backoff_ms": 2000,           // 2 second backoff
    "max_concurrent_operations": 200,   // 200 concurrent ops
    "enable_geographic_routing": true,
    "peer_discovery_interval_secs": 600 // 10 minute discovery
  },
  "content": {
    "optimal_chunk_size": 262144,       // Keep 256KB
    "min_chunk_size": 512,              // 512B minimum
    "max_chunk_size": 2097152,          // 2MB maximum
    "enable_compression": true,
    "integrity_algorithm": "blake3"
  },
  "policies": {
    "private_max_size_mb": 500,         // 500MB limit
    "private_scoped_size_mb": 200,      // 200MB limit
    "group_scoped_size_mb": 50,         // 50MB limit
    "public_markdown_size_mb": 5        // 5MB limit
  },
  "monitoring": {
    "enable_performance_metrics": true,
    "metric_collection_interval_secs": 60,
    "enable_security_audit_log": true,
    "max_log_file_size_mb": 100
  }
}
```

### Environment Variables
Override configuration with environment variables:

```bash
# Configuration file path
export SAORSA_CONFIG_PATH="/etc/communitas/storage.json"

# Logging configuration
export SAORSA_LOG_LEVEL="info"  # debug, info, warn, error
export RUST_LOG="saorsa_storage=debug,communitas=info"

# Performance tuning
export SAORSA_CACHE_SIZE="2147483648"  # 2GB cache
export SAORSA_MAX_CONCURRENT_OPS="500"
export SAORSA_CHUNK_SIZE="524288"      # 512KB chunks

# Security settings
export SAORSA_DISABLE_COMPRESSION="false"
export SAORSA_ENABLE_INTEGRITY_CHECK="true"
export SAORSA_KEY_ROTATION_INTERVAL="2592000"  # 30 days

# Network configuration
export SAORSA_DHT_BOOTSTRAP_NODES="159.89.81.21:9001,..."
export SAORSA_PEER_DISCOVERY_INTERVAL="600"
export SAORSA_NETWORK_TIMEOUT="120"
```

## Master Key Management

### Key Generation
```bash
# Generate a secure master key
cargo run --bin generate-master-key

# Or use the Tauri command
npm run tauri dev
# In the app, call: invoke('generate_master_key')
```

### Key Storage
**Development**: Store in environment variable
```bash
export SAORSA_MASTER_KEY="a1b2c3d4e5f6..." # 64 hex characters
```

**Production**: Use platform-specific secure storage
- **macOS**: Keychain integration
- **Windows**: Windows Credential Manager
- **Linux**: libsecret/Secret Service

### Key Rotation
```rust
// Implement key rotation for long-term security
let new_master_key = generate_master_key();
engine.rotate_master_key(old_key, new_master_key).await?;
```

## Deployment Scenarios

### Single User Desktop Application

```bash
# Build for production
npm run tauri build

# The built application includes:
# - Complete storage engine
# - Local DHT node
# - Secure key storage
# - Performance optimization
```

Configuration optimizations:
```json
{
  "cache": { "max_size_bytes": 268435456 },  // 256MB
  "network": { "max_concurrent_operations": 25 },
  "policies": {
    "private_max_size_mb": 200,
    "private_scoped_size_mb": 100
  }
}
```

### Multi-User Collaboration

Deploy with shared DHT infrastructure:

```bash
# Set up bootstrap nodes
export SAORSA_DHT_BOOTSTRAP_NODES="node1:9001,node2:9001,node3:9001"

# Configure for group collaboration
export SAORSA_MAX_CONCURRENT_OPS="100"
export SAORSA_CACHE_SIZE="1073741824"  # 1GB

# Build and deploy
npm run tauri build --target production
```

Configuration for collaboration:
```json
{
  "network": {
    "enable_geographic_routing": true,
    "peer_discovery_interval_secs": 300,
    "max_concurrent_operations": 100
  },
  "policies": {
    "group_scoped_size_mb": 25,
    "public_markdown_size_mb": 2
  }
}
```

### Enterprise Deployment

Large-scale deployment with monitoring:

```bash
# Set up monitoring
export SAORSA_ENABLE_METRICS="true"
export SAORSA_METRICS_ENDPOINT="http://prometheus:9090"
export SAORSA_LOG_LEVEL="info"

# Configure for scale
export SAORSA_CACHE_SIZE="4294967296"     # 4GB
export SAORSA_MAX_CONCURRENT_OPS="500"
export SAORSA_CHUNK_SIZE="1048576"        # 1MB chunks

# Security hardening
export SAORSA_ENABLE_SECURITY_AUDIT="true"
export SAORSA_KEY_ROTATION_INTERVAL="604800"  # 7 days
```

## Performance Tuning

### Memory Optimization
```json
{
  "cache": {
    "max_size_bytes": 2147483648,  // Adjust based on available RAM
    "max_entries": 50000,          // Balance memory vs. entries
    "cleanup_interval_secs": 600   // Less frequent cleanup
  }
}
```

### Network Optimization
```json
{
  "network": {
    "operation_timeout_secs": 45,
    "retry_attempts": 3,
    "max_concurrent_operations": 150,  // Based on bandwidth
    "enable_geographic_routing": true
  }
}
```

### Storage Optimization
```json
{
  "content": {
    "optimal_chunk_size": 524288,    // 512KB for fast networks
    "enable_compression": true,      // Reduce bandwidth usage
    "integrity_algorithm": "blake3"  // Fast hashing
  }
}
```

### Policy Optimization
```json
{
  "policies": {
    "private_max_size_mb": 1000,     // Generous limits
    "private_scoped_size_mb": 500,
    "group_scoped_size_mb": 100,
    "public_markdown_size_mb": 10
  }
}
```

## Monitoring and Observability

### Built-in Metrics
```typescript
// Get storage statistics
const stats = await invoke('get_storage_stats');
console.log('Cache hit ratio:', stats.cache_hit_ratio);
console.log('Average operation time:', stats.avg_operation_time_ms);
console.log('Policy distribution:', stats.policy_distribution);
```

### Performance Testing
```typescript
// Run comprehensive performance test
const result = await invoke('run_storage_performance_comprehensive_test');
if (result) {
    console.log('All performance targets met');
} else {
    console.log('Performance issues detected');
}

// Custom performance test
const config = {
    iterations: 100,
    warmup_iterations: 10,
    content_sizes: [1024, 10240, 102400, 1048576],
    local_target_ms: 50,
    remote_target_ms: 300
};
const customResult = await invoke('run_storage_performance_test_custom', { config });
```

### Log Configuration
```bash
# Structured logging for production
export RUST_LOG="info,saorsa_storage=debug"

# JSON logs for centralized logging
export SAORSA_LOG_FORMAT="json"
export SAORSA_LOG_FILE="/var/log/communitas/storage.log"
```

## Security Hardening

### Cryptographic Configuration
```json
{
  "security": {
    "key_derivation_iterations": 100000,
    "enable_secure_random": true,
    "require_authenticated_encryption": true,
    "minimum_key_strength": 256
  }
}
```

### Access Control
```json
{
  "access_control": {
    "enable_namespace_isolation": true,
    "require_group_membership": true,
    "enable_content_verification": true,
    "audit_all_operations": true
  }
}
```

### Network Security
```json
{
  "network_security": {
    "enable_tls": true,
    "verify_peer_certificates": true,
    "enable_peer_authentication": true,
    "blacklist_malicious_peers": true
  }
}
```

## Backup and Recovery

### Content Backup
```rust
// Backup critical content with multiple policies
let backup_policies = vec![
    StoragePolicy::PrivateScoped { namespace: "backup:primary".to_string() },
    StoragePolicy::PrivateScoped { namespace: "backup:secondary".to_string() },
];

for policy in backup_policies {
    engine.transition_policy(&original_address, policy, &user_id).await?;
}
```

### Key Backup
```bash
# Backup master key securely
saorsa-backup-key --master-key "$SAORSA_MASTER_KEY" \
                  --output encrypted-key-backup.json \
                  --passphrase-file /secure/passphrase
```

### Recovery Procedures
1. **Content Recovery**: Use Reed-Solomon reconstruction for corrupted data
2. **Key Recovery**: Restore from secure backup with proper passphrase
3. **Cache Recovery**: Automatic cache rebuilding from distributed storage
4. **Network Recovery**: Automatic peer discovery and reconnection

## Troubleshooting

### Common Issues

#### Storage Engine Not Initialized
```typescript
// Check initialization status
const isInitialized = await invoke('is_storage_initialized');
if (!isInitialized) {
    // Initialize with master key
    await invoke('init_storage_engine', { 
        request: { master_key_hex: masterKey, config_path: null }
    });
}
```

#### Cache Performance Issues
```bash
# Monitor cache statistics
tail -f /var/log/communitas/storage.log | grep "cache_stats"

# Clear cache if needed
cargo run --bin clear-storage-cache
```

#### Network Connectivity Problems
```bash
# Test DHT connectivity
cargo test --test test_dht_connectivity

# Check peer discovery
export RUST_LOG="network=debug"
cargo run --bin test-peer-discovery
```

#### Performance Degradation
```typescript
// Run performance diagnostics
const diagnostics = await invoke('run_storage_performance_smoke_test');
if (!diagnostics) {
    // Run comprehensive analysis
    const fullTest = await invoke('run_storage_performance_comprehensive_test');
    console.log('Performance analysis complete');
}
```

### Debug Tools

#### Enable Debug Logging
```bash
export RUST_LOG="debug"
export SAORSA_LOG_LEVEL="debug"
npm run tauri dev
```

#### Performance Profiling
```bash
# Run with profiling enabled
cargo build --features profiling
export SAORSA_ENABLE_PROFILING="true"
npm run tauri dev
```

#### Memory Analysis
```bash
# Memory usage analysis
cargo build --features memory-profiling
valgrind --tool=massif target/debug/communitas
```

## Updates and Maintenance

### Regular Maintenance
```bash
# Daily maintenance tasks
0 2 * * * /usr/local/bin/saorsa-maintenance daily

# Weekly maintenance tasks  
0 3 * * 0 /usr/local/bin/saorsa-maintenance weekly

# Monthly maintenance tasks
0 4 1 * * /usr/local/bin/saorsa-maintenance monthly
```

### Version Updates
```bash
# Update to new version
git pull origin main
npm install
cargo update
npm run tauri build --target production
```

### Migration Procedures
```rust
// Migrate content to new storage format
let migration_plan = engine.plan_migration(old_version, new_version).await?;
engine.execute_migration(migration_plan).await?;
```

This deployment guide provides comprehensive instructions for deploying the Saorsa Storage System in various environments with appropriate security, performance, and reliability considerations.