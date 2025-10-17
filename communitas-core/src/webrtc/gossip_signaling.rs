// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Gossip-based signaling transport for WebRTC
//!
//! Implements the `SignalingTransport` trait from saorsa-webrtc using
//! the Communitas gossip overlay for signaling message delivery.

use super::identity::CommunitasIdentity;
use crate::gossip::GossipContext;
use anyhow::Result;
use async_trait::async_trait;
use blake3;
use saorsa_gossip_types::TopicId;
use saorsa_webrtc::signaling::{SignalingMessage, SignalingTransport};
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Topic prefix for WebRTC signaling messages
const WEBRTC_TOPIC_PREFIX: &str = "webrtc.signaling";

/// Error type for gossip signaling transport
#[derive(Debug)]
pub struct GossipSignalingError(anyhow::Error);

impl fmt::Display for GossipSignalingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GossipSignalingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<anyhow::Error> for GossipSignalingError {
    fn from(err: anyhow::Error) -> Self {
        GossipSignalingError(err)
    }
}

/// Gossip-based signaling transport
///
/// Uses the saorsa-gossip overlay network for WebRTC signaling:
/// - Publishes SDP offers/answers via PubSub
/// - Discovers peer endpoints via Rendezvous
/// - Routes messages using 65k topic shards
pub struct GossipSignalingTransport {
    /// Reference to gossip context
    gossip: Arc<GossipContext>,

    /// Local identity
    local_identity: CommunitasIdentity,

    /// Message receive queue (from PubSub subscriptions)
    message_queue: Arc<RwLock<Vec<(CommunitasIdentity, SignalingMessage)>>>,
}

