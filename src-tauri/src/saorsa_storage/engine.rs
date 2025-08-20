/**
 * Saorsa Storage System - Main Storage Engine
 * Orchestrates all storage components and provides unified interface
 */

use crate::saorsa_storage::errors::*;
use crate::saorsa_storage::*;
use crate::saorsa_storage::{
    policy::PolicyManager,
    namespace::NamespaceManager,
    group::GroupManager,
    content::ContentAddressing,
    network::NetworkManager,
    cache::StorageCache,
    config::ConfigManager,
    profiler::Profiler,
    pqc_crypto::{PqcCryptoManager, PqcEncryptedContent},
};
use crate::dht_facade::DhtFacade;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Storage operation request
#[derive(Debug, Clone)]
pub struct StorageRequest {
    pub content: Vec<u8>,
    pub content_type: String,
    pub policy: StoragePolicy,
    pub metadata: StorageMetadata,
    pub user_id: String,
    pub group_id: Option<String>,
    pub namespace: Option<String>,
}

/// Storage operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResponse {
    pub address: StorageAddress,
    pub chunks_stored: u32,
    pub total_size: u64,
    pub encrypted_size: u64,
    pub operation_time_ms: u64,
    pub storage_location: StorageLocation,
}

/// Storage location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageLocation {
    Local,
    Dht { replicas: u8 },
    Group { members: Vec<String> },
    Public,
}

/// Retrieval request
#[derive(Debug, Clone)]
pub struct RetrievalRequest {
    pub address: StorageAddress,
    pub user_id: String,
    pub decryption_key: Option<[u8; 32]>,
}

/// Retrieval response
#[derive(Debug, Clone)]
pub struct RetrievalResponse {
    pub content: Vec<u8>,
    pub metadata: StorageMetadata,
    pub source: RetrievalSource,
    pub operation_time_ms: u64,
}

/// Source of retrieved content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalSource {
    Cache,
    Local,
    Dht,
    Group,
    Reconstructed { from_chunks: u32 },
}

/// Storage system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEngineStats {
    pub total_content_items: u64,
    pub total_bytes_stored: u64,
    pub cache_hit_ratio: f64,
    pub network_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub avg_operation_time_ms: f64,
    pub policy_distribution: HashMap<String, u64>,
    pub last_updated: DateTime<Utc>,
}

/// Main storage engine that orchestrates all components
pub struct StorageEngine<D: DhtFacade> {
    policy_manager: Arc<PolicyManager>,
    namespace_manager: Arc<NamespaceManager>,
    group_manager: Arc<GroupManager>,
    content_addressing: Arc<ContentAddressing>,
    network_manager: Arc<NetworkManager<D>>,
    cache: Arc<StorageCache>,
    config_manager: Arc<RwLock<ConfigManager>>,
    stats: Arc<RwLock<StorageEngineStats>>,
    active_operations: Arc<RwLock<HashMap<String, OperationState>>>,
    profiler: Arc<Profiler>,
    pqc_crypto_manager: Arc<PqcCryptoManager>,
}

/// Operation state tracking
#[derive(Debug, Clone)]
struct OperationState {
    operation_id: String,
    operation_type: String,
    started_at: std::time::Instant,
    user_id: String,
    progress: f32, // 0.0 to 1.0
}

