//! Drive DTOs for virtual disk, directory, file, and transfer views.
//!
//! These types are designed for UI rendering and MCP tool responses.
//! They provide a simplified view of the underlying storage system
//! with upload/download progress tracking and checksum verification.

use serde::{Deserialize, Serialize};

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
}
