/**
 * Saorsa Storage System - Policy Management
 * Implements storage policy validation, enforcement, and transitions
 */
use crate::saorsa_storage::errors::*;
use crate::saorsa_storage::{DeduplicationScope, EncryptionMode, StoragePolicy};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Policy enforcement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub max_content_size_mb: u64,
    pub allow_policy_transitions: bool,
    pub require_audit_trail: bool,
    pub encryption_required: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            max_content_size_mb: 1024, // 1GB default
            allow_policy_transitions: true,
            require_audit_trail: true,
            encryption_required: true,
        }
    }
}

/// Policy enforcement record for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEnforcementRecord {
    pub policy: StoragePolicy,
    pub content_id: String,
    pub enforced_at: DateTime<Utc>,
    pub user_id: String,
    pub action: String,
    pub success: bool,
    pub reason: Option<String>,
}

/// Policy transition plan
#[derive(Debug, Clone)]
pub struct PolicyTransition {
    pub from_policy: StoragePolicy,
    pub to_policy: StoragePolicy,
    pub requires_re_encryption: bool,
    pub requires_key_migration: bool,
    pub estimated_cost: f64, // Relative cost estimate
}

/// Storage policy manager
pub struct PolicyManager {
    config: PolicyConfig,
    enforcement_log: Arc<RwLock<Vec<PolicyEnforcementRecord>>>,
    policy_cache: Arc<RwLock<HashMap<String, StoragePolicy>>>,
    transition_rules: HashMap<(String, String), bool>,
}

impl PolicyManager {
    /// Create a new policy manager with default configuration
    pub fn new() -> Self {
        let mut transition_rules = HashMap::new();

        // Define allowed policy transitions
        // PrivateMax -> PrivateScoped: Allowed (relaxing security)
        transition_rules.insert(
            ("PrivateMax".to_string(), "PrivateScoped".to_string()),
            true,
        );

        // PrivateScoped -> GroupScoped: Allowed (sharing with group)
        transition_rules.insert(
            ("PrivateScoped".to_string(), "GroupScoped".to_string()),
            true,
        );

        // GroupScoped -> PublicMarkdown: Allowed (making public)
        transition_rules.insert(
            ("GroupScoped".to_string(), "PublicMarkdown".to_string()),
            true,
        );

        // Reverse transitions (tightening security): Generally not allowed
        transition_rules.insert(
            ("PublicMarkdown".to_string(), "GroupScoped".to_string()),
            false,
        );
        transition_rules.insert(
            ("GroupScoped".to_string(), "PrivateScoped".to_string()),
            false,
        );
        transition_rules.insert(
            ("PrivateScoped".to_string(), "PrivateMax".to_string()),
            false,
        );

        Self {
            config: PolicyConfig::default(),
            enforcement_log: Arc::new(RwLock::new(Vec::new())),
            policy_cache: Arc::new(RwLock::new(HashMap::new())),
            transition_rules,
        }
    }

    /// Create policy manager with custom configuration
    pub fn with_config(config: PolicyConfig) -> Self {
        let mut manager = Self::new();
        manager.config = config;
        manager
    }

    /// Validate a storage policy for a given context
    pub async fn validate_policy(
        &self,
        policy: &StoragePolicy,
        content_size: u64,
        user_id: &str,
        content_type: &str,
    ) -> PolicyResult<()> {
        // Size validation
        if let Some(max_size) = policy.max_content_size() {
            if content_size > max_size {
                return Err(PolicyError::ValidationFailed {
                    reason: format!(
                        "Content size {} exceeds policy limit {}",
                        content_size, max_size
                    ),
                });
            }
        }

        // Content type validation for PublicMarkdown
        if let StoragePolicy::PublicMarkdown = policy {
            if !self.is_markdown_content(content_type) {
                return Err(PolicyError::ValidationFailed {
                    reason: "PublicMarkdown policy only allows markdown content".to_string(),
                });
            }
        }

        // Namespace validation for PrivateScoped
        if let StoragePolicy::PrivateScoped { namespace } = policy {
            if namespace.is_empty() {
                return Err(PolicyError::MissingParameter {
                    parameter: "namespace".to_string(),
                });
            }

            // Validate namespace format
            if !self.is_valid_namespace(namespace) {
                return Err(PolicyError::ValidationFailed {
                    reason: format!("Invalid namespace format: {}", namespace),
                });
            }
        }

        // Group validation for GroupScoped
        if let StoragePolicy::GroupScoped { group_id } = policy {
            if group_id.is_empty() {
                return Err(PolicyError::MissingParameter {
                    parameter: "group_id".to_string(),
                });
            }

            // Validate user has access to group
            if !self.validate_group_access(user_id, group_id).await? {
                return Err(PolicyError::ValidationFailed {
                    reason: format!(
                        "User {} does not have access to group {}",
                        user_id, group_id
                    ),
                });
            }
        }

        Ok(())
    }

