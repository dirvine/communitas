# Communitas Network Architecture

> **Status**: Production-ready
> **Last Updated**: 2025-10-16
> **Version**: 1.0

## Overview

Communitas uses a decentralized P2P networking architecture built on the Saorsa Gossip ecosystem. The network layer provides:

- **Identity Management**: Four-word human-verifiable addresses
- **Gossip Overlay**: Epidemic broadcast for messaging and signaling
- **P2P Connectivity**: Direct peer-to-peer connections via QUIC
- **WebRTC Integration**: Real-time voice, video, and screen sharing

## Network Stack

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  (Chat, Files, Calls, Groups, Channels, Projects)      │
└─────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    WebRTC Layer                          │
│  Voice/Video Calls • Screen Sharing • Media Streams     │
│  (saorsa-webrtc + ant-quic transport)                   │
└─────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│                 Gossip Overlay Network                   │
│  PubSub • Rendezvous • DHT • Signaling Transport       │
│  (saorsa-gossip)                                        │
└─────────────────────────────────────────────────────────┘
                           ▼
┌─────────────────────────────────────────────────────────┐
│                   Transport Layer                        │
│  QUIC (ant-quic) • IPv4/IPv6 • NAT Traversal           │
└─────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Identity System (Four-Word Addresses)

**Implementation**: `communitas-core/src/identity/`

Four-word addresses provide human-verifiable peer identities:
- Format: `ocean-forest-moon-star`
- Based on BIP-39 wordlist (2048 words)
- Cryptographically bound to ML-DSA public keys
- Collision-resistant (2048^4 = 17.6 trillion combinations)

**Key Features**:
- Human-readable and verifiable
- Works offline (no DNS/ICANN)
- Survives IP address changes
- Enables anti-phishing verification

### 2. Gossip Overlay Network

**Implementation**: `saorsa-gossip` crate

The gossip overlay provides:
- **PubSub**: Topic-based message broadcasting
- **Rendezvous**: Peer discovery and connection
- **DHT**: Distributed hash table for routing
- **Signaling**: WebRTC signaling transport

**Message Flow**:
```
Peer A                Gossip Overlay              Peer B
  │                         │                        │
  ├─ publish("topic") ─────►│                        │
  │                         ├──── broadcast ────────►│
  │                         │                        │
  │                         │◄── subscribe("topic")──┤
  │◄──── deliver ───────────┤                        │
```

### 3. WebRTC Integration

**Implementation**: `communitas-core/src/webrtc/`

WebRTC provides real-time multimedia capabilities over the P2P network.

#### Architecture

```
┌──────────────────────────────────────────────────────────┐
│                 Tauri Frontend (TypeScript)              │
│  Call UI • Media Controls • Device Selection             │
└──────────────────────────────────────────────────────────┘
                           ▼ IPC
┌──────────────────────────────────────────────────────────┐
│              Tauri Commands (webrtc_commands.rs)         │
│  webrtc_initiate_call • webrtc_accept_call              │
│  webrtc_set_video_enabled • webrtc_set_audio_enabled    │
│  webrtc_start_screen_share • webrtc_get_media_devices   │
└──────────────────────────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│         CommunitasWebRtcService (service.rs)             │
│  Call Management • State Tracking • Media Control        │
└──────────────────────────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│      GossipSignalingTransport (gossip_signaling.rs)     │
│  SDP Offer/Answer • ICE Candidates • Signaling          │
└──────────────────────────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│              Gossip Overlay Network                       │
│  PubSub for signaling • Rendezvous for discovery         │
└──────────────────────────────────────────────────────────┘
                           ▼
┌──────────────────────────────────────────────────────────┐
│                  ant-quic Transport                       │
│  P2P QUIC connections • NAT traversal • Media streams    │
└──────────────────────────────────────────────────────────┘
```

#### Key Features

**Voice & Video Calls**:
- Direct P2P connections (no relay servers)
- Native NAT traversal via ant-quic
- Post-quantum cryptography ready
- Audio/video codec negotiation

**Screen Sharing**:
- Screen capture via platform APIs
- Efficient screen sharing over QUIC
- Multiple screen/window selection

**Media Controls**:
- Dynamic video enable/disable
- Audio mute/unmute
- Device switching (camera, microphone)
- Screen share start/stop

