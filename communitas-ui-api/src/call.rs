//! Call and WebRTC related DTOs for real-time communication.

use serde::{Deserialize, Serialize};

/// State of the current call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CallState {
    /// No active call.
    #[default]
    Idle,
    /// Establishing connection.
    Connecting,
    /// Active call in progress.
    InCall,
    /// Connection lost, attempting to reconnect.
    Reconnecting,
    /// Call ended normally or connection failed.
    Disconnected,
    /// Media capture failed.
    MediaError,
}

impl CallState {
    /// Returns true if this state represents an active or connecting call.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Connecting | Self::InCall | Self::Reconnecting)
    }

    /// Returns true if this state indicates the call has ended.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Idle | Self::Disconnected | Self::MediaError)
    }
}

/// State of call recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecordingState {
    /// Not recording.
    #[default]
    NotRecording,
    /// Recording in progress.
    Recording,
    /// Recording paused.
    Paused,
    /// Recording stopped, file being finalized.
    Finalizing,
}

impl RecordingState {
    /// Returns true if recording is active (recording or paused).
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Recording | Self::Paused | Self::Finalizing)
    }

    /// Returns a human-readable label for the recording state.
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotRecording => "Not Recording",
            Self::Recording => "Recording",
            Self::Paused => "Paused",
            Self::Finalizing => "Saving...",
        }
    }

    /// Returns a CSS class for UI styling.
    pub fn status_class(&self) -> &'static str {
        match self {
            Self::NotRecording => "text-slate-400",
            Self::Recording => "text-red-500 animate-pulse",
            Self::Paused => "text-amber-500",
            Self::Finalizing => "text-blue-500",
        }
    }
}

/// Information about an ongoing or completed recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingInfo {
    /// Unique recording identifier.
    pub id: String,
    /// Start timestamp (ms since epoch).
    pub started_at: u64,
    /// Total duration recorded (ms), excluding paused time.
    pub duration_ms: u64,
    /// Current recording state.
    pub state: RecordingState,
    /// File path where recording will be saved (if known).
    pub file_path: Option<String>,
    /// Estimated file size in bytes.
    pub file_size_bytes: u64,
    /// Whether audio is being recorded.
    pub includes_audio: bool,
    /// Whether video is being recorded.
    pub includes_video: bool,
    /// Whether screen share is being recorded.
    pub includes_screen: bool,
    /// Participant who started the recording.
    pub started_by: String,
}

impl Default for RecordingInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            started_at: 0,
            duration_ms: 0,
            state: RecordingState::NotRecording,
            file_path: None,
            file_size_bytes: 0,
            includes_audio: true,
            includes_video: false,
            includes_screen: false,
            started_by: String::new(),
        }
    }
}

impl RecordingInfo {
    /// Returns the formatted duration as MM:SS.
    pub fn formatted_duration(&self) -> String {
        let total_seconds = self.duration_ms / 1000;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Returns the formatted file size (KB, MB, etc.).
    pub fn formatted_size(&self) -> String {
        if self.file_size_bytes < 1024 {
            format!("{} B", self.file_size_bytes)
        } else if self.file_size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.file_size_bytes as f64 / 1024.0)
        } else if self.file_size_bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", self.file_size_bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!(
                "{:.2} GB",
                self.file_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        }
    }
}

/// Type of media device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Microphone,
    Speaker,
    Camera,
}

impl DeviceType {
    /// Returns a human-readable label for the device type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Microphone => "Microphone",
            Self::Speaker => "Speaker",
            Self::Camera => "Camera",
        }
    }
}

/// Information about an available media device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaDevice {
    /// Unique device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Type of device.
    pub device_type: DeviceType,
    /// Whether this is the system default device.
    pub is_default: bool,
    /// Whether the device is currently available.
    pub is_available: bool,
}

