//! Secure storage wrapper that provides additional security layers
//!
//! This module provides:
//! - Additional validation before storing to keyring
//! - Secure data sanitization
//! - Access logging for security audits
//! - Rate limiting for storage operations

use super::input_validation::InputValidator;
use super::rate_limiter::RateLimiter;
use crate::secure_storage::{SecureKeyMetadata, SecureStorageManager};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Enhanced secure storage with additional security layers
#[derive(Debug)]
pub struct EnhancedSecureStorage {
    storage_manager: SecureStorageManager,
    input_validator: InputValidator,
    rate_limiter: Arc<RateLimiter>,
}

impl EnhancedSecureStorage {
    /// Create a new enhanced secure storage instance
    pub fn new(user_id: String) -> Self {
        Self {
            storage_manager: SecureStorageManager::new(user_id),
            input_validator: InputValidator::new(),
            rate_limiter: Arc::new(RateLimiter::with_limit(20, Duration::from_secs(60))), // 20 operations per minute
        }
    }

    /// Store encryption keys with additional validation and rate limiting
    pub async fn store_encryption_keys_secure(
        &self,
        user_id: &str,
        master_key: &str,
        key_pair: &str,
    ) -> Result<()> {
        // Rate limiting check
        if !self.rate_limiter.is_allowed(user_id)? {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded for secure storage operations"
            ));
        }

        // Input validation
        self.input_validator.sanitize_string(master_key, 10000)?;
        self.input_validator.sanitize_string(key_pair, 10000)?;

        // Additional security checks
        if master_key.len() < 32 {
            return Err(anyhow::anyhow!(
                "Master key too short for security requirements"
            ));
        }

        // Log access for security audit
        info!(
            "Secure storage: Storing encryption keys for user: {}",
            user_id
        );

        // Store using the underlying storage manager
        self.storage_manager
            .store_encryption_keys(master_key, key_pair)
            .await
            .context("Failed to store encryption keys in secure storage")
    }

    /// Retrieve encryption keys with rate limiting and audit logging
    pub async fn get_encryption_keys_secure(&self, user_id: &str) -> Result<serde_json::Value> {
        // Rate limiting check
        if !self.rate_limiter.is_allowed(user_id)? {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded for secure storage operations"
            ));
        }

        // Log access for security audit
        info!(
            "Secure storage: Retrieving encryption keys for user: {}",
            user_id
        );

        self.storage_manager
            .get_encryption_keys()
            .await
            .context("Failed to retrieve encryption keys from secure storage")
    }

    /// Store derived key with enhanced security
    pub async fn store_derived_key_secure(
        &self,
        user_id: &str,
        key_id: &str,
        key_data: &str,
        metadata: &SecureKeyMetadata,
    ) -> Result<()> {
        // Rate limiting check
        if !self.rate_limiter.is_allowed(user_id)? {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded for secure storage operations"
            ));
        }

        // Validate key ID format
        self.input_validator.sanitize_string(key_id, 100)?;

        // Validate key data
        self.input_validator.sanitize_string(key_data, 10000)?;

        // Log access for security audit
        info!(
            "Secure storage: Storing derived key {} for user: {}",
            key_id, user_id
        );

        self.storage_manager
            .store_derived_key(key_id, key_data, metadata)
            .await
            .context("Failed to store derived key in secure storage")
    }

    /// Retrieve derived key with enhanced security
    pub async fn get_derived_key_secure(
        &self,
        user_id: &str,
        key_id: &str,
    ) -> Result<(String, SecureKeyMetadata)> {
        // Rate limiting check
        if !self.rate_limiter.is_allowed(user_id)? {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded for secure storage operations"
            ));
        }

        // Validate key ID
        self.input_validator.sanitize_string(key_id, 100)?;

        // Log access for security audit
        info!(
            "Secure storage: Retrieving derived key {} for user: {}",
            key_id, user_id
        );

        self.storage_manager
            .get_derived_key(key_id)
            .await
            .context("Failed to retrieve derived key from secure storage")
    }

    /// Clear all keys for a user with enhanced security
    pub async fn delete_all_keys_secure(&self, user_id: &str) -> Result<()> {
        // Rate limiting check - use stricter limit for destructive operations
        if !self.rate_limiter.check_rate_limit(user_id, 5)? {
            // Only 5 delete operations per hour
            return Err(anyhow::anyhow!(
                "Rate limit exceeded for destructive storage operations"
            ));
        }

        // Log destructive operation for security audit
        warn!("Secure storage: Deleting all keys for user: {}", user_id);

        self.storage_manager
            .delete_all_keys()
            .await
            .context("Failed to delete all keys from secure storage")
    }

    /// Check if secure storage is available
    pub fn is_available() -> bool {
        SecureStorageManager::is_available()
    }

    /// Get storage backend information
    pub fn get_storage_info() -> String {
        SecureStorageManager::get_storage_info()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio_test::tokio::test]
    async fn test_enhanced_secure_storage_rate_limiting() {
        let storage = EnhancedSecureStorage::new("test_user".to_string());
        let user_id = "test_user";

        // Should work for first few requests
        for i in 0..5 {
            let result = storage.rate_limiter.is_allowed(user_id);
            assert!(result.unwrap(), "Request {} should be allowed", i);
        }
    }

    #[tokio_test::tokio::test]
    async fn test_input_validation() {
        let storage = EnhancedSecureStorage::new("test_user".to_string());

        // Test key validation
        assert!(
            storage
                .input_validator
                .sanitize_string("valid_key", 100)
                .is_ok()
        );
        assert!(storage.input_validator.sanitize_string("", 100).is_err()); // Empty string
        assert!(
            storage
                .input_validator
                .sanitize_string(&"x".repeat(101), 100)
                .is_err()
        ); // Too long
    }
}
