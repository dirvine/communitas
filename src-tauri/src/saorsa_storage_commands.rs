/**
 * Saorsa Storage System - Tauri Commands
 * Exposes storage functionality to the frontend through Tauri commands
 */

use crate::saorsa_storage::{
    StorageEngine, StorageRequest, StorageResponse, RetrievalRequest, RetrievalResponse,
    StoragePolicy, StorageMetadata, StorageAddress, StorageEngineStats,
    ConfigManager, run_performance_smoke_test, run_comprehensive_performance_test, PerformanceTestRunner,
};
use crate::saorsa_storage::errors::StorageError;
use crate::dht_facade::{DhtFacade, LocalDht};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tauri::State;
use chrono::Utc;

/// Global storage engine state
pub struct StorageEngineState<D: DhtFacade + Send + Sync> {
    engine: Arc<RwLock<Option<StorageEngine<D>>>>,
}

impl<D: DhtFacade + Send + Sync> StorageEngineState<D> {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(RwLock::new(None)),
        }
    }
}

/// Storage initialization parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInitRequest {
    pub master_key_hex: String,
    pub config_path: Option<String>,
}

/// Storage operation request from frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct FrontendStorageRequest {
    pub content: Vec<u8>,
    pub content_type: String,
    pub policy: StoragePolicy,
    pub author: String,
    pub tags: Vec<String>,
    pub user_id: String,
    pub group_id: Option<String>,
    pub namespace: Option<String>,
}

/// Retrieval request from frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct FrontendRetrievalRequest {
    pub address: StorageAddress,
    pub user_id: String,
    pub decryption_key_hex: Option<String>,
}

/// Content listing request
#[derive(Debug, Serialize, Deserialize)]
pub struct ContentListRequest {
    pub user_id: String,
    pub policy_filter: Option<StoragePolicy>,
    pub limit: Option<u32>,
}

/// Error response for frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageErrorResponse {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
}

impl From<StorageError> for StorageErrorResponse {
    fn from(error: StorageError) -> Self {
        Self {
            error_type: format!("{:?}", std::mem::discriminant(&error)),
            message: error.to_string(),
            details: Some(format!("{:#?}", error)),
        }
    }
}

