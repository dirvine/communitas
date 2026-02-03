// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Sites Message Dispatcher
//!
//! Coordinates message routing between SitesListener (serving requests) and
//! SiteFetcher (receiving responses). Prevents the race condition where both
//! components try to receive from the same transport.
//!
//! ## Architecture
//! - Single receive loop on the Sites transport
//! - Routes SitesWire::Request to SitesListener
//! - Routes SitesWire::Response to waiting fetchers via channel
//! - Prevents message loss and ensures proper correlation

use anyhow::Result;
use bytes::Bytes;
use saorsa_gossip_transport::{GossipStreamType, GossipTransport};
use saorsa_gossip_types::PeerId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, warn};

use super::sites::{SiteResponse, SitesWire};
use super::sites_listener::SitesListener;

/// Channel capacity for pending responses
const RESPONSE_CHANNEL_SIZE: usize = 100;

/// Dispatcher for Sites protocol messages
///
/// This component runs a single receive loop on the Sites transport and
/// routes messages to the appropriate handler (listener for requests,
/// fetcher channels for responses).
pub struct SitesDispatcher {
    /// Transport to receive from
    transport: Arc<dyn GossipTransport + Send + Sync>,

    /// Sites listener for handling requests
    listener: Arc<SitesListener>,

    /// Response channels indexed by correlation ID
    /// When a fetcher makes a request, it registers a channel here
    response_channels: Arc<RwLock<HashMap<u64, mpsc::Sender<SiteResponse>>>>,

    /// Shutdown signal
    shutdown: Arc<tokio::sync::Notify>,
}

impl SitesDispatcher {
    /// Create a new Sites dispatcher
    pub fn new(
        transport: Arc<dyn GossipTransport + Send + Sync>,
        listener: Arc<SitesListener>,
    ) -> Self {
        Self {
            transport,
            listener,
            response_channels: Arc::new(RwLock::new(HashMap::new())),
            shutdown: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Start the dispatcher's receive loop
    ///
    /// Returns a JoinHandle for the background task
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self.shutdown.notified() => {
                        debug!("Sites dispatcher shutting down");
                        break;
                    }
                    result = self.transport.receive_message() => {
                        match result {
                            Ok((peer_id, stream_type, data)) => {
                                if let Err(e) = self.handle_message(peer_id, stream_type, data).await {
                                    warn!("Failed to handle Sites message: {}", e);
                                }
                            }
                            Err(e) => {
                                let err_str = e.to_string();
                                if err_str.contains("No messages available") {
                                    // This is likely a timeout or empty queue in the transport layer
                                    // Just sleep and retry without spamming logs
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                } else {
                                    warn!("Sites transport receive error: {}", e);
                                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    /// Stop the dispatcher
    pub fn stop(&self) {
        self.shutdown.notify_one();
    }

    /// Handle an incoming message
    async fn handle_message(
        &self,
        peer_id: PeerId,
        stream_type: GossipStreamType,
        data: Bytes,
    ) -> Result<()> {
        // Only handle Bulk stream
        if stream_type != GossipStreamType::Bulk {
            return Ok(());
        }

        // Try to deserialize as SitesWire
        let wire_msg: SitesWire = match postcard::from_bytes(&data) {
            Ok(msg) => msg,
            Err(_) => {
                // Not a Sites message - ignore
                return Ok(());
            }
        };

        match wire_msg {
            SitesWire::Request { .. } => {
                // Forward to listener for handling
                // listener.maybe_handle_incoming will process the request
                // and send back a response
                self.listener
                    .maybe_handle_incoming(peer_id, stream_type, data)
                    .await;
                Ok(())
            }
            SitesWire::Response { id, body } => {
                // Route to the waiting fetcher
                let channels = self.response_channels.read().await;
                if let Some(tx) = channels.get(&id) {
                    // Send response to the waiting fetcher
                    if tx.send(body).await.is_err() {
                        warn!("Failed to send response {} - receiver dropped", id);
                    } else {
                        debug!("Routed response {} to fetcher", id);
                    }
                } else {
                    warn!("Received response {} with no waiting fetcher", id);
                }
                Ok(())
            }
        }
    }

    /// Register a response channel for a request
    ///
    /// Called by SiteFetcher before sending a request
    pub async fn register_response_channel(&self, request_id: u64) -> mpsc::Receiver<SiteResponse> {
        let (tx, rx) = mpsc::channel(RESPONSE_CHANNEL_SIZE);
        let mut channels = self.response_channels.write().await;
        channels.insert(request_id, tx);
        rx
    }

    /// Unregister a response channel
    ///
    /// Called by SiteFetcher after receiving a response (or timeout)
    pub async fn unregister_response_channel(&self, request_id: u64) {
        let mut channels = self.response_channels.write().await;
        channels.remove(&request_id);
    }
}
