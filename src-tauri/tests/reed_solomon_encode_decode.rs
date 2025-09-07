use communitas_tauri::storage::*;
use saorsa_seal::Dht;

// Mock DHT implementation for testing
struct MockDht;

impl Dht for MockDht {
    fn put(&self, key: &[u8; 32], value: &[u8], ttl: Option<u64>) -> anyhow::Result<()> {
        Ok(())
    }

    fn get(&self, key: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
}

impl communitas_tauri::storage::reed_solomon_manager::DhtStorage for MockDht {
    fn put(&self, key: &[u8; 32], value: &[u8], ttl: Option<u64>) -> anyhow::Result<()> {
        Ok(())
    }

    fn get(&self, key: &[u8; 32]) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
}

#[tokio::test]
async fn encode_decode_round_trip_with_missing_parity() {
    let dht = MockDht;
    let mgr = EnhancedReedSolomonManager::new(dht);
    let group_id = "test-group";
    let data_id = "file-1";
    let input: Vec<u8> = (0..8192).map(|i| (i % 251) as u8).collect();

    // Encode with an assumed group size
    let shards = mgr
        .encode_group_data(group_id, data_id, &input, 8)
        .await
        .expect("encode ok");

    // Drop up to all parity shards and keep data shards only
    let available: Vec<_> = shards
        .into_iter()
        .filter(|s| s.shard_type == ShardType::Data)
        .collect();

    let output = mgr
        .decode_group_data(group_id, data_id, &available)
        .await
        .expect("decode ok");

    assert_eq!(input, output);
}
