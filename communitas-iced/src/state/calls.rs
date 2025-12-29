// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! WebRTC call state.

use std::time::Duration;

/// Call status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallStatus {
    /// Initiating the call.
    #[default]
    Initiating,
    /// Ringing (waiting for answer).
    Ringing,
    /// Connecting.
    Connecting,
    /// Call is active.
    Connected,
    /// Call is on hold.
    OnHold,
    /// Call is ending.
    Ending,
}

impl CallStatus {
    /// Get display text for this status.
    #[must_use]
    pub fn display(&self) -> &'static str {
        match self {
            Self::Initiating => "Initiating...",
            Self::Ringing => "Ringing...",
            Self::Connecting => "Connecting...",
            Self::Connected => "Connected",
            Self::OnHold => "On Hold",
            Self::Ending => "Ending...",
        }
    }
}

/// Information about an active call.
#[derive(Debug, Clone)]
pub struct CallInfo {
    /// Call ID.
    pub call_id: String,
    /// Remote peer's four-word identity.
    pub peer_four_words: String,
    /// Remote peer's display name.
    pub peer_display_name: Option<String>,
    /// Whether this is an outgoing call.
    pub is_outgoing: bool,
    /// Current call status.
    pub status: CallStatus,
    /// Whether local video is enabled.
    pub is_video_enabled: bool,
    /// Whether local audio is enabled.
    pub is_audio_enabled: bool,
    /// Whether screen sharing is active.
    pub is_screen_sharing: bool,
    /// Call start time (for duration calculation).
    pub start_time: Option<std::time::Instant>,
}

impl CallInfo {
    /// Create a new outgoing call.
    #[must_use]
    pub fn new_outgoing(call_id: String, peer_four_words: String, has_video: bool) -> Self {
        Self {
            call_id,
            peer_four_words,
            peer_display_name: None,
            is_outgoing: true,
            status: CallStatus::Initiating,
            is_video_enabled: has_video,
            is_audio_enabled: true,
            is_screen_sharing: false,
            start_time: None,
        }
    }

    /// Create a new incoming call.
    #[must_use]
    pub fn new_incoming(call_id: String, peer_four_words: String, has_video: bool) -> Self {
        Self {
            call_id,
            peer_four_words,
            peer_display_name: None,
            is_outgoing: false,
            status: CallStatus::Ringing,
            is_video_enabled: has_video,
            is_audio_enabled: true,
            is_screen_sharing: false,
            start_time: None,
        }
    }

    /// Get the call duration.
    #[must_use]
    pub fn duration(&self) -> Duration {
        self.start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Format the call duration.
    #[must_use]
    pub fn formatted_duration(&self) -> String {
        let secs = self.duration().as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        if mins >= 60 {
            let hours = mins / 60;
            let mins = mins % 60;
            format!("{hours}:{mins:02}:{secs:02}")
        } else {
            format!("{mins}:{secs:02}")
        }
    }
}

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

/// A media device.
#[derive(Debug, Clone)]
pub struct MediaDevice {
    /// Device ID.
    pub id: String,
    /// Device name.
    pub name: String,
    /// Whether this is the default device.
    pub is_default: bool,
}

/// Available media devices.
#[derive(Debug, Clone, Default)]
pub struct MediaDevices {
    /// Available audio input devices.
    pub audio_inputs: Vec<MediaDevice>,
    /// Available audio output devices.
    pub audio_outputs: Vec<MediaDevice>,
    /// Available video devices.
    pub video_devices: Vec<MediaDevice>,
    /// Selected audio input device ID.
    pub selected_audio_input: Option<String>,
    /// Selected audio output device ID.
    pub selected_audio_output: Option<String>,
    /// Selected video device ID.
    pub selected_video: Option<String>,
}

/// Call state for the application.
#[derive(Debug, Clone, Default)]
pub struct CallState {
    /// Currently active call.
    pub active_call: Option<CallInfo>,
    /// Incoming call notification.
    pub incoming_call: Option<CallInfo>,
    /// Participants in the active call.
    pub participants: Vec<CallParticipant>,
    /// Available media devices.
    pub devices: MediaDevices,
}

impl CallState {
    /// Check if there's an active call.
    #[must_use]
    pub fn has_active_call(&self) -> bool {
        self.active_call.is_some()
    }

    /// Check if there's an incoming call.
    #[must_use]
    pub fn has_incoming_call(&self) -> bool {
        self.incoming_call.is_some()
    }
}
