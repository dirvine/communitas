# Network Integration Test Plan - Deep Analysis

**Date:** 2025-01-29  
**Goal:** Prove Sites protocol works over real QUIC network  
**Duration:** 1 day (8-10 hours)  
**Approach:** Comprehensive, real network, no mocks

---

## 🎯 CRITICAL TEST PHILOSOPHY

### What Makes a REAL Test

**NOT REAL:**
- ❌ Calling `handle_request()` directly (no network)
- ❌ Mocked transports
- ❌ Same-process communication
- ❌ Hardcoded localhost:5000

**REAL:**
- ✅ Two separate GossipContext instances
- ✅ QUIC over actual sockets (IPv4 AND IPv6)
- ✅ Random port allocation (parallel test safe)
- ✅ Real signature creation and verification
- ✅ Network timeouts and errors
- ✅ Process cleanup (no port leaks)

---

## 🏗️ TEST INFRASTRUCTURE

### Random Port Allocation

**Problem:** Hardcoded ports cause test conflicts

**Solution:**
```rust
use std::net::{TcpListener, SocketAddr};

/// Get a random available port
fn get_random_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Release immediately
    port
}

/// Create test node with unique ports
async fn create_test_node(name: &str) -> (GossipContext, u16, u16) {
    let gossip_port = get_random_port();
    let sites_port = get_random_port();
    
    let ctx = GossipContext::initialize(
        format!("{}-test-node-alpha", name),
        name.to_string(),
        "test-device".to_string(),
        Some(gossip_port), // Main gossip port
    )
    .await
    .expect("Failed to initialize node");
    
    // Sites port is gossip_port + 1 by our convention
    (ctx, gossip_port, sites_port)
}
```

---

### IPv4 AND IPv6 Testing

**Both protocols must work!**

```rust
#[tokio::test]
async fn test_sites_over_ipv4() {
    test_sites_over_ip("127.0.0.1").await;
}

#[tokio::test]
async fn test_sites_over_ipv6() {
    test_sites_over_ip("::1").await;
}

async fn test_sites_over_ip(ip: &str) {
    let port = get_random_port();
    let addr: SocketAddr = format!("{}:{}", ip, port).parse().unwrap();
    
    let ctx = GossipContext::initialize_with_addr(
        "test-node".into(),
        "Test".into(),
        "device".into(),
        Some(addr),
    ).await.unwrap();
    
    // Test publish + fetch
    // ...
}
```

---

### Proper Cleanup

**Problem:** Failed tests leave ports bound, transports hanging

**Solution:**
```rust
struct TestNode {
    ctx: GossipContext,
    gossip_port: u16,
    sites_port: u16,
}

impl Drop for TestNode {
    fn drop(&mut self) {
        // Shutdown transports
        // Stop listeners
        // Clean up resources
        tracing::info!("Cleaning up test node on ports {}/{}", 
            self.gossip_port, self.sites_port);
    }
}

#[tokio::test]
async fn test_with_cleanup() {
    let _node = TestNode::new("alice").await;
    // Test logic
    // Drop handles cleanup automatically
}
```

---

## 🧪 CRITICAL TEST SCENARIOS

### Test 1: Type Conversion & Signing

**Goal:** Prove we can sign manifests using Identity keys

```rust
#[tokio::test]
async fn test_identity_key_signs_manifest() {
    // Create GossipContext with Identity
    let ctx = GossipContext::initialize(
        "alice-bob-carol-dave".into(),
        "Alice".into(),
        "laptop".into(),
        None,
    ).await.unwrap();
    
    // Get ML-DSA keys via type conversion
    let (public_key, private_key) = ctx
        .get_sites_signing_keys()
        .expect("Should convert keys");
    
    // Create a manifest
    let site_id = SiteId::from_public_key(&public_key);
    let publisher = SitePublisher::new(site_id.clone());
    
    let hash = publisher
        .add_asset("test.html".into(), b"<html>Test</html>".to_vec())
        .await
        .unwrap();
    
    let mut manifest = publisher
        .build_manifest(&public_key, 1, vec![("test.html".into(), hash)])
        .await
        .unwrap();
    
    // Sign with converted private key
    manifest.sign(&private_key)
        .expect("Should sign with converted key");
    
    // Verify with public key
    manifest.verify()
        .expect("Signature should verify");
    
    // SUCCESS: Type conversion works!
}
```

