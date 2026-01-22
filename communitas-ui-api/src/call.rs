//! Call and WebRTC related DTOs for real-time communication.

use serde::{Deserialize, Serialize};

/// Type of call (determines participant limits and features).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CallType {
    /// Direct one-on-one call.
    #[default]
    Direct,
    /// Group call with multiple participants.
    Group,
    /// Channel-wide call (like Discord voice channel).
    Channel,
}

impl CallType {
    /// Returns a human-readable label for the call type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Direct => "Call",
            Self::Group => "Group Call",
            Self::Channel => "Voice Channel",
        }
    }

    /// Returns the default maximum participants for this call type.
    pub fn default_max_participants(&self) -> u32 {
        match self {
            Self::Direct => 2,
            Self::Group => 25,
            Self::Channel => 100,
        }
    }

    /// Returns true if this call type supports host controls.
    pub fn has_host_controls(&self) -> bool {
        !matches!(self, Self::Direct)
    }
}

/// Role of a participant in a call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ParticipantRole {
    /// Regular participant.
    #[default]
    Participant,
    /// Co-host with limited admin capabilities.
    CoHost,
    /// Full host with all controls.
    Host,
}

impl ParticipantRole {
    /// Returns a human-readable label for the role.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Participant => "Participant",
            Self::CoHost => "Co-Host",
            Self::Host => "Host",
        }
    }

    /// Returns true if this role can mute other participants.
    pub fn can_mute_others(&self) -> bool {
        matches!(self, Self::CoHost | Self::Host)
    }

    /// Returns true if this role can remove participants.
    pub fn can_remove_participants(&self) -> bool {
        matches!(self, Self::CoHost | Self::Host)
    }

    /// Returns true if this role can promote others.
    pub fn can_promote(&self) -> bool {
        matches!(self, Self::Host)
    }

    /// Returns true if this role can end the call for everyone.
    pub fn can_end_call(&self) -> bool {
        matches!(self, Self::Host)
    }

    /// Returns true if this role can start/stop recording.
    pub fn can_manage_recording(&self) -> bool {
        matches!(self, Self::CoHost | Self::Host)
    }
}

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
    /// Participant's role in the call.
    pub role: ParticipantRole,
    /// Whether the participant's microphone is muted.
    pub is_muted: bool,
    /// Whether the participant was muted by the host (not self-muted).
    pub is_muted_by_host: bool,
    /// Whether the participant's camera is enabled.
    pub is_video_enabled: bool,
    /// Whether the participant is currently speaking.
    pub is_speaking: bool,
    /// Whether the participant is currently screen sharing.
    pub is_screen_sharing: bool,
    /// Whether the participant has raised their hand.
    pub hand_raised: bool,
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

    /// Returns true if this participant is the host.
    pub fn is_host(&self) -> bool {
        matches!(self.role, ParticipantRole::Host)
    }

    /// Returns true if this participant has elevated privileges (host or co-host).
    pub fn has_elevated_role(&self) -> bool {
        matches!(self.role, ParticipantRole::Host | ParticipantRole::CoHost)
    }

    /// Returns true if this participant can unmute themselves.
    /// Participants muted by host may not be able to unmute without permission.
    pub fn can_self_unmute(&self) -> bool {
        !self.is_muted_by_host || self.has_elevated_role()
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
    /// Type of call (direct, group, channel).
    pub call_type: CallType,
    /// Current participants in the call.
    pub participants: Vec<Participant>,
    /// Timestamp when the call started (ms since epoch).
    pub started_at: i64,
    /// Duration in seconds since call started.
    pub duration_seconds: u64,
    /// Participant ID of the current user.
    pub my_participant_id: String,
    /// Participant ID of the call host.
    pub host_id: String,
    /// Maximum number of participants allowed.
    pub max_participants: u32,
    /// Whether new participants can join.
    pub is_locked: bool,
    /// Whether all participants are muted by default when joining.
    pub mute_on_entry: bool,
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

    /// Returns the host participant.
    pub fn host(&self) -> Option<&Participant> {
        self.participants.iter().find(|p| p.id == self.host_id)
    }

    /// Returns true if the current user is the host.
    pub fn am_i_host(&self) -> bool {
        self.my_participant_id == self.host_id
    }

    /// Returns true if the current user has elevated privileges.
    pub fn am_i_elevated(&self) -> bool {
        self.my_participant()
            .map(|p| p.has_elevated_role())
            .unwrap_or(false)
    }

    /// Returns the current user's role.
    pub fn my_role(&self) -> ParticipantRole {
        self.my_participant()
            .map(|p| p.role)
            .unwrap_or(ParticipantRole::Participant)
    }

    /// Returns true if this is a group call (not direct).
    pub fn is_group_call(&self) -> bool {
        !matches!(self.call_type, CallType::Direct)
    }

    /// Returns true if more participants can join.
    pub fn can_accept_more_participants(&self) -> bool {
        !self.is_locked && (self.participants.len() as u32) < self.max_participants
    }

    /// Returns participants with elevated roles (hosts and co-hosts).
    pub fn elevated_participants(&self) -> Vec<&Participant> {
        self.participants
            .iter()
            .filter(|p| p.has_elevated_role())
            .collect()
    }

    /// Returns participants with raised hands, sorted by raise time (oldest first).
    pub fn participants_with_raised_hands(&self) -> Vec<&Participant> {
        self.participants.iter().filter(|p| p.hand_raised).collect()
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

// =============================================================================
// Call History Types
// =============================================================================

/// Outcome of a call (for history entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CallOutcome {
    /// Call completed successfully.
    Completed,
    /// Outgoing call that wasn't answered.
    NoAnswer,
    /// Incoming call that was declined.
    Declined,
    /// Incoming call that was missed (not answered).
    Missed,
    /// Call failed due to technical issues.
    Failed,
    /// Call was cancelled before connecting.
    Cancelled,
    /// Call is ongoing (shouldn't appear in history, but useful for transitions).
    #[default]
    InProgress,
}

impl CallOutcome {
    /// Returns a human-readable label for the outcome.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Completed => "Completed",
            Self::NoAnswer => "No Answer",
            Self::Declined => "Declined",
            Self::Missed => "Missed",
            Self::Failed => "Failed",
            Self::Cancelled => "Cancelled",
            Self::InProgress => "In Progress",
        }
    }

    /// Returns true if this represents a missed call (incoming, not answered).
    pub fn is_missed(&self) -> bool {
        matches!(self, Self::Missed)
    }

    /// Returns true if this represents a successful connection.
    pub fn was_connected(&self) -> bool {
        matches!(self, Self::Completed | Self::InProgress)
    }

    /// Returns a CSS class hint for styling this outcome.
    pub fn status_class(&self) -> &'static str {
        match self {
            Self::Completed => "text-emerald-500",
            Self::NoAnswer => "text-amber-500",
            Self::Declined => "text-slate-500",
            Self::Missed => "text-red-500",
            Self::Failed => "text-red-600",
            Self::Cancelled => "text-slate-400",
            Self::InProgress => "text-blue-500",
        }
    }
}

