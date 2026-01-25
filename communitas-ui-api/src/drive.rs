//! Drive DTOs for virtual disk, directory, file, and transfer views.
//!
//! These types are designed for UI rendering and MCP tool responses.
//! They provide a simplified view of the underlying storage system
//! with upload/download progress tracking and checksum verification.

use serde::{Deserialize, Serialize};

use crate::SyncState;

/// Type of virtual disk (storage area).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiskType {
    /// Private disk - encrypted, local-only storage.
    Private,
    /// Public disk - content-addressed, distributed storage.
    Public,
    /// Shared disk - group-accessible with shared encryption.
    Shared,
}

impl DiskType {
    /// Returns a human-readable label for the disk type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Private => "Private",
            Self::Public => "Public",
            Self::Shared => "Shared",
        }
    }
}

/// Information about a virtual disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Type of the disk.
    pub disk_type: DiskType,
    /// Entity that owns this disk.
    pub entity_id: String,
    /// Total storage capacity in bytes.
    pub total_bytes: u64,
    /// Storage used in bytes.
    pub used_bytes: u64,
    /// Available storage in bytes.
    pub available_bytes: u64,
    /// Number of files stored.
    pub file_count: u64,
}

/// Entry in a directory listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    /// File or directory name.
    pub name: String,
    /// Full path within the disk.
    pub path: String,
    /// Whether this entry is a directory.
    pub is_directory: bool,
    /// Size in bytes (0 for directories).
    pub size_bytes: u64,
    /// MIME type (None for directories).
    pub mime_type: Option<String>,
    /// Unix timestamp (ms) of last modification.
    pub modified_at: i64,
    /// Unix timestamp (ms) of creation.
    pub created_at: i64,
    /// BLAKE3 checksum (None for directories).
    pub checksum: Option<String>,
    /// Sync state of this entry.
    pub sync_state: SyncState,
}

/// Metadata for a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Unix timestamp (ms) of creation.
    pub created_at: i64,
    /// Unix timestamp (ms) of last modification.
    pub modified_at: i64,
    /// BLAKE3 checksum of the file content.
    pub checksum: String,
    /// Number of storage blocks.
    pub block_count: u32,
    /// Encryption method (None for unencrypted).
    pub encryption: Option<String>,
}

/// Preview of a file with optional thumbnail and text excerpt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilePreview {
    /// Full path within the disk.
    pub path: String,
    /// MIME type of the file.
    pub mime_type: String,
    /// Size in bytes.
    pub size_bytes: u64,
    /// Optional thumbnail data (e.g., for images).
    pub thumbnail: Option<Vec<u8>>,
    /// Optional text preview (first N characters for text files).
    pub text_preview: Option<String>,
    /// File metadata.
    pub metadata: FileMetadata,
}

/// State of an upload operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UploadState {
    /// Upload is queued but not started.
    Pending,
    /// Upload is in progress.
    Uploading,
    /// Upload complete, verifying checksum.
    Verifying,
    /// Upload completed successfully.
    Complete,
    /// Upload failed with error message.
    Failed(String),
    /// Upload was cancelled by user.
    Cancelled,
}

impl UploadState {
    /// Returns true if the upload is complete (successfully or not).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Failed(_) | Self::Cancelled)
    }
}

/// Progress of an upload operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadProgress {
    /// Unique upload identifier.
    pub id: String,
    /// Name of the file being uploaded.
    pub file_name: String,
    /// Destination path within the disk.
    pub file_path: String,
    /// Bytes uploaded so far.
    pub bytes_uploaded: u64,
    /// Total file size in bytes.
    pub total_bytes: u64,
    /// Current state of the upload.
    pub state: UploadState,
    /// Unix timestamp (ms) when upload started.
    pub started_at: i64,
    /// Whether checksum was verified after upload.
    pub checksum_verified: bool,
}

impl UploadProgress {
    /// Returns upload progress as a percentage (0-100).
    pub fn percent_complete(&self) -> u32 {
        if self.total_bytes == 0 {
            0
        } else {
            ((self.bytes_uploaded as f64 / self.total_bytes as f64) * 100.0) as u32
        }
    }
}

/// State of a download operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadState {
    /// Download is queued but not started.
    Pending,
    /// Download is in progress.
    Downloading,
    /// Download complete, verifying checksum.
    Verifying,
    /// Download completed successfully.
    Complete,
    /// Download failed with error message.
    Failed(String),
    /// Download was cancelled by user.
    Cancelled,
}

impl DownloadState {
    /// Returns true if the download is complete (successfully or not).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Complete | Self::Failed(_) | Self::Cancelled)
    }
}

/// Progress of a download operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// Unique download identifier.
    pub id: String,
    /// Name of the file being downloaded.
    pub file_name: String,
    /// Local destination path.
    pub destination_path: String,
    /// Bytes downloaded so far.
    pub bytes_downloaded: u64,
    /// Total file size in bytes.
    pub total_bytes: u64,
    /// Current state of the download.
    pub state: DownloadState,
    /// Whether checksum was verified after download.
    pub checksum_verified: bool,
}

impl DownloadProgress {
    /// Returns download progress as a percentage (0-100).
    pub fn percent_complete(&self) -> u32 {
        if self.total_bytes == 0 {
            0
        } else {
            ((self.bytes_downloaded as f64 / self.total_bytes as f64) * 100.0) as u32
        }
    }
}

/// Progress information for a single chunk during transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkProgress {
    /// Zero-based index of the current chunk.
    pub chunk_index: u32,
    /// Total number of chunks in the transfer.
    pub total_chunks: u32,
    /// Bytes transferred in this chunk.
    pub bytes_transferred: u64,
    /// Total bytes for this chunk.
    pub chunk_size: u64,
    /// BLAKE3 checksum of this chunk (hex-encoded).
    pub chunk_checksum: Option<String>,
}

impl ChunkProgress {
    /// Returns the chunk progress as a percentage (0-100).
    pub fn chunk_percent(&self) -> u32 {
        if self.chunk_size == 0 {
            100
        } else {
            ((self.bytes_transferred as f64 / self.chunk_size as f64) * 100.0) as u32
        }
    }

    /// Returns true if this is the last chunk.
    pub fn is_last(&self) -> bool {
        self.chunk_index + 1 >= self.total_chunks
    }
}

/// Capability to resume an interrupted transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResumeCapability {
    /// Transfer cannot be resumed (no state available).
    None,
    /// Transfer can be partially resumed (some chunks valid).
    Partial,
    /// Transfer can be fully resumed from last checkpoint.
    Full,
}