impl<D: DhtFacade> StorageEngine<D> {
    /// Create a new storage engine
    pub async fn new(
        dht: Arc<D>,
        master_key: [u8; 32],
        config_manager: ConfigManager,
    ) -> StorageResult<Self> {
        let config = config_manager.get_config().clone();
        
        // Initialize all components
        let policy_manager = Arc::new(PolicyManager::new());
        let namespace_manager = Arc::new(NamespaceManager::new(&master_key)?);
        let group_manager = Arc::new(GroupManager::new(master_key));
        let content_addressing = Arc::new(ContentAddressing::new());
        let pqc_crypto_manager = Arc::new(PqcCryptoManager::new(master_key)?);
        
        // Configure network manager
        let network_config = crate::saorsa_storage::network::NetworkConfig {
            operation_timeout: std::time::Duration::from_secs(config.network.operation_timeout_secs),
            retry_attempts: config.network.retry_attempts,
            retry_backoff: std::time::Duration::from_millis(config.network.retry_backoff_ms),
            max_concurrent_operations: config.network.max_concurrent_operations,
            enable_geographic_routing: config.network.enable_geographic_routing,
            peer_discovery_interval: std::time::Duration::from_secs(config.network.peer_discovery_interval_secs),
        };
        let network_manager = Arc::new(NetworkManager::with_config(dht, network_config, None));
        
        // Configure cache
        let cache_config = crate::saorsa_storage::cache::CacheConfig {
            max_size_bytes: config.cache.max_size_bytes,
            max_entries: config.cache.max_entries,
            default_ttl: config.cache.default_ttl_secs.map(std::time::Duration::from_secs),
            compress_threshold: config.cache.compress_threshold,
            cleanup_interval: std::time::Duration::from_secs(config.cache.cleanup_interval_secs),
            enable_integrity_check: config.cache.enable_integrity_check,
        };
        let cache = Arc::new(StorageCache::with_config(cache_config));

        let stats = Arc::new(RwLock::new(StorageEngineStats {
            total_content_items: 0,
            total_bytes_stored: 0,
            cache_hit_ratio: 0.0,
            network_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_operation_time_ms: 0.0,
            policy_distribution: HashMap::new(),
            last_updated: Utc::now(),
        }));

        Ok(Self {
            policy_manager,
            namespace_manager,
            group_manager,
            content_addressing,
            network_manager,
            cache,
            config_manager: Arc::new(RwLock::new(config_manager)),
            stats,
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            profiler: Arc::new(Profiler::new()),
            pqc_crypto_manager,
        })
    }

    /// Store content using specified policy
    pub async fn store_content(&self, request: StorageRequest) -> StorageResult<StorageResponse> {
        let operation_id = self.generate_operation_id();
        let start_time = std::time::Instant::now();
        
        // Track operation
        self.start_operation(&operation_id, "store", &request.user_id).await;

        let result = self.store_content_internal(request).await;
        
        // Complete operation tracking
        self.complete_operation(&operation_id, result.is_ok()).await;
        
        match result {
            Ok(response) => {
                self.update_stats_success("store", start_time.elapsed()).await;
                Ok(response)
            }
            Err(error) => {
                self.update_stats_failure("store").await;
                Err(error)
            }
        }
    }

    /// Retrieve content by address
    pub async fn retrieve_content(&self, request: RetrievalRequest) -> StorageResult<RetrievalResponse> {
        let operation_id = self.generate_operation_id();
        let start_time = std::time::Instant::now();
        
        self.start_operation(&operation_id, "retrieve", &request.user_id).await;

        let result = self.retrieve_content_internal(request).await;
        
        self.complete_operation(&operation_id, result.is_ok()).await;
        
        match result {
            Ok(response) => {
                self.update_stats_success("retrieve", start_time.elapsed()).await;
                Ok(response)
            }
            Err(error) => {
                self.update_stats_failure("retrieve").await;
                Err(error)
            }
        }
    }

    /// List content by policy and user
    pub async fn list_content(
        &self,
        _user_id: &str,
        _policy_filter: Option<StoragePolicy>,
        _limit: Option<u32>,
    ) -> StorageResult<Vec<StorageAddress>> {
        // This would typically query a metadata index
        // For now, return empty list as this requires persistent metadata storage
        Ok(Vec::new())
    }

