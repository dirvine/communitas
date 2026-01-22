// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Call Recording Module
//!
//! Provides recording capabilities for WebRTC calls, saving audio and video
//! streams to WebM files in the user's drive.
//!
//! ## Architecture
//!
//! The recorder captures media streams from the WebRTC connection and encodes
//! them to WebM format using the matroska container with VP8/VP9 video and
//! Opus audio codecs.
//!
//! ## File Location
//!
//! Recordings are saved to the user's private drive by default, in the
//! `recordings/` subdirectory. Each recording is named with the call ID
//! and timestamp.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during recording.
#[derive(Error, Debug)]
pub enum RecordingError {
    /// Recording is not currently active.
    #[error("not recording")]
    NotRecording,
    /// Recording is already active.
    #[error("recording already active")]
    AlreadyActive,
    /// Failed to start recording.
    #[error("failed to start recording: {0}")]
    StartFailed(String),
    /// Failed to stop recording.
    #[error("failed to stop recording: {0}")]
    StopFailed(String),
    /// Failed to write recording file.
    #[error("failed to write file: {0}")]
    WriteFailed(String),
    /// Invalid recording configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    /// Storage error.
    #[error("storage error: {0}")]
    StorageError(String),
}

/// Configuration for call recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingConfig {
    /// Whether to include audio in the recording.
    pub include_audio: bool,
    /// Whether to include video in the recording.
    pub include_video: bool,
    /// Whether to include screen share in the recording.
    pub include_screen_share: bool,
    /// Video quality preset.
    pub video_quality: VideoQuality,
    /// Audio quality preset.
    pub audio_quality: AudioQuality,
    /// Maximum recording duration in seconds (0 = unlimited).
    pub max_duration_secs: u64,
    /// Custom output directory (None uses default drive location).
    pub output_directory: Option<PathBuf>,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            include_audio: true,
            include_video: true,
            include_screen_share: true,
            video_quality: VideoQuality::Medium,
            audio_quality: AudioQuality::High,
            max_duration_secs: 0, // unlimited
            output_directory: None,
        }
    }
}

/// Video quality preset for recordings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VideoQuality {
    /// 480p at 15fps, low bitrate (~500kbps).
    Low,
    /// 720p at 24fps, medium bitrate (~1500kbps).
    #[default]
    Medium,
    /// 1080p at 30fps, high bitrate (~3000kbps).
    High,
}

impl VideoQuality {
    /// Returns the target resolution width.
    pub fn width(&self) -> u32 {
        match self {
            Self::Low => 854,
            Self::Medium => 1280,
            Self::High => 1920,
        }
    }

    /// Returns the target resolution height.
    pub fn height(&self) -> u32 {
        match self {
            Self::Low => 480,
            Self::Medium => 720,
            Self::High => 1080,
        }
    }

    /// Returns the target framerate.
    pub fn fps(&self) -> u32 {
        match self {
            Self::Low => 15,
            Self::Medium => 24,
            Self::High => 30,
        }
    }

    /// Returns the target bitrate in kbps.
    pub fn bitrate_kbps(&self) -> u32 {
        match self {
            Self::Low => 500,
            Self::Medium => 1500,
            Self::High => 3000,
        }
    }
}

/// Audio quality preset for recordings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AudioQuality {
    /// Low quality (32kbps Opus).
    Low,
    /// Standard quality (64kbps Opus).
    Standard,
    /// High quality (128kbps Opus).
    #[default]
    High,
}

impl AudioQuality {
    /// Returns the target bitrate in kbps.
    pub fn bitrate_kbps(&self) -> u32 {
        match self {
            Self::Low => 32,
            Self::Standard => 64,
            Self::High => 128,
        }
    }
}

/// State of a recording session.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordingSession {
    /// Unique session identifier.
    pub id: String,
    /// Call ID being recorded.
    pub call_id: String,
    /// Recording configuration.
    pub config: RecordingConfig,
    /// When recording started (Unix ms).
    pub started_at: u64,
    /// Current duration recorded (ms), excluding paused time.
    pub duration_ms: u64,
    /// Whether recording is paused.
    pub is_paused: bool,
    /// Estimated file size in bytes.
    pub file_size_bytes: u64,
    /// Output file path (once determined).
    pub output_path: Option<PathBuf>,
}

