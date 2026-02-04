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
use bytes::Bytes;
use saorsa_gossip_transport::TransportAdapter;
use saorsa_gossip_transport::{GossipStreamType, GossipTransport};
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};

// Phase 2 TDD: Import retry utilities for exponential backoff
use crate::retry_utils::{RetryConfig, retry_dial};

/// Boot sequence orchestrator
pub struct GossipBootSequence {
    context: GossipContext,
    boot_data: BootData,
}

#[derive(Debug, Clone, Default)]
pub struct BootData {
    favourites: Vec<String>,
    entities: Vec<(String, String)>,
    introducers: Vec<String>,
}

impl GossipBootSequence {
    /// Create a new boot sequence
    pub fn new(context: GossipContext) -> Self {
        Self {
            context,
            boot_data: BootData::default(),
        }
    }

    pub fn with_favourites(mut self, favourites: Vec<String>) -> Self {
        self.boot_data.favourites = favourites;
        self
    }

    pub fn with_entities(mut self, entities: Vec<(String, String)>) -> Self {
        self.boot_data.entities = entities;
        self
    }

    pub fn with_introducers(mut self, introducers: Vec<String>) -> Self {
        self.boot_data.introducers = introducers;
        self
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

        // Step 6: Start connectivity watchdog monitoring (Phase 3 TDD)
        self.start_watchdog_monitoring().await?;
        info!("✓ Step 6: Connectivity watchdog monitoring active");

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
        let favourites = self
            .boot_data
            .favourites
            .iter()
            .filter(|fw| !fw.trim().is_empty())
            .cloned()
            .collect();
        Ok(favourites)
    }

    /// Use optional introducer nodes for cold start
    async fn use_introducer_nodes(&self) -> Result<()> {
        // SPEC.md §3: Keep a small optional introducer list for cold start only
        // Load bootstrap nodes from production config
        //
        // NOTE: Use IP:port format directly. The four-word-networking crate CAN encode/decode
        // IPs via conn_words(), but the previous values were USER identities (random dictionary
        // words hashed one-way to seeds), NOT connection identities (encoded IPs).
        // User identities cannot be decoded back to IP addresses.
        let bootstrap_nodes = if self.boot_data.introducers.is_empty() {
            super::discovery::IntroducerConfig::default().addresses
        } else {
            self.boot_data.introducers.clone()
        };

        // Seed peer cache with bootstrap nodes for fast boot
        match self
            .context
            .peer_cache
            .seed_bootstrap_nodes(&bootstrap_nodes)
            .await
        {
            Ok(count) => info!("Seeded {} bootstrap nodes into peer cache", count),
            Err(e) => warn!("Failed to seed bootstrap nodes: {}", e),
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

        let transport_ref = self.context.transport.as_ref();
        match super::discovery::cold_start_discovery(config, transport_ref).await {
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
        if let Ok(value) = std::env::var("COMMUNITAS_SKIP_FOAF") {
            let normalized = value.trim().to_ascii_lowercase();
            if normalized == "1" || normalized == "true" || normalized == "yes" {
                info!("COMMUNITAS_SKIP_FOAF set; skipping coordinator discovery");
                return Ok(());
            }
        }

        // Check if peer cache is cold (no recent peers)
        let peers = self.context.peer_cache.get_top_peers(10).await;
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
                let count = coordinators.len();
                info!("Discovered {} coordinators via FOAF", count);
                // Coordinators are automatically cached by find_coordinators_via_foaf
                Ok(())
            }
            Err(e) => {
                warn!("Coordinator discovery failed: {}", e);
                Ok(()) // Non-fatal, continue with boot sequence
            }
        }
    }

