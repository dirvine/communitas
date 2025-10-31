# Sites Protocol Implementation Plan

**Date:** 2025-01-29  
**Status:** Design Document  
**Goal:** Wire QUIC protocol loop for DNS-free website serving

---

## Current State Analysis

### ✅ What's Already Implemented

1. **Client Side (SiteFetcher)**
   - `fetch_block()` - sends GetBlock request on Bulk stream
   - `fetch_manifest()` - sends GetManifest request on Bulk stream
   - Uses `transport.send_to_peer(provider, StreamType::Bulk, ...)`
   - Receives responses via `transport.receive_message()`
   - Cache implementation (in-memory HashMap)

2. **Server Side (SitePublisher)**
   - `handle_request(Bytes) -> Bytes` - transport-agnostic request handler
   - Processes GetManifest and GetBlock requests
   - Returns serialized responses

3. **Discovery**
   - Rendezvous shard-based provider discovery
   - ProviderSummary advertisements

### ❌ What's Missing

1. **No Server Listener**
   - Nothing listens for incoming Bulk stream requests
   - No routing from transport to SitePublisher.handle_request()

2. **No Concurrent Block Fetching**
   - Sequential block fetches (slow for large sites)
   - Should fetch 4-8 blocks in parallel

3. **No Timeouts/Retries**
   - No request timeouts
   - No automatic failover to next provider

4. **No Persistent Cache**
   - In-memory only (data lost on restart)
   - No LRU eviction
   - No pinning for owned sites

5. **No Backpressure**
   - Can overwhelm providers with requests
   - No rate limiting

---

## Implementation Plan

### Phase 1: Server-Side Listener (This Session)

**Goal:** Route incoming Bulk stream requests to SitePublisher

**Approach:**
Add a background task to GossipContext that listens for Sites requests.

**File:** `communitas-core/src/gossip/sites_listener.rs` (new)

```rust
//! Sites Protocol Listener
//!
//! Listens for incoming Sites requests on Bulk streams and routes
//! them to the appropriate SitePublisher.

use anyhow::Result;
use bytes::Bytes;
use saorsa_gossip_transport::{GossipTransport, StreamType};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

use super::sites::{SitePublisher, SiteRequest, SiteResponse};

/// Timeout for processing a single request
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum concurrent requests per publisher
const MAX_CONCURRENT_REQUESTS: usize = 10;

/// Sites protocol listener
pub struct SitesListener {
    /// Transport for receiving requests
    transport: Arc<RwLock<Box<dyn GossipTransport>>>,
    
    /// Site publisher (if we're publishing)
    publisher: Option<Arc<SitePublisher>>,
    
    /// Active request count (for backpressure)
    active_requests: Arc<tokio::sync::Semaphore>,
}

impl SitesListener {
    pub fn new(
        transport: Arc<RwLock<Box<dyn GossipTransport>>>,
        publisher: Option<Arc<SitePublisher>>,
    ) -> Self {
        Self {
            transport,
            publisher,
            active_requests: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_REQUESTS)),
        }
    }
    
    /// Start listening for Sites requests (runs in background)
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                if let Err(e) = self.handle_next_request().await {
                    tracing::warn!("Sites listener error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        })
    }
    
    /// Handle next incoming request
    async fn handle_next_request(&self) -> Result<()> {
        // Wait for incoming message
        let (peer_id, stream_type, request_bytes) = self
            .transport
            .read()
            .await
            .receive_message()
            .await?;
        
        // Only handle Bulk stream (Sites protocol)
        if stream_type != StreamType::Bulk {
            return Ok(()); // Ignore non-Bulk streams
        }
        
        // Check if this is a Sites request (try to deserialize)
        let request: SiteRequest = match bincode::deserialize(&request_bytes) {
            Ok(req) => req,
            Err(_) => return Ok(()), // Not a Sites request, ignore
        };
        
        // Acquire semaphore permit (backpressure)
        let permit = self.active_requests.clone().acquire_owned().await?;
        
        // Spawn task to handle request
        let publisher = self.publisher.clone();
        let transport = self.transport.clone();
        tokio::spawn(async move {
            let _permit = permit; // Hold permit until done
            
            if let Some(pub) = publisher {
                match timeout(REQUEST_TIMEOUT, pub.handle_request(request_bytes)).await {
                    Ok(Ok(response_bytes)) => {
                        // Send response
                        if let Err(e) = transport
                            .read()
                            .await
                            .send_to_peer(peer_id, StreamType::Bulk, response_bytes)
                            .await
                        {
                            tracing::warn!("Failed to send Sites response: {}", e);
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::warn!("Request processing failed: {}", e);
                        // Send error response
                        let error_response = SiteResponse::Error(e.to_string());
                        if let Ok(error_bytes) = bincode::serialize(&error_response) {
                            let _ = transport
                                .read()
                                .await
                                .send_to_peer(peer_id, StreamType::Bulk, Bytes::from(error_bytes))
                                .await;
                        }
                    }
                    Err(_) => {
                        tracing::warn!("Request timeout");
                        let error_response = SiteResponse::Error("Request timeout".to_string());
                        if let Ok(error_bytes) = bincode::serialize(&error_response) {
                            let _ = transport
                                .read()
                                .await
                                .send_to_peer(peer_id, StreamType::Bulk, Bytes::from(error_bytes))
                                .await;
                        }
                    }
                }
            } else {
                // Not publishing, send error
                let error_response = SiteResponse::Error("Not publishing sites".to_string());
                if let Ok(error_bytes) = bincode::serialize(&error_response) {
                    let _ = transport
                        .read()
                        .await
                        .send_to_peer(peer_id, StreamType::Bulk, Bytes::from(error_bytes))
                        .await;
                }
            }
        });
        
        Ok(())
    }
}
```

