//! Call service for real-time voice communication.
//!
//! Provides a UI-friendly abstraction over WebRTC with:
//! - Device management (microphone, speaker, camera)
//! - Call state tracking via watch channels
//! - Graceful fallback for media errors (listen-only mode)
//! - Participant state updates

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use communitas_ui_api::call::{
    CallInfo, CallSettings, CallSnapshot, CallState, DeviceType, MediaDevice, MediaError,
    Participant,
};
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::instrument;

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};

/// Get current timestamp in milliseconds since Unix epoch.
fn current_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Errors returned by the call service.
#[derive(Debug, Error)]
pub enum CallError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("not in a call")]
    NotInCall,
    #[error("already in a call")]
    AlreadyInCall,
    #[error("media error: {0}")]
    MediaError(String),
    #[error("signaling error: {0}")]
    SignalingError(String),
    #[error("connection error: {0}")]
    ConnectionError(String),
    #[error("device not found: {0}")]
    DeviceNotFound(String),
    #[error("operation timed out")]
    Timeout,
}

/// Service for real-time voice/video communication.
pub struct CallService {
    auth: Arc<AuthController>,
    tx: watch::Sender<CallSnapshot>,
    rx: watch::Receiver<CallSnapshot>,
    state: Arc<RwLock<CallServiceState>>,
}

/// Internal state for the call service.
struct CallServiceState {
    call_state: CallState,
    current_call: Option<CallInfo>,
    participants: Vec<Participant>,
    settings: CallSettings,
    media_errors: Vec<MediaError>,
    available_devices: Vec<MediaDevice>,
    listen_only_mode: bool,
}

impl Default for CallServiceState {
    fn default() -> Self {
        Self {
            call_state: CallState::Idle,
            current_call: None,
            participants: Vec::new(),
            settings: CallSettings::default(),
            media_errors: Vec::new(),
            available_devices: Vec::new(),
            listen_only_mode: false,
        }
    }
}

#[allow(unused_variables)] // Mock implementation - params used for tracing but not actual logic
impl CallService {
    /// Create a new call service linked to the auth controller.
    pub fn new(auth: Arc<AuthController>) -> Self {
        let (tx, rx) = watch::channel(CallSnapshot::default());
        Self {
            auth,
            tx,
            rx,
            state: Arc::new(RwLock::new(CallServiceState::default())),
        }
    }

    /// Subscribe to call state updates.
    pub fn subscribe(&self) -> watch::Receiver<CallSnapshot> {
        self.rx.clone()
    }

    /// Subscribe to call state updates (alias for consistency with other services).
    pub fn subscribe_state(&self) -> watch::Receiver<CallSnapshot> {
        self.subscribe()
    }

    /// Subscribe to participant updates (returns same channel, filter in UI).
    pub fn subscribe_participants(&self) -> watch::Receiver<CallSnapshot> {
        self.subscribe()
    }

    /// Get the current call snapshot without subscribing.
    pub fn current_snapshot(&self) -> CallSnapshot {
        self.rx.borrow().clone()
    }

    /// Get the current call state.
    pub fn get_call_state(&self) -> CallState {
        self.rx.borrow().state
    }

    /// Get the current participants.
    pub fn get_participants(&self) -> Vec<Participant> {
        self.rx.borrow().participants.clone()
    }

    /// Broadcast updated state to all subscribers.
    async fn broadcast(&self) {
        let state = self.state.read().await;
        let snapshot = CallSnapshot {
            state: state.call_state,
            call_info: state.current_call.clone(),
            participants: state.participants.clone(),
            media_errors: state.media_errors.clone(),
            available_devices: state.available_devices.clone(),
            settings: state.settings.clone(),
            listen_only_mode: state.listen_only_mode,
        };
        // Ignore send error if no receivers
        let _ = self.tx.send(snapshot);
    }

    // ===== Device Management =====

    /// List available media devices.
    #[instrument(skip(self), name = "ui.call.list_devices")]
    pub async fn list_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        let rx = self.auth.subscribe();
        if matches!(&*rx.borrow(), AuthStateSnapshot::LoggedOut) {
            return Err(CallError::NotAuthenticated);
        }

        // Mock implementation: return simulated devices
        let devices = vec![
            MediaDevice {
                id: "mic-default".to_string(),
                name: "Default Microphone".to_string(),
                device_type: DeviceType::Microphone,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "mic-builtin".to_string(),
                name: "Built-in Microphone".to_string(),
                device_type: DeviceType::Microphone,
                is_default: false,
                is_available: true,
            },
            MediaDevice {
                id: "speaker-default".to_string(),
                name: "Default Speaker".to_string(),
                device_type: DeviceType::Speaker,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "speaker-builtin".to_string(),
                name: "Built-in Speakers".to_string(),
                device_type: DeviceType::Speaker,
                is_default: false,
                is_available: true,
            },
            MediaDevice {
                id: "camera-default".to_string(),
                name: "FaceTime HD Camera".to_string(),
                device_type: DeviceType::Camera,
                is_default: true,
                is_available: true,
            },
        ];

