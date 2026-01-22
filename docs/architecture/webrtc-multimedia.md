# WebRTC Multimedia Architecture

**Status**: Production Ready (Phase 6.4)

This document describes the WebRTC multimedia architecture in Communitas, covering the call signaling flow, service layer design, and platform integration boundaries.

## Overview

Communitas provides real-time voice and video communication through a layered architecture:

- **Signaling**: Gossip-based signaling via the Saorsa gossip overlay
- **Identity**: Four-word addresses for call participants
- **Transport**: QUIC connections via `ant-quic` with PQC encryption
- **UI Services**: `communitas-ui-service` provides reactive state management
- **Platform**: Device enumeration and media capture via platform hosts

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                     Dioxus UI Components                         │
│  (CallView, CallControls, ParticipantGrid, ScreenSharePicker)   │
├─────────────────────────────────────────────────────────────────┤
│                   communitas-ui-service                          │
│  (CallService, CallHistory, MissedCalls, QualityMetrics)        │
├─────────────────────────────────────────────────────────────────┤
│                    communitas-core                               │
│  (Commands: StartCall, JoinCall, LeaveCall, etc.)               │
├─────────────────────────────────────────────────────────────────┤
│                  saorsa-webrtc-core                              │
│  (WebRTC state machines, ICE, media tracks)                     │
├─────────────────────────────────────────────────────────────────┤
│                  Saorsa Gossip Overlay                           │
│  (P2P signaling, presence, NAT traversal)                       │
└─────────────────────────────────────────────────────────────────┘
```

## Signaling Flow

### Call Initiation

```
Caller                    Gossip Network                   Callee
  │                            │                              │
  │ Command::StartCall         │                              │
  │───────────────────────>────│                              │
  │                            │ CallOffer (via gossip)       │
  │                            │─────────────────────────────>│
  │                            │                              │
  │                            │ CallAnswer (via gossip)      │
  │<───────────────────────────│<─────────────────────────────│
  │                            │                              │
  │ Event::CallStarted         │                              │
  │<───────────────────────────│                              │
  │                            │ ICE Candidates (bilateral)   │
  │<───────────────────────────│<────────────────────────────>│
  │                            │                              │
  │ Event::CallConnected       │ Event::CallConnected         │
  │<───────────────────────────│─────────────────────────────>│
```

### Group Call Join

```
New Participant            Gossip Network                Existing Participants
  │                            │                              │
  │ Command::JoinCall          │                              │
  │───────────────────────────>│                              │
  │                            │ ParticipantJoined (gossip)   │
  │                            │─────────────────────────────>│
  │                            │                              │
  │ Event::ParticipantJoined   │                              │
  │<───────────────────────────│<─────────────────────────────│
  │                            │ (for each existing member)   │
```

## Call Service (communitas-ui-service)

The `CallService` provides a UI-friendly abstraction over WebRTC:

### State Management

```rust
pub struct CallService {
    state: Arc<RwLock<CallServiceState>>,
    device_enumerator: Arc<dyn DeviceEnumerator>,
    screen_source_enumerator: Arc<dyn ScreenSourceEnumerator>,
    history: Arc<RwLock<CallHistory>>,
    // ... channels for reactive updates
}
```

### Reactive Updates via Watch Channels

```rust
// Subscribe to call state changes
let mut rx = call_service.subscribe();

// React to state changes
rx.changed().await?;
let snapshot = rx.borrow().clone();

