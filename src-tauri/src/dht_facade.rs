use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::error::AppResult;

#[async_trait]
pub trait DhtFacade: Send + Sync {
    async fn self_id(&self) -> String;
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> AppResult<()>;
    async fn get(&self, key: Vec<u8>) -> AppResult<Option<Vec<u8>>>;
    async fn send(&self, peer_id: String, topic: String, payload: Vec<u8>) -> AppResult<Vec<u8>>;
    async fn peers(&self) -> AppResult<Vec<String>>;
}

/// Simple in-memory DHT facade for local dev and tests
#[derive(Debug)]
pub struct LocalDht {
    self_id: String,
    store: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    inboxes: Arc<RwLock<HashMap<String, Vec<Vec<u8>>>>>,
    peers: Arc<RwLock<Vec<String>>>,
}

impl LocalDht {
    pub fn new(self_id: String) -> Self {
        Self {
            self_id,
            store: Arc::new(RwLock::new(HashMap::new())),
            inboxes: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_peer(&self, peer_id: String) {
        let mut peers = self.peers.write().await;
        if !peers.contains(&peer_id) { peers.push(peer_id); }
    }

    /// Test helper: drain all messages for a peer
    pub async fn drain_inbox(&self, peer_id: &str) -> Vec<Vec<u8>> {
        let mut inboxes = self.inboxes.write().await;
        inboxes.remove(peer_id).unwrap_or_default()
    }
}

#[async_trait]
impl DhtFacade for LocalDht {
    async fn self_id(&self) -> String { self.self_id.clone() }

    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> AppResult<()> {
        let mut store = self.store.write().await;
        store.insert(key, value);
        Ok(())
    }

    async fn get(&self, key: Vec<u8>) -> AppResult<Option<Vec<u8>>> {
        let store = self.store.read().await;
        Ok(store.get(&key).cloned())
    }

    async fn send(&self, peer_id: String, _topic: String, payload: Vec<u8>) -> AppResult<Vec<u8>> {
        let mut inboxes = self.inboxes.write().await;
        let inbox = inboxes.entry(peer_id).or_default();
        inbox.push(payload.clone());
        Ok(Vec::new())
    }

    async fn peers(&self) -> AppResult<Vec<String>> {
        Ok(self.peers.read().await.clone())
    }
}