        // Update available devices in state
        {
            let mut state = self.state.write().await;
            state.available_devices = devices.clone();
        }
        self.broadcast().await;

        Ok(devices)
    }

    /// Select a microphone device.
    #[instrument(skip(self), name = "ui.call.select_microphone", fields(device_id))]
    pub async fn select_microphone(&self, device_id: &str) -> Result<(), CallError> {
        let mut state = self.state.write().await;

        // Verify device exists
        if !state
            .available_devices
            .iter()
            .any(|d| d.id == device_id && d.device_type == DeviceType::Microphone)
        {
            return Err(CallError::DeviceNotFound(device_id.to_string()));
        }

        state.settings.selected_microphone = Some(device_id.to_string());
        drop(state);
        self.broadcast().await;
        Ok(())
    }

    /// Select a speaker device.
    #[instrument(skip(self), name = "ui.call.select_speaker", fields(device_id))]
    pub async fn select_speaker(&self, device_id: &str) -> Result<(), CallError> {
        let mut state = self.state.write().await;

        // Verify device exists
        if !state
            .available_devices
            .iter()
            .any(|d| d.id == device_id && d.device_type == DeviceType::Speaker)
        {
            return Err(CallError::DeviceNotFound(device_id.to_string()));
        }

        state.settings.selected_speaker = Some(device_id.to_string());
        drop(state);
        self.broadcast().await;
        Ok(())
    }

    /// Select a camera device.
    #[instrument(skip(self), name = "ui.call.select_camera", fields(device_id))]
    pub async fn select_camera(&self, device_id: &str) -> Result<(), CallError> {
        let mut state = self.state.write().await;

        // Verify device exists
        if !state
            .available_devices
            .iter()
            .any(|d| d.id == device_id && d.device_type == DeviceType::Camera)
        {
            return Err(CallError::DeviceNotFound(device_id.to_string()));
        }

        state.settings.selected_camera = Some(device_id.to_string());
        drop(state);
        self.broadcast().await;
        Ok(())
    }

    /// Test a microphone and return the current audio level (0.0-1.0).
    #[instrument(skip(self), name = "ui.call.test_microphone", fields(device_id))]
    pub async fn test_microphone(&self, device_id: &str) -> Result<f32, CallError> {
        let state = self.state.read().await;

        // Verify device exists
        if !state
            .available_devices
            .iter()
            .any(|d| d.id == device_id && d.device_type == DeviceType::Microphone)
        {
            return Err(CallError::DeviceNotFound(device_id.to_string()));
        }

        // Mock implementation: return a simulated audio level
        Ok(0.35)
    }

    /// Test a speaker by playing a test sound.
    #[instrument(skip(self), name = "ui.call.test_speaker", fields(device_id))]
    pub async fn test_speaker(&self, device_id: &str) -> Result<(), CallError> {
        let state = self.state.read().await;

        // Verify device exists
        if !state
            .available_devices
            .iter()
            .any(|d| d.id == device_id && d.device_type == DeviceType::Speaker)
        {
            return Err(CallError::DeviceNotFound(device_id.to_string()));
        }

        // Mock implementation: would play test sound
        Ok(())
    }

    // ===== Call Management =====

    /// Join a call for the specified entity.
    #[instrument(skip(self), name = "ui.call.join", fields(entity_id))]
    pub async fn join_call(&self, entity_id: &str) -> Result<CallInfo, CallError> {
        let rx = self.auth.subscribe();
        let (identity_name, four_words) = match &*rx.borrow() {
            AuthStateSnapshot::LoggedOut | AuthStateSnapshot::Authenticating => {
                return Err(CallError::NotAuthenticated);
            }
            AuthStateSnapshot::Authenticated(session) => {
                (session.display_name.clone(), session.four_words.clone())
            }
        };

        let mut state = self.state.write().await;

        // Check if already in a call
        if state.call_state.is_active() {
            return Err(CallError::AlreadyInCall);
        }

        // Transition to connecting state
        state.call_state = CallState::Connecting;
        drop(state);
        self.broadcast().await;

        // Mock implementation: simulate connection delay and create call
        let my_participant_id = format!("participant-{}", current_timestamp_millis());
        let participant = Participant {
            id: my_participant_id.clone(),
            display_name: identity_name,
            four_words,
            is_muted: false,
            is_video_enabled: false,
            is_speaking: false,
            audio_level: 0.0,
            joined_at: current_timestamp_millis(),
        };

        let call_info = CallInfo {
            call_id: format!("call-{}-{}", entity_id, current_timestamp_millis()),
            entity_id: entity_id.to_string(),
            entity_name: format!("Call: {}", entity_id),
            participants: vec![participant.clone()],
            started_at: current_timestamp_millis(),
            duration_seconds: 0,
            my_participant_id,
        };

        let mut state = self.state.write().await;
        state.call_state = CallState::InCall;
        state.current_call = Some(call_info.clone());
        state.participants = vec![participant];
        drop(state);
        self.broadcast().await;

        Ok(call_info)
    }

    /// Leave the current call.
    #[instrument(skip(self), name = "ui.call.leave")]
    pub async fn leave_call(&self) -> Result<(), CallError> {
        let mut state = self.state.write().await;

        if !state.call_state.is_active() {
            return Err(CallError::NotInCall);
        }

        // Clean up call state
        state.call_state = CallState::Disconnected;
        state.current_call = None;
        state.participants.clear();
        state.media_errors.clear();
        state.listen_only_mode = false;

        drop(state);
        self.broadcast().await;

        // Reset to idle after brief delay
        let tx = self.tx.clone();
        let state_clone = self.state.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let mut state = state_clone.write().await;
            state.call_state = CallState::Idle;
            drop(state);
            let state = state_clone.read().await;
            let snapshot = CallSnapshot {
                state: state.call_state,
                call_info: state.current_call.clone(),
                participants: state.participants.clone(),
                media_errors: state.media_errors.clone(),
                available_devices: state.available_devices.clone(),
                settings: state.settings.clone(),
                listen_only_mode: state.listen_only_mode,
            };
            let _ = tx.send(snapshot);
        });

        Ok(())
    }

    /// Get the current call info.
    pub fn get_current_call(&self) -> Option<CallInfo> {
        self.rx.borrow().call_info.clone()
    }

    // ===== Call Controls =====

    /// Toggle mute state and return the new muted state.
    #[instrument(skip(self), name = "ui.call.toggle_mute")]
    pub async fn toggle_mute(&self) -> Result<bool, CallError> {
        let mut state = self.state.write().await;

        if !state.call_state.is_active() {
            return Err(CallError::NotInCall);
        }

        let my_id = state
            .current_call
            .as_ref()
            .map(|c| c.my_participant_id.clone());

        if let Some(my_id) = my_id
            && let Some(participant) = state.participants.iter_mut().find(|p| p.id == my_id)
        {
            participant.is_muted = !participant.is_muted;
            let new_muted = participant.is_muted;

            // Update call_info participants too
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call.participants.iter_mut().find(|p| p.id == my_id)
            {
                p.is_muted = new_muted;
            }

            drop(state);
            self.broadcast().await;
            return Ok(new_muted);
        }

        Err(CallError::NotInCall)
    }

    /// Toggle video state and return the new enabled state.
    #[instrument(skip(self), name = "ui.call.toggle_video")]
    pub async fn toggle_video(&self) -> Result<bool, CallError> {
        let mut state = self.state.write().await;

        if !state.call_state.is_active() {
            return Err(CallError::NotInCall);
        }

        let my_id = state
            .current_call
            .as_ref()
            .map(|c| c.my_participant_id.clone());

        if let Some(my_id) = my_id
            && let Some(participant) = state.participants.iter_mut().find(|p| p.id == my_id)
        {
            participant.is_video_enabled = !participant.is_video_enabled;
            let new_video = participant.is_video_enabled;

            // Update call_info participants too
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call.participants.iter_mut().find(|p| p.id == my_id)
            {
                p.is_video_enabled = new_video;
            }

            drop(state);
            self.broadcast().await;
            return Ok(new_video);
        }

        Err(CallError::NotInCall)
    }

    /// Set audio input enabled state.
    #[instrument(skip(self), name = "ui.call.set_audio_input")]
    pub async fn set_audio_input_enabled(&self, enabled: bool) -> Result<(), CallError> {
        let mut state = self.state.write().await;

        if !state.call_state.is_active() {
            return Err(CallError::NotInCall);
        }

        let my_id = state
            .current_call
            .as_ref()
            .map(|c| c.my_participant_id.clone());

        if let Some(my_id) = my_id
            && let Some(participant) = state.participants.iter_mut().find(|p| p.id == my_id)
        {
            participant.is_muted = !enabled;

            // Update call_info participants too
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call.participants.iter_mut().find(|p| p.id == my_id)
            {
                p.is_muted = !enabled;
            }

            drop(state);
            self.broadcast().await;
            return Ok(());
        }

        Err(CallError::NotInCall)
    }

    // ===== Media Error Handling =====

    /// Get current media errors.
    pub fn get_media_errors(&self) -> Vec<MediaError> {
        self.rx.borrow().media_errors.clone()
    }

    /// Retry media capture for the specified device type.
    #[instrument(skip(self), name = "ui.call.retry_media", fields(?device_type))]
    pub async fn retry_media(&self, device_type: DeviceType) -> Result<(), CallError> {
        let mut state = self.state.write().await;

        // Remove errors for this device type
        state.media_errors.retain(|e| e.device_type != device_type);

        // If no more media errors, exit listen-only mode
        if state.media_errors.is_empty() {
            state.listen_only_mode = false;
        }

        drop(state);
        self.broadcast().await;
        Ok(())
    }

    /// Report a media error (used internally or by UI).
    pub async fn report_media_error(&self, error: MediaError) {
        let mut state = self.state.write().await;
        state.media_errors.push(error);

        // Check if we should enter listen-only mode
        let has_mic_error = state
            .media_errors
            .iter()
            .any(|e| e.device_type == DeviceType::Microphone);
        if has_mic_error {
            state.listen_only_mode = true;
        }

        drop(state);
        self.broadcast().await;
    }

    /// Check if in listen-only mode.
    pub fn is_listen_only(&self) -> bool {
        self.rx.borrow().listen_only_mode
    }

    // ===== Settings =====

    /// Get current call settings.
    pub fn get_settings(&self) -> CallSettings {
        self.rx.borrow().settings.clone()
    }

    /// Update call settings.
    #[instrument(skip(self, settings), name = "ui.call.update_settings")]
    pub async fn update_settings(&self, settings: CallSettings) -> Result<(), CallError> {
        let mut state = self.state.write().await;
        state.settings = settings;
        drop(state);
        self.broadcast().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use communitas_ui_api::MediaErrorKind;
    use tempfile::TempDir;

    fn make_auth(temp: &TempDir) -> Arc<AuthController> {
        let storage = UiStorage::from_path(temp.path()).expect("storage should init");
        Arc::new(AuthController::new(storage).expect("auth should init"))
    }

    #[tokio::test]
    async fn call_service_starts_idle() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let snap = service.current_snapshot();
        assert_eq!(snap.state, CallState::Idle);
        assert!(snap.call_info.is_none());
        assert!(snap.participants.is_empty());
        assert!(!snap.listen_only_mode);
    }

    #[tokio::test]
    async fn list_devices_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let result = service.list_devices().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    #[tokio::test]
    async fn join_call_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let result = service.join_call("entity1").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    #[tokio::test]
    async fn leave_call_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let result = service.leave_call().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn toggle_mute_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let result = service.toggle_mute().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn toggle_video_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let result = service.toggle_video().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn select_device_requires_device_list() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        // Device not in available list
        let result = service.select_microphone("unknown-device").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::DeviceNotFound(_)));
    }

    #[tokio::test]
    async fn media_error_enables_listen_only_mode() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        assert!(!service.is_listen_only());

        let error = MediaError::new(
            DeviceType::Microphone,
            MediaErrorKind::PermissionDenied,
            "Permission denied",
        );
        service.report_media_error(error).await;

        assert!(service.is_listen_only());
        assert_eq!(service.get_media_errors().len(), 1);
    }

    #[tokio::test]
    async fn retry_media_clears_error() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let error = MediaError::new(
            DeviceType::Microphone,
            MediaErrorKind::DeviceInUse,
            "Device busy",
        );
        service.report_media_error(error).await;

        assert!(service.is_listen_only());

        service
            .retry_media(DeviceType::Microphone)
            .await
            .expect("retry should succeed");

        assert!(!service.is_listen_only());
        assert!(service.get_media_errors().is_empty());
    }

    #[tokio::test]
    async fn update_settings_broadcasts() {
        let temp = TempDir::new().expect("temp dir");
        let auth = make_auth(&temp);
        let service = CallService::new(auth);

        let mut rx = service.subscribe();

        let settings = CallSettings {
            selected_microphone: Some("mic1".to_string()),
            auto_mute_on_join: true,
            ..Default::default()
        };

        service
            .update_settings(settings)
            .await
            .expect("update should succeed");

        // Wait for broadcast
        rx.changed().await.expect("should receive update");

        let snap = rx.borrow().clone();
        assert_eq!(snap.settings.selected_microphone, Some("mic1".to_string()));
        assert!(snap.settings.auto_mute_on_join);
    }

    #[test]
    fn call_error_display() {
        let err = CallError::NotInCall;
        assert_eq!(format!("{err}"), "not in a call");

        let err = CallError::MediaError("No microphone".to_string());
        assert_eq!(format!("{err}"), "media error: No microphone");
    }
}
