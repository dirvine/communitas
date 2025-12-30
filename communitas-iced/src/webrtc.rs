// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! WebRTC integration for voice/video calls.
//!
//! This module bridges the saorsa-webrtc-core library with the Iced GUI,
//! handling call initiation, media streams, and participant management.
//!
//! Note: This is currently a mock implementation for UI development.
//! Full WebRTC integration will replace the mock implementations.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use futures::channel::mpsc;
use saorsa_webrtc_core::{CallId, WebRtcConfig};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::message::{CallMessage, Message};
use crate::state::{CallStatus, MediaDevice, MediaDevices};

/// A participant in a call.
#[derive(Debug, Clone)]
pub struct CallParticipant {
    /// Participant's four-word identity.
    pub four_words: String,
    /// Display name (if known).
    pub display_name: Option<String>,
    /// Whether video is enabled.
    pub video_enabled: bool,
    /// Whether audio is enabled.
    pub audio_enabled: bool,
    /// Whether currently speaking.
    pub is_speaking: bool,
}

/// Internal WebRTC events for UI updates.
///
/// These are translated from saorsa-webrtc-core events when full integration is done.
#[derive(Debug, Clone)]
pub enum WebRtcUiEvent {
    /// Call state changed.
    CallStateChanged {
        /// Call identifier.
        call_id: String,
        /// New call state.
        state: CallStatus,
    },
    /// Participant joined the call.
    ParticipantJoined {
        /// Call identifier.
        call_id: String,
        /// Participant's four-word identity.
        peer_id: String,
        /// Whether video is enabled.
        video_enabled: bool,
    },
    /// Participant left the call.
    ParticipantLeft {
        /// Call identifier.
        call_id: String,
        /// Participant's four-word identity.
        peer_id: String,
    },
    /// Participant's media state changed.
    MediaStateChanged {
        /// Call identifier.
        call_id: String,
        /// Participant's four-word identity.
        peer_id: String,
        /// Whether audio is enabled.
        audio_enabled: bool,
        /// Whether video is enabled.
        video_enabled: bool,
    },
    /// An error occurred.
    Error {
        /// Call identifier.
        call_id: String,
        /// Error message.
        message: String,
    },
    /// Call ended.
    CallEnded {
        /// Call identifier.
        call_id: String,
        /// Reason for ending (if known).
        reason: Option<String>,
    },
}

/// WebRTC service manager for the Iced GUI.
pub struct WebRtcManager {
    /// Current call participants.
    participants: Arc<RwLock<HashMap<String, CallParticipant>>>,
    /// Available media devices.
    devices: Arc<RwLock<MediaDevices>>,
    /// Event sender to forward events to the UI.
    event_tx: Option<mpsc::UnboundedSender<Message>>,
    /// Whether the service is initialized.
    initialized: bool,
}

impl Default for WebRtcManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WebRtcManager {
    /// Create a new WebRTC manager (uninitialized).
    #[must_use]
    pub fn new() -> Self {
        Self {
            participants: Arc::new(RwLock::new(HashMap::new())),
            devices: Arc::new(RwLock::new(MediaDevices::default())),
            event_tx: None,
            initialized: false,
        }
    }

    /// Check if the WebRTC service is initialized.
    #[must_use]
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Initialize the WebRTC service.
    ///
    /// This should be called after the network is started.
    pub async fn initialize(&mut self, _config: WebRtcConfig) -> Result<()> {
        info!("Initializing WebRTC service");

        // Note: Full initialization requires signaling transport from the network layer.
        // For now, we mark as initialized but don't create the actual service
        // until the network provides a signaling transport.
        self.initialized = true;

        // Enumerate available devices
        self.refresh_devices().await?;

        info!("WebRTC manager ready (awaiting signaling transport)");
        Ok(())
    }

    /// Set the event sender for forwarding events to the UI.
    pub fn set_event_sender(&mut self, tx: mpsc::UnboundedSender<Message>) {
        self.event_tx = Some(tx);
    }

