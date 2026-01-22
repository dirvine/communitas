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
    CallInfo, CallSettings, CallSnapshot, CallState, CallType, ConnectionQuality, DeviceType,
    MediaDevice, MediaError, MediaErrorKind, Participant, ParticipantQuality, ParticipantRole,
    QualityMetrics, RecordingInfo, RecordingState,
};
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::{debug, error, instrument, warn};

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::util::current_timestamp_millis;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{
    CallResponse, CallStatusResponse, Command, Event, Query, QueryResponse,
};

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
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("participant not found: {0}")]
    ParticipantNotFound(String),
    #[error("call is full")]
    CallFull,
    #[error("call is locked")]
    CallLocked,
    #[error("not recording")]
    NotRecording,
    #[error("recording already active")]
    AlreadyRecording,
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

/// Check if a selected device is missing from the available devices.
///
/// Returns `Some(device_id)` if the device was selected but is no longer available,
/// or `None` if no device was selected or the device is still available.
fn find_missing_device(
    selected_id: &Option<String>,
    available_devices: &[MediaDevice],
    device_type: DeviceType,
) -> Option<String> {
    let id = selected_id.as_ref()?;
    let still_available = available_devices
        .iter()
        .any(|d| d.id == *id && d.device_type == device_type);
    if still_available {
        None
    } else {
        Some(id.clone())
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
    is_screen_sharing: bool,
    quality_metrics: QualityMetrics,
    participant_quality: Vec<ParticipantQuality>,
    is_recording: bool,
    recording_info: Option<RecordingInfo>,
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
            is_screen_sharing: false,
            quality_metrics: QualityMetrics::default(),
            participant_quality: Vec::new(),
            is_recording: false,
            recording_info: None,
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

    // ===== Call Queries =====

    /// Check that the user is authenticated for query operations.
    fn require_auth_for_query(&self) -> Result<(), CallError> {
        let rx = self.auth.subscribe();
        if matches!(
            &*rx.borrow(),
            AuthStateSnapshot::LoggedOut | AuthStateSnapshot::Authenticating
        ) {
            return Err(CallError::NotAuthenticated);
        }
        Ok(())
    }

    /// Query the current status of a call from the core.
    ///
    /// Unlike [`get_call_state`] which returns locally cached state, this method
    /// queries the Communitas core for the authoritative call status including
    /// mute, video, and screen sharing states.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::CoreError`] if the query fails or returns unexpected data.
    #[instrument(skip(self), name = "ui.call.query_call_status")]
    pub async fn query_call_status(&self, call_id: &str) -> Result<CallStatusResponse, CallError> {
        self.require_auth_for_query()?;

        let response = self
            .app
            .query(Query::GetCallStatus {
                call_id: call_id.to_string(),
            })
            .await
            .map_err(|e| CallError::CoreError(format!("query failed: {e}")))?;

        match response {
            QueryResponse::CallStatus(status) => Ok(status),
            other => Err(CallError::CoreError(format!(
                "unexpected response: {other:?}"
            ))),
        }
    }

    /// Query the participant IDs for a specific call from the core.
    ///
    /// Returns a list of participant identifiers. To get full participant details,
    /// use the locally cached participants via [`get_participants`].
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::CoreError`] if the query fails or returns unexpected data.
    #[instrument(skip(self), name = "ui.call.query_call_participants")]
    pub async fn query_call_participants(&self, call_id: &str) -> Result<Vec<String>, CallError> {
        self.require_auth_for_query()?;

        let response = self
            .app
            .query(Query::GetCallParticipants {
                call_id: call_id.to_string(),
            })
            .await
            .map_err(|e| CallError::CoreError(format!("query failed: {e}")))?;

        match response {
            QueryResponse::CallParticipants(participants) => Ok(participants),
            other => Err(CallError::CoreError(format!(
                "unexpected response: {other:?}"
            ))),
        }
    }

    /// List all active calls from the core.
    ///
    /// Returns a list of calls that are currently active across all entities.
    /// This can be used to show a "rejoin call" UI or to check if a call is
    /// already in progress for an entity.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::CoreError`] if the query fails or returns unexpected data.
    #[instrument(skip(self), name = "ui.call.list_active_calls")]
    pub async fn list_active_calls(&self) -> Result<Vec<CallResponse>, CallError> {
        self.require_auth_for_query()?;

        let response = self
            .app
            .query(Query::ListActiveCalls)
            .await
            .map_err(|e| CallError::CoreError(format!("query failed: {e}")))?;

        match response {
            QueryResponse::CallList(calls) => Ok(calls),
            other => Err(CallError::CoreError(format!(
                "unexpected response: {other:?}"
            ))),
        }
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
            is_screen_sharing: state.is_screen_sharing,
            quality_metrics: state.quality_metrics.clone(),
            participant_quality: state.participant_quality.clone(),
            is_recording: state.is_recording,
            recording_info: state.recording_info.clone(),
        };
        // Ignore send error if no receivers
        let _ = self.tx.send(snapshot);
    }

    /// Update a field on the participant with the given ID in both participant lists.
    ///
    /// This updates the participant in `state.participants` and in
    /// `state.current_call.participants` to keep them in sync.
    async fn update_my_participant<F>(&self, my_id: &str, updater: F)
    where
        F: Fn(&mut Participant),
    {
        let mut state = self.state.write().await;
        if let Some(participant) = state.participants.iter_mut().find(|p| p.id == my_id) {
            updater(participant);
        }
        if let Some(ref mut call) = state.current_call
            && let Some(p) = call.participants.iter_mut().find(|p| p.id == my_id)
        {
            updater(p);
        }
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
            AuthStateSnapshot::Authenticated { session, .. } => {
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

        // Create participant for self (as host since we're starting the call)
        let now = current_timestamp_millis();
        let my_participant_id = format!("participant-{}", now);
        let participant = Participant {
            id: my_participant_id.clone(),
            display_name: identity_name,
            four_words,
            role: ParticipantRole::Host,
            is_muted: false,
            is_muted_by_host: false,
            is_video_enabled: video_enabled,
            is_speaking: false,
            is_screen_sharing: false,
            hand_raised: false,
            audio_level: 0.0,
            joined_at: now,
        };

        // Build call info (use entity_id reference for formatting before moving)
        let call_info = CallInfo {
            call_id,
            entity_name: format!("Call: {}", returned_entity_id),
            entity_id: returned_entity_id,
            call_type: CallType::Direct, // Default to direct call; can be changed for group calls
            participants: vec![participant.clone()],
            started_at: now,
            duration_seconds: 0,
            my_participant_id: my_participant_id.clone(),
            host_id: my_participant_id, // Starter is the host
            max_participants: CallType::Direct.default_max_participants(),
            is_locked: false,
            mute_on_entry: false,
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
            AuthStateSnapshot::Authenticated { session, .. } => {
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

        // Create participant for self (as regular participant since we're joining)
        let now = current_timestamp_millis();
        let my_participant_id = format!("participant-{}", now);
        let participant = Participant {
            id: my_participant_id.clone(),
            display_name: identity_name,
            four_words,
            role: ParticipantRole::Participant,
            is_muted: false,
            is_muted_by_host: false,
            is_video_enabled: false,
            is_speaking: false,
            is_screen_sharing: false,
            hand_raised: false,
            audio_level: 0.0,
            joined_at: now,
        };

        // Build call info (host_id will be updated when we receive full call state)
        let call_info = CallInfo {
            call_id: joined_call_id,
            entity_id: String::new(), // Will be populated by actual call metadata
            entity_name: String::new(),
            call_type: CallType::Direct, // Will be updated by actual call metadata
            participants: vec![participant.clone()],
            started_at: now,
            duration_seconds: 0,
            my_participant_id: my_participant_id.clone(),
            host_id: String::new(), // Will be populated by actual call metadata
            max_participants: CallType::Direct.default_max_participants(),
            is_locked: false,
            mute_on_entry: false,
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
            state.is_screen_sharing = false;
        }
        self.broadcast().await;

        debug!("Left call successfully");
        Ok(())
    }

    // ===== Group Call Methods =====

    /// Start a group call in an entity (channel, group, etc.).
    ///
    /// Similar to `start_call` but explicitly sets the call type to Group
    /// and configures group-specific settings.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity (channel, group, etc.) to start the call in.
    /// * `max_participants` - Maximum number of participants allowed (defaults to 25).
    /// * `mute_on_entry` - Whether new participants should be muted when joining.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotAuthenticated`] if the user is not logged in.
    /// Returns [`CallError::AlreadyInCall`] if the user is already in an active call.
    #[instrument(
        skip(self),
        name = "ui.call.start_group_call",
        fields(entity_id, max_participants)
    )]
    pub async fn start_group_call(
        &self,
        entity_id: &str,
        max_participants: Option<u32>,
        mute_on_entry: bool,
    ) -> Result<CallInfo, CallError> {
        // Start a regular call first
        let mut call_info = self.start_call(entity_id, false).await?;

        // Update to group call settings
        let max = max_participants.unwrap_or(CallType::Group.default_max_participants());
        {
            let mut state = self.state.write().await;
            if let Some(ref mut call) = state.current_call {
                call.call_type = CallType::Group;
                call.max_participants = max;
                call.mute_on_entry = mute_on_entry;
            }
            call_info.call_type = CallType::Group;
            call_info.max_participants = max;
            call_info.mute_on_entry = mute_on_entry;
        }
        self.broadcast().await;

        Ok(call_info)
    }

    /// Mute another participant (host/co-host only).
    ///
    /// # Arguments
    ///
    /// * `participant_id` - The ID of the participant to mute.
    /// * `mute` - Whether to mute (true) or unmute (false).
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if not in a call.
    /// Returns [`CallError::PermissionDenied`] if the caller lacks permission.
    /// Returns [`CallError::ParticipantNotFound`] if the participant is not in the call.
    #[instrument(
        skip(self),
        name = "ui.call.mute_participant",
        fields(participant_id, mute)
    )]
    pub async fn mute_participant(
        &self,
        participant_id: &str,
        mute: bool,
    ) -> Result<(), CallError> {
        // Check permissions and find participant
        let (call_id, my_role) = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;

            // Check if we have permission
            let my_role = call.my_role();
            if !my_role.can_mute_others() {
                return Err(CallError::PermissionDenied(
                    "Only hosts and co-hosts can mute others".to_string(),
                ));
            }

            // Check if participant exists
            if !call.participants.iter().any(|p| p.id == participant_id) {
                return Err(CallError::ParticipantNotFound(participant_id.to_string()));
            }

            (call.call_id.clone(), my_role)
        };

        debug!(
            call_id = %call_id,
            participant_id = %participant_id,
            mute = mute,
            my_role = ?my_role,
            "Muting participant"
        );

        // Update the participant's mute state
        {
            let mut state = self.state.write().await;
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call
                    .participants
                    .iter_mut()
                    .find(|p| p.id == participant_id)
            {
                p.is_muted = mute;
                p.is_muted_by_host = mute;
            }
            if let Some(p) = state
                .participants
                .iter_mut()
                .find(|p| p.id == participant_id)
            {
                p.is_muted = mute;
                p.is_muted_by_host = mute;
            }
        }
        self.broadcast().await;

        Ok(())
    }

    /// Remove a participant from the call (host/co-host only).
    ///
    /// # Arguments
    ///
    /// * `participant_id` - The ID of the participant to remove.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if not in a call.
    /// Returns [`CallError::PermissionDenied`] if the caller lacks permission.
    /// Returns [`CallError::ParticipantNotFound`] if the participant is not in the call.
    #[instrument(
        skip(self),
        name = "ui.call.remove_participant",
        fields(participant_id)
    )]
    pub async fn remove_participant(&self, participant_id: &str) -> Result<(), CallError> {
        // Check permissions and find participant
        {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;

            // Check if we have permission
            let my_role = call.my_role();
            if !my_role.can_remove_participants() {
                return Err(CallError::PermissionDenied(
                    "Only hosts and co-hosts can remove participants".to_string(),
                ));
            }

            // Can't remove the host
            if participant_id == call.host_id {
                return Err(CallError::PermissionDenied(
                    "Cannot remove the host".to_string(),
                ));
            }

            // Check if participant exists
            if !call.participants.iter().any(|p| p.id == participant_id) {
                return Err(CallError::ParticipantNotFound(participant_id.to_string()));
            }
        }

        // Remove the participant
        {
            let mut state = self.state.write().await;
            if let Some(ref mut call) = state.current_call {
                call.participants.retain(|p| p.id != participant_id);
            }
            state.participants.retain(|p| p.id != participant_id);
        }
        self.broadcast().await;

        debug!(participant_id = %participant_id, "Removed participant from call");
        Ok(())
    }

    /// Promote a participant to a higher role (host only).
    ///
    /// # Arguments
    ///
    /// * `participant_id` - The ID of the participant to promote.
    /// * `new_role` - The new role for the participant.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if not in a call.
    /// Returns [`CallError::PermissionDenied`] if the caller is not the host.
    /// Returns [`CallError::ParticipantNotFound`] if the participant is not in the call.
    #[instrument(skip(self), name = "ui.call.promote_participant", fields(participant_id, ?new_role))]
    pub async fn promote_participant(
        &self,
        participant_id: &str,
        new_role: ParticipantRole,
    ) -> Result<(), CallError> {
        // Check permissions
        {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;

            // Only host can promote
            if !call.am_i_host() {
                return Err(CallError::PermissionDenied(
                    "Only the host can promote participants".to_string(),
                ));
            }

            // Check if participant exists
            if !call.participants.iter().any(|p| p.id == participant_id) {
                return Err(CallError::ParticipantNotFound(participant_id.to_string()));
            }
        }

        // Update the participant's role
        {
            let mut state = self.state.write().await;
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call
                    .participants
                    .iter_mut()
                    .find(|p| p.id == participant_id)
            {
                p.role = new_role;
            }
            if let Some(p) = state
                .participants
                .iter_mut()
                .find(|p| p.id == participant_id)
            {
                p.role = new_role;
            }
        }
        self.broadcast().await;

        debug!(participant_id = %participant_id, role = ?new_role, "Promoted participant");
        Ok(())
    }

    /// Lock or unlock the call (prevent new participants from joining).
    ///
    /// # Arguments
    ///
    /// * `locked` - Whether to lock (true) or unlock (false) the call.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if not in a call.
    /// Returns [`CallError::PermissionDenied`] if the caller is not host/co-host.
    #[instrument(skip(self), name = "ui.call.set_call_locked", fields(locked))]
    pub async fn set_call_locked(&self, locked: bool) -> Result<(), CallError> {
        // Check permissions
        {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;

            if !call.am_i_elevated() {
                return Err(CallError::PermissionDenied(
                    "Only hosts and co-hosts can lock/unlock the call".to_string(),
                ));
            }
        }

        // Update lock state
        {
            let mut state = self.state.write().await;
            if let Some(ref mut call) = state.current_call {
                call.is_locked = locked;
            }
        }
        self.broadcast().await;

        debug!(locked = locked, "Call lock state updated");
        Ok(())
    }

    /// Raise or lower hand.
    ///
    /// # Arguments
    ///
    /// * `raised` - Whether to raise (true) or lower (false) hand.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if not in a call.
    #[instrument(skip(self), name = "ui.call.set_hand_raised", fields(raised))]
    pub async fn set_hand_raised(&self, raised: bool) -> Result<(), CallError> {
        let my_id = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;
            call.my_participant_id.clone()
        };

        // Update hand raised state
        self.update_my_participant(&my_id, |p| {
            p.hand_raised = raised;
        })
        .await;
        self.broadcast().await;

        debug!(raised = raised, "Hand raised state updated");
        Ok(())
    }

    /// Lower another participant's hand (host/co-host only).
    ///
    /// # Arguments
    ///
    /// * `participant_id` - The ID of the participant whose hand to lower.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if not in a call.
    /// Returns [`CallError::PermissionDenied`] if the caller lacks permission.
    /// Returns [`CallError::ParticipantNotFound`] if the participant is not in the call.
    #[instrument(skip(self), name = "ui.call.lower_hand", fields(participant_id))]
    pub async fn lower_hand(&self, participant_id: &str) -> Result<(), CallError> {
        // Check permissions
        {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;

            if !call.am_i_elevated() {
                return Err(CallError::PermissionDenied(
                    "Only hosts and co-hosts can lower others' hands".to_string(),
                ));
            }

            if !call.participants.iter().any(|p| p.id == participant_id) {
                return Err(CallError::ParticipantNotFound(participant_id.to_string()));
            }
        }

        // Update hand raised state
        {
            let mut state = self.state.write().await;
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call
                    .participants
                    .iter_mut()
                    .find(|p| p.id == participant_id)
            {
                p.hand_raised = false;
            }
            if let Some(p) = state
                .participants
                .iter_mut()
                .find(|p| p.id == participant_id)
            {
                p.hand_raised = false;
            }
        }
        self.broadcast().await;

        debug!(participant_id = %participant_id, "Lowered participant's hand");
        Ok(())
    }

    /// Get the current user's role in the call.
    pub fn get_my_role(&self) -> ParticipantRole {
        self.rx
            .borrow()
            .call_info
            .as_ref()
            .map(|c| c.my_role())
            .unwrap_or(ParticipantRole::Participant)
    }

    /// Check if the current user has elevated privileges (host or co-host).
    pub fn am_i_elevated(&self) -> bool {
        self.rx
            .borrow()
            .call_info
            .as_ref()
            .map(|c| c.am_i_elevated())
            .unwrap_or(false)
    }

    /// Check if the current user is the host.
    pub fn am_i_host(&self) -> bool {
        self.rx
            .borrow()
            .call_info
            .as_ref()
            .map(|c| c.am_i_host())
            .unwrap_or(false)
    }

    /// Get the current call type.
    pub fn get_call_type(&self) -> Option<CallType> {
        self.rx.borrow().call_info.as_ref().map(|c| c.call_type)
    }

    /// Check if this is a group call.
    pub fn is_group_call(&self) -> bool {
        self.rx
            .borrow()
            .call_info
            .as_ref()
            .map(|c| c.is_group_call())
            .unwrap_or(false)
    }

    /// Get the current call info.
    pub fn get_current_call(&self) -> Option<CallInfo> {
        self.rx.borrow().call_info.clone()
    }

    // ===== Call Controls =====

    /// Toggle mute state and return the new muted state.
    ///
    /// This method toggles the local audio mute state by executing `Command::ToggleAudio`
    /// through the Communitas core. The state is updated when the `Event::AudioToggled`
    /// response is received.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.toggle_mute")]
    pub async fn toggle_mute(&self) -> Result<bool, CallError> {
        // Extract call_id and participant info from state
        let (call_id, my_id, current_is_muted) = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;
            let my_id = call.my_participant_id.clone();
            let is_muted = state
                .participants
                .iter()
                .find(|p| p.id == my_id)
                .map(|p| p.is_muted)
                .unwrap_or(false);
            (call.call_id.clone(), my_id, is_muted)
        };

        // Calculate new enabled state: if muted, enable audio; if not muted, disable audio
        let new_enabled = current_is_muted;

        // Build and execute the ToggleAudio command
        let cmd = Command::ToggleAudio {
            call_id: call_id.clone(),
            enabled: new_enabled,
        };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Find the AudioToggled event in the response
        let toggled_enabled = events.iter().find_map(|event| match event {
            Event::AudioToggled {
                call_id: id,
                enabled,
            } if *id == call_id => Some(*enabled),
            _ => None,
        });

        let final_enabled = match toggled_enabled {
            Some(enabled) => enabled,
            None => {
                warn!("No AudioToggled event returned from core, using expected state");
                new_enabled
            }
        };

        // Update state: is_muted is the inverse of enabled
        let new_muted = !final_enabled;
        self.update_my_participant(&my_id, |p| p.is_muted = new_muted)
            .await;
        self.broadcast().await;

        debug!(call_id = %call_id, muted = %new_muted, "Audio toggled successfully");
        Ok(new_muted)
    }

    /// Toggle video state and return the new enabled state.
    ///
    /// This method toggles the local video state by executing `Command::ToggleVideo`
    /// through the Communitas core. The state is updated when the `Event::VideoToggled`
    /// response is received.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.toggle_video")]
    pub async fn toggle_video(&self) -> Result<bool, CallError> {
        // Extract call_id and participant info from state
        let (call_id, my_id, current_video_enabled) = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;
            let my_id = call.my_participant_id.clone();
            let is_video_enabled = state
                .participants
                .iter()
                .find(|p| p.id == my_id)
                .map(|p| p.is_video_enabled)
                .unwrap_or(false);
            (call.call_id.clone(), my_id, is_video_enabled)
        };

        // Calculate new enabled state: toggle current state
        let new_enabled = !current_video_enabled;

        // Build and execute the ToggleVideo command
        let cmd = Command::ToggleVideo {
            call_id: call_id.clone(),
            enabled: new_enabled,
        };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Find the VideoToggled event in the response
        let toggled_enabled = events.iter().find_map(|event| match event {
            Event::VideoToggled {
                call_id: id,
                enabled,
            } if *id == call_id => Some(*enabled),
            _ => None,
        });

        let final_enabled = match toggled_enabled {
            Some(enabled) => enabled,
            None => {
                warn!("No VideoToggled event returned from core, using expected state");
                new_enabled
            }
        };

        // Update state
        self.update_my_participant(&my_id, |p| p.is_video_enabled = final_enabled)
            .await;
        self.broadcast().await;

        debug!(call_id = %call_id, video_enabled = %final_enabled, "Video toggled successfully");
        Ok(final_enabled)
    }

    /// Start screen sharing in the current call.
    ///
    /// This method starts screen sharing by executing `Command::StartScreenShare` through
    /// the Communitas core. The state is updated when the `Event::ScreenShareStarted`
    /// response is received.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.start_screen_share")]
    pub async fn start_screen_share(&self) -> Result<(), CallError> {
        // Extract call_id and my_id from state
        let (call_id, my_id) = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;
            (call.call_id.clone(), call.my_participant_id.clone())
        };

        // Build and execute the StartScreenShare command
        let cmd = Command::StartScreenShare {
            call_id: call_id.clone(),
        };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Find the ScreenShareStarted event in the response
        let screen_share_started = events.iter().any(
            |event| matches!(event, Event::ScreenShareStarted { call_id: id } if *id == call_id),
        );

        if !screen_share_started {
            warn!("No ScreenShareStarted event returned from core, updating state anyway");
        }

        // Update state in single lock acquisition
        {
            let mut state = self.state.write().await;
            state.is_screen_sharing = true;
            if let Some(participant) = state.participants.iter_mut().find(|p| p.id == my_id) {
                participant.is_screen_sharing = true;
            }
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call.participants.iter_mut().find(|p| p.id == my_id)
            {
                p.is_screen_sharing = true;
            }
        }
        self.broadcast().await;

        debug!(call_id = %call_id, "Screen share started successfully");
        Ok(())
    }

    /// Stop screen sharing in the current call.
    ///
    /// This method stops screen sharing by executing `Command::StopScreenShare` through
    /// the Communitas core. The state is updated when the `Event::ScreenShareStopped`
    /// response is received.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
    /// Returns [`CallError::CoreError`] if the core command execution fails.
    #[instrument(skip(self), name = "ui.call.stop_screen_share")]
    pub async fn stop_screen_share(&self) -> Result<(), CallError> {
        // Extract call_id and my_id from state
        let (call_id, my_id) = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            let call = state.current_call.as_ref().ok_or(CallError::NotInCall)?;
            (call.call_id.clone(), call.my_participant_id.clone())
        };

        // Build and execute the StopScreenShare command
        let cmd = Command::StopScreenShare {
            call_id: call_id.clone(),
        };

        let events = match self.app.execute(cmd).await {
            Ok(events) => events,
            Err(e) => {
                return Err(CallError::CoreError(e.message.clone()));
            }
        };

        // Find the ScreenShareStopped event in the response
        let screen_share_stopped = events.iter().any(
            |event| matches!(event, Event::ScreenShareStopped { call_id: id } if *id == call_id),
        );

        if !screen_share_stopped {
            warn!("No ScreenShareStopped event returned from core, updating state anyway");
        }

        // Update state in single lock acquisition
        {
            let mut state = self.state.write().await;
            state.is_screen_sharing = false;
            if let Some(participant) = state.participants.iter_mut().find(|p| p.id == my_id) {
                participant.is_screen_sharing = false;
            }
            if let Some(ref mut call) = state.current_call
                && let Some(p) = call.participants.iter_mut().find(|p| p.id == my_id)
            {
                p.is_screen_sharing = false;
            }
        }
        self.broadcast().await;

        debug!(call_id = %call_id, "Screen share stopped successfully");
        Ok(())
    }

    /// Set audio input enabled state.
    ///
    /// # Errors
    ///
    /// Returns [`CallError::NotInCall`] if the user is not currently in a call.
    #[instrument(skip(self), name = "ui.call.set_audio_input")]
    pub async fn set_audio_input_enabled(&self, enabled: bool) -> Result<(), CallError> {
        // Extract my_id while validating call state
        let my_id = {
            let state = self.state.read().await;
            if !state.call_state.is_active() {
                return Err(CallError::NotInCall);
            }
            state
                .current_call
                .as_ref()
                .map(|c| c.my_participant_id.clone())
                .ok_or(CallError::NotInCall)?
        };

        self.update_my_participant(&my_id, |p| p.is_muted = !enabled)
            .await;
        self.broadcast().await;
        Ok(())
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

    /// Handle disconnection of a media device during a call.
    ///
    /// This method should be called when a device is disconnected while the application
    /// is running. It reports a `DeviceNotFound` error, clears the selected device from
    /// settings, and enters listen-only mode if the disconnected device was a microphone.
    ///
    /// # Arguments
    ///
    /// * `device_id` - The ID of the disconnected device.
    /// * `device_type` - The type of the disconnected device.
    #[instrument(skip(self), name = "ui.call.handle_device_disconnection", fields(?device_id, ?device_type))]
    pub async fn handle_device_disconnection(&self, device_id: &str, device_type: DeviceType) {
        warn!(
            device_id = %device_id,
            device_type = ?device_type,
            "Device disconnected"
        );

        // Report the device not found error
        let error = MediaError::new(
            device_type,
            MediaErrorKind::DeviceNotFound,
            format!("Device '{}' was disconnected", device_id),
        );

        {
            let mut state = self.state.write().await;
            state.media_errors.push(error);

            // Clear the selected device setting
            match device_type {
                DeviceType::Microphone => {
                    state.settings.selected_microphone = None;
                    // Enter listen-only mode when microphone is lost
                    state.listen_only_mode = true;
                }
                DeviceType::Speaker => {
                    state.settings.selected_speaker = None;
                }
                DeviceType::Camera => {
                    state.settings.selected_camera = None;
                }
            }

            // Remove the device from available devices
            state.available_devices.retain(|d| d.id != device_id);
        }

        self.broadcast().await;
    }

    /// Handle device changes (hot-plug detection).
    ///
    /// This method should be called by the platform layer when media devices are
    /// added or removed (e.g., USB microphone plugged in/unplugged). It refreshes
    /// the device list and checks if any currently selected devices are missing.
    ///
    /// If a selected device is no longer available, [`handle_device_disconnection`]
    /// is called to report the error and update settings.
    #[instrument(skip(self), name = "ui.call.on_devices_changed")]
    pub async fn on_devices_changed(&self) {
        debug!("Device change detected, refreshing device list");

        // Try to enumerate devices, tolerating failures
        let new_devices = match self.device_enumerator.enumerate_devices().await {
            Ok(devices) => devices,
            Err(e) => {
                error!(error = %e, "Failed to enumerate devices after device change");
                // Report the enumeration failure as a media error
                let error = MediaError::new(
                    DeviceType::Microphone, // Use microphone as generic device type
                    MediaErrorKind::Unknown,
                    format!("Failed to enumerate devices: {}", e),
                );
                self.report_media_error(error).await;
                return;
            }
        };

        // Check which selected devices are missing from the new device list
        let (missing_mic, missing_speaker, missing_camera) = {
            let state = self.state.read().await;
            (
                find_missing_device(
                    &state.settings.selected_microphone,
                    &new_devices,
                    DeviceType::Microphone,
                ),
                find_missing_device(
                    &state.settings.selected_speaker,
                    &new_devices,
                    DeviceType::Speaker,
                ),
                find_missing_device(
                    &state.settings.selected_camera,
                    &new_devices,
                    DeviceType::Camera,
                ),
            )
        };

        // Update the available devices
        {
            let mut state = self.state.write().await;
            state.available_devices = new_devices;
        }

        // Handle disconnections for missing devices
        if let Some(mic_id) = missing_mic {
            self.handle_device_disconnection(&mic_id, DeviceType::Microphone)
                .await;
        }
        if let Some(speaker_id) = missing_speaker {
            self.handle_device_disconnection(&speaker_id, DeviceType::Speaker)
                .await;
        }
        if let Some(camera_id) = missing_camera {
            self.handle_device_disconnection(&camera_id, DeviceType::Camera)
                .await;
        }

        self.broadcast().await;
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

    // ─────────────────────────────────────────────────────────────────────────
    // Quality Metrics
    // ─────────────────────────────────────────────────────────────────────────

    /// Get current overall quality metrics.
    pub fn get_quality_metrics(&self) -> QualityMetrics {
        self.rx.borrow().quality_metrics.clone()
    }

    /// Get current connection quality level.
    pub fn get_connection_quality(&self) -> ConnectionQuality {
        self.rx.borrow().quality_metrics.quality
    }

    /// Update overall quality metrics (called by WebRTC stats collector).
    #[instrument(skip(self, metrics), name = "ui.call.update_quality_metrics")]
    pub async fn update_quality_metrics(&self, metrics: QualityMetrics) {
        let mut state = self.state.write().await;
        state.quality_metrics = metrics;
        drop(state);
        self.broadcast().await;
    }

    /// Update quality metrics from raw values (convenience method).
    #[instrument(skip(self), name = "ui.call.update_quality_from_stats")]
    pub async fn update_quality_from_stats(
        &self,
        latency_ms: u32,
        packet_loss_percent: f32,
        jitter_ms: u32,
        audio_bitrate_kbps: u32,
        video_bitrate_kbps: u32,
    ) {
        let quality = ConnectionQuality::from_metrics(latency_ms, packet_loss_percent);
        let timestamp = current_timestamp_millis() as u64;

        let metrics = QualityMetrics {
            latency_ms,
            packet_loss_percent,
            jitter_ms,
            audio_bitrate_kbps,
            video_bitrate_kbps,
            quality,
            timestamp,
            ..Default::default()
        };

        self.update_quality_metrics(metrics).await;
    }

    /// Update video quality metrics.
    #[instrument(skip(self), name = "ui.call.update_video_quality")]
    pub async fn update_video_quality(&self, width: u32, height: u32, fps: u32, bitrate_kbps: u32) {
        let mut state = self.state.write().await;
        state.quality_metrics.video_width = width;
        state.quality_metrics.video_height = height;
        state.quality_metrics.video_fps = fps;
        state.quality_metrics.video_bitrate_kbps = bitrate_kbps;
        state.quality_metrics.timestamp = current_timestamp_millis() as u64;
        drop(state);
        self.broadcast().await;
    }

    /// Update participant-specific quality metrics.
    #[instrument(skip(self, incoming), name = "ui.call.update_participant_quality")]
    pub async fn update_participant_quality(
        &self,
        participant_id: &str,
        incoming: QualityMetrics,
        outgoing: Option<QualityMetrics>,
    ) {
        let mut state = self.state.write().await;

        // Find and update or insert participant quality
        if let Some(pq) = state
            .participant_quality
            .iter_mut()
            .find(|q| q.participant_id == participant_id)
        {
            pq.incoming = incoming;
            pq.outgoing = outgoing;
        } else {
            state.participant_quality.push(ParticipantQuality {
                participant_id: participant_id.to_string(),
                incoming,
                outgoing,
            });
        }

        drop(state);
        self.broadcast().await;
    }

    /// Get quality metrics for a specific participant.
    pub fn get_participant_quality(&self, participant_id: &str) -> Option<ParticipantQuality> {
        self.rx
            .borrow()
            .participant_quality
            .iter()
            .find(|q| q.participant_id == participant_id)
            .cloned()
    }

    /// Clear all quality metrics (e.g., when leaving a call).
    #[instrument(skip(self), name = "ui.call.clear_quality_metrics")]
    pub async fn clear_quality_metrics(&self) {
        let mut state = self.state.write().await;
        state.quality_metrics = QualityMetrics::default();
        state.participant_quality.clear();
        drop(state);
        self.broadcast().await;
    }

    /// Update bandwidth usage statistics.
    #[instrument(skip(self), name = "ui.call.update_bandwidth_stats")]
    pub async fn update_bandwidth_stats(&self, bytes_sent: u64, bytes_received: u64) {
        let mut state = self.state.write().await;
        state.quality_metrics.bytes_sent = bytes_sent;
        state.quality_metrics.bytes_received = bytes_received;
        state.quality_metrics.timestamp = current_timestamp_millis() as u64;
        drop(state);
        self.broadcast().await;
    }

    /// Check if quality issues should trigger an alert.
    pub fn should_show_quality_warning(&self) -> bool {
        let snap = self.rx.borrow();
        matches!(
            snap.quality_metrics.quality,
            ConnectionQuality::Poor | ConnectionQuality::Critical
        )
    }

    // ===== Recording Management =====

    /// Start recording the current call.
    ///
    /// Returns an error if not currently in a call or if recording is already active.
    #[instrument(skip(self), name = "ui.call.start_recording")]
    pub async fn start_recording(&self, include_video: bool) -> Result<(), CallError> {
        let state = self.state.read().await;
        if state.call_state != CallState::InCall {
            return Err(CallError::NotInCall);
        }
        if state.is_recording {
            return Err(CallError::MediaError(
                "Recording already active".to_string(),
            ));
        }
        drop(state);

        // Get the current user identity for the recording info
        let started_by = self.get_my_identity().await.unwrap_or_default();

        // Generate recording ID
        let recording_id = format!("rec-{}", current_timestamp_millis());
        let started_at = current_timestamp_millis() as u64;

        let recording_info = RecordingInfo {
            id: recording_id,
            started_at,
            duration_ms: 0,
            state: RecordingState::Recording,
            file_path: None,
            file_size_bytes: 0,
            includes_audio: true,
            includes_video: include_video,
            includes_screen: false,
            started_by,
        };

        let mut state = self.state.write().await;
        state.is_recording = true;
        state.recording_info = Some(recording_info);
        drop(state);

        self.broadcast().await;
        debug!("Recording started");
        Ok(())
    }

    /// Stop recording and finalize the file.
    #[instrument(skip(self), name = "ui.call.stop_recording")]
    pub async fn stop_recording(&self) -> Result<Option<RecordingInfo>, CallError> {
        let state = self.state.read().await;
        if !state.is_recording {
            return Err(CallError::MediaError("Not recording".to_string()));
        }
        drop(state);

        // Set state to finalizing
        let mut state = self.state.write().await;
        if let Some(ref mut info) = state.recording_info {
            info.state = RecordingState::Finalizing;
            // Calculate final duration
            let now = current_timestamp_millis() as u64;
            info.duration_ms = now.saturating_sub(info.started_at);
        }
        drop(state);
        self.broadcast().await;

        // In a real implementation, we would finalize the recording file here
        // For now, we just update the state

        let mut state = self.state.write().await;
        let final_info = state.recording_info.take();
        state.is_recording = false;
        drop(state);

        self.broadcast().await;
        debug!("Recording stopped");
        Ok(final_info)
    }

    /// Pause an active recording.
    #[instrument(skip(self), name = "ui.call.pause_recording")]
    pub async fn pause_recording(&self) -> Result<(), CallError> {
        let mut state = self.state.write().await;
        if !state.is_recording {
            return Err(CallError::MediaError("Not recording".to_string()));
        }

        if let Some(ref mut info) = state.recording_info {
            if info.state != RecordingState::Recording {
                return Err(CallError::MediaError("Recording not active".to_string()));
            }
            info.state = RecordingState::Paused;
        }
        drop(state);

        self.broadcast().await;
        debug!("Recording paused");
        Ok(())
    }

    /// Resume a paused recording.
    #[instrument(skip(self), name = "ui.call.resume_recording")]
    pub async fn resume_recording(&self) -> Result<(), CallError> {
        let mut state = self.state.write().await;
        if !state.is_recording {
            return Err(CallError::MediaError("Not recording".to_string()));
        }

        if let Some(ref mut info) = state.recording_info {
            if info.state != RecordingState::Paused {
                return Err(CallError::MediaError("Recording not paused".to_string()));
            }
            info.state = RecordingState::Recording;
        }
        drop(state);

        self.broadcast().await;
        debug!("Recording resumed");
        Ok(())
    }

    /// Update recording duration and file size.
    ///
    /// Called periodically during recording to update stats.
    #[instrument(skip(self), name = "ui.call.update_recording_stats")]
    pub async fn update_recording_stats(&self, file_size_bytes: u64) {
        let mut state = self.state.write().await;
        if let Some(ref mut info) = state.recording_info {
            let now = current_timestamp_millis() as u64;
            info.duration_ms = now.saturating_sub(info.started_at);
            info.file_size_bytes = file_size_bytes;
        }
        drop(state);
        self.broadcast().await;
    }

    /// Get the current recording state.
    pub fn get_recording_state(&self) -> RecordingState {
        self.rx
            .borrow()
            .recording_info
            .as_ref()
            .map(|r| r.state)
            .unwrap_or(RecordingState::NotRecording)
    }

    /// Check if recording is active.
    pub fn is_recording(&self) -> bool {
        self.rx.borrow().is_recording
    }

    /// Get the current recording info if available.
    pub fn get_recording_info(&self) -> Option<RecordingInfo> {
        self.rx.borrow().recording_info.clone()
    }

    /// Get the current user's identity (four_words) from the call info or auth state.
    async fn get_my_identity(&self) -> Option<String> {
        // First try to get from current call
        let state = self.state.read().await;
        if let Some(ref call) = state.current_call {
            // Find the participant that matches the current user
            if let Some(participant) = call
                .participants
                .iter()
                .find(|p| p.id == call.my_participant_id)
            {
                return Some(participant.four_words.clone());
            }
        }
        drop(state);

        // Otherwise get from auth state
        let auth_snap = self.auth.subscribe().borrow().clone();
        if let AuthStateSnapshot::Authenticated { session, .. } = auth_snap {
            return Some(session.four_words);
        }
        None
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
    async fn start_screen_share_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.start_screen_share().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn stop_screen_share_fails_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.stop_screen_share().await;
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

    // ===== Query Tests =====

    #[tokio::test]
    async fn query_call_status_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.query_call_status("call1").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    #[tokio::test]
    async fn query_call_participants_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.query_call_participants("call1").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    #[tokio::test]
    async fn list_active_calls_requires_auth() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.list_active_calls().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotAuthenticated));
    }

    // ===== Device Disconnection Tests =====

    #[tokio::test]
    async fn handle_device_disconnection_reports_error() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Initially no errors
        assert!(service.get_media_errors().is_empty());

        // Simulate device disconnection
        service
            .handle_device_disconnection("mic-1", DeviceType::Microphone)
            .await;

        let errors = service.get_media_errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].device_type, DeviceType::Microphone);
        assert_eq!(errors[0].error_kind, MediaErrorKind::DeviceNotFound);
        assert!(errors[0].message.contains("mic-1"));
    }

    #[tokio::test]
    async fn handle_device_disconnection_clears_selection() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Set up initial settings with selected microphone
        let settings = CallSettings {
            selected_microphone: Some("mic-1".to_string()),
            selected_speaker: Some("speaker-1".to_string()),
            selected_camera: None,
            ..Default::default()
        };
        service
            .update_settings(settings)
            .await
            .expect("should update");

        // Verify microphone is selected
        assert_eq!(
            service.get_settings().selected_microphone,
            Some("mic-1".to_string())
        );

        // Disconnect the microphone
        service
            .handle_device_disconnection("mic-1", DeviceType::Microphone)
            .await;

        // Microphone selection should be cleared
        assert!(service.get_settings().selected_microphone.is_none());
        // Speaker should remain selected
        assert_eq!(
            service.get_settings().selected_speaker,
            Some("speaker-1".to_string())
        );
        // Should enter listen-only mode because microphone was lost
        assert!(service.is_listen_only());
    }

    #[tokio::test]
    async fn on_devices_changed_detects_missing_device() {
        let temp = TempDir::new().expect("temp dir");
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

        // Create service with mock enumerator (has mock-mic-default, etc.)
        let service = CallService::new(auth, app);

        // Initially populate devices
        let _ = service.list_devices().await; // Will fail due to no auth, but sets up mock devices internally

        // Set selected microphone to a mock device
        {
            let mut state = service.state.write().await;
            state.available_devices = vec![MediaDevice {
                id: "mic-to-remove".to_string(),
                name: "Microphone to Remove".to_string(),
                device_type: DeviceType::Microphone,
                is_default: true,
                is_available: true,
            }];
            state.settings.selected_microphone = Some("mic-to-remove".to_string());
        }

        // Broadcast to sync watch channel with state
        service.broadcast().await;

        // Verify microphone is selected
        assert_eq!(
            service.get_settings().selected_microphone,
            Some("mic-to-remove".to_string())
        );

        // Simulate device change - the mock enumerator will return different devices
        // (mock-mic-default, etc.), so "mic-to-remove" will be missing
        service.on_devices_changed().await;

        // The selected microphone should be cleared because it's no longer in the device list
        assert!(service.get_settings().selected_microphone.is_none());

        // Should have a DeviceNotFound error
        let errors = service.get_media_errors();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| {
            e.device_type == DeviceType::Microphone
                && e.error_kind == MediaErrorKind::DeviceNotFound
        }));

        // Should be in listen-only mode
        assert!(service.is_listen_only());
    }

    // ===== Quality Metrics Tests =====

    #[tokio::test]
    async fn quality_metrics_default() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let metrics = service.get_quality_metrics();
        assert_eq!(metrics.latency_ms, 0);
        assert_eq!(metrics.quality, ConnectionQuality::Unknown);
    }

    #[tokio::test]
    async fn update_quality_metrics_broadcasts() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;
        let mut rx = service.subscribe();

        let metrics = QualityMetrics {
            latency_ms: 50,
            packet_loss_percent: 0.5,
            jitter_ms: 10,
            audio_bitrate_kbps: 32,
            quality: ConnectionQuality::Excellent,
            timestamp: 12345,
            ..Default::default()
        };

        service.update_quality_metrics(metrics).await;
        rx.changed().await.expect("should receive update");

        let snap = rx.borrow().clone();
        assert_eq!(snap.quality_metrics.latency_ms, 50);
        assert_eq!(snap.quality_metrics.quality, ConnectionQuality::Excellent);
    }

    #[tokio::test]
    async fn update_quality_from_stats_calculates_quality() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Poor quality stats
        service.update_quality_from_stats(350, 4.0, 30, 32, 0).await;

        let metrics = service.get_quality_metrics();
        assert_eq!(metrics.latency_ms, 350);
        assert_eq!(metrics.packet_loss_percent, 4.0);
        assert_eq!(metrics.quality, ConnectionQuality::Poor);
    }

    #[tokio::test]
    async fn update_video_quality() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        service.update_video_quality(1920, 1080, 30, 1500).await;

        let metrics = service.get_quality_metrics();
        assert_eq!(metrics.video_width, 1920);
        assert_eq!(metrics.video_height, 1080);
        assert_eq!(metrics.video_fps, 30);
        assert_eq!(metrics.video_bitrate_kbps, 1500);
        assert!(metrics.has_video());
    }

    #[tokio::test]
    async fn participant_quality_tracking() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let incoming = QualityMetrics {
            latency_ms: 80,
            quality: ConnectionQuality::Good,
            ..Default::default()
        };

        service
            .update_participant_quality("alice", incoming.clone(), None)
            .await;

        let pq = service.get_participant_quality("alice");
        assert!(pq.is_some());
        assert_eq!(pq.unwrap().incoming.latency_ms, 80);

        // Non-existent participant
        assert!(service.get_participant_quality("bob").is_none());
    }

    #[tokio::test]
    async fn clear_quality_metrics() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Set some metrics
        let metrics = QualityMetrics {
            latency_ms: 100,
            quality: ConnectionQuality::Good,
            ..Default::default()
        };
        service.update_quality_metrics(metrics).await;
        service
            .update_participant_quality("alice", QualityMetrics::default(), None)
            .await;

        // Clear
        service.clear_quality_metrics().await;

        let snap = service.current_snapshot();
        assert_eq!(snap.quality_metrics.latency_ms, 0);
        assert_eq!(snap.quality_metrics.quality, ConnectionQuality::Unknown);
        assert!(snap.participant_quality.is_empty());
    }

    #[tokio::test]
    async fn quality_warning_detection() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        assert!(!service.should_show_quality_warning());

        let poor_metrics = QualityMetrics {
            latency_ms: 400,
            packet_loss_percent: 6.0,
            quality: ConnectionQuality::Poor,
            ..Default::default()
        };
        service.update_quality_metrics(poor_metrics).await;

        assert!(service.should_show_quality_warning());
    }

    #[tokio::test]
    async fn bandwidth_stats_update() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        service.update_bandwidth_stats(1_000_000, 2_500_000).await;

        let metrics = service.get_quality_metrics();
        assert_eq!(metrics.bytes_sent, 1_000_000);
        assert_eq!(metrics.bytes_received, 2_500_000);
    }

    // ===== Recording Tests =====

    #[tokio::test]
    async fn start_recording_requires_active_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.start_recording(false).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn stop_recording_requires_active_recording() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.stop_recording().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn pause_recording_requires_active_recording() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.pause_recording().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn resume_recording_requires_paused_recording() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.resume_recording().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn recording_state_default() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        assert!(!service.is_recording());
        assert_eq!(service.get_recording_state(), RecordingState::NotRecording);
        assert!(service.get_recording_info().is_none());
    }

    #[tokio::test]
    async fn update_recording_stats() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Manually set recording state for testing (simulating active recording)
        {
            let mut state = service.state.write().await;
            state.is_recording = true;
            state.recording_info = Some(RecordingInfo {
                id: "test-recording".to_string(),
                started_at: current_timestamp_millis() as u64 - 5000, // Started 5 seconds ago
                state: RecordingState::Recording,
                ..Default::default()
            });
        }
        service.broadcast().await;

        // Update stats
        service.update_recording_stats(1024 * 1024).await;

        let info = service.get_recording_info();
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.file_size_bytes, 1024 * 1024);
        assert!(info.duration_ms >= 5000); // At least 5 seconds
    }

    // ===== Group Call Tests =====

    #[tokio::test]
    async fn mute_participant_requires_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.mute_participant("participant-1", true).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn remove_participant_requires_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.remove_participant("participant-1").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn set_hand_raised_requires_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.set_hand_raised(true).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn set_call_locked_requires_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.set_call_locked(true).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn get_my_role_returns_participant_by_default() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        assert_eq!(service.get_my_role(), ParticipantRole::Participant);
        assert!(!service.am_i_elevated());
        assert!(!service.am_i_host());
    }

    #[tokio::test]
    async fn get_call_type_returns_none_when_not_in_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        assert!(service.get_call_type().is_none());
        assert!(!service.is_group_call());
    }

    #[tokio::test]
    async fn promote_participant_requires_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service
            .promote_participant("participant-1", ParticipantRole::CoHost)
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn lower_hand_requires_call() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        let result = service.lower_hand("participant-1").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CallError::NotInCall));
    }

    #[tokio::test]
    async fn mute_participant_permission_check() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Set up a group call state where we are NOT the host
        {
            let mut state = service.state.write().await;
            state.call_state = CallState::InCall;
            state.current_call = Some(CallInfo {
                call_id: "test-call".to_string(),
                entity_id: "entity-1".to_string(),
                entity_name: "Test Group".to_string(),
                call_type: CallType::Group,
                participants: vec![
                    Participant {
                        id: "me".to_string(),
                        display_name: "Me".to_string(),
                        four_words: "a-b-c-d".to_string(),
                        role: ParticipantRole::Participant, // Not elevated
                        is_muted: false,
                        is_muted_by_host: false,
                        is_video_enabled: false,
                        is_speaking: false,
                        is_screen_sharing: false,
                        hand_raised: false,
                        audio_level: 0.0,
                        joined_at: 0,
                    },
                    Participant {
                        id: "other".to_string(),
                        display_name: "Other".to_string(),
                        four_words: "e-f-g-h".to_string(),
                        role: ParticipantRole::Host,
                        is_muted: false,
                        is_muted_by_host: false,
                        is_video_enabled: false,
                        is_speaking: false,
                        is_screen_sharing: false,
                        hand_raised: false,
                        audio_level: 0.0,
                        joined_at: 0,
                    },
                ],
                started_at: 0,
                duration_seconds: 0,
                my_participant_id: "me".to_string(),
                host_id: "other".to_string(),
                max_participants: 25,
                is_locked: false,
                mute_on_entry: false,
            });
            state.participants = state.current_call.as_ref().unwrap().participants.clone();
        }

        // Try to mute - should fail with permission denied
        let result = service.mute_participant("other", true).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CallError::PermissionDenied(_)
        ));
    }

    #[tokio::test]
    async fn host_can_mute_participant() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Set up a group call state where we ARE the host
        {
            let mut state = service.state.write().await;
            state.call_state = CallState::InCall;
            state.current_call = Some(CallInfo {
                call_id: "test-call".to_string(),
                entity_id: "entity-1".to_string(),
                entity_name: "Test Group".to_string(),
                call_type: CallType::Group,
                participants: vec![
                    Participant {
                        id: "me".to_string(),
                        display_name: "Me".to_string(),
                        four_words: "a-b-c-d".to_string(),
                        role: ParticipantRole::Host,
                        is_muted: false,
                        is_muted_by_host: false,
                        is_video_enabled: false,
                        is_speaking: false,
                        is_screen_sharing: false,
                        hand_raised: false,
                        audio_level: 0.0,
                        joined_at: 0,
                    },
                    Participant {
                        id: "other".to_string(),
                        display_name: "Other".to_string(),
                        four_words: "e-f-g-h".to_string(),
                        role: ParticipantRole::Participant,
                        is_muted: false,
                        is_muted_by_host: false,
                        is_video_enabled: false,
                        is_speaking: false,
                        is_screen_sharing: false,
                        hand_raised: false,
                        audio_level: 0.0,
                        joined_at: 0,
                    },
                ],
                started_at: 0,
                duration_seconds: 0,
                my_participant_id: "me".to_string(),
                host_id: "me".to_string(),
                max_participants: 25,
                is_locked: false,
                mute_on_entry: false,
            });
            state.participants = state.current_call.as_ref().unwrap().participants.clone();
        }

        // Host can mute others
        let result = service.mute_participant("other", true).await;
        assert!(result.is_ok());

        // Verify participant is now muted by host
        let snap = service.current_snapshot();
        let other = snap.participants.iter().find(|p| p.id == "other");
        assert!(other.is_some());
        let other = other.unwrap();
        assert!(other.is_muted);
        assert!(other.is_muted_by_host);
    }

    #[tokio::test]
    async fn cannot_remove_host() {
        let temp = TempDir::new().expect("temp dir");
        let service = make_service(&temp).await;

        // Set up a group call where we are co-host trying to remove host
        {
            let mut state = service.state.write().await;
            state.call_state = CallState::InCall;
            state.current_call = Some(CallInfo {
                call_id: "test-call".to_string(),
                entity_id: "entity-1".to_string(),
                entity_name: "Test Group".to_string(),
                call_type: CallType::Group,
                participants: vec![
                    Participant {
                        id: "me".to_string(),
                        display_name: "Me".to_string(),
                        four_words: "a-b-c-d".to_string(),
                        role: ParticipantRole::CoHost,
                        is_muted: false,
                        is_muted_by_host: false,
                        is_video_enabled: false,
                        is_speaking: false,
                        is_screen_sharing: false,
                        hand_raised: false,
                        audio_level: 0.0,
                        joined_at: 0,
                    },
                    Participant {
                        id: "host".to_string(),
                        display_name: "Host".to_string(),
                        four_words: "e-f-g-h".to_string(),
                        role: ParticipantRole::Host,
                        is_muted: false,
                        is_muted_by_host: false,
                        is_video_enabled: false,
                        is_speaking: false,
                        is_screen_sharing: false,
                        hand_raised: false,
                        audio_level: 0.0,
                        joined_at: 0,
                    },
                ],
                started_at: 0,
                duration_seconds: 0,
                my_participant_id: "me".to_string(),
                host_id: "host".to_string(),
                max_participants: 25,
                is_locked: false,
                mute_on_entry: false,
            });
            state.participants = state.current_call.as_ref().unwrap().participants.clone();
        }

        // Try to remove host - should fail
        let result = service.remove_participant("host").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CallError::PermissionDenied(_)
        ));
    }
}
