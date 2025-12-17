# mDNS Local Network Discovery Integration Plan

## Executive Summary

This document outlines a production-ready mDNS/Bonjour integration for Communitas that enables local network peer discovery while maintaining compatibility with the existing WAN-based peer discovery system. The solution addresses hairpinning issues, implements address prioritization, and maintains dual-address publication for external connectivity.

---

## 1. Architecture Overview

### 1.1 Current Discovery Architecture

The existing peer discovery system follows this hierarchy:

```
+-------------------------------------------------------------+
|                    FoafDiscovery                            |
+-------------------------------------------------------------+
|  1. Local Cache (O(1) lookup)                               |
|  2. Presence Manager (group-scoped beacons)                 |
|  3. FOAF Queries (2-hop max through contact network)        |
|  4. Introducer Nodes (cold start fallback)                  |
+-------------------------------------------------------------+
```

**Key Files:**
- `/communitas-core/src/gossip/discovery.rs` - FoafDiscovery manager
- `/communitas-core/src/gossip/peer_cache.rs` - Persistent PeerCache with scoring
- `/communitas-core/src/gossip/boot.rs` - GossipBootSequence orchestration

### 1.2 Proposed mDNS Architecture

```
+------------------------------------------------------------------------+
|                         Discovery Stack                                |
+------------------------------------------------------------------------+
|                                                                        |
|  +----------------------+     +----------------------+                 |
|  |   mDNS Discovery     |     |    WAN Discovery     |                 |
|  |  (_communitas._tcp)  |     |   (FOAF/Presence)    |                 |
|  |                      |     |                      |                 |
|  |  - NWBrowser (Swift) |     |  - FoafDiscovery     |                 |
|  |  - NWListener (Swift)|     |  - PresenceManager   |                 |
|  |  - TXT records       |     |  - IntroducerNodes   |                 |
|  +----------+-----------+     +----------+-----------+                 |
|             |                            |                             |
|             +------------+---------------+                             |
|                          v                                             |
|             +------------------------+                                 |
|             |   Unified Peer Cache   |                                 |
|             |                        |                                 |
|             |  - Address Prioritization                                |
|             |  - Local vs Public scoring                               |
|             |  - Hairpin detection                                     |
|             +------------------------+                                 |
|                          |                                             |
|                          v                                             |
|             +------------------------+                                 |
|             |   Connection Manager   |                                 |
|             |                        |                                 |
|             |  - Try local first     |                                 |
|             |  - Fallback to public  |                                 |
|             |  - QUIC over ant-quic  |                                 |
|             +------------------------+                                 |
+------------------------------------------------------------------------+
```

---

## 2. Implementation Layer Decision

### 2.1 Recommendation: Swift Layer (Network.framework)

**Why Swift over Rust:**

| Aspect | Swift (Network.framework) | Rust (libmdns) |
|--------|---------------------------|----------------|
| Platform Integration | Native macOS APIs | Cross-platform but less native |
| Permissions | Automatic sandbox handling | Manual entitlements |
| Power Efficiency | Coalesced with system mDNS | Separate daemon |
| Bonjour Support | First-class NWBrowser/NWListener | Third-party crate |
| iOS Compatibility | Full support | Limited |
| Code Complexity | ~200 lines | ~500+ lines |

**Architectural Rationale:**
1. Network.framework's NWBrowser/NWListener are designed for exactly this use case
2. macOS/iOS share the same mDNS infrastructure - we get cross-device discovery free
3. UniFFI bindings already exist for Rust<->Swift communication
4. Local discovery is platform-specific; keep platform code in platform layer

### 2.2 Integration Point

The Swift mDNS service will communicate with the Rust core via:
1. **Discovered peers** -> Push to `CommunitasClient.gossip_add_contact()` with local address hints
2. **Service registration** -> Use four-word identity from `CommunitasClient.get_profile()`

---

## 3. Detailed Design

### 3.1 Service Registration Format

**Service Type:** `_communitas._tcp.local`

**TXT Record Fields:**
```
fw=ocean-forest-moon-star    // Four-word identity (required)
pid=<hex-encoded-peer-id>    // 32-byte PeerId (required)
port=4433                    // QUIC port (required)
ver=1                        // Protocol version (required)
name=Alice                   // Display name (optional)
```

