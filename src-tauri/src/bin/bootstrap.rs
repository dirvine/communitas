use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
struct BootstrapState {
    connected_peers: Arc<RwLock<HashMap<String, std::time::Instant>>>,
}

impl BootstrapState {
    fn new() -> Self {
        Self {
            connected_peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_peer(&self, peer_addr: String) {
        let mut peers = self.connected_peers.write().await;
        peers.insert(peer_addr.clone(), std::time::Instant::now());
        info!("Added peer: {} (Total peers: {})", peer_addr, peers.len());
    }

    async fn remove_peer(&self, peer_addr: &str) {
        let mut peers = self.connected_peers.write().await;
        peers.remove(peer_addr);
        info!("Removed peer: {} (Total peers: {})", peer_addr, peers.len());
    }

    async fn get_peer_list(&self) -> Vec<String> {
        let peers = self.connected_peers.read().await;
        peers.keys().cloned().collect()
    }

    async fn cleanup_stale_peers(&self) {
        let mut peers = self.connected_peers.write().await;
        let now = std::time::Instant::now();
        let stale_threshold = std::time::Duration::from_secs(300); // 5 minutes

        let initial_count = peers.len();
        peers.retain(|_, last_seen| now.duration_since(*last_seen) < stale_threshold);

        if peers.len() != initial_count {
            info!("Cleaned up stale peers. Active peers: {}", peers.len());
        }
    }
}

async fn handle_peer_connection(
    mut socket: tokio::net::TcpStream,
    peer_addr: std::net::SocketAddr,
    state: BootstrapState,
) -> Result<(), Box<dyn std::error::Error>> {
    let peer_id = peer_addr.to_string();
    info!("Handling connection from {}", peer_id);

    // Add peer to active list
    state.add_peer(peer_id.clone()).await;

    // Simple handshake
    let handshake_msg = b"COMMUNITAS_BOOTSTRAP_V1\n";
    if let Err(e) = socket.write_all(handshake_msg).await {
        warn!("Failed to send handshake to {}: {}", peer_id, e);
        state.remove_peer(&peer_id).await;
        return Ok(());
    }

    // Read peer's handshake response
    let mut buffer = [0u8; 1024];
    match tokio::time::timeout(std::time::Duration::from_secs(10), socket.read(&mut buffer)).await {
        Ok(Ok(n)) if n > 0 => {
            let response = String::from_utf8_lossy(&buffer[..n]);
            info!("Received handshake from {}: {}", peer_id, response.trim());

            // Send peer list
            let peer_list = state.get_peer_list().await;
            let peer_list_msg = format!("PEERS:{}\n", peer_list.join(","));
            if let Err(e) = socket.write_all(peer_list_msg.as_bytes()).await {
                warn!("Failed to send peer list to {}: {}", peer_id, e);
            }
        }
        Ok(Ok(_)) => {
            warn!("Empty handshake from {}", peer_id);
        }
        Ok(Err(e)) => {
            warn!("Failed to read handshake from {}: {}", peer_id, e);
        }
        Err(_) => {
            warn!("Handshake timeout from {}", peer_id);
        }
    }

    // Keep connection alive briefly
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;

    state.remove_peer(&peer_id).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "9001".to_string());
    let addr = format!("0.0.0.0:{}", port).parse::<SocketAddr>()?;

    info!("Starting Communitas Bootstrap Node on {}", addr);
    info!(
        "Environment: {}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    let listener = TcpListener::bind(addr).await?;
    let state = BootstrapState::new();

    // Start cleanup task
    let cleanup_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            cleanup_state.cleanup_stale_peers().await;
        }
    });

    info!("Bootstrap node ready. Waiting for connections...");

    loop {
        match listener.accept().await {
            Ok((socket, peer_addr)) => {
                info!("New connection from {}", peer_addr);
                let state_clone = state.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_peer_connection(socket, peer_addr, state_clone).await {
                        error!("Error handling peer connection: {}", e);
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
            }
        }
    }
}