/// A participant in a call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Participant {
    /// Unique participant identifier.
    pub id: String,
    /// Display name of the participant.
    pub display_name: String,
    /// Four-word identity.
    pub four_words: String,
    /// Whether the participant's microphone is muted.
    pub is_muted: bool,
    /// Whether the participant's camera is enabled.
    pub is_video_enabled: bool,
    /// Whether the participant is currently speaking.
    pub is_speaking: bool,
    /// Whether the participant is currently screen sharing.
    pub is_screen_sharing: bool,
    /// Current audio level (0.0 to 1.0).
    pub audio_level: f32,
    /// Timestamp when the participant joined (ms since epoch).
    pub joined_at: i64,
}

impl Participant {
    /// Returns true if this participant has active media.
    pub fn has_active_media(&self) -> bool {
        !self.is_muted || self.is_video_enabled || self.is_screen_sharing
    }
}

/// Information about an active call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallInfo {
    /// Unique call identifier.
    pub call_id: String,
    /// Entity ID this call is associated with.
    pub entity_id: String,
    /// Entity name for display.
    pub entity_name: String,
    /// Current participants in the call.
    pub participants: Vec<Participant>,
    /// Timestamp when the call started (ms since epoch).
    pub started_at: i64,
    /// Duration in seconds since call started.
    pub duration_seconds: u64,
    /// Participant ID of the current user.
    pub my_participant_id: String,
}

impl CallInfo {
    /// Returns the number of participants including the current user.
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Returns the participant for the current user.
    pub fn my_participant(&self) -> Option<&Participant> {
        self.participants
            .iter()
            .find(|p| p.id == self.my_participant_id)
    }

    /// Returns other participants (excluding the current user).
    pub fn other_participants(&self) -> Vec<&Participant> {
        self.participants
            .iter()
            .filter(|p| p.id != self.my_participant_id)
            .collect()
    }
}

/// Kind of media error that occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaErrorKind {
    /// User denied permission to access the device.
    PermissionDenied,
    /// The requested device was not found.
    DeviceNotFound,
    /// The device is in use by another application.
    DeviceInUse,
    /// The requested feature is not supported.
    NotSupported,
    /// An unknown error occurred.
    Unknown,
}

impl MediaErrorKind {
    /// Returns a human-readable description of the error kind.
    pub fn description(&self) -> &'static str {
        match self {
            Self::PermissionDenied => "Permission denied",
            Self::DeviceNotFound => "Device not found",
            Self::DeviceInUse => "Device is in use",
            Self::NotSupported => "Not supported",
            Self::Unknown => "Unknown error",
        }
    }

    /// Returns a suggested action for the user.
    pub fn suggestion(&self) -> &'static str {
        match self {
            Self::PermissionDenied => "Check your system permissions for this application",
            Self::DeviceNotFound => "Connect a microphone or camera and try again",
            Self::DeviceInUse => "Close other applications using this device",
            Self::NotSupported => "Try using a different device",
            Self::Unknown => "Try again or restart the application",
        }
    }
}

/// Information about a media capture error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaError {
    /// Type of device that failed.
    pub device_type: DeviceType,
    /// Kind of error that occurred.
    pub error_kind: MediaErrorKind,
    /// Detailed error message.
    pub message: String,
    /// Whether the error might be recoverable by retrying.
    pub recoverable: bool,
}

impl MediaError {
    /// Creates a new media error.
    pub fn new(
        device_type: DeviceType,
        error_kind: MediaErrorKind,
        message: impl Into<String>,
    ) -> Self {
        let recoverable = !matches!(error_kind, MediaErrorKind::NotSupported);
        Self {
            device_type,
            error_kind,
            message: message.into(),
            recoverable,
        }
    }
}

/// Overall quality level for connection health display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectionQuality {
    /// Quality is unknown or not yet measured.
    #[default]
    Unknown,
    /// Excellent quality (< 50ms latency, < 1% packet loss).
    Excellent,
    /// Good quality (< 150ms latency, < 2% packet loss).
    Good,
    /// Fair quality (< 300ms latency, < 5% packet loss).
    Fair,
    /// Poor quality (> 300ms latency or > 5% packet loss).
    Poor,
    /// Very poor / failing connection.
    Critical,
}