impl ResumeCapability {
    /// Returns true if the transfer can be resumed at all.
    pub fn can_resume(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Returns a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "Cannot resume - no saved state",
            Self::Partial => "Can resume - some chunks may need re-transfer",
            Self::Full => "Can resume - all checkpoints verified",
        }
    }
}

/// State of an in-progress or resumable transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransferState {
    /// Unique transfer identifier.
    pub id: String,
    /// Path of the file being transferred.
    pub file_path: String,
    /// Total file size in bytes.
    pub total_bytes: u64,
    /// Bytes successfully transferred.
    pub bytes_transferred: u64,
    /// Number of completed chunks.
    pub chunks_completed: u32,
    /// Total number of chunks.
    pub total_chunks: u32,
    /// BLAKE3 checksum of bytes transferred so far (hex-encoded).
    pub checksum_so_far: String,
    /// Unix timestamp (ms) when transfer started.
    pub started_at: i64,
    /// Unix timestamp (ms) of last activity.
    pub last_updated: i64,
    /// Resume capability for this transfer.
    pub resume_capability: ResumeCapability,
    /// Whether this is an upload (true) or download (false).
    pub is_upload: bool,
}

impl TransferState {
    /// Returns transfer progress as a percentage (0-100).
    pub fn percent_complete(&self) -> u32 {
        if self.total_bytes == 0 {
            0
        } else {
            ((self.bytes_transferred as f64 / self.total_bytes as f64) * 100.0) as u32
        }
    }

    /// Returns the number of remaining chunks.
    pub fn chunks_remaining(&self) -> u32 {
        self.total_chunks.saturating_sub(self.chunks_completed)
    }

    /// Returns bytes remaining to transfer.
    pub fn bytes_remaining(&self) -> u64 {
        self.total_bytes.saturating_sub(self.bytes_transferred)
    }

    /// Returns true if transfer is complete.
    pub fn is_complete(&self) -> bool {
        self.chunks_completed >= self.total_chunks
    }
}

/// Error that may occur when resuming a transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferError {
    /// Transfer state not found for the given ID.
    StateNotFound(String),
    /// File was modified since transfer started.
    FileModified {
        /// Expected checksum from transfer state.
        expected_checksum: String,
        /// Actual checksum of file content.
        actual_checksum: String,
    },
    /// Source file no longer exists.
    SourceNotFound(String),
    /// Destination path is no longer valid.
    DestinationInvalid(String),
    /// Transfer state is corrupted.
    StateCorrupted(String),
    /// Network error during transfer.
    NetworkError(String),
    /// Storage quota exceeded.
    QuotaExceeded {
        /// Required bytes for transfer.
        required_bytes: u64,
        /// Available bytes.
        available_bytes: u64,
    },
    /// Checksum verification failed for a chunk.
    ChunkVerificationFailed {
        /// Index of the failed chunk.
        chunk_index: u32,
        /// Expected checksum.
        expected: String,
        /// Actual checksum.
        actual: String,
    },
    /// Transfer was cancelled.
    Cancelled,
    /// Generic I/O error.
    IoError(String),
}

impl TransferError {
    /// Returns true if the error is recoverable by retrying.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::NetworkError(_) | Self::ChunkVerificationFailed { .. }
        )
    }

    /// Returns a human-readable error message.
    pub fn message(&self) -> String {
        match self {
            Self::StateNotFound(id) => format!("Transfer state not found: {id}"),
            Self::FileModified { .. } => "File was modified during transfer".to_string(),
            Self::SourceNotFound(path) => format!("Source file not found: {path}"),
            Self::DestinationInvalid(path) => format!("Invalid destination: {path}"),
            Self::StateCorrupted(reason) => format!("Transfer state corrupted: {reason}"),
            Self::NetworkError(msg) => format!("Network error: {msg}"),
            Self::QuotaExceeded {
                required_bytes,
                available_bytes,
            } => {
                format!(
                    "Storage quota exceeded: need {} bytes, have {} available",
                    required_bytes, available_bytes
                )
            }
            Self::ChunkVerificationFailed { chunk_index, .. } => {
                format!("Chunk {chunk_index} verification failed")
            }
            Self::Cancelled => "Transfer was cancelled".to_string(),
            Self::IoError(msg) => format!("I/O error: {msg}"),
        }
    }
}

/// Quota information for a disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuotaInfo {
    /// Type of disk.
    pub disk_type: DiskType,
    /// Storage used in bytes.
    pub used_bytes: u64,
    /// Storage quota (limit) in bytes.
    pub quota_bytes: u64,
    /// Percentage of quota used (0.0 - 100.0).
    pub percent_used: f32,
}

impl QuotaInfo {
    /// Returns the remaining bytes available.
    pub fn remaining_bytes(&self) -> u64 {
        self.quota_bytes.saturating_sub(self.used_bytes)
    }

    /// Returns true if quota is exceeded.
    pub fn is_exceeded(&self) -> bool {
        self.used_bytes >= self.quota_bytes
    }

    /// Returns true if quota usage is above the warning threshold (90%).
    pub fn is_warning(&self) -> bool {
        self.percent_used >= 90.0
    }
}

/// Share link for a file with optional expiry and password protection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLink {
    /// Unique share link ID.
    pub id: String,
    /// Entity ID that owns the file.
    pub entity_id: String,
    /// Disk type containing the file.
    pub disk_type: DiskType,
    /// Path to the shared file.
    pub file_path: String,
    /// File name for display.
    pub file_name: String,
    /// Shareable URL.
    pub url: String,
    /// Unix timestamp (ms) when link was created.
    pub created_at: i64,
    /// Unix timestamp (ms) when link expires (None = never).
    pub expires_at: Option<i64>,
    /// Whether the link is password protected.
    pub password_protected: bool,
    /// Number of times the link has been accessed.
    pub access_count: u64,
    /// Maximum number of accesses allowed (None = unlimited).
    pub max_accesses: Option<u64>,
    /// Whether the link is currently active.
    pub active: bool,
}

