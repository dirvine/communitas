//! WebRTC Integration Tests for Communitas
//!
//! These tests verify the WebRTC call infrastructure at the core level,
//! including call state management, identity handling, media constraints,
//! and group call topology.
//!
//! ## Test Categories
//!
//! - **Call State**: Direct and group call state management
//! - **Identity**: Four-word identity handling for WebRTC
//! - **Constraints**: Media constraint validation
//! - **Topology**: Mesh topology for group calls
//!
//! ## Notes on Full Integration Tests
//!
//! Full WebRTC integration (signaling, SDP exchange, peer connections) requires:
//! - Active gossip network for signaling transport
//! - Platform-specific media device access
//! - Multiple peer instances for multi-party calls
//!
//! These scenarios are tested in `communitas-ui-service/tests/call_integration.rs`
//! through the CallService interface, which handles the graceful degradation when
//! networking is unavailable.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::generate_id_words;
use communitas_core::webrtc::{CallId, CommunitasCallState, CommunitasIdentity, MediaConstraints};
// Import PeerIdentity trait for unique_id() method
use saorsa_webrtc_core::identity::PeerIdentity;

// =====================================================
// Identity Tests
// =====================================================

#[test]
fn test_communitas_identity_from_four_words() {
    let identity = CommunitasIdentity::new("ocean-forest-moon-star".to_string())
        .expect("Should create identity from valid four words");

    assert_eq!(identity.four_words(), "ocean-forest-moon-star");
    assert!(!identity.is_placeholder());
}

#[test]
fn test_communitas_identity_placeholder() {
    let placeholder = CommunitasIdentity::placeholder();

    assert!(placeholder.is_placeholder());
    assert_eq!(placeholder.four_words(), "group-call-room-host");
}

#[test]
fn test_communitas_identity_unique_id() {
    // Generate two different valid identities
    let words1 = generate_id_words().unwrap();
    let words2 = generate_id_words().unwrap();

    let id1 = CommunitasIdentity::new(words1.clone()).unwrap();
    let id2 = CommunitasIdentity::new(words2).unwrap();

    assert_ne!(id1.unique_id(), id2.unique_id());

    // Same words should give same unique_id
    let id1_clone = CommunitasIdentity::new(words1).unwrap();
    assert_eq!(id1.unique_id(), id1_clone.unique_id());
}

#[test]
fn test_communitas_identity_equality() {
    // Generate two different valid identities
    let words1 = generate_id_words().unwrap();
    let words2 = generate_id_words().unwrap();

    let id1 = CommunitasIdentity::new(words1.clone()).unwrap();
    let id2 = CommunitasIdentity::new(words1).unwrap();
    let id3 = CommunitasIdentity::new(words2).unwrap();

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

// =====================================================
// Media Constraints Tests
// =====================================================

#[test]
fn test_media_constraints_audio_only() {
    let constraints = MediaConstraints::audio_only();

    assert!(constraints.has_audio());
    assert!(!constraints.has_video());
}

#[test]
fn test_media_constraints_video_call() {
    let constraints = MediaConstraints::video_call();

    assert!(constraints.has_audio());
    assert!(constraints.has_video());
}

// =====================================================
// Call ID Tests
// =====================================================

#[test]
fn test_call_id_uniqueness() {
    let id1 = CallId::new();
    let id2 = CallId::new();

    assert_ne!(id1, id2, "Call IDs should be unique");
}

#[test]
fn test_call_id_display() {
    let id = CallId::new();
    let display = format!("{id}");

    assert!(!display.is_empty(), "CallId should have a display format");
}

// =====================================================
// Direct Call State Tests
// =====================================================

#[test]
fn test_direct_call_state_creation() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::new("ocean-forest-moon-star".to_string()).unwrap();
    let constraints = MediaConstraints::audio_only();

    let state = CommunitasCallState::new_direct(call_id, target.clone(), constraints);

    assert_eq!(state.call_id, call_id);
    assert_eq!(state.target, target);
    assert!(!state.is_group_call());
    assert!(state.entity_id.is_none());
    assert_eq!(state.max_participants, 2);
}

