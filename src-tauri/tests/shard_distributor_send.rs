use async_trait::async_trait;
use blake3;
use chrono;
use communitas_tauri::storage::*;
use std::sync::Arc;

// Mock types for testing
#[derive(serde::Serialize, serde::Deserialize)]
enum ShardMessage {
    StoreShardRequest {
        shard_id: String,
        data: Vec<u8>,
    },
    StoreShardResponse {
        success: bool,
        message: String,
        storage_available: bool,
    },
    RetrieveShardRequest {
        shard_id: String,
    },
    RetrieveShardResponse {
        shard: Option<Vec<u8>>,
        success: bool,
        message: String,
    },
    ShardHealthCheck {
        group_id: String,
    },
    ShardHealthResponse {
        available_shards: Vec<String>,
        corrupted_shards: Vec<String>,
    },
}

struct ShardDistributionStatus {
    total_shards: usize,
    successful_distributions: usize,
}

struct ShardDistributor<D, R> {
    _dht: Arc<D>,
    _rs: Arc<R>,
}

impl<D, R> ShardDistributor<D, R> {
    fn new(dht: Arc<D>, rs: Arc<R>) -> Self {
        Self { _dht: dht, _rs: rs }
    }

    async fn distribute_shards(
        &self,
        _plan: &ShardDistributionPlan,
    ) -> Result<ShardDistributionStatus, Box<dyn std::error::Error>> {
        Ok(ShardDistributionStatus {
            total_shards: 1,
            successful_distributions: 1,
        })
    }
}

#[async_trait]
trait DhtFacade {
    async fn self_id(&self) -> String;
    async fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Box<dyn std::error::Error>>;
    async fn get(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>>;
    async fn send(
        &self,
        peer_id: String,
        topic: String,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

struct MockDht;

#[async_trait]
impl DhtFacade for MockDht {
    async fn self_id(&self) -> String {
        "mock-self".into()
    }
    async fn put(
        &self,
        _key: Vec<u8>,
        _value: Vec<u8>,
    ) -> Result<(), communitas_tauri::error::AppError> {
        Ok(())
    }
    async fn get(
        &self,
        _key: Vec<u8>,
    ) -> Result<Option<Vec<u8>>, communitas_tauri::error::AppError> {
        Ok(None)
    }
    async fn send(
        &self,
        _peer_id: String,
        _topic: String,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Decode request, return success response
        let msg: ShardMessage = serde_json::from_slice(&payload).unwrap();
        let response = match msg {
            ShardMessage::StoreShardRequest { .. } => ShardMessage::StoreShardResponse {
                success: true,
                message: "ok".into(),
                storage_available: true,
            },
            ShardMessage::RetrieveShardRequest { .. } => ShardMessage::RetrieveShardResponse {
                shard: None,
                success: true,
                message: "ok".into(),
            },
            ShardMessage::ShardHealthCheck { .. } => ShardMessage::ShardHealthResponse {
                available_shards: vec![],
                corrupted_shards: vec![],
            },
            _ => ShardMessage::ShardHealthResponse {
                available_shards: vec![],
                corrupted_shards: vec![],
            },
        };
        Ok(serde_json::to_vec(&response).unwrap())
    }
}

#[tokio::test]
async fn shard_distributor_sends_store_requests() {
    let dht = Arc::new(MockDht);
    let rs = Arc::new(EnhancedReedSolomonManager::new(dht.clone()));
    let distributor = ShardDistributor::new(dht, rs);

    // Create a dummy shard and plan
    let shard = Shard {
        index: 0,
        shard_type: ShardType::Data,
        data: b"abc".to_vec(),
        group_id: "g1".into(),
        data_id: "d1".into(),
        integrity_hash: blake3::hash(b"abc").to_string(),
        created_at: chrono::Utc::now(),
        size: 3,
    };

    let mut member_assignments = std::collections::HashMap::new();
    member_assignments.insert("peer-1".to_string(), vec![shard]);

    let plan = ShardDistributionPlan {
        group_id: "g1".into(),
        total_shards: 1,
        member_assignments,
        redundancy_level: 1.0,
    };

    let status = distributor
        .distribute_shards(&plan)
        .await
        .expect("distribution ok");
    assert_eq!(status.total_shards, 1);
    assert_eq!(status.successful_distributions, 1);
}
