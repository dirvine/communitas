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
use saorsa_webrtc_core::signaling::{SignalingHandler, SignalingMessage};
use saorsa_webrtc_core::types::{CallEvent, CallId, MediaConstraints};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, info, warn};

/// Active call state
#[derive(Debug, Clone)]
pub struct CallState {
    /// Call ID
    pub call_id: CallId,
    /// Target peer identity
    pub target: CommunitasIdentity,
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
    /// SDP offer
    pub sdp_offer: String,
    /// Media constraints from the offer
    pub has_video: bool,
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

        // Generate SDP offer
        let sdp_offer = self
            .call_manager
            .create_offer(call_id)
            .await
            .map_err(|e| anyhow!("Failed to create SDP offer: {}", e))?;

        debug!("Created SDP offer for call {}", call_id);

        // Create session ID for signaling (using call_id as session_id)
        let session_id = call_id.to_string();

        // Send SDP offer via signaling transport
        let offer_message = SignalingMessage::Offer {
            session_id: session_id.clone(),
            sdp: sdp_offer,
            quic_endpoint: None, // QUIC endpoint discovery handled by gossip
        };

        self.signaling_handler
            .send_message(&target, offer_message)
            .await
            .map_err(|e| anyhow!("Failed to send SDP offer: {}", e))?;

        info!("Sent SDP offer to {} for call {}", target, call_id);

        // Create call state for tracking
        let call_state = CallState {
            call_id,
            target: target.clone(),
            constraints: constraints.clone(),
            is_video_enabled: constraints.has_video(),
            is_audio_enabled: constraints.has_audio(),
            is_screen_sharing: false,
            started_at: std::time::SystemTime::now(),
        };

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

            // Send SDP answer back to caller
            let answer_message = SignalingMessage::Answer {
                session_id: session_id.clone(),
                sdp: info.sdp_offer.clone(), // In real impl, create actual answer SDP
                quic_endpoint: None,
            };

            self.signaling_handler
                .send_message(&info.caller, answer_message)
                .await
                .map_err(|e| anyhow!("Failed to send SDP answer: {}", e))?;

            // Create and store call state
            let call_state = CallState {
                call_id,
                target: info.caller.clone(),
                constraints: constraints.clone(),
                is_video_enabled: constraints.has_video(),
                is_audio_enabled: constraints.has_audio(),
                is_screen_sharing: false,
                started_at: std::time::SystemTime::now(),
            };

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
                "Call {} accepted, sent SDP answer to {}",
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
    /// control will be handled by the Swift layer using AVFoundation.
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
    /// control will be handled by the Swift layer using AVFoundation.
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
    /// On macOS, this will be handled by Swift using ScreenCaptureKit.
    /// On other platforms, platform-specific screen capture APIs are required.
    /// This method updates the call state; the actual screen capture track
    /// will be added by the Swift layer through the media stream manager.
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

        // Note: The actual screen capture track is added by the Swift layer
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
    /// Note: Screen capture is handled by the platform layer (Swift/ScreenCaptureKit).
    /// This method updates the call state; the actual screen capture track
    /// will be removed by the Swift layer through the media stream manager.
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

        // Note: The actual screen capture track is removed by the Swift layer.
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

    /// Get call participants for an active call
    pub async fn get_call_participants(&self, call_id: CallId) -> Result<Vec<CommunitasIdentity>> {
        let calls = self.active_calls.read().await;
        let call = calls
            .get(&call_id)
            .ok_or_else(|| anyhow!("Call not found"))?;

        Ok(vec![self.local_identity.clone(), call.target.clone()])
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
}