/// Direction of a call (for history entries).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CallDirection {
    /// Outgoing call (we initiated it).
    #[default]
    Outgoing,
    /// Incoming call (they called us).
    Incoming,
}

impl CallDirection {
    /// Returns a human-readable label for the direction.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Outgoing => "Outgoing",
            Self::Incoming => "Incoming",
        }
    }

    /// Returns an icon hint for the direction.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Outgoing => "↗️",
            Self::Incoming => "↙️",
        }
    }
}

/// Summary of a participant in a historical call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryParticipant {
    /// Unique identifier of the participant.
    pub id: String,
    /// Display name at the time of the call.
    pub display_name: String,
    /// Four-word identifier at the time of the call.
    pub four_words: String,
    /// How long they were in the call (seconds).
    pub duration_seconds: u64,
    /// Whether they joined late or left early.
    pub partial_participation: bool,
}

/// A single call history entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallHistoryEntry {
    /// Unique ID of this call (same as CallInfo.call_id).
    pub call_id: String,
    /// Entity ID the call was associated with (contact, group, channel).
    pub entity_id: String,
    /// Entity name at the time of the call.
    pub entity_name: String,
    /// Type of call.
    pub call_type: CallType,
    /// Direction of the call.
    pub direction: CallDirection,
    /// Outcome of the call.
    pub outcome: CallOutcome,
    /// When the call started (Unix ms).
    pub started_at: i64,
    /// When the call ended (Unix ms, 0 if still in progress).
    pub ended_at: i64,
    /// Total duration in seconds (0 if not connected).
    pub duration_seconds: u64,
    /// List of participants who were in the call.
    pub participants: Vec<HistoryParticipant>,
    /// Whether video was used during the call.
    pub had_video: bool,
    /// Whether screen sharing was used during the call.
    pub had_screen_share: bool,
    /// Whether the call was recorded.
    pub was_recorded: bool,
    /// Path to recording file, if recorded and available.
    pub recording_path: Option<String>,
    /// Whether this entry has been marked as read/seen.
    pub is_read: bool,
}

