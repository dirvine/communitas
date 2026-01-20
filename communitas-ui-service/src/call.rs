//! Call service for real-time voice communication.
//!
//! Provides a UI-friendly abstraction over WebRTC with:
//! - Device management (microphone, speaker, camera)
//! - Call state tracking via watch channels
//! - Graceful fallback for media errors (listen-only mode)
//! - Participant state updates
//!
//! ## Device Enumeration Architecture
//!
//! Device enumeration requires platform-specific APIs (e.g., CoreAudio on macOS,
//! WASAPI on Windows, PulseAudio on Linux). The Rust backend cannot directly
//! access these APIs - instead, the platform host layer (Tauri/Dioxus) must
//! implement the [`DeviceEnumerator`] trait.
//!
//! When no platform enumerator is provided, a mock implementation returns
//! placeholder devices for development and testing purposes.
//!
//! See `saorsa_webrtc_core` for the underlying WebRTC infrastructure.

use std::sync::Arc;

use async_trait::async_trait;
use communitas_ui_api::call::{
    CallInfo, CallSettings, CallSnapshot, CallState, DeviceType, MediaDevice, MediaError,
    Participant,
};
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::{debug, instrument, warn};

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::util::current_timestamp_millis;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Event};

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
    #[error("device enumeration failed: {0}")]
    DeviceEnumerationFailed(String),
    #[error("operation timed out")]
    Timeout,
    #[error("core error: {0}")]
    CoreError(String),
}

/// Trait for platform-specific device enumeration.
///
/// Media device enumeration requires platform-specific APIs that are not accessible
/// from the Rust backend. Platform hosts (e.g., Tauri, Dioxus desktop) should
/// implement this trait to provide real device enumeration.
///
/// # Example Implementation (Tauri)
///
/// ```ignore
/// struct TauriDeviceEnumerator;
///
/// #[async_trait]
/// impl DeviceEnumerator for TauriDeviceEnumerator {
///     async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
///         // Use Tauri's media device APIs here
///         todo!()
///     }
/// }
/// ```
#[async_trait]
pub trait DeviceEnumerator: Send + Sync {
    /// Enumerate all available media devices.
    ///
    /// Returns a list of microphones, speakers, and cameras available on the system.
    async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError>;

    /// Check if a specific device is available.
    ///
    /// Default implementation checks the enumerated devices list.
    async fn is_device_available(&self, device_id: &str) -> Result<bool, CallError> {
        let devices = self.enumerate_devices().await?;
        Ok(devices.iter().any(|d| d.id == device_id && d.is_available))
    }
}

/// Mock device enumerator for development and testing.
///
/// Returns placeholder devices when no platform-specific enumerator is available.
/// This allows the UI to function in development mode without real hardware access.
///
/// **Note**: These are simulated devices and will not provide actual audio/video.
#[derive(Debug, Default, Clone)]
pub struct MockDeviceEnumerator;

#[async_trait]
impl DeviceEnumerator for MockDeviceEnumerator {
    async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        debug!("Using mock device enumeration - no real devices available");
        warn!(
            "Device enumeration is using mock implementation. \
             For real devices, implement DeviceEnumerator for your platform host."
        );

        Ok(vec![
            MediaDevice {
                id: "mock-mic-default".to_string(),
                name: "Default Microphone (Mock)".to_string(),
                device_type: DeviceType::Microphone,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "mock-mic-builtin".to_string(),
                name: "Built-in Microphone (Mock)".to_string(),
                device_type: DeviceType::Microphone,
                is_default: false,
                is_available: true,
            },
            MediaDevice {
                id: "mock-speaker-default".to_string(),
                name: "Default Speaker (Mock)".to_string(),
                device_type: DeviceType::Speaker,
                is_default: true,
                is_available: true,
            },
            MediaDevice {
                id: "mock-speaker-builtin".to_string(),
                name: "Built-in Speakers (Mock)".to_string(),
                device_type: DeviceType::Speaker,
                is_default: false,
                is_available: true,
            },
            MediaDevice {
                id: "mock-camera-default".to_string(),
                name: "Default Camera (Mock)".to_string(),
                device_type: DeviceType::Camera,
                is_default: true,
                is_available: true,
            },
        ])
    }
}

