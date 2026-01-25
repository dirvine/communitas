// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Sites Protocol Listener
//!
//! Listens for incoming Sites requests on Bulk streams and routes
//! them to the appropriate SitePublisher.
//!
//! ## Architecture
//! - Runs as background task in GossipContext
//! - Listens on Bulk stream for SiteRequest messages
//! - Routes to SitePublisher.handle_request()
//! - Sends SiteResponse back to requester
//! - Implements backpressure and timeouts

use bytes::Bytes;
use saorsa_gossip_transport::{GossipStreamType, GossipTransport};
use saorsa_gossip_types::PeerId;
use std::sync::Arc;
use tokio::time::{Duration, timeout};
use tracing::{debug, info, warn};

use super::sites::{SitePublisher, SiteResponse, SitesWire};

/// Timeout for processing a single request (30 seconds)
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum concurrent requests per publisher (backpressure)
const MAX_CONCURRENT_REQUESTS: usize = 10;

/// Sites protocol listener
///
/// Handles incoming Sites requests pushed by the central message dispatcher.
/// Does NOT own a receive loop - messages are routed to it by GossipContext.
pub struct SitesListener {
    /// Transport for sending responses only (receive is done by central dispatcher)
    transport: Arc<dyn GossipTransport + Send + Sync>,

    /// Site publisher (if we're publishing sites)
    publisher: Option<Arc<SitePublisher>>,

    /// Semaphore for backpressure (limits concurrent requests)
    active_requests: Arc<tokio::sync::Semaphore>,

    /// Flag to stop the listener
    shutdown: Arc<tokio::sync::Notify>,
}

impl SitesListener {
    /// Create a new Sites protocol listener
    ///
    /// # Arguments
    /// * `transport` - Bound gossip transport for sending responses (receive is centralized)
    /// * `publisher` - Optional site publisher (None if not publishing)
    pub fn new(
        transport: Arc<dyn GossipTransport + Send + Sync>,
        publisher: Option<Arc<SitePublisher>>,
    ) -> Self {
        Self {
            transport,
            publisher,
            active_requests: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_REQUESTS)),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Start listening on a dedicated transport
    ///
    /// Creates a receive loop that calls maybe_handle_incoming for each message.
    /// This should ONLY be used with a dedicated Sites transport, never the main
    /// gossip transport (which is shared by Membership/PubSub/Presence).
    ///
    /// # Arguments
    /// * `transport` - Dedicated bound transport for Sites (not shared!)
    ///
    /// # Returns
    /// JoinHandle for the background task
    pub fn start_on_transport(
        self: Arc<Self>,
        transport: Arc<impl GossipTransport + 'static>,
    ) -> tokio::task::JoinHandle<()> {
        info!("Starting Sites listener on dedicated transport");

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self.shutdown.notified() => {
                        info!("Sites listener shutting down");
                        break;
                    }
                    result = transport.receive_message() => {
                        match result {
                            Ok((peer_id, stream_type, data)) => {
                                // Process via maybe_handle_incoming
                                let _ = self.maybe_handle_incoming(peer_id, stream_type, data).await;
                            }
                            Err(e) => {
                                warn!("Sites transport receive error: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            }
        })
    }

    /// Start listening (for tests only - uses internal transport)
    ///
    /// DEPRECATED: Use start_on_transport() with a dedicated bound transport instead.
    /// This method is kept for backward compatibility with tests.
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        warn!("SitesListener.start() called without transport - this is for tests only");

        tokio::spawn(async move {
            // Just wait for shutdown signal
            self.shutdown.notified().await;
            info!("Sites listener shutting down");
        })
    }

    /// Stop the listener gracefully
    pub fn stop(&self) {
        self.shutdown.notify_one();
    }

    /// Get access to the underlying transport (for peer routing setup)
    pub fn transport(&self) -> &Arc<dyn GossipTransport + Send + Sync> {
        &self.transport
    }

    /// Try to handle an incoming message (called by central dispatcher)
    ///
    /// This checks if the message is a Sites request and processes it if so.
    /// Uses SitesWire envelope to distinguish requests from responses - only
    /// handles requests, returns false for responses so SiteFetcher can handle them.
    ///
    /// # Arguments
    /// * `peer_id` - Sender peer ID
    /// * `stream_type` - Stream type (we only handle Bulk)
    /// * `message_bytes` - Raw message bytes
    ///
    /// # Returns
    /// true if this was a Sites request (consumed), false otherwise (including responses)
    pub async fn maybe_handle_incoming(
        &self,
        peer_id: PeerId,
        stream_type: GossipStreamType,
        message_bytes: Bytes,
    ) -> bool {
        debug!(
            "SitesListener.maybe_handle_incoming called: peer_id={:?}, stream_type={:?}, bytes={}",
            peer_id,
            stream_type,
            message_bytes.len()
        );

        // Only handle Bulk stream (Sites protocol uses Bulk)
        if stream_type != GossipStreamType::Bulk {
            debug!(
                "SitesListener: ignoring non-Bulk stream type: {:?}",
                stream_type
            );
            return false;
        }

        // Try to deserialize as SitesWire envelope
        let wire_msg: SitesWire = match bincode::deserialize(&message_bytes) {
            Ok(msg) => msg,
            Err(_) => return false, // Not a Sites message; let others handle
        };

        // Only handle Requests, let SiteFetcher handle Responses
        let (request_id, request) = match wire_msg {
            SitesWire::Request { id, body } => (id, body),
            SitesWire::Response { .. } => {
                // This is a response meant for SiteFetcher - don't consume it
                return false;
            }
        };

        debug!(
            "Received Sites request {} from {}: {:?}",
            request_id, peer_id, request
        );

        // Acquire semaphore permit (backpressure)
        let permit = match self.active_requests.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                warn!(
                    "Max concurrent requests reached ({}), dropping request from {}",
                    MAX_CONCURRENT_REQUESTS, peer_id
                );
                // Send overload error with correlation ID
                let error_response = SiteResponse::Error("Server overloaded".to_string());
                let wire_response = SitesWire::Response {
                    id: request_id,
                    body: error_response,
                };
                if let Ok(error_bytes) = bincode::serialize(&wire_response) {
                    let _ = self
                        .transport
                        .send_to_peer(peer_id, GossipStreamType::Bulk, Bytes::from(error_bytes))
                        .await;
                }
                return true; // We recognized it as Sites, even if rejected
            }
        };