**Validates:**
- ✅ get_sites_signing_keys() works
- ✅ Type conversion successful
- ✅ Can sign manifests
- ✅ Signatures verify

---

### Test 2: Two-Node QUIC Communication

**Goal:** Prove QUIC actually transmits messages between nodes

```rust
#[tokio::test]
async fn test_two_nodes_quic_publish_and_fetch() {
    // Setup: Two independent nodes
    let (node_a, port_a_gossip, port_a_sites) = create_test_node("alice").await;
    let (node_b, port_b_gossip, port_b_sites) = create_test_node("bob").await;
    
    tracing::info!("Node A: gossip={}, sites={}", port_a_gossip, port_a_sites);
    tracing::info!("Node B: gossip={}, sites={}", port_b_gossip, port_b_sites);
    
    // === PUBLISH ON NODE A ===
    
    let publisher = node_a.site_publisher.as_ref().unwrap();
    
    // Get signing keys from Node A's identity
    let (public_key, private_key) = node_a
        .get_sites_signing_keys()
        .expect("Node A: Failed to get signing keys");
    
    let site_id = SiteId::from_public_key(&public_key);
    
    // Create content
    let html = b"<html><body><h1>Hello from Node A!</h1></body></html>".to_vec();
    let hash = publisher
        .add_asset("index.html".into(), html.clone())
        .await
        .expect("Node A: Failed to add asset");
    
    // Build manifest
    let mut manifest = publisher
        .build_manifest(&public_key, 1, vec![("index.html".into(), hash)])
        .await
        .expect("Node A: Failed to build manifest");
    
    // CRITICAL: Sign the manifest
    manifest.sign(&private_key)
        .expect("Node A: Failed to sign manifest");
    
    // Verify signature locally first
    manifest.verify()
        .expect("Node A: Manifest signature should verify locally");
    
    // Store signed manifest
    publisher.set_manifest(manifest.clone()).await
        .expect("Node A: Failed to store manifest");
    
    tracing::info!("Node A: Published site_id={:?}", site_id);
    
    // === ADVERTISE VIA PROVIDER SUMMARY ===
    
    // TODO: Implement this!
    // For now, Node B will connect directly using peer_id
    
    // === FETCH ON NODE B ===
    
    let fetcher = node_b.site_fetcher.as_ref().unwrap();
    
    // Give nodes time to establish connection
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Fetch manifest from Node A
    // Note: Using node_a.peer_id assumes they can connect
    let fetched_manifest = fetcher
        .fetch_manifest(&site_id, node_a.peer_id())
        .await
        .expect("Node B: Failed to fetch manifest");
    
    // CRITICAL: Verify signature over the network
    fetched_manifest.verify()
        .expect("Node B: Manifest signature should verify after fetch");
    
    // Verify content matches
    assert_eq!(fetched_manifest.site_id, site_id);
    assert_eq!(fetched_manifest.manifest_version, 1);
    assert_eq!(fetched_manifest.blocks.len(), 1);
    
    // Fetch the block
    let (path, block_hash) = &fetched_manifest.blocks[0];
    assert_eq!(path, "index.html");
    
    let fetched_block = fetcher
        .fetch_block(block_hash, node_a.peer_id())
        .await
        .expect("Node B: Failed to fetch block");
    
    // CRITICAL: Verify block hash
    assert!(fetched_block.verify(), "Block hash should verify");
    assert_eq!(fetched_block.content, html);
    
    tracing::info!("SUCCESS: Two-node publish and fetch works!");
}
```

**Validates:**
- ✅ QUIC communication works
- ✅ Manifests transmit correctly
- ✅ Signatures verify over network
- ✅ Blocks transmit correctly
- ✅ Hash verification works
- ✅ End-to-end flow functional

---

### Test 3: Provider Advertisement & Discovery

**Goal:** Prove rendezvous-based discovery works