    /// Dial a contact by four-word address using FOAF discovery with exponential backoff
    async fn dial_contact(&self, four_words: &str) -> Result<()> {
        // Phase 3 TDD: Check if WAN operations should be attempted
        if !self.context.should_attempt_wan_operations() {
            info!(
                "Skipping WAN dial to {} (local-only mode active)",
                four_words
            );
            return Ok(()); // Non-fatal, just skip the dial
        }

        if let Err(err) = self.context.enforce_resource_limits().await {
            warn!(
                "Resource limits prevent dialing contact {}: {}",
                four_words, err
            );
            return Err(anyhow::anyhow!(err));
        }

        // Phase 2 TDD: Use retry_dial with exponential backoff (MESH_CAPABILITIES.md §3.2)
        let retry_config = RetryConfig::default();
        let four_words_str = four_words.to_string();
        let discovery = self.context.discovery.clone();
        let anti_entropy = self.context.anti_entropy.clone();
        let transport = self.context.transport.clone();
        let peer_cache = self.context.peer_cache.clone();
        let contact_store = self.context.contact_store.clone();

        retry_dial(four_words, retry_config, || {
            let four_words = four_words_str.clone();
            let discovery = discovery.clone();
            let anti_entropy = anti_entropy.clone();
            let transport = transport.clone();
            let peer_cache = peer_cache.clone();
            let contact_store = contact_store.clone();
            async move {
                let discovery_result = discovery.find_contact_with_hints(&four_words).await?;
                let peer_id = discovery_result.peer_id;

                let mut candidates: Vec<SocketAddr> = Vec::new();

                // Contact store endpoint (if any)
                if let Some(contact) = contact_store.get(&four_words).await
                    && let Some(addr) = contact.get_valid_endpoint()
                {
                    candidates.push(addr);
                }

                // Address hints from discovery (presence/FOAF)
                for hint in &discovery_result.addr_hints {
                    if let Ok(addr) = hint.parse::<SocketAddr>() {
                        candidates.push(addr);
                        continue;
                    }
                    if let Ok(addr) = crate::identity::conn_from_words(hint) {
                        candidates.push(addr);
                        continue;
                    }
                    let normalized = hint.replace('-', " ");
                    if normalized != *hint
                        && let Ok(addr) = crate::identity::conn_from_words(&normalized)
                    {
                        candidates.push(addr);
                    }
                }

                // Address hints from peer cache
                for addr in peer_cache.get_addr_hints(peer_id).await {
                    candidates.push(addr);
                }

                // Deduplicate
                let mut seen = HashSet::new();
                candidates.retain(|addr| seen.insert(*addr));

                if candidates.is_empty() {
                    return Err(anyhow::anyhow!(
                        "No address hints available for contact {}",
                        four_words
                    ));
                }

                let mut last_error = None;
                for addr in candidates {
                    match GossipTransport::dial(&transport, peer_id, addr).await {
                        Ok(_) => {
                            info!(
                                "Connected to contact {} at {} (peer {:?})",
                                four_words, addr, peer_id
                            );
                            anti_entropy.add_peer(peer_id).await;

                            // Update peer cache
                            if let Err(e) = peer_cache.record_success(peer_id, addr).await {
                                warn!("Failed to update peer cache for {}: {}", addr, e);
                            }

                            // Update contact store
                            if contact_store.exists(&four_words).await {
                                if let Err(e) =
                                    contact_store.record_success(&four_words, addr).await
                                {
                                    warn!("Failed to record contact success: {}", e);
                                }
                            } else {
                                let mut record =
                                    super::contact_storage::ContactRecord::new(four_words.clone());
                                record.record_success(addr);
                                if let Err(e) = contact_store.add(record).await {
                                    warn!("Failed to add contact for {}: {}", four_words, e);
                                }
                            }

                            return Ok(());
                        }
                        Err(e) => {
                            warn!("Dial to {} at {} failed: {}", four_words, addr, e);
                            last_error = Some(e);
                        }
                    }
                }

                // Record failure once
                if contact_store.exists(&four_words).await
                    && let Err(e) = contact_store.record_failure(&four_words).await
                {
                    warn!("Failed to record contact failure: {}", e);
                }

                if let Some(err) = last_error {
                    Err(err)
                } else {
                    Err(anyhow::anyhow!("Failed to dial contact {}", four_words))
                }
            }
        })
        .await
    }