        // Spawn task to handle request (don't block listener)
        let publisher = self.publisher.clone();
        let transport = self.transport.clone();

        tokio::spawn(async move {
            let _permit = permit; // Hold permit until done

            // Helper to send wrapped response
            let send_response = |response: SiteResponse| async {
                let wire_response = SitesWire::Response {
                    id: request_id,
                    body: response,
                };
                if let Ok(response_bytes) = bincode::serialize(&wire_response)
                    && let Err(e) = transport
                        .send_to_peer(peer_id, GossipStreamType::Bulk, Bytes::from(response_bytes))
                        .await
                {
                    warn!("Failed to send Sites response: {}", e);
                }
            };

            if let Some(pub_arc) = publisher {
                // Serialize the request for the publisher
                let request_bytes = match bincode::serialize(&request) {
                    Ok(bytes) => Bytes::from(bytes),
                    Err(e) => {
                        warn!("Failed to serialize request: {}", e);
                        send_response(SiteResponse::Error(format!("Serialization error: {}", e)))
                            .await;
                        return;
                    }
                };

                // Process request with timeout
                match timeout(REQUEST_TIMEOUT, pub_arc.handle_request(request_bytes)).await {
                    Ok(Ok(response_bytes)) => {
                        // Deserialize the response from publisher
                        match bincode::deserialize::<SiteResponse>(&response_bytes) {
                            Ok(response) => {
                                // Success - send wrapped response
                                debug!("Sending Sites response {} to {}", request_id, peer_id);
                                send_response(response).await;
                            }
                            Err(e) => {
                                warn!("Failed to deserialize publisher response: {}", e);
                                send_response(SiteResponse::Error(format!(
                                    "Deserialization error: {}",
                                    e
                                )))
                                .await;
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        // Request processing failed
                        warn!("Request processing failed: {}", e);
                        send_response(SiteResponse::Error(e.to_string())).await;
                    }
                    Err(_) => {
                        // Timeout
                        warn!("Request timeout after {:?}", REQUEST_TIMEOUT);
                        send_response(SiteResponse::Error("Request timeout".to_string())).await;
                    }
                }
            } else {
                // Not publishing sites
                debug!("Received Sites request but not publishing");
                send_response(SiteResponse::Error("Not publishing sites".to_string())).await;
            }
        });

        true // We consumed this message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::sites::SiteId;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;
    use saorsa_gossip_transport::{UdpTransportAdapter, UdpTransportAdapterConfig};
    use saorsa_pqc::ml_dsa_65::try_keygen_with_rng;

    fn generate_test_keypair(
        seed: u64,
    ) -> (
        saorsa_pqc::ml_dsa_65::PrivateKey,
        saorsa_pqc::ml_dsa_65::PublicKey,
    ) {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let (pk, sk) = try_keygen_with_rng(&mut rng).expect("Failed to generate test keypair");
        (sk, pk)
    }

    #[tokio::test]
    async fn test_listener_creation() {
        let config = UdpTransportAdapterConfig::new("0.0.0.0:0".parse().unwrap(), vec![]);
        let qt = UdpTransportAdapter::with_config(config, None)
            .await
            .expect("transport");
        let transport: Arc<dyn GossipTransport + Send + Sync> = Arc::new(qt);

        let listener = SitesListener::new(transport, None);
        assert!(listener.publisher.is_none());
    }

    #[tokio::test]
    async fn test_listener_with_publisher() {
        let config = UdpTransportAdapterConfig::new("0.0.0.0:0".parse().unwrap(), vec![]);
        let qt = UdpTransportAdapter::with_config(config, None)
            .await
            .expect("transport");
        let transport: Arc<dyn GossipTransport + Send + Sync> = Arc::new(qt);

        let (_sk, pk) = generate_test_keypair(1);
        let site_id = SiteId::from_public_key(&pk);
        let publisher = Arc::new(SitePublisher::new(site_id));

        let listener = SitesListener::new(transport, Some(publisher));
        assert!(listener.publisher.is_some());
    }
}
