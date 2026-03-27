//! Exact WebSocket client for real-time x0xd sessions.
//!
//! Supports the daemon's `/ws` and `/ws/direct` endpoints. Note that `/events`
//! and `/direct/events` are server-sent event streams, not WebSocket endpoints.
//!
//! Authentication is handled by appending `?token=<token>` to the WebSocket URL,
//! as required by the x0xd WebSocket API. Use [`X0xWebSocket::connect`] or
//! [`X0xWebSocket::connect_direct`] with auto-discovery, or
//! [`X0xWebSocket::connect_with_token`] / [`X0xWebSocket::connect_direct_with_token`]
//! when you already have a token.

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::error::{Result, X0xError};
use crate::types::{WsInbound, WsOutbound};

/// A WebSocket connection to the x0xd daemon.
///
/// Provides typed send/receive for the x0x WebSocket protocol.
/// The connection is maintained by a background task; dropping the
/// handle closes the connection.
///
/// Authentication is performed by appending `?token=<token>` to the WebSocket
/// URL. Use [`X0xWebSocket::connect`] to auto-discover the token from the x0xd
/// config files, or [`X0xWebSocket::connect_with_token`] when you already have
/// a [`crate::config::X0xConfig`].
pub struct X0xWebSocket {
    tx: mpsc::UnboundedSender<WsOutbound>,
    rx: mpsc::UnboundedReceiver<WsInbound>,
}

impl X0xWebSocket {
    /// Connect to the daemon's general-purpose WebSocket endpoint.
    ///
    /// Attempts to auto-discover the daemon address and token from x0xd config
    /// files. Falls back to `ws://127.0.0.1:12700/ws` with no token if the
    /// config files are not found.
    pub async fn connect() -> Result<Self> {
        match crate::config::discover() {
            Ok(cfg) => Self::connect_with_config(&cfg, "/ws").await,
            Err(e) => {
                tracing::debug!(
                    "x0x config discovery failed ({e}); falling back to ws://127.0.0.1:12700/ws with no token"
                );
                Self::connect_to("ws://127.0.0.1:12700/ws").await
            }
        }
    }

    /// Connect to the daemon's direct-message WebSocket endpoint.
    ///
    /// Attempts to auto-discover the daemon address and token from x0xd config
    /// files. Falls back to `ws://127.0.0.1:12700/ws/direct` with no token if
    /// the config files are not found.
    pub async fn connect_direct() -> Result<Self> {
        match crate::config::discover() {
            Ok(cfg) => Self::connect_with_config(&cfg, "/ws/direct").await,
            Err(e) => {
                tracing::debug!(
                    "x0x config discovery failed ({e}); falling back to ws://127.0.0.1:12700/ws/direct with no token"
                );
                Self::connect_to("ws://127.0.0.1:12700/ws/direct").await
            }
        }
    }

    /// Connect to the general-purpose WebSocket endpoint using an explicit token.
    pub async fn connect_with_token(address: &str, token: &str) -> Result<Self> {
        let url = format!("ws://{address}/ws?token={token}");
        Self::connect_to(&url).await
    }

    /// Connect to the direct-message WebSocket endpoint using an explicit token.
    pub async fn connect_direct_with_token(address: &str, token: &str) -> Result<Self> {
        let url = format!("ws://{address}/ws/direct?token={token}");
        Self::connect_to(&url).await
    }

    /// Connect using a discovered [`crate::config::X0xConfig`] and a path (e.g. `/ws`).
    async fn connect_with_config(cfg: &crate::config::X0xConfig, path: &str) -> Result<Self> {
        let url = format!("ws://{}{}?token={}", cfg.address, path, cfg.token);
        Self::connect_to(&url).await
    }

    /// Connect to a custom WebSocket URL.
    pub async fn connect_to(url: &str) -> Result<Self> {
        let (ws_stream, _) = connect_async(url)
            .await
            .map_err(|e| X0xError::WebSocket(Box::new(e)))?;
        let (mut ws_sink, mut ws_source) = ws_stream.split();

        // Channel: caller -> websocket
        let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<WsOutbound>();
        // Channel: websocket -> caller
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel::<WsInbound>();

        // Send task: forward outbound messages to the WebSocket.
        tokio::spawn(async move {
            while let Some(msg) = outbound_rx.recv().await {
                let json = match serde_json::to_string(&msg) {
                    Ok(j) => j,
                    Err(e) => {
                        tracing::error!("failed to serialize outbound WS message: {e}");
                        continue;
                    }
                };
                if ws_sink.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        });

        // Receive task: forward inbound WebSocket messages to the channel.
        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_source.next().await {
                let text = match msg {
                    Message::Text(t) => t.to_string(),
                    Message::Close(_) => break,
                    _ => continue,
                };
                match serde_json::from_str::<WsInbound>(&text) {
                    Ok(inbound) => {
                        if inbound_tx.send(inbound).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("failed to parse inbound WS message: {e}: {text}");
                    }
                }
            }
        });

        Ok(Self {
            tx: outbound_tx,
            rx: inbound_rx,
        })
    }

    /// Subscribe to one or more gossip topics.
    pub fn subscribe(&self, topics: Vec<String>) -> Result<()> {
        self.tx.send(WsOutbound::Subscribe { topics }).map_err(|_| {
            X0xError::WebSocket(Box::new(
                tokio_tungstenite::tungstenite::Error::ConnectionClosed,
            ))
        })
    }

    /// Unsubscribe from one or more gossip topics.
    pub fn unsubscribe(&self, topics: Vec<String>) -> Result<()> {
        self.tx
            .send(WsOutbound::Unsubscribe { topics })
            .map_err(|_| {
                X0xError::WebSocket(Box::new(
                    tokio_tungstenite::tungstenite::Error::ConnectionClosed,
                ))
            })
    }

    /// Publish a payload to a gossip topic.
    pub fn publish(&self, topic: String, payload: String) -> Result<()> {
        self.tx
            .send(WsOutbound::Publish { topic, payload })
            .map_err(|_| {
                X0xError::WebSocket(Box::new(
                    tokio_tungstenite::tungstenite::Error::ConnectionClosed,
                ))
            })
    }

    /// Send a direct message to an agent.
    pub fn send_direct(&self, agent_id: String, payload: String) -> Result<()> {
        self.tx
            .send(WsOutbound::SendDirect { agent_id, payload })
            .map_err(|_| {
                X0xError::WebSocket(Box::new(
                    tokio_tungstenite::tungstenite::Error::ConnectionClosed,
                ))
            })
    }

    /// Send a ping.
    pub fn ping(&self) -> Result<()> {
        self.tx.send(WsOutbound::Ping).map_err(|_| {
            X0xError::WebSocket(Box::new(
                tokio_tungstenite::tungstenite::Error::ConnectionClosed,
            ))
        })
    }

    /// Receive the next inbound message. Returns `None` if the connection is closed.
    pub async fn recv(&mut self) -> Option<WsInbound> {
        self.rx.recv().await
    }
}