**Service Name:** `{four-words}._communitas._tcp.local`

### 3.2 Swift mDNS Service Interface

```swift
// Sources/CommunitasApp/Services/MDNSDiscoveryService.swift

import Network
import CommunitasKit

/// Local network peer discovery via mDNS/Bonjour
@MainActor
class MDNSDiscoveryService: ObservableObject {
    // MARK: - Published State
    @Published private(set) var isAdvertising: Bool = false
    @Published private(set) var isBrowsing: Bool = false
    @Published private(set) var discoveredPeers: [MDNSPeer] = []
    
    // MARK: - Private Properties
    private var listener: NWListener?
    private var browser: NWBrowser?
    private var appState: AppState
    
    // Service configuration
    private let serviceType = "_communitas._tcp"
    private let serviceDomain = "local"
    
    // MARK: - Initialization
    init(appState: AppState) {
        self.appState = appState
    }
    
    // MARK: - Service Advertisement
    
    /// Start advertising our presence on local network
    func startAdvertising(port: UInt16) throws {
        guard let profile = appState.client?.getProfile() else {
            throw MDNSError.notInitialized
        }
        
        // Create TXT record with peer info
        let txtRecord = createTXTRecord(
            fourWords: profile.fourWords,
            peerId: appState.connectionIdentity ?? "",
            port: port,
            displayName: profile.displayName
        )
        
        // Configure listener
        let parameters = NWParameters.tcp
        parameters.includePeerToPeer = true
        
        listener = try NWListener(using: parameters, on: NWEndpoint.Port(rawValue: port)!)
        
        // Set service registration
        listener?.service = NWListener.Service(
            name: profile.fourWords,
            type: serviceType,
            domain: serviceDomain,
            txtRecord: txtRecord
        )
        
        listener?.stateUpdateHandler = { [weak self] state in
            self?.handleListenerState(state)
        }
        
        listener?.start(queue: .main)
        isAdvertising = true
    }
    
    /// Stop advertising
    func stopAdvertising() {
        listener?.cancel()
        listener = nil
        isAdvertising = false
    }
    
    // MARK: - Peer Discovery
    
    /// Start browsing for other Communitas peers
    func startBrowsing() {
        let descriptor = NWBrowser.Descriptor.bonjour(
            type: serviceType,
            domain: serviceDomain
        )
        
        let parameters = NWParameters()
        parameters.includePeerToPeer = true
        
        browser = NWBrowser(for: descriptor, using: parameters)
        
        browser?.stateUpdateHandler = { [weak self] state in
            self?.handleBrowserState(state)
        }
        
        browser?.browseResultsChangedHandler = { [weak self] results, changes in
            self?.handleBrowseResults(results, changes: changes)
        }
        
        browser?.start(queue: .main)
        isBrowsing = true
    }
    
    /// Stop browsing
    func stopBrowsing() {
        browser?.cancel()
        browser = nil
        isBrowsing = false
        discoveredPeers.removeAll()
    }
    
    // MARK: - Private Helpers
    
    private func createTXTRecord(
        fourWords: String,
        peerId: String,
        port: UInt16,
        displayName: String
    ) -> NWTXTRecord {
        var record = NWTXTRecord()
        record["fw"] = fourWords
        record["pid"] = peerId
        record["port"] = String(port)
        record["ver"] = "1"
        record["name"] = displayName
        return record
    }
    
    private func handleBrowseResults(_ results: Set<NWBrowser.Result>, changes: Set<NWBrowser.Result.Change>) {
        for change in changes {
            switch change {
            case .added(let result):
                resolveAndAddPeer(result)
            case .removed(let result):
                removePeer(result)
            default:
                break
            }
        }
    }
    
    private func resolveAndAddPeer(_ result: NWBrowser.Result) {
        // Extract TXT record data
        guard case .service(let name, let type, let domain, _) = result.endpoint else {
            return
        }
        
        // Skip our own service
        if name == appState.fourWords {
            return
        }
        
        // Resolve endpoint to get IP address
        let connection = NWConnection(to: result.endpoint, using: .tcp)
        connection.stateUpdateHandler = { [weak self] state in
            if case .ready = state {
                // Extract resolved address
                if let innerEndpoint = connection.currentPath?.remoteEndpoint,
                   case .hostPort(let host, let port) = innerEndpoint {
                    let peer = MDNSPeer(
                        fourWords: name,
                        host: host,
                        port: port,
                        txtRecord: result.metadata
                    )
                    
                    DispatchQueue.main.async {
                        self?.addDiscoveredPeer(peer)
                    }
                }
                connection.cancel()
            }
        }
        connection.start(queue: .global())
    }
    
    private func addDiscoveredPeer(_ peer: MDNSPeer) {
        // Check for duplicate
        if !discoveredPeers.contains(where: { $0.fourWords == peer.fourWords }) {
            discoveredPeers.append(peer)
            
            // Notify Rust core about discovered peer with local address hints
            notifyRustCore(peer: peer)
        }
    }
    
    private func notifyRustCore(peer: MDNSPeer) {
        // Add contact with local address priority
        appState.addContact(
            fourWords: peer.fourWords,
            displayName: peer.displayName
        )
        
        // TODO: Pass local address hints to peer cache
        // This requires extending the CommunitasClient bindings
    }
}

/// Represents a peer discovered via mDNS
struct MDNSPeer: Identifiable, Equatable {
    let id = UUID()
    let fourWords: String
    let host: NWEndpoint.Host
    let port: NWEndpoint.Port
    let displayName: String?
    let peerId: String?
    let discoveredAt: Date = Date()
    
    var localAddress: String {
        "\(host):\(port)"
    }
    
    init(fourWords: String, host: NWEndpoint.Host, port: NWEndpoint.Port, txtRecord: NWBrowser.Result.Metadata?) {
        self.fourWords = fourWords
        self.host = host
        self.port = port
        
        // Extract from TXT record
        if case .bonjour(let txt) = txtRecord {
            self.displayName = txt["name"]
            self.peerId = txt["pid"]
        } else {
            self.displayName = nil
            self.peerId = nil
        }
    }
}

enum MDNSError: Error {
    case notInitialized
    case advertisingFailed(String)
    case browsingFailed(String)
}
```

