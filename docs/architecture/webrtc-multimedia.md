# WebRTC Multimedia Architecture

**Status**: Early / Experimental

This document outlines the current WebRTC integration in Communitas and the boundaries between the Rust core and the Dioxus/Tauri host layer.

## Overview

- **Signaling**: Gossip-based signaling (`GossipSignalingTransport`)
- **Identity**: Four-word addresses wrapped as `PeerIdentity`
- **Transport**: QUIC via `saorsa-webrtc-core`
- **Media Capture**: Implemented by the platform host layer (Dioxus/Tauri desktop/mobile)

## Current Implementation (Core)

- Call setup, teardown, and state tracking in `communitas-core/src/webrtc`
- Signaling messages exchanged over the gossip overlay
- Device discovery and actual audio/video/screen capture are delegated to the host layer

## Host Layer Responsibilities

- Audio capture / playback
- Video capture / rendering
- Screen sharing (platform APIs such as ScreenCaptureKit on macOS)
- UI for permissions, device selection, and in-call controls

## Limitations

- Multi-device and multi-party call flows are not fully validated yet
- Screen sharing and device controls are no-ops in the core without host support

## Next Steps

- Validate multi-device scenarios across platforms
- Harden signaling error paths
- Add end-to-end test coverage for call setup/teardown