impl CallHistoryEntry {
    /// Create a new history entry for an outgoing call.
    pub fn new_outgoing(
        call_id: String,
        entity_id: String,
        entity_name: String,
        call_type: CallType,
    ) -> Self {
        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Self {
            call_id,
            entity_id,
            entity_name,
            call_type,
            direction: CallDirection::Outgoing,
            outcome: CallOutcome::InProgress,
            started_at,
            ended_at: 0,
            duration_seconds: 0,
            participants: Vec::new(),
            had_video: false,
            had_screen_share: false,
            was_recorded: false,
            recording_path: None,
            is_read: true, // Outgoing calls are always "read"
        }
    }

    /// Create a new history entry for an incoming call.
    pub fn new_incoming(
        call_id: String,
        entity_id: String,
        entity_name: String,
        call_type: CallType,
    ) -> Self {
        let started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        Self {
            call_id,
            entity_id,
            entity_name,
            call_type,
            direction: CallDirection::Incoming,
            outcome: CallOutcome::InProgress,
            started_at,
            ended_at: 0,
            duration_seconds: 0,
            participants: Vec::new(),
            had_video: false,
            had_screen_share: false,
            was_recorded: false,
            recording_path: None,
            is_read: false, // Incoming calls start as unread
        }
    }

    /// Finalize the call entry with ending details.
    pub fn finalize(&mut self, outcome: CallOutcome) {
        let ended_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        self.ended_at = ended_at;
        self.outcome = outcome;

        // Calculate duration if connected
        if outcome.was_connected() && self.started_at > 0 && ended_at > self.started_at {
            self.duration_seconds = ((ended_at - self.started_at) / 1000) as u64;
        }
    }

    /// Returns the formatted duration (e.g., "1:23:45" or "0:00").
    pub fn formatted_duration(&self) -> String {
        let hours = self.duration_seconds / 3600;
        let minutes = (self.duration_seconds % 3600) / 60;
        let seconds = self.duration_seconds % 60;

        if hours > 0 {
            format!("{}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{}:{:02}", minutes, seconds)
        }
    }

    /// Returns the number of participants (excluding self).
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// Returns a display string for the participants.
    pub fn participants_display(&self) -> String {
        match self.participants.len() {
            0 => self.entity_name.clone(),
            1 => self.participants[0].display_name.clone(),
            2 => format!(
                "{} and {}",
                self.participants[0].display_name, self.participants[1].display_name
            ),
            n => format!("{} and {} others", self.participants[0].display_name, n - 1),
        }
    }

    /// Returns true if this is a missed call that hasn't been seen.
    pub fn is_unread_missed(&self) -> bool {
        self.outcome.is_missed() && !self.is_read
    }

    /// Mark this entry as read.
    pub fn mark_read(&mut self) {
        self.is_read = true;
    }

    /// Returns an icon/emoji hint for the call direction and outcome.
    pub fn icon(&self) -> &'static str {
        match (&self.direction, &self.outcome) {
            (_, CallOutcome::Missed) => "📵",
            (_, CallOutcome::Failed) => "❌",
            (CallDirection::Outgoing, CallOutcome::NoAnswer) => "📤",
            (CallDirection::Incoming, _) => "📲",
            (CallDirection::Outgoing, _) => "📱",
        }
    }
}

/// Collection of call history entries with utility methods.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CallHistory {
    /// All history entries, newest first.
    pub entries: Vec<CallHistoryEntry>,
    /// Maximum entries to keep (0 = unlimited).
    pub max_entries: usize,
}