### 3.3 Address Prioritization in Peer Cache

Extend the Rust `PeerCacheEntry` to support local addresses:

```rust
// In communitas-core/src/gossip/peer_cache.rs

/// Address type for prioritization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressSource {
    /// Discovered via mDNS on local network
    Local,
    /// Public address from presence/FOAF
    Public,
    /// Manually configured
    Manual,
}

/// Extended address hint with source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressHint {
    pub addr: SocketAddr,
    pub source: AddressSource,
    pub last_verified: SystemTime,
    pub latency_ms: Option<u32>,
}

impl PeerCacheEntry {
    /// Get prioritized addresses (local first, then public)
    pub fn get_prioritized_addresses(&self) -> Vec<&AddressHint> {
        let mut hints: Vec<&AddressHint> = self.addr_hints_v2.iter().collect();
        
        // Sort: Local > Manual > Public, then by latency
        hints.sort_by(|a, b| {
            let source_priority = |s: &AddressSource| match s {
                AddressSource::Local => 0,
                AddressSource::Manual => 1,
                AddressSource::Public => 2,
            };
            
            let a_priority = source_priority(&a.source);
            let b_priority = source_priority(&b.source);
            
            if a_priority != b_priority {
                return a_priority.cmp(&b_priority);
            }
            
            // Same source, prefer lower latency
            a.latency_ms.cmp(&b.latency_ms)
        });
        
        hints
    }
    
    /// Check if we have a local address for this peer
    pub fn has_local_address(&self) -> bool {
        self.addr_hints_v2.iter().any(|h| h.source == AddressSource::Local)
    }
    
    /// Add or update a local address hint from mDNS discovery
    pub fn add_local_address(&mut self, addr: SocketAddr) {
        // Remove any existing local hint (mDNS gives us current address)
        self.addr_hints_v2.retain(|h| h.source != AddressSource::Local);
        
        self.addr_hints_v2.push(AddressHint {
            addr,
            source: AddressSource::Local,
            last_verified: SystemTime::now(),
            latency_ms: None, // Will be measured on first connection
        });
    }
}
```

### 3.4 Hairpin Detection and Avoidance