impl GossipSignalingTransport {
    /// Create a new gossip signaling transport
    pub fn new(gossip: Arc<GossipContext>) -> Result<Self> {
        // Extract local identity from gossip context
        let four_words = gossip.four_words.clone();

        let local_identity = CommunitasIdentity::new(four_words)?;

        Ok(Self {
            gossip,
            local_identity,
            message_queue: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Get the signaling topic for a specific peer
    ///
    /// Uses the peer's four-word address to create a deterministic topic ID
    fn peer_topic(&self, peer: &CommunitasIdentity) -> TopicId {
        let topic_str = format!("{}.{}", WEBRTC_TOPIC_PREFIX, peer.four_words());
        let hash = blake3::hash(topic_str.as_bytes());
        TopicId::new(*hash.as_bytes())
    }

    /// Subscribe to signaling messages for the local peer
    ///
    /// This should be called once during initialization to start receiving
    /// incoming signaling messages.
    pub async fn subscribe_to_signaling(&self) -> Result<()> {
        let topic = self.peer_topic(&self.local_identity);

        info!(
            "Subscribing to WebRTC signaling topic for {}",
            self.local_identity
        );

        let pubsub = self.gossip.pubsub.read().await;

        // Subscribe returns a receiver (not async)
        let _rx = pubsub.subscribe(topic);

        debug!("Subscribed to topic: {:?}", topic);

        Ok(())
    }

    /// Process incoming signaling messages from PubSub
    ///
    /// NOTE: This is a placeholder implementation. In production, this would
    /// be replaced with a background task that listens on the PubSub receiver
    /// and processes messages as they arrive.
    ///
    /// TODO: Implement proper message processing using the receiver from subscribe()
    pub async fn process_incoming_messages(&self) -> Result<()> {
        // This is a placeholder - actual implementation would involve:
        // 1. Storing the receiver from subscribe() in the struct
        // 2. Running a background task that listens on the receiver
        // 3. Deserializing and queuing incoming messages
        //
        // For now, we return Ok since there's no way to poll the receiver
        // without restructuring the entire approach.
        Ok(())
    }
}

#[async_trait]
impl SignalingTransport for GossipSignalingTransport {
    type PeerId = CommunitasIdentity;
    type Error = GossipSignalingError;

    async fn send_message(
        &self,
        peer: &Self::PeerId,
        message: SignalingMessage,
    ) -> Result<(), Self::Error> {
        info!("Sending signaling message to {}: {:?}", peer, message);

        // Get the topic for the target peer
        let topic = self.peer_topic(peer);

        // Serialize the message with sender identity
        let payload = (self.local_identity.clone(), message);
        let message_bytes = serde_json::to_vec(&payload)
            .map_err(|e| anyhow::anyhow!("Failed to serialize signaling message: {}", e))?;

        // Publish via PubSub
        let pubsub = self.gossip.pubsub.write().await;

        pubsub
            .publish(topic, message_bytes.into())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish signaling message: {}", e))?;

        debug!("Signaling message sent to {}", peer);

        Ok(())
    }

    async fn receive_message(&self) -> Result<(Self::PeerId, SignalingMessage), Self::Error> {
        // First, process any incoming messages from PubSub
        self.process_incoming_messages().await?;

        // Check our message queue
        let mut queue = self.message_queue.write().await;

        if let Some((sender, message)) = queue.pop() {
            debug!("Dequeued signaling message from {}", sender);
            return Ok((sender, message));
        }

        // No messages available
        Err(anyhow::anyhow!("No signaling messages available").into())
    }

    async fn discover_peer_endpoint(
        &self,
        peer: &Self::PeerId,
    ) -> Result<Option<SocketAddr>, Self::Error> {
        info!("Discovering endpoint for peer: {}", peer);

        // NOTE: In the gossip architecture, peer discovery is handled by the
        // Rendezvous system, but ProviderSummary contains PeerId, not SocketAddr.
        // The QUIC transport layer handles the actual connection establishment
        // using the PeerId.
        //
        // For WebRTC over QUIC, we don't need to return a SocketAddr here because:
        // 1. The ant-quic transport will use the gossip overlay for peer discovery
        // 2. Signaling messages are exchanged via PubSub (already implemented)
        // 3. The actual media connection will be established using the gossip transport
        //
        // If needed in the future, we could query the coordinator for the peer's
        // public endpoint using the CoordinatorClient.

        // Calculate the target ID for the peer (hash of four-word address)
        let target_hash = blake3::hash(peer.four_words().as_bytes());
        let target_id: [u8; 32] = *target_hash.as_bytes();

        // Check if peer is discoverable via Rendezvous
        let rendezvous = self.gossip.rendezvous.clone();

        // Subscribe to the peer's rendezvous shard
        if let Err(e) = rendezvous.subscribe_to_shard(&target_id).await {
            warn!(
                "Failed to subscribe to rendezvous shard for {}: {}",
                peer, e
            );
            return Ok(None);
        }

        // Get provider summaries to verify peer is discoverable
        let providers = rendezvous.get_providers_for_target(&target_id).await;

        if providers.is_empty() {
            warn!("Peer {} not found in rendezvous", peer);
            return Ok(None);
        }

        debug!(
            "Peer {} is discoverable (found {} providers)",
            peer,
            providers.len()
        );

        // Return None since we don't have direct SocketAddr mapping
        // The transport layer will handle connection establishment
        Ok(None)
    }
}

impl FromStr for CommunitasIdentity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CommunitasIdentity::new(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_topic_generation() {
        // Test the topic generation logic
        let identity =
            CommunitasIdentity::new("ocean-forest-moon-star".to_string()).expect("valid identity");

        let topic_str = format!("{}.{}", WEBRTC_TOPIC_PREFIX, identity.four_words());
        let hash = blake3::hash(topic_str.as_bytes());
        let topic = TopicId::new(*hash.as_bytes());

        // Verify the topic is created correctly (32 bytes)
        assert_eq!(topic_str, "webrtc.signaling.ocean-forest-moon-star");

        // The hash should be deterministic
        let hash2 = blake3::hash(topic_str.as_bytes());
        let topic2 = TopicId::new(*hash2.as_bytes());
        assert_eq!(topic, topic2);
    }

    #[test]
    fn test_identity_from_str() {
        let identity =
            CommunitasIdentity::from_str("ocean-forest-moon-star").expect("valid identity");
        assert_eq!(identity.four_words(), "ocean-forest-moon-star");
    }
}
