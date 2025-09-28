//! Bootstrap integration with saorsa-core
//!
//! This module provides a unified bootstrap management system that leverages
//! saorsa-core's BootstrapCache and BootstrapManager for both desktop and headless nodes.

use anyhow::{Context, Result};
use saorsa_core::bootstrap::{BootstrapManager, CacheConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Configuration for bootstrap integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    /// Maximum number of bootstrap nodes to cache
    pub max_contacts: usize,
    /// Default bootstrap nodes (four-word addresses or socket addresses)
    pub default_nodes: Vec<String>,
    /// Enable automatic peer discovery
    pub auto_discovery: bool,
    /// Persistence path for bootstrap cache
    pub cache_dir: PathBuf,
    /// Quality threshold for keeping peers (0.0-1.0)
    pub quality_threshold: f64,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            max_contacts: 5000,
            default_nodes: vec![
                "ocean-forest-moon-star".to_string(),
                "river-mountain-sun-cloud".to_string(),
            ],
            auto_discovery: true,
            cache_dir: get_bootstrap_cache_path(),
            quality_threshold: 0.3,
        }
    }
}

/// Enhanced bootstrap integration that uses saorsa-core's bootstrap system
pub struct EnhancedBootstrapManager {
    /// Core bootstrap manager from saorsa-core
    core_manager: Arc<RwLock<BootstrapManager>>,
    /// Configuration
    config: BootstrapConfig,
    /// Custom bootstrap nodes added by user
    custom_nodes: Arc<RwLock<HashSet<String>>>,
}

impl EnhancedBootstrapManager {
    /// Create a new bootstrap manager with the given configuration
    pub async fn new(config: BootstrapConfig) -> Result<Self> {
        // Create saorsa-core cache configuration
        let cache_config = CacheConfig {
            cache_dir: config.cache_dir.clone(),
            max_contacts: config.max_contacts,
            merge_interval: Duration::from_secs(30),
            cleanup_interval: Duration::from_secs(3600),
            quality_update_interval: Duration::from_secs(300),
            stale_threshold: Duration::from_secs(86400 * 7), // 7 days
            connectivity_check_interval: Duration::from_secs(900),
            connectivity_check_count: 100,
        };

        // Create the bootstrap manager with the configuration
        let core_manager = Arc::new(RwLock::new(
            BootstrapManager::with_config(cache_config)
                .await
                .context("Failed to create bootstrap manager")?,
        ));

        let manager = Self {
            core_manager,
            config,
            custom_nodes: Arc::new(RwLock::new(HashSet::new())),
        };

        // Add default nodes to custom nodes list for tracking
        for node in &manager.config.default_nodes.clone() {
            manager.add_custom_node(node).await;
        }

        Ok(manager)
    }

    /// Add a custom bootstrap node for tracking
    async fn add_custom_node(&self, node: &str) {
        let node = node.trim();
        let normalized = self.normalize_address(node);

        let mut custom = self.custom_nodes.write().await;
        custom.insert(normalized);
    }

    /// Add a bootstrap node (four-word address or socket address)
    /// Note: Actual connection happens through the DHT network manager
    pub async fn add_bootstrap_node(&self, node: &str) -> Result<()> {
        let node = node.trim();

        // Check if it's a four-word address or socket address
        if self.is_four_word_address(node) || node.parse::<SocketAddr>().is_ok() {
            self.add_custom_node(node).await;
            info!("Added custom bootstrap node: {}", node);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Invalid bootstrap node format: {}", node))
        }
    }

    /// Get bootstrap peers from the cache
    pub async fn get_bootstrap_candidates(&self, count: usize) -> Result<Vec<String>> {
        // Get custom nodes first
        let custom = self.custom_nodes.read().await;
        let mut candidates: Vec<String> = custom.iter().cloned().collect();

        // Get additional peers from saorsa-core's bootstrap manager
        let manager = self.core_manager.read().await;
        let contacts = manager
            .get_bootstrap_peers(count.saturating_sub(candidates.len()))
            .await
            .context("Failed to get bootstrap peers")?;

        // Add contact addresses to candidates
        for contact in contacts {
            for addr in contact.addresses {
                candidates.push(addr.to_string());
            }
        }

        Ok(candidates)
    }

    /// Start background maintenance tasks
    pub async fn start_background_tasks(&self) -> Result<()> {
        let mut manager = self.core_manager.write().await;
        manager
            .start_background_tasks()
            .await
            .context("Failed to start background tasks")?;
        Ok(())
    }