```rust
// In communitas-core/src/gossip/peer_cache.rs

impl PeerCache {
    /// Detect if an address is our own public IP (hairpin scenario)
    pub fn is_hairpin_address(&self, addr: &SocketAddr) -> bool {
        // Compare against our known public addresses
        if let Some(our_public) = &self.our_public_address {
            return addr.ip() == our_public.ip();
        }
        false
    }
    
    /// Filter out hairpin addresses when selecting connection targets
    pub fn get_safe_addresses(&self, entry: &PeerCacheEntry) -> Vec<&AddressHint> {
        entry.get_prioritized_addresses()
            .into_iter()
            .filter(|hint| !self.is_hairpin_address(&hint.addr))
            .collect()
    }
}
```

### 3.5 Dual Address Publication

The node must advertise both local AND public addresses:

```rust
// In communitas-core/src/gossip/presence.rs (extend PresenceRecord)

impl PresenceRecord {
    /// Create presence record with both local and public addresses
    pub fn with_dual_addresses(
        peer_id: [u8; 32],
        local_addrs: Vec<String>,   // From mDNS service
        public_addrs: Vec<String>,  // From STUN/public discovery
        ttl_seconds: u32,
        four_words: String,
    ) -> Self {
        let mut addr_hints = Vec::new();
        
        // Add local addresses first (for LAN peers)
        for addr in local_addrs {
            addr_hints.push(format!("local:{}", addr));
        }
        
        // Add public addresses (for WAN peers)
        for addr in public_addrs {
            addr_hints.push(format!("public:{}", addr));
        }
        
        Self {
            peer_id,
            addr_hints,
            ttl_seconds,
            four_words: Some(four_words),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
}
```

---

## 4. Boot Sequence Integration

### 4.1 Modified Boot Sequence

```
+------------------------------------------------------------------+
|                    Boot Sequence (Updated)                       |
+------------------------------------------------------------------+
|  1. Load ML-DSA identity                                         |
|  2. Start mDNS advertisement (NEW)                               |
|  3. Start mDNS browsing for local peers (NEW)                    |
|  4. Dial local peers first (NEW - skip hairpin)                  |
|  5. Dial favourite contacts over QUIC (WAN)                      |
|  6. Start membership (HyParView + SWIM)                          |
|  7. Join channels/orgs                                           |
|  8. Start presence beacons (include both local & public addrs)   |
|  9. Start CRDT anti-entropy                                      |
+------------------------------------------------------------------+
```

### 4.2 Swift AppState Integration

```swift
// In Sources/CommunitasApp/AppState.swift

class AppState: ObservableObject {
    // ... existing properties ...
    
    /// mDNS discovery service
    lazy var mdnsService: MDNSDiscoveryService = {
        MDNSDiscoveryService(appState: self)
    }()
    
    /// Start networking with mDNS support
    func startNetworkingWithMDNS(port: UInt16? = nil) {
        do {
            // 1. Start Rust networking core
            let identity = try client?.gossipStart(port: port)
            connectionIdentity = identity
            isNetworking = true
            
            // 2. Start mDNS advertisement
            let quicPort = port ?? NetworkConfig.defaultPort
            try mdnsService.startAdvertising(port: quicPort)
            
            // 3. Start mDNS browsing
            mdnsService.startBrowsing()
            
            // 4. Start presence beacons
            startPresenceBeacons()
            
            print("[Communitas] Networking started with mDNS support")
        } catch {
            errorMessage = "Failed to start networking: \(error.localizedDescription)"
        }
    }
    
    /// Stop networking
    func stopNetworkingWithMDNS() {
        // Stop mDNS services
        mdnsService.stopBrowsing()
        mdnsService.stopAdvertising()
        
        // Stop Rust networking
        do {
            stopContactPolling()
            try client?.gossipStop()
            isNetworking = false
        } catch {
            errorMessage = "Failed to stop networking: \(error.localizedDescription)"
        }
    }
}
```

---

## 5. Rust Bindings Extension

### 5.1 New UniFFI Functions