impl ConnectionQuality {
    /// Returns a human-readable label for the quality level.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unknown => "Unknown",
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
            Self::Critical => "Critical",
        }
    }

    /// Returns a color class for UI rendering (Tailwind style).
    pub fn color_class(&self) -> &'static str {
        match self {
            Self::Unknown => "text-slate-400",
            Self::Excellent => "text-emerald-400",
            Self::Good => "text-emerald-500",
            Self::Fair => "text-amber-400",
            Self::Poor => "text-orange-500",
            Self::Critical => "text-red-500",
        }
    }

    /// Returns number of "bars" to display (0-5).
    pub fn signal_bars(&self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Critical => 1,
            Self::Poor => 2,
            Self::Fair => 3,
            Self::Good => 4,
            Self::Excellent => 5,
        }
    }

    /// Determines quality level from latency and packet loss.
    pub fn from_metrics(latency_ms: u32, packet_loss_percent: f32) -> Self {
        if packet_loss_percent > 10.0 || latency_ms > 500 {
            Self::Critical
        } else if packet_loss_percent > 5.0 || latency_ms > 300 {
            Self::Poor
        } else if packet_loss_percent > 2.0 || latency_ms > 150 {
            Self::Fair
        } else if packet_loss_percent > 1.0 || latency_ms > 50 {
            Self::Good
        } else {
            Self::Excellent
        }
    }
}

/// Real-time WebRTC quality metrics for a connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Round-trip time in milliseconds.
    pub latency_ms: u32,
    /// Packet loss percentage (0.0 to 100.0).
    pub packet_loss_percent: f32,
    /// Jitter in milliseconds.
    pub jitter_ms: u32,
    /// Current audio bitrate in kbps.
    pub audio_bitrate_kbps: u32,
    /// Current video bitrate in kbps (0 if no video).
    pub video_bitrate_kbps: u32,
    /// Video resolution width (0 if no video).
    pub video_width: u32,
    /// Video resolution height (0 if no video).
    pub video_height: u32,
    /// Video framerate (0 if no video).
    pub video_fps: u32,
    /// Total bytes sent in this session.
    pub bytes_sent: u64,
    /// Total bytes received in this session.
    pub bytes_received: u64,
    /// Overall connection quality assessment.
    pub quality: ConnectionQuality,
    /// Timestamp when these metrics were collected (Unix ms).
    pub timestamp: u64,
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0,
            packet_loss_percent: 0.0,
            jitter_ms: 0,
            audio_bitrate_kbps: 0,
            video_bitrate_kbps: 0,
            video_width: 0,
            video_height: 0,
            video_fps: 0,
            bytes_sent: 0,
            bytes_received: 0,
            quality: ConnectionQuality::Unknown,
            timestamp: 0,
        }
    }
}

impl QualityMetrics {
    /// Returns true if video is active (has resolution).
    pub fn has_video(&self) -> bool {
        self.video_width > 0 && self.video_height > 0
    }

    /// Returns the video resolution as a string (e.g., "1280x720").
    pub fn video_resolution(&self) -> String {
        if self.has_video() {
            format!("{}x{}", self.video_width, self.video_height)
        } else {
            "None".to_string()
        }
    }

    /// Returns total bandwidth usage in kbps.
    pub fn total_bitrate_kbps(&self) -> u32 {
        self.audio_bitrate_kbps + self.video_bitrate_kbps
    }

    /// Recalculates the quality assessment from current metrics.
    pub fn recalculate_quality(&mut self) {
        self.quality = ConnectionQuality::from_metrics(self.latency_ms, self.packet_loss_percent);
    }
}

/// Quality metrics for a specific participant in a call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParticipantQuality {
    /// Participant ID this applies to.
    pub participant_id: String,
    /// Incoming stream quality (receiving from this participant).
    pub incoming: QualityMetrics,
    /// Outgoing stream quality (sending to this participant, if measured).
    pub outgoing: Option<QualityMetrics>,
}

