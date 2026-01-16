use communitas_core::gossip::peer_cache::PeerCache;
use tempfile::tempdir;

#[tokio::test]
async fn test_peer_cache_bootstrap_roundtrip() {
    let temp_dir = tempdir().expect("tempdir");
    let cache_path = temp_dir.path().join("peer_cache.json");

    let mut cache = PeerCache::load(&cache_path).await.expect("load cache");

    let addr1: std::net::SocketAddr = "127.0.0.1:11000".parse().expect("addr1");
    let addr2: std::net::SocketAddr = "127.0.0.1:11001".parse().expect("addr2");

    cache
        .add_bootstrap_addr(addr1, true)
        .expect("add bootstrap addr1");
    cache
        .add_bootstrap_addr(addr2, false)
        .expect("add bootstrap addr2");

    let nodes = cache.get_bootstrap_nodes();
    assert_eq!(nodes.len(), 2);

    let cache_reloaded = PeerCache::load(&cache_path).await.expect("reload cache");
    let nodes_reloaded = cache_reloaded.get_bootstrap_nodes();
    assert_eq!(nodes_reloaded.len(), 2);
}

#[tokio::test]
async fn test_seed_bootstrap_nodes_marks_bootstrap() {
    let temp_dir = tempdir().expect("tempdir");
    let cache_path = temp_dir.path().join("peer_cache.json");

    let mut cache = PeerCache::load(&cache_path).await.expect("load cache");
    let seeds = vec![
        "ocean-forest-moon-star".to_string(),
        "river-mountain-cloud-light".to_string(),
    ];

    let seeded = cache
        .seed_bootstrap_nodes(&seeds)
        .await
        .expect("seed bootstrap nodes");
    assert_eq!(seeded, 2);
}