```rust
#[tokio::test]
async fn test_provider_advertisement_and_discovery() {
    let (node_a, _, _) = create_test_node("publisher").await;
    let (node_b, _, _) = create_test_node("fetcher").await;
    
    // === NODE A: PUBLISH AND ADVERTISE ===
    
    let publisher = node_a.site_publisher.as_ref().unwrap();
    let (pk, sk) = node_a.get_sites_signing_keys().unwrap();
    let site_id = SiteId::from_public_key(&pk);
    
    // Publish content (same as Test 2)
    let hash = publisher.add_asset("test.txt".into(), b"Hello".to_vec()).await.unwrap();
    let mut manifest = publisher.build_manifest(&pk, 1, vec![("test.txt".into(), hash)]).await.unwrap();
    manifest.sign(&sk).unwrap();
    publisher.set_manifest(manifest.clone()).await.unwrap();
    
    // CRITICAL: Advertise as provider via ProviderSummary
    let rendezvous = &node_a.rendezvous;
    
    let mut provider_summary = ProviderSummary::new(
        site_id.hash,                // target
        node_a.peer_id(),            // provider
        vec![Capability::Site],      // capabilities
        3_600_000,                   // 1 hour validity
    )
    .with_manifest_version(1)
    .with_root(true);
    
    // Sign the ProviderSummary with Identity's keypair
    let identity_kp = node_a.identity().key_pair();
    provider_summary.sign(identity_kp)
        .expect("Should sign ProviderSummary");
    
    // Publish to rendezvous shard
    rendezvous.publish_provider_summary(provider_summary)
        .await
        .expect("Should publish to rendezvous");
    
    tracing::info!("Node A: Advertised as provider for site_id={:?}", site_id);
    
    // === NODE B: DISCOVER VIA RENDEZVOUS ===
    
    let fetcher = node_b.site_fetcher.as_ref().unwrap();
    
    // Subscribe to site's rendezvous shard
    fetcher.start_discovery(&site_id)
        .await
        .expect("Should start discovery");
    
    // Wait for provider advertisements to propagate
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Get providers from rendezvous
    let providers = fetcher.get_providers(&site_id).await;
    
    // CRITICAL: Should find Node A as provider
    assert!(!providers.is_empty(), "Should discover at least one provider");
    assert_eq!(providers[0].provider, node_a.peer_id(), "Should discover Node A");
    assert!(providers[0].have_root, "Provider should have root");
    assert_eq!(providers[0].manifest_ver, Some(1), "Manifest version should be 1");
    
    // Verify the provider summary signature
    let verified = providers[0].verify(identity_kp.public_key())
        .expect("Verification should succeed");
    assert!(verified, "Provider summary signature should be valid");
    
    tracing::info!("SUCCESS: Discovery via rendezvous works!");
}
```

**Validates:**
- ✅ ProviderSummary creation works
- ✅ Rendezvous shard routing works
- ✅ Provider discovery works
- ✅ Signature on ProviderSummary works
- ✅ Manifest version propagates

---

### Test 4: IPv6 Communication

**Goal:** Ensure IPv6 works (not just IPv4)

```rust
#[tokio::test]
async fn test_sites_over_ipv6() {
    // Force IPv6 addresses
    let ipv6_loopback = "::1";
    
    let port_a = get_random_port();
    let addr_a: SocketAddr = format!("{}:{}", ipv6_loopback, port_a).parse().unwrap();
    
    let port_b = get_random_port();
    let addr_b: SocketAddr = format!("{}:{}", ipv6_loopback, port_b).parse().unwrap();
    
    // Initialize with IPv6 addresses
    let node_a = GossipContext::initialize_with_bind_addr(
        "alice-ipv6-test-node".into(),
        "Alice".into(),
        "laptop".into(),
        addr_a,
    ).await.expect("Should initialize Node A on IPv6");
    
    let node_b = GossipContext::initialize_with_bind_addr(
        "bob-ipv6-test-node".into(),
        "Bob".into(),
        "desktop".into(),
        addr_b,
    ).await.expect("Should initialize Node B on IPv6");
    
    // Verify we're actually using IPv6
    assert!(addr_a.is_ipv6(), "Node A should be IPv6");
    assert!(addr_b.is_ipv6(), "Node B should be IPv6");
    
    // Run same publish/fetch test over IPv6
    // ... (same logic as Test 2)
    
    tracing::info!("SUCCESS: IPv6 communication works!");
}
```

**Validates:**
- ✅ IPv6 socket binding works
- ✅ QUIC over IPv6 works
- ✅ ant-quic handles both IP versions