    /// Step 3: Start membership layer (HyParView + SWIM)
    async fn start_membership(&mut self) -> Result<()> {
        let membership = self.context.membership.clone();
        let seeds = self.get_seed_peers().await?;

        let join_handle = tokio::spawn(async move {
            let membership = membership.write().await;
            membership.join(seeds).await
        });

        match tokio::time::timeout(std::time::Duration::from_secs(3), join_handle).await {
            Ok(Ok(Ok(()))) => {
                info!("Membership layer active, starting periodic shuffle and probes");
            }
            Ok(Ok(Err(e))) => {
                warn!("Membership join failed (continuing without peers): {}", e);
            }
            Ok(Err(e)) => {
                warn!(
                    "Membership join task failed (continuing without peers): {}",
                    e
                );
            }
            Err(_) => {
                warn!("Membership join still in progress; continuing boot sequence");
            }
        }

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
            let introducers = if self.boot_data.introducers.is_empty() {
                super::discovery::IntroducerConfig::default().addresses
            } else {
                self.boot_data.introducers.clone()
            };
            seeds.extend(introducers);
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
        Ok(self.boot_data.entities.clone())
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
                .start(move |peer_id, delta| {
                    let transport = transport.clone();
                    Box::pin(async move {
                        // Serialize delta using postcard for wire transmission
                        let delta_bytes = postcard::to_stdvec(&delta).map_err(|e| {
                            anyhow::anyhow!("Failed to serialize CRDT delta: {}", e)
                        })?;

                        // Send delta to peer via transport using Bulk stream
                        transport
                            .send_to_peer(peer_id, GossipStreamType::Bulk, Bytes::from(delta_bytes))
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!("Failed to send CRDT delta to {:?}: {}", peer_id, e)
                            })?;

                        info!(
                            "Sent CRDT delta ({} bytes) to peer {:?}",
                            delta.added.len() + delta.removed.len(),
                            peer_id
                        );
                        Ok(())
                    })
                })
                .await
                .context("Failed to start CRDT anti-entropy")?;
            info!("CRDT anti-entropy active (60s interval, delta-based sync)");
        }

        // Start transport-to-anti-entropy peer sync (5 second interval)
        // This ensures ALL transport-level connections are registered for CRDT sync,
        // not just membership peers. Critical for bootstrap nodes that receive
        // direct connections from clients.
        {
            let transport = Arc::clone(&self.context.transport);
            let anti_entropy = Arc::clone(&self.context.anti_entropy);
            let pubsub = Arc::clone(&self.context.pubsub);
            let topics = Arc::clone(&self.context.topics);
            let self_peer_id = self.context.peer_id;

            tokio::spawn(async move {
                info!("Starting transport peer sync task (5s interval)");
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

                loop {
                    interval.tick().await;

                    // Get all connected peers from transport layer (includes direct connections)
                    let mut connected = transport.connected_peers().await;

                    // Guard against the local peer showing up in the connected list
                    let before_len = connected.len();
                    connected.retain(|(peer_id, _)| *peer_id != self_peer_id);
                    if connected.len() != before_len {
                        debug!("Filtered out local peer id from connected peers");
                    }

                    if connected.is_empty() {
                        debug!("No connected peers in transport layer");
                        continue;
                    }

                    // Extract peer IDs for reuse
                    let peer_ids: Vec<_> = connected.iter().map(|(pid, _)| *pid).collect();

                    // Register all connected peers for CRDT anti-entropy sync
                    let mut registered = 0;
                    for (peer_id, addr) in &connected {
                        anti_entropy.add_peer(*peer_id).await;
                        registered += 1;
                        debug!(
                            "Registered transport peer {:?} ({}) for CRDT sync",
                            peer_id, addr
                        );
                    }

                    if registered > 0 {
                        info!(
                            "Synced {} transport peers to anti-entropy registry",
                            registered
                        );
                    }

                    // Also sync peers to all subscribed pubsub topics
                    // This ensures topics get peer updates when new connections are made
                    let topic_ids: Vec<_> = {
                        let topics_guard = topics.read().await;
                        topics_guard.values().copied().collect()
                    };

                    if !topic_ids.is_empty() && !peer_ids.is_empty() {
                        let pubsub_guard = pubsub.read().await;
                        for topic_id in topic_ids {
                            (**pubsub_guard)
                                .initialize_topic_peers(topic_id, peer_ids.clone())
                                .await;
                        }
                        debug!("Synced {} transport peers to pubsub topics", peer_ids.len());
                    }
                }
            });
            info!("Transport peer sync task active (5s interval)");
        }

        // Start PubSub message processing loop
        // This receives incoming PubSub messages from transport and routes them to handlers
        {
            let transport = Arc::clone(&self.context.transport);
            let pubsub = Arc::clone(&self.context.pubsub);
            let entity_handler = self.context.entity_message_handler();
            let direct_enabled = matches!(
                std::env::var("COMMUNITAS_DIRECT_ENTITY_BROADCAST")
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase()
                    .as_str(),
                "1" | "true" | "yes"
            );
            let log_transport = matches!(
                std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase()
                    .as_str(),
                "1" | "true" | "yes"
            );

            tokio::spawn(async move {
                info!("Starting PubSub message processing loop");

                loop {
                    match transport.receive_message().await {
                        Ok((peer_id, stream_type, data)) => {
                            if log_transport {
                                info!(
                                    "Transport received {:?} ({} bytes) from {:?}",
                                    stream_type,
                                    data.len(),
                                    peer_id
                                );
                            }
                            // Only process PubSub messages in this loop
                            if stream_type != GossipStreamType::PubSub {
                                if direct_enabled
                                    && matches!(
                                        stream_type,
                                        GossipStreamType::Bulk | GossipStreamType::Membership
                                    )
                                {
                                    if let Ok(envelope) = serde_json::from_slice::<
                                        super::context::DirectEntityEnvelope,
                                    >(
                                        &data
                                    ) {
                                        if log_transport {
                                            info!(
                                                "Received direct entity envelope for {} ({} bytes) from {:?}",
                                                envelope.entity_id,
                                                envelope.payload.len(),
                                                peer_id
                                            );
                                        }
                                        if let Some(handler) = entity_handler.read().await.as_ref()
                                        {
                                            handler(
                                                envelope.entity_id,
                                                peer_id,
                                                Bytes::from(envelope.payload),
                                            );
                                        }
                                    }
                                }
                                debug!("Ignoring non-PubSub message from {:?}", peer_id);
                                continue;
                            }

                            debug!(
                                "Received PubSub message ({} bytes) from {:?}",
                                data.len(),
                                peer_id
                            );

                            // Route to pubsub handler via the trait method
                            let pubsub_guard = pubsub.read().await;
                            if let Err(e) = pubsub_guard.handle_message(peer_id, data).await {
                                warn!("Failed to handle PubSub message from {:?}: {}", peer_id, e);
                            }
                        }
                        Err(e) => {
                            // Log at debug level since receive may return errors when no messages available
                            debug!("Error receiving message from transport: {}", e);
                            // Brief pause to avoid busy loop on persistent errors
                            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        }
                    }
                }
            });
            info!("PubSub message processing loop active");
        }

        // Optional transport probe to validate stream delivery across peers.
        if matches!(
            std::env::var("COMMUNITAS_TRANSPORT_PROBE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        ) {
            let transport = Arc::clone(&self.context.transport);
            tokio::spawn(async move {
                let payload = vec![0xAB; 1024];
                for attempt in 0..5 {
                    let peers = transport.connected_peers().await;
                    if !peers.is_empty() {
                        for (peer_id, _) in peers {
                            let _ = transport
                                .send_to_peer(
                                    peer_id,
                                    GossipStreamType::Membership,
                                    bytes::Bytes::from(payload.clone()),
                                )
                                .await;
                            let _ = transport
                                .send_to_peer(
                                    peer_id,
                                    GossipStreamType::PubSub,
                                    bytes::Bytes::from(payload.clone()),
                                )
                                .await;
                            let _ = transport
                                .send_to_peer(
                                    peer_id,
                                    GossipStreamType::Bulk,
                                    bytes::Bytes::from(payload.clone()),
                                )
                                .await;
                        }
                        break;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(1 + attempt)).await;
                }
            });
            info!("Transport probe enabled");
        }

        Ok(())
    }

    /// Step 6: Start connectivity watchdog monitoring (Phase 3 TDD)
    async fn start_watchdog_monitoring(&self) -> Result<()> {
        // Get reference to watchdog and coordinator
        let watchdog = Arc::clone(&self.context.watchdog);
        let transport = Arc::clone(&self.context.transport);
        let membership = Arc::clone(&self.context.membership);

        // Define health check function that pings bootstrap/coordinator
        let health_check = move || {
            let transport = transport.clone();
            let membership = membership.clone();
            async move {
                let active_peers = membership.read().await.active_view();
                if !active_peers.is_empty() {
                    return true;
                }

                let connected = transport.connected_peers().await;
                !connected.is_empty()
            }
        };

        // Start monitoring in background task
        // Note: start_monitoring takes ownership of a ConnectivityWatchdog, not Arc
        // We need to clone the inner value
        let watchdog_inner = (*watchdog).clone();
        let _handle = watchdog_inner.start_monitoring(health_check);

        // Note: We don't await the handle - it runs in the background
        // The watchdog will update local_only_mode state as needed

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