#[test]
fn test_direct_call_peer_count() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let constraints = MediaConstraints::audio_only();

    let state = CommunitasCallState::new_direct(call_id, target, constraints);

    assert_eq!(state.peer_count(), 1, "Direct call should have 1 peer");
}

#[test]
fn test_direct_call_cannot_add_participants() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let constraints = MediaConstraints::audio_only();

    let state = CommunitasCallState::new_direct(call_id, target, constraints);

    assert!(
        !state.can_add_participant(),
        "Direct calls should not allow adding participants"
    );
}

#[test]
fn test_direct_call_get_all_peers() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::new("ocean-forest-moon-star".to_string()).unwrap();
    let constraints = MediaConstraints::audio_only();

    let state = CommunitasCallState::new_direct(call_id, target.clone(), constraints);

    let peers = state.get_all_peers();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0], target);
}

#[test]
fn test_direct_call_initial_media_state() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();

    // Audio only call
    let audio_state =
        CommunitasCallState::new_direct(call_id, target.clone(), MediaConstraints::audio_only());
    assert!(audio_state.is_audio_enabled);
    assert!(!audio_state.is_video_enabled);
    assert!(!audio_state.is_screen_sharing);

    // Video call
    let video_state =
        CommunitasCallState::new_direct(CallId::new(), target, MediaConstraints::video_call());
    assert!(video_state.is_audio_enabled);
    assert!(video_state.is_video_enabled);
    assert!(!video_state.is_screen_sharing);
}

// =====================================================
// Group Call State Tests
// =====================================================

#[test]
fn test_group_call_state_creation() {
    let call_id = CallId::new();
    let entity_id = "test-channel".to_string();
    let constraints = MediaConstraints::audio_only();

    let state = CommunitasCallState::new_group(call_id, entity_id.clone(), constraints, 4);

    assert_eq!(state.call_id, call_id);
    assert!(state.is_group_call());
    assert_eq!(state.entity_id, Some(entity_id));
    assert_eq!(state.max_participants, 4);
    assert!(state.target.is_placeholder());
}

#[test]
fn test_group_call_max_participants_capped_at_4() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();

    // Request 100 participants, but mesh topology caps at 4
    let state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 100);

    assert_eq!(
        state.max_participants, 4,
        "Group call max participants should be capped at 4 for mesh topology"
    );
}

#[test]
fn test_group_call_add_peer() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();
    let mut state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 4);

    let peer1 = CommunitasIdentity::new("ocean-forest-moon-star".to_string()).unwrap();

    assert!(state.add_peer(peer1.clone()), "Should add first peer");
    assert_eq!(state.peer_count(), 1);
}

#[test]
fn test_group_call_prevent_duplicate_peer() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();
    let mut state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 4);

    let peer = CommunitasIdentity::new("ocean-forest-moon-star".to_string()).unwrap();

    assert!(state.add_peer(peer.clone()), "First add should succeed");
    assert!(!state.add_peer(peer), "Duplicate add should fail");
    assert_eq!(state.peer_count(), 1, "Should still have only 1 peer");
}

#[test]
fn test_group_call_remove_peer() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();
    let mut state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 4);

    let peer = CommunitasIdentity::new("ocean-forest-moon-star".to_string()).unwrap();

    state.add_peer(peer.clone());
    assert_eq!(state.peer_count(), 1);

    assert!(state.remove_peer(&peer), "Should remove existing peer");
    assert_eq!(state.peer_count(), 0);

    assert!(
        !state.remove_peer(&peer),
        "Removing non-existent peer should fail"
    );
}

