// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Rendezvous Shards Client for User Discovery
//!
//! Implements SPEC2.md §4 and §9: Rendezvous Shards for global findability
//! without DNS/DHT or centralized directory services.
//!
//! ## Architecture (per SPEC2.md)
//! - 65,536 total shards (k=16)
//! - Shard calculated via BLAKE3 hash of target ID
//! - Providers gossip ProviderSummaries to target's specific shard
//! - Clients subscribe to shards and score providers
//!
//! ## Use Cases
//! 1. **Find User**: Subscribe to user's rendezvous shard for Provider Summaries
//! 2. **Publish Availability**: Publish ProviderSummary to own shard
//! 3. **Score Providers**: Rank providers by latency, NAT class, capabilities

use anyhow::Result;
use bytes::Bytes;
use saorsa_gossip_pubsub::PubSub;
use saorsa_gossip_rendezvous::{ProviderSummary, calculate_shard};
use saorsa_gossip_transport::GossipTransport;
use saorsa_gossip_types::{PeerId, TopicId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Rendezvous client for shard-based user discovery
pub struct RendezvousClient {
    /// Our peer ID
    peer_id: PeerId,

    /// Transport layer for network communication (reserved for future use)
    #[allow(dead_code)]
    transport: Arc<RwLock<Box<dyn GossipTransport>>>,

    /// PubSub layer for topic subscriptions
    pubsub: Arc<RwLock<Box<dyn PubSub>>>,

    /// Cached provider summaries by target ID
    cached_summaries: Arc<RwLock<HashMap<[u8; 32], Vec<ProviderSummary>>>>,

    /// Active shard subscriptions (shard_id -> topic_id)
    subscriptions: Arc<RwLock<HashMap<u16, TopicId>>>,

    /// Active background collectors (target_id -> task handle)
    active_collectors: Arc<RwLock<HashMap<[u8; 32], JoinHandle<()>>>>,
}

impl RendezvousClient {
    /// Create a new rendezvous client
    ///
    /// # Arguments
    /// * `peer_id` - Our peer ID
    /// * `transport` - Transport layer for network communication
    /// * `pubsub` - PubSub layer for topic subscriptions
    pub fn new(
        peer_id: PeerId,
        transport: Arc<RwLock<Box<dyn GossipTransport>>>,
        pubsub: Arc<RwLock<Box<dyn PubSub>>>,
    ) -> Self {
        Self {
            peer_id,
            transport,
            pubsub,
            cached_summaries: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            active_collectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get our peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Get the underlying transport for direct QUIC operations
    pub fn get_transport(&self) -> Arc<RwLock<Box<dyn GossipTransport>>> {
        self.transport.clone()
    }

    /// Get cached provider summaries for a target
    pub async fn get_cached_summaries(&self, target_id: &[u8; 32]) -> Vec<ProviderSummary> {
        self.cached_summaries
            .read()
            .await
            .get(target_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Subscribe to a rendezvous shard for a target
    ///
    /// Per SPEC2.md §4: Subscribe to user's rendezvous shard for Provider Summaries
    ///
    /// # Arguments
    /// * `target_id` - The target user/entity ID to find
    ///
    /// # Returns
    /// The shard number (0-65535) that was subscribed to
    pub async fn subscribe_to_shard(&self, target_id: &[u8; 32]) -> Result<u16> {
        // Calculate which shard this target belongs to
        let shard = calculate_shard(target_id);

        // Check if already subscribed
        {
            let subscriptions = self.subscriptions.read().await;
            if subscriptions.contains_key(&shard) {
                return Ok(shard);
            }
        }

        // Create topic ID for this shard (hash the shard number to get 32 bytes)
        let shard_name = format!("rendezvous_shard_{}", shard);
        let hash = blake3::hash(shard_name.as_bytes());
        let topic_id = TopicId::new(*hash.as_bytes());

        // Subscribe via pubsub
        let pubsub = self.pubsub.read().await;
        let _rx = pubsub.subscribe(topic_id);

        // Store subscription
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(shard, topic_id);

        Ok(shard)
    }

    /// Publish a provider summary to a rendezvous shard
    ///
    /// Per SPEC2.md §5: Gossip Provider Summaries to target's shard
    ///
    /// # Arguments
    /// * `summary` - The provider summary to publish
    pub async fn publish_provider_summary(&self, summary: ProviderSummary) -> Result<()> {
        // Extract target_id from summary (public field)
        let target_id = &summary.target;

        // Calculate which shard this target belongs to
        let shard = calculate_shard(target_id);

        // Create topic ID for this shard (same pattern as subscribe_to_shard)
        let shard_name = format!("rendezvous_shard_{}", shard);
        let hash = blake3::hash(shard_name.as_bytes());
        let topic_id = TopicId::new(*hash.as_bytes());

        // Serialize summary with serde_cbor
        let mut summary_bytes = Vec::new();
        ciborium::ser::into_writer(&summary, &mut summary_bytes)
            .map_err(|e| anyhow::anyhow!("CBOR encoding failed: {:?}", e))?;

        // Publish via pubsub
        let pubsub = self.pubsub.read().await;
        pubsub.publish(topic_id, summary_bytes.into()).await?;

        Ok(())
    }

    /// Get providers for a target, sorted by quality score
    ///
    /// Per SPEC2.md §4: Score providers by validity and quality
    ///
    /// # Arguments
    /// * `target_id` - The target to find providers for
    ///
    /// # Returns
    /// Vec of ProviderSummary sorted by score (best first)
    pub async fn get_providers_for_target(&self, target_id: &[u8; 32]) -> Vec<ProviderSummary> {
        let cache = self.cached_summaries.read().await;
        let summaries = cache.get(target_id).cloned().unwrap_or_default();
        drop(cache);

        if summaries.is_empty() {
            return Vec::new();
        }

        // Score and sort providers
        let mut scored: Vec<(ProviderSummary, u64)> = summaries
            .into_iter()
            .filter_map(|summary| {
                // Calculate validity remaining (exp is absolute timestamp in ms)
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_millis() as u64;

                if summary.exp <= now {
                    // Expired, skip
                    return None;
                }

                let remaining_ms = summary.exp - now;

                // Score based on remaining validity (longer = better)
                // In future: also factor in latency, NAT class, capabilities
                Some((summary, remaining_ms))
            })
            .collect();

        // Sort by score descending (best first)
        scored.sort_by(|a, b| b.1.cmp(&a.1));

        scored.into_iter().map(|(summary, _)| summary).collect()
    }

    /// Process an incoming ProviderSummary message and add to cache
    ///
    /// # Arguments
    /// * `target_id` - The target this summary is for
    /// * `message_bytes` - Serialized ProviderSummary (CBOR)
    pub async fn process_incoming_summary(
        &self,
        target_id: &[u8; 32],
        message_bytes: Bytes,
    ) -> Result<()> {
        // Deserialize the ProviderSummary
        let summary: ProviderSummary = match ciborium::de::from_reader(&message_bytes[..]) {
            Ok(summary) => summary,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Invalid CBOR, not a ProviderSummary: {}",
                    e
                ));
            }
        };

        // Verify the summary is for the expected target
        if &summary.target != target_id {
            return Err(anyhow::anyhow!(
                "Summary target mismatch: expected {:?}, got {:?}",
                target_id,
                summary.target
            ));
        }

        // Add to cache
        let mut cache = self.cached_summaries.write().await;
        let summaries = cache.entry(*target_id).or_insert_with(Vec::new);

        // Check if we already have this provider (deduplicate)
        if let Some(existing) = summaries
            .iter_mut()
            .find(|s| s.provider == summary.provider)
        {
            // Update existing entry with newer data
            *existing = summary;
        } else {
            // Add new provider
            summaries.push(summary);
        }

        Ok(())
    }

    /// Start collecting ProviderSummaries in the background for a target
    ///
    /// Spawns a background task that listens on the subscription receiver
    /// and populates the cache with incoming summaries.
    ///
    /// # Arguments
    /// * `target_id` - The target to collect summaries for
    pub async fn start_collecting_for_target(&self, target_id: [u8; 32]) -> Result<()> {
        // Check if already collecting
        {
            let collectors = self.active_collectors.read().await;
            if collectors.contains_key(&target_id) {
                return Ok(()); // Already collecting
            }
        }

        // Get the shard for this target
        let shard = calculate_shard(&target_id);

        // Get topic_id and subscription receiver
        let topic_id = {
            let subscriptions = self.subscriptions.read().await;
            subscriptions
                .get(&shard)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Not subscribed to shard for target"))?
        };

        // Get a new receiver for this target
        let pubsub = self.pubsub.read().await;
        let mut rx = pubsub.subscribe(topic_id);
        drop(pubsub);

        // Clone Arc for the background task
        let client = Arc::new(self.clone_for_background());

        // Spawn background collection task
        let handle = tokio::spawn(async move {
            while let Some((_peer_id, message_bytes)) = rx.recv().await {
                // Process the incoming summary
                if let Err(e) = client
                    .process_incoming_summary(&target_id, message_bytes)
                    .await
                {
                    // Log error but continue collecting
                    eprintln!("Error processing summary: {}", e);
                }
            }
        });

        // Store the task handle
        let mut collectors = self.active_collectors.write().await;
        collectors.insert(target_id, handle);

        Ok(())
    }

    /// Helper to clone self for background tasks
    fn clone_for_background(&self) -> Self {
        Self {
            peer_id: self.peer_id,
            transport: self.transport.clone(),
            pubsub: self.pubsub.clone(),
            cached_summaries: self.cached_summaries.clone(),
            subscriptions: self.subscriptions.clone(),
            active_collectors: self.active_collectors.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_gossip_transport::{QuicTransport, TransportConfig};

    fn create_test_peer_id(seed: u8) -> PeerId {
        let mut bytes = [0u8; 32];
        bytes[0] = seed;
        PeerId::new(bytes)
    }

    async fn create_test_rendezvous_client() -> RendezvousClient {
        let peer_id = create_test_peer_id(1);

        // Create test transport
        let config = TransportConfig::default();
        let qt1 = QuicTransport::new(config.clone());
        let qt2 = QuicTransport::new(config);
        let transport: Arc<RwLock<Box<dyn GossipTransport>>> = Arc::new(RwLock::new(Box::new(qt1)));

        // Create test identity for signing
        let identity = saorsa_gossip_identity::Identity::new("TestUser".to_string())
            .expect("identity creation");
        let signing_key = identity.key_pair().clone();

        // Create test pubsub (needs separate transport instance)
        let pubsub_impl =
            saorsa_gossip_pubsub::PlumtreePubSub::new(peer_id.clone(), Arc::new(qt2), signing_key);
        let pubsub: Arc<RwLock<Box<dyn PubSub>>> = Arc::new(RwLock::new(Box::new(pubsub_impl)));

        RendezvousClient::new(peer_id, transport, pubsub)
    }

    #[tokio::test]
    async fn test_rendezvous_client_creation() {
        let client = create_test_rendezvous_client().await;

        // Verify client was created
        assert_eq!(client.peer_id().as_bytes()[0], 1);

        // Verify empty cache initially
        let target_id = [0u8; 32];
        let cached = client.get_cached_summaries(&target_id).await;
        assert_eq!(cached.len(), 0);
    }

    #[tokio::test]
    async fn test_calculate_shard_deterministic() {
        // Test that shard calculation is deterministic
        let target_id = [42u8; 32];
        let shard1 = calculate_shard(&target_id);
        let shard2 = calculate_shard(&target_id);
        assert_eq!(shard1, shard2);

        // Test different targets give different shards (usually)
        let different_id = [99u8; 32];
        let shard3 = calculate_shard(&different_id);
        // Very unlikely to be the same (1 in 65536 chance)
        assert_ne!(shard1, shard3);
    }

    #[tokio::test]
    async fn test_subscribe_to_shard() {
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];

        // Calculate expected shard
        let expected_shard = calculate_shard(&target_id);

        // Subscribe to shard
        let result = client.subscribe_to_shard(&target_id).await;
        assert!(result.is_ok());
        let shard = result.unwrap();
        assert_eq!(shard, expected_shard);

        // Verify subscription is stored
        let subscriptions = client.subscriptions.read().await;
        assert!(subscriptions.contains_key(&shard));
    }

    #[tokio::test]
    async fn test_publish_provider_summary() {
        let client = create_test_rendezvous_client().await;

        // Create a provider summary for our peer
        let target_id = client.peer_id().as_bytes().clone();
        let summary = ProviderSummary::new(
            target_id,
            client.peer_id(),
            vec![saorsa_gossip_rendezvous::Capability::Site],
            3600_000, // 1 hour validity
        );

        // Publish the summary
        let result = client.publish_provider_summary(summary).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_providers_for_target() {
        // RED: This will fail until get_providers_for_target is implemented
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];

        // Add some mock summaries to cache (simulating received summaries)
        {
            let mut cache = client.cached_summaries.write().await;

            // Provider 1: expires in 1 hour (valid)
            let provider1 = PeerId::new([1u8; 32]);
            let summary1 = ProviderSummary::new(
                target_id,
                provider1,
                vec![saorsa_gossip_rendezvous::Capability::Site],
                3600_000, // 1 hour validity
            );

            // Provider 2: expires in 5 minutes (less valid)
            let provider2 = PeerId::new([2u8; 32]);
            let summary2 = ProviderSummary::new(
                target_id,
                provider2,
                vec![saorsa_gossip_rendezvous::Capability::Identity],
                300_000, // 5 minutes validity
            );

            cache.insert(target_id, vec![summary1, summary2]);
        }

        // Get providers sorted by validity (longer validity = higher score)
        let providers = client.get_providers_for_target(&target_id).await;
        assert_eq!(providers.len(), 2);

        // First provider should have longer validity (1 hour)
        assert_eq!(providers[0].provider.as_bytes()[0], 1);

        // Second provider should have shorter validity (5 min)
        assert_eq!(providers[1].provider.as_bytes()[0], 2);
    }

    #[tokio::test]
    async fn test_subscribe_to_same_shard_twice_idempotent() {
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];

        // Subscribe first time
        let shard1 = client.subscribe_to_shard(&target_id).await.ok().unwrap();

        // Subscribe again to same shard (should be idempotent)
        let shard2 = client.subscribe_to_shard(&target_id).await.ok().unwrap();

        assert_eq!(shard1, shard2);

        // Verify only one subscription exists
        let subscriptions = client.subscriptions.read().await;
        assert_eq!(subscriptions.len(), 1);
        assert!(subscriptions.contains_key(&shard1));
    }

    #[tokio::test]
    async fn test_expired_summaries_filtered_out() {
        let client = create_test_rendezvous_client().await;
        let target_id = [99u8; 32];

        // Add summaries with different expiration times
        {
            let mut cache = client.cached_summaries.write().await;

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            // Provider 1: already expired
            let provider1 = PeerId::new([1u8; 32]);
            let summary1 = ProviderSummary {
                v: 1,
                target: target_id,
                provider: provider1,
                cap: vec![saorsa_gossip_rendezvous::Capability::Site],
                have_root: false,
                manifest_ver: None,
                summary: None,
                exp: now - 1000, // expired 1 second ago
                sig: vec![],
            };

            // Provider 2: valid for 1 hour
            let provider2 = PeerId::new([2u8; 32]);
            let summary2 = ProviderSummary::new(
                target_id,
                provider2,
                vec![saorsa_gossip_rendezvous::Capability::Identity],
                3600_000, // 1 hour validity
            );

            cache.insert(target_id, vec![summary1, summary2]);
        }

        // Get providers - should only return valid one
        let providers = client.get_providers_for_target(&target_id).await;
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].provider.as_bytes()[0], 2);
    }

    #[tokio::test]
    async fn test_process_incoming_provider_summary() {
        // RED: This will fail until we implement message processing
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];

        // Create a provider summary
        let provider = PeerId::new([99u8; 32]);
        let summary = ProviderSummary::new(
            target_id,
            provider,
            vec![saorsa_gossip_rendezvous::Capability::Site],
            3600_000, // 1 hour validity
        );

        // Serialize it (simulating what we'd receive from pubsub)
        let mut summary_bytes = Vec::new();
        ciborium::ser::into_writer(&summary, &mut summary_bytes)
            .map_err(|e| anyhow::anyhow!("CBOR encoding failed: {:?}", e))?;

        // Process the incoming message
        let result = client
            .process_incoming_summary(&target_id, summary_bytes.into())
            .await;
        assert!(result.is_ok());

        // Verify it was added to cache
        let cached = client.get_cached_summaries(&target_id).await;
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].provider.as_bytes()[0], 99);
    }

    #[tokio::test]
    async fn test_start_collecting_for_target() {
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];

        // Subscribe to shard
        client.subscribe_to_shard(&target_id).await.unwrap();

        // Start collecting summaries in background
        let result = client.start_collecting_for_target(target_id).await;
        assert!(result.is_ok());

        // Verify collector is running (we'll check internal state)
        let collectors = client.active_collectors.read().await;
        assert!(collectors.contains_key(&target_id));
    }

    #[tokio::test]
    async fn test_deduplication_updates_existing_provider() {
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];

        // Add initial summary
        let provider = PeerId::new([99u8; 32]);
        let summary1 = ProviderSummary::new(
            target_id,
            provider,
            vec![saorsa_gossip_rendezvous::Capability::Site],
            3600_000,
        );
        let bytes1 = serde_cbor::to_vec(&summary1).unwrap();
        client
            .process_incoming_summary(&target_id, bytes1.into())
            .await
            .unwrap();

        // Verify one entry
        let cached = client.get_cached_summaries(&target_id).await;
        assert_eq!(cached.len(), 1);

        // Add updated summary for same provider
        let summary2 = ProviderSummary::new(
            target_id,
            provider,
            vec![
                saorsa_gossip_rendezvous::Capability::Site,
                saorsa_gossip_rendezvous::Capability::Identity,
            ],
            7200_000, // Different validity
        );
        let bytes2 = serde_cbor::to_vec(&summary2).unwrap();
        client
            .process_incoming_summary(&target_id, bytes2.into())
            .await
            .unwrap();

        // Verify still one entry (deduplication)
        let cached = client.get_cached_summaries(&target_id).await;
        assert_eq!(cached.len(), 1);

        // Verify it was updated (check capabilities)
        assert_eq!(cached[0].cap.len(), 2);
    }

    #[tokio::test]
    async fn test_rejects_mismatched_target() {
        let client = create_test_rendezvous_client().await;
        let target_id = [42u8; 32];
        let wrong_target = [99u8; 32];

        // Create summary for wrong target
        let provider = PeerId::new([1u8; 32]);
        let summary = ProviderSummary::new(
            wrong_target,
            provider,
            vec![saorsa_gossip_rendezvous::Capability::Site],
            3600_000,
        );
        let bytes = serde_cbor::to_vec(&summary).unwrap();

        // Should fail with mismatch error
        let result = client
            .process_incoming_summary(&target_id, bytes.into())
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("target mismatch"));

        // Verify nothing was cached
        let cached = client.get_cached_summaries(&target_id).await;
        assert_eq!(cached.len(), 0);
    }
}
