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
    /// Current audio level (0.0 to 1.0).
    pub audio_level: f32,
    /// Timestamp when the participant joined (ms since epoch).
    pub joined_at: i64,
}

impl Participant {
    /// Returns true if this participant has active media.
    pub fn has_active_media(&self) -> bool {
        !self.is_muted || self.is_video_enabled
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
            ..participant
        };
        assert!(participant_with_video.has_active_media());
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
}
