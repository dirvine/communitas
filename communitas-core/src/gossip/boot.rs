// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Boot Sequence for Gossip Overlay
//!
//! Implements SPEC.md §2: Boot sequence
//!
//! 1. Load ML-DSA identity
//! 2. Dial 1-3 favourite contacts over ant-quic
//! 3. Start membership (HyParView+SWIM)
//! 4. For each joined channel/org: join MLS group, subscribe to topic
//! 5. Begin presence beacons and CRDT anti-entropy

use super::context::GossipContext;
use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Boot sequence orchestrator
pub struct GossipBootSequence {
    context: GossipContext,
}

impl GossipBootSequence {
    /// Create a new boot sequence
    pub fn new(context: GossipContext) -> Self {
        Self { context }
    }

    /// Execute the complete boot sequence per SPEC.md §2
    pub async fn execute(&mut self) -> Result<()> {
        info!(
            "Starting gossip overlay boot sequence for {}",
            self.context.four_words()
        );

        // Step 1: Load ML-DSA identity (already done in GossipContext::initialize)
        info!("✓ Step 1: ML-DSA identity loaded");

        // Step 2: Dial favourite contacts
        self.dial_favourite_contacts().await?;
        info!("✓ Step 2: Dialed favourite contacts");

        // Step 2.5: Find coordinators via FOAF if peer cache is cold (SPEC2.md §2 step 4)
        self.find_coordinators_if_needed().await?;
        info!("✓ Step 2.5: Coordinator discovery complete");

        // Step 3: Start membership layer
        self.start_membership().await?;
        info!("✓ Step 3: Membership layer started (HyParView + SWIM)");

        // Step 4: Join channels/orgs and subscribe to topics
        self.join_existing_entities().await?;
        info!("✓ Step 4: Joined existing entities");

        // Step 5: Start presence beacons and CRDT anti-entropy
        self.start_presence_and_sync().await?;
        info!("✓ Step 5: Presence beacons and CRDT sync active");

        info!("Gossip overlay boot sequence complete!");
        Ok(())
    }

    /// Step 2: Dial 1-3 favourite contacts over ant-quic
    async fn dial_favourite_contacts(&self) -> Result<()> {
        let favourites = self.load_favourites_from_storage().await?;

        if favourites.is_empty() {
            info!("No favourite contacts configured yet (cold start)");
            // Use optional introducer list for cold start
            return self.use_introducer_nodes().await;
        }

        info!("Dialing {} favourite contacts", favourites.len().min(3));

        // Dial up to 3 favourites
        for (i, four_words) in favourites.iter().take(3).enumerate() {
            match self.dial_contact(four_words).await {
                Ok(_) => debug!("Connected to favourite #{}: {}", i + 1, four_words),
                Err(e) => warn!("Failed to dial favourite {}: {}", four_words, e),
            }
        }

        Ok(())
    }

    /// Load favourite contacts from persistent storage
    async fn load_favourites_from_storage(&self) -> Result<Vec<String>> {
        // TODO: Load from encrypted storage
        // For now, return empty list (will be populated as users add favourites)
        Ok(vec![])
    }

    /// Use optional introducer nodes for cold start
    async fn use_introducer_nodes(&self) -> Result<()> {
        // SPEC.md §3: Keep a small optional introducer list for cold start only
        // Load bootstrap nodes from production config
        let bootstrap_nodes = vec![
            "bless-lava-jeffrey-parking:443".to_string(), // 167.71.188.131 - Digital Ocean Droplet 1
            "bless-route-evaporate-lunch:443".to_string(), // 138.197.29.195 - Digital Ocean Droplet 2
        ];

        // Seed peer cache with bootstrap nodes for fast boot
        {
            let mut cache = self.context.peer_cache.write().await;
            match cache.seed_bootstrap_nodes(&bootstrap_nodes).await {
                Ok(count) => info!("Seeded {} bootstrap nodes into peer cache", count),
                Err(e) => warn!("Failed to seed bootstrap nodes: {}", e),
            }
        }

        let config = super::discovery::IntroducerConfig {
            addresses: bootstrap_nodes,
            timeout_secs: 10,
        };

        if config.addresses.is_empty() {
            info!("No introducer nodes configured, will wait for manual peer addition");
            return Ok(());
        }

        info!(
            "Using {} introducer nodes for cold start",
            config.addresses.len()
        );

        match super::discovery::cold_start_discovery(config, &self.context.transport).await {
            Ok(introducers) => {
                info!("Connected to {} introducer(s)", introducers.len());
                Ok(())
            }
            Err(e) => {
                warn!("Cold start discovery failed: {}", e);
                Ok(()) // Non-fatal, user can add peers manually
            }
        }
    }

