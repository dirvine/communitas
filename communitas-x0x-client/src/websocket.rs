//! WebSocket client for real-time x0xd event streaming.
//!
//! Connects to `ws://127.0.0.1:12700/ws` and provides a typed send/receive
//! interface for gossip messages, direct messages, and topic subscriptions.

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
pub struct X0xWebSocket {
    tx: mpsc::UnboundedSender<WsOutbound>,
    rx: mpsc::UnboundedReceiver<WsInbound>,
}

impl X0xWebSocket {
    /// Connect to the x0xd WebSocket at the default address.
    pub async fn connect() -> Result<Self> {
        Self::connect_to("ws://127.0.0.1:12700/ws").await
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