**Call State Management**:
\`\`\`rust
pub struct CallState {
    pub call_id: CallId,
    pub target: CommunitasIdentity,
    pub constraints: MediaConstraints,
    pub is_video_enabled: bool,
    pub is_audio_enabled: bool,
    pub is_screen_sharing: bool,
}
\`\`\`

### 4. Signaling Protocol

**Implementation**: `communitas-core/src/webrtc/gossip_signaling.rs`

WebRTC signaling over gossip uses PubSub topics:

**Topics**:
- `webrtc.signaling.{peer_id}` - Direct peer signaling
- `webrtc.discovery` - Peer discovery for calls

**Message Types**:
- `Offer` - SDP offer from caller
- `Answer` - SDP answer from callee
- `IceCandidate` - ICE candidate for connection
- `CallEnd` - Call termination signal

**Flow**:
```
Caller                  Gossip Network                Callee
  │                          │                           │
  ├─ Offer ─────────────────►│                           │
  │                          ├─────── deliver ──────────►│
  │                          │                           │
  │                          │◄────── Answer ────────────┤
  │◄──── deliver ────────────┤                           │
  │                          │                           │
  ├─ ICE Candidates ────────►│◄──── ICE Candidates ──────┤
  │                          │                           │
  │◄───────── QUIC Connection Established ──────────────►│
  │                                                       │
  │◄────────── Media Streams (RTP over QUIC) ───────────►│