/// Initialize the storage engine
#[tauri::command]
pub async fn init_storage_engine(
    request: StorageInitRequest,
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<bool, StorageErrorResponse> {
    // Parse master key
    let master_key_bytes = hex::decode(&request.master_key_hex)
        .map_err(|e| StorageErrorResponse {
            error_type: "InvalidMasterKey".to_string(),
            message: format!("Invalid master key hex: {}", e),
            details: None,
        })?;

    if master_key_bytes.len() != 32 {
        return Err(StorageErrorResponse {
            error_type: "InvalidMasterKey".to_string(),
            message: format!("Master key must be 32 bytes, got {}", master_key_bytes.len()),
            details: None,
        });
    }

    let mut master_key = [0u8; 32];
    master_key.copy_from_slice(&master_key_bytes);

    // Initialize DHT (using LocalDht for now)
    let dht = Arc::new(LocalDht::new("storage_node".to_string()));

    // Initialize config manager
    let config_manager = if let Some(config_path) = request.config_path {
        ConfigManager::load_from_file(&config_path).map_err(StorageError::from)?
    } else {
        ConfigManager::new()
    };

    // Create storage engine
    let engine = StorageEngine::new(dht, master_key, config_manager)
        .await
        .map_err(StorageErrorResponse::from)?;

    // Store in state
    let mut engine_guard = state.engine.write().await;
    *engine_guard = Some(engine);

    Ok(true)
}

/// Store content using the storage engine
#[tauri::command]
pub async fn store_content(
    request: FrontendStorageRequest,
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<StorageResponse, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    // Create metadata
    let metadata = StorageMetadata {
        content_type: request.content_type.clone(),
        author: request.author,
        tags: request.tags,
        created_at: Utc::now(),
        modified_at: None,
        size: request.content.len() as u64,
        checksum: blake3::hash(&request.content).to_hex().to_string(),
    };

    // Create storage request
    let storage_request = StorageRequest {
        content: request.content,
        content_type: request.content_type,
        policy: request.policy,
        metadata,
        user_id: request.user_id,
        group_id: request.group_id,
        namespace: request.namespace,
    };

    engine.store_content(storage_request)
        .await
        .map_err(StorageErrorResponse::from)
}

/// Retrieve content by address
#[tauri::command]
pub async fn retrieve_content(
    request: FrontendRetrievalRequest,
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<RetrievalResponse, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    // Parse decryption key if provided
    let decryption_key = if let Some(key_hex) = request.decryption_key_hex {
        let key_bytes = hex::decode(&key_hex)
            .map_err(|e| StorageErrorResponse {
                error_type: "InvalidDecryptionKey".to_string(),
                message: format!("Invalid decryption key hex: {}", e),
                details: None,
            })?;

        if key_bytes.len() != 32 {
            return Err(StorageErrorResponse {
                error_type: "InvalidDecryptionKey".to_string(),
                message: format!("Decryption key must be 32 bytes, got {}", key_bytes.len()),
                details: None,
            });
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        Some(key)
    } else {
        None
    };

    // Create retrieval request
    let retrieval_request = RetrievalRequest {
        address: request.address,
        user_id: request.user_id,
        decryption_key,
    };

    engine.retrieve_content(retrieval_request)
        .await
        .map_err(StorageErrorResponse::from)
}

/// List content for a user
#[tauri::command]
pub async fn list_content(
    request: ContentListRequest,
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<Vec<StorageAddress>, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    engine.list_content(&request.user_id, request.policy_filter, request.limit)
        .await
        .map_err(StorageErrorResponse::from)
}

/// Delete content by address
#[tauri::command]
pub async fn delete_content(
    address: StorageAddress,
    user_id: String,
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<bool, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    engine.delete_content(&address, &user_id)
        .await
        .map_err(StorageErrorResponse::from)
}

/// Get storage engine statistics
#[tauri::command]
pub async fn get_storage_stats(
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<StorageEngineStats, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    Ok(engine.get_stats().await)
}

/// Transition content to new policy
#[tauri::command]
pub async fn transition_content_policy(
    address: StorageAddress,
    new_policy: StoragePolicy,
    user_id: String,
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<StorageAddress, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    engine.transition_policy(&address, new_policy, &user_id)
        .await
        .map_err(StorageErrorResponse::from)
}

/// Perform storage maintenance
#[tauri::command]
pub async fn perform_storage_maintenance(
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<bool, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    let engine = engine_guard.as_ref().ok_or_else(|| StorageErrorResponse {
        error_type: "EngineNotInitialized".to_string(),
        message: "Storage engine not initialized".to_string(),
        details: None,
    })?;

    engine.maintenance()
        .await
        .map_err(StorageErrorResponse::from)?;

    Ok(true)
}

/// Validate storage policy
#[tauri::command]
pub async fn validate_storage_policy(
    policy: StoragePolicy,
    content_size: u64,
    user_id: String,
    content_type: String,
) -> Result<bool, StorageErrorResponse> {
    // Basic validation without requiring engine state
    
    // Check content size limits
    if let Some(max_size) = policy.max_content_size() {
        if content_size > max_size {
            return Err(StorageErrorResponse {
                error_type: "ContentTooLarge".to_string(),
                message: format!("Content size {} exceeds policy limit {}", content_size, max_size),
                details: None,
            });
        }
    }

    // Check binary content restrictions
    if !policy.allows_binary_content() && content_type != "text/markdown" {
        return Err(StorageErrorResponse {
            error_type: "BinaryContentNotAllowed".to_string(),
            message: "Policy does not allow binary content".to_string(),
            details: None,
        });
    }

    // Validate user ID format (basic check)
    if user_id.is_empty() || user_id.len() > 256 {
        return Err(StorageErrorResponse {
            error_type: "InvalidUserId".to_string(),
            message: "User ID must be non-empty and less than 256 characters".to_string(),
            details: None,
        });
    }

    Ok(true)
}

/// Generate a random master key for initialization
#[tauri::command]
pub async fn generate_master_key() -> Result<String, StorageErrorResponse> {
    use rand::RngCore;
    
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    
    Ok(hex::encode(key))
}

/// Check if storage engine is initialized
#[tauri::command]
pub async fn is_storage_initialized(
    state: State<'_, StorageEngineState<LocalDht>>,
) -> Result<bool, StorageErrorResponse> {
    let engine_guard = state.engine.read().await;
    Ok(engine_guard.is_some())
}

/// Run performance smoke test (quick validation)
#[tauri::command]
pub async fn run_storage_performance_smoke_test() -> Result<bool, StorageErrorResponse> {
    run_performance_smoke_test()
        .await
        .map_err(|e| StorageErrorResponse {
            error_type: "PerformanceTestFailed".to_string(),
            message: format!("Performance smoke test failed: {}", e),
            details: Some(format!("{:#?}", e)),
        })
}

/// Run comprehensive performance test (full validation)
#[tauri::command]
pub async fn run_storage_performance_comprehensive_test() -> Result<bool, StorageErrorResponse> {
    run_comprehensive_performance_test()
        .await
        .map_err(|e| StorageErrorResponse {
            error_type: "PerformanceTestFailed".to_string(),
            message: format!("Comprehensive performance test failed: {}", e),
            details: Some(format!("{:#?}", e)),
        })
}

/// Performance test configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceTestConfig {
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub content_sizes: Vec<usize>,
    pub local_target_ms: u64,
    pub remote_target_ms: u64,
}

impl Default for PerformanceTestConfig {
    fn default() -> Self {
        Self {
            iterations: 50,
            warmup_iterations: 5,
            content_sizes: vec![1024, 10 * 1024, 100 * 1024],
            local_target_ms: 100,
            remote_target_ms: 500,
        }
    }
}

/// Performance test result summary
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceTestResultSummary {
    pub overall_passed: bool,
    pub passed_tests: usize,
    pub total_tests: usize,
    pub total_duration_ms: u64,
    pub failed_operations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Run custom performance test with configuration
#[tauri::command]
pub async fn run_storage_performance_test_custom(
    config: PerformanceTestConfig,
) -> Result<PerformanceTestResultSummary, StorageErrorResponse> {
    use crate::saorsa_storage::benchmarks::BenchmarkConfig;
    
    let benchmark_config = BenchmarkConfig {
        iterations: config.iterations,
        warmup_iterations: config.warmup_iterations,
        content_sizes: config.content_sizes,
        local_target_ms: config.local_target_ms,
        remote_target_ms: config.remote_target_ms,
    };

    let mut runner = PerformanceTestRunner::with_config(benchmark_config);
    
    let report = runner.run_performance_validation()
        .await
        .map_err(|e| StorageErrorResponse {
            error_type: "PerformanceTestFailed".to_string(),
            message: format!("Custom performance test failed: {}", e),
            details: Some(format!("{:#?}", e)),
        })?;

    let failed_operations: Vec<String> = report.validation_results.validations
        .iter()
        .filter(|v| !v.passed)
        .map(|v| format!("{} ({}ms > {}ms)", v.operation, v.actual_p95_ms, v.target_ms))
        .collect();

    let recommendations: Vec<String> = if let Some(recs) = report.recommendations {
        recs.recommendations
            .iter()
            .take(5) // Top 5 recommendations
            .map(|r| r.title.clone())
            .collect()
    } else {
        Vec::new()
    };

    Ok(PerformanceTestResultSummary {
        overall_passed: report.validation_results.overall_passed,
        passed_tests: report.validation_results.passed_count,
        total_tests: report.validation_results.total_count,
        total_duration_ms: report.total_duration.as_millis() as u64,
        failed_operations,
        recommendations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_storage_error_response_conversion() {
        let storage_error = StorageError::NotFound {
            address: "test_address".to_string(),
        };
        
        let error_response = StorageErrorResponse::from(storage_error);
        assert!(error_response.message.contains("test_address"));
        assert!(error_response.error_type.contains("NotFound"));
    }
    
    #[test]
    fn test_master_key_validation() {
        // Test valid key
        let valid_key = "a".repeat(64); // 32 bytes in hex
        assert_eq!(hex::decode(&valid_key).unwrap().len(), 32);
        
        // Test invalid key
        let invalid_key = "a".repeat(30); // 15 bytes in hex
        assert_ne!(hex::decode(&invalid_key).unwrap().len(), 32);
    }
    
    #[tokio::test]
    async fn test_policy_validation() {
        let result = validate_storage_policy(
            StoragePolicy::PrivateMax,
            1000,
            "test_user".to_string(),
            "text/plain".to_string(),
        ).await;
        
        assert!(result.is_ok());
        
        // Test content too large
        let result = validate_storage_policy(
            StoragePolicy::PrivateMax,
            200 * 1024 * 1024, // 200MB, exceeds 100MB limit
            "test_user".to_string(),
            "text/plain".to_string(),
        ).await;
        
        assert!(result.is_err());
    }
}