impl ShareLink {
    /// Returns true if the link has expired.
    pub fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at.is_some_and(|exp| now_ms >= exp)
    }

    /// Returns true if the link has reached max accesses.
    pub fn is_access_limit_reached(&self) -> bool {
        self.max_accesses
            .is_some_and(|max| self.access_count >= max)
    }

    /// Returns true if the link is currently usable.
    pub fn is_usable(&self, now_ms: i64) -> bool {
        self.active && !self.is_expired(now_ms) && !self.is_access_limit_reached()
    }

    /// Returns remaining accesses (None if unlimited).
    pub fn remaining_accesses(&self) -> Option<u64> {
        self.max_accesses
            .map(|max| max.saturating_sub(self.access_count))
    }
}

/// Configuration for creating a share link.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLinkConfig {
    /// Optional expiry duration in milliseconds from creation.
    pub expires_in_ms: Option<i64>,
    /// Optional password for the link.
    pub password: Option<String>,
    /// Maximum number of accesses (None = unlimited).
    pub max_accesses: Option<u64>,
}

impl ShareLinkConfig {
    /// Create a config that expires in a given number of hours.
    pub fn expires_in_hours(hours: u32) -> Self {
        Self {
            expires_in_ms: Some(hours as i64 * 60 * 60 * 1000),
            password: None,
            max_accesses: None,
        }
    }

    /// Create a config that expires in a given number of days.
    pub fn expires_in_days(days: u32) -> Self {
        Self {
            expires_in_ms: Some(days as i64 * 24 * 60 * 60 * 1000),
            password: None,
            max_accesses: None,
        }
    }

    /// Add password protection.
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Limit number of accesses.
    pub fn with_max_accesses(mut self, max: u64) -> Self {
        self.max_accesses = Some(max);
        self
    }
}

/// Result of a share link access attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareLinkAccessResult {
    /// Access granted, contains file metadata for download.
    Granted {
        /// File path on the disk.
        file_path: String,
        /// File name.
        file_name: String,
        /// File size in bytes.
        size_bytes: u64,
        /// MIME type.
        mime_type: Option<String>,
        /// BLAKE3 checksum.
        checksum: String,
    },
    /// Password required to access the link.
    PasswordRequired,
    /// Incorrect password provided.
    IncorrectPassword,
    /// Link has expired.
    Expired,
    /// Access limit reached.
    AccessLimitReached,
    /// Link has been revoked.
    Revoked,
    /// Link not found.
    NotFound,
}

impl ShareLinkAccessResult {
    /// Returns true if access was granted.
    pub fn is_granted(&self) -> bool {
        matches!(self, Self::Granted { .. })
    }

    /// Returns a user-friendly error message (None if granted).
    pub fn error_message(&self) -> Option<&'static str> {
        match self {
            Self::Granted { .. } => None,
            Self::PasswordRequired => Some("This link requires a password"),
            Self::IncorrectPassword => Some("Incorrect password"),
            Self::Expired => Some("This link has expired"),
            Self::AccessLimitReached => Some("This link has reached its access limit"),
            Self::Revoked => Some("This link has been revoked"),
            Self::NotFound => Some("Link not found"),
        }
    }
}

/// Usage statistics for a share link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLinkStats {
    /// Total number of accesses.
    pub total_accesses: u64,
    /// Number of successful downloads.
    pub successful_downloads: u64,
    /// Number of failed password attempts.
    pub failed_password_attempts: u64,
    /// Unix timestamp (ms) of last access.
    pub last_accessed_at: Option<i64>,
    /// Unique IP addresses that accessed (hashed for privacy).
    pub unique_accessors: u64,
}

// === Offline Staging Types ===

/// State of a staged upload in the offline queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StagedUploadState {
    /// Queued and waiting for network connectivity.
    Pending,
    /// Upload in progress (network restored).
    Uploading,
    /// Upload paused due to conflict.
    Conflicted,
    /// Upload completed successfully.
    Completed,
    /// Upload failed after retries exhausted.
    Failed,
}

impl StagedUploadState {
    /// Returns true if the staged upload is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }

    /// Returns true if the staged upload requires user action.
    pub fn requires_action(&self) -> bool {
        matches!(self, Self::Conflicted | Self::Failed)
    }

    /// Returns a human-readable label for the state.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Uploading => "Uploading",
            Self::Conflicted => "Conflict",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

/// A file staged for upload while offline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagedUpload {
    /// Unique identifier for this staged upload.
    pub id: String,
    /// Entity ID that owns the destination disk.
    pub entity_id: String,
    /// Destination disk type.
    pub disk_type: DiskType,
    /// Destination path within the disk.
    pub destination_path: String,
    /// Local file path (source).
    pub local_path: String,
    /// File name for display.
    pub file_name: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// MIME type of the file.
    pub mime_type: Option<String>,
    /// BLAKE3 checksum of the local file (at staging time).
    pub local_checksum: String,
    /// Current state of the staged upload.
    pub state: StagedUploadState,
    /// Number of upload retry attempts.
    pub retry_count: u32,
    /// Maximum retry attempts before marking as failed.
    pub max_retries: u32,
    /// Error message if state is Failed.
    pub error: Option<String>,
    /// Unix timestamp (ms) when the file was staged.
    pub staged_at: i64,
    /// Unix timestamp (ms) of last state update.
    pub updated_at: i64,
    /// Associated conflict (if state is Conflicted).
    pub conflict: Option<StagingConflict>,
}

impl StagedUpload {
    /// Returns true if the upload can be retried.
    pub fn can_retry(&self) -> bool {
        matches!(self.state, StagedUploadState::Failed) && self.retry_count < self.max_retries
    }

    /// Returns remaining retry attempts.
    pub fn retries_remaining(&self) -> u32 {
        self.max_retries.saturating_sub(self.retry_count)
    }

    /// Returns the age of this staged upload in milliseconds.
    /// Returns 0 if `now_ms` is before `staged_at`.
    pub fn age_ms(&self, now_ms: i64) -> i64 {
        (now_ms - self.staged_at).max(0)
    }
}

/// Type of conflict detected during staging sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictType {
    /// File already exists at destination with different content.
    FileExists,
    /// Local file was modified after staging.
    LocalModified,
    /// Destination file was modified since staging.
    RemoteModified,
    /// Both local and remote were modified.
    BothModified,
    /// Destination path is now occupied by a directory.
    PathTypeChanged,
    /// Insufficient quota for the upload.
    QuotaExceeded,
}

impl ConflictType {
    /// Returns a human-readable description of the conflict.
    pub fn description(&self) -> &'static str {
        match self {
            Self::FileExists => "A file with this name already exists",
            Self::LocalModified => "The local file was modified after staging",
            Self::RemoteModified => "The destination file was modified",
            Self::BothModified => "Both local and remote files were modified",
            Self::PathTypeChanged => "The destination path is now a directory",
            Self::QuotaExceeded => "Insufficient storage quota",
        }
    }
}

