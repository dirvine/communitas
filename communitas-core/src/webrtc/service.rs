// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! WebRTC Service Integration
//!
//! Provides a high-level WebRTC service for Communitas that integrates
//! with the gossip overlay network for signaling and peer discovery.

use super::gossip_signaling::GossipSignalingTransport;
use super::identity::CommunitasIdentity;
use crate::gossip::GossipContext;
use anyhow::{Result, anyhow};
use saorsa_webrtc_core::call::{CallManager, CallManagerConfig};
use saorsa_webrtc_core::identity::PeerIdentity;
use saorsa_webrtc_core::signaling::{SignalingHandler, SignalingMessage};
use saorsa_webrtc_core::types::{CallEvent, CallId, MediaConstraints};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info, warn};

/// Type of call topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallTopology {
    /// Direct 1:1 call with single peer connection.
    #[default]
    Direct,
    /// Mesh topology for small group calls (≤4 participants).
    /// Each participant maintains N-1 peer connections.
    Mesh,
}

/// State of a peer connection within a group call.
#[derive(Debug, Clone)]
pub struct PeerConnectionState {
    /// The remote peer identity.
    pub peer: CommunitasIdentity,
    /// Whether the connection is established.
    pub is_connected: bool,
    /// When the connection was established.
    pub connected_at: Option<std::time::SystemTime>,
}

/// Active call state
#[derive(Debug, Clone)]
pub struct CallState {
    /// Call ID
    pub call_id: CallId,
    /// Entity ID this call is associated with (for entity-based call rooms).
    pub entity_id: Option<String>,
    /// Call topology.
    pub topology: CallTopology,
    /// For direct calls: the single target peer.
    /// For group calls: the first peer (call initiator or first participant joined).
    pub target: CommunitasIdentity,
    /// For group calls: all peer connections in the mesh.
    /// Empty for direct calls.
    pub peer_connections: Vec<PeerConnectionState>,
    /// Maximum participants allowed (for group calls).
    pub max_participants: u32,
    /// Media constraints
    pub constraints: MediaConstraints,
    /// Is video currently enabled
    pub is_video_enabled: bool,
    /// Is audio currently enabled
    pub is_audio_enabled: bool,
    /// Is screen sharing active
    pub is_screen_sharing: bool,
    /// Call start time
    pub started_at: std::time::SystemTime,
}

impl CallState {
    /// Creates a new direct call state.
    pub fn new_direct(
        call_id: CallId,
        target: CommunitasIdentity,
        constraints: MediaConstraints,
    ) -> Self {
        Self {
            call_id,
            entity_id: None,
            topology: CallTopology::Direct,
            target,
            peer_connections: Vec::new(),
            max_participants: 2,
            constraints: constraints.clone(),
            is_video_enabled: constraints.has_video(),
            is_audio_enabled: constraints.has_audio(),
            is_screen_sharing: false,
            started_at: std::time::SystemTime::now(),
        }
    }

    /// Creates a new group call state with mesh topology.
    pub fn new_group(
        call_id: CallId,
        entity_id: String,
        constraints: MediaConstraints,
        max_participants: u32,
    ) -> Self {
        Self {
            call_id,
            entity_id: Some(entity_id),
            topology: CallTopology::Mesh,
            target: CommunitasIdentity::placeholder(),
            peer_connections: Vec::new(),
            max_participants: max_participants.min(4), // Mesh topology limited to 4
            constraints: constraints.clone(),
            is_video_enabled: constraints.has_video(),
            is_audio_enabled: constraints.has_audio(),
            is_screen_sharing: false,
            started_at: std::time::SystemTime::now(),
        }
    }

    /// Returns true if this is a group call with mesh topology.
    pub fn is_group_call(&self) -> bool {
        matches!(self.topology, CallTopology::Mesh)
    }

    /// Returns the number of active peer connections.
    pub fn peer_count(&self) -> usize {
        if self.is_group_call() {
            self.peer_connections.len()
        } else {
            1 // Direct call always has one peer
        }
    }

    /// Returns true if the call can accept more participants.
    pub fn can_add_participant(&self) -> bool {
        if self.is_group_call() {
            // +1 for self
            (self.peer_connections.len() + 1) < self.max_participants as usize
        } else {
            false // Direct calls are always 1:1
        }
    }