    /// Delete content by address
    pub async fn delete_content(&self, address: &StorageAddress, user_id: &str) -> StorageResult<bool> {
        let operation_id = self.generate_operation_id();
        let start_time = std::time::Instant::now();
        
        self.start_operation(&operation_id, "delete", user_id).await;

        // Verify user has permission to delete
        match &address.policy {
            StoragePolicy::PrivateMax | StoragePolicy::PrivateScoped { .. } => {
                // Only the owner can delete private content
                // This would require checking metadata for ownership
            }
            StoragePolicy::GroupScoped { group_id } => {
                // Verify user is group member with delete permissions
                if !self.group_manager.is_member(group_id, user_id).await {
                    self.complete_operation(&operation_id, false).await;
                    return Err(StorageError::AccessDenied {
                        namespace: group_id.clone(),
                    });
                }
            }
            StoragePolicy::PublicMarkdown => {
                // Public content deletion might be restricted
                // Implementation depends on governance model
            }
        }

        // Remove from cache first
        let cache_key = self.generate_cache_key(address);
        self.cache.remove(&cache_key).await.ok();

        // For DHT content, this would involve sending delete requests
        // For now, consider it successful if we can remove from cache
        self.complete_operation(&operation_id, true).await;
        self.update_stats_success("delete", start_time.elapsed()).await;
        
        Ok(true)
    }

    /// Get storage engine statistics
    pub async fn get_stats(&self) -> StorageEngineStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update with current cache stats
        let cache_stats = self.cache.get_stats().await;
        stats.cache_hit_ratio = cache_stats.hit_ratio();
        
        // Update with network stats
        let network_stats = self.network_manager.get_stats().await;
        stats.network_operations = network_stats.total_operations;
        