/// User's call settings and preferences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CallSettings {
    /// Selected microphone device ID.
    pub selected_microphone: Option<String>,
    /// Selected speaker device ID.
    pub selected_speaker: Option<String>,
    /// Selected camera device ID.
    pub selected_camera: Option<String>,
    /// Whether to start calls with microphone muted.
    pub auto_mute_on_join: bool,
    /// Whether noise suppression is enabled.
    pub noise_suppression: bool,
}

/// Snapshot of the current call state for UI updates.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CallSnapshot {
    /// Current call state.
    pub state: CallState,
    /// Active call information (if in a call).
    pub call_info: Option<CallInfo>,
    /// List of participants (mirrored from call_info for convenience).
    pub participants: Vec<Participant>,
    /// Media errors that have occurred.
    pub media_errors: Vec<MediaError>,
    /// Available media devices.
    pub available_devices: Vec<MediaDevice>,
    /// Current call settings.
    pub settings: CallSettings,
    /// Whether in listen-only mode due to media failure.
    pub listen_only_mode: bool,
    /// Whether the current user is screen sharing.
    pub is_screen_sharing: bool,
    /// Overall connection quality metrics.
    pub quality_metrics: QualityMetrics,
    /// Per-participant quality metrics.
    pub participant_quality: Vec<ParticipantQuality>,
    /// Whether the call is being recorded.
    pub is_recording: bool,
    /// Recording information (if recording is active or recently stopped).
    pub recording_info: Option<RecordingInfo>,
}

impl CallSnapshot {
    /// Returns true if currently in an active call.
    pub fn is_in_call(&self) -> bool {
        self.state == CallState::InCall
    }

    /// Returns true if there are unrecoverable media errors.
    pub fn has_critical_errors(&self) -> bool {
        self.media_errors.iter().any(|e| !e.recoverable)
    }

    /// Returns quality metrics for a specific participant.
    pub fn get_participant_quality(&self, participant_id: &str) -> Option<&ParticipantQuality> {
        self.participant_quality
            .iter()
            .find(|q| q.participant_id == participant_id)
    }

    /// Returns the overall connection quality level.
    pub fn connection_quality(&self) -> ConnectionQuality {
        self.quality_metrics.quality
    }

    /// Returns true if connection quality is poor or worse.
    pub fn has_quality_issues(&self) -> bool {
        matches!(
            self.quality_metrics.quality,
            ConnectionQuality::Poor | ConnectionQuality::Critical
        )
    }

    /// Returns the current recording state.
    pub fn recording_state(&self) -> RecordingState {
        self.recording_info
            .as_ref()
            .map(|r| r.state)
            .unwrap_or(RecordingState::NotRecording)
    }

    /// Returns true if recording is currently active (recording or paused).
    pub fn is_recording_active(&self) -> bool {
        self.recording_state().is_active()
    }