---

### Test 5: Signature Verification Rejects Invalid Content

**Goal:** Prove security actually works

```rust
#[tokio::test]
async fn test_reject_unsigned_manifest() {
    let (node_a, _, _) = create_test_node("publisher").await;
    let (node_b, _, _) = create_test_node("fetcher").await;
    
    // Node A publishes UNSIGNED manifest (security hole!)
    let publisher = node_a.site_publisher.as_ref().unwrap();
    let (pk, _sk) = node_a.get_sites_signing_keys().unwrap();
    let site_id = SiteId::from_public_key(&pk);
    
    let hash = publisher.add_asset("bad.html".into(), b"<html>Unsigned</html>".to_vec()).await.unwrap();
    let manifest = publisher.build_manifest(&pk, 1, vec![("bad.html".into(), hash)]).await.unwrap();
    
    // DON'T SIGN IT!
    // manifest.sign(&sk) ← Skipped intentionally
    
    publisher.set_manifest(manifest.clone()).await.unwrap();
    
    // Node B tries to fetch
    let fetcher = node_b.site_fetcher.as_ref().unwrap();
    
    let result = fetcher.fetch_manifest(&site_id, node_a.peer_id()).await;
    
    // CRITICAL: Should REJECT unsigned manifest
    assert!(result.is_err(), "Should reject unsigned manifest");
    assert!(result.unwrap_err().to_string().contains("signature"), 
        "Error should mention signature verification");
    
    tracing::info!("SUCCESS: Unsigned manifests rejected!");
}

#[tokio::test]
async fn test_reject_tampered_manifest() {
    let (node_a, _, _) = create_test_node("attacker").await;
    let (node_b, _, _) = create_test_node("victim").await;
    
    // Node A creates valid manifest
    let publisher = node_a.site_publisher.as_ref().unwrap();
    let (pk, sk) = node_a.get_sites_signing_keys().unwrap();
    let site_id = SiteId::from_public_key(&pk);
    
    let hash = publisher.add_asset("real.html".into(), b"Real content".to_vec()).await.unwrap();
    let mut manifest = publisher.build_manifest(&pk, 1, vec![("real.html".into(), hash)]).await.unwrap();
    manifest.sign(&sk).unwrap();
    
    // TAMPER with manifest after signing
    manifest.manifest_version = 999; // ← Tampering!
    
    publisher.set_manifest(manifest).await.unwrap();
    
    // Node B fetches
    let fetcher = node_b.site_fetcher.as_ref().unwrap();
    let result = fetcher.fetch_manifest(&site_id, node_a.peer_id()).await;
    
    // CRITICAL: Should REJECT tampered manifest
    assert!(result.is_err(), "Should reject tampered manifest");
    
    tracing::info!("SUCCESS: Tampered manifests rejected!");
}

#[tokio::test]
async fn test_reject_wrong_site_id() {
    let (node_a, _, _) = create_test_node("publisher").await;
    let (node_b, _, _) = create_test_node("fetcher").await;
    
    // Node A publishes site X
    let (pk_a, sk_a) = node_a.get_sites_signing_keys().unwrap();
    let site_id_a = SiteId::from_public_key(&pk_a);
    
    let publisher = node_a.site_publisher.as_ref().unwrap();
    let hash = publisher.add_asset("x.html".into(), b"Site X".to_vec()).await.unwrap();
    let mut manifest = publisher.build_manifest(&pk_a, 1, vec![("x.html".into(), hash)]).await.unwrap();
    manifest.sign(&sk_a).unwrap();
    publisher.set_manifest(manifest).await.unwrap();
    
    // Node B requests site Y (different SiteId)
    let (pk_y, _) = generate_test_keypair(999);
    let site_id_y = SiteId::from_public_key(&pk_y);
    
    let fetcher = node_b.site_fetcher.as_ref().unwrap();
    let result = fetcher.fetch_manifest(&site_id_y, node_a.peer_id()).await;
    
    // CRITICAL: Should reject (site ID mismatch)
    // fetch_manifest checks: manifest.site_id == requested site_id
    assert!(result.is_err(), "Should reject wrong site_id");
    
    tracing::info!("SUCCESS: Site ID validation works!");
}
```