/// Empty device enumerator that returns no devices.
///
/// Use this when running in a headless environment where no media devices
/// should be expected (e.g., CI, server-side).
#[derive(Debug, Default, Clone)]
pub struct NoDeviceEnumerator;

#[async_trait]
impl DeviceEnumerator for NoDeviceEnumerator {
    async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        debug!("No device enumeration available - running headless");
        Ok(Vec::new())
    }
}

/// Verify that a device with the given ID and type exists in the available devices list.
fn verify_device_exists(
    available_devices: &[MediaDevice],
    device_id: &str,
    device_type: DeviceType,
) -> Result<(), CallError> {
    if available_devices
        .iter()
        .any(|d| d.id == device_id && d.device_type == device_type)
    {
        Ok(())
    } else {
        Err(CallError::DeviceNotFound(device_id.to_string()))
    }
}

/// Service for real-time voice/video communication.
pub struct CallService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    tx: watch::Sender<CallSnapshot>,
    rx: watch::Receiver<CallSnapshot>,
    state: Arc<RwLock<CallServiceState>>,
    device_enumerator: Arc<dyn DeviceEnumerator>,
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
    /// Create a new call service with the default mock device enumerator.
    ///
    /// For production use with real devices, use [`CallService::with_device_enumerator`]
    /// and provide a platform-specific [`DeviceEnumerator`] implementation.
    pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        Self::with_device_enumerator(auth, app, Arc::new(MockDeviceEnumerator))
    }

    /// Create a new call service with a custom device enumerator.
    ///
    /// # Arguments
    ///
    /// * `auth` - Authentication controller for user identity
    /// * `app` - Communitas application context
    /// * `device_enumerator` - Platform-specific device enumerator implementation
    ///
    /// # Example
    ///
    /// ```ignore
    /// let enumerator = Arc::new(TauriDeviceEnumerator::new());
    /// let call_service = CallService::with_device_enumerator(auth, app, enumerator);
    /// ```
    pub fn with_device_enumerator(
        auth: Arc<AuthController>,
        app: Arc<CommunitasApp>,
        device_enumerator: Arc<dyn DeviceEnumerator>,
    ) -> Self {
        let (tx, rx) = watch::channel(CallSnapshot::default());
        Self {
            auth,
            app,
            tx,
            rx,
            state: Arc::new(RwLock::new(CallServiceState::default())),
            device_enumerator,
        }
    }

    /// Create a new call service for headless operation (no devices).
    ///
    /// Use this in server-side or CI environments where no media devices are available.
    pub fn headless(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        Self::with_device_enumerator(auth, app, Arc::new(NoDeviceEnumerator))
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

    /// Get a reference to the Communitas app.
    pub fn app(&self) -> Arc<CommunitasApp> {
        self.app.clone()
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
    ///
    /// Uses the configured [`DeviceEnumerator`] to discover audio and video devices.
    /// Returns mock devices by default; for real devices, provide a platform-specific
    /// enumerator via [`CallService::with_device_enumerator`].
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::DeviceEnumerationFailed`] if the platform device enumeration fails.
    #[instrument(skip(self), name = "ui.call.list_devices")]
    pub async fn list_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        let rx = self.auth.subscribe();
        if matches!(&*rx.borrow(), AuthStateSnapshot::LoggedOut) {
            return Err(CallError::NotAuthenticated);
        }

        // Use the configured device enumerator
        let devices = self.device_enumerator.enumerate_devices().await?;

        // Update available devices in state
        {
            let mut state = self.state.write().await;
            state.available_devices = devices.clone();
        }
        self.broadcast().await;

        Ok(devices)
    }

    /// Refresh the list of available devices.
    ///
    /// Call this when the user plugs in or unplugs a device.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::DeviceEnumerationFailed`] if the platform device enumeration fails.
    #[instrument(skip(self), name = "ui.call.refresh_devices")]
    pub async fn refresh_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
        self.list_devices().await
    }

    /// Select a device of the specified type.
    async fn select_device(
        &self,
        device_id: &str,
        device_type: DeviceType,
    ) -> Result<(), CallError> {
        let mut state = self.state.write().await;
        verify_device_exists(&state.available_devices, device_id, device_type)?;

        match device_type {
            DeviceType::Microphone => {
                state.settings.selected_microphone = Some(device_id.to_string());
            }
            DeviceType::Speaker => {
                state.settings.selected_speaker = Some(device_id.to_string());
            }
            DeviceType::Camera => {
                state.settings.selected_camera = Some(device_id.to_string());
            }
        }
        drop(state);
        self.broadcast().await;
        Ok(())
    }

    /// Select a microphone device.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::DeviceNotFound`] if the device ID is not in the available devices list.
    #[instrument(skip(self), name = "ui.call.select_microphone", fields(device_id))]
    pub async fn select_microphone(&self, device_id: &str) -> Result<(), CallError> {
        self.select_device(device_id, DeviceType::Microphone).await
    }

    /// Select a speaker device.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::DeviceNotFound`] if the device ID is not in the available devices list.
    #[instrument(skip(self), name = "ui.call.select_speaker", fields(device_id))]
    pub async fn select_speaker(&self, device_id: &str) -> Result<(), CallError> {
        self.select_device(device_id, DeviceType::Speaker).await
    }

    /// Select a camera device.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::DeviceNotFound`] if the device ID is not in the available devices list.
    #[instrument(skip(self), name = "ui.call.select_camera", fields(device_id))]
    pub async fn select_camera(&self, device_id: &str) -> Result<(), CallError> {
        self.select_device(device_id, DeviceType::Camera).await
    }

    /// Test a microphone and return the current audio level (0.0-1.0).
    ///
    /// # Errors
    ///
    /// Returns [`CallError::DeviceNotFound`] if the device ID is not in the available devices list.
    #[instrument(skip(self), name = "ui.call.test_microphone", fields(device_id))]
    pub async fn test_microphone(&self, device_id: &str) -> Result<f32, CallError> {
        let state = self.state.read().await;
        verify_device_exists(&state.available_devices, device_id, DeviceType::Microphone)?;

        // Mock implementation: return a simulated audio level
        // Real implementation would use platform APIs to capture audio
        Ok(0.35)
    }

    /// Test a speaker by playing a test sound.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::DeviceNotFound`] if the device ID is not in the available devices list.
    #[instrument(skip(self), name = "ui.call.test_speaker", fields(device_id))]
    pub async fn test_speaker(&self, device_id: &str) -> Result<(), CallError> {
        let state = self.state.read().await;
        verify_device_exists(&state.available_devices, device_id, DeviceType::Speaker)?;

        // Mock implementation: would play test sound via platform APIs
        Ok(())
    }

    // ===== Call Management =====

    /// Start a new call for the specified entity using CommunitasApp.
    ///
    /// This method creates a real call by executing `Command::StartCall` through
    /// the Communitas core. The call state is updated via the watch channel when
    /// the `Event::CallStarted` response is received.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity (channel, group, etc.) to start the call in.
    /// * `video_enabled` - Whether to enable video from the start.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::AlreadyInCall`] if the user is already in an active call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.start", fields(entity_id, video_enabled))]
    pub async fn start_call(
        &self,
        entity_id: &str,
        video_enabled: bool,
    ) -> Result<CallInfo, CallError> {
        let rx = self.auth.subscribe();
        let (identity_name, four_words) = match &*rx.borrow() {
            AuthStateSnapshot::LoggedOut | AuthStateSnapshot::Authenticating => {
                return Err(CallError::NotAuthenticated);
            }
            AuthStateSnapshot::Authenticated(session) => {
                (session.display_name.clone(), session.four_words.clone())
            }
        };

        // Check if already in call and transition to connecting state
        {
            let mut state = self.state.write().await;
            if state.call_state.is_active() {
                return Err(CallError::AlreadyInCall);
            }
            state.call_state = CallState::Connecting;
        }
        self.broadcast().await;

        // Build and execute the StartCall command
        let cmd = Command::StartCall {
            entity_id: entity_id.to_string(),
            video_enabled,
        };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                // Reset state to Idle on failure
                {
                    let mut state = self.state.write().await;
                    state.call_state = CallState::Idle;
                }
                self.broadcast().await;
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Find the CallStarted event in the response
        let call_started = events.iter().find_map(|event| {
            if let Event::CallStarted {
                call_id,
                entity_id: eid,
            } = event
            {
                Some((call_id.clone(), eid.clone()))
            } else {
                None
            }
        });

        let (call_id, returned_entity_id) = match call_started {
            Some(result) => result,
            None => {
                // Reset state to Idle if no CallStarted event
                {
                    let mut state = self.state.write().await;
                    state.call_state = CallState::Idle;
                }
                self.broadcast().await;
                return Err(CallError::CoreError(
                    "No CallStarted event returned from core".to_string(),
                ));
            }
        };

        debug!(call_id = %call_id, entity_id = %returned_entity_id, "Call started successfully");

        // Create participant for self
        let now = current_timestamp_millis();
        let my_participant_id = format!("participant-{}", now);
        let participant = Participant {
            id: my_participant_id.clone(),
            display_name: identity_name,
            four_words,
            is_muted: false,
            is_video_enabled: video_enabled,
            is_speaking: false,
            audio_level: 0.0,
            joined_at: now,
        };

        // Build call info (use entity_id reference for formatting before moving)
        let call_info = CallInfo {
            call_id,
            entity_name: format!("Call: {}", returned_entity_id),
            entity_id: returned_entity_id,
            participants: vec![participant.clone()],
            started_at: now,
            duration_seconds: 0,
            my_participant_id,
        };

        // Update state to InCall
        {
            let mut state = self.state.write().await;
            state.call_state = CallState::InCall;
            state.current_call = Some(call_info.clone());
            state.participants = vec![participant];
        }
        self.broadcast().await;

        Ok(call_info)
    }

    /// Join an existing call by its call ID.
    ///
    /// This method joins a call by executing `Command::JoinCall` through
    /// the Communitas core. The call state is updated via the watch channel when
    /// the `Event::CallJoined` response is received.
    ///
    /// # Arguments
    ///
    /// * `call_id` - The ID of the call to join.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::AlreadyInCall`] if the user is already in an active call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.join", fields(call_id))]
    pub async fn join_call(&self, call_id: &str) -> Result<CallInfo, CallError> {
        let rx = self.auth.subscribe();
        let (identity_name, four_words) = match &*rx.borrow() {
            AuthStateSnapshot::LoggedOut | AuthStateSnapshot::Authenticating => {
                return Err(CallError::NotAuthenticated);
            }
            AuthStateSnapshot::Authenticated(session) => {
                (session.display_name.clone(), session.four_words.clone())
            }
        };

        // Check if already in call and transition to connecting state
        {
            let mut state = self.state.write().await;
            if state.call_state.is_active() {
                return Err(CallError::AlreadyInCall);
            }
            state.call_state = CallState::Connecting;
        }
        self.broadcast().await;

        // Build and execute the JoinCall command
        let cmd = Command::JoinCall {
            call_id: call_id.to_string(),
        };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                // Reset state to Idle on failure
                {
                    let mut state = self.state.write().await;
                    state.call_state = CallState::Idle;
                }
                self.broadcast().await;
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Find the CallJoined event in the response
        let call_joined = events.iter().find_map(|event| {
            if let Event::CallJoined {
                call_id: joined_call_id,
            } = event
            {
                Some(joined_call_id.clone())
            } else {
                None
            }
        });

        let joined_call_id = match call_joined {
            Some(id) => id,
            None => {
                // Reset state to Idle if no CallJoined event
                {
                    let mut state = self.state.write().await;
                    state.call_state = CallState::Idle;
                }
                self.broadcast().await;
                return Err(CallError::CoreError(
                    "No CallJoined event returned from core".to_string(),
                ));
            }
        };

        debug!(call_id = %joined_call_id, "Joined call successfully");

        // Create participant for self
        let now = current_timestamp_millis();
        let my_participant_id = format!("participant-{}", now);
        let participant = Participant {
            id: my_participant_id.clone(),
            display_name: identity_name,
            four_words,
            is_muted: false,
            is_video_enabled: false,
            is_speaking: false,
            audio_level: 0.0,
            joined_at: now,
        };

        // Build call info
        let call_info = CallInfo {
            call_id: joined_call_id,
            entity_id: String::new(), // Will be populated by actual call metadata
            entity_name: String::new(),
            participants: vec![participant.clone()],
            started_at: now,
            duration_seconds: 0,
            my_participant_id,
        };

        // Update state to InCall
        {
            let mut state = self.state.write().await;
            state.call_state = CallState::InCall;
            state.current_call = Some(call_info.clone());
            state.participants = vec![participant];
        }
        self.broadcast().await;

        Ok(call_info)
    }

    /// Leave the current call.
    ///
    /// This method leaves the current call by executing `Command::LeaveCall` through
    /// the Communitas core. The call state is cleaned up when the `Event::CallLeft`
    /// response is received.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.leave")]
    pub async fn leave_call(&self) -> Result<(), CallError> {
        // Get current call_id from state
        let call_id = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            state
                .current_call
                .as_ref()
                .map(|c| c.call_id.clone())
                .ok_or(CallError::NotInCall)?
        };

        // Transition to disconnecting state
        {
            let mut state = self.state.write().await;
            state.call_state = CallState::Disconnected;
        }
        self.broadcast().await;

        // Build and execute the LeaveCall command
        let cmd = Command::LeaveCall { call_id };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                // Reset state to InCall on failure (we're still in the call)
                {
                    let mut state = self.state.write().await;
                    state.call_state = CallState::InCall;
                }
                self.broadcast().await;
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Verify CallLeft event in the response
        let call_left = events
            .iter()
            .any(|event| matches!(event, Event::CallLeft { .. }));

        if !call_left {
            warn!("No CallLeft event returned from core, but cleaning up state anyway");
        }

        // Clean up call state
        {
            let mut state = self.state.write().await;
            state.call_state = CallState::Idle;
            state.current_call = None;
            state.participants.clear();
            state.media_errors.clear();
            state.listen_only_mode = false;
        }
        self.broadcast().await;

        debug!("Left call successfully");
        Ok(())
    }
    /// Get the current call info.
    pub fn get_current_call(&self) -> Option<CallInfo> {
        self.rx.borrow().call_info.clone()
    }

    // ===== Call Controls =====

    /// Toggle mute state and return the new muted state.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
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
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
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
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
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
    ///
    /// Clears any media errors for the specified device type and exits listen-only mode
    /// if no errors remain.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::MediaError`] if the retry attempt fails to capture media.
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

    async fn make_service(temp: &TempDir) -> CallService {
        let storage = UiStorage::from_path(temp.path()).expect("storage should init");
        let auth = Arc::new(AuthController::new(storage).expect("auth should init"));
        let app = Arc::new(
            CommunitasApp::new(
                "ocean-forest-moon-star".to_string(),
                "TestUser".to_string(),
                "TestDevice".to_string(),
                temp.path()
                    .join("app_storage")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            .expect("app should init"),
        );
        CallService::new(auth, app)
    }

    #[tokio::test]
    async fn call_service_starts_idle() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let snap = service.current_snapshot();
        assert_eq!(snap.state, CallState::Idle);
        assert!(snap.call_info.is_none());
        assert!(snap.participants.is_empty());
        assert!(!snap.listen_only_mode);
    }

    #[tokio::test]
    async fn list_devices_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.list_devices().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    #[tokio::test]
    async fn headless_enumerator_returns_no_devices() {
        // Test the NoDeviceEnumerator trait directly without full service setup
        let enumerator = NoDeviceEnumerator;
        let devices = enumerator
            .enumerate_devices()
            .await
            .expect("should enumerate");
        assert!(devices.is_empty(), "headless should return no devices");
    }

    #[tokio::test]
    async fn mock_enumerator_returns_devices() {
        // Test the MockDeviceEnumerator trait directly without full service setup
        let enumerator = MockDeviceEnumerator;
        let devices = enumerator
            .enumerate_devices()
            .await
            .expect("should enumerate");
        assert!(!devices.is_empty(), "mock should return devices");

        // Verify we have at least one of each type
        assert!(
            devices
                .iter()
                .any(|d| d.device_type == DeviceType::Microphone)
        );
        assert!(devices.iter().any(|d| d.device_type == DeviceType::Speaker));
        assert!(devices.iter().any(|d| d.device_type == DeviceType::Camera));
    }

    #[tokio::test]
    async fn custom_device_enumerator_trait() {
        // Test that custom enumerators work
        struct TestEnumerator;

        #[async_trait]
        impl DeviceEnumerator for TestEnumerator {
            async fn enumerate_devices(&self) -> Result<Vec<MediaDevice>, CallError> {
                Ok(vec![MediaDevice {
                    id: "test-mic".to_string(),
                    name: "Test Microphone".to_string(),
                    device_type: DeviceType::Microphone,
                    is_default: true,
                    is_available: true,
                }])
            }
        }

        let enumerator = TestEnumerator;
        let devices = enumerator
            .enumerate_devices()
            .await
            .expect("should enumerate");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "test-mic");
    }

    #[tokio::test]
    async fn device_enumerator_is_available_default_impl() {
        // Test the default is_device_available implementation
        let enumerator = MockDeviceEnumerator;

        // Check that a mock device is available
        let available = enumerator
            .is_device_available("mock-mic-default")
            .await
            .expect("should check availability");
        assert!(available);

        // Check that a non-existent device is not available
        let not_available = enumerator
            .is_device_available("non-existent-device")
            .await
            .expect("should check availability");
        assert!(!not_available);
    }

    #[tokio::test]
    async fn join_call_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.join_call("call-123").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    #[tokio::test]
    async fn leave_call_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.leave_call().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn toggle_mute_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.toggle_mute().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn toggle_video_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.toggle_video().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn select_device_requires_device_list() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Device not in available list
        let result = service.select_microphone("unknown-device").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::DeviceNotFound(_)));
    }

    #[tokio::test]
    async fn media_error_enables_listen_only_mode() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

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
        let service = make_service(&temp).await;

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
        let service = make_service(&temp).await;

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

    #[tokio::test]
    async fn start_call_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // State should be Idle initially
        assert_eq!(service.get_call_state(), CallState::Idle);

        let result = service.start_call("entity1", false).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));

        // State should remain Idle after auth failure (no state transition)
        assert_eq!(service.get_call_state(), CallState::Idle);
    }

    #[test]
    fn call_error_display() {
        let err = CallError::NotInCall;
        assert_eq!(format!("{err}"), "not in a call");

        let err = CallError::MediaError("No microphone".to_string());
        assert_eq!(format!("{err}"), "media error: No microphone");

        let err = CallError::DeviceEnumerationFailed("Platform error".to_string());
        assert_eq!(
            format!("{err}"),
            "device enumeration failed: Platform error"
        );

        let err = CallError::CoreError("Command execution failed".to_string());
        assert_eq!(format!("{err}"), "core error: Command execution failed");
    }
}