impl RecordingSession {
    /// Create a new recording session.
    pub fn new(call_id: String, config: RecordingConfig, started_at: u64) -> Self {
        let id = format!("rec-{}-{}", call_id, started_at);
        Self {
            id,
            call_id,
            config,
            started_at,
            duration_ms: 0,
            is_paused: false,
            file_size_bytes: 0,
            output_path: None,
        }
    }

    /// Update the session's duration and file size.
    pub fn update_stats(&mut self, duration_ms: u64, file_size_bytes: u64) {
        self.duration_ms = duration_ms;
        self.file_size_bytes = file_size_bytes;
    }

    /// Set the output file path.
    pub fn set_output_path(&mut self, path: PathBuf) {
        self.output_path = Some(path);
    }
}

/// Result of a completed recording.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordingResult {
    /// Recording session ID.
    pub session_id: String,
    /// Call ID that was recorded.
    pub call_id: String,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// File size in bytes.
    pub file_size_bytes: u64,
    /// Path to the saved recording file.
    pub file_path: PathBuf,
    /// When recording started.
    pub started_at: u64,
    /// When recording ended.
    pub ended_at: u64,
    /// Whether audio was included.
    pub includes_audio: bool,
    /// Whether video was included.
    pub includes_video: bool,
    /// Whether screen share was included.
    pub includes_screen_share: bool,
}

/// Generate a recording filename from call info.
pub fn generate_recording_filename(call_id: &str, timestamp: u64, has_video: bool) -> String {
    let extension = if has_video { "webm" } else { "opus" };
    format!("call-{}-{}.{}", call_id, timestamp, extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_config_default() {
        let config = RecordingConfig::default();
        assert!(config.include_audio);
        assert!(config.include_video);
        assert!(config.include_screen_share);
        assert_eq!(config.video_quality, VideoQuality::Medium);
        assert_eq!(config.audio_quality, AudioQuality::High);
        assert_eq!(config.max_duration_secs, 0);
        assert!(config.output_directory.is_none());
    }

    #[test]
    fn video_quality_values() {
        assert_eq!(VideoQuality::Low.width(), 854);
        assert_eq!(VideoQuality::Low.height(), 480);
        assert_eq!(VideoQuality::Low.fps(), 15);
        assert_eq!(VideoQuality::Low.bitrate_kbps(), 500);

        assert_eq!(VideoQuality::Medium.width(), 1280);
        assert_eq!(VideoQuality::Medium.height(), 720);
        assert_eq!(VideoQuality::Medium.fps(), 24);
        assert_eq!(VideoQuality::Medium.bitrate_kbps(), 1500);

        assert_eq!(VideoQuality::High.width(), 1920);
        assert_eq!(VideoQuality::High.height(), 1080);
        assert_eq!(VideoQuality::High.fps(), 30);
        assert_eq!(VideoQuality::High.bitrate_kbps(), 3000);
    }

    #[test]
    fn audio_quality_bitrates() {
        assert_eq!(AudioQuality::Low.bitrate_kbps(), 32);
        assert_eq!(AudioQuality::Standard.bitrate_kbps(), 64);
        assert_eq!(AudioQuality::High.bitrate_kbps(), 128);
    }

    #[test]
    fn recording_session_creation() {
        let config = RecordingConfig::default();
        let session = RecordingSession::new("call-123".to_string(), config, 1234567890);

        assert!(session.id.starts_with("rec-call-123-"));
        assert_eq!(session.call_id, "call-123");
        assert_eq!(session.started_at, 1234567890);
        assert_eq!(session.duration_ms, 0);
        assert!(!session.is_paused);
        assert!(session.output_path.is_none());
    }

    #[test]
    fn recording_session_update() {
        let config = RecordingConfig::default();
        let mut session = RecordingSession::new("call-456".to_string(), config, 1234567890);

        session.update_stats(30000, 1_500_000);
        assert_eq!(session.duration_ms, 30000);
        assert_eq!(session.file_size_bytes, 1_500_000);

        session.set_output_path(PathBuf::from("/recordings/test.webm"));
        assert_eq!(
            session.output_path,
            Some(PathBuf::from("/recordings/test.webm"))
        );
    }

    #[test]
    fn generate_filename_with_video() {
        let filename = generate_recording_filename("abc123", 1234567890, true);
        assert_eq!(filename, "call-abc123-1234567890.webm");
    }

    #[test]
    fn generate_filename_audio_only() {
        let filename = generate_recording_filename("xyz789", 9876543210, false);
        assert_eq!(filename, "call-xyz789-9876543210.opus");
    }
}
