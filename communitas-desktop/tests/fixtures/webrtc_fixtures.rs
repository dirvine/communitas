// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! WebRTC Test Fixtures
//!
//! Provides mock implementations and test utilities for WebRTC command testing.

use communitas_core::webrtc::{CallEvent, CallId, CommunitasIdentity, MediaConstraints};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Mock state for WebRTC service testing
#[derive(Debug, Clone)]
#[cfg(test)]
pub struct MockState {
    /// Active calls indexed by call ID
    pub active_calls: HashMap<String, MockCall>,
    /// Emitted call events
    pub emitted_events: Vec<CallEvent<CommunitasIdentity>>,
    /// Device state
    pub devices: Vec<MockDevice>,
    /// Whether the service is initialized
    pub is_initialized: bool,
    /// Error mode for testing
    pub error_mode: Option<MockError>,
}

impl Default for MockState {
    fn default() -> Self {
        Self {
            active_calls: HashMap::new(),
            emitted_events: Vec::new(),
            devices: create_default_devices(),
            is_initialized: false,
            error_mode: None,
        }
    }
}

/// Mock call representation
#[derive(Debug, Clone)]
pub struct MockCall {
    pub id: String,
    pub target: String,
    pub constraints: MediaConstraints,
    pub is_video_enabled: bool,
    pub is_audio_enabled: bool,
    pub is_screen_sharing: bool,
    pub state: CallState,
}

/// Call state for mock calls
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState {
    Initiating,
    Ringing,
    Active,
    Ended,
    Rejected,
}

/// Mock device for testing
#[derive(Debug, Clone)]
pub struct MockDevice {
    pub device_id: String,
    pub label: String,
    pub kind: DeviceKind,
}

/// Device type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceKind {
    AudioInput,
    AudioOutput,
    VideoInput,
}

/// Simulated error conditions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockError {
    NetworkFailure,
    InvalidCallId,
    DeviceNotFound,
    PermissionDenied,
}

/// Mock WebRTC service for testing
pub struct MockWebRtcService {
    state: Arc<Mutex<MockState>>,
    event_tx: broadcast::Sender<CallEvent<CommunitasIdentity>>,
}