    /// Enforce policy constraints on content storage
    pub async fn enforce_policy(
        &self,
        policy: &StoragePolicy,
        content_id: &str,
        content: &[u8],
        user_id: &str,
        action: &str,
    ) -> PolicyResult<()> {
        let start_time = std::time::Instant::now();

        // Basic policy validation
        self.validate_policy(
            policy,
            content.len() as u64,
            user_id,
            "application/octet-stream",
        )
        .await?;

        // Encryption requirement enforcement
        if self.config.encryption_required && !self.policy_requires_encryption(policy) {
            return Err(PolicyError::EnforcementFailed {
                reason: "Encryption is required but policy does not enforce it".to_string(),
            });
        }

        // Content integrity checks
        if !self.validate_content_integrity(content) {
            return Err(PolicyError::EnforcementFailed {
                reason: "Content integrity validation failed".to_string(),
            });
        }

        // Record enforcement
        let record = PolicyEnforcementRecord {
            policy: policy.clone(),
            content_id: content_id.to_string(),
            enforced_at: Utc::now(),
            user_id: user_id.to_string(),
            action: action.to_string(),
            success: true,
            reason: None,
        };

        if self.config.require_audit_trail {
            let mut log = self.enforcement_log.write().await;
            log.push(record);
        }

        // Cache the policy for this content
        let mut cache = self.policy_cache.write().await;
        cache.insert(content_id.to_string(), policy.clone());

        // Performance monitoring
        let duration = start_time.elapsed();
        if duration.as_millis() > 100 {
            // Log slow enforcement
            tracing::warn!(
                "Slow policy enforcement: {}ms for {}",
                duration.as_millis(),
                content_id
            );
        }

        Ok(())
    }

    /// Check if a policy transition is allowed
    pub fn is_transition_allowed(&self, from: &StoragePolicy, to: &StoragePolicy) -> bool {
        let from_type = self.policy_type_string(from);
        let to_type = self.policy_type_string(to);

        self.transition_rules
            .get(&(from_type, to_type))
            .copied()
            .unwrap_or(false)
    }

    /// Plan a policy transition
    pub fn plan_transition(
        &self,
        from: &StoragePolicy,
        to: &StoragePolicy,
    ) -> PolicyResult<PolicyTransition> {
        if !self.is_transition_allowed(from, to) {
            return Err(PolicyError::TransitionNotAllowed {
                from: self.policy_type_string(from),
                to: self.policy_type_string(to),
            });
        }

        let requires_re_encryption = from.encryption_mode() != to.encryption_mode();
        let requires_key_migration = self.requires_key_migration(from, to);
        let estimated_cost = self.calculate_transition_cost(from, to);

        Ok(PolicyTransition {
            from_policy: from.clone(),
            to_policy: to.clone(),
            requires_re_encryption,
            requires_key_migration,
            estimated_cost,
        })
    }

    /// Get policy for content if cached
    pub async fn get_cached_policy(&self, content_id: &str) -> Option<StoragePolicy> {
        let cache = self.policy_cache.read().await;
        cache.get(content_id).cloned()
    }

    /// Get enforcement history for content
    pub async fn get_enforcement_history(&self, content_id: &str) -> Vec<PolicyEnforcementRecord> {
        let log = self.enforcement_log.read().await;
        log.iter()
            .filter(|record| record.content_id == content_id)
            .cloned()
            .collect()
    }

    /// Get policy recommendations for content type and size
    pub fn recommend_policy(
        &self,
        content_size: u64,
        content_type: &str,
        sharing_requirements: &str,
    ) -> PolicyResult<StoragePolicy> {
        // Markdown content -> PublicMarkdown if sharing publicly
        if self.is_markdown_content(content_type) && sharing_requirements == "public" {
            return Ok(StoragePolicy::PublicMarkdown);
        }

        // Large files -> GroupScoped for efficient sharing
        if content_size > 100 * 1024 * 1024 && sharing_requirements == "group" {
            return Ok(StoragePolicy::GroupScoped {
                group_id: "default".to_string(),
            });
        }

        // Sensitive content -> PrivateMax
        if sharing_requirements == "none" || content_type.contains("private") {
            return Ok(StoragePolicy::PrivateMax);
        }

        // Default to PrivateScoped
        Ok(StoragePolicy::PrivateScoped {
            namespace: "default".to_string(),
        })
    }

