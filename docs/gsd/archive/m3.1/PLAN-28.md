# PLAN-28: Phase 2.2 — CallService Integration

**Milestone**: M3.1 Remediation
**Phase**: 2.2 (CallService WebRTC)
**Status**: Pending
**Created**: 2026-01-20
**Depends on**: PLAN-27 (DriveService Complete)

---

## Overview

Wire `communitas-ui-service/src/call.rs` to `saorsa-webrtc-core` for real device enumeration, call management, and media controls. The CommunitasApp already has WebRTC Commands (StartCall, JoinCall, LeaveCall, ToggleVideo, ToggleAudio, etc.).

## Prerequisites

- [x] CommunitasApp WebRTC Commands exist (command.rs lines 375-398)
- [x] saorsa-webrtc-core = "0.2.1" available in workspace
- [ ] PLAN-27 complete (DriveService pattern established)

---

## Tasks

<task type="auto" priority="p1">
  <n>Add CommunitasApp to CallService constructor</n>
  <files>
    communitas-ui-service/src/call.rs,
    communitas-ui-service/src/lib.rs
  </files>
  <action>
    1. Add `app: Arc<CommunitasApp>` field to CallService struct
    2. Update CallService::new() to accept `app` parameter
    3. Update UiServices::new() in lib.rs to pass app to CallService
    4. Add `pub fn app(&self) -> Arc<CommunitasApp>` accessor
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo build -p communitas-ui-service
  </verify>
  <done>
    - CallService holds Arc<CommunitasApp>
    - UiServices wires app to CallService
    - Compiles without errors
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement device enumeration with saorsa-webrtc-core</n>
  <files>
    communitas-ui-service/src/call.rs,
    communitas-ui-service/Cargo.toml
  </files>
  <action>
    1. Add saorsa-webrtc-core dependency to communitas-ui-service
    2. Replace mock get_audio_devices() with real enumeration
    3. Replace mock get_video_devices() with real enumeration
    4. Convert saorsa-webrtc Device types to UI MediaDevice type
    5. Handle device unavailability gracefully
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Real audio devices returned (microphones, speakers)
    - Real video devices returned (cameras)
    - Default device identification works
    - Unavailable devices marked correctly
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement start_call with Command::StartCall</n>
  <files>
    communitas-ui-service/src/call.rs
  </files>
  <action>
    1. Replace mock start_call() implementation
    2. Build Command::StartCall { entity_id, video_enabled }
    3. Execute via app.execute()
    4. Handle CallStarted event
    5. Update call state in watch channel
    6. Store active call_id
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - start_call initiates real call via CommunitasApp
    - Call state updated in watch channel
    - call_id tracked for subsequent operations
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement join_call and leave_call</n>
  <files>
    communitas-ui-service/src/call.rs
  </files>
  <action>
    1. Replace mock join_call() with Command::JoinCall
    2. Replace mock leave_call() with Command::LeaveCall
    3. Handle CallJoined/CallLeft events
    4. Update participant list in watch channel
    5. Clean up call state on leave
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - join_call connects to existing call
    - leave_call disconnects cleanly
    - Watch channel reflects participant changes
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement audio/video toggles</n>
  <files>
    communitas-ui-service/src/call.rs
  </files>
  <action>
    1. Replace mock toggle_mute() with Command::ToggleAudio
    2. Replace mock toggle_video() with Command::ToggleVideo
    3. Handle AudioToggled/VideoToggled events
    4. Update local call state
    5. Ensure UI reflects current mute/video state
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Mute/unmute works via CommunitasApp
    - Video enable/disable works
    - UI state matches actual media state
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement screen sharing</n>
  <files>
    communitas-ui-service/src/call.rs
  </files>
  <action>
    1. Replace mock start_screen_share() with Command::StartScreenShare
    2. Replace mock stop_screen_share() with Command::StopScreenShare
    3. Handle ScreenShareStarted/ScreenShareStopped events
    4. Update watch channel with screen share state
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Screen sharing starts/stops via CommunitasApp
    - Watch channel reflects screen share state
  </done>
</task>

<task type="auto" priority="p1">
  <n>Implement call queries (status, participants)</n>
  <files>
    communitas-ui-service/src/call.rs
  </files>
  <action>
    1. Implement get_call_status() using Query::GetCallStatus
    2. Implement get_participants() using Query::GetCallParticipants
    3. Implement list_active_calls() using Query::ListActiveCalls
    4. Convert response types to UI types
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Call status reflects real state
    - Participant list accurate
    - Active calls list works
  </done>
</task>

<task type="auto" priority="p1">
  <n>Handle media errors and device changes</n>
  <files>
    communitas-ui-service/src/call.rs
  </files>
  <action>
    1. Add error handling for device access failures
    2. Handle device disconnection during call
    3. Implement device change detection (hot-plug)
    4. Report errors through watch channel
  </action>
  <verify>
    cargo fmt --all -- --check
    cargo clippy -p communitas-ui-service --all-features -- -D warnings
    cargo test -p communitas-ui-service
  </verify>
  <done>
    - Device access errors reported clearly
    - Device disconnection handled gracefully
    - UI notified of device changes
  </done>
</task>

<task type="auto" priority="p1">
  <n>Add integration tests for CallService</n>
  <files>
    communitas-ui-service/tests/call_integration.rs
  </files>
  <action>
    1. Create integration test file
    2. Test device enumeration (may need mocking for CI)
    3. Test call lifecycle: start -> join -> toggle -> leave
    4. Test error handling for invalid call_id
    5. Test watch channel updates
  </action>
  <verify>
    cargo test -p communitas-ui-service --test call_integration
  </verify>
  <done>
    - Integration tests pass
    - Call state machine verified
    - Error cases covered
  </done>
</task>

---

## Exit Criteria

- [ ] Real device enumeration from saorsa-webrtc-core
- [ ] All call Commands wired through CommunitasApp
- [ ] Watch channel updates on call state changes
- [ ] Media error reporting works
- [ ] Integration tests pass

---

## Notes

- Device enumeration may require platform-specific handling
- CI tests may need to skip actual device access
- Consider device permission prompts on macOS/Windows

---

## Next

PLAN-29: Canvas Collaboration (Hook into saorsa-canvas)