impl CallHistory {
    /// Create a new empty call history with a max entry limit.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Add a new entry to the history.
    pub fn add(&mut self, entry: CallHistoryEntry) {
        // Insert at the beginning (newest first)
        self.entries.insert(0, entry);

        // Trim if over limit
        if self.max_entries > 0 && self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Update an existing entry (e.g., when call ends).
    pub fn update(&mut self, call_id: &str, updater: impl FnOnce(&mut CallHistoryEntry)) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.call_id == call_id) {
            updater(entry);
        }
    }

    /// Get an entry by call ID.
    pub fn get(&self, call_id: &str) -> Option<&CallHistoryEntry> {
        self.entries.iter().find(|e| e.call_id == call_id)
    }

    /// Get a mutable entry by call ID.
    pub fn get_mut(&mut self, call_id: &str) -> Option<&mut CallHistoryEntry> {
        self.entries.iter_mut().find(|e| e.call_id == call_id)
    }

    /// Remove an entry by call ID.
    pub fn remove(&mut self, call_id: &str) -> Option<CallHistoryEntry> {
        if let Some(pos) = self.entries.iter().position(|e| e.call_id == call_id) {
            Some(self.entries.remove(pos))
        } else {
            None
        }
    }

    /// Clear all history entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns all missed calls.
    pub fn missed_calls(&self) -> Vec<&CallHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.outcome.is_missed())
            .collect()
    }

    /// Returns all unread missed calls.
    pub fn unread_missed_calls(&self) -> Vec<&CallHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.is_unread_missed())
            .collect()
    }

    /// Returns the count of unread missed calls.
    pub fn unread_missed_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_unread_missed()).count()
    }

    /// Mark all missed calls as read.
    pub fn mark_all_read(&mut self) {
        for entry in &mut self.entries {
            entry.is_read = true;
        }
    }

    /// Get history filtered by entity ID.
    pub fn for_entity(&self, entity_id: &str) -> Vec<&CallHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.entity_id == entity_id)
            .collect()
    }

    /// Get history filtered by call type.
    pub fn by_type(&self, call_type: CallType) -> Vec<&CallHistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.call_type == call_type)
            .collect()
    }

    /// Get recent entries (up to limit).
    pub fn recent(&self, limit: usize) -> Vec<&CallHistoryEntry> {
        self.entries.iter().take(limit).collect()
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
            role: ParticipantRole::Participant,
            is_muted: true,
            is_muted_by_host: false,
            is_video_enabled: false,
            is_speaking: false,
            is_screen_sharing: false,
            hand_raised: false,
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

    #[test]
    fn call_type_labels_and_limits() {
        assert_eq!(CallType::Direct.label(), "Call");
        assert_eq!(CallType::Group.label(), "Group Call");
        assert_eq!(CallType::Channel.label(), "Voice Channel");

        assert_eq!(CallType::Direct.default_max_participants(), 2);
        assert_eq!(CallType::Group.default_max_participants(), 25);
        assert_eq!(CallType::Channel.default_max_participants(), 100);

        assert!(!CallType::Direct.has_host_controls());
        assert!(CallType::Group.has_host_controls());
        assert!(CallType::Channel.has_host_controls());
    }

    #[test]
    fn participant_role_permissions() {
        // Participant has no elevated permissions
        let participant = ParticipantRole::Participant;
        assert!(!participant.can_mute_others());
        assert!(!participant.can_remove_participants());
        assert!(!participant.can_promote());
        assert!(!participant.can_end_call());
        assert!(!participant.can_manage_recording());

        // CoHost can mute, remove, and manage recording
        let cohost = ParticipantRole::CoHost;
        assert!(cohost.can_mute_others());
        assert!(cohost.can_remove_participants());
        assert!(!cohost.can_promote());
        assert!(!cohost.can_end_call());
        assert!(cohost.can_manage_recording());

        // Host can do everything
        let host = ParticipantRole::Host;
        assert!(host.can_mute_others());
        assert!(host.can_remove_participants());
        assert!(host.can_promote());
        assert!(host.can_end_call());
        assert!(host.can_manage_recording());
    }

    #[test]
    fn participant_role_labels() {
        assert_eq!(ParticipantRole::Participant.label(), "Participant");
        assert_eq!(ParticipantRole::CoHost.label(), "Co-Host");
        assert_eq!(ParticipantRole::Host.label(), "Host");
    }

    #[test]
    fn participant_role_helpers() {
        let mut p = Participant {
            id: "p1".to_string(),
            display_name: "Alice".to_string(),
            four_words: "a-b-c-d".to_string(),
            role: ParticipantRole::Participant,
            is_muted: true,
            is_muted_by_host: false,
            is_video_enabled: false,
            is_speaking: false,
            is_screen_sharing: false,
            hand_raised: false,
            audio_level: 0.0,
            joined_at: 0,
        };

        assert!(!p.is_host());
        assert!(!p.has_elevated_role());
        assert!(p.can_self_unmute()); // Not muted by host

        p.is_muted_by_host = true;
        assert!(!p.can_self_unmute()); // Muted by host

        p.role = ParticipantRole::CoHost;
        assert!(!p.is_host());
        assert!(p.has_elevated_role());
        assert!(p.can_self_unmute()); // Elevated role can always unmute

        p.role = ParticipantRole::Host;
        assert!(p.is_host());
        assert!(p.has_elevated_role());
    }

    #[test]
    fn call_info_group_helpers() {
        let call = CallInfo {
            call_id: "call1".to_string(),
            entity_id: "ent1".to_string(),
            entity_name: "Team Chat".to_string(),
            call_type: CallType::Group,
            participants: vec![
                Participant {
                    id: "host".to_string(),
                    display_name: "Host".to_string(),
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
                    id: "cohost".to_string(),
                    display_name: "CoHost".to_string(),
                    four_words: "e-f-g-h".to_string(),
                    role: ParticipantRole::CoHost,
                    is_muted: false,
                    is_muted_by_host: false,
                    is_video_enabled: false,
                    is_speaking: false,
                    is_screen_sharing: false,
                    hand_raised: true,
                    audio_level: 0.0,
                    joined_at: 0,
                },
                Participant {
                    id: "participant".to_string(),
                    display_name: "Participant".to_string(),
                    four_words: "i-j-k-l".to_string(),
                    role: ParticipantRole::Participant,
                    is_muted: false,
                    is_muted_by_host: false,
                    is_video_enabled: false,
                    is_speaking: false,
                    is_screen_sharing: false,
                    hand_raised: true,
                    audio_level: 0.0,
                    joined_at: 0,
                },
            ],
            started_at: 0,
            duration_seconds: 0,
            my_participant_id: "host".to_string(),
            host_id: "host".to_string(),
            max_participants: 25,
            is_locked: false,
            mute_on_entry: false,
        };

        assert!(call.is_group_call());
        assert!(call.am_i_host());
        assert!(call.am_i_elevated());
        assert_eq!(call.my_role(), ParticipantRole::Host);

        assert!(call.host().is_some());
        assert_eq!(call.host().unwrap().id, "host");

        assert!(call.can_accept_more_participants());
        assert_eq!(call.elevated_participants().len(), 2);
        assert_eq!(call.participants_with_raised_hands().len(), 2);
    }

    #[test]
    fn call_info_locked_and_full() {
        let call = CallInfo {
            call_id: "call1".to_string(),
            entity_id: "ent1".to_string(),
            entity_name: "Full Call".to_string(),
            call_type: CallType::Direct,
            participants: vec![
                Participant {
                    id: "p1".to_string(),
                    display_name: "P1".to_string(),
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
                    id: "p2".to_string(),
                    display_name: "P2".to_string(),
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
            my_participant_id: "p1".to_string(),
            host_id: "p1".to_string(),
            max_participants: 2,
            is_locked: false,
            mute_on_entry: false,
        };

        // Call is full
        assert!(!call.can_accept_more_participants());

        // Test locked call
        let locked_call = CallInfo {
            is_locked: true,
            max_participants: 25,
            ..call
        };
        assert!(!locked_call.can_accept_more_participants());
    }

    #[test]
    fn call_outcome_labels() {
        assert_eq!(CallOutcome::Completed.label(), "Completed");
        assert_eq!(CallOutcome::NoAnswer.label(), "No Answer");
        assert_eq!(CallOutcome::Declined.label(), "Declined");
        assert_eq!(CallOutcome::Missed.label(), "Missed");
        assert_eq!(CallOutcome::Failed.label(), "Failed");
        assert_eq!(CallOutcome::Cancelled.label(), "Cancelled");
        assert_eq!(CallOutcome::InProgress.label(), "In Progress");
    }

    #[test]
    fn call_outcome_is_missed() {
        assert!(CallOutcome::Missed.is_missed());
        assert!(!CallOutcome::Completed.is_missed());
        assert!(!CallOutcome::NoAnswer.is_missed());
    }

    #[test]
    fn call_outcome_was_connected() {
        assert!(CallOutcome::Completed.was_connected());
        assert!(CallOutcome::InProgress.was_connected());
        assert!(!CallOutcome::Missed.was_connected());
        assert!(!CallOutcome::NoAnswer.was_connected());
        assert!(!CallOutcome::Failed.was_connected());
    }

    #[test]
    fn call_direction_labels() {
        assert_eq!(CallDirection::Outgoing.label(), "Outgoing");
        assert_eq!(CallDirection::Incoming.label(), "Incoming");
    }

    #[test]
    fn call_history_entry_new_outgoing() {
        let entry = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Test Entity".to_string(),
            CallType::Direct,
        );

        assert_eq!(entry.call_id, "call1");
        assert_eq!(entry.direction, CallDirection::Outgoing);
        assert_eq!(entry.outcome, CallOutcome::InProgress);
        assert!(entry.is_read); // Outgoing calls are always read
        assert!(entry.started_at > 0);
    }

    #[test]
    fn call_history_entry_new_incoming() {
        let entry = CallHistoryEntry::new_incoming(
            "call1".to_string(),
            "ent1".to_string(),
            "Test Entity".to_string(),
            CallType::Direct,
        );

        assert_eq!(entry.direction, CallDirection::Incoming);
        assert!(!entry.is_read); // Incoming calls start as unread
    }

    #[test]
    fn call_history_entry_finalize() {
        let mut entry = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Test".to_string(),
            CallType::Direct,
        );

        // Simulate some time passing
        entry.started_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
            - 5000; // 5 seconds ago

        entry.finalize(CallOutcome::Completed);

        assert_eq!(entry.outcome, CallOutcome::Completed);
        assert!(entry.ended_at > entry.started_at);
        assert!(entry.duration_seconds >= 4); // At least 4 seconds
    }

    #[test]
    fn call_history_entry_formatted_duration() {
        let mut entry = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Test".to_string(),
            CallType::Direct,
        );

        entry.duration_seconds = 0;
        assert_eq!(entry.formatted_duration(), "0:00");

        entry.duration_seconds = 65;
        assert_eq!(entry.formatted_duration(), "1:05");

        entry.duration_seconds = 3723; // 1 hour, 2 minutes, 3 seconds
        assert_eq!(entry.formatted_duration(), "1:02:03");
    }

    #[test]
    fn call_history_entry_participants_display() {
        let mut entry = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Group Chat".to_string(),
            CallType::Group,
        );

        // No participants
        assert_eq!(entry.participants_display(), "Group Chat");

        // One participant
        entry.participants.push(HistoryParticipant {
            id: "p1".to_string(),
            display_name: "Alice".to_string(),
            four_words: "a-b-c-d".to_string(),
            duration_seconds: 60,
            partial_participation: false,
        });
        assert_eq!(entry.participants_display(), "Alice");

        // Two participants
        entry.participants.push(HistoryParticipant {
            id: "p2".to_string(),
            display_name: "Bob".to_string(),
            four_words: "e-f-g-h".to_string(),
            duration_seconds: 60,
            partial_participation: false,
        });
        assert_eq!(entry.participants_display(), "Alice and Bob");

        // Three participants
        entry.participants.push(HistoryParticipant {
            id: "p3".to_string(),
            display_name: "Charlie".to_string(),
            four_words: "i-j-k-l".to_string(),
            duration_seconds: 60,
            partial_participation: false,
        });
        assert_eq!(entry.participants_display(), "Alice and 2 others");
    }

    #[test]
    fn call_history_entry_is_unread_missed() {
        let mut entry = CallHistoryEntry::new_incoming(
            "call1".to_string(),
            "ent1".to_string(),
            "Test".to_string(),
            CallType::Direct,
        );

        entry.outcome = CallOutcome::Missed;
        entry.is_read = false;
        assert!(entry.is_unread_missed());

        entry.mark_read();
        assert!(!entry.is_unread_missed());
    }

    #[test]
    fn call_history_add_and_get() {
        let mut history = CallHistory::new(100);
        assert!(history.is_empty());

        let entry1 = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Test 1".to_string(),
            CallType::Direct,
        );
        let entry2 = CallHistoryEntry::new_outgoing(
            "call2".to_string(),
            "ent2".to_string(),
            "Test 2".to_string(),
            CallType::Direct,
        );

        history.add(entry1);
        history.add(entry2);

        assert_eq!(history.len(), 2);
        assert_eq!(history.entries[0].call_id, "call2"); // Newest first
        assert_eq!(history.entries[1].call_id, "call1");

        assert!(history.get("call1").is_some());
        assert!(history.get("nonexistent").is_none());
    }

    #[test]
    fn call_history_max_entries() {
        let mut history = CallHistory::new(2);

        for i in 0..5 {
            let entry = CallHistoryEntry::new_outgoing(
                format!("call{}", i),
                "ent".to_string(),
                "Test".to_string(),
                CallType::Direct,
            );
            history.add(entry);
        }

        // Should only have 2 entries (newest)
        assert_eq!(history.len(), 2);
        assert_eq!(history.entries[0].call_id, "call4");
        assert_eq!(history.entries[1].call_id, "call3");
    }

    #[test]
    fn call_history_update() {
        let mut history = CallHistory::new(100);

        let entry = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Test".to_string(),
            CallType::Direct,
        );
        history.add(entry);

        history.update("call1", |e| {
            e.outcome = CallOutcome::Completed;
            e.duration_seconds = 120;
        });

        let updated = history.get("call1").unwrap();
        assert_eq!(updated.outcome, CallOutcome::Completed);
        assert_eq!(updated.duration_seconds, 120);
    }

    #[test]
    fn call_history_remove() {
        let mut history = CallHistory::new(100);

        let entry = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Test".to_string(),
            CallType::Direct,
        );
        history.add(entry);

        let removed = history.remove("call1");
        assert!(removed.is_some());
        assert!(history.is_empty());

        let not_found = history.remove("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn call_history_missed_calls() {
        let mut history = CallHistory::new(100);

        let mut missed = CallHistoryEntry::new_incoming(
            "call1".to_string(),
            "ent1".to_string(),
            "Missed".to_string(),
            CallType::Direct,
        );
        missed.outcome = CallOutcome::Missed;
        missed.is_read = false;

        let completed = CallHistoryEntry::new_outgoing(
            "call2".to_string(),
            "ent1".to_string(),
            "Completed".to_string(),
            CallType::Direct,
        );

        history.add(missed);
        history.add(completed);

        assert_eq!(history.missed_calls().len(), 1);
        assert_eq!(history.unread_missed_count(), 1);

        history.mark_all_read();
        assert_eq!(history.unread_missed_count(), 0);
    }

    #[test]
    fn call_history_for_entity() {
        let mut history = CallHistory::new(100);

        let entry1 = CallHistoryEntry::new_outgoing(
            "call1".to_string(),
            "ent1".to_string(),
            "Entity 1".to_string(),
            CallType::Direct,
        );
        let entry2 = CallHistoryEntry::new_outgoing(
            "call2".to_string(),
            "ent2".to_string(),
            "Entity 2".to_string(),
            CallType::Direct,
        );
        let entry3 = CallHistoryEntry::new_outgoing(
            "call3".to_string(),
            "ent1".to_string(),
            "Entity 1 Again".to_string(),
            CallType::Direct,
        );

        history.add(entry1);
        history.add(entry2);
        history.add(entry3);

        let ent1_calls = history.for_entity("ent1");
        assert_eq!(ent1_calls.len(), 2);
    }

    #[test]
    fn call_history_recent() {
        let mut history = CallHistory::new(100);

        for i in 0..10 {
            let entry = CallHistoryEntry::new_outgoing(
                format!("call{}", i),
                "ent".to_string(),
                "Test".to_string(),
                CallType::Direct,
            );
            history.add(entry);
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].call_id, "call9"); // Newest
    }
}