    /// Returns the formatted recording duration, if recording.
    pub fn recording_duration(&self) -> Option<String> {
        self.recording_info.as_ref().map(|r| r.formatted_duration())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_state_is_active() {
        assert!(!CallState::Idle.is_active());
        assert!(CallState::Connecting.is_active());
        assert!(CallState::InCall.is_active());
        assert!(CallState::Reconnecting.is_active());
        assert!(!CallState::Disconnected.is_active());
        assert!(!CallState::MediaError.is_active());
    }

    #[test]
    fn call_state_is_terminal() {
        assert!(CallState::Idle.is_terminal());
        assert!(!CallState::Connecting.is_terminal());
        assert!(!CallState::InCall.is_terminal());
        assert!(!CallState::Reconnecting.is_terminal());
        assert!(CallState::Disconnected.is_terminal());
        assert!(CallState::MediaError.is_terminal());
    }

    #[test]
    fn device_type_label() {
        assert_eq!(DeviceType::Microphone.label(), "Microphone");
        assert_eq!(DeviceType::Speaker.label(), "Speaker");
        assert_eq!(DeviceType::Camera.label(), "Camera");
    }

    #[test]
    fn participant_has_active_media() {
        let participant = Participant {
            id: "p1".to_string(),
            display_name: "Alice".to_string(),
            four_words: "ocean-forest-moon-star".to_string(),
            is_muted: true,
            is_video_enabled: false,
            is_speaking: false,
            is_screen_sharing: false,
            audio_level: 0.0,
            joined_at: 0,
        };
        assert!(!participant.has_active_media());

        let participant_with_audio = Participant {
            is_muted: false,
            ..participant.clone()
        };
        assert!(participant_with_audio.has_active_media());

        let participant_with_video = Participant {
            is_video_enabled: true,
            ..participant.clone()
        };
        assert!(participant_with_video.has_active_media());

        let participant_with_screen_share = Participant {
            is_screen_sharing: true,
            ..participant
        };
        assert!(participant_with_screen_share.has_active_media());
    }

    #[test]
    fn call_info_participant_helpers() {
        let call = CallInfo {
            call_id: "call1".to_string(),
            entity_id: "ent1".to_string(),
            entity_name: "Team Chat".to_string(),
            participants: vec![
                Participant {
                    id: "me".to_string(),
                    display_name: "Me".to_string(),
                    four_words: "a-b-c-d".to_string(),
                    is_muted: false,
                    is_video_enabled: false,
                    is_speaking: false,
                    is_screen_sharing: false,
                    audio_level: 0.0,
                    joined_at: 0,
                },
                Participant {
                    id: "other".to_string(),
                    display_name: "Other".to_string(),
                    four_words: "e-f-g-h".to_string(),
                    is_muted: false,
                    is_video_enabled: false,
                    is_speaking: false,
                    is_screen_sharing: false,
                    audio_level: 0.0,
                    joined_at: 0,
                },
            ],
            started_at: 0,
            duration_seconds: 0,
            my_participant_id: "me".to_string(),
        };

        assert_eq!(call.participant_count(), 2);
        assert_eq!(
            call.my_participant().map(|p| &p.display_name),
            Some(&"Me".to_string())
        );
        assert_eq!(call.other_participants().len(), 1);
        assert_eq!(call.other_participants()[0].display_name, "Other");
    }

    #[test]
    fn media_error_kind_descriptions() {
        assert_eq!(
            MediaErrorKind::PermissionDenied.description(),
            "Permission denied"
        );
        assert_eq!(
            MediaErrorKind::DeviceNotFound.description(),
            "Device not found"
        );
        assert_eq!(
            MediaErrorKind::DeviceInUse.description(),
            "Device is in use"
        );
        assert_eq!(MediaErrorKind::NotSupported.description(), "Not supported");
        assert_eq!(MediaErrorKind::Unknown.description(), "Unknown error");
    }

    #[test]
    fn media_error_recoverable() {
        let recoverable = MediaError::new(
            DeviceType::Microphone,
            MediaErrorKind::DeviceInUse,
            "Device busy",
        );
        assert!(recoverable.recoverable);

        let not_recoverable = MediaError::new(
            DeviceType::Camera,
            MediaErrorKind::NotSupported,
            "No camera support",
        );
        assert!(!not_recoverable.recoverable);
    }

    #[test]
    fn call_snapshot_helpers() {
        let mut snapshot = CallSnapshot::default();
        assert!(!snapshot.is_in_call());
        assert!(!snapshot.has_critical_errors());
        assert!(!snapshot.is_screen_sharing);

        snapshot.state = CallState::InCall;
        assert!(snapshot.is_in_call());

        snapshot.media_errors.push(MediaError::new(
            DeviceType::Microphone,
            MediaErrorKind::NotSupported,
            "No microphone",
        ));
        assert!(snapshot.has_critical_errors());
    }

    #[test]
    fn call_settings_default() {
        let settings = CallSettings::default();
        assert!(settings.selected_microphone.is_none());
        assert!(settings.selected_speaker.is_none());
        assert!(settings.selected_camera.is_none());
        assert!(!settings.auto_mute_on_join);
        assert!(!settings.noise_suppression);
    }

    #[test]
    fn connection_quality_labels() {
        assert_eq!(ConnectionQuality::Unknown.label(), "Unknown");
        assert_eq!(ConnectionQuality::Excellent.label(), "Excellent");
        assert_eq!(ConnectionQuality::Good.label(), "Good");
        assert_eq!(ConnectionQuality::Fair.label(), "Fair");
        assert_eq!(ConnectionQuality::Poor.label(), "Poor");
        assert_eq!(ConnectionQuality::Critical.label(), "Critical");
    }

    #[test]
    fn connection_quality_signal_bars() {
        assert_eq!(ConnectionQuality::Unknown.signal_bars(), 0);
        assert_eq!(ConnectionQuality::Critical.signal_bars(), 1);
        assert_eq!(ConnectionQuality::Poor.signal_bars(), 2);
        assert_eq!(ConnectionQuality::Fair.signal_bars(), 3);
        assert_eq!(ConnectionQuality::Good.signal_bars(), 4);
        assert_eq!(ConnectionQuality::Excellent.signal_bars(), 5);
    }

    #[test]
    fn connection_quality_from_metrics() {
        // Excellent: low latency, low packet loss
        assert_eq!(
            ConnectionQuality::from_metrics(30, 0.5),
            ConnectionQuality::Excellent
        );

        // Good: moderate latency
        assert_eq!(
            ConnectionQuality::from_metrics(80, 0.8),
            ConnectionQuality::Good
        );

        // Fair: higher latency or packet loss
        assert_eq!(
            ConnectionQuality::from_metrics(200, 1.5),
            ConnectionQuality::Fair
        );

        // Poor: significant issues
        assert_eq!(
            ConnectionQuality::from_metrics(350, 3.0),
            ConnectionQuality::Poor
        );

        // Critical: severe issues
        assert_eq!(
            ConnectionQuality::from_metrics(600, 2.0),
            ConnectionQuality::Critical
        );
        assert_eq!(
            ConnectionQuality::from_metrics(100, 15.0),
            ConnectionQuality::Critical
        );
    }

    #[test]
    fn quality_metrics_default() {
        let metrics = QualityMetrics::default();
        assert_eq!(metrics.latency_ms, 0);
        assert_eq!(metrics.packet_loss_percent, 0.0);
        assert_eq!(metrics.quality, ConnectionQuality::Unknown);
        assert!(!metrics.has_video());
    }

    #[test]
    fn quality_metrics_video_detection() {
        let mut metrics = QualityMetrics::default();
        assert!(!metrics.has_video());
        assert_eq!(metrics.video_resolution(), "None");

        metrics.video_width = 1920;
        metrics.video_height = 1080;
        assert!(metrics.has_video());
        assert_eq!(metrics.video_resolution(), "1920x1080");
    }

    #[test]
    fn quality_metrics_total_bitrate() {
        let mut metrics = QualityMetrics::default();
        metrics.audio_bitrate_kbps = 32;
        metrics.video_bitrate_kbps = 1500;
        assert_eq!(metrics.total_bitrate_kbps(), 1532);
    }

    #[test]
    fn quality_metrics_recalculate() {
        let mut metrics = QualityMetrics::default();
        metrics.latency_ms = 200;
        metrics.packet_loss_percent = 3.0;
        metrics.recalculate_quality();
        assert_eq!(metrics.quality, ConnectionQuality::Fair);
    }

    #[test]
    fn call_snapshot_quality_helpers() {
        let mut snapshot = CallSnapshot::default();
        assert_eq!(snapshot.connection_quality(), ConnectionQuality::Unknown);
        assert!(!snapshot.has_quality_issues());

        snapshot.quality_metrics.quality = ConnectionQuality::Poor;
        assert!(snapshot.has_quality_issues());

        snapshot.quality_metrics.quality = ConnectionQuality::Critical;
        assert!(snapshot.has_quality_issues());

        snapshot.quality_metrics.quality = ConnectionQuality::Good;
        assert!(!snapshot.has_quality_issues());
    }

    #[test]
    fn call_snapshot_participant_quality() {
        let mut snapshot = CallSnapshot::default();
        snapshot.participant_quality.push(ParticipantQuality {
            participant_id: "alice".to_string(),
            incoming: QualityMetrics {
                latency_ms: 50,
                packet_loss_percent: 0.5,
                quality: ConnectionQuality::Excellent,
                ..Default::default()
            },
            outgoing: None,
        });

        assert!(snapshot.get_participant_quality("alice").is_some());
        assert!(snapshot.get_participant_quality("bob").is_none());

        let quality = snapshot.get_participant_quality("alice").unwrap();
        assert_eq!(quality.incoming.quality, ConnectionQuality::Excellent);
    }

    #[test]
    fn recording_state_is_active() {
        assert!(!RecordingState::NotRecording.is_active());
        assert!(RecordingState::Recording.is_active());
        assert!(RecordingState::Paused.is_active());
        assert!(RecordingState::Finalizing.is_active());
    }

    #[test]
    fn recording_state_labels() {
        assert_eq!(RecordingState::NotRecording.label(), "Not Recording");
        assert_eq!(RecordingState::Recording.label(), "Recording");
        assert_eq!(RecordingState::Paused.label(), "Paused");
        assert_eq!(RecordingState::Finalizing.label(), "Saving...");
    }

    #[test]
    fn recording_state_status_class() {
        assert_eq!(
            RecordingState::NotRecording.status_class(),
            "text-slate-400"
        );
        assert_eq!(
            RecordingState::Recording.status_class(),
            "text-red-500 animate-pulse"
        );
        assert_eq!(RecordingState::Paused.status_class(), "text-amber-500");
        assert_eq!(RecordingState::Finalizing.status_class(), "text-blue-500");
    }

    #[test]
    fn recording_info_formatted_duration() {
        let info = RecordingInfo {
            duration_ms: 65_000, // 1 minute 5 seconds
            ..Default::default()
        };
        assert_eq!(info.formatted_duration(), "01:05");

        let info2 = RecordingInfo {
            duration_ms: 3_661_000, // 61 minutes 1 second
            ..Default::default()
        };
        assert_eq!(info2.formatted_duration(), "61:01");
    }

    #[test]
    fn recording_info_formatted_size() {
        let bytes_info = RecordingInfo {
            file_size_bytes: 500,
            ..Default::default()
        };
        assert_eq!(bytes_info.formatted_size(), "500 B");

        let kb_info = RecordingInfo {
            file_size_bytes: 2048,
            ..Default::default()
        };
        assert_eq!(kb_info.formatted_size(), "2.0 KB");

        let mb_info = RecordingInfo {
            file_size_bytes: 5 * 1024 * 1024,
            ..Default::default()
        };
        assert_eq!(mb_info.formatted_size(), "5.0 MB");

        let gb_info = RecordingInfo {
            file_size_bytes: 2 * 1024 * 1024 * 1024,
            ..Default::default()
        };
        assert_eq!(gb_info.formatted_size(), "2.00 GB");
    }

    #[test]
    fn recording_info_default() {
        let info = RecordingInfo::default();
        assert_eq!(info.state, RecordingState::NotRecording);
        assert!(info.includes_audio);
        assert!(!info.includes_video);
        assert!(!info.includes_screen);
    }

    #[test]
    fn call_snapshot_recording_helpers() {
        let mut snapshot = CallSnapshot::default();

        // No recording
        assert_eq!(snapshot.recording_state(), RecordingState::NotRecording);
        assert!(!snapshot.is_recording_active());
        assert!(snapshot.recording_duration().is_none());

        // With active recording
        snapshot.is_recording = true;
        snapshot.recording_info = Some(RecordingInfo {
            state: RecordingState::Recording,
            duration_ms: 30_000,
            ..Default::default()
        });

        assert_eq!(snapshot.recording_state(), RecordingState::Recording);
        assert!(snapshot.is_recording_active());
        assert_eq!(snapshot.recording_duration(), Some("00:30".to_string()));

        // Paused recording
        snapshot.recording_info.as_mut().unwrap().state = RecordingState::Paused;
        assert_eq!(snapshot.recording_state(), RecordingState::Paused);
        assert!(snapshot.is_recording_active());
    }
}
