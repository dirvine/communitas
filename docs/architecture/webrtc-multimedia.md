# WebRTC Multimedia Architecture

**Status**: Production Ready (Phase 8.1)

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
| **Device Enumeration** | Platform-specific via `DeviceEnumerator` trait (cpal + nokhwa) |
| **Screen Sharing** | Source selection via `ScreenSourceEnumerator` trait (scap) |
| **Quality Metrics** | Real-time RTT, jitter, packet loss tracking |
| **Call History** | Persistent storage with search and filtering |
| **Missed Calls** | Notification system with acknowledgment |
| **Offline Invites** | Queue for call invitations received while offline |
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

### Lazy Initialization

Platform enumerators are initialized lazily to avoid blocking app startup and to handle platform permissions gracefully:

```rust
// In CallService, enumerators start as mocks
device_enumerator: Arc<RwLock<Arc<dyn DeviceEnumerator>>>,
device_enumerator_is_real: AtomicBool,

// UI components trigger lazy initialization on first render
use_effect(move || {
    if !call_service.has_real_device_enumerator() {
        let enumerator = platform::create_device_enumerator();
        call_service.set_device_enumerator(enumerator);
    }
});
```

This pattern ensures:
- Fast app startup (no hardware enumeration blocking)
- Graceful degradation when hardware unavailable
- Platform permission prompts appear in user context
- Tests can run without hardware dependencies

### Camera Enumeration (nokhwa)

Camera devices are enumerated using the `nokhwa` crate, which provides cross-platform webcam access:

| Platform | Backend | Notes |
|----------|---------|-------|
| macOS | AVFoundation | Requires camera permission in Info.plist |
| Windows | Media Foundation | Works out of the box |
| Linux | V4L2 | Requires video group membership |

**Permission Requirements:**

```xml
<!-- macOS: Add to Info.plist -->
<key>NSCameraUsageDescription</key>
<string>Communitas needs camera access for video calls</string>
```

**Device Discovery:**

```rust
// nokhwa provides synchronous enumeration
let cameras = nokhwa::query(nokhwa::utils::ApiBackend::Auto)?;

for camera in cameras {
    MediaDevice {
        id: camera.index().to_string(),
        name: camera.human_name(),
        device_type: DeviceType::Camera,
        is_default: camera.index().as_index() == Some(0),
        is_available: true,
    }
}
```

### Screen Source Enumeration (scap)

Screen and window sources are enumerated using the `scap` crate for cross-platform screen capture:

| Platform | Backend | Notes |
|----------|---------|-------|
| macOS | ScreenCaptureKit | Requires screen recording permission |
| Windows | Windows Graphics Capture | Windows 10 1903+ required |
| Linux | PipeWire | Portal-based permission model |

**Permission Requirements:**

- **macOS**: System Preferences → Privacy & Security → Screen Recording
- **Windows**: No explicit permission needed (app must be in foreground)
- **Linux**: PipeWire portal handles permissions via desktop environment

**Source Discovery:**

```rust
// scap provides async-friendly enumeration
pub async fn enumerate_sources(&self) -> Result<Vec<ScreenShareSource>, CallError> {
    let displays = scap::get_all_displays()?;
    let windows = scap::get_all_windows()?;

    let mut sources = Vec::new();

    // Add monitors
    for (i, display) in displays.iter().enumerate() {
        sources.push(ScreenShareSource::monitor(
            display.id.to_string(),
            display.name.clone(),
            i == 0, // Primary display
        ));
    }

    // Add windows
    for window in windows {
        sources.push(ScreenShareSource::window(
            window.id.to_string(),
            window.title.clone(),
            window.app_name.clone(),
        ));
    }

    Ok(sources)
}
```

### Mock vs Real Implementations

For development and testing, mock enumerators are provided:

- `MockDeviceEnumerator` - Returns placeholder devices
- `MockScreenSourceEnumerator` - Returns mock monitors/windows
- `NoDeviceEnumerator` / `NoScreenSourceEnumerator` - For headless operation

Production hosts (Tauri/Dioxus) provide real implementations using:
- **Audio**: cpal (CoreAudio, WASAPI, ALSA/PulseAudio)
- **Camera**: nokhwa (AVFoundation, Media Foundation, V4L2)
- **Screen**: scap (ScreenCaptureKit, Windows Graphics Capture, PipeWire)

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

## Offline Call Invites

When users are offline or temporarily disconnected, incoming call invitations are queued for later processing:

### Queue Behavior

```rust
pub struct PendingCallInvite {
    pub id: String,
    pub call_id: String,
    pub caller_id: String,
    pub caller_name: String,
    pub entity_id: String,
    pub call_type: CallType,
    pub received_at: i64,
    pub expires_at: i64,
}

// Queue constraints
const MAX_PENDING_INVITES: usize = 10;
const PENDING_INVITE_EXPIRY_MS: i64 = 5 * 60 * 1000; // 5 minutes
```

### Features

| Feature | Description |
|---------|-------------|
| **FIFO Ordering** | Oldest invites processed first |
| **Max Limit** | Queue holds up to 10 invites; oldest dropped when full |
| **Expiration** | Invites expire after 5 minutes |
| **Deduplication** | Same call_id updates existing invite |
| **Reactive Updates** | Watch channel broadcasts queue changes |

### Processing on Reconnect

When the user comes back online, pending invites are automatically processed:

```rust
// Triggered by network reconnection
call_service.process_pending_invites_on_reconnect().await;

// For each valid invite:
// 1. Check if not expired
// 2. Check if call still active
// 3. Show notification to user
// 4. Remove from queue after user action
```

### UI Integration

```rust
// Subscribe to pending invites
let mut rx = call_service.subscribe_pending_invites();

// React to new invites
rx.changed().await?;
let snapshot = rx.borrow().clone();

for invite in &snapshot.invites {
    // Show notification badge or toast
    show_pending_call_notification(invite);
}
```

## Security Considerations

- All signaling messages are encrypted via the gossip overlay
- Media streams use DTLS-SRTP encryption
- Participant identity verified via four-word addresses
- Recording requires explicit user consent
- Screen sharing shows active indicator

## Testing

The call service includes comprehensive test coverage:

- 90+ unit tests covering all features
- Integration tests for device and screen enumeration
- Mock enumerators enable headless CI testing
- Hardware tests available for manual execution

```bash
# Run all call service tests
cargo test -p communitas-ui-service call::

# Run device integration tests
cargo test -p communitas-ui-service --test call_device_integration

# Run platform integration tests (mocks only, CI-safe)
cargo test -p communitas-dioxus --test platform_integration

# Run hardware tests manually (requires devices)
cargo test -p communitas-dioxus --test platform_integration -- --ignored
```

### Test Categories

| Category | Location | CI Safe |
|----------|----------|---------|
| Unit tests | `communitas-ui-service/src/call.rs` | Yes |
| Device integration | `communitas-ui-service/tests/call_device_integration.rs` | Yes |
| Platform mocks | `communitas-dioxus/tests/platform_integration.rs` | Yes |
| Hardware tests | Same file, `#[ignore]` attribute | No (manual) |

## Future Enhancements

- [ ] Virtual backgrounds
- [ ] Noise cancellation settings
- [ ] Breakout rooms for large calls
- [ ] Call transcription
- [ ] WebRTC statistics dashboard