impl MockWebRtcService {
    /// Create a new mock service
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            state: Arc::new(Mutex::new(MockState::default())),
            event_tx,
        }
    }

    /// Initialize the mock service
    pub fn initialize(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        state.is_initialized = true;
        Ok(())
    }

    /// Initiate a mock call
    pub fn initiate_call(
        &self,
        target: String,
        constraints: MediaConstraints,
    ) -> Result<String, String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if !state.is_initialized {
            return Err("Service not initialized".to_string());
        }

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call_id = format!("call_{}", uuid::Uuid::new_v4());
        let mock_call = MockCall {
            id: call_id.clone(),
            target: target.clone(),
            constraints: constraints.clone(),
            is_video_enabled: constraints.has_video(),
            is_audio_enabled: constraints.has_audio(),
            is_screen_sharing: false,
            state: CallState::Initiating,
        };

        state.active_calls.insert(call_id.clone(), mock_call);

        // Emit event
        let target_identity = CommunitasIdentity::new(target)
            .map_err(|e| format!("Invalid target identity: {}", e))?;
        let event = CallEvent::CallInitiated {
            call_id: CallId::new(),
            callee: target_identity,
            constraints,
        };
        state.emitted_events.push(event.clone());
        let _ = self.event_tx.send(event);

        Ok(call_id)
    }

    /// Accept a mock call
    pub fn accept_call(&self, call_id: String) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.state = CallState::Active;
        Ok(())
    }

    /// Reject a mock call
    pub fn reject_call(&self, call_id: String) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.state = CallState::Rejected;

        // Emit event
        let event = CallEvent::CallRejected {
            call_id: CallId::new(),
        };
        state.emitted_events.push(event.clone());
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// End a mock call
    pub fn end_call(&self, call_id: String) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.state = CallState::Ended;

        // Emit event
        let event = CallEvent::CallEnded {
            call_id: CallId::new(),
        };
        state.emitted_events.push(event.clone());
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// Set video enabled/disabled
    pub fn set_video_enabled(&self, call_id: String, enabled: bool) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.is_video_enabled = enabled;
        Ok(())
    }

    /// Set audio enabled/disabled
    pub fn set_audio_enabled(&self, call_id: String, enabled: bool) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.is_audio_enabled = enabled;
        Ok(())
    }

    /// Start screen sharing
    pub fn start_screen_share(&self, call_id: String) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.is_screen_sharing = true;
        Ok(())
    }

    /// Stop screen sharing
    pub fn stop_screen_share(&self, call_id: String) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        let call = state
            .active_calls
            .get_mut(&call_id)
            .ok_or_else(|| "Call not found".to_string())?;

        call.is_screen_sharing = false;
        Ok(())
    }

    /// Get available media devices
    pub fn get_media_devices(&self) -> Result<Vec<MockDevice>, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;

        if let Some(ref error) = state.error_mode {
            return Err(format!("Mock error: {:?}", error));
        }

        Ok(state.devices.clone())
    }

    /// Subscribe to call events
    pub fn subscribe_events(&self) -> broadcast::Receiver<CallEvent<CommunitasIdentity>> {
        self.event_tx.subscribe()
    }

    /// Set error mode for testing error scenarios
    pub fn set_error_mode(&self, error: Option<MockError>) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        state.error_mode = error;
        Ok(())
    }

    /// Get call by ID (for testing)
    pub fn get_call(&self, call_id: &str) -> Result<MockCall, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        state
            .active_calls
            .get(call_id)
            .cloned()
            .ok_or_else(|| "Call not found".to_string())
    }

    /// Get all emitted events (for testing)
    pub fn get_emitted_events(&self) -> Result<Vec<CallEvent<CommunitasIdentity>>, String> {
        let state = self.state.lock().map_err(|e| e.to_string())?;
        Ok(state.emitted_events.clone())
    }

    /// Clear all state (for test cleanup)
    pub fn reset(&self) -> Result<(), String> {
        let mut state = self.state.lock().map_err(|e| e.to_string())?;
        *state = MockState::default();
        Ok(())
    }
}

impl Default for MockWebRtcService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock gossip context for testing
pub struct MockGossipContext {
    identity: String,
    sent_messages: Arc<Mutex<Vec<GossipMessage>>>,
}

