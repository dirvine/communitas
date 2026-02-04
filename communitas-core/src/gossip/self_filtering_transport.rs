// Copyright (c) 2025 Saorsa Labs Limited
//
// Transport wrapper that ignores self-send attempts to avoid pubsub errors.

use saorsa_gossip_transport::{
    GossipStreamType, GossipTransport, GossipTransportResult, TransportAdapter,
};
use saorsa_gossip_types::PeerId;
use std::net::SocketAddr;
use std::sync::Arc;

/// Wraps a transport and suppresses sends to the local peer.
#[derive(Clone)]
pub struct SelfFilteringTransport<T> {
    inner: Arc<T>,
    local_peer: PeerId,
}

impl<T> SelfFilteringTransport<T> {
    pub fn new(inner: Arc<T>) -> Self
    where
        T: GossipTransport,
    {
        let local_peer = inner.local_peer_id();
        Self { inner, local_peer }
    }

    pub fn inner(&self) -> &Arc<T> {
        &self.inner
    }
}

#[async_trait::async_trait]
impl<T> GossipTransport for SelfFilteringTransport<T>
where
    T: GossipTransport + Send + Sync + 'static,
{
    async fn dial(&self, peer: PeerId, addr: SocketAddr) -> anyhow::Result<()> {
        if peer == self.local_peer {
            return Ok(());
        }
        self.inner.dial(peer, addr).await
    }

    async fn dial_bootstrap(&self, addr: SocketAddr) -> anyhow::Result<PeerId> {
        self.inner.dial_bootstrap(addr).await
    }

    async fn listen(&self, bind: SocketAddr) -> anyhow::Result<()> {
        self.inner.listen(bind).await
    }

    async fn close(&self) -> anyhow::Result<()> {
        self.inner.close().await
    }

    async fn send_to_peer(
        &self,
        peer: PeerId,
        stream_type: GossipStreamType,
        data: bytes::Bytes,
    ) -> anyhow::Result<()> {
        if peer == self.local_peer {
            return Ok(());
        }
        let log_transport = matches!(
            std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        if log_transport {
            tracing::info!(
                "Transport send_to_peer {:?} ({} bytes) -> {:?}",
                stream_type,
                data.len(),
                peer
            );
        }
        self.inner.send_to_peer(peer, stream_type, data).await
    }

    async fn receive_message(&self) -> anyhow::Result<(PeerId, GossipStreamType, bytes::Bytes)> {
        let result = self.inner.receive_message().await;
        if let Ok((peer_id, stream_type, data)) = &result {
            let log_transport = matches!(
                std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase()
                    .as_str(),
                "1" | "true" | "yes"
            );
            if log_transport {
                tracing::info!(
                    "Transport receive_message {:?} ({} bytes) <- {:?}",
                    stream_type,
                    data.len(),
                    peer_id
                );
            }
        }
        result
    }

    fn local_peer_id(&self) -> PeerId {
        self.local_peer
    }
}

#[async_trait::async_trait]
impl<T> TransportAdapter for SelfFilteringTransport<T>
where
    T: TransportAdapter + Send + Sync,
{
    fn local_peer_id(&self) -> PeerId {
        self.local_peer
    }

    async fn dial(&self, addr: SocketAddr) -> GossipTransportResult<PeerId> {
        self.inner.dial(addr).await
    }

    async fn send(
        &self,
        peer_id: PeerId,
        stream_type: GossipStreamType,
        data: bytes::Bytes,
    ) -> GossipTransportResult<()> {
        if peer_id == self.local_peer {
            return Ok(());
        }
        let log_transport = matches!(
            std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
                .as_str(),
            "1" | "true" | "yes"
        );
        if log_transport {
            tracing::info!(
                "Transport adapter send {:?} ({} bytes) -> {:?}",
                stream_type,
                data.len(),
                peer_id
            );
        }
        self.inner.send(peer_id, stream_type, data).await
    }

    async fn recv(&self) -> GossipTransportResult<(PeerId, GossipStreamType, bytes::Bytes)> {
        let result = self.inner.recv().await;
        if let Ok((peer_id, stream_type, data)) = &result {
            let log_transport = matches!(
                std::env::var("COMMUNITAS_LOG_TRANSPORT_RECEIVE")
                    .unwrap_or_default()
                    .trim()
                    .to_ascii_lowercase()
                    .as_str(),
                "1" | "true" | "yes"
            );
            if log_transport {
                tracing::info!(
                    "Transport adapter recv {:?} ({} bytes) <- {:?}",
                    stream_type,
                    data.len(),
                    peer_id
                );
            }
        }
        result
    }

    async fn close(&self) -> GossipTransportResult<()> {
        self.inner.close().await
    }

    async fn connected_peers(&self) -> Vec<(PeerId, SocketAddr)> {
        let peers = self.inner.connected_peers().await;
        peers
            .into_iter()
            .filter(|(peer, _)| *peer != self.local_peer)
            .collect()
    }

    fn capabilities(&self) -> saorsa_gossip_transport::TransportCapabilities {
        self.inner.capabilities()
    }
}