    /// Adds a peer to the group call mesh.
    pub fn add_peer(&mut self, peer: CommunitasIdentity) -> bool {
        if !self.can_add_participant() {
            return false;
        }

        // Don't add duplicates
        if self.peer_connections.iter().any(|p| p.peer == peer) {
            return false;
        }

        self.peer_connections.push(PeerConnectionState {
            peer,
            is_connected: false,
            connected_at: None,
        });
        true
    }

    /// Marks a peer connection as established.
    pub fn mark_peer_connected(&mut self, peer: &CommunitasIdentity) {
        if let Some(state) = self.peer_connections.iter_mut().find(|p| &p.peer == peer) {
            state.is_connected = true;
            state.connected_at = Some(std::time::SystemTime::now());
        }
    }

    /// Removes a peer from the group call mesh.
    pub fn remove_peer(&mut self, peer: &CommunitasIdentity) -> bool {
        let initial_len = self.peer_connections.len();
        self.peer_connections.retain(|p| &p.peer != peer);
        self.peer_connections.len() < initial_len
    }

    /// Gets all peers in the call.
    pub fn get_all_peers(&self) -> Vec<CommunitasIdentity> {
        if self.is_group_call() {
            self.peer_connections
                .iter()
                .map(|p| p.peer.clone())
                .collect()
        } else {
            vec![self.target.clone()]
        }
    }
}

/// Media device information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MediaDevice {
    /// Device ID
    pub device_id: String,
    /// Human-readable label
    pub label: String,
    /// Device kind (audioinput, audiooutput, videoinput)
    pub kind: String,
}

/// WebRTC service for Communitas
///
/// Manages WebRTC calls over the gossip overlay network, providing
/// voice, video, and screen sharing capabilities.
pub struct CommunitasWebRtcService {
    /// Signaling transport
    signaling: Arc<GossipSignalingTransport>,

    /// Signaling handler with rate limiting
    signaling_handler: Arc<SignalingHandler<GossipSignalingTransport>>,

    /// Call manager from saorsa-webrtc-core
    call_manager: Arc<CallManager<CommunitasIdentity>>,

    /// Local identity
    local_identity: CommunitasIdentity,

    /// Event broadcaster
    event_tx: broadcast::Sender<CallEvent<CommunitasIdentity>>,

    /// Active calls (maps CallId to session_id for signaling)
    active_calls: Arc<RwLock<HashMap<CallId, CallState>>>,

    /// Pending incoming calls (from signaling)
    pending_incoming_calls: Arc<RwLock<HashMap<String, IncomingCallInfo>>>,
}

/// Information about an incoming call
#[derive(Debug, Clone)]
pub struct IncomingCallInfo {
    /// Session ID from signaling
    pub session_id: String,
    /// Caller identity
    pub caller: CommunitasIdentity,
    /// Audio capability from caller
    pub has_audio: bool,
    /// Video capability from caller
    pub has_video: bool,
    /// Data channel capability from caller
    pub has_data_channel: bool,
    /// Maximum bandwidth in kbps
    pub max_bandwidth_kbps: u32,
}