impl MockGossipContext {
    /// Create a new mock gossip context
    pub fn new(identity: String) -> Self {
        Self {
            identity,
            sent_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the identity
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Simulate sending a gossip message
    pub fn send_message(&self, message: GossipMessage) -> Result<(), String> {
        let mut messages = self.sent_messages.lock().map_err(|e| e.to_string())?;
        messages.push(message);
        Ok(())
    }

    /// Get all sent messages (for testing)
    pub fn get_sent_messages(&self) -> Result<Vec<GossipMessage>, String> {
        let messages = self.sent_messages.lock().map_err(|e| e.to_string())?;
        Ok(messages.clone())
    }

    /// Clear sent messages (for test cleanup)
    pub fn clear_messages(&self) -> Result<(), String> {
        let mut messages = self.sent_messages.lock().map_err(|e| e.to_string())?;
        messages.clear();
        Ok(())
    }
}

/// Mock gossip message
#[derive(Debug, Clone)]
pub struct GossipMessage {
    pub topic: String,
    pub payload: Vec<u8>,
}

/// WebRTC test fixture
pub struct WebRtcTestFixture {
    pub webrtc_service: MockWebRtcService,
    pub gossip_context: MockGossipContext,
    pub test_identity: String,
}

impl WebRtcTestFixture {
    /// Create a new test fixture with default configuration
    pub fn new() -> Self {
        let test_identity = "ocean-forest-moon-star".to_string();
        let webrtc_service = MockWebRtcService::new();
        let gossip_context = MockGossipContext::new(test_identity.clone());

        Self {
            webrtc_service,
            gossip_context,
            test_identity,
        }
    }

    /// Create a fixture with custom identity
    pub fn with_identity(identity: String) -> Self {
        let webrtc_service = MockWebRtcService::new();
        let gossip_context = MockGossipContext::new(identity.clone());

        Self {
            webrtc_service,
            gossip_context,
            test_identity: identity,
        }
    }

    /// Initialize the fixture
    pub fn initialize(&self) -> Result<(), String> {
        self.webrtc_service.initialize()
    }

    /// Cleanup the fixture
    pub fn cleanup(&self) -> Result<(), String> {
        self.webrtc_service.reset()?;
        self.gossip_context.clear_messages()?;
        Ok(())
    }
}

impl Default for WebRtcTestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Create default mock devices
pub fn create_default_devices() -> Vec<MockDevice> {
    vec![
        MockDevice {
            device_id: "audio_input_1".to_string(),
            label: "Default Microphone".to_string(),
            kind: DeviceKind::AudioInput,
        },
        MockDevice {
            device_id: "audio_output_1".to_string(),
            label: "Default Speakers".to_string(),
            kind: DeviceKind::AudioOutput,
        },
        MockDevice {
            device_id: "video_input_1".to_string(),
            label: "Built-in Camera".to_string(),
            kind: DeviceKind::VideoInput,
        },
    ]
}

/// Test identities for multi-peer scenarios
///
/// Generates valid four-word addresses for testing
#[cfg(test)]
pub fn test_identities() -> Vec<String> {
    vec![
        "ocean-forest-moon-star".to_string(),
        "river-mountain-cloud-tree".to_string(),
        "sunshine-rainbow-breeze-flower".to_string(),
    ]
}

#[cfg(test)]
pub fn test_identity() -> String {
    "ocean-forest-moon-star".to_string()
}



/// Test media constraints helpers
pub mod constraints {
    use communitas_core::webrtc::MediaConstraints;

    /// Audio-only call constraints
    pub fn audio_only() -> MediaConstraints {
        MediaConstraints::audio_only()
    }

    /// Video call constraints (audio + video)
    pub fn video_call() -> MediaConstraints {
        MediaConstraints::video_call()
    }

    /// Screen sharing constraints
    pub fn screen_share() -> MediaConstraints {
        MediaConstraints::screen_share()
    }

    /// Full multimedia constraints (audio + video + screen)
    pub fn full_multimedia() -> MediaConstraints {
        let constraints = MediaConstraints::video_call();
        // Note: Actual implementation would need to add screen share
        // This is a placeholder for testing
        constraints
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_service_creation() {
        let service = MockWebRtcService::new();
        let state = service.state.lock().expect("lock state");
        assert!(!state.is_initialized);
        assert_eq!(state.active_calls.len(), 0);
        assert_eq!(state.devices.len(), 3);
    }

    #[test]
    fn test_fixture_creation() {
        let fixture = WebRtcTestFixture::new();
        assert_eq!(fixture.test_identity, "ocean-forest-moon-star");
    }

    #[test]
    fn test_default_devices() {
        let devices = create_default_devices();
        assert_eq!(devices.len(), 3);
        assert!(devices.iter().any(|d| d.kind == DeviceKind::AudioInput));
        assert!(devices.iter().any(|d| d.kind == DeviceKind::AudioOutput));
        assert!(devices.iter().any(|d| d.kind == DeviceKind::VideoInput));
    }

    #[test]
    fn test_constraints_helpers() {
        let audio = constraints::audio_only();
        assert!(audio.has_audio());
        assert!(!audio.has_video());

        let video = constraints::video_call();
        assert!(video.has_audio());
        assert!(video.has_video());
    }
}

#[allow(dead_code)]
pub mod unused_fixtures {
    // Placeholder for potentially unused test fixtures
    // These can be removed or implemented as needed
}
