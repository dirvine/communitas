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
}
