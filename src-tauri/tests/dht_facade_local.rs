use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Mock LocalDht implementation for testing
#[derive(Clone)]
struct LocalDht {
    node_id: String,
    storage: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    peers: Arc<RwLock<HashMap<String, Vec<Vec<u8>>>>>,
}

impl LocalDht {
    fn new(node_id: String) -> Self {
        Self {
            node_id,
            storage: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_peer(&self, peer_id: String) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id, Vec::new());
    }

    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let mut storage = self.storage.write().await;
        storage.insert(key, value);
        Ok(())
    }

    async fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let storage = self.storage.read().await;
        Ok(storage.get(&key).cloned())
    }

    async fn send(
        &self,
        peer_id: String,
        _topic: String,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut peers = self.peers.write().await;
        if let Some(inbox) = peers.get_mut(&peer_id) {
            inbox.push(payload);
        }
        Ok(vec![])
    }

    async fn drain_inbox(&self, peer_id: &str) -> Vec<Vec<u8>> {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id).unwrap_or_default()
    }
}

#[tokio::test]
async fn local_dht_put_get_send() {
    let dht = LocalDht::new("node-a".to_string());
    dht.add_peer("node-b".to_string()).await;

    // put/get
    dht.put(b"k".to_vec(), b"v".to_vec()).await.expect("put ok");
    let v = dht.get(b"k".to_vec()).await.expect("get ok");
    assert_eq!(v, Some(b"v".to_vec()));

    // send
    let resp = dht
        .send("node-b".to_string(), "topic".into(), b"payload".to_vec())
        .await
        .expect("send ok");
    assert!(resp.is_empty());

    // drain inbox (test helper) to assert message arrived
    let inbox = dht.drain_inbox("node-b").await;
    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0], b"payload".to_vec());
}