/// Conflict detected during staging sync.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingConflict {
    /// Type of conflict.
    pub conflict_type: ConflictType,
    /// Staged file checksum (at staging time).
    pub staged_checksum: String,
    /// Current local file checksum (if different).
    pub local_checksum: Option<String>,
    /// Remote file checksum (if exists).
    pub remote_checksum: Option<String>,
    /// Remote file size (if exists).
    pub remote_size_bytes: Option<u64>,
    /// Unix timestamp (ms) when conflict was detected.
    pub detected_at: i64,
}

impl StagingConflict {
    /// Returns true if the conflict can be auto-resolved.
    pub fn can_auto_resolve(&self) -> bool {
        // Only quota conflicts require external action
        !matches!(self.conflict_type, ConflictType::QuotaExceeded)
    }
}

/// Resolution action for a staging conflict.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Keep the local version (overwrite remote).
    KeepLocal,
    /// Keep the remote version (discard staged).
    KeepRemote,
    /// Keep both (rename staged file).
    KeepBoth,
    /// Skip this file (remove from staging).
    Skip,
    /// Retry the upload (for transient errors).
    Retry,
}

impl ConflictResolution {
    /// Returns a human-readable label for the resolution.
    pub fn label(&self) -> &'static str {
        match self {
            Self::KeepLocal => "Upload my version",
            Self::KeepRemote => "Keep existing",
            Self::KeepBoth => "Keep both",
            Self::Skip => "Skip",
            Self::Retry => "Retry",
        }
    }

    /// Returns a description of what this resolution does.
    pub fn description(&self) -> &'static str {
        match self {
            Self::KeepLocal => "Replace the remote file with your local version",
            Self::KeepRemote => "Discard your local changes and keep the remote version",
            Self::KeepBoth => "Upload with a new name to keep both versions",
            Self::Skip => "Remove this file from the upload queue",
            Self::Retry => "Try uploading again",
        }
    }
}

/// Status summary of the offline staging queue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StagingQueueStatus {
    /// Total number of files in the staging queue.
    pub total_files: u32,
    /// Number of files pending upload.
    pub pending_files: u32,
    /// Number of files currently uploading.
    pub uploading_files: u32,
    /// Number of files with conflicts.
    pub conflicted_files: u32,
    /// Number of files that failed.
    pub failed_files: u32,
    /// Number of files completed.
    pub completed_files: u32,
    /// Total bytes pending upload.
    pub total_bytes: u64,
    /// Bytes uploaded so far.
    pub bytes_uploaded: u64,
    /// Whether the queue is currently syncing.
    pub is_syncing: bool,
    /// Whether network is available.
    pub network_available: bool,
    /// Unix timestamp (ms) of last sync attempt.
    pub last_sync_at: Option<i64>,
    /// Error from last sync attempt (if failed).
    pub last_sync_error: Option<String>,
}

impl StagingQueueStatus {
    /// Returns true if there are files requiring user action.
    pub fn has_action_required(&self) -> bool {
        self.conflicted_files > 0 || self.failed_files > 0
    }

    /// Returns true if the queue is empty (all terminal states).
    pub fn is_empty(&self) -> bool {
        self.pending_files == 0 && self.uploading_files == 0 && self.conflicted_files == 0
    }

    /// Returns true if all uploads completed successfully.
    pub fn all_completed(&self) -> bool {
        self.completed_files == self.total_files && self.total_files > 0
    }

    /// Returns upload progress as a percentage (0-100).
    pub fn percent_complete(&self) -> u32 {
        if self.total_bytes == 0 {
            if self.total_files == 0 { 100 } else { 0 }
        } else {
            ((self.bytes_uploaded as f64 / self.total_bytes as f64) * 100.0) as u32
        }
    }

    /// Returns the number of actionable items (pending + uploading).
    pub fn active_count(&self) -> u32 {
        self.pending_files + self.uploading_files
    }
}