    /// Refresh the list of available media devices.
    pub async fn refresh_devices(&self) -> Result<()> {
        debug!("Refreshing media device list");

        // In a real implementation, this would query the platform for devices.
        // For now, we provide mock devices for UI development.
        let mut devices = self.devices.write().await;

        // Mock audio input devices
        devices.audio_inputs = vec![
            MediaDevice {
                id: "default-mic".to_string(),
                name: "Default Microphone".to_string(),
                is_default: true,
            },
            MediaDevice {
                id: "external-mic".to_string(),
                name: "External USB Microphone".to_string(),
                is_default: false,
            },
        ];

        // Mock audio output devices
        devices.audio_outputs = vec![
            MediaDevice {
                id: "default-speaker".to_string(),
                name: "Default Speakers".to_string(),
                is_default: true,
            },
            MediaDevice {
                id: "headphones".to_string(),
                name: "Headphones".to_string(),
                is_default: false,
            },
        ];

        // Mock video devices
        devices.video_devices = vec![
            MediaDevice {
                id: "default-camera".to_string(),
                name: "Built-in Camera".to_string(),
                is_default: true,
            },
            MediaDevice {
                id: "external-camera".to_string(),
                name: "External Webcam".to_string(),
                is_default: false,
            },
        ];

        // Select defaults
        devices.selected_audio_input = Some("default-mic".to_string());
        devices.selected_audio_output = Some("default-speaker".to_string());
        devices.selected_video = Some("default-camera".to_string());

        debug!(
            "Found {} audio inputs, {} audio outputs, {} video devices",
            devices.audio_inputs.len(),
            devices.audio_outputs.len(),
            devices.video_devices.len()
        );

        Ok(())
    }

    /// Get the current media devices.
    pub async fn get_devices(&self) -> MediaDevices {
        self.devices.read().await.clone()
    }

    /// Select an audio input device.
    pub async fn select_audio_input(&self, device_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        if devices.audio_inputs.iter().any(|d| d.id == device_id) {
            devices.selected_audio_input = Some(device_id.to_string());
            info!("Selected audio input: {device_id}");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Audio input device not found: {device_id}"))
        }
    }