**Validates:**
- ✅ Unsigned manifests rejected
- ✅ Tampered manifests rejected
- ✅ Site ID mismatches caught
- ✅ Security checks enforce

---

### Test 6: Concurrent Fetches (Race Conditions)

**Goal:** Prove thread-safety and concurrent access

```rust
#[tokio::test]
async fn test_concurrent_fetches_from_multiple_nodes() {
    let (publisher_node, _, _) = create_test_node("publisher").await;
    
    // Publish one site
    let (pk, sk) = publisher_node.get_sites_signing_keys().unwrap();
    let site_id = SiteId::from_public_key(&pk);
    
    let publisher = publisher_node.site_publisher.as_ref().unwrap();
    let hash = publisher.add_asset("popular.html".into(), b"Popular!".to_vec()).await.unwrap();
    let mut manifest = publisher.build_manifest(&pk, 1, vec![("popular.html".into(), hash)]).await.unwrap();
    manifest.sign(&sk).unwrap();
    publisher.set_manifest(manifest).await.unwrap();
    
    // Spawn 10 fetcher nodes that all try to fetch concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let site_id_clone = site_id.clone();
        let provider_peer_id = publisher_node.peer_id();
        
        let handle = tokio::spawn(async move {
            let (node, _, _) = create_test_node(&format!("fetcher-{}", i)).await;
            let fetcher = node.site_fetcher.as_ref().unwrap();
            
            // All 10 fetch simultaneously
            let result = fetcher.fetch_manifest(&site_id_clone, provider_peer_id).await;
            
            result.expect(&format!("Fetcher {} should succeed", i))
        });
        
        handles.push(handle);
    }
    
    // Wait for all fetches
    for handle in handles {
        let manifest = handle.await.unwrap();
        manifest.verify().expect("All fetches should verify");
    }
    
    tracing::info!("SUCCESS: 10 concurrent fetches all succeeded!");
}
```

**Validates:**
- ✅ Backpressure works (Semaphore limit: 10)
- ✅ No race conditions
- ✅ Thread-safe caching
- ✅ QUIC handles concurrent connections

---

### Test 7: Network Timeouts & Error Handling

**Goal:** Graceful failure when things go wrong

```rust
#[tokio::test]
async fn test_fetch_from_offline_provider() {
    let (node, _, _) = create_test_node("fetcher").await;
    let fetcher = node.site_fetcher.as_ref().unwrap();
    
    // Try to fetch from non-existent provider
    let fake_site_id = SiteId::new([42u8; 32]);
    let fake_peer_id = PeerId::new([99u8; 32]);
    
    let result = fetcher.fetch_manifest(&fake_site_id, fake_peer_id).await;
    
    // Should timeout/error, not hang forever
    assert!(result.is_err(), "Should fail gracefully");
    
    tracing::info!("SUCCESS: Timeouts work correctly!");
}

#[tokio::test]
async fn test_fetch_non_existent_site() {
    let (node_a, _, _) = create_test_node("empty-publisher").await;
    let (node_b, _, _) = create_test_node("fetcher").await;
    
    // Node A has NO published sites
    let fake_site_id = SiteId::new([1u8; 32]);
    
    let fetcher = node_b.site_fetcher.as_ref().unwrap();
    let result = fetcher.fetch_manifest(&fake_site_id, node_a.peer_id()).await;
    
    // Should get clear error response
    assert!(result.is_err(), "Should fail for non-existent site");
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("not found") || error_msg.contains("No manifest"),
        "Error should be clear: {}", error_msg);
    
    tracing::info!("SUCCESS: Clear errors for missing content!");
}
```

**Validates:**
- ✅ Timeouts prevent hanging
- ✅ Error messages are clear
- ✅ Graceful degradation
- ✅ No panics on errors

---

## 🔧 IMPLEMENTATION TASKS

### Task 1: Type Conversion Helper (1 hour)

**File:** `communitas-core/src/gossip/context.rs`