/// Event emitted when staging queue state changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StagingEvent {
    /// A file was added to the staging queue.
    FileStaged {
        /// ID of the staged upload.
        upload_id: String,
        /// File name.
        file_name: String,
    },
    /// Upload started for a staged file.
    UploadStarted {
        /// ID of the staged upload.
        upload_id: String,
    },
    /// Upload progress update.
    UploadProgress {
        /// ID of the staged upload.
        upload_id: String,
        /// Bytes uploaded so far.
        bytes_uploaded: u64,
        /// Total bytes.
        total_bytes: u64,
    },
    /// Upload completed successfully.
    UploadCompleted {
        /// ID of the staged upload.
        upload_id: String,
        /// Final destination path.
        destination_path: String,
    },
    /// Conflict detected during upload.
    ConflictDetected {
        /// ID of the staged upload.
        upload_id: String,
        /// Type of conflict.
        conflict_type: ConflictType,
    },
    /// Upload failed.
    UploadFailed {
        /// ID of the staged upload.
        upload_id: String,
        /// Error message.
        error: String,
    },
    /// Staging queue cleared.
    QueueCleared {
        /// Number of files removed.
        files_removed: u32,
    },
    /// Network connectivity changed.
    NetworkStatusChanged {
        /// Whether network is now available.
        available: bool,
    },
    /// Sync started.
    SyncStarted,
    /// Sync completed.
    SyncCompleted {
        /// Number of files uploaded.
        files_uploaded: u32,
        /// Number of files failed.
        files_failed: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_type_label() {
        assert_eq!(DiskType::Private.label(), "Private");
        assert_eq!(DiskType::Public.label(), "Public");
        assert_eq!(DiskType::Shared.label(), "Shared");
    }

    #[test]
    fn upload_state_is_terminal() {
        assert!(!UploadState::Pending.is_terminal());
        assert!(!UploadState::Uploading.is_terminal());
        assert!(!UploadState::Verifying.is_terminal());
        assert!(UploadState::Complete.is_terminal());
        assert!(UploadState::Failed("error".to_string()).is_terminal());
        assert!(UploadState::Cancelled.is_terminal());
    }

    #[test]
    fn download_state_is_terminal() {
        assert!(!DownloadState::Pending.is_terminal());
        assert!(!DownloadState::Downloading.is_terminal());
        assert!(!DownloadState::Verifying.is_terminal());
        assert!(DownloadState::Complete.is_terminal());
        assert!(DownloadState::Failed("error".to_string()).is_terminal());
        assert!(DownloadState::Cancelled.is_terminal());
    }

    #[test]
    fn upload_progress_percent() {
        let progress = UploadProgress {
            id: "upload-1".to_string(),
            file_name: "test.txt".to_string(),
            file_path: "/test.txt".to_string(),
            bytes_uploaded: 50,
            total_bytes: 100,
            state: UploadState::Uploading,
            started_at: 0,
            checksum_verified: false,
        };
        assert_eq!(progress.percent_complete(), 50);
    }

    #[test]
    fn upload_progress_percent_zero_total() {
        let progress = UploadProgress {
            id: "upload-1".to_string(),
            file_name: "empty.txt".to_string(),
            file_path: "/empty.txt".to_string(),
            bytes_uploaded: 0,
            total_bytes: 0,
            state: UploadState::Complete,
            started_at: 0,
            checksum_verified: true,
        };
        assert_eq!(progress.percent_complete(), 0);
    }

    #[test]
    fn download_progress_percent() {
        let progress = DownloadProgress {
            id: "download-1".to_string(),
            file_name: "test.txt".to_string(),
            destination_path: "/tmp/test.txt".to_string(),
            bytes_downloaded: 75,
            total_bytes: 100,
            state: DownloadState::Downloading,
            checksum_verified: false,
        };
        assert_eq!(progress.percent_complete(), 75);
    }

    #[test]
    fn quota_remaining_bytes() {
        let quota = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 60,
            quota_bytes: 100,
            percent_used: 60.0,
        };
        assert_eq!(quota.remaining_bytes(), 40);
    }

    #[test]
    fn quota_is_exceeded() {
        let over_quota = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 110,
            quota_bytes: 100,
            percent_used: 110.0,
        };
        assert!(over_quota.is_exceeded());

        let under_quota = QuotaInfo {
            disk_type: DiskType::Private,
            used_bytes: 50,
            quota_bytes: 100,
            percent_used: 50.0,
        };
        assert!(!under_quota.is_exceeded());
    }

    #[test]
    fn quota_warning_threshold() {
        let warning = QuotaInfo {
            disk_type: DiskType::Shared,
            used_bytes: 92,
            quota_bytes: 100,
            percent_used: 92.0,
        };
        assert!(warning.is_warning());

        let safe = QuotaInfo {
            disk_type: DiskType::Shared,
            used_bytes: 85,
            quota_bytes: 100,
            percent_used: 85.0,
        };
        assert!(!safe.is_warning());
    }

    #[test]
    fn chunk_progress_percent() {
        let progress = ChunkProgress {
            chunk_index: 5,
            total_chunks: 10,
            bytes_transferred: 512 * 1024,
            chunk_size: 1024 * 1024,
            chunk_checksum: Some("abc123".to_string()),
        };
        assert_eq!(progress.chunk_percent(), 50);
        assert!(!progress.is_last());
    }

    #[test]
    fn chunk_progress_is_last() {
        let last_chunk = ChunkProgress {
            chunk_index: 9,
            total_chunks: 10,
            bytes_transferred: 1024 * 1024,
            chunk_size: 1024 * 1024,
            chunk_checksum: None,
        };
        assert!(last_chunk.is_last());
    }

    #[test]
    fn chunk_progress_zero_size() {
        let zero_chunk = ChunkProgress {
            chunk_index: 0,
            total_chunks: 1,
            bytes_transferred: 0,
            chunk_size: 0,
            chunk_checksum: None,
        };
        assert_eq!(zero_chunk.chunk_percent(), 100);
    }

    #[test]
    fn resume_capability_can_resume() {
        assert!(!ResumeCapability::None.can_resume());
        assert!(ResumeCapability::Partial.can_resume());
        assert!(ResumeCapability::Full.can_resume());
    }

    #[test]
    fn resume_capability_descriptions() {
        assert!(ResumeCapability::None.description().contains("Cannot"));
        assert!(ResumeCapability::Partial.description().contains("some"));
        assert!(ResumeCapability::Full.description().contains("verified"));
    }

    #[test]
    fn transfer_state_progress() {
        let state = TransferState {
            id: "transfer-1".to_string(),
            file_path: "/data/file.bin".to_string(),
            total_bytes: 10 * 1024 * 1024,
            bytes_transferred: 3 * 1024 * 1024,
            chunks_completed: 3,
            total_chunks: 10,
            checksum_so_far: "abc123".to_string(),
            started_at: 1000,
            last_updated: 2000,
            resume_capability: ResumeCapability::Full,
            is_upload: true,
        };
        assert_eq!(state.percent_complete(), 30);
        assert_eq!(state.chunks_remaining(), 7);
        assert_eq!(state.bytes_remaining(), 7 * 1024 * 1024);
        assert!(!state.is_complete());
    }

    #[test]
    fn transfer_state_complete() {
        let complete_state = TransferState {
            id: "transfer-2".to_string(),
            file_path: "/data/done.bin".to_string(),
            total_bytes: 5 * 1024 * 1024,
            bytes_transferred: 5 * 1024 * 1024,
            chunks_completed: 5,
            total_chunks: 5,
            checksum_so_far: "final_hash".to_string(),
            started_at: 1000,
            last_updated: 3000,
            resume_capability: ResumeCapability::None,
            is_upload: false,
        };
        assert_eq!(complete_state.percent_complete(), 100);
        assert_eq!(complete_state.chunks_remaining(), 0);
        assert!(complete_state.is_complete());
    }

    #[test]
    fn transfer_error_messages() {
        let not_found = TransferError::StateNotFound("xyz".to_string());
        assert!(not_found.message().contains("xyz"));
        assert!(!not_found.is_recoverable());

        let network = TransferError::NetworkError("timeout".to_string());
        assert!(network.message().contains("timeout"));
        assert!(network.is_recoverable());

        let chunk_fail = TransferError::ChunkVerificationFailed {
            chunk_index: 5,
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert!(chunk_fail.message().contains("5"));
        assert!(chunk_fail.is_recoverable());

        let quota = TransferError::QuotaExceeded {
            required_bytes: 1000,
            available_bytes: 500,
        };
        assert!(quota.message().contains("1000"));
        assert!(quota.message().contains("500"));
        assert!(!quota.is_recoverable());
    }

    #[test]
    fn transfer_error_file_modified() {
        let modified = TransferError::FileModified {
            expected_checksum: "abc".to_string(),
            actual_checksum: "def".to_string(),
        };
        assert!(modified.message().contains("modified"));
        assert!(!modified.is_recoverable());
    }

    #[test]
    fn transfer_error_cancelled() {
        let cancelled = TransferError::Cancelled;
        assert!(cancelled.message().contains("cancelled"));
        assert!(!cancelled.is_recoverable());
    }

    // === Share Link Tests ===

    #[test]
    fn share_link_is_expired() {
        let link = ShareLink {
            id: "link-1".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Public,
            file_path: "/public/doc.pdf".to_string(),
            file_name: "doc.pdf".to_string(),
            url: "https://example.com/s/abc123".to_string(),
            created_at: 1000,
            expires_at: Some(2000),
            password_protected: false,
            access_count: 0,
            max_accesses: None,
            active: true,
        };

        assert!(!link.is_expired(1500)); // Before expiry
        assert!(link.is_expired(2000)); // At expiry
        assert!(link.is_expired(3000)); // After expiry
    }

    #[test]
    fn share_link_no_expiry() {
        let link = ShareLink {
            id: "link-2".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Public,
            file_path: "/public/forever.txt".to_string(),
            file_name: "forever.txt".to_string(),
            url: "https://example.com/s/xyz".to_string(),
            created_at: 1000,
            expires_at: None,
            password_protected: false,
            access_count: 0,
            max_accesses: None,
            active: true,
        };

        assert!(!link.is_expired(1_000_000_000)); // Never expires
    }

    #[test]
    fn share_link_access_limit() {
        let link = ShareLink {
            id: "link-3".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Public,
            file_path: "/public/limited.pdf".to_string(),
            file_name: "limited.pdf".to_string(),
            url: "https://example.com/s/lim".to_string(),
            created_at: 1000,
            expires_at: None,
            password_protected: false,
            access_count: 5,
            max_accesses: Some(5),
            active: true,
        };

        assert!(link.is_access_limit_reached());
        assert_eq!(link.remaining_accesses(), Some(0));
    }

    #[test]
    fn share_link_has_accesses_remaining() {
        let link = ShareLink {
            id: "link-4".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Public,
            file_path: "/public/file.pdf".to_string(),
            file_name: "file.pdf".to_string(),
            url: "https://example.com/s/abc".to_string(),
            created_at: 1000,
            expires_at: None,
            password_protected: false,
            access_count: 3,
            max_accesses: Some(10),
            active: true,
        };

        assert!(!link.is_access_limit_reached());
        assert_eq!(link.remaining_accesses(), Some(7));
    }

    #[test]
    fn share_link_unlimited_accesses() {
        let link = ShareLink {
            id: "link-5".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Public,
            file_path: "/public/unlimited.txt".to_string(),
            file_name: "unlimited.txt".to_string(),
            url: "https://example.com/s/unl".to_string(),
            created_at: 1000,
            expires_at: None,
            password_protected: false,
            access_count: 1000,
            max_accesses: None,
            active: true,
        };

        assert!(!link.is_access_limit_reached());
        assert_eq!(link.remaining_accesses(), None);
    }

    #[test]
    fn share_link_is_usable() {
        let active_link = ShareLink {
            id: "link-6".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Public,
            file_path: "/public/active.pdf".to_string(),
            file_name: "active.pdf".to_string(),
            url: "https://example.com/s/act".to_string(),
            created_at: 1000,
            expires_at: Some(5000),
            password_protected: true,
            access_count: 2,
            max_accesses: Some(10),
            active: true,
        };

        assert!(active_link.is_usable(3000)); // Within expiry, under limit, active

        let inactive_link = ShareLink {
            active: false,
            ..active_link.clone()
        };
        assert!(!inactive_link.is_usable(3000)); // Inactive

        let expired_link = ShareLink {
            expires_at: Some(2000),
            ..active_link.clone()
        };
        assert!(!expired_link.is_usable(3000)); // Expired

        let maxed_link = ShareLink {
            access_count: 10,
            ..active_link
        };
        assert!(!maxed_link.is_usable(3000)); // Limit reached
    }

    #[test]
    fn share_link_config_default() {
        let config = ShareLinkConfig::default();
        assert_eq!(config.expires_in_ms, None);
        assert_eq!(config.password, None);
        assert_eq!(config.max_accesses, None);
    }

    #[test]
    fn share_link_config_expires_in_hours() {
        let config = ShareLinkConfig::expires_in_hours(24);
        assert_eq!(config.expires_in_ms, Some(24 * 60 * 60 * 1000));
        assert_eq!(config.password, None);
        assert_eq!(config.max_accesses, None);
    }

    #[test]
    fn share_link_config_expires_in_days() {
        let config = ShareLinkConfig::expires_in_days(7);
        assert_eq!(config.expires_in_ms, Some(7 * 24 * 60 * 60 * 1000));
        assert_eq!(config.password, None);
        assert_eq!(config.max_accesses, None);
    }

    #[test]
    fn share_link_config_builder_chain() {
        let config = ShareLinkConfig::expires_in_days(30)
            .with_password("secret123")
            .with_max_accesses(100);

        assert_eq!(config.expires_in_ms, Some(30 * 24 * 60 * 60 * 1000));
        assert_eq!(config.password, Some("secret123".to_string()));
        assert_eq!(config.max_accesses, Some(100));
    }

    #[test]
    fn share_link_access_result_granted() {
        let granted = ShareLinkAccessResult::Granted {
            file_path: "/public/doc.pdf".to_string(),
            file_name: "doc.pdf".to_string(),
            size_bytes: 1024 * 1024,
            mime_type: Some("application/pdf".to_string()),
            checksum: "abc123".to_string(),
        };

        assert!(granted.is_granted());
        assert_eq!(granted.error_message(), None);
    }

    #[test]
    fn share_link_access_result_errors() {
        assert!(!ShareLinkAccessResult::PasswordRequired.is_granted());
        assert!(
            ShareLinkAccessResult::PasswordRequired
                .error_message()
                .is_some()
        );

        assert!(!ShareLinkAccessResult::IncorrectPassword.is_granted());
        assert!(
            ShareLinkAccessResult::IncorrectPassword
                .error_message()
                .unwrap()
                .contains("Incorrect")
        );

        assert!(!ShareLinkAccessResult::Expired.is_granted());
        assert!(
            ShareLinkAccessResult::Expired
                .error_message()
                .unwrap()
                .contains("expired")
        );

        assert!(!ShareLinkAccessResult::AccessLimitReached.is_granted());
        assert!(
            ShareLinkAccessResult::AccessLimitReached
                .error_message()
                .unwrap()
                .contains("limit")
        );

        assert!(!ShareLinkAccessResult::Revoked.is_granted());
        assert!(
            ShareLinkAccessResult::Revoked
                .error_message()
                .unwrap()
                .contains("revoked")
        );

        assert!(!ShareLinkAccessResult::NotFound.is_granted());
        assert!(
            ShareLinkAccessResult::NotFound
                .error_message()
                .unwrap()
                .contains("not found")
        );
    }

    #[test]
    fn share_link_stats_construction() {
        let stats = ShareLinkStats {
            total_accesses: 150,
            successful_downloads: 120,
            failed_password_attempts: 5,
            last_accessed_at: Some(1234567890),
            unique_accessors: 45,
        };

        assert_eq!(stats.total_accesses, 150);
        assert_eq!(stats.successful_downloads, 120);
        assert_eq!(stats.failed_password_attempts, 5);
        assert_eq!(stats.last_accessed_at, Some(1234567890));
        assert_eq!(stats.unique_accessors, 45);
    }

    // === Staging Queue Tests ===

    #[test]
    fn staged_upload_state_is_terminal() {
        assert!(!StagedUploadState::Pending.is_terminal());
        assert!(!StagedUploadState::Uploading.is_terminal());
        assert!(!StagedUploadState::Conflicted.is_terminal());
        assert!(StagedUploadState::Completed.is_terminal());
        assert!(StagedUploadState::Failed.is_terminal());
    }

    #[test]
    fn staged_upload_state_requires_action() {
        assert!(!StagedUploadState::Pending.requires_action());
        assert!(!StagedUploadState::Uploading.requires_action());
        assert!(StagedUploadState::Conflicted.requires_action());
        assert!(!StagedUploadState::Completed.requires_action());
        assert!(StagedUploadState::Failed.requires_action());
    }

    #[test]
    fn staged_upload_state_labels() {
        assert_eq!(StagedUploadState::Pending.label(), "Pending");
        assert_eq!(StagedUploadState::Uploading.label(), "Uploading");
        assert_eq!(StagedUploadState::Conflicted.label(), "Conflict");
        assert_eq!(StagedUploadState::Completed.label(), "Completed");
        assert_eq!(StagedUploadState::Failed.label(), "Failed");
    }

    fn make_staged_upload(state: StagedUploadState, retry_count: u32) -> StagedUpload {
        StagedUpload {
            id: "staged-1".to_string(),
            entity_id: "entity-1".to_string(),
            disk_type: DiskType::Private,
            destination_path: "/docs/report.pdf".to_string(),
            local_path: "/tmp/report.pdf".to_string(),
            file_name: "report.pdf".to_string(),
            size_bytes: 1024 * 1024,
            mime_type: Some("application/pdf".to_string()),
            local_checksum: "abc123".to_string(),
            state,
            retry_count,
            max_retries: 3,
            error: None,
            staged_at: 1000,
            updated_at: 2000,
            conflict: None,
        }
    }

    #[test]
    fn staged_upload_can_retry() {
        let pending = make_staged_upload(StagedUploadState::Pending, 0);
        assert!(!pending.can_retry()); // Not failed

        let failed_can_retry = make_staged_upload(StagedUploadState::Failed, 1);
        assert!(failed_can_retry.can_retry()); // Failed but retries remaining

        let failed_maxed = make_staged_upload(StagedUploadState::Failed, 3);
        assert!(!failed_maxed.can_retry()); // Retries exhausted
    }

    #[test]
    fn staged_upload_retries_remaining() {
        let zero_retries = make_staged_upload(StagedUploadState::Pending, 0);
        assert_eq!(zero_retries.retries_remaining(), 3);

        let one_retry = make_staged_upload(StagedUploadState::Failed, 1);
        assert_eq!(one_retry.retries_remaining(), 2);

        let maxed_retries = make_staged_upload(StagedUploadState::Failed, 3);
        assert_eq!(maxed_retries.retries_remaining(), 0);

        let over_retries = make_staged_upload(StagedUploadState::Failed, 5);
        assert_eq!(over_retries.retries_remaining(), 0);
    }

    #[test]
    fn staged_upload_age() {
        let upload = make_staged_upload(StagedUploadState::Pending, 0);
        assert_eq!(upload.age_ms(5000), 4000);
        assert_eq!(upload.age_ms(1000), 0);
        assert_eq!(upload.age_ms(500), 0); // saturating_sub
    }

    #[test]
    fn conflict_type_descriptions() {
        assert!(ConflictType::FileExists.description().contains("exists"));
        assert!(ConflictType::LocalModified.description().contains("local"));
        assert!(
            ConflictType::RemoteModified
                .description()
                .contains("destination")
        );
        assert!(ConflictType::BothModified.description().contains("Both"));
        assert!(
            ConflictType::PathTypeChanged
                .description()
                .contains("directory")
        );
        assert!(ConflictType::QuotaExceeded.description().contains("quota"));
    }

    #[test]
    fn staging_conflict_can_auto_resolve() {
        let file_exists = StagingConflict {
            conflict_type: ConflictType::FileExists,
            staged_checksum: "abc".to_string(),
            local_checksum: None,
            remote_checksum: Some("def".to_string()),
            remote_size_bytes: Some(1024),
            detected_at: 1000,
        };
        assert!(file_exists.can_auto_resolve());

        let quota_exceeded = StagingConflict {
            conflict_type: ConflictType::QuotaExceeded,
            staged_checksum: "abc".to_string(),
            local_checksum: None,
            remote_checksum: None,
            remote_size_bytes: None,
            detected_at: 1000,
        };
        assert!(!quota_exceeded.can_auto_resolve());
    }

    #[test]
    fn conflict_resolution_labels() {
        assert_eq!(ConflictResolution::KeepLocal.label(), "Upload my version");
        assert_eq!(ConflictResolution::KeepRemote.label(), "Keep existing");
        assert_eq!(ConflictResolution::KeepBoth.label(), "Keep both");
        assert_eq!(ConflictResolution::Skip.label(), "Skip");
        assert_eq!(ConflictResolution::Retry.label(), "Retry");
    }

    #[test]
    fn conflict_resolution_descriptions() {
        assert!(
            ConflictResolution::KeepLocal
                .description()
                .contains("Replace")
        );
        assert!(
            ConflictResolution::KeepRemote
                .description()
                .contains("Discard")
        );
        assert!(ConflictResolution::KeepBoth.description().contains("both"));
        assert!(ConflictResolution::Skip.description().contains("Remove"));
        assert!(ConflictResolution::Retry.description().contains("again"));
    }

    #[test]
    fn staging_queue_status_has_action_required() {
        let no_action = StagingQueueStatus {
            total_files: 5,
            pending_files: 3,
            uploading_files: 2,
            conflicted_files: 0,
            failed_files: 0,
            completed_files: 0,
            total_bytes: 1000,
            bytes_uploaded: 500,
            is_syncing: true,
            network_available: true,
            last_sync_at: Some(1000),
            last_sync_error: None,
        };
        assert!(!no_action.has_action_required());

        let has_conflict = StagingQueueStatus {
            conflicted_files: 1,
            ..no_action.clone()
        };
        assert!(has_conflict.has_action_required());

        let has_failed = StagingQueueStatus {
            failed_files: 2,
            ..no_action
        };
        assert!(has_failed.has_action_required());
    }

    #[test]
    fn staging_queue_status_is_empty() {
        let empty = StagingQueueStatus {
            total_files: 5,
            pending_files: 0,
            uploading_files: 0,
            conflicted_files: 0,
            failed_files: 0,
            completed_files: 5,
            total_bytes: 1000,
            bytes_uploaded: 1000,
            is_syncing: false,
            network_available: true,
            last_sync_at: Some(1000),
            last_sync_error: None,
        };
        assert!(empty.is_empty());

        let has_pending = StagingQueueStatus {
            pending_files: 2,
            completed_files: 3,
            ..empty.clone()
        };
        assert!(!has_pending.is_empty());

        let has_uploading = StagingQueueStatus {
            uploading_files: 1,
            completed_files: 4,
            ..empty.clone()
        };
        assert!(!has_uploading.is_empty());

        let has_conflicted = StagingQueueStatus {
            conflicted_files: 1,
            completed_files: 4,
            ..empty
        };
        assert!(!has_conflicted.is_empty());
    }

    #[test]
    fn staging_queue_status_all_completed() {
        let all_done = StagingQueueStatus {
            total_files: 5,
            pending_files: 0,
            uploading_files: 0,
            conflicted_files: 0,
            failed_files: 0,
            completed_files: 5,
            total_bytes: 1000,
            bytes_uploaded: 1000,
            is_syncing: false,
            network_available: true,
            last_sync_at: Some(1000),
            last_sync_error: None,
        };
        assert!(all_done.all_completed());

        let partial = StagingQueueStatus {
            completed_files: 3,
            ..all_done.clone()
        };
        assert!(!partial.all_completed());

        let empty_queue = StagingQueueStatus {
            total_files: 0,
            completed_files: 0,
            ..all_done
        };
        assert!(!empty_queue.all_completed());
    }

    #[test]
    fn staging_queue_status_percent_complete() {
        let half_done = StagingQueueStatus {
            total_files: 4,
            pending_files: 2,
            uploading_files: 0,
            conflicted_files: 0,
            failed_files: 0,
            completed_files: 2,
            total_bytes: 1000,
            bytes_uploaded: 500,
            is_syncing: false,
            network_available: true,
            last_sync_at: None,
            last_sync_error: None,
        };
        assert_eq!(half_done.percent_complete(), 50);

        let zero_bytes = StagingQueueStatus {
            total_bytes: 0,
            bytes_uploaded: 0,
            ..half_done.clone()
        };
        assert_eq!(zero_bytes.percent_complete(), 0); // has files but zero bytes

        let empty_queue = StagingQueueStatus {
            total_files: 0,
            total_bytes: 0,
            bytes_uploaded: 0,
            pending_files: 0,
            completed_files: 0,
            ..half_done
        };
        assert_eq!(empty_queue.percent_complete(), 100); // empty = complete
    }

    #[test]
    fn staging_queue_status_active_count() {
        let status = StagingQueueStatus {
            total_files: 10,
            pending_files: 5,
            uploading_files: 2,
            conflicted_files: 1,
            failed_files: 1,
            completed_files: 1,
            total_bytes: 5000,
            bytes_uploaded: 500,
            is_syncing: true,
            network_available: true,
            last_sync_at: Some(1000),
            last_sync_error: None,
        };
        assert_eq!(status.active_count(), 7); // pending + uploading
    }

    #[test]
    fn staging_event_variants() {
        // Test that all event variants can be constructed
        let staged = StagingEvent::FileStaged {
            upload_id: "u1".to_string(),
            file_name: "file.txt".to_string(),
        };
        assert!(matches!(staged, StagingEvent::FileStaged { .. }));

        let started = StagingEvent::UploadStarted {
            upload_id: "u1".to_string(),
        };
        assert!(matches!(started, StagingEvent::UploadStarted { .. }));

        let progress = StagingEvent::UploadProgress {
            upload_id: "u1".to_string(),
            bytes_uploaded: 500,
            total_bytes: 1000,
        };
        assert!(matches!(progress, StagingEvent::UploadProgress { .. }));

        let completed = StagingEvent::UploadCompleted {
            upload_id: "u1".to_string(),
            destination_path: "/docs/file.txt".to_string(),
        };
        assert!(matches!(completed, StagingEvent::UploadCompleted { .. }));

        let conflict = StagingEvent::ConflictDetected {
            upload_id: "u1".to_string(),
            conflict_type: ConflictType::FileExists,
        };
        assert!(matches!(conflict, StagingEvent::ConflictDetected { .. }));

        let failed = StagingEvent::UploadFailed {
            upload_id: "u1".to_string(),
            error: "network error".to_string(),
        };
        assert!(matches!(failed, StagingEvent::UploadFailed { .. }));

        let cleared = StagingEvent::QueueCleared { files_removed: 5 };
        assert!(matches!(cleared, StagingEvent::QueueCleared { .. }));

        let network = StagingEvent::NetworkStatusChanged { available: true };
        assert!(matches!(network, StagingEvent::NetworkStatusChanged { .. }));

        let sync_start = StagingEvent::SyncStarted;
        assert!(matches!(sync_start, StagingEvent::SyncStarted));

        let sync_done = StagingEvent::SyncCompleted {
            files_uploaded: 10,
            files_failed: 2,
        };
        assert!(matches!(sync_done, StagingEvent::SyncCompleted { .. }));
    }
}