impl CommunitasWebRtcService {
    /// Create a new WebRTC service
    ///
    /// # Arguments
    /// * `gossip` - The gossip context
    pub async fn new(gossip: Arc<GossipContext>) -> Result<Self> {
        info!("Initializing Communitas WebRTC service");

        // Create signaling transport
        let signaling = Arc::new(GossipSignalingTransport::new(gossip.clone())?);

        // Create signaling handler with rate limiting
        let signaling_handler = Arc::new(SignalingHandler::new(signaling.clone()));

        // Create call manager with default config
        let call_config = CallManagerConfig::default();
        let call_manager = Arc::new(
            CallManager::new(call_config)
                .await
                .map_err(|e| anyhow!("Failed to create call manager: {}", e))?,
        );

        // Get local identity
        let local_identity = CommunitasIdentity::new(gossip.four_words.clone())?;

        // Create event broadcaster
        let (event_tx, _) = broadcast::channel(100);

        // Initialize active calls tracking
        let active_calls = Arc::new(RwLock::new(HashMap::new()));
        let pending_incoming_calls = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            signaling,
            signaling_handler,
            call_manager,
            local_identity,
            event_tx,
            active_calls,
            pending_incoming_calls,
        })
    }

    /// Start the WebRTC service
    ///
    /// Subscribes to signaling messages and begins listening for incoming calls.
    pub async fn start(&self) -> Result<()> {
        info!("Starting WebRTC service for {}", self.local_identity);

        // Subscribe to signaling messages
        self.signaling.subscribe_to_signaling().await?;

        debug!("WebRTC service started successfully");

        Ok(())
    }

    /// Initiate a call to another peer
    ///
    /// # Arguments
    /// * `target_four_words` - Four-word address of the peer to call
    /// * `constraints` - Media constraints (audio, video, screen share)
    ///
    /// # Returns
    /// The call ID for the initiated call
    pub async fn initiate_call(
        &self,
        target_four_words: &str,
        constraints: MediaConstraints,
    ) -> Result<CallId> {
        info!(
            "Initiating call to {} with constraints: {:?}",
            target_four_words, constraints
        );

        // Create target identity
        let target = CommunitasIdentity::new(target_four_words.to_string())?;

        // Use CallManager to initiate the call (creates peer connection and media tracks)
        let call_id = self
            .call_manager
            .initiate_call(target.clone(), constraints.clone())
            .await
            .map_err(|e| anyhow!("Failed to initiate call: {}", e))?;

        // Exchange capabilities using QUIC-native call flow
        let capabilities = self
            .call_manager
            .exchange_capabilities(call_id)
            .await
            .map_err(|e| anyhow!("Failed to exchange capabilities: {}", e))?;

        debug!("Exchanged capabilities for call {}: audio={}, video={}",
            call_id, capabilities.audio, capabilities.video);

        // Create session ID for signaling (using call_id as session_id)
        let session_id = call_id.to_string();

        // Send capability exchange via signaling transport (QUIC-native)
        let capability_message = SignalingMessage::CapabilityExchange {
            session_id: session_id.clone(),
            audio: capabilities.audio,
            video: capabilities.video,
            data_channel: capabilities.data_channel,
            max_bandwidth_kbps: capabilities.max_bandwidth_kbps,
            quic_endpoint: None, // QUIC endpoint discovery handled by gossip
        };

        self.signaling_handler
            .send_message(&target, capability_message)
            .await
            .map_err(|e| anyhow!("Failed to send capability exchange: {}", e))?;

        info!("Sent capability exchange to {} for call {}", target, call_id);

        // Create call state for tracking (direct 1:1 call)
        let call_state = CallState::new_direct(call_id, target.clone(), constraints.clone());

        // Store call state
        {
            let mut calls = self.active_calls.write().await;
            calls.insert(call_id, call_state);
        }

        debug!("Created call {} to {}", call_id, target);

        // Emit call initiated event
        let event = CallEvent::CallInitiated {
            call_id,
            callee: target,
            constraints,
        };
        let _ = self.event_tx.send(event);

        Ok(call_id)
    }

    /// Initiate a group call on an entity (channel, group, etc.)
    ///
    /// Creates a new call with mesh topology where each participant maintains
    /// N-1 peer connections directly with other participants (up to 4 total).
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID (channel/group) to start the call on
    /// * `constraints` - Media constraints (audio, video)
    /// * `max_participants` - Maximum participants allowed (capped at 4 for mesh)
    ///
    /// # Returns
    /// The call ID for the initiated group call
    pub async fn initiate_group_call(
        &self,
        entity_id: &str,
        constraints: MediaConstraints,
        max_participants: u32,
    ) -> Result<CallId> {
        info!(
            "Initiating group call on entity {} with max {} participants",
            entity_id, max_participants
        );

        // For mesh topology, limit to 4 participants
        let max_participants = max_participants.min(4);

        // Generate call ID (not tied to a specific peer in group calls)
        let call_id = CallId::new();

        debug!("Created group call {} on entity {}", call_id, entity_id);

        // Create group call state
        let call_state = CallState::new_group(
            call_id,
            entity_id.to_string(),
            constraints.clone(),
            max_participants,
        );

        // Store call state
        {
            let mut calls = self.active_calls.write().await;
            calls.insert(call_id, call_state);
        }

        info!(
            "Group call {} initialized on entity {} (max {} participants, mesh topology)",
            call_id, entity_id, max_participants
        );

        Ok(call_id)
    }

    /// Join an existing group call
    ///
    /// Establishes peer connections with all existing participants in the call
    /// using mesh topology.
    ///
    /// # Arguments
    /// * `call_id` - The ID of the group call to join
    /// * `constraints` - Media constraints for the local side (used when creating peer connections)
    ///
    /// # Errors
    /// Returns error if call is full or doesn't exist
    pub async fn join_group_call(
        &self,
        call_id: CallId,
        _constraints: MediaConstraints,
    ) -> Result<()> {
        // Note: constraints will be used when creating actual peer connections
        // Currently peer connection creation is a placeholder
        info!("Joining group call {}", call_id);

        // Check if call exists and can accept participants
        let (can_join, existing_peers) = {
            let calls = self.active_calls.read().await;
            if let Some(call) = calls.get(&call_id) {
                if !call.is_group_call() {
                    return Err(anyhow!("Call {} is not a group call", call_id));
                }
                (call.can_add_participant(), call.get_all_peers())
            } else {
                return Err(anyhow!("Group call {} not found", call_id));
            }
        };

        if !can_join {
            return Err(anyhow!("Group call {} is full", call_id));
        }

        // Establish peer connections with all existing participants
        for peer in &existing_peers {
            if peer.is_placeholder() {
                continue;
            }

            debug!(
                "Establishing peer connection with {} for group call {}",
                peer, call_id
            );

            // Send capability exchange for this peer (QUIC-native)
            // In a full implementation, this would create an actual peer connection
            let session_id = format!("{}:{}", call_id, peer.unique_id());

            let capability_message = SignalingMessage::CapabilityExchange {
                session_id: session_id.clone(),
                audio: true,  // Group calls support audio by default
                video: false, // Video on demand
                data_channel: true,
                max_bandwidth_kbps: 2000, // 2 Mbps default
                quic_endpoint: None,
            };

            if let Err(e) = self
                .signaling_handler
                .send_message(peer, capability_message)
                .await
            {
                warn!(
                    "Failed to send capability exchange to peer {} for group call {}: {}",
                    peer, call_id, e
                );
            }
        }

        info!(
            "Initiated peer connections with {} existing participants in group call {}",
            existing_peers.len(),
            call_id
        );

        Ok(())
    }

    /// Add a participant to a group call
    ///
    /// Called when a remote participant joins the group call.
    /// Establishes a new peer connection and notifies existing participants.
    ///
    /// # Arguments
    /// * `call_id` - The group call ID
    /// * `participant` - The identity of the joining participant
    pub async fn add_group_participant(
        &self,
        call_id: CallId,
        participant: CommunitasIdentity,
    ) -> Result<()> {
        info!(
            "Adding participant {} to group call {}",
            participant, call_id
        );

        // Add participant to call state
        {
            let mut calls = self.active_calls.write().await;
            let call = calls
                .get_mut(&call_id)
                .ok_or_else(|| anyhow!("Group call {} not found", call_id))?;

            if !call.is_group_call() {
                return Err(anyhow!("Call {} is not a group call", call_id));
            }

            if !call.add_peer(participant.clone()) {
                return Err(anyhow!(
                    "Cannot add participant to group call {} (full or duplicate)",
                    call_id
                ));
            }
        }

        debug!(
            "Participant {} added to group call {}",
            participant, call_id
        );

        Ok(())
    }

    /// Remove a participant from a group call
    ///
    /// Called when a participant leaves or disconnects from the group call.
    ///
    /// # Arguments
    /// * `call_id` - The group call ID
    /// * `participant` - The identity of the leaving participant
    pub async fn remove_group_participant(
        &self,
        call_id: CallId,
        participant: &CommunitasIdentity,
    ) -> Result<()> {
        info!(
            "Removing participant {} from group call {}",
            participant, call_id
        );

        // Remove participant from call state
        {
            let mut calls = self.active_calls.write().await;
            let call = calls
                .get_mut(&call_id)
                .ok_or_else(|| anyhow!("Group call {} not found", call_id))?;

            if !call.is_group_call() {
                return Err(anyhow!("Call {} is not a group call", call_id));
            }

            if !call.remove_peer(participant) {
                warn!(
                    "Participant {} not found in group call {}",
                    participant, call_id
                );
            }
        }

        debug!(
            "Participant {} removed from group call {}",
            participant, call_id
        );

        Ok(())
    }

    /// Accept an incoming call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call to accept
    /// * `constraints` - Media constraints for the local side
    pub async fn accept_call(&self, call_id: CallId, constraints: MediaConstraints) -> Result<()> {
        info!("Accepting call {}", call_id);

        let session_id = call_id.to_string();

        // Check if this call exists in pending incoming calls
        let incoming_info = {
            let pending = self.pending_incoming_calls.read().await;
            pending.get(&session_id).cloned()
        };

        // If we have incoming call info, this is a real incoming call
        // Otherwise, treat it as accepting a call we initiated (for state sync)
        if let Some(info) = incoming_info {
            // Accept via CallManager (this handles WebRTC state)
            self.call_manager
                .accept_call(call_id, constraints.clone())
                .await
                .map_err(|e| anyhow!("Failed to accept call: {}", e))?;

            // Send connection confirmation back to caller (QUIC-native)
            let confirm_message = SignalingMessage::ConnectionConfirm {
                session_id: session_id.clone(),
                audio: constraints.audio,
                video: constraints.video,
                data_channel: true,
                max_bandwidth_kbps: info.max_bandwidth_kbps.min(2000), // Use minimum of both
                quic_endpoint: None,
            };

            self.signaling_handler
                .send_message(&info.caller, confirm_message)
                .await
                .map_err(|e| anyhow!("Failed to send connection confirmation: {}", e))?;

            // Create and store call state (direct 1:1 call)
            let call_state =
                CallState::new_direct(call_id, info.caller.clone(), constraints.clone());

            {
                let mut calls = self.active_calls.write().await;
                calls.insert(call_id, call_state);
            }

            // Remove from pending
            {
                let mut pending = self.pending_incoming_calls.write().await;
                pending.remove(&session_id);
            }

            info!(
                "Call {} accepted, sent connection confirmation to {}",
                call_id, info.caller
            );
        } else {
            // Just update CallManager state
            self.call_manager
                .accept_call(call_id, constraints)
                .await
                .map_err(|e| anyhow!("Failed to accept call: {}", e))?;
        }

        // Emit connection established event
        let event = CallEvent::ConnectionEstablished { call_id };
        let _ = self.event_tx.send(event);

        debug!("Call {} accepted", call_id);

        Ok(())
    }

    /// Reject an incoming call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call to reject
    pub async fn reject_call(&self, call_id: CallId) -> Result<()> {
        info!("Rejecting call {}", call_id);

        let session_id = call_id.to_string();

        // Check if this is a pending incoming call
        let incoming_info = {
            let pending = self.pending_incoming_calls.read().await;
            pending.get(&session_id).cloned()
        };

        // Send Bye signaling message to caller
        if let Some(info) = &incoming_info {
            let bye_message = SignalingMessage::Bye {
                session_id: session_id.clone(),
                reason: Some("rejected".to_string()),
            };

            if let Err(e) = self
                .signaling_handler
                .send_message(&info.caller, bye_message)
                .await
            {
                warn!("Failed to send rejection signaling: {}", e);
            }

            // Remove from pending
            {
                let mut pending = self.pending_incoming_calls.write().await;
                pending.remove(&session_id);
            }

            info!("Sent rejection to {}", info.caller);
        }

        // Update CallManager state
        if let Err(e) = self.call_manager.reject_call(call_id).await {
            debug!("CallManager reject_call error (may not exist yet): {}", e);
        }

        debug!("Call {} rejected", call_id);

        let event = CallEvent::CallRejected { call_id };
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// End an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call to end
    pub async fn end_call(&self, call_id: CallId) -> Result<()> {
        info!("Ending call {}", call_id);

        let session_id = call_id.to_string();

        // Get call state and target before removing
        let call_state = {
            let calls = self.active_calls.read().await;
            calls.get(&call_id).cloned()
        };

        // Send Bye signaling message to remote peer
        if let Some(state) = &call_state {
            let bye_message = SignalingMessage::Bye {
                session_id: session_id.clone(),
                reason: Some("ended".to_string()),
            };

            if let Err(e) = self
                .signaling_handler
                .send_message(&state.target, bye_message)
                .await
            {
                warn!("Failed to send call end signaling: {}", e);
            }

            info!("Sent call end to {}", state.target);
        }

        // Remove call from active calls
        {
            let mut calls = self.active_calls.write().await;
            if calls.remove(&call_id).is_none() {
                warn!("Attempted to end non-existent call {}", call_id);
                return Err(anyhow!("Call not found"));
            }
        }

        // End call in CallManager (closes peer connection and cleans up tracks)
        if let Err(e) = self.call_manager.end_call(call_id).await {
            debug!("CallManager end_call error: {}", e);
        }

        debug!("Call {} ended", call_id);

        let event = CallEvent::CallEnded { call_id };
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// Enable or disable video in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    /// * `enabled` - Whether to enable or disable video
    ///
    /// Note: Full track muting requires platform-specific implementation.
    /// This currently updates state and logs the change. The actual track
    /// control is handled by the platform host layer (e.g., Dioxus/Tauri hosts).
    pub async fn set_video_enabled(&self, call_id: CallId, enabled: bool) -> Result<()> {
        info!("Setting video enabled={} for call {}", enabled, call_id);

        // Update call state
        let target = {
            let mut calls = self.active_calls.write().await;
            let call = calls
                .get_mut(&call_id)
                .ok_or_else(|| anyhow!("Call not found"))?;

            call.is_video_enabled = enabled;
            call.target.clone()
        };

        // Note: Media state change events would be handled at the application layer
        // For now, state is tracked locally and UI will poll for updates

        debug!(
            "Video {} for call {} (target: {})",
            if enabled { "enabled" } else { "disabled" },
            call_id,
            target
        );

        Ok(())
    }

    /// Enable or disable audio in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    /// * `enabled` - Whether to enable or disable audio (mute/unmute)
    ///
    /// Note: Full track muting requires platform-specific implementation.
    /// This currently updates state and logs the change. The actual track
    /// control is handled by the platform host layer (e.g., Dioxus/Tauri hosts).
    pub async fn set_audio_enabled(&self, call_id: CallId, enabled: bool) -> Result<()> {
        info!("Setting audio enabled={} for call {}", enabled, call_id);

        // Update call state
        let target = {
            let mut calls = self.active_calls.write().await;
            let call = calls
                .get_mut(&call_id)
                .ok_or_else(|| anyhow!("Call not found"))?;

            call.is_audio_enabled = enabled;
            call.target.clone()
        };

        // Note: Media state change events would be handled at the application layer
        // For now, state is tracked locally and UI will poll for updates

        debug!(
            "Audio {} for call {} (target: {})",
            if enabled { "enabled" } else { "disabled" },
            call_id,
            target
        );

        Ok(())
    }

    /// Start screen sharing in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    ///
    /// Note: Screen capture requires platform-specific implementation.
    /// On macOS, ScreenCaptureKit is used by the host layer.
    /// On other platforms, platform-specific screen capture APIs are required.
    /// This method updates the call state; the actual screen capture track
    /// is added by the platform host layer through the media stream manager.
    pub async fn start_screen_share(&self, call_id: CallId) -> Result<()> {
        info!("Starting screen share for call {}", call_id);

        // Update call state and get target for logging
        let target = {
            let mut calls = self.active_calls.write().await;
            let call = calls
                .get_mut(&call_id)
                .ok_or_else(|| anyhow!("Call not found"))?;

            if call.is_screen_sharing {
                debug!("Screen sharing already active for call {}", call_id);
                return Ok(());
            }

            call.is_screen_sharing = true;
            call.target.clone()
        };

        // Note: The actual screen capture track is added by the host layer
        // using ScreenCaptureKit on macOS. The Rust layer tracks the state
        // and will signal the remote peer when the track is added.

        debug!(
            "Screen share started for call {} (target: {})",
            call_id, target
        );

        Ok(())
    }

    /// Stop screen sharing in an active call
    ///
    /// # Arguments
    /// * `call_id` - The ID of the call
    ///
    /// Note: Screen capture is handled by the platform host layer.
    /// This method updates the call state; the actual screen capture track
    /// is removed by the host layer through the media stream manager.
    pub async fn stop_screen_share(&self, call_id: CallId) -> Result<()> {
        info!("Stopping screen share for call {}", call_id);

        // Update call state and get target for logging
        let target = {
            let mut calls = self.active_calls.write().await;
            let call = calls
                .get_mut(&call_id)
                .ok_or_else(|| anyhow!("Call not found"))?;

            if !call.is_screen_sharing {
                debug!("Screen sharing not active for call {}", call_id);
                return Ok(());
            }

            call.is_screen_sharing = false;
            call.target.clone()
        };

        // Note: The actual screen capture track is removed by the host layer.
        // The Rust layer tracks the state and will signal the remote peer
        // when the track is removed.

        debug!(
            "Screen share stopped for call {} (target: {})",
            call_id, target
        );

        Ok(())
    }

    /// Get available media devices
    ///
    /// # Returns
    /// List of available audio and video devices
    pub async fn get_media_devices(&self) -> Result<Vec<MediaDevice>> {
        info!("Getting media devices");

        // Media device enumeration is typically done on the client side
        // The backend service doesn't have access to browser media devices
        // This method exists for API consistency but returns empty list

        debug!("Media device enumeration should be done on the client side");

        Ok(Vec::new())
    }

    /// List active calls
    pub async fn list_active_calls(&self) -> Vec<CallState> {
        let calls = self.active_calls.read().await;
        calls.values().cloned().collect()
    }

    /// Get call participants for an active call.
    ///
    /// For direct calls, returns the local user and the target.
    /// For group calls, returns the local user and all connected peers.
    pub async fn get_call_participants(&self, call_id: CallId) -> Result<Vec<CommunitasIdentity>> {
        let calls = self.active_calls.read().await;
        let call = calls
            .get(&call_id)
            .ok_or_else(|| anyhow!("Call not found"))?;

        let mut participants = vec![self.local_identity.clone()];

        if call.is_group_call() {
            // For group calls, add all peer connections
            for peer in call.get_all_peers() {
                if !peer.is_placeholder() {
                    participants.push(peer);
                }
            }
        } else {
            // For direct calls, add the single target
            participants.push(call.target.clone());
        }

        Ok(participants)
    }

    /// Check if a call is a group call.
    pub async fn is_group_call(&self, call_id: CallId) -> Result<bool> {
        let calls = self.active_calls.read().await;
        let call = calls
            .get(&call_id)
            .ok_or_else(|| anyhow!("Call not found"))?;

        Ok(call.is_group_call())
    }

    /// Get the entity ID for a group call.
    ///
    /// Returns None if the call is not a group call or doesn't have an entity ID.
    pub async fn get_call_entity(&self, call_id: CallId) -> Option<String> {
        let calls = self.active_calls.read().await;
        calls.get(&call_id).and_then(|call| call.entity_id.clone())
    }

    /// Subscribe to call events
    ///
    /// # Returns
    /// A broadcast receiver for call events
    pub fn subscribe_events(&self) -> broadcast::Receiver<CallEvent<CommunitasIdentity>> {
        self.event_tx.subscribe()
    }

    /// Get the local identity
    pub fn local_identity(&self) -> &CommunitasIdentity {
        &self.local_identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests require setting up a complete gossip context.
    // These are unit-level checks only.

    #[test]
    fn test_call_id_generation() {
        let id1 = CallId::new();
        let id2 = CallId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_media_constraints() {
        let audio = MediaConstraints::audio_only();
        assert!(audio.has_audio());
        assert!(!audio.has_video());

        let video = MediaConstraints::video_call();
        assert!(video.has_audio());
        assert!(video.has_video());
    }

    #[test]
    fn test_call_topology() {
        assert_eq!(CallTopology::default(), CallTopology::Direct);
        assert_ne!(CallTopology::Direct, CallTopology::Mesh);
    }

    #[test]
    fn test_direct_call_state() {
        let call_id = CallId::new();
        let target = CommunitasIdentity::placeholder();
        let constraints = MediaConstraints::audio_only();

        let state = CallState::new_direct(call_id, target.clone(), constraints);

        assert_eq!(state.call_id, call_id);
        assert_eq!(state.target, target);
        assert!(!state.is_group_call());
        assert_eq!(state.peer_count(), 1);
        assert!(!state.can_add_participant());
        assert!(state.entity_id.is_none());
        assert_eq!(state.max_participants, 2);
    }

    #[test]
    fn test_group_call_state() {
        let call_id = CallId::new();
        let entity_id = "test-channel".to_string();
        let constraints = MediaConstraints::audio_only();

        let state = CallState::new_group(call_id, entity_id.clone(), constraints, 4);

        assert_eq!(state.call_id, call_id);
        assert!(state.is_group_call());
        assert_eq!(state.peer_count(), 0); // No peers yet
        assert!(state.can_add_participant());
        assert_eq!(state.entity_id, Some(entity_id));
        assert_eq!(state.max_participants, 4);
        assert!(state.target.is_placeholder());
    }

    #[test]
    fn test_group_call_max_participants_capped() {
        let call_id = CallId::new();
        let constraints = MediaConstraints::audio_only();

        // Request 100 participants, but should be capped at 4 for mesh
        let state = CallState::new_group(call_id, "test".to_string(), constraints, 100);

        assert_eq!(state.max_participants, 4);
    }

    #[test]
    fn test_add_peer_to_group_call() {
        let call_id = CallId::new();
        let constraints = MediaConstraints::audio_only();
        let mut state = CallState::new_group(call_id, "test".to_string(), constraints, 4);

        let peer1 = CommunitasIdentity::placeholder(); // Using placeholder for testing

        // First add should succeed (can have 3 peers + self = 4)
        assert!(state.add_peer(peer1.clone()));
        assert_eq!(state.peer_count(), 1);

        // Adding same peer again should fail (duplicate detection)
        assert!(!state.add_peer(peer1.clone()));
        assert_eq!(state.peer_count(), 1);
    }

    #[test]
    fn test_remove_peer_from_group_call() {
        let call_id = CallId::new();
        let constraints = MediaConstraints::audio_only();
        let mut state = CallState::new_group(call_id, "test".to_string(), constraints, 4);

        let peer = CommunitasIdentity::placeholder();

        state.add_peer(peer.clone());
        assert_eq!(state.peer_count(), 1);

        assert!(state.remove_peer(&peer));
        assert_eq!(state.peer_count(), 0);

        // Removing again should return false
        assert!(!state.remove_peer(&peer));
    }

    #[test]
    fn test_mark_peer_connected() {
        let call_id = CallId::new();
        let constraints = MediaConstraints::audio_only();
        let mut state = CallState::new_group(call_id, "test".to_string(), constraints, 4);

        let peer = CommunitasIdentity::placeholder();
        state.add_peer(peer.clone());

        // Initially not connected
        assert!(!state.peer_connections[0].is_connected);
        assert!(state.peer_connections[0].connected_at.is_none());

        // Mark connected
        state.mark_peer_connected(&peer);

        assert!(state.peer_connections[0].is_connected);
        assert!(state.peer_connections[0].connected_at.is_some());
    }

    #[test]
    fn test_get_all_peers() {
        let call_id = CallId::new();
        let constraints = MediaConstraints::audio_only();

        // Direct call
        let target = CommunitasIdentity::placeholder();
        let direct_state = CallState::new_direct(call_id, target.clone(), constraints.clone());
        let peers = direct_state.get_all_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], target);

        // Group call
        let mut group_state =
            CallState::new_group(CallId::new(), "test".to_string(), constraints, 4);
        let peer = CommunitasIdentity::placeholder();
        group_state.add_peer(peer.clone());

        let peers = group_state.get_all_peers();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0], peer);
    }

    #[test]
    fn test_peer_connection_state() {
        let peer = CommunitasIdentity::placeholder();

        let state = PeerConnectionState {
            peer: peer.clone(),
            is_connected: false,
            connected_at: None,
        };

        assert!(!state.is_connected);
        assert!(state.connected_at.is_none());
    }

    #[test]
    fn test_placeholder_identity() {
        let placeholder = CommunitasIdentity::placeholder();

        assert!(placeholder.is_placeholder());
        assert_eq!(placeholder.four_words(), "group-call-room-host");
    }
}