```rust
pub fn get_sites_signing_keys(&self) -> Result<(
    saorsa_pqc::ml_dsa_87::PublicKey,
    saorsa_pqc::ml_dsa_87::PrivateKey,
)> {
    use fips204::traits::SerDes;
    
    let kp = self.identity.key_pair();
    
    // Get public key bytes
    let pub_bytes = kp.public_key();
    let pk_array: [u8; 2592] = pub_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Public key wrong size"))?;
    let public_key = saorsa_pqc::ml_dsa_87::PublicKey::try_from_bytes(pk_array)?;
    
    // Get secret key using typed method
    let secret_key = kp.get_secret_key_typed()?;
    
    // Convert MlDsaSecretKey to ml_dsa_87::PrivateKey if needed
    // (Check if they're the same type or need conversion)
    
    Ok((public_key, secret_key))
}
```

---

### Task 2: Provider Advertisement (2 hours)

**File:** `communitas-core/src/gossip/sites.rs`

```rust
impl SitePublisher {
    /// Advertise this site to rendezvous network
    pub async fn advertise_to_rendezvous(
        &self,
        rendezvous: &super::rendezvous::RendezvousClient,
        peer_id: PeerId,
        identity_keypair: &saorsa_gossip_identity::MlDsaKeyPair,
        manifest_version: u64,
    ) -> Result<()> {
        use saorsa_gossip_rendezvous::{Capability, ProviderSummary};
        
        let mut summary = ProviderSummary::new(
            self.site_id.hash,           // target (BLAKE3 of public key)
            peer_id,                     // this node
            vec![Capability::Site],      // we serve sites
            3_600_000,                   // valid for 1 hour
        )
        .with_manifest_version(manifest_version)
        .with_root(true);
        
        // Sign with identity keypair
        summary.sign(identity_keypair)
            .map_err(|e| anyhow::anyhow!("Failed to sign ProviderSummary: {}", e))?;
        
        // Publish to rendezvous shard
        rendezvous.publish_provider_summary(summary)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to publish to rendezvous: {}", e))?;
        
        tracing::info!("Advertised as provider for site {:?}", self.site_id);
        
        Ok(())
    }
}
```

---

### Task 3: Test Infrastructure (2 hours)

**File:** `communitas-core/tests/network_test_utils.rs`

```rust
use communitas_core::gossip::GossipContext;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Get random available port for testing
pub fn get_random_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .unwrap()
        .port()
}

/// Create test node with random ports
pub async fn create_test_node(name: &str) -> TestNode {
    let port = get_random_port();
    
    let ctx = GossipContext::initialize(
        format!("{}-{}-test-node", name, port), // Unique four-words
        name.to_string(),
        "test-device".to_string(),
        Some(port),
    )
    .await
    .expect("Failed to initialize test node");
    
    TestNode {
        name: name.to_string(),
        ctx: Arc::new(ctx),
        gossip_port: port,
        sites_port: port + 1,
    }
}

/// Test node wrapper with cleanup
pub struct TestNode {
    pub name: String,
    pub ctx: Arc<GossipContext>,
    pub gossip_port: u16,
    pub sites_port: u16,
}

impl TestNode {
    pub fn peer_id(&self) -> saorsa_gossip_types::PeerId {
        self.ctx.peer_id()
    }
    
    pub async fn publish_site(
        &self,
        content: Vec<(String, Vec<u8>)>, // path, content
    ) -> Result<(SiteId, SiteManifest), String> {
        let publisher = self.ctx.site_publisher.as_ref()
            .ok_or("No publisher")?;
        
        let (pk, sk) = self.ctx.get_sites_signing_keys()
            .map_err(|e| e.to_string())?;
        
        let site_id = SiteId::from_public_key(&pk);
        
        // Add assets
        let mut asset_paths = vec![];
        for (path, data) in content {
            let hash = publisher.add_asset(path.clone(), data).await
                .map_err(|e| e.to_string())?;
            asset_paths.push((path, hash));
        }
        
        // Build and sign manifest
        let mut manifest = publisher.build_manifest(&pk, 1, asset_paths).await
            .map_err(|e| e.to_string())?;
        manifest.sign(&sk).map_err(|e| e.to_string())?;
        
        // Store
        publisher.set_manifest(manifest.clone()).await
            .map_err(|e| e.to_string())?;
        
        Ok((site_id, manifest))
    }
}

impl Drop for TestNode {
    fn drop(&mut self) {
        tracing::info!("Cleaning up test node '{}' (ports {}/{})", 
            self.name, self.gossip_port, self.sites_port);
        // Cleanup logic
    }
}
```

