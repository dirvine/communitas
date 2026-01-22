# Voice and Video Calls Guide

This guide explains how to use voice and video calling in Communitas, including one-on-one calls, group calls, screen sharing, and call management features.

## Overview

Communitas provides secure, peer-to-peer voice and video calls with:

- **End-to-end encryption** via DTLS-SRTP
- **No central servers** - calls connect directly between participants
- **Identity verification** via four-word addresses
- **Call history** with search and filtering
- **Quality metrics** for connection monitoring

## Starting a Call

### One-on-One Calls

1. Navigate to a contact's profile or conversation
2. Click the **Phone** icon for voice call or **Video** icon for video call
3. Wait for the recipient to answer

**Tip**: You can upgrade a voice call to video during the call by clicking the video button.

### Group Calls

1. Open a group or channel conversation
2. Click the **Call** button in the group header
3. Select call type (voice or video)
4. Group members receive a notification and can join

**Note**: Group calls support multiple participants. New members can join an active call at any time.

## Joining a Call

When someone calls you:

1. A **Ringing** notification appears with the caller's identity
2. Click **Accept** to join or **Decline** to reject
3. For group calls, you can join any active call from the group conversation

### Missed Calls

If you miss a call:

- A **missed call badge** appears on the conversation
- Missed calls are listed in **Call History** with the option to call back
- Click the badge to acknowledge and clear the notification

## In-Call Controls

During a call, you have access to these controls:

| Control | Description |
|---------|-------------|
| **Mute** | Toggle your microphone on/off |
| **Video** | Toggle your camera on/off |
| **Screen Share** | Share your screen or a specific window |
| **Record** | Start/stop call recording (requires consent) |
| **Participants** | View who's on the call |
| **Quality** | View connection quality metrics |
| **End Call** | Leave the call |

### Audio Controls

- **Mute/Unmute**: Click the microphone icon to toggle your audio
- **Speaker Selection**: Access audio settings to choose output device
- **Microphone Selection**: Access audio settings to choose input device

### Video Controls

- **Camera Toggle**: Click the video icon to turn camera on/off
- **Camera Selection**: Access video settings to switch cameras
- **Video Quality**: Adjusts automatically based on connection

## Screen Sharing

### Starting Screen Share

1. Click the **Screen Share** icon during a call
2. Select what to share:
   - **Entire Screen**: Share everything visible on a monitor
   - **Application Window**: Share a specific app window
3. Click **Share** to begin

### Screen Share Options

| Option | Description |
|--------|-------------|
| **Include Audio** | Share system audio (if supported) |
| **Allow Control** | Let others control your screen (requires confirmation) |

### Stopping Screen Share

- Click the **Stop Sharing** button in the call controls
- Or click the **Screen Share** icon again to toggle off

**Security Note**: A visible indicator shows when screen sharing is active. You cannot silently share your screen.

## Call Quality

### Quality Indicators

Communitas displays connection quality in real-time:

| Quality | Indicator | Meaning |
|---------|-----------|---------|
| **Excellent** | Green | RTT < 50ms, packet loss < 1% |
| **Good** | Light Green | RTT < 150ms, packet loss < 3% |
| **Fair** | Yellow | RTT < 300ms, packet loss < 5% |
| **Poor** | Red | Above thresholds |

### Quality Metrics

Click the quality indicator to see detailed metrics:

- **Round-trip time (RTT)**: Latency in milliseconds
- **Jitter**: Variation in latency
- **Packet loss**: Percentage of lost data
- **Bitrate**: Current bandwidth usage

### Troubleshooting Poor Quality

1. **Check your network** - Use a stable connection, prefer wired over WiFi
2. **Close other apps** - Reduce bandwidth competition
3. **Lower video quality** - Switch to audio-only if needed
4. **Move closer to router** - If using WiFi
5. **Restart the call** - Reconnects may establish a better path

## Call Recording

### Starting a Recording

1. Click the **Record** button during a call
2. All participants see a recording indicator
3. Recording begins after all participants consent (if required)

### Recording Consent

- Recording notifications are shown to all participants
- In some configurations, explicit consent may be required
- Recording can be paused and resumed

### Recording Storage

Recordings are saved to your local storage:
- Format: Audio (WebM/Opus) or Video (WebM/VP8)
- Location: Your entity's virtual disk under `/recordings/`
- Files are encrypted with your entity keys

## Call History

### Viewing History

Access call history from the main menu:

1. Click **Calls** in the navigation
2. View all past calls with details:
   - Call type (voice/video)
   - Participants
   - Duration
   - Outcome (completed, missed, declined)

### Filtering History

Filter calls by:
- **Type**: Voice, video, or group calls
- **Entity**: Specific contact or group
- **Outcome**: Completed, missed, or declined
- **Date Range**: Recent calls or specific periods

### Call Back

From call history, click any entry to:
- View call details
- Call back the same participant(s)
- Message the participant(s)

## Presence Indicators

During calls, presence status updates automatically:

| Status | Meaning |
|--------|---------|
| **In Call** | Currently on a voice or video call |
| **Screen Sharing** | Sharing screen (visible to contacts) |
| **Recording** | Recording in progress |

Contacts see your in-call status and know not to interrupt.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `M` | Toggle mute |
| `V` | Toggle video |
| `S` | Toggle screen share |
| `H` | End/hang up call |
| `R` | Toggle recording |
| `Space` | Push-to-talk (when muted) |

## Security Considerations

### Encryption

All calls are encrypted:
- **Signaling**: Encrypted via gossip overlay
- **Media**: DTLS-SRTP encryption
- **Keys**: Ephemeral, derived per-call

### Identity Verification

- Participants identified by four-word addresses
- You can verify identities out-of-band if needed
- Unknown callers show verification warnings

### Recording Security

- Recording requires explicit UI action
- All participants see recording indicators
- Recordings are encrypted at rest

## Troubleshooting

### "Cannot connect to peer"

- Check network connectivity
- Verify the recipient is online
- Try reconnecting after a few seconds

### "No audio from participant"

- Check the participant isn't muted
- Verify your speaker selection
- Try leaving and rejoining the call

### "Video not showing"

- Check camera permissions
- Verify camera selection in settings
- Try toggling video off and on

### "Poor connection quality"

- See Quality Troubleshooting section above
- Consider switching to audio-only
- Reconnect to establish a new path

## See Also

- [WebRTC Architecture](../architecture/webrtc-multimedia.md) - Technical architecture details
- [MCP Call Tools](../api/mcp-api.md#callwebrtc-tools) - API for automation
- [Identity Recovery](recovery.md) - Recovering your identity