match snapshot.state {
    CallState::Idle => { /* Show call button */ }
    CallState::Ringing => { /* Show incoming call UI */ }
    CallState::Connecting => { /* Show connecting indicator */ }
    CallState::InCall => { /* Show call controls */ }
    CallState::Reconnecting => { /* Show reconnecting status */ }
}
```

### Key Features

| Feature | Description |
|---------|-------------|
| **Device Enumeration** | Platform-specific via `DeviceEnumerator` trait |
| **Screen Sharing** | Source selection via `ScreenSourceEnumerator` trait |
| **Quality Metrics** | Real-time RTT, jitter, packet loss tracking |
| **Call History** | Persistent storage with search and filtering |
| **Missed Calls** | Notification system with acknowledgment |
| **Recording** | Start/stop/pause with progress tracking |
| **Presence** | In-call status integrated with contacts |

## Platform Integration

### Device Enumerator Trait

Platform hosts implement this trait for real device discovery:

```rust
#[async_trait]
pub trait DeviceEnumerator: Send + Sync {
    async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError>;
    async fn is_device_available(&self, device_id: &str) -> bool;
}
```

### Screen Source Enumerator Trait

Platform hosts implement this for screen capture source discovery:

```rust
#[async_trait]
pub trait ScreenSourceEnumerator: Send + Sync {
    async fn enumerate_sources(&self) -> Result<Vec<ScreenShareSource>, CallError>;
    async fn refresh_thumbnails(&self) -> Result<Vec<ScreenShareSource>, CallError>;
}
```

### Mock vs Real Implementations

For development and testing, mock enumerators are provided:

- `MockDeviceEnumerator` - Returns placeholder devices
- `MockScreenSourceEnumerator` - Returns mock monitors/windows
- `NoDeviceEnumerator` / `NoScreenSourceEnumerator` - For headless operation

Production hosts (Tauri/Dioxus) provide real implementations using:
- macOS: CoreAudio, AVFoundation, ScreenCaptureKit
- Windows: WASAPI, Media Foundation
- Linux: PulseAudio, PipeWire

## Quality Metrics

The service tracks real-time quality metrics:

```rust
pub struct QualityMetrics {
    pub round_trip_time_ms: u32,      // Latency
    pub jitter_ms: u32,               // Variation in latency
    pub packet_loss_percent: f32,     // Lost packets
    pub bitrate_kbps: u32,            // Current bandwidth
    pub connection_quality: ConnectionQuality,
}

pub enum ConnectionQuality {
    Excellent,  // RTT < 50ms, loss < 1%
    Good,       // RTT < 150ms, loss < 3%
    Fair,       // RTT < 300ms, loss < 5%
    Poor,       // Above thresholds
    Unknown,    // Insufficient data
}
```

## Call History

Calls are automatically recorded in persistent storage:

```rust
pub struct CallHistoryEntry {
    pub id: String,
    pub call_type: CallType,       // Voice, Video, Group
    pub outcome: CallOutcome,      // Completed, Missed, Declined, Failed
    pub participants: Vec<HistoryParticipant>,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub duration_secs: Option<u64>,
    // ...
}
```

History supports:
- Filtering by entity, contact, or call type
- Marking as read/unread
- Call-back tracking for missed calls
- Export and persistence

## Event Handling

The service processes events from the core in a background task:

```rust
// Key events handled:
Event::CallStarted { call_id, participants }
Event::ParticipantJoined { call_id, participant }
Event::ParticipantLeft { call_id, participant_id }
Event::CallEnded { call_id, reason }
Event::CallReconnected { call_id }
Event::QualityChanged { call_id, metrics }
Event::ScreenShareStarted { call_id }
Event::ScreenShareStopped { call_id }
```

## Security Considerations

- All signaling messages are encrypted via the gossip overlay
- Media streams use DTLS-SRTP encryption
- Participant identity verified via four-word addresses
- Recording requires explicit user consent
- Screen sharing shows active indicator

## Testing

The call service includes comprehensive test coverage:

- 79 unit tests covering all features
- Integration tests for end-to-end flows
- Mock enumerators enable headless testing

```bash
# Run call tests
cargo test -p communitas-ui-service call::

# Run integration tests
cargo test -p communitas-ui-service --test call_integration
```

## Future Enhancements

- [ ] Virtual backgrounds
- [ ] Noise cancellation settings
- [ ] Breakout rooms for large calls
- [ ] Call transcription
- [ ] WebRTC statistics dashboard
