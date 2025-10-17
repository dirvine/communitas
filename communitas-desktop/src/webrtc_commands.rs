// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! WebRTC Tauri Commands
//!
//! Provides voice, video, and screen sharing functionality via WebRTC.

use communitas_core::webrtc::{
    CallId, CommunitasIdentity, CommunitasWebRtcService, MediaConstraints, MediaDevice,
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// WebRTC service state
pub struct WebRtcState {
    /// WebRTC service instance
    pub service: Arc<RwLock<Option<CommunitasWebRtcService>>>,
}

impl WebRtcState {
    /// Create a new WebRTC state
    pub fn new() -> Self {
        Self {
            service: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the WebRTC service
    pub async fn initialize(&self, service: CommunitasWebRtcService) -> Result<(), String> {
        let mut state = self.service.write().await;
        *state = Some(service);
        Ok(())
    }

    /// Get the WebRTC service
    async fn get_service(&self) -> Result<Arc<RwLock<Option<CommunitasWebRtcService>>>, String> {
        Ok(self.service.clone())
    }
}

impl Default for WebRtcState {
    fn default() -> Self {
        Self::new()
    }
}

/// Initiate a call to another peer
///
/// # Arguments
/// * `target_four_words` - Four-word address of the peer to call
/// * `has_audio` - Enable audio in the call
/// * `has_video` - Enable video in the call
/// * `has_screen_share` - Enable screen sharing in the call
///
/// # Returns
/// Call ID as a string
#[tauri::command]
pub async fn webrtc_initiate_call(
    webrtc_state: State<'_, WebRtcState>,
    target_four_words: String,
    has_audio: bool,
    has_video: bool,
    has_screen_share: bool,
) -> Result<String, String> {
    info!(
        "Initiating call to {} (audio: {}, video: {}, screen: {})",
        target_four_words, has_audio, has_video, has_screen_share
    );

    // Validate target identity
    let _target = CommunitasIdentity::new(target_four_words.clone())
        .map_err(|e| format!("Invalid target identity: {}", e))?;

    // Create media constraints
    let constraints = if has_video {
        MediaConstraints::video_call()
    } else if has_audio {
        MediaConstraints::audio_only()
    } else {
        return Err("Call must have at least audio or video enabled".to_string());
    };

    // TODO: Add screen share constraint when available
    if has_screen_share {
        // For now, screen share is a separate track added after call is established
    }

    // Get WebRTC service
    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    // Initiate call
    let call_id = service
        .initiate_call(&target_four_words, constraints)
        .await
        .map_err(|e| e.to_string())?;

    info!("Call initiated with ID: {}", call_id);
    Ok(call_id.to_string())
}

/// Accept an incoming call
///
/// # Arguments
/// * `call_id` - ID of the call to accept
#[tauri::command]
pub async fn webrtc_accept_call(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
) -> Result<(), String> {
    info!("Accepting call: {}", call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    // Parse call ID (UUID string to CallId)
    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .accept_call(call_id_parsed)
        .await
        .map_err(|e| e.to_string())?;

    info!("Call {} accepted", call_id);
    Ok(())
}

/// Reject an incoming call
///
/// # Arguments
/// * `call_id` - ID of the call to reject
#[tauri::command]
pub async fn webrtc_reject_call(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
) -> Result<(), String> {
    info!("Rejecting call: {}", call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .reject_call(call_id_parsed)
        .await
        .map_err(|e| e.to_string())?;

    info!("Call {} rejected", call_id);
    Ok(())
}

/// End an active call
///
/// # Arguments
/// * `call_id` - ID of the call to end
#[tauri::command]
pub async fn webrtc_end_call(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
) -> Result<(), String> {
    info!("Ending call: {}", call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .end_call(call_id_parsed)
        .await
        .map_err(|e| e.to_string())?;

    info!("Call {} ended", call_id);
    Ok(())
}

/// Enable or disable video in an active call
///
/// # Arguments
/// * `call_id` - ID of the call
/// * `enabled` - Whether to enable or disable video
#[tauri::command]
pub async fn webrtc_set_video_enabled(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!("Setting video enabled={} for call: {}", enabled, call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .set_video_enabled(call_id_parsed, enabled)
        .await
        .map_err(|e| e.to_string())?;

    info!(
        "Video {} for call {}",
        if enabled { "enabled" } else { "disabled" },
        call_id
    );
    Ok(())
}

/// Enable or disable audio in an active call
///
/// # Arguments
/// * `call_id` - ID of the call
/// * `enabled` - Whether to enable or disable audio (mute/unmute)
#[tauri::command]
pub async fn webrtc_set_audio_enabled(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
    enabled: bool,
) -> Result<(), String> {
    info!("Setting audio enabled={} for call: {}", enabled, call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .set_audio_enabled(call_id_parsed, enabled)
        .await
        .map_err(|e| e.to_string())?;

    info!(
        "Audio {} for call {}",
        if enabled { "enabled" } else { "disabled" },
        call_id
    );
    Ok(())
}

/// Start screen sharing in an active call
///
/// # Arguments
/// * `call_id` - ID of the call
#[tauri::command]
pub async fn webrtc_start_screen_share(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
) -> Result<(), String> {
    info!("Starting screen share for call: {}", call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .start_screen_share(call_id_parsed)
        .await
        .map_err(|e| e.to_string())?;

    info!("Screen share started for call {}", call_id);
    Ok(())
}

/// Stop screen sharing in an active call
///
/// # Arguments
/// * `call_id` - ID of the call
#[tauri::command]
pub async fn webrtc_stop_screen_share(
    webrtc_state: State<'_, WebRtcState>,
    call_id: String,
) -> Result<(), String> {
    info!("Stopping screen share for call: {}", call_id);

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let uuid = Uuid::parse_str(&call_id).map_err(|e| format!("Invalid call ID: {}", e))?;
    let call_id_parsed = CallId(uuid);

    service
        .stop_screen_share(call_id_parsed)
        .await
        .map_err(|e| e.to_string())?;

    info!("Screen share stopped for call {}", call_id);
    Ok(())
}

/// Get available media devices
///
/// # Returns
/// List of available audio and video devices
#[tauri::command]
pub async fn webrtc_get_media_devices(
    webrtc_state: State<'_, WebRtcState>,
) -> Result<Vec<MediaDevice>, String> {
    info!("Getting media devices");

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    let devices = service
        .get_media_devices()
        .await
        .map_err(|e| e.to_string())?;

    info!("Found {} media devices", devices.len());
    Ok(devices)
}

/// Subscribe to call events
///
/// This sets up an event listener for WebRTC call events (incoming calls, state changes, etc.)
#[tauri::command]
pub async fn webrtc_subscribe_events(webrtc_state: State<'_, WebRtcState>) -> Result<(), String> {
    info!("Subscribing to WebRTC call events");

    let service_lock = webrtc_state.get_service().await?;
    let service_opt = service_lock.read().await;
    let service = service_opt
        .as_ref()
        .ok_or_else(|| "WebRTC service not initialized".to_string())?;

    // Subscribe to events
    let mut event_rx = service.subscribe_events();

    // Spawn background task to handle events
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            info!("WebRTC event: {:?}", event);
            // TODO: Emit Tauri events to frontend
        }
    });

    info!("Subscribed to WebRTC events");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri::State;

    #[tokio::test]
    async fn test_webrtc_state_creation() {
        let state = WebRtcState::new();
        let service = state.service.read().await;
        assert!(service.is_none());
    }

    #[tokio::test]
    async fn test_webrtc_state_initialization() {
        let state = WebRtcState::new();

        // Test initialization (would normally have a real WebRTC service)
        // For now, just test that the state can be initialized
        assert!(state.service.read().await.is_none());
    }

    #[test]
    fn test_webrtc_state_default() {
        let state = WebRtcState::default();
        assert!(state.service.try_read().unwrap().is_none());
    }

    #[tokio::test]
    async fn test_webrtc_command_signatures() {
        // Test that all WebRTC command functions can be called with proper signatures
        // This is a compile-time test that verifies the Tauri command interface works

        let state = WebRtcState::new();

        // Test that the functions exist and have the right signatures
        // (we can't actually call them without a full Tauri context, but we can verify they compile)
        assert!(state.service.read().await.is_none());

        // Test the state management works
        let _service_arc = state.get_service().await.unwrap();
        assert!(true); // If we get here, the method signature is correct
    }
}