    /// Validate deduplication scope compliance
    pub fn validate_deduplication_scope(
        &self,
        policy: &StoragePolicy,
        _user_id: &str,
        group_id: Option<&str>,
    ) -> PolicyResult<DeduplicationScope> {
        let scope = policy.deduplication_scope();

        match &scope {
            DeduplicationScope::None => Ok(scope),
            DeduplicationScope::Global => Ok(scope),
            DeduplicationScope::User(namespace) => {
                if namespace.is_empty() {
                    return Err(PolicyError::ValidationFailed {
                        reason: "User deduplication requires valid namespace".to_string(),
                    });
                }
                Ok(scope)
            }
            DeduplicationScope::Group(required_group_id) => {
                if let Some(gid) = group_id {
                    if gid == required_group_id {
                        Ok(scope)
                    } else {
                        Err(PolicyError::ValidationFailed {
                            reason: format!(
                                "Group mismatch: expected {}, got {}",
                                required_group_id, gid
                            ),
                        })
                    }
                } else {
                    Err(PolicyError::ValidationFailed {
                        reason: "Group deduplication requires group context".to_string(),
                    })
                }
            }
        }
    }

    // Private helper methods

    fn policy_type_string(&self, policy: &StoragePolicy) -> String {
        match policy {
            StoragePolicy::PrivateMax => "PrivateMax".to_string(),
            StoragePolicy::PrivateScoped { .. } => "PrivateScoped".to_string(),
            StoragePolicy::GroupScoped { .. } => "GroupScoped".to_string(),
            StoragePolicy::PublicMarkdown => "PublicMarkdown".to_string(),
        }
    }

    fn is_markdown_content(&self, content_type: &str) -> bool {
        content_type.contains("markdown")
            || content_type.contains("text/plain")
            || content_type.ends_with(".md")
    }

    fn is_valid_namespace(&self, namespace: &str) -> bool {
        !namespace.is_empty()
            && namespace.len() <= 255
            && namespace.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    async fn validate_group_access(&self, _user_id: &str, _group_id: &str) -> PolicyResult<bool> {
        // Simplified validation - in production this would check actual group membership
        Ok(true)
    }

    fn policy_requires_encryption(&self, policy: &StoragePolicy) -> bool {
        match policy.encryption_mode() {
            EncryptionMode::ChaCha20Poly1305Local
            | EncryptionMode::ChaCha20Poly1305Derived
            | EncryptionMode::ChaCha20Poly1305Shared
            | EncryptionMode::Convergent => true,
        }
    }

    fn validate_content_integrity(&self, content: &[u8]) -> bool {
        // Basic integrity checks
        !content.is_empty() && content.len() < (10 * 1024 * 1024 * 1024) // 10GB max
    }

    fn requires_key_migration(&self, from: &StoragePolicy, to: &StoragePolicy) -> bool {
        match (from, to) {
            (StoragePolicy::PrivateScoped { .. }, StoragePolicy::GroupScoped { .. }) => true,
            (StoragePolicy::GroupScoped { .. }, StoragePolicy::PublicMarkdown) => true,
            _ => false,
        }
    }

    fn calculate_transition_cost(&self, from: &StoragePolicy, to: &StoragePolicy) -> f64 {
        let base_cost = 1.0;

        // Re-encryption cost
        let encryption_cost = if from.encryption_mode() != to.encryption_mode() {
            2.0
        } else {
            0.0
        };

        // Key migration cost
        let key_cost = if self.requires_key_migration(from, to) {
            1.5
        } else {
            0.0
        };

        base_cost + encryption_cost + key_cost
    }
}

// Thread-safe implementations
unsafe impl Send for PolicyManager {}
unsafe impl Sync for PolicyManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_policy_validation() {
        let manager = PolicyManager::new();
        let policy = StoragePolicy::PrivateScoped {
            namespace: "test_user".to_string(),
        };

        let result = manager
            .validate_policy(&policy, 1024, "user123", "text/plain")
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_policy_enforcement() {
        let manager = PolicyManager::new();
        let policy = StoragePolicy::PrivateMax;
        let content = b"test content";

        let result = manager
            .enforce_policy(&policy, "content123", content, "user123", "store")
            .await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_transitions() {
        let manager = PolicyManager::new();
        let from = StoragePolicy::PrivateMax;
        let to = StoragePolicy::PrivateScoped {
            namespace: "test".to_string(),
        };

        assert!(manager.is_transition_allowed(&from, &to));

        let transition = manager.plan_transition(&from, &to).unwrap();
        assert!(transition.requires_re_encryption);
    }

    #[test]
    fn test_policy_recommendations() {
        let manager = PolicyManager::new();

        let policy = manager
            .recommend_policy(1024, "text/markdown", "public")
            .unwrap();
        assert_eq!(policy, StoragePolicy::PublicMarkdown);
    }

    #[test]
    fn test_deduplication_scope_validation() {
        let manager = PolicyManager::new();
        let policy = StoragePolicy::GroupScoped {
            group_id: "team1".to_string(),
        };

        let result = manager.validate_deduplication_scope(&policy, "user123", Some("team1"));
        assert!(result.is_ok());
    }
}