```

### 5. NAT Traversal

**Implementation**: ant-quic with native NAT traversal

**Techniques**:
- **STUN**: Discover public IP/port mappings
- **TURN**: Relay connections when direct fails (fallback)
- **Happy Eyeballs**: IPv4/IPv6 dual-stack with fallback
- **Hole Punching**: Coordinate connection through gossip signaling

**Connection Establishment**:
1. Both peers publish their ICE candidates via gossip
2. ant-quic establishes direct QUIC connection
3. If direct fails, gossip provides relay fallback
4. Media flows over established QUIC connection

## API Reference

### Tauri Commands (Frontend → Backend)

**Call Lifecycle**:
\`\`\`typescript
// Initiate a call
await invoke('webrtc_initiate_call', {
  targetFourWords: 'ocean-forest-moon-star',
  hasAudio: true,
  hasVideo: true,
  hasScreenShare: false
}); // Returns: call_id

// Accept incoming call
await invoke('webrtc_accept_call', { callId: 'call_uuid' });

// Reject incoming call
await invoke('webrtc_reject_call', { callId: 'call_uuid' });

// End active call
await invoke('webrtc_end_call', { callId: 'call_uuid' });
\`\`\`

**Media Controls**:
\`\`\`typescript
// Toggle video
await invoke('webrtc_set_video_enabled', {
  callId: 'call_uuid',
  enabled: false
});

// Mute/unmute audio
await invoke('webrtc_set_audio_enabled', {
  callId: 'call_uuid',
  enabled: false
});

// Screen sharing
await invoke('webrtc_start_screen_share', { callId: 'call_uuid' });
await invoke('webrtc_stop_screen_share', { callId: 'call_uuid' });

// Device enumeration
const devices = await invoke('webrtc_get_media_devices');
// Returns: Array<{ device_id, label, kind }>
\`\`\`

**Event Subscription**:
\`\`\`typescript
// Subscribe to call events
await invoke('webrtc_subscribe_events');

// Listen for events via Tauri event system
listen('webrtc:call-initiated', (event) => { ... });
listen('webrtc:call-accepted', (event) => { ... });
listen('webrtc:call-ended', (event) => { ... });
\`\`\`

### Rust API (Backend)

**Service Creation**:
\`\`\`rust
use communitas_core::webrtc::CommunitasWebRtcService;
use communitas_core::gossip::GossipContext;

let gossip = Arc::new(RwLock::new(gossip_context));
let webrtc = CommunitasWebRtcService::new(gossip)?;
webrtc.start().await?;
\`\`\`

**Call Management**:
\`\`\`rust
// Initiate call
let call_id = webrtc.initiate_call(
    "ocean-forest-moon-star",
    MediaConstraints::video_call()
).await?;

// Accept call
webrtc.accept_call(call_id).await?;

// Media controls
webrtc.set_video_enabled(call_id, false).await?;
webrtc.set_audio_enabled(call_id, false).await?;
webrtc.start_screen_share(call_id).await?;

// Device enumeration
let devices = webrtc.get_media_devices().await?;
\`\`\`

**Event Handling**:
\`\`\`rust
let mut events = webrtc.subscribe_events();
while let Ok(event) = events.recv().await {
    match event {
        CallEvent::CallInitiated { call_id, callee, .. } => { ... }
        CallEvent::CallAccepted { call_id } => { ... }
        CallEvent::CallEnded { call_id } => { ... }
        _ => {}
    }
}
\`\`\`

## Performance Characteristics

### Latency Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Local Message | <100ms | Within same network |
| Remote Message | <500ms | Geographic routing |
| Call Signaling | <2s | SDP offer/answer exchange |
| Call Connection | <5s | Including NAT traversal |
| Media Latency | <200ms | Audio/video end-to-end |

### Bandwidth Usage

| Media Type | Bandwidth | Quality |
|------------|-----------|---------|
| Audio Only | 32-64 kbps | Voice codec |
| Video (360p) | 500 kbps | Mobile quality |
| Video (720p) | 1.5 Mbps | HD quality |
| Video (1080p) | 3 Mbps | Full HD |
| Screen Share | 1-2 Mbps | Variable |

## Security Model

### Encryption

**Transport Layer**:
- QUIC with TLS 1.3
- Post-quantum ready (ML-KEM integration planned)

**Application Layer**:
- End-to-end encrypted media (SRTP)
- Forward secrecy for call keys
- Identity-based authentication

### Authentication

**Peer Verification**:
- Four-word address checksum verification
- ML-DSA signature verification
- Out-of-band verification support

**Call Security**:
- Caller ID verification via four-word address
- Optional ZRTP-style verification phrases
- Call encryption key fingerprints

## Testing

### Unit Tests

Located in:
- `communitas-desktop/tests/webrtc_call_lifecycle_tests.rs` (15 tests)
- `communitas-desktop/tests/webrtc_media_controls_tests.rs` (17 tests)

Run with:
\`\`\`bash
cargo test -p communitas-desktop webrtc_
\`\`\`

### Integration Tests

Test real call flows with multiple peers:
\`\`\`bash
cargo test -p communitas-desktop --test integration_tests
\`\`\`

### Manual Testing

1. Start two instances on different ports
2. Initiate call from instance A to B
3. Accept call on instance B
4. Test media controls (video, audio, screen share)
5. End call from either side

## Troubleshooting

### Common Issues

**Call Fails to Connect**:
- Check gossip network connectivity
- Verify NAT traversal configuration
- Check firewall rules for QUIC traffic

**No Media Streams**:
- Verify device permissions (camera, microphone)
- Check media constraints in call initiation
- Inspect media device enumeration

**High Latency**:
- Check network conditions (ping, traceroute)
- Verify direct P2P connection (not relayed)
- Monitor QUIC connection statistics

### Debug Logging

Enable debug logs:
\`\`\`bash
RUST_LOG=communitas_core::webrtc=debug cargo run
\`\`\`

Monitor signaling:
\`\`\`bash
RUST_LOG=communitas_core::webrtc::gossip_signaling=trace cargo run
\`\`\`

## Future Enhancements

### Planned Features

- [ ] Group video calls (multi-peer)
- [ ] Recording and playback
- [ ] Virtual backgrounds
- [ ] Noise cancellation
- [ ] Bandwidth adaptation
- [ ] Network quality indicators

### Research Areas

- Post-quantum key exchange for media
- Mesh networking for group calls
- Codec optimization for bandwidth
- Battery optimization on mobile

## References

- [WebRTC Specification](https://www.w3.org/TR/webrtc/)
- [saorsa-webrtc Documentation](https://docs.rs/saorsa-webrtc)
- [ant-quic Documentation](https://docs.rs/ant-quic)
- [Four-Word Networking](https://docs.rs/four-word-networking)

## See Also

- [WebRTC Multimedia Architecture](./webrtc-multimedia.md)
- [Gossip Protocol Details](./gossip-protocol.md)
- [Security Architecture](./security.md)
- [API Reference](../api/tauri-commands.md)