#[test]
fn test_group_call_mark_peer_connected() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();
    let mut state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 4);

    let peer = CommunitasIdentity::new("ocean-forest-moon-star".to_string()).unwrap();
    state.add_peer(peer.clone());

    // Initially not connected
    assert!(!state.peer_connections[0].is_connected);
    assert!(state.peer_connections[0].connected_at.is_none());

    // Mark as connected
    state.mark_peer_connected(&peer);

    assert!(state.peer_connections[0].is_connected);
    assert!(state.peer_connections[0].connected_at.is_some());
}

#[test]
fn test_group_call_capacity() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();
    let mut state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 3);

    // Generate valid identities for all three peers
    let peer1 = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    let peer2 = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    let peer3 = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();

    // Max participants is 3, so we can add 2 peers (self + 2 peers = 3)
    assert!(state.can_add_participant());
    assert!(state.add_peer(peer1));

    assert!(state.can_add_participant());
    assert!(state.add_peer(peer2));

    // Now at capacity (self + 2 = 3)
    assert!(!state.can_add_participant(), "Should be at capacity");
    assert!(
        !state.add_peer(peer3),
        "Should not add peer when at capacity"
    );
}

#[test]
fn test_group_call_get_all_peers() {
    let call_id = CallId::new();
    let constraints = MediaConstraints::audio_only();
    let mut state = CommunitasCallState::new_group(call_id, "test".to_string(), constraints, 4);

    // Generate valid identities
    let peer1 = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    let peer2 = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();

    state.add_peer(peer1.clone());
    state.add_peer(peer2.clone());

    let peers = state.get_all_peers();
    assert_eq!(peers.len(), 2);
    assert!(peers.contains(&peer1));
    assert!(peers.contains(&peer2));
}

// =====================================================
// Media State Tests
// =====================================================

#[test]
fn test_call_state_video_toggle() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let mut state =
        CommunitasCallState::new_direct(call_id, target, MediaConstraints::video_call());

    // Start with video enabled
    assert!(state.is_video_enabled);

    // Disable video
    state.is_video_enabled = false;
    assert!(!state.is_video_enabled);

    // Re-enable video
    state.is_video_enabled = true;
    assert!(state.is_video_enabled);
}

#[test]
fn test_call_state_audio_toggle() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let mut state =
        CommunitasCallState::new_direct(call_id, target, MediaConstraints::audio_only());

    // Start with audio enabled
    assert!(state.is_audio_enabled);

    // Mute
    state.is_audio_enabled = false;
    assert!(!state.is_audio_enabled);

    // Unmute
    state.is_audio_enabled = true;
    assert!(state.is_audio_enabled);
}

#[test]
fn test_call_state_screen_share_toggle() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let mut state =
        CommunitasCallState::new_direct(call_id, target, MediaConstraints::audio_only());

    // Start without screen sharing
    assert!(!state.is_screen_sharing);

    // Start screen share
    state.is_screen_sharing = true;
    assert!(state.is_screen_sharing);

    // Stop screen share
    state.is_screen_sharing = false;
    assert!(!state.is_screen_sharing);
}

// =====================================================
// Timestamp Tests
// =====================================================

#[test]
fn test_call_state_started_at_is_set() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let state = CommunitasCallState::new_direct(call_id, target, MediaConstraints::audio_only());

    // started_at should be set to approximately now
    let now = std::time::SystemTime::now();
    let elapsed = now
        .duration_since(state.started_at)
        .expect("started_at should be in the past");

    assert!(
        elapsed.as_secs() < 1,
        "started_at should be within 1 second of now"
    );
}

// =====================================================
// Topology Tests
// =====================================================

#[test]
fn test_call_topology_default_is_direct() {
    use communitas_core::webrtc::service::CallTopology;

    assert_eq!(
        CallTopology::default(),
        CallTopology::Direct,
        "Default topology should be Direct"
    );
}