---

## 📊 TEST EXECUTION PLAN

### Morning Session (4 hours)

**9:00-10:00: Type Conversion**
- Implement `get_sites_signing_keys()`
- Test type conversion works
- Verify signing succeeds

**10:00-12:00: Provider Advertisement**
- Implement `advertise_to_rendezvous()`
- Test ProviderSummary creation
- Test signature on summary

**12:00-13:00: Test Infrastructure**
- Create `network_test_utils.rs`
- Random port allocation
- TestNode helper
- Cleanup logic

### Afternoon Session (4 hours)

**14:00-15:30: Write Core Tests**
- Test 1: Type conversion & signing
- Test 2: Two-node QUIC
- Test 3: Provider advertisement

**15:30-17:00: Run & Debug Tests**
- Run tests
- They WILL fail (expected!)
- Debug issues
- Fix bugs

**17:00-18:00: Security Tests**
- Test invalid signatures
- Test tampering
- Test Site ID validation

---

## 🎯 SUCCESS CRITERIA

### By End of Day

**Must Pass:**
- [ ] Type conversion test passes
- [ ] Two-node QUIC test passes
- [ ] Provider advertisement test passes
- [ ] IPv6 test passes
- [ ] Security rejection tests pass

**Can Defer:**
- Multi-provider failover
- Cache integration
- Advanced error scenarios

### What Success Looks Like

```
Running network integration tests...

test test_identity_key_signs_manifest ... ok
test test_two_nodes_quic_publish_and_fetch ... ok
test test_provider_advertisement_and_discovery ... ok
test test_sites_over_ipv6 ... ok
test test_reject_unsigned_manifest ... ok
test test_reject_tampered_manifest ... ok
test test_reject_wrong_site_id ... ok

test result: ok. 7 passed; 0 failed
```

**This proves the system works!**

---

## ⚠️ EXPECTED CHALLENGES

### Challenge 1: get_secret_key_typed() Return Type

**Issue:** Might return `MlDsaSecretKey`, not `ml_dsa_87::PrivateKey`

**Solution:**
- Check actual return type
- Convert if needed
- Both are saorsa-pqc types, should be compatible

---

### Challenge 2: Nodes Can't Connect

**Issue:** QUIC connection might fail

**Debug:**
- Check ports are actually bound
- Check firewall/network settings
- Try IPv4 AND IPv6
- Check ant-quic logs

---

### Challenge 3: Provider Discovery Fails

**Issue:** Rendezvous routing might not work

**Debug:**
- Check shard calculation
- Check pubsub subscription
- Check ProviderSummary serialization
- Add extensive logging

---

### Challenge 4: Signatures Fail Over Network

**Issue:** Serialization might corrupt signatures

**Debug:**
- Compare signature bytes before/after network
- Check CBOR vs bincode serialization
- Verify public key bytes match

---

## 📝 CODE STRUCTURE

### New Files to Create

```
communitas-core/
├── tests/
│   ├── network_test_utils.rs         (NEW - test infrastructure)
│   ├── sites_network_e2e_test.rs     (NEW - 2-node tests)
│   ├── sites_security_test.rs        (NEW - security tests)
│   ├── sites_ipv6_test.rs            (NEW - IPv6 tests)
│   └── sites_discovery_test.rs       (NEW - rendezvous tests)
└── src/
    └── gossip/
        ├── context.rs                 (MODIFY - add get_sites_signing_keys)
        └── sites.rs                   (MODIFY - add advertise_to_rendezvous)
```

---

## 🎯 TOMORROW'S DELIVERABLES

1. ✅ Type conversion working
2. ✅ Provider advertisement working  
3. ✅ 7+ network integration tests passing
4. ✅ IPv4 and IPv6 validated
5. ✅ Security verified over network
6. ✅ Backend proven bulletproof

**Then:** Fix Tauri commands and build UI with confidence!

---

## 💡 DEEP INSIGHT

**Today we built the algorithms.**  
**Tomorrow we prove they work together.**  
**Next week we make them usable.**

This is the right order!

---

**Session Status:** ✅ Excellent Progress  
**Next Session:** Network Integration Testing  
**Timeline:** On track for 3-week MVP  
**Confidence:** Very High 🚀