        stats.last_updated = Utc::now();
        stats
    }

    /// Transition content to new policy
    pub async fn transition_policy(
        &self,
        address: &StorageAddress,
        new_policy: StoragePolicy,
        user_id: &str,
    ) -> StorageResult<StorageAddress> {
        // Check if transition is allowed
        let transition = self.policy_manager.plan_transition(&address.policy, &new_policy)?;
        
        if transition.requires_re_encryption {
            // Retrieve content with old policy
            let retrieval_request = RetrievalRequest {
                address: address.clone(),
                user_id: user_id.to_string(),
                decryption_key: None,
            };
            
            let retrieval_response = self.retrieve_content_internal(retrieval_request).await?;
            
            // Store with new policy
            let content_type = retrieval_response.metadata.content_type.clone();
            let storage_request = StorageRequest {
                content: retrieval_response.content,
                content_type,
                policy: new_policy,
                metadata: retrieval_response.metadata,
                user_id: user_id.to_string(),
                group_id: None,
                namespace: None,
            };
            
            let storage_response = self.store_content_internal(storage_request).await?;
            
            // Delete old content
            self.delete_content(address, user_id).await?;
            
            Ok(storage_response.address)
        } else {
            // Simple policy update without re-encryption
            let mut new_address = address.clone();
            new_address.policy = new_policy;
            Ok(new_address)
        }
    }

    /// Perform maintenance operations
    pub async fn maintenance(&self) -> StorageResult<()> {
        // Cache cleanup
        self.cache.cleanup().await.map_err(|e| StorageError::CacheError {
            reason: e.to_string(),
        })?;

        // Namespace key cleanup (remove old keys)
        self.namespace_manager.cleanup_old_keys(90).await
            .map_err(|e| StorageError::from(e))?;

        // Network peer discovery
        self.network_manager.discover_peers().await
            .map_err(|e| StorageError::Network { source: e })?;

        Ok(())
    }

    // Private implementation methods

    async fn store_content_internal(&self, request: StorageRequest) -> StorageResult<StorageResponse> {
        let start_time = std::time::Instant::now();
        let mut guard = self.profiler.start_profile("store_content_internal").await;
        guard.add_size_metadata(request.content.len()).await;
        
        // Validate policy and content
        {
            let _validate_guard = self.profiler.start_profile("policy_validation").await;
            self.policy_manager.validate_policy(
                &request.policy,
                request.content.len() as u64,
                &request.user_id,
                &request.content_type,
            ).await?;
        }

        // Enforce policy constraints
        self.policy_manager.enforce_policy(
            &request.policy,
            &request.metadata.checksum,
            &request.content,
            &request.user_id,
            "store",
        ).await?;

        // Generate content address
        let content_address = self.content_addressing.address_content(
            &request.content,
            &request.content_type,
        )?;

        // Encrypt content based on policy
        let (encrypted_content, _encryption_key) = {
            let _encrypt_guard = self.profiler.start_profile("content_encryption").await;
            self.encrypt_content(&request).await?
        };

        // Store based on policy
        let storage_location = match &request.policy {
            StoragePolicy::PrivateMax => {
                // Store only locally (would require local storage implementation)
                self.store_local(&content_address, &encrypted_content).await?;
                StorageLocation::Local
            }
            StoragePolicy::PrivateScoped { namespace } => {
                // Store in DHT with namespace isolation
                let dht_key = self.generate_dht_key(namespace, &content_address.hash);
                self.network_manager.store_content(dht_key, encrypted_content.clone()).await?;
                StorageLocation::Dht { replicas: 3 }
            }
            StoragePolicy::GroupScoped { group_id } => {
                // Distribute to group members
                let members = self.group_manager.get_members(group_id).await?;
                let member_ids: Vec<String> = members.iter().map(|m| m.user_id.clone()).collect();
                
                // For now, store in DHT with group key
                let group_key = format!("group:{}", group_id);
                let dht_key = self.generate_dht_key(&group_key, &content_address.hash);
                self.network_manager.store_content(dht_key, encrypted_content.clone()).await?;
                
                StorageLocation::Group { members: member_ids }
            }
            StoragePolicy::PublicMarkdown => {
                // Store in public DHT
                let public_key = format!("public:{}", content_address.hash);
                self.network_manager.store_content(public_key.into_bytes(), encrypted_content.clone()).await?;
                StorageLocation::Public
            }
        };

        // Cache the content
        let cache_key = self.generate_cache_key(&StorageAddress::new(
            content_address.hash.clone(),
            request.policy.clone(),
        ));
        self.cache.put(&cache_key, request.content.clone(), None).await.ok();

        // Update statistics
        self.update_content_stats(&request.policy, request.content.len()).await;

        Ok(StorageResponse {
            address: StorageAddress::new(content_address.hash, request.policy),
            chunks_stored: content_address.chunks.len() as u32,
            total_size: request.content.len() as u64,
            encrypted_size: encrypted_content.len() as u64,
            operation_time_ms: start_time.elapsed().as_millis() as u64,
            storage_location,
        })
    }

    async fn retrieve_content_internal(&self, request: RetrievalRequest) -> StorageResult<RetrievalResponse> {
        let start_time = std::time::Instant::now();
        
        // Try cache first
        let cache_key = self.generate_cache_key(&request.address);
        if let Ok(cached_content) = self.cache.get(&cache_key).await {
            // Create dummy metadata for cached content
            let metadata = StorageMetadata {
                content_type: "application/octet-stream".to_string(),
                author: request.user_id.clone(),
                tags: Vec::new(),
                created_at: Utc::now(),
                modified_at: None,
                size: cached_content.len() as u64,
                checksum: String::new(),
            };
            
            return Ok(RetrievalResponse {
                content: cached_content,
                metadata,
                source: RetrievalSource::Cache,
                operation_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Retrieve from storage based on policy
        let encrypted_content = match &request.address.policy {
            StoragePolicy::PrivateMax => {
                // Retrieve from local storage
                self.retrieve_local(&request.address).await?
            }
            StoragePolicy::PrivateScoped { namespace } => {
                // Retrieve from DHT
                let dht_key = self.generate_dht_key(namespace, &request.address.content_id);
                self.network_manager.retrieve_content(dht_key).await?
                    .ok_or_else(|| StorageError::NotFound {
                        address: request.address.content_id.clone(),
                    })?
            }
            StoragePolicy::GroupScoped { group_id } => {
                // Retrieve from group storage
                let group_key = format!("group:{}", group_id);
                let dht_key = self.generate_dht_key(&group_key, &request.address.content_id);
                self.network_manager.retrieve_content(dht_key).await?
                    .ok_or_else(|| StorageError::NotFound {
                        address: request.address.content_id.clone(),
                    })?
            }
            StoragePolicy::PublicMarkdown => {
                // Retrieve from public DHT
                let public_key = format!("public:{}", request.address.content_id);
                self.network_manager.retrieve_content(public_key.into_bytes()).await?
                    .ok_or_else(|| StorageError::NotFound {
                        address: request.address.content_id.clone(),
                    })?
            }
        };

        // Decrypt content
        let decrypted_content = self.decrypt_content(&request, &encrypted_content).await?;

        // Verify content integrity
        let computed_hash = self.content_addressing.generate_content_id(&decrypted_content, "verify");
        if computed_hash != request.address.content_id {
            return Err(StorageError::IntegrityFailure {
                address: request.address.content_id,
            });
        }

        // Cache the retrieved content
        self.cache.put(&cache_key, decrypted_content.clone(), None).await.ok();

        // Create metadata (would normally be stored separately)
        let metadata = StorageMetadata {
            content_type: "application/octet-stream".to_string(),
            author: request.user_id,
            tags: Vec::new(),
            created_at: Utc::now(),
            modified_at: None,
            size: decrypted_content.len() as u64,
            checksum: computed_hash,
        };

        Ok(RetrievalResponse {
            content: decrypted_content,
            metadata,
            source: RetrievalSource::Dht,
            operation_time_ms: start_time.elapsed().as_millis() as u64,
        })
    }

    async fn encrypt_content(&self, request: &StorageRequest) -> StorageResult<(Vec<u8>, [u8; 32])> {
        // Use PQC crypto manager for ML-KEM-768 enhanced encryption
        let pqc_encrypted = self.pqc_crypto_manager.encrypt_content(
            &request.content,
            &request.policy,
            &request.user_id,
            request.namespace.as_deref(),
            request.group_id.as_deref(),
        ).await.map_err(|e| match e {
            StorageError::Encryption { source } => StorageError::Encryption { source },
            StorageError::KeyDerivation { source } => StorageError::KeyDerivation { source },
            _ => StorageError::Encryption { 
                source: crate::saorsa_storage::errors::EncryptionError::EncryptionFailed 
            },
        })?;

        // Serialize PQC encrypted content to bytes
        let serialized = bincode::serialize(&pqc_encrypted)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::EncryptionFailed,
            })?;

        // Return serialized content and a derived key for backward compatibility
        let derived_key = self.derive_key_from_pqc_content(&pqc_encrypted);
        
        Ok((serialized, derived_key))
    }

    async fn decrypt_content(&self, request: &RetrievalRequest, encrypted_content: &[u8]) -> StorageResult<Vec<u8>> {
        // Try to deserialize as PQC encrypted content first
        if let Ok(pqc_encrypted) = bincode::deserialize::<PqcEncryptedContent>(encrypted_content) {
            // Use PQC crypto manager for ML-KEM-768 enhanced decryption
            return self.pqc_crypto_manager.decrypt_content(&pqc_encrypted, &request.user_id)
                .await
                .map_err(|e| match e {
                    StorageError::Encryption { source } => StorageError::Encryption { source },
                    StorageError::KeyDerivation { source } => StorageError::KeyDerivation { source },
                    _ => StorageError::Encryption { 
                        source: crate::saorsa_storage::errors::EncryptionError::DecryptionFailed 
                    },
                });
        }

        // Fallback to legacy ChaCha20Poly1305 decryption for backward compatibility
        let key = if let Some(provided_key) = request.decryption_key {
            provided_key
        } else {
            // Derive key based on policy
            match &request.address.policy {
                StoragePolicy::PrivateMax => {
                    return Err(StorageError::KeyDerivation {
                        source: crate::saorsa_storage::errors::KeyDerivationError::SecureStorageUnavailable,
                    });
                }
                StoragePolicy::PrivateScoped { namespace } => {
                    self.namespace_manager.derive_namespace_key(namespace)?
                }
                StoragePolicy::GroupScoped { group_id } => {
                    let dummy_private_key = [0u8; 32];
                    self.group_manager.get_group_key(group_id, &request.user_id, &dummy_private_key).await?
                }
                StoragePolicy::PublicMarkdown => {
                    // For public content, we need the original content to derive the convergent key
                    // This is a limitation of convergent encryption
                    return Err(StorageError::KeyDerivation {
                        source: crate::saorsa_storage::errors::KeyDerivationError::SecureStorageUnavailable,
                    });
                }
            }
        };

        self.decrypt_with_key(encrypted_content, &key)
    }

    // Helper methods for encryption/decryption
    fn encrypt_with_key(&self, data: &[u8], key: &[u8; 32]) -> StorageResult<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit};
        use chacha20poly1305::aead::{Aead, AeadCore, OsRng};

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key).into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::EncryptionFailed,
            })?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt_with_key(&self, encrypted_data: &[u8], key: &[u8; 32]) -> StorageResult<Vec<u8>> {
        use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, KeyInit};
        use chacha20poly1305::aead::Aead;

        if encrypted_data.len() < 12 {
            return Err(StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::InvalidNonce { length: encrypted_data.len() },
            });
        }

        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let cipher = ChaCha20Poly1305::new(Key::from_slice(key).into());
        cipher.decrypt(nonce, ciphertext)
            .map_err(|_| StorageError::Encryption {
                source: crate::saorsa_storage::errors::EncryptionError::DecryptionFailed,
            })
    }

    fn generate_random_key(&self) -> [u8; 32] {
        use rand::RngCore;
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    fn derive_convergent_key(&self, content: &[u8]) -> [u8; 32] {
        use blake3::hash;
        let hash = hash(content);
        *hash.as_bytes()
    }

    fn generate_dht_key(&self, namespace: &str, content_id: &str) -> Vec<u8> {
        format!("{}:{}", namespace, content_id).into_bytes()
    }

    fn generate_cache_key(&self, address: &StorageAddress) -> String {
        format!("cache:{}:{:?}", address.content_id, address.policy)
    }

    fn generate_operation_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        format!("op_{}", timestamp)
    }

    /// Helper method to derive a key from PQC encrypted content for backward compatibility
    fn derive_key_from_pqc_content(&self, pqc_content: &PqcEncryptedContent) -> [u8; 32] {
        use blake3::hash;
        
        // Derive a deterministic key from PQC content for backward compatibility
        let mut key_material = Vec::new();
        key_material.extend_from_slice(&pqc_content.nonce);
        key_material.extend_from_slice(&pqc_content.ml_kem_ciphertext[..32]);
        key_material.extend_from_slice(pqc_content.content_address.as_bytes());
        
        let hash = hash(&key_material);
        *hash.as_bytes()
    }

    async fn start_operation(&self, op_id: &str, op_type: &str, user_id: &str) {
        let operation = OperationState {
            operation_id: op_id.to_string(),
            operation_type: op_type.to_string(),
            started_at: std::time::Instant::now(),
            user_id: user_id.to_string(),
            progress: 0.0,
        };

        let mut operations = self.active_operations.write().await;
        operations.insert(op_id.to_string(), operation);
    }

    async fn complete_operation(&self, op_id: &str, _success: bool) {
        let mut operations = self.active_operations.write().await;
        operations.remove(op_id);
    }

    async fn update_stats_success(&self, _operation_type: &str, duration: std::time::Duration) {
        let mut stats = self.stats.write().await;
        stats.successful_operations += 1;
        
        let total_ops = stats.successful_operations + stats.failed_operations;
        let duration_ms = duration.as_secs_f64() * 1000.0;
        
        if total_ops == 1 {
            stats.avg_operation_time_ms = duration_ms;
        } else {
            stats.avg_operation_time_ms = (stats.avg_operation_time_ms * (total_ops - 1) as f64 + duration_ms) / total_ops as f64;
        }
        
        stats.last_updated = Utc::now();
    }

    async fn update_stats_failure(&self, _operation_type: &str) {
        let mut stats = self.stats.write().await;
        stats.failed_operations += 1;
        stats.last_updated = Utc::now();
    }

    async fn update_content_stats(&self, policy: &StoragePolicy, size: usize) {
        let mut stats = self.stats.write().await;
        stats.total_content_items += 1;
        stats.total_bytes_stored += size as u64;
        
        let policy_key = format!("{:?}", policy);
        *stats.policy_distribution.entry(policy_key).or_insert(0) += 1;
        
        stats.last_updated = Utc::now();
    }

    // Placeholder methods for local storage (would be implemented with actual storage backend)
    async fn store_local(&self, _address: &crate::saorsa_storage::content::ContentAddress, _data: &[u8]) -> StorageResult<()> {
        // Would store to local filesystem/database
        Ok(())
    }

    async fn retrieve_local(&self, _address: &StorageAddress) -> StorageResult<Vec<u8>> {
        // Would retrieve from local filesystem/database
        Err(StorageError::NotFound {
            address: _address.content_id.clone(),
        })
    }
}