```rust
// In communitas-bindings/src/lib.rs

/// Add a local address hint for a peer (called from mDNS discovery)
pub fn peer_cache_add_local_address(
    &self,
    four_words: String,
    local_addr: String,
) -> Result<(), ClientError> {
    block_on(async {
        let ctx = self.inner.read().await;
        
        // Parse address
        let addr: SocketAddr = local_addr
            .parse()
            .map_err(|e| ClientError::NetworkError(format!("Invalid address: {}", e)))?;
        
        // Find or create peer entry
        let mut cache = ctx.peer_cache.write().await;
        
        // Create peer_id from four_words (consistent with existing logic)
        let peer_id_bytes = blake3::hash(four_words.as_bytes());
        let peer_id = PeerId::new(*peer_id_bytes.as_bytes());
        
        cache.add_local_address_hint(peer_id, addr).await
            .map_err(|e| ClientError::NetworkError(e.to_string()))
    })
}

/// Get our local addresses for mDNS advertisement
pub fn get_local_addresses(&self) -> Vec<String> {
    block_on(async {
        let ctx = self.inner.read().await;
        ctx.get_local_interface_addresses()
            .await
            .unwrap_or_default()
    })
}

/// Check if a peer is reachable via local network
pub fn is_peer_local(&self, four_words: String) -> bool {
    block_on(async {
        let ctx = self.inner.read().await;
        ctx.peer_cache.read().await
            .get_peer_by_four_words(&four_words)
            .map(|e| e.has_local_address())
            .unwrap_or(false)
    })
}
```

---

## 6. Connection Flow

### 6.1 Connection Attempt Order

```
+-------------------------------------------------------------+
|           Peer Connection Attempt (dial_contact)            |
+-------------------------------------------------------------+
|                                                             |
|  1. Get peer from cache by four-words                       |
|     |                                                       |
|     +- Not found? -> Try FOAF discovery -> Continue         |
|     |                                                       |
|  2. Get prioritized addresses                               |
|     |                                                       |
|     +- Local addresses (from mDNS)                          |
|     +- Manual addresses                                     |
|     +- Public addresses (from presence)                     |
|                                                             |
|  3. Filter out hairpin addresses                            |
|     |                                                       |
|  4. For each address (in priority order):                   |
|     |                                                       |
|     +- Attempt QUIC connection (with timeout)               |
|     |   |                                                   |
|     |   +- Success? -> Update cache, return                 |
|     |   |                                                   |
|     |   +- Failure? -> Try next address                     |
|     |                                                       |
|  5. All addresses failed? -> Return error                   |
+-------------------------------------------------------------+
```

### 6.2 Connection Manager Update

```rust
// In communitas-core/src/gossip/context.rs

impl GossipContext {
    /// Connect to peer with local-first priority
    pub async fn connect_to_peer_prioritized(&self, four_words: &str) -> Result<()> {
        // 1. Get peer from discovery
        let peer_id = self.discovery.find_contact(four_words).await?;
        
        // 2. Get prioritized addresses from cache
        let cache = self.peer_cache.read().await;
        let entry = cache.get_peer(&peer_id)?;
        let addresses = cache.get_safe_addresses(&entry);
        
        if addresses.is_empty() {
            return Err(anyhow::anyhow!("No reachable addresses for {}", four_words));
        }
        
        // 3. Try connections in priority order (local first)
        for hint in addresses {
            debug!("Attempting connection to {} via {:?} ({:?})", 
                   four_words, hint.addr, hint.source);
            
            match self.transport.connect(peer_id, hint.addr).await {
                Ok(_) => {
                    info!("Connected to {} via {:?}", four_words, hint.source);
                    // Update latency measurement
                    drop(cache);
                    let mut cache_mut = self.peer_cache.write().await;
                    cache_mut.update_success(peer_id, hint.addr).await?;
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to connect to {} via {}: {}", 
                          four_words, hint.addr, e);
                    continue;
                }
            }
        }
        
        Err(anyhow::anyhow!("All connection attempts to {} failed", four_words))
    }
}
```

---

## 7. App Entitlements

### 7.1 Required macOS Entitlements

```xml
<!-- CommunitasApp.entitlements -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <!-- Existing entitlements... -->
    
    <!-- mDNS/Bonjour -->
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
    
    <!-- Local network access -->
    <key>com.apple.developer.networking.multicast</key>
    <true/>
</dict>
</plist>
```

### 7.2 Info.plist Additions