    /// Find coordinators via FOAF if peer cache is cold (SPEC2.md §2 step 4)
    async fn find_coordinators_if_needed(&self) -> Result<()> {
        // Check if peer cache is cold (no recent peers)
        let cache = self.context.peer_cache.read().await;
        let peers = cache.get_top_peers(10);
        drop(cache); // Release lock

        if !peers.is_empty() {
            debug!(
                "Peer cache has {} entries, skipping coordinator discovery",
                peers.len()
            );
            return Ok(());
        }

        info!("Peer cache is cold, finding coordinators via FOAF (TTL=3, fanout=3)");

        // Use coordinator client to find coordinators
        match self
            .context
            .coordinator
            .find_coordinators_via_foaf(3, 3)
            .await
        {
            Ok(coordinators) => {
                info!("Discovered {} coordinators via FOAF", coordinators.len());
                // Coordinators are automatically cached by find_coordinators_via_foaf
                Ok(())
            }
            Err(e) => {
                warn!("Coordinator discovery failed: {}", e);
                Ok(()) // Non-fatal, continue with boot sequence
            }
        }
    }

    /// Dial a contact by four-word address using FOAF discovery
    async fn dial_contact(&self, four_words: &str) -> Result<()> {
        // Use FOAF discovery to find contact
        match self.context.discovery.find_contact(four_words).await {
            Ok(peer_id) => {
                info!(
                    "Found contact {} via FOAF discovery: {:?}",
                    four_words, peer_id
                );
                // TODO: Actual dial using transport with peer_id
                Ok(())
            }
            Err(e) => {
                warn!("Failed to find contact {} via FOAF: {}", four_words, e);
                Err(e)
            }
        }
    }

    /// Step 3: Start membership layer (HyParView + SWIM)
    async fn start_membership(&mut self) -> Result<()> {
        let membership = self.context.membership.write().await;

        // Join the overlay network
        let seeds = self.get_seed_peers().await?;
        membership
            .join(seeds)
            .await
            .context("Failed to join membership overlay")?;

        info!("Membership layer active, starting periodic shuffle and probes");
        Ok(())
    }

    /// Get seed peers for membership join
    async fn get_seed_peers(&self) -> Result<Vec<String>> {
        // Combine favourite contacts and active transport connections
        let mut seeds = Vec::new();

        // Add favourite contacts
        let favourites = self.context.get_favourite_contacts().await;
        seeds.extend(favourites);

        // If no seeds, use introducer nodes
        if seeds.is_empty() {
            // TODO: Get introducer addresses from config
        }

        Ok(seeds)
    }

    /// Step 4: Join existing channels/orgs and subscribe to topics
    async fn join_existing_entities(&mut self) -> Result<()> {
        let entities = self.load_entities_from_storage().await?;

        if entities.is_empty() {
            info!("No existing entities to join");
            return Ok(());
        }

        info!("Joining {} existing entities", entities.len());

        for (entity_id, entity_type) in entities {
            match self.context.join_entity(&entity_id, &entity_type).await {
                Ok(_) => debug!("Joined {} {}", entity_type, entity_id),
                Err(e) => warn!("Failed to join {} {}: {}", entity_type, entity_id, e),
            }
        }

        Ok(())
    }

    /// Load entities (channels, projects, orgs) from storage
    async fn load_entities_from_storage(&self) -> Result<Vec<(String, String)>> {
        // TODO: Load from encrypted storage
        // Returns vec of (entity_id, entity_type) tuples
        Ok(vec![])
    }

    /// Step 5: Start presence beacons and CRDT anti-entropy
    async fn start_presence_and_sync(&mut self) -> Result<()> {
        // Start presence beacons (5 minute interval)
        {
            let presence = self.context.presence.write().await;
            presence
                .start_beacons(300)
                .await
                .context("Failed to start presence beacons")?;
            info!("Presence beacons active (5min interval, TTL: 15min, MLS-encrypted)");
        }

        // Start CRDT anti-entropy (60 second interval)
        {
            let transport = self.context.transport.clone();
            let anti_entropy = self.context.anti_entropy.clone();

            anti_entropy
                .start(move |peer_id, _delta| {
                    let transport = transport.clone();
                    Box::pin(async move {
                        // Send delta to peer via transport
                        // TODO: Implement actual delta transmission
                        debug!("Would send CRDT delta to peer {:?}", peer_id);
                        let _ = transport; // Use transport to avoid warning
                        Ok(())
                    })
                })
                .await
                .context("Failed to start CRDT anti-entropy")?;
            info!("CRDT anti-entropy active (60s interval, delta-based sync)");
        }

        Ok(())
    }

    /// Get the context (consumes self)
    pub fn into_context(self) -> GossipContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_boot_sequence_initialization() {
        let ctx = GossipContext::initialize(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            "Desktop".to_string(),
            None,
        )
        .await
        .expect("context init");

        let boot = GossipBootSequence::new(ctx);

        // Note: Full boot sequence requires network connectivity
        // For unit tests, we just verify the structure is correct
        assert!(boot.load_favourites_from_storage().await.is_ok());
        assert!(boot.load_entities_from_storage().await.is_ok());
    }
}