**Integration into GossipContext:**

```rust
// In communitas-core/src/gossip/context.rs

pub struct GossipContext {
    // ... existing fields ...
    
    /// Sites protocol listener
    pub sites_listener: Option<Arc<super::sites_listener::SitesListener>>,
    
    /// Sites listener task handle
    sites_listener_handle: Option<tokio::task::JoinHandle<()>>,
}

impl GossipContext {
    pub async fn initialize(...) -> Result<Self> {
        // ... existing initialization ...
        
        // Start Sites listener if we have a publisher
        let sites_listener = if site_publisher.is_some() {
            let listener = Arc::new(SitesListener::new(
                transport.clone(),
                site_publisher.clone(),
            ));
            let handle = listener.clone().start();
            
            Some((listener, handle))
        } else {
            None
        };
        
        Ok(Self {
            // ... existing fields ...
            sites_listener: sites_listener.as_ref().map(|(l, _)| l.clone()),
            sites_listener_handle: sites_listener.map(|(_, h)| h),
        })
    }
}
```

---

### Phase 2: Concurrent Block Fetching (Next)

**Goal:** Fetch multiple blocks in parallel for faster site loading

**File:** `communitas-core/src/gossip/sites.rs`

```rust
impl SiteFetcher {
    /// Fetch multiple blocks concurrently
    pub async fn fetch_blocks_concurrent(
        &self,
        hashes: &[[u8; 32]],
        provider: PeerId,
        concurrency: usize,
    ) -> Result<Vec<Block>> {
        use futures::stream::{self, StreamExt};
        
        let results: Vec<_> = stream::iter(hashes)
            .map(|hash| async move {
                self.fetch_block(hash, provider).await
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;
        
        // Check for errors
        let mut blocks = Vec::new();
        for result in results {
            blocks.push(result?);
        }
        
        Ok(blocks)
    }
    
    /// Fetch complete site (manifest + all blocks)
    pub async fn fetch_site_complete(
        &self,
        site_id: &SiteId,
        concurrency: usize,
    ) -> Result<(SiteManifest, Vec<Block>)> {
        // 1. Discover providers
        let providers = self.get_providers(site_id).await;
        if providers.is_empty() {
            anyhow::bail!("No providers found for site");
        }
        
        // 2. Try providers in order until success
        let mut last_error = None;
        for provider_summary in providers {
            let provider = provider_summary.peer_id;
            
            // Fetch manifest
            let manifest = match self.fetch_manifest(site_id, provider).await {
                Ok(m) => m,
                Err(e) => {
                    last_error = Some(e);
                    continue; // Try next provider
                }
            };
            
            // Verify signature
            if let Err(e) = manifest.verify() {
                tracing::warn!("Manifest verification failed: {}", e);
                last_error = Some(e);
                continue;
            }
            
            // Extract block hashes
            let hashes: Vec<[u8; 32]> = manifest.blocks.iter().map(|(_, h)| *h).collect();
            
            // Fetch blocks concurrently
            let blocks = match self.fetch_blocks_concurrent(&hashes, provider, concurrency).await {
                Ok(b) => b,
                Err(e) => {
                    last_error = Some(e);
                    continue; // Try next provider
                }
            };
            
            return Ok((manifest, blocks));
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All providers failed")))
    }
}
```

---