#[test]
fn test_direct_call_topology() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let state = CommunitasCallState::new_direct(call_id, target, MediaConstraints::audio_only());

    assert!(
        !state.is_group_call(),
        "Direct call should not be a group call"
    );
}

#[test]
fn test_group_call_mesh_topology() {
    let call_id = CallId::new();
    let state = CommunitasCallState::new_group(
        call_id,
        "test".to_string(),
        MediaConstraints::audio_only(),
        4,
    );

    assert!(
        state.is_group_call(),
        "Group call should be identified as group call"
    );
}

// =====================================================
// Entity Association Tests
// =====================================================

#[test]
fn test_direct_call_has_no_entity() {
    let call_id = CallId::new();
    let target = CommunitasIdentity::placeholder();
    let state = CommunitasCallState::new_direct(call_id, target, MediaConstraints::audio_only());

    assert!(
        state.entity_id.is_none(),
        "Direct call should not have an entity ID"
    );
}

#[test]
fn test_group_call_has_entity() {
    let call_id = CallId::new();
    let entity_id = "channel-abc123".to_string();
    let state = CommunitasCallState::new_group(
        call_id,
        entity_id.clone(),
        MediaConstraints::audio_only(),
        4,
    );

    assert_eq!(
        state.entity_id,
        Some(entity_id),
        "Group call should have entity ID"
    );
}

// =====================================================
// Integration Scenario Tests
// =====================================================

/// Test a typical 1:1 call flow at the state level.
#[test]
fn test_direct_call_flow_scenario() {
    let call_id = CallId::new();
    let callee = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    let constraints = MediaConstraints::video_call();

    // 1. Create call state (caller initiates)
    let mut state = CommunitasCallState::new_direct(call_id, callee.clone(), constraints);
    assert!(state.is_video_enabled);
    assert!(state.is_audio_enabled);

    // 2. During call - mute audio
    state.is_audio_enabled = false;
    assert!(!state.is_audio_enabled);
    assert!(state.is_video_enabled);

    // 3. During call - disable video
    state.is_video_enabled = false;
    assert!(!state.is_video_enabled);

    // 4. Screen share
    state.is_screen_sharing = true;
    assert!(state.is_screen_sharing);

    // 5. Stop screen share and unmute
    state.is_screen_sharing = false;
    state.is_audio_enabled = true;
    assert!(!state.is_screen_sharing);
    assert!(state.is_audio_enabled);
}

/// Test a typical group call flow at the state level.
#[test]
fn test_group_call_flow_scenario() {
    let call_id = CallId::new();
    let entity_id = "team-standup-channel".to_string();
    let constraints = MediaConstraints::video_call();

    // 1. Host creates group call
    let mut state = CommunitasCallState::new_group(call_id, entity_id, constraints, 4);
    assert!(state.is_group_call());
    assert_eq!(state.peer_count(), 0);

    // 2. Participants join (using valid generated identities)
    let alice = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    let bob = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    let charlie = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();

    assert!(state.add_peer(alice.clone()));
    assert!(state.add_peer(bob.clone()));
    assert!(state.add_peer(charlie.clone()));

    assert_eq!(state.peer_count(), 3);
    assert!(!state.can_add_participant()); // At capacity (self + 3 = 4)

    // 3. Peer connections established
    state.mark_peer_connected(&alice);
    state.mark_peer_connected(&bob);
    assert!(state.peer_connections[0].is_connected);
    assert!(state.peer_connections[1].is_connected);
    assert!(!state.peer_connections[2].is_connected); // Charlie not yet connected

    // 4. Bob leaves
    assert!(state.remove_peer(&bob));
    assert_eq!(state.peer_count(), 2);
    assert!(state.can_add_participant()); // Now has room again

    // 5. New participant joins
    let dave = CommunitasIdentity::new(generate_id_words().unwrap()).unwrap();
    assert!(state.add_peer(dave.clone()));
    assert_eq!(state.peer_count(), 3);
}