```xml
<!-- Required for local network permission prompt on macOS 11+ -->
<key>NSLocalNetworkUsageDescription</key>
<string>Communitas uses local network access to discover other devices running Communitas for peer-to-peer communication.</string>

<key>NSBonjourServices</key>
<array>
    <string>_communitas._tcp</string>
</array>
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

1. **TXT Record Parsing** - Verify four-word extraction from mDNS records
2. **Address Prioritization** - Verify local addresses sort before public
3. **Hairpin Detection** - Verify own public IP is filtered
4. **Dual Publication** - Verify presence includes both address types

### 8.2 Integration Tests

1. **Two-node LAN discovery** - Start two instances, verify mutual discovery
2. **Hairpin avoidance** - Both nodes on same NAT, verify local connection
3. **Mixed network** - One local, one remote, verify appropriate routing
4. **Cold start with mDNS** - New node with no contacts discovers LAN peers

### 8.3 E2E Tests (via MCP)

```swift
// Test scenario: Two Mac instances on same network
// 1. Start instance A, verify mDNS advertisement
// 2. Start instance B, verify discovers A
// 3. Send message A->B via local address
// 4. Verify message delivery
```

---

## 9. Implementation Phases

### Phase 1: Swift mDNS Service (2-3 days)
- [ ] Create `MDNSDiscoveryService.swift`
- [ ] Implement NWListener for advertisement
- [ ] Implement NWBrowser for discovery
- [ ] Add to AppState lifecycle

### Phase 2: Rust Peer Cache Extension (2 days)
- [ ] Add `AddressSource` enum
- [ ] Add `AddressHint` struct with source
- [ ] Implement `get_prioritized_addresses()`
- [ ] Add hairpin detection

### Phase 3: UniFFI Bindings (1 day)
- [ ] Add `peer_cache_add_local_address` function
- [ ] Add `get_local_addresses` function
- [ ] Add `is_peer_local` function
- [ ] Regenerate Swift bindings

### Phase 4: Boot Sequence Integration (1 day)
- [ ] Modify `startNetworking` to include mDNS
- [ ] Add mDNS to stop sequence
- [ ] Update presence to include local addresses

### Phase 5: Testing and Polish (2 days)
- [ ] Unit tests for all components
- [ ] Integration tests with two nodes
- [ ] E2E test via Chrome DevTools MCP
- [ ] Documentation updates

---

## 10. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| mDNS conflicts with other apps | Use unique service type `_communitas._tcp` |
| Privacy concerns (local discovery) | Only advertise when networking enabled |
| Battery drain on mobile | Use system mDNS daemon (efficient) |
| Stale mDNS records | TTL-based expiry (2 minutes) |
| Firewall blocking mDNS | Document port 5353/UDP requirement |

---

## 11. Future Considerations

1. **iOS Support** - Same Network.framework APIs work on iOS
2. **Android** - Would need NSD (Network Service Discovery) implementation
3. **Windows** - Would need Bonjour SDK or alternative
4. **Linux** - Avahi integration via D-Bus

---

## 12. Conclusion

This design provides a robust mDNS integration that:
- Prioritizes local connections for speed and reliability
- Avoids hairpinning issues automatically
- Maintains WAN connectivity for external peers
- Uses native platform APIs for efficiency
- Integrates cleanly with existing discovery infrastructure

The Swift-layer implementation leverages Network.framework's mature mDNS support while the Rust core handles address prioritization and connection management, maintaining a clean separation of concerns.

---

## Critical Files for Implementation

### Primary Files to Modify/Create:

1. **`/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-swift/CommunitasApp/Sources/CommunitasApp/Services/MDNSDiscoveryService.swift`** (NEW)
   - Core mDNS service using Network.framework
   - NWBrowser and NWListener implementation
   - TXT record parsing and creation

2. **`/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-core/src/gossip/peer_cache.rs`**
   - Add `AddressSource` enum and `AddressHint` struct
   - Implement address prioritization logic
   - Add hairpin detection

3. **`/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-swift/CommunitasApp/Sources/CommunitasApp/AppState.swift`**
   - Integrate MDNSDiscoveryService
   - Add mDNS lifecycle management
   - Connect discovery events to peer cache

4. **`/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-bindings/src/lib.rs`**
   - Add new UniFFI functions for local address handling
   - Bridge mDNS discoveries to Rust core

5. **`/Users/davidirvine/Desktop/Devel/projects/communitas/communitas-core/src/gossip/presence.rs`**
   - Extend PresenceRecord to include address source tags
   - Support dual address publication