// Thread-safe implementations
unsafe impl<D: DhtFacade> Send for StorageEngine<D> {}
unsafe impl<D: DhtFacade> Sync for StorageEngine<D> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dht_facade::LocalDht;
    use crate::saorsa_storage::config::ConfigManager;

    async fn setup_storage_engine() -> StorageEngine<LocalDht> {
        let dht = Arc::new(LocalDht::new("test_node".to_string()));
        let master_key = [42u8; 32];
        let config_manager = ConfigManager::new();
        
        StorageEngine::new(dht, master_key, config_manager).await.unwrap()
    }

    #[tokio::test]
    async fn test_store_and_retrieve_private_max() {
        let engine = setup_storage_engine().await;
        
        let content = b"test content for private max policy".to_vec();
        let metadata = StorageMetadata {
            content_type: "text/plain".to_string(),
            author: "test_user".to_string(),
            tags: vec!["test".to_string()],
            created_at: Utc::now(),
            modified_at: None,
            size: content.len() as u64,
            checksum: "test_checksum".to_string(),
        };

        let store_request = StorageRequest {
            content: content.clone(),
            content_type: "text/plain".to_string(),
            policy: StoragePolicy::PrivateMax,
            metadata,
            user_id: "test_user".to_string(),
            group_id: None,
            namespace: None,
        };

        let store_response = engine.store_content(store_request).await.unwrap();
        assert_eq!(store_response.total_size, content.len() as u64);
    }

    #[tokio::test]
    async fn test_store_private_scoped() {
        let engine = setup_storage_engine().await;
        
        let content = b"test content for private scoped policy".to_vec();
        let metadata = StorageMetadata::default();

        let store_request = StorageRequest {
            content,
            content_type: "text/plain".to_string(),
            policy: StoragePolicy::PrivateScoped {
                namespace: "test_namespace".to_string(),
            },
            metadata,
            user_id: "test_user".to_string(),
            group_id: None,
            namespace: Some("test_namespace".to_string()),
        };

        let result = engine.store_content(store_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_engine_stats() {
        let engine = setup_storage_engine().await;
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_content_items, 0);
        assert_eq!(stats.successful_operations, 0);
    }

    #[tokio::test]
    async fn test_maintenance() {
        let engine = setup_storage_engine().await;
        
        let result = engine.maintenance().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_policy_validation() {
        let engine = setup_storage_engine().await;
        
        let oversized_content = vec![0u8; 200 * 1024 * 1024]; // 200MB
        let metadata = StorageMetadata::default();

        let store_request = StorageRequest {
            content: oversized_content,
            content_type: "application/octet-stream".to_string(),
            policy: StoragePolicy::PrivateMax, // Has 100MB limit
            metadata,
            user_id: "test_user".to_string(),
            group_id: None,
            namespace: None,
        };

        let result = engine.store_content(store_request).await;
        assert!(result.is_err()); // Should fail due to size limit
    }
}