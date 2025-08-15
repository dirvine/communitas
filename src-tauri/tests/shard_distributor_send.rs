use communitas_tauri::dht_facade::DhtFacade;
use communitas_tauri::storage::reed_solomon_manager::{EnhancedReedSolomonManager, Shard, ShardDistributionPlan, ShardType};
use communitas_tauri::storage::shard_distributor::{ShardDistributor, ShardMessage};
use async_trait::async_trait;
use std::sync::Arc;

struct MockDht;

#[async_trait]
impl DhtFacade for MockDht {
    async fn self_id(&self) -> String { "mock-self".into() }
    async fn put(&self, _key: Vec<u8>, _value: Vec<u8>) -> Result<(), communitas_tauri::error::AppError> { Ok(()) }
    async fn get(&self, _key: Vec<u8>) -> Result<Option<Vec<u8>>, communitas_tauri::error::AppError> { Ok(None) }
    async fn send(&self, _peer_id: String, _topic: String, payload: Vec<u8>) -> Result<Vec<u8>, communitas_tauri::error::AppError> {
        // Decode request, return success response
        let msg: ShardMessage = serde_json::from_slice(&payload).unwrap();
        let response = match msg {
            ShardMessage::StoreShardRequest { .. } => ShardMessage::StoreShardResponse { success: true, message: "ok".into(), storage_available: true },
            ShardMessage::RetrieveShardRequest { .. } => ShardMessage::RetrieveShardResponse { shard: None, success: true, message: "ok".into() },
            ShardMessage::ShardHealthCheck { .. } => ShardMessage::ShardHealthResponse { available_shards: vec![], corrupted_shards: vec![] },
            _ => ShardMessage::ShardHealthResponse { available_shards: vec![], corrupted_shards: vec![] },
        };
        Ok(serde_json::to_vec(&response).unwrap())
    }
    async fn peers(&self) -> Result<Vec<String>, communitas_tauri::error::AppError> { Ok(vec![]) }
}

#[tokio::test]
async fn shard_distributor_sends_store_requests() {
    let dht = Arc::new(MockDht);
    let rs = Arc::new(EnhancedReedSolomonManager::new());
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

    let status = distributor.distribute_shards(&plan).await.expect("distribution ok");
    assert_eq!(status.total_shards, 1);
    assert_eq!(status.successful_distributions, 1);
}