### Phase 3: Request Timeouts & Retries

**File:** `communitas-core/src/gossip/sites.rs`

```rust
use tokio::time::{timeout, Duration};

const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RETRIES: usize = 3;

impl SiteFetcher {
    async fn fetch_block_with_timeout_retry(
        &self,
        hash: &[u8; 32],
        provider: PeerId,
    ) -> Result<Block> {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < MAX_RETRIES {
            match timeout(FETCH_TIMEOUT, self.fetch_block(hash, provider)).await {
                Ok(Ok(block)) => return Ok(block),
                Ok(Err(e)) => last_error = Some(e),
                Err(_) => last_error = Some(anyhow::anyhow!("Timeout")),
            }
            attempts += 1;
            
            // Exponential backoff
            if attempts < MAX_RETRIES {
                let delay = Duration::from_millis(100 * 2_u64.pow(attempts as u32));
                tokio::time::sleep(delay).await;
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Max retries exceeded")))
    }
}
```

---

## Success Criteria

**Phase 1 (Server Listener):**
- [ ] SitesListener running in background
- [ ] Incoming Bulk stream requests routed to SitePublisher
- [ ] Responses sent back to requester
- [ ] Backpressure via semaphore (max 10 concurrent)
- [ ] Timeouts on request processing (30s)
- [ ] Integration test: fetch manifest from local publisher

**Phase 2 (Concurrent Fetching):**
- [ ] fetch_blocks_concurrent() implemented
- [ ] fetch_site_complete() with manifest + blocks
- [ ] Configurable concurrency (default 4-8)
- [ ] Multi-provider failover
- [ ] Integration test: fetch complete site

**Phase 3 (Timeouts & Retries):**
- [ ] Per-request timeouts (10s)
- [ ] Exponential backoff retry (3 attempts)
- [ ] Integration test: timeout handling

---

## Implementation Order

**Today (2-3 hours):**
1. Create `sites_listener.rs`
2. Integrate into `GossipContext`
3. Write integration test
4. Fix any compilation issues

**Tomorrow:**
1. Implement concurrent block fetching
2. Add multi-provider failover
3. Add timeouts and retries
4. Test with real network conditions

**Next:**
1. Persistent cache (LRU + pinning)
2. Rendezvous anti-spam
3. Name binding protocol

---

## Testing Strategy

**Unit Tests:**
- SitesListener request handling
- Concurrent block fetching
- Timeout/retry logic

**Integration Tests:**
```rust
#[tokio::test]
async fn test_end_to_end_site_serving() {
    // 1. Create publisher with content
    let (sk, pk) = generate_test_keypair(1);
    let site_id = SiteId::from_public_key(&pk);
    let publisher = Arc::new(SitePublisher::new(site_id.clone()));
    
    // Add content
    let hash = publisher.add_asset("index.html", b"<html>").await.unwrap();
    let mut manifest = publisher.build_manifest(&pk, 1, vec![("index.html", hash)]).await.unwrap();
    manifest.sign(&sk).unwrap();
    
    // 2. Start listener
    let listener = Arc::new(SitesListener::new(transport.clone(), Some(publisher)));
    let _handle = listener.start();
    
    // 3. Create fetcher
    let fetcher = SiteFetcher::new(rendezvous);
    
    // 4. Fetch manifest
    let fetched_manifest = fetcher.fetch_manifest(&site_id, peer_id).await.unwrap();
    assert_eq!(fetched_manifest.root_hash, manifest.root_hash);
    fetched_manifest.verify().unwrap();
    
    // 5. Fetch block
    let block = fetcher.fetch_block(&hash, peer_id).await.unwrap();
    assert!(block.verify());
}
```

---

## Open Questions

1. **Should we use a dedicated QUIC endpoint for Sites or multiplex over gossip?**
   - Decision: **Multiplex over Bulk stream** (already implemented)
   - Rationale: Simpler, reuses existing transport, NAT traversal already solved

2. **How to handle large manifests (>10k files)?**
   - Option A: Stream manifest in chunks
   - Option B: Limit manifest size, use directories
   - Decision: Defer to Phase 4, add size limit for MVP

3. **Should we verify ALL blocks before returning site?**
   - Yes for security (prevents poisoning)
   - No for performance (incremental loading)
   - Decision: Verify manifest signature immediately, verify blocks as fetched

4. **How to handle concurrent requests to same provider?**
   - Add connection pooling
   - Limit concurrent streams per provider
   - Decision: Start simple, add pooling if needed

---

**Next Step:** Implement `sites_listener.rs`