    /// Select an audio output device.
    pub async fn select_audio_output(&self, device_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        if devices.audio_outputs.iter().any(|d| d.id == device_id) {
            devices.selected_audio_output = Some(device_id.to_string());
            info!("Selected audio output: {device_id}");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Audio output device not found: {device_id}"
            ))
        }
    }

    /// Select a video device.
    pub async fn select_video_device(&self, device_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        if devices.video_devices.iter().any(|d| d.id == device_id) {
            devices.selected_video = Some(device_id.to_string());
            info!("Selected video device: {device_id}");
            Ok(())
        } else {
            Err(anyhow::anyhow!("Video device not found: {device_id}"))
        }
    }

    /// Initiate a call to a peer.
    pub async fn initiate_call(&self, peer_four_words: &str, has_video: bool) -> Result<CallId> {
        info!("Initiating call to {peer_four_words} (video: {has_video})");

        // Generate a call ID
        let call_id = CallId::new();

        // In a real implementation, this would:
        // 1. Create a call offer via WebRtcService
        // 2. Send signaling message to peer via DHT
        // 3. Wait for answer
        // 4. Establish media streams

        // For now, add the peer as a participant (will connect when they answer)
        let mut participants = self.participants.write().await;
        participants.insert(
            peer_four_words.to_string(),
            CallParticipant {
                four_words: peer_four_words.to_string(),
                display_name: None,
                video_enabled: has_video,
                audio_enabled: true,
                is_speaking: false,
            },
        );

        info!("Call {call_id:?} initiated to {peer_four_words}");
        Ok(call_id)
    }

    /// Accept an incoming call.
    pub async fn accept_call(&self, call_id: &CallId) -> Result<()> {
        info!("Accepting call {call_id:?}");

        // In a real implementation:
        // 1. Create answer via WebRtcService
        // 2. Send signaling response
        // 3. Establish media streams

        Ok(())
    }

    /// Reject an incoming call.
    pub async fn reject_call(&self, call_id: &CallId) -> Result<()> {
        info!("Rejecting call {call_id:?}");

        // In a real implementation:
        // 1. Send rejection signaling message
        // 2. Clean up any pending state

        Ok(())
    }

    /// End an active call.
    pub async fn end_call(&self, call_id: &CallId) -> Result<()> {
        info!("Ending call {call_id:?}");

        // Clear participants
        let mut participants = self.participants.write().await;
        participants.clear();

        // In a real implementation:
        // 1. Send hangup signaling message
        // 2. Close media streams
        // 3. Clean up WebRTC peer connection

        Ok(())
    }

    /// Toggle audio mute.
    pub async fn toggle_audio(&self, call_id: &CallId, enabled: bool) -> Result<()> {
        info!("Setting audio to {enabled} for call {call_id:?}");

        // In a real implementation:
        // 1. Mute/unmute audio track
        // 2. Send media state update to peers

        Ok(())
    }

    /// Toggle video.
    pub async fn toggle_video(&self, call_id: &CallId, enabled: bool) -> Result<()> {
        info!("Setting video to {enabled} for call {call_id:?}");

        // In a real implementation:
        // 1. Enable/disable video track
        // 2. Send media state update to peers

        Ok(())
    }

    /// Toggle screen sharing.
    pub async fn toggle_screen_share(&self, call_id: &CallId, enabled: bool) -> Result<()> {
        info!("Setting screen share to {enabled} for call {call_id:?}");

        // In a real implementation:
        // 1. Start/stop screen capture
        // 2. Replace video track with screen share
        // 3. Send media state update to peers

        Ok(())
    }

    /// Get current call participants.
    pub async fn get_participants(&self) -> Vec<CallParticipant> {
        self.participants.read().await.values().cloned().collect()
    }

    /// Handle a UI-facing WebRTC event.
    ///
    /// This method processes events that are translated from the actual
    /// saorsa-webrtc-core events by the integration layer.
    pub async fn handle_event(&self, event: WebRtcUiEvent) {
        match event {
            WebRtcUiEvent::CallStateChanged { call_id, state } => {
                info!("Call {call_id} state changed to {state:?}");
                if let Some(tx) = &self.event_tx {
                    let _ = tx.unbounded_send(Message::Call(CallMessage::StatusChanged(state)));
                }
            }
            WebRtcUiEvent::ParticipantJoined {
                call_id,
                peer_id,
                video_enabled,
            } => {
                info!("Participant {peer_id} joined call {call_id}");
                let mut participants = self.participants.write().await;
                participants.insert(
                    peer_id.clone(),
                    CallParticipant {
                        four_words: peer_id,
                        display_name: None,
                        video_enabled,
                        audio_enabled: true,
                        is_speaking: false,
                    },
                );
            }
            WebRtcUiEvent::ParticipantLeft { call_id, peer_id } => {
                info!("Participant {peer_id} left call {call_id}");
                let mut participants = self.participants.write().await;
                participants.remove(&peer_id);
            }
            WebRtcUiEvent::MediaStateChanged {
                call_id,
                peer_id,
                audio_enabled,
                video_enabled,
            } => {
                debug!("Media state changed for {peer_id} in call {call_id}");
                let mut participants = self.participants.write().await;
                if let Some(p) = participants.get_mut(&peer_id) {
                    p.audio_enabled = audio_enabled;
                    p.video_enabled = video_enabled;
                }
            }
            WebRtcUiEvent::Error { call_id, message } => {
                error!("WebRTC error in call {call_id}: {message}");
                if let Some(tx) = &self.event_tx {
                    let _ = tx.unbounded_send(Message::Call(CallMessage::CallEnded(Some(message))));
                }
            }
            WebRtcUiEvent::CallEnded { call_id, reason } => {
                info!("Call {call_id} ended: {reason:?}");
                let mut participants = self.participants.write().await;
                participants.clear();

                if let Some(tx) = &self.event_tx {
                    let _ = tx.unbounded_send(Message::Call(CallMessage::CallEnded(reason)));
                }
            }
        }
    }

    /// Shutdown the WebRTC service.
    pub async fn shutdown(&mut self) {
        info!("Shutting down WebRTC manager");

        // Clear state
        let mut participants = self.participants.write().await;
        participants.clear();

        self.initialized = false;
    }
}

/// Global WebRTC manager instance.
static WEBRTC_MANAGER: std::sync::OnceLock<tokio::sync::Mutex<WebRtcManager>> =
    std::sync::OnceLock::new();

/// Get the global WebRTC manager.
pub fn get_webrtc_manager() -> &'static tokio::sync::Mutex<WebRtcManager> {
    WEBRTC_MANAGER.get_or_init(|| tokio::sync::Mutex::new(WebRtcManager::new()))
}