    /// Normalize an address
    fn normalize_address(&self, input: &str) -> String {
        input
            .trim()
            .to_lowercase()
            .replace(' ', "-")
            .replace('_', "-")
    }

    /// Check if a string is a valid four-word address using dictionary validation
    fn is_four_word_address(&self, input: &str) -> bool {
        let candidate = input.trim().to_lowercase().replace([' ', '_'], "-");
        
        if let Ok(parsed) = saorsa_core::identity::FourWordAddress::parse_str(&candidate) {
            let words_vec = parsed.words();
            if let Ok(words) = words_vec.try_into() {
                return saorsa_core::fwid::fw_check(words);
            }
        }
        false
    }

    /// Get custom bootstrap nodes added by user
    pub async fn get_custom_nodes(&self) -> Vec<String> {
        let custom = self.custom_nodes.read().await;
        custom.iter().cloned().collect()
    }

    /// Clear all custom bootstrap nodes (keeps default nodes)
    pub async fn clear_custom_nodes(&self) -> Result<()> {
        let mut custom = self.custom_nodes.write().await;
        custom.clear();

        // Re-add default nodes
        for node in &self.config.default_nodes {
            custom.insert(self.normalize_address(node));
        }

        info!("Cleared custom bootstrap nodes, keeping defaults");
        Ok(())
    }

    /// Get bootstrap statistics
    pub async fn get_stats(&self) -> Result<BootstrapStats> {
        let custom_count = self.custom_nodes.read().await.len();

        // Try to get some peers to count
        let manager = self.core_manager.read().await;
        let peers = manager
            .get_bootstrap_peers(100)
            .await
            .unwrap_or_default();

        let total_nodes = custom_count + peers.len();
        let quality_nodes = peers
            .iter()
            .filter(|contact| contact.quality_metrics.quality_score >= self.config.quality_threshold)
            .count();

        Ok(BootstrapStats {
            total_nodes,
            custom_nodes: custom_count,
            quality_nodes,
            cache_path: self.config.cache_dir.clone(),
        })
    }

    /// Save cache to disk (handled automatically by saorsa-core)
    pub async fn save_cache(&self) -> Result<()> {
        // Saorsa-core handles persistence automatically
        debug!("Cache is automatically persisted by saorsa-core");
        Ok(())
    }

    /// Load cache from disk (handled automatically by saorsa-core)
    pub async fn load_cache(&self) -> Result<()> {
        // Saorsa-core handles loading automatically
        debug!("Cache is automatically loaded by saorsa-core");
        Ok(())
    }
}

/// Bootstrap statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapStats {
    pub total_nodes: usize,
    pub custom_nodes: usize,
    pub quality_nodes: usize,
    pub cache_path: PathBuf,
}

/// Platform-specific storage paths
pub fn get_bootstrap_cache_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Library")
            .join("Application Support")
            .join("com.p2pfoundation.communitas")
            .join("bootstrap")
    }

    #[cfg(target_os = "windows")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
            .join("communitas")
            .join("bootstrap")
    }

    #[cfg(target_os = "linux")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("communitas")
            .join("bootstrap")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_bootstrap_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = BootstrapConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = EnhancedBootstrapManager::new(config).await.unwrap();
        let stats = manager.get_stats().await.unwrap();

        // Should have default nodes
        assert!(stats.custom_nodes >= 2);
    }

    #[tokio::test]
    async fn test_four_word_validation() {
        let temp_dir = TempDir::new().unwrap();
        let config = BootstrapConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let manager = EnhancedBootstrapManager::new(config).await.unwrap();

        assert!(manager.is_four_word_address("ocean-forest-moon-star"));
        assert!(manager.is_four_word_address("alpha-beta-gamma-delta-epsilon")); // IPv6
        assert!(!manager.is_four_word_address("192.168.1.1"));
        assert!(!manager.is_four_word_address("not-enough"));
    }

    #[tokio::test]
    async fn test_add_socket_bootstrap() {
        let temp_dir = TempDir::new().unwrap();
        let config = BootstrapConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = EnhancedBootstrapManager::new(config).await.unwrap();

        // Add socket address
        manager.add_bootstrap_node("127.0.0.1:8080").await.unwrap();

        let custom = manager.get_custom_nodes().await;
        assert!(custom.contains(&"127.0.0.1:8080".to_string()));
    }
}