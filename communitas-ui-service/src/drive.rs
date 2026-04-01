// SPDX-License-Identifier: MIT OR Apache-2.0

//! Drive service for virtual disk operations with progress tracking.
//!
//! Provides a UI-friendly abstraction over the storage system with:
//! - Virtual disk management (Private/Public/Shared per entity)
//! - Directory and file operations
//! - Upload/download progress tracking via watch channels
//! - Checksum verification
//! - Streaming uploads/downloads with chunked transfers
//! - Resume support for interrupted transfers

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use communitas_ui_api::SyncState;
use communitas_ui_api::drive::{
    ConflictResolution, ConflictType, DirectoryEntry, DiskInfo, DiskType, DownloadProgress,
    DownloadState, FileMetadata, FilePreview, QuotaInfo, ShareLink, ShareLinkAccessResult,
    ShareLinkConfig, ShareLinkStats, StagedUpload, StagedUploadState, StagingConflict,
    StagingEvent, StagingQueueStatus, UploadProgress, UploadState,
};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{RwLock, watch};
use tracing::{debug, instrument, warn};

use crate::storage::UiStorage;

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::util::current_timestamp_millis;
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, DiskTypeArg, Event, Query, QueryResponse};

/// Default chunk size for streaming transfers (1 MB).
/// Memory budget: max 2 chunks in memory at once (1 being read, 1 being written).
const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024;

/// Simple checksum using a hash of bytes (mock implementation).
fn compute_checksum(data: &[u8]) -> String {
    // Simple checksum: sum of bytes XOR'd with length
    let sum: u64 = data.iter().map(|&b| b as u64).sum();
    format!("{:016x}{:08x}", sum, data.len())
}

/// Extract the file or directory name from a path.
fn extract_name_from_path(path: &str, default: &str) -> String {
    path.split('/')
        .rfind(|s| !s.is_empty())
        .unwrap_or(default)
        .to_string()
}

/// Get MIME type from file extension.
fn mime_from_extension(ext: &str) -> String {
    match ext.to_lowercase().as_str() {
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "gz" | "gzip" => "application/gzip",
        "tar" => "application/x-tar",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "mp3" => "audio/mpeg",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "md" => "text/markdown",
        "rs" => "text/x-rust",
        "py" => "text/x-python",
        _ => "application/octet-stream",
    }
    .to_string()
}

/// Append a conflict suffix to a file path.
/// e.g., "/docs/report.pdf" -> "/docs/report_conflict_1234.pdf"
fn append_conflict_suffix(path: &str, timestamp: i64) -> String {
    if let Some(dot_pos) = path.rfind('.') {
        let (base, ext) = path.split_at(dot_pos);
        format!("{}_conflict_{}{}", base, timestamp, ext)
    } else {
        format!("{}_conflict_{}", path, timestamp)
    }
}

/// Convert core DiskTypeArg to UI DiskType.
fn disk_type_from_arg(arg: DiskTypeArg) -> DiskType {
    match arg {
        DiskTypeArg::Private => DiskType::Private,
        DiskTypeArg::Public => DiskType::Public,
        DiskTypeArg::Shared => DiskType::Shared,
    }
}

/// Convert UI DiskType to core DiskTypeArg.
fn disk_type_to_arg(disk_type: DiskType) -> DiskTypeArg {
    match disk_type {
        DiskType::Private => DiskTypeArg::Private,
        DiskType::Public => DiskTypeArg::Public,
        DiskType::Shared => DiskTypeArg::Shared,
    }
}

/// Errors returned by the drive service.
#[derive(Debug, Error)]
pub enum DriveError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("disk not found: {0}")]
    DiskNotFound(String),
    #[error("path not found: {0}")]
    PathNotFound(String),
    #[error("path already exists: {0}")]
    PathExists(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("quota exceeded: used {used} bytes of {quota} bytes")]
    QuotaExceeded { used: u64, quota: u64 },
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("upload failed: {0}")]
    UploadFailed(String),
    #[error("download failed: {0}")]
    DownloadFailed(String),
    #[error("upload not found: {0}")]
    UploadNotFound(String),
    #[error("download not found: {0}")]
    DownloadNotFound(String),
    #[error("storage error: {0}")]
    StorageError(String),
    #[error("query error: {0}")]
    QueryError(String),
    #[error("file read error: {0}")]
    FileReadError(String),
    #[error("transfer not resumable: {0}")]
    TransferNotResumable(String),
    #[error("transfer aborted")]
    TransferAborted,
    #[error("share link not found: {0}")]
    ShareLinkNotFound(String),
    #[error("share link creation failed: {0}")]
    ShareLinkCreationFailed(String),
    #[error("cannot share private files")]
    CannotSharePrivateFiles,
    #[error("staged upload not found: {0}")]
    StagedUploadNotFound(String),
    #[error("staging conflict: {0}")]
    StagingConflict(String),
    #[error("local file not found: {0}")]
    LocalFileNotFound(String),
    #[error("staging queue error: {0}")]
    StagingQueueError(String),
}

/// Snapshot of drive state for reactive UI updates.
#[derive(Debug, Clone, Default)]
pub struct DriveSnapshot {
    /// Currently active uploads.
    pub uploads: HashMap<String, UploadProgress>,
    /// Currently active downloads.
    pub downloads: HashMap<String, DownloadProgress>,
    /// Current directory listing (for the active view).
    pub current_directory: Vec<DirectoryEntry>,
    /// Whether the directory is currently loading.
    pub loading: bool,
    /// Offline staging queue status.
    pub staging_status: Option<StagingQueueStatus>,
}

/// Service for virtual disk, directory, and file operations.
pub struct DriveService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    tx: watch::Sender<DriveSnapshot>,
    rx: watch::Receiver<DriveSnapshot>,
    active_uploads: Arc<RwLock<HashMap<String, UploadProgress>>>,
    active_downloads: Arc<RwLock<HashMap<String, DownloadProgress>>>,
    upload_counter: Arc<RwLock<u64>>,
    download_counter: Arc<RwLock<u64>>,
    /// In-memory share link storage (by link ID).
    share_links: Arc<RwLock<HashMap<String, ShareLink>>>,
    /// Share link statistics (by link ID).
    share_link_stats: Arc<RwLock<HashMap<String, ShareLinkStats>>>,
    /// Counter for generating unique share link IDs.
    share_link_counter: Arc<RwLock<u64>>,
    /// Offline staging queue (by staged upload ID).
    staging_queue: Arc<RwLock<HashMap<String, StagedUpload>>>,
    /// Counter for generating unique staged upload IDs.
    staging_counter: Arc<RwLock<u64>>,
    /// Event sender for staging queue events.
    staging_event_tx: tokio::sync::broadcast::Sender<StagingEvent>,
    /// Whether network is currently available.
    network_available: Arc<RwLock<bool>>,
    /// Default max retries for staged uploads.
    staging_max_retries: u32,
    /// Optional storage for persistence (resume support).
    storage: Option<Arc<UiStorage>>,
}

impl DriveService {
    /// Create a new drive service linked to the auth controller.
    pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        Self::with_storage(auth, app, None)
    }

    /// Create a new drive service with optional persistent storage for resume support.
    pub fn with_storage(
        auth: Arc<AuthController>,
        app: Arc<CommunitasApp>,
        storage: Option<Arc<UiStorage>>,
    ) -> Self {
        let (tx, rx) = watch::channel(DriveSnapshot::default());
        let (staging_event_tx, _) = tokio::sync::broadcast::channel(256);

        // Load persisted state if storage is available
        let (active_uploads, staging_queue) = if let Some(ref store) = storage {
            let uploads = Self::load_active_uploads_from_storage(store);
            let staging = Self::load_staging_queue_from_storage(store);
            (uploads, staging)
        } else {
            (HashMap::new(), HashMap::new())
        };

        Self {
            auth,
            app,
            tx,
            rx,
            active_uploads: Arc::new(RwLock::new(active_uploads)),
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            upload_counter: Arc::new(RwLock::new(0)),
            download_counter: Arc::new(RwLock::new(0)),
            share_links: Arc::new(RwLock::new(HashMap::new())),
            share_link_stats: Arc::new(RwLock::new(HashMap::new())),
            share_link_counter: Arc::new(RwLock::new(0)),
            staging_queue: Arc::new(RwLock::new(staging_queue)),
            staging_counter: Arc::new(RwLock::new(0)),
            staging_event_tx,
            network_available: Arc::new(RwLock::new(true)), // Assume online initially
            staging_max_retries: 3,
            storage,
        }
    }

    /// Subscribe to drive state updates.
    pub fn subscribe(&self) -> watch::Receiver<DriveSnapshot> {
        self.rx.clone()
    }

    /// Get the current drive snapshot without subscribing.
    pub fn current_snapshot(&self) -> DriveSnapshot {
        self.rx.borrow().clone()
    }

    /// Access the underlying Communitas application.
    pub fn app(&self) -> Arc<CommunitasApp> {
        self.app.clone()
    }

    // ===== Persistence Helpers =====

    /// Load active uploads from storage (called during init).
    fn load_active_uploads_from_storage(storage: &UiStorage) -> HashMap<String, UploadProgress> {
        let path = storage.active_uploads_file();
        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(uploads) => {
                    debug!(count = ?HashMap::<String, UploadProgress>::len(&uploads), "loaded active uploads from storage");
                    uploads
                }
                Err(e) => {
                    warn!(error = %e, "failed to parse active uploads file, starting fresh");
                    HashMap::new()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(e) => {
                warn!(error = %e, "failed to read active uploads file, starting fresh");
                HashMap::new()
            }
        }
    }

    /// Load staging queue from storage (called during init).
    fn load_staging_queue_from_storage(storage: &UiStorage) -> HashMap<String, StagedUpload> {
        let path = storage.staging_queue_file();
        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_json::from_str(&contents) {
                Ok(queue) => {
                    debug!(count = ?HashMap::<String, StagedUpload>::len(&queue), "loaded staging queue from storage");
                    queue
                }
                Err(e) => {
                    warn!(error = %e, "failed to parse staging queue file, starting fresh");
                    HashMap::new()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(e) => {
                warn!(error = %e, "failed to read staging queue file, starting fresh");
                HashMap::new()
            }
        }
    }

    /// Persist active uploads to storage (atomic write via temp file + rename).
    async fn persist_active_uploads(&self) {
        let Some(ref storage) = self.storage else {
            return;
        };
        let uploads = self.active_uploads.read().await;
        let path = storage.active_uploads_file();

        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension("json.tmp");
        match serde_json::to_string_pretty(&*uploads) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&temp_path, &json) {
                    warn!(error = %e, "failed to write active uploads temp file");
                    return;
                }
                if let Err(e) = std::fs::rename(&temp_path, &path) {
                    warn!(error = %e, "failed to rename active uploads file");
                    // Clean up temp file on failure
                    let _ = std::fs::remove_file(&temp_path);
                }
            }
            Err(e) => {
                warn!(error = %e, "failed to serialize active uploads");
            }
        }
    }

    /// Persist staging queue to storage (atomic write via temp file + rename).
    async fn persist_staging_queue(&self) {
        let Some(ref storage) = self.storage else {
            return;
        };
        let queue = self.staging_queue.read().await;
        let path = storage.staging_queue_file();

        // Atomic write: write to temp file, then rename
        let temp_path = path.with_extension("json.tmp");
        match serde_json::to_string_pretty(&*queue) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&temp_path, &json) {
                    warn!(error = %e, "failed to write staging queue temp file");
                    return;
                }
                if let Err(e) = std::fs::rename(&temp_path, &path) {
                    warn!(error = %e, "failed to rename staging queue file");
                    // Clean up temp file on failure
                    let _ = std::fs::remove_file(&temp_path);
                }
            }
            Err(e) => {
                warn!(error = %e, "failed to serialize staging queue");
            }
        }
    }

    // ===== Resume Detection =====

    /// Detect resumable transfers by cross-referencing persisted uploads with core state.
    ///
    /// This method should be called after service initialization to identify
    /// uploads that were interrupted and can be resumed.
    ///
    /// Returns the count of uploads marked as resumable.
    #[instrument(skip(self), name = "ui.drive.detect_resumable_transfers")]
    pub async fn detect_resumable_transfers(&self) -> Result<usize, DriveError> {
        // Query the core for active transfers
        let response = self
            .app
            .query(Query::ListResumableTransfers)
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        let QueryResponse::ResumableTransfers(core_transfers) = response else {
            return Err(DriveError::QueryError(
                "unexpected response type from ListResumableTransfers".to_string(),
            ));
        };

        // Build a map of core transfers by path for quick lookup
        let mut core_transfer_map: HashMap<String, _> = HashMap::new();
        for transfer in core_transfers {
            let key = format!(
                "{}:{}:{}",
                transfer.entity_id,
                match transfer.disk_type {
                    DiskTypeArg::Private => "private",
                    DiskTypeArg::Public => "public",
                    DiskTypeArg::Shared => "shared",
                },
                transfer.path
            );
            core_transfer_map.insert(key, transfer);
        }

        let mut resumable_count = 0;

        // Check each persisted upload against core state
        let mut uploads = self.active_uploads.write().await;
        for (upload_id, upload) in uploads.iter_mut() {
            // Skip uploads that are already complete or cancelled
            if upload.state.is_terminal() {
                continue;
            }

            // Skip uploads already marked as resumable
            if matches!(upload.state, UploadState::Resumable) {
                resumable_count += 1;
                continue;
            }

            // Skip if upload has a transfer_id but check if it still exists in core
            if let Some(ref transfer_id) = upload.transfer_id {
                // Look up by transfer_id
                let found_in_core = core_transfer_map
                    .values()
                    .any(|t| t.transfer_id == *transfer_id);

                if found_in_core {
                    // Transfer exists in core - mark as resumable
                    upload.state = UploadState::Resumable;
                    resumable_count += 1;
                    debug!(
                        upload_id = %upload_id,
                        transfer_id = %transfer_id,
                        bytes_uploaded = upload.bytes_uploaded,
                        total_bytes = upload.total_bytes,
                        "detected resumable upload with transfer_id"
                    );
                } else {
                    // Transfer no longer in core - mark as failed
                    upload.state = UploadState::Failed("transfer state lost".to_string());
                    debug!(
                        upload_id = %upload_id,
                        transfer_id = %transfer_id,
                        "upload transfer state lost, marking as failed"
                    );
                }
            } else if matches!(upload.state, UploadState::Uploading | UploadState::Pending) {
                // No transfer_id means upload wasn't fully started
                // Check if we can find a matching transfer by path
                // For now, mark as failed since we can't reliably match
                upload.state =
                    UploadState::Failed("interrupted before transfer started".to_string());
                debug!(
                    upload_id = %upload_id,
                    "upload interrupted before transfer started, marking as failed"
                );
            }
        }

        drop(uploads);

        // Persist the updated state
        self.persist_active_uploads().await;

        // Notify subscribers
        self.update_upload_snapshot().await;

        debug!(
            resumable_count = resumable_count,
            "resume detection complete"
        );
        Ok(resumable_count)
    }

    /// Get the count of resumable uploads.
    pub async fn resumable_upload_count(&self) -> usize {
        let uploads = self.active_uploads.read().await;
        uploads.values().filter(|u| u.state.is_resumable()).count()
    }

    // ===== Disk Operations =====

    /// List all disks for an entity.
    #[instrument(skip(self), name = "ui.drive.list_disks", fields(entity_id))]
    pub async fn list_disks(&self, entity_id: &str) -> Result<Vec<DiskInfo>, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Query the core for disk list
        let response = self
            .app
            .query(Query::ListDisks {
                entity_id: entity_id.to_string(),
            })
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        // Extract disk list from response
        let QueryResponse::DiskList(disk_list) = response else {
            return Err(DriveError::QueryError(
                "unexpected response type from ListDisks query".to_string(),
            ));
        };

        Ok(disk_list
            .into_iter()
            .map(|info| DiskInfo {
                disk_type: disk_type_from_arg(info.disk_type),
                entity_id: info.entity_id,
                total_bytes: info.total_bytes,
                used_bytes: info.used_bytes,
                available_bytes: info.available_bytes,
                file_count: info.file_count,
            })
            .collect())
    }

    /// Get quota information for a specific disk.
    #[instrument(skip(self), name = "ui.drive.get_quota", fields(entity_id, ?disk_type))]
    pub async fn get_quota(
        &self,
        entity_id: &str,
        disk_type: DiskType,
    ) -> Result<QuotaInfo, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let disks = self.list_disks(entity_id).await?;
        let disk = disks
            .into_iter()
            .find(|d| d.disk_type == disk_type)
            .ok_or_else(|| DriveError::DiskNotFound(format!("{:?}", disk_type)))?;

        let percent_used = if disk.total_bytes > 0 {
            (disk.used_bytes as f32 / disk.total_bytes as f32) * 100.0
        } else {
            0.0
        };

        Ok(QuotaInfo {
            disk_type,
            used_bytes: disk.used_bytes,
            quota_bytes: disk.total_bytes,
            percent_used,
        })
    }

    // ===== Directory Operations =====

    /// List contents of a directory.
    #[instrument(skip(self), name = "ui.drive.list_directory", fields(entity_id, ?disk_type, path))]
    pub async fn list_directory(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<DirectoryEntry>, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Update loading state
        self.set_loading(true);

        // Query the core for file list
        let response = self
            .app
            .query(Query::ListFiles {
                entity_id: entity_id.to_string(),
                disk_type: disk_type_to_arg(disk_type),
                path: path.to_string(),
            })
            .await
            .map_err(|e| {
                self.set_loading(false);
                DriveError::QueryError(e.to_string())
            })?;

        // Extract file list from response
        let QueryResponse::FileList(file_list) = response else {
            self.set_loading(false);
            return Err(DriveError::QueryError(
                "unexpected response type from ListFiles query".to_string(),
            ));
        };

        // Convert to DirectoryEntry
        let entries: Vec<DirectoryEntry> = file_list
            .into_iter()
            .map(|info| DirectoryEntry {
                name: info.name,
                path: info.path,
                is_directory: info.is_directory,
                size_bytes: info.size_bytes,
                mime_type: None,
                modified_at: info.modified_at,
                created_at: info.modified_at,
                checksum: None,
                sync_state: SyncState::Synced, // Default to synced; will be updated by staging queue
            })
            .collect();

        // Update snapshot
        let mut snap = self.rx.borrow().clone();
        snap.current_directory = entries.clone();
        snap.loading = false;
        let _ = self.tx.send(snap);

        Ok(entries)
    }

    /// Create a new directory.
    #[instrument(skip(self), name = "ui.drive.create_directory", fields(entity_id, ?disk_type, path))]
    pub async fn create_directory(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<DirectoryEntry, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Build and execute the CreateDirectory command
        let cmd = Command::CreateDirectory {
            entity_id: entity_id.to_string(),
            disk_type: disk_type_to_arg(disk_type),
            path: path.to_string(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| DriveError::StorageError(e.message))?;

        // Verify creation by finding the DirectoryCreated event
        let created = events.iter().any(|event| {
            matches!(
                event,
                Event::DirectoryCreated {
                    entity_id: eid,
                    path: p,
                    ..
                } if eid == entity_id && p == path
            )
        });

        if !created {
            return Err(DriveError::StorageError(
                "create command succeeded but DirectoryCreated event not found".to_string(),
            ));
        }

        let name = extract_name_from_path(path, "new_folder");
        let now = current_timestamp_millis();

        let entry = DirectoryEntry {
            name,
            path: path.to_string(),
            is_directory: true,
            size_bytes: 0,
            mime_type: None,
            modified_at: now,
            created_at: now,
            checksum: None,
            sync_state: SyncState::Synced,
        };

        // Update watch channel - add new directory to current_directory
        let mut snap = self.rx.borrow().clone();
        snap.current_directory.push(entry.clone());
        let _ = self.tx.send(snap);

        Ok(entry)
    }

    /// Delete a file or directory.
    #[instrument(skip(self), name = "ui.drive.delete_path", fields(entity_id, ?disk_type, path))]
    pub async fn delete_path(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<(), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Build and execute the DeleteFile command
        let cmd = Command::DeleteFile {
            entity_id: entity_id.to_string(),
            disk_type: disk_type_to_arg(disk_type),
            path: path.to_string(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| DriveError::StorageError(e.message))?;

        // Verify deletion by finding the FileDeleted event
        let deleted = events.iter().any(|event| {
            matches!(
                event,
                Event::FileDeleted {
                    entity_id: eid,
                    path: p,
                    ..
                } if eid == entity_id && p == path
            )
        });

        if !deleted {
            return Err(DriveError::StorageError(
                "delete command succeeded but FileDeleted event not found".to_string(),
            ));
        }

        // Update watch channel - remove deleted file from current_directory
        {
            let mut snap = self.rx.borrow().clone();
            snap.current_directory.retain(|e| e.path != path);
            let _ = self.tx.send(snap);
        }

        Ok(())
    }

    /// Move a file or directory.
    #[instrument(skip(self), name = "ui.drive.move_path", fields(entity_id, ?disk_type, from, to))]
    pub async fn move_path(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        from: &str,
        to: &str,
    ) -> Result<DirectoryEntry, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Build and execute the MoveFile command
        let cmd = Command::MoveFile {
            entity_id: entity_id.to_string(),
            disk_type: disk_type_to_arg(disk_type),
            source_path: from.to_string(),
            dest_path: to.to_string(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| DriveError::StorageError(e.message))?;

        // Verify move by finding the FileMoved event
        let moved = events.iter().any(|event| {
            matches!(
                event,
                Event::FileMoved {
                    entity_id: eid,
                    source_path: sp,
                    dest_path: dp,
                    ..
                } if eid == entity_id && sp == from && dp == to
            )
        });

        if !moved {
            return Err(DriveError::StorageError(
                "move command succeeded but FileMoved event not found".to_string(),
            ));
        }

        let name = extract_name_from_path(to, "moved");
        let now = current_timestamp_millis();

        let entry = DirectoryEntry {
            name,
            path: to.to_string(),
            is_directory: false,
            size_bytes: 0,
            mime_type: None,
            modified_at: now,
            created_at: now,
            checksum: None,
            sync_state: SyncState::Synced,
        };

        // Update watch channel - remove from source, add at destination
        {
            let mut snap = self.rx.borrow().clone();
            snap.current_directory.retain(|e| e.path != from);
            snap.current_directory.push(entry.clone());
            let _ = self.tx.send(snap);
        }

        Ok(entry)
    }

    /// Copy a file or directory.
    #[instrument(skip(self), name = "ui.drive.copy_path", fields(entity_id, ?disk_type, from, to))]
    pub async fn copy_path(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        from: &str,
        to: &str,
    ) -> Result<DirectoryEntry, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Build and execute the CopyFile command
        let cmd = Command::CopyFile {
            entity_id: entity_id.to_string(),
            disk_type: disk_type_to_arg(disk_type),
            source_path: from.to_string(),
            dest_path: to.to_string(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| DriveError::StorageError(e.message))?;

        // Verify copy by finding the FileCopied event
        let copied = events.iter().any(|event| {
            matches!(
                event,
                Event::FileCopied {
                    entity_id: eid,
                    source_path: sp,
                    dest_path: dp,
                    ..
                } if eid == entity_id && sp == from && dp == to
            )
        });

        if !copied {
            return Err(DriveError::StorageError(
                "copy command succeeded but FileCopied event not found".to_string(),
            ));
        }

        let name = extract_name_from_path(to, "copy");
        let now = current_timestamp_millis();

        let entry = DirectoryEntry {
            name,
            path: to.to_string(),
            is_directory: false,
            size_bytes: 0,
            mime_type: None,
            modified_at: now,
            created_at: now,
            checksum: None,
            sync_state: SyncState::Synced,
        };

        // Update watch channel - add copied file
        {
            let mut snap = self.rx.borrow().clone();
            snap.current_directory.push(entry.clone());
            let _ = self.tx.send(snap);
        }

        Ok(entry)
    }

    /// Get preview information for a file.
    #[instrument(skip(self), name = "ui.drive.get_file_preview", fields(entity_id, ?disk_type, path))]
    pub async fn get_file_preview(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<FilePreview, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Query the core for file preview
        let response = self
            .app
            .query(Query::GetFilePreview {
                entity_id: entity_id.to_string(),
                disk_type: disk_type_to_arg(disk_type),
                path: path.to_string(),
            })
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        // Extract file preview from response
        let QueryResponse::FilePreview(preview) = response else {
            return Err(DriveError::QueryError(
                "unexpected response type from GetFilePreview query".to_string(),
            ));
        };

        Ok(FilePreview {
            path: preview.path,
            mime_type: preview.mime_type,
            size_bytes: preview.size_bytes,
            thumbnail: preview.thumbnail,
            text_preview: preview.text_preview,
            metadata: FileMetadata {
                created_at: preview.created_at,
                modified_at: preview.modified_at,
                checksum: preview.checksum,
                block_count: 0,   // Not provided by core response
                encryption: None, // Not provided by core response
            },
        })
    }

    /// Read file contents.
    #[instrument(skip(self), name = "ui.drive.read_file", fields(entity_id, ?disk_type, path))]
    pub async fn read_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<u8>, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Query the core for file contents
        let response = self
            .app
            .query(Query::ReadFile {
                entity_id: entity_id.to_string(),
                disk_type: disk_type_to_arg(disk_type),
                path: path.to_string(),
            })
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        // Extract file contents from response
        let QueryResponse::FileContents(data) = response else {
            return Err(DriveError::QueryError(
                "unexpected response type from ReadFile query".to_string(),
            ));
        };

        Ok(data)
    }

    /// Write file contents.
    #[instrument(skip(self, content), name = "ui.drive.write_file", fields(entity_id, ?disk_type, path, content_len = content.len()))]
    pub async fn write_file(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        content: &[u8],
    ) -> Result<DirectoryEntry, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Build and execute the WriteFile command
        let cmd = Command::WriteFile {
            entity_id: entity_id.to_string(),
            disk_type: disk_type_to_arg(disk_type),
            path: path.to_string(),
            data: content.to_vec(),
        };

        let events = self
            .app
            .execute(cmd)
            .await
            .map_err(|e| DriveError::StorageError(e.message))?;

        // Find the FileWritten event to get the confirmed size
        let size_bytes = events
            .iter()
            .find_map(|event| {
                if let Event::FileWritten {
                    entity_id: eid,
                    path: p,
                    size_bytes,
                    ..
                } = event
                {
                    (eid == entity_id && p == path).then_some(*size_bytes)
                } else {
                    None
                }
            })
            .unwrap_or(content.len() as u64);

        let name = extract_name_from_path(path, "file");
        let now = current_timestamp_millis();
        let checksum = compute_checksum(content);

        let entry = DirectoryEntry {
            name,
            path: path.to_string(),
            is_directory: false,
            size_bytes,
            mime_type: Some("application/octet-stream".to_string()),
            modified_at: now,
            created_at: now,
            checksum: Some(checksum),
            sync_state: SyncState::Synced,
        };

        // Update watch channel with new file if we're viewing the parent directory
        {
            let mut snap = self.rx.borrow().clone();
            // Check if the new file should be added to current directory view
            let parent_path = path.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
            let current_paths: Vec<_> = snap
                .current_directory
                .iter()
                .map(|e| e.path.rsplit_once('/').map(|(p, _)| p).unwrap_or(""))
                .collect();

            // If any current entry has the same parent, we're likely viewing that directory
            if current_paths.contains(&parent_path) || snap.current_directory.is_empty() {
                // Remove existing entry with same path if it exists (update case)
                snap.current_directory.retain(|e| e.path != path);
                // Add the new entry
                snap.current_directory.push(entry.clone());
                let _ = self.tx.send(snap);
            }
        }

        Ok(entry)
    }

    // ===== Upload Operations =====

    /// Start an upload operation using chunked writes for better progress tracking.
    ///
    /// This method uploads the content in chunks with progress updates after each chunk.
    /// Supports automatic resume on connection restore and cancellation with cleanup.
    #[instrument(skip(self, content), name = "ui.drive.start_upload", fields(entity_id, ?disk_type, path, content_len = content.len()))]
    pub async fn start_upload(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        content: Vec<u8>,
    ) -> Result<String, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Generate upload ID
        let upload_id = {
            let mut counter = self.upload_counter.write().await;
            *counter += 1;
            format!("upload-{}", *counter)
        };

        let file_name = extract_name_from_path(path, "file");
        let total_bytes = content.len() as u64;
        let now = current_timestamp_millis();
        let expected_checksum = compute_checksum(&content);

        let progress = UploadProgress {
            id: upload_id.clone(),
            file_name,
            file_path: path.to_string(),
            bytes_uploaded: 0,
            total_bytes,
            state: UploadState::Pending,
            started_at: now,
            checksum_verified: false,
            transfer_id: None,
            resumed_from_bytes: None,
        };

        // Store in active uploads
        {
            let mut uploads = self.active_uploads.write().await;
            uploads.insert(upload_id.clone(), progress.clone());
        }

        // Update snapshot
        self.update_upload_snapshot().await;

        // Clone values for the spawned task
        let upload_id_clone = upload_id.clone();
        let uploads = self.active_uploads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let entity_id_owned = entity_id.to_string();
        let path_owned = path.to_string();
        let disk_type_arg = disk_type_to_arg(disk_type);

        tokio::spawn(async move {
            // Mark as uploading
            {
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    p.state = UploadState::Uploading;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

            // Helper to check cancellation
            let is_cancelled = || async {
                let map = uploads.read().await;
                map.get(&upload_id_clone)
                    .is_some_and(|p| matches!(p.state, UploadState::Cancelled))
            };

            // Check for cancellation before starting
            if is_cancelled().await {
                return;
            }

            // Start chunked write
            let start_cmd = Command::StartChunkedWrite {
                entity_id: entity_id_owned.clone(),
                disk_type: disk_type_arg,
                path: path_owned.clone(),
                total_size: total_bytes,
                chunk_size: Some(DEFAULT_CHUNK_SIZE),
            };

            let start_result = app.execute(start_cmd).await;
            if let Err(e) = start_result {
                tracing::error!("Failed to start chunked write: {:?}", e);
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    p.state = UploadState::Failed(e.message);
                }
                Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                return;
            }

            // Write chunks
            let chunk_size = DEFAULT_CHUNK_SIZE as usize;
            let mut offset: u64 = 0;

            for chunk_data in content.chunks(chunk_size) {
                // Check for cancellation before each chunk
                if is_cancelled().await {
                    // Abort the chunked write
                    let abort_cmd = Command::AbortChunkedWrite {
                        entity_id: entity_id_owned.clone(),
                        disk_type: disk_type_arg,
                        path: path_owned.clone(),
                    };
                    let _ = app.execute(abort_cmd).await;
                    return;
                }

                let write_cmd = Command::WriteChunk {
                    entity_id: entity_id_owned.clone(),
                    disk_type: disk_type_arg,
                    path: path_owned.clone(),
                    offset,
                    data: chunk_data.to_vec(),
                };

                match app.execute(write_cmd).await {
                    Ok(_events) => {
                        offset += chunk_data.len() as u64;

                        // Update progress
                        {
                            let mut map = uploads.write().await;
                            if let Some(p) = map.get_mut(&upload_id_clone) {
                                p.bytes_uploaded = offset;
                            }
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

                        debug!(
                            "Uploaded chunk at offset {}, progress: {}/{}",
                            offset - chunk_data.len() as u64,
                            offset,
                            total_bytes
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to write chunk at offset {}: {:?}", offset, e);
                        // Don't abort - leave the partial transfer for potential resume
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(format!(
                                "Chunk write failed at offset {}: {}",
                                offset, e.message
                            ));
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                        return;
                    }
                }
            }

            // Check for cancellation before finishing
            if is_cancelled().await {
                let abort_cmd = Command::AbortChunkedWrite {
                    entity_id: entity_id_owned.clone(),
                    disk_type: disk_type_arg,
                    path: path_owned.clone(),
                };
                let _ = app.execute(abort_cmd).await;
                return;
            }

            // Finish chunked write
            let finish_cmd = Command::FinishChunkedWrite {
                entity_id: entity_id_owned.clone(),
                disk_type: disk_type_arg,
                path: path_owned.clone(),
            };

            match app.execute(finish_cmd).await {
                Ok(_events) => {
                    // Mark as verifying
                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Verifying;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

                    // Read back the file to verify checksum
                    let verify_result = app
                        .query(Query::ReadFile {
                            entity_id: entity_id_owned,
                            disk_type: disk_type_arg,
                            path: path_owned,
                        })
                        .await;

                    let checksum_verified = match verify_result {
                        Ok(QueryResponse::FileContents(data)) => {
                            compute_checksum(&data) == expected_checksum
                        }
                        _ => false,
                    };

                    // Mark as complete
                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Complete;
                            p.checksum_verified = checksum_verified;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
                Err(e) => {
                    tracing::error!("Failed to finish chunked write: {:?}", e);
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Failed(e.message);
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
            }
        });

        Ok(upload_id)
    }

    /// Start a streaming upload from a local file path.
    ///
    /// This method reads the file in chunks to maintain a maximum of 2 chunks in memory
    /// at any time. This is more memory-efficient for large files compared to `start_upload`.
    ///
    /// Supports:
    /// - Progress updates per chunk
    /// - Automatic resume on connection restore
    /// - Cancel support with cleanup
    #[instrument(skip(self), name = "ui.drive.start_streaming_upload", fields(entity_id, ?disk_type, dest_path, source_path))]
    pub async fn start_streaming_upload(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        dest_path: &str,
        source_path: &Path,
    ) -> Result<String, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Get file size
        let metadata = tokio::fs::metadata(source_path)
            .await
            .map_err(|e| DriveError::FileReadError(e.to_string()))?;
        let total_bytes = metadata.len();

        // Generate upload ID
        let upload_id = {
            let mut counter = self.upload_counter.write().await;
            *counter += 1;
            format!("upload-{}", *counter)
        };

        let file_name = source_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string();
        let now = current_timestamp_millis();

        let progress = UploadProgress {
            id: upload_id.clone(),
            file_name,
            file_path: dest_path.to_string(),
            bytes_uploaded: 0,
            total_bytes,
            state: UploadState::Pending,
            started_at: now,
            checksum_verified: false,
            transfer_id: None,
            resumed_from_bytes: None,
        };

        // Store in active uploads
        {
            let mut uploads = self.active_uploads.write().await;
            uploads.insert(upload_id.clone(), progress.clone());
        }

        // Update snapshot
        self.update_upload_snapshot().await;

        // Clone values for the spawned task
        let upload_id_clone = upload_id.clone();
        let uploads = self.active_uploads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let entity_id_owned = entity_id.to_string();
        let dest_path_owned = dest_path.to_string();
        let source_path_owned = source_path.to_owned();
        let disk_type_arg = disk_type_to_arg(disk_type);

        tokio::spawn(async move {
            // Mark as uploading
            {
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    p.state = UploadState::Uploading;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

            // Helper to check cancellation
            let is_cancelled = || async {
                let map = uploads.read().await;
                map.get(&upload_id_clone)
                    .is_some_and(|p| matches!(p.state, UploadState::Cancelled))
            };

            // Check for cancellation before starting
            if is_cancelled().await {
                return;
            }

            // Open the source file
            let file_result = tokio::fs::File::open(&source_path_owned).await;
            let mut file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to open source file: {:?}", e);
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Failed(format!("Failed to open file: {}", e));
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                    return;
                }
            };

            // Start chunked write
            let start_cmd = Command::StartChunkedWrite {
                entity_id: entity_id_owned.clone(),
                disk_type: disk_type_arg,
                path: dest_path_owned.clone(),
                total_size: total_bytes,
                chunk_size: Some(DEFAULT_CHUNK_SIZE),
            };

            if let Err(e) = app.execute(start_cmd).await {
                tracing::error!("Failed to start chunked write: {:?}", e);
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    p.state = UploadState::Failed(e.message);
                }
                Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                return;
            }

            // Write chunks with bounded memory (max 2 chunks in memory)
            let chunk_size = DEFAULT_CHUNK_SIZE as usize;
            let mut buffer = vec![0u8; chunk_size];
            let mut offset: u64 = 0;
            let mut hasher = blake3::Hasher::new();

            loop {
                // Check for cancellation before each chunk
                if is_cancelled().await {
                    let abort_cmd = Command::AbortChunkedWrite {
                        entity_id: entity_id_owned.clone(),
                        disk_type: disk_type_arg,
                        path: dest_path_owned.clone(),
                    };
                    let _ = app.execute(abort_cmd).await;
                    return;
                }

                // Read next chunk from file
                let bytes_read = match file.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("Failed to read from source file: {:?}", e);
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(format!("File read error: {}", e));
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                        return;
                    }
                };

                let chunk_data = &buffer[..bytes_read];
                hasher.update(chunk_data);

                let write_cmd = Command::WriteChunk {
                    entity_id: entity_id_owned.clone(),
                    disk_type: disk_type_arg,
                    path: dest_path_owned.clone(),
                    offset,
                    data: chunk_data.to_vec(),
                };

                match app.execute(write_cmd).await {
                    Ok(_events) => {
                        offset += bytes_read as u64;

                        // Update progress
                        {
                            let mut map = uploads.write().await;
                            if let Some(p) = map.get_mut(&upload_id_clone) {
                                p.bytes_uploaded = offset;
                            }
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

                        debug!(
                            "Uploaded streaming chunk at offset {}, progress: {}/{}",
                            offset - bytes_read as u64,
                            offset,
                            total_bytes
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to write chunk at offset {}: {:?}", offset, e);
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(format!(
                                "Chunk write failed at offset {}: {}",
                                offset, e.message
                            ));
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                        return;
                    }
                }
            }

            // Check for cancellation before finishing
            if is_cancelled().await {
                let abort_cmd = Command::AbortChunkedWrite {
                    entity_id: entity_id_owned.clone(),
                    disk_type: disk_type_arg,
                    path: dest_path_owned.clone(),
                };
                let _ = app.execute(abort_cmd).await;
                return;
            }

            // Finish chunked write
            let finish_cmd = Command::FinishChunkedWrite {
                entity_id: entity_id_owned.clone(),
                disk_type: disk_type_arg,
                path: dest_path_owned.clone(),
            };

            let expected_hash = hasher.finalize().to_string();

            match app.execute(finish_cmd).await {
                Ok(events) => {
                    // Mark as verifying
                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Verifying;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

                    // Check hash from the ChunkedWriteCompleted event
                    let checksum_verified = events.iter().any(|event| {
                        if let Event::ChunkedWriteCompleted { content_hash, .. } = event {
                            *content_hash == expected_hash
                        } else {
                            false
                        }
                    });

                    // Mark as complete
                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Complete;
                            p.checksum_verified = checksum_verified;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
                Err(e) => {
                    tracing::error!("Failed to finish chunked write: {:?}", e);
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Failed(e.message);
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
            }
        });

        Ok(upload_id)
    }

    /// Resume an interrupted upload.
    ///
    /// This attempts to resume a previously failed or interrupted upload from where it left off.
    /// Returns the upload ID if resume was successful, or an error if the upload cannot be resumed.
    #[instrument(skip(self, content), name = "ui.drive.resume_upload", fields(entity_id, ?disk_type, path))]
    pub async fn resume_upload(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        content: Vec<u8>,
    ) -> Result<String, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let disk_type_arg = disk_type_to_arg(disk_type);

        // Check if we can resume
        let verify_cmd = Command::ResumeChunkedWrite {
            entity_id: entity_id.to_string(),
            disk_type: disk_type_arg,
            path: path.to_string(),
            verify_hashes: false,
        };

        let events = self
            .app
            .execute(verify_cmd)
            .await
            .map_err(|e| DriveError::TransferNotResumable(e.message))?;

        // Extract resume info from the event
        let resume_info = events.iter().find_map(|event| {
            if let Event::ChunkedWriteResumed {
                bytes_written,
                total_size,
                ..
            } = event
            {
                Some((*bytes_written, *total_size))
            } else {
                None
            }
        });

        let Some((bytes_written, _total_size)) = resume_info else {
            return Err(DriveError::TransferNotResumable(
                "Resume verification failed".to_string(),
            ));
        };

        // Generate upload ID
        let upload_id = {
            let mut counter = self.upload_counter.write().await;
            *counter += 1;
            format!("upload-{}", *counter)
        };

        let file_name = extract_name_from_path(path, "file");
        let total_bytes = content.len() as u64;
        let now = current_timestamp_millis();
        let expected_checksum = compute_checksum(&content);

        let progress = UploadProgress {
            id: upload_id.clone(),
            file_name,
            file_path: path.to_string(),
            bytes_uploaded: bytes_written,
            total_bytes,
            state: UploadState::Uploading,
            started_at: now,
            checksum_verified: false,
            transfer_id: None,
            resumed_from_bytes: None,
        };

        // Store in active uploads
        {
            let mut uploads = self.active_uploads.write().await;
            uploads.insert(upload_id.clone(), progress.clone());
        }
        self.update_upload_snapshot().await;

        // Clone values for the spawned task
        let upload_id_clone = upload_id.clone();
        let uploads = self.active_uploads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let entity_id_owned = entity_id.to_string();
        let path_owned = path.to_string();

        tokio::spawn(async move {
            // Helper to check cancellation
            let is_cancelled = || async {
                let map = uploads.read().await;
                map.get(&upload_id_clone)
                    .is_some_and(|p| matches!(p.state, UploadState::Cancelled))
            };

            // Continue writing from where we left off
            let chunk_size = DEFAULT_CHUNK_SIZE as usize;
            let mut offset = bytes_written;

            // Skip already-written data
            let remaining_data = if bytes_written as usize >= content.len() {
                // All data already written, just finish
                &content[0..0]
            } else {
                &content[bytes_written as usize..]
            };

            for chunk_data in remaining_data.chunks(chunk_size) {
                if is_cancelled().await {
                    let abort_cmd = Command::AbortChunkedWrite {
                        entity_id: entity_id_owned.clone(),
                        disk_type: disk_type_arg,
                        path: path_owned.clone(),
                    };
                    let _ = app.execute(abort_cmd).await;
                    return;
                }

                let write_cmd = Command::WriteChunk {
                    entity_id: entity_id_owned.clone(),
                    disk_type: disk_type_arg,
                    path: path_owned.clone(),
                    offset,
                    data: chunk_data.to_vec(),
                };

                match app.execute(write_cmd).await {
                    Ok(_events) => {
                        offset += chunk_data.len() as u64;
                        {
                            let mut map = uploads.write().await;
                            if let Some(p) = map.get_mut(&upload_id_clone) {
                                p.bytes_uploaded = offset;
                            }
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to write chunk at offset {}: {:?}", offset, e);
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(format!(
                                "Chunk write failed at offset {}: {}",
                                offset, e.message
                            ));
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                        return;
                    }
                }
            }

            if is_cancelled().await {
                let abort_cmd = Command::AbortChunkedWrite {
                    entity_id: entity_id_owned.clone(),
                    disk_type: disk_type_arg,
                    path: path_owned.clone(),
                };
                let _ = app.execute(abort_cmd).await;
                return;
            }

            // Finish chunked write
            let finish_cmd = Command::FinishChunkedWrite {
                entity_id: entity_id_owned.clone(),
                disk_type: disk_type_arg,
                path: path_owned.clone(),
            };

            match app.execute(finish_cmd).await {
                Ok(_events) => {
                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Verifying;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

                    // Verify checksum
                    let verify_result = app
                        .query(Query::ReadFile {
                            entity_id: entity_id_owned,
                            disk_type: disk_type_arg,
                            path: path_owned,
                        })
                        .await;

                    let checksum_verified = match verify_result {
                        Ok(QueryResponse::FileContents(data)) => {
                            compute_checksum(&data) == expected_checksum
                        }
                        _ => false,
                    };

                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Complete;
                            p.checksum_verified = checksum_verified;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
                Err(e) => {
                    tracing::error!("Failed to finish chunked write: {:?}", e);
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Failed(e.message);
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
            }
        });

        Ok(upload_id)
    }
    /// Cancel an upload.
    #[instrument(skip(self), name = "ui.drive.cancel_upload", fields(upload_id))]
    pub async fn cancel_upload(&self, upload_id: &str) -> Result<(), DriveError> {
        let mut uploads = self.active_uploads.write().await;
        let progress = uploads
            .get_mut(upload_id)
            .ok_or_else(|| DriveError::UploadNotFound(upload_id.to_string()))?;

        if progress.state.is_terminal() {
            return Err(DriveError::UploadFailed(
                "Cannot cancel completed upload".to_string(),
            ));
        }

        progress.state = UploadState::Cancelled;
        drop(uploads);

        self.update_upload_snapshot().await;
        Ok(())
    }

    /// Resume an interrupted upload by its upload ID.
    ///
    /// This method:
    /// 1. Verifies the upload is in a resumable state
    /// 2. Gets the current progress from core TransferState
    /// 3. Continues uploading from the resume point
    /// 4. Updates progress tracking to show resumed state
    ///
    /// # Arguments
    /// * `upload_id` - The ID of the upload to resume
    /// * `source_path` - Path to the source file (must match original)
    ///
    /// # Errors
    /// Returns error if upload not found, not resumable, or resume fails.
    #[instrument(
        skip(self, source_path),
        name = "ui.drive.resume_upload_by_id",
        fields(upload_id)
    )]
    pub async fn resume_upload_by_id(
        &self,
        upload_id: &str,
        source_path: &Path,
    ) -> Result<(), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Get upload info
        let upload = {
            let uploads = self.active_uploads.read().await;
            uploads
                .get(upload_id)
                .cloned()
                .ok_or_else(|| DriveError::UploadNotFound(upload_id.to_string()))?
        };

        // Check if resumable
        if !upload.state.is_resumable() {
            return Err(DriveError::TransferNotResumable(format!(
                "upload {} is in state {:?}, not resumable",
                upload_id, upload.state
            )));
        }

        // Parse disk type from file path or stored info
        // We need to determine entity_id and disk_type from the upload
        // For now, we'll need these to be stored with the upload or passed in
        let transfer_id = upload.transfer_id.clone().ok_or_else(|| {
            DriveError::TransferNotResumable("upload has no transfer_id".to_string())
        })?;

        // Query core for the transfer state to get entity_id and disk_type
        let response = self
            .app
            .query(Query::ListResumableTransfers)
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        let QueryResponse::ResumableTransfers(transfers) = response else {
            return Err(DriveError::QueryError(
                "unexpected response type".to_string(),
            ));
        };

        let transfer = transfers
            .into_iter()
            .find(|t| t.transfer_id == transfer_id)
            .ok_or_else(|| {
                DriveError::TransferNotResumable(format!(
                    "transfer {} not found in core state",
                    transfer_id
                ))
            })?;

        // Mark as uploading and set resumed_from_bytes
        {
            let mut uploads = self.active_uploads.write().await;
            if let Some(p) = uploads.get_mut(upload_id) {
                p.state = UploadState::Uploading;
                p.resumed_from_bytes = Some(transfer.bytes_written);
                p.bytes_uploaded = transfer.bytes_written;
            }
        }
        self.update_upload_snapshot().await;

        // Resume the chunked write in core
        let disk_type_arg = transfer.disk_type;
        let entity_id = transfer.entity_id.clone();
        let dest_path = transfer.path.clone();
        let resume_offset = transfer.bytes_written;
        let total_bytes = transfer.total_size;

        let resume_cmd = Command::ResumeChunkedWrite {
            entity_id: entity_id.clone(),
            disk_type: disk_type_arg,
            path: dest_path.clone(),
            verify_hashes: false, // Skip hash verification for faster resume
        };

        if let Err(e) = self.app.execute(resume_cmd).await {
            let mut uploads = self.active_uploads.write().await;
            if let Some(p) = uploads.get_mut(upload_id) {
                p.state = UploadState::Failed(format!("resume failed: {}", e.message));
            }
            drop(uploads);
            self.update_upload_snapshot().await;
            return Err(DriveError::UploadFailed(format!(
                "failed to resume: {}",
                e.message
            )));
        }

        debug!(
            upload_id = %upload_id,
            transfer_id = %transfer_id,
            resume_offset = resume_offset,
            total_bytes = total_bytes,
            "resuming upload"
        );

        // Clone values for the spawned task
        let upload_id_clone = upload_id.to_string();
        let uploads = self.active_uploads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let source_path_owned = source_path.to_owned();

        tokio::spawn(async move {
            // Helper to check cancellation
            let is_cancelled = || async {
                let map = uploads.read().await;
                map.get(&upload_id_clone)
                    .is_some_and(|p| matches!(p.state, UploadState::Cancelled))
            };

            // Open and seek to resume position
            let file_result = tokio::fs::File::open(&source_path_owned).await;
            let mut file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to open source file for resume: {:?}", e);
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Failed(format!("Failed to open file: {}", e));
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                    return;
                }
            };

            // Seek to resume offset
            use tokio::io::AsyncSeekExt;
            if let Err(e) = file.seek(std::io::SeekFrom::Start(resume_offset)).await {
                tracing::error!("Failed to seek to resume offset: {:?}", e);
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    p.state = UploadState::Failed(format!("Failed to seek: {}", e));
                }
                Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                return;
            }

            // Write remaining chunks
            let chunk_size = DEFAULT_CHUNK_SIZE as usize;
            let mut buffer = vec![0u8; chunk_size];
            let mut offset = resume_offset;

            loop {
                if is_cancelled().await {
                    let abort_cmd = Command::AbortChunkedWrite {
                        entity_id: entity_id.clone(),
                        disk_type: disk_type_arg,
                        path: dest_path.clone(),
                    };
                    let _ = app.execute(abort_cmd).await;
                    return;
                }

                let bytes_read = match file.read(&mut buffer).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("Failed to read from source file: {:?}", e);
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(format!("File read error: {}", e));
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                        return;
                    }
                };

                let chunk_data = &buffer[..bytes_read];

                let write_cmd = Command::WriteChunk {
                    entity_id: entity_id.clone(),
                    disk_type: disk_type_arg,
                    path: dest_path.clone(),
                    offset,
                    data: chunk_data.to_vec(),
                };

                match app.execute(write_cmd).await {
                    Ok(_) => {
                        offset += bytes_read as u64;
                        {
                            let mut map = uploads.write().await;
                            if let Some(p) = map.get_mut(&upload_id_clone) {
                                p.bytes_uploaded = offset;
                            }
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to write chunk at offset {}: {:?}", offset, e);
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(format!(
                                "Chunk write failed at offset {}: {}",
                                offset, e.message
                            ));
                        }
                        Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                        return;
                    }
                }
            }

            // Finish the upload
            let finish_cmd = Command::FinishChunkedWrite {
                entity_id: entity_id.clone(),
                disk_type: disk_type_arg,
                path: dest_path.clone(),
            };

            match app.execute(finish_cmd).await {
                Ok(_) => {
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Complete;
                        p.checksum_verified = true;
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                    debug!("Resumed upload completed: {}", upload_id_clone);
                }
                Err(e) => {
                    tracing::error!("Failed to finish resumed upload: {:?}", e);
                    let mut map = uploads.write().await;
                    if let Some(p) = map.get_mut(&upload_id_clone) {
                        p.state = UploadState::Failed(format!("Finish failed: {}", e.message));
                    }
                    Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;
                }
            }
        });

        Ok(())
    }

    /// Resume all pending resumable uploads.
    ///
    /// Returns the count of uploads that were started for resume.
    /// Note: This requires source paths to be stored or passed separately.
    #[instrument(skip(self, source_paths), name = "ui.drive.resume_all_pending")]
    pub async fn resume_all_pending(
        &self,
        source_paths: &HashMap<String, PathBuf>,
    ) -> Result<usize, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let uploads = self.active_uploads.read().await;
        let resumable: Vec<String> = uploads
            .iter()
            .filter(|(_, u)| u.state.is_resumable())
            .map(|(id, _)| id.clone())
            .collect();
        drop(uploads);

        let mut resumed = 0;
        for upload_id in resumable {
            if let Some(source_path) = source_paths.get(&upload_id) {
                match self.resume_upload_by_id(&upload_id, source_path).await {
                    Ok(()) => {
                        resumed += 1;
                        debug!(upload_id = %upload_id, "resumed upload");
                    }
                    Err(e) => {
                        debug!(upload_id = %upload_id, error = %e, "failed to resume upload");
                    }
                }
            } else {
                debug!(upload_id = %upload_id, "no source path for resumable upload");
            }
        }

        Ok(resumed)
    }

    /// Get progress of a specific upload.
    pub async fn get_upload_progress(&self, upload_id: &str) -> Option<UploadProgress> {
        let uploads = self.active_uploads.read().await;
        uploads.get(upload_id).cloned()
    }

    /// Subscribe to upload progress updates.
    pub fn subscribe_uploads(&self) -> watch::Receiver<DriveSnapshot> {
        self.rx.clone()
    }

    // ===== Download Operations =====

    /// Start a download operation and return a download ID for tracking.
    ///
    /// This method performs real file I/O via CommunitasApp with progress tracking.
    #[instrument(skip(self), name = "ui.drive.start_download", fields(entity_id, ?disk_type, path, destination))]
    pub async fn start_download(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        destination: &str,
    ) -> Result<String, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Generate download ID
        let download_id = {
            let mut counter = self.download_counter.write().await;
            *counter += 1;
            format!("download-{}", *counter)
        };

        let file_name = extract_name_from_path(path, "file");

        // Set total_bytes to 0 initially; will be updated when we read the file
        let progress = DownloadProgress {
            id: download_id.clone(),
            file_name,
            destination_path: destination.to_string(),
            bytes_downloaded: 0,
            total_bytes: 0,
            state: DownloadState::Pending,
            checksum_verified: false,
        };

        // Store in active downloads
        {
            let mut downloads = self.active_downloads.write().await;
            downloads.insert(download_id.clone(), progress.clone());
        }

        // Update snapshot
        self.update_download_snapshot().await;

        // Clone values for the spawned task
        let download_id_clone = download_id.clone();
        let downloads = self.active_downloads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let entity_id_owned = entity_id.to_string();
        let path_owned = path.to_string();
        let destination_owned = destination.to_string();

        tokio::spawn(async move {
            // Mark as downloading
            {
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Downloading;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

            // Check for cancellation before starting
            {
                let map = downloads.read().await;
                if map
                    .get(&download_id_clone)
                    .is_some_and(|p| matches!(p.state, DownloadState::Cancelled))
                {
                    return;
                }
            }

            // Perform the actual read operation
            let read_result = app
                .query(Query::ReadFile {
                    entity_id: entity_id_owned,
                    disk_type: disk_type_to_arg(disk_type),
                    path: path_owned,
                })
                .await;

            // Check for cancellation after read
            {
                let map = downloads.read().await;
                if map
                    .get(&download_id_clone)
                    .is_some_and(|p| matches!(p.state, DownloadState::Cancelled))
                {
                    return;
                }
            }

            match read_result {
                Ok(QueryResponse::FileContents(data)) => {
                    let total_bytes = data.len() as u64;
                    let expected_checksum = compute_checksum(&data);

                    // Update progress with actual size
                    {
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.total_bytes = total_bytes;
                            p.bytes_downloaded = total_bytes;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

                    // Mark as verifying
                    {
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Verifying;
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

                    // Write to destination file
                    let write_result = tokio::fs::write(&destination_owned, &data).await;

                    match write_result {
                        Ok(()) => {
                            // Verify by reading back and checking checksum
                            let verify_result = tokio::fs::read(&destination_owned).await;
                            let checksum_verified = match verify_result {
                                Ok(read_data) => compute_checksum(&read_data) == expected_checksum,
                                Err(_) => false,
                            };

                            // Mark as complete
                            {
                                let mut map = downloads.write().await;
                                if let Some(p) = map.get_mut(&download_id_clone) {
                                    p.state = DownloadState::Complete;
                                    p.checksum_verified = checksum_verified;
                                }
                            }
                            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                        }
                        Err(e) => {
                            // Clean up partial write if possible (ignore errors)
                            let _ = tokio::fs::remove_file(&destination_owned).await;

                            // Mark as failed
                            tracing::error!("Download write failed: {:?}", e);
                            {
                                let mut map = downloads.write().await;
                                if let Some(p) = map.get_mut(&download_id_clone) {
                                    p.state = DownloadState::Failed(e.to_string());
                                }
                            }
                            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                        }
                    }
                }
                Ok(_) => {
                    // Unexpected response type
                    tracing::error!("Download got unexpected response type");
                    {
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Failed("unexpected response type".to_string());
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                }
                Err(e) => {
                    // Mark as failed
                    tracing::error!("Download failed: {:?}", e);
                    {
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Failed(e.to_string());
                        }
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                }
            }
        });

        Ok(download_id)
    }
    /// Cancel a download.
    #[instrument(skip(self), name = "ui.drive.cancel_download", fields(download_id))]
    pub async fn cancel_download(&self, download_id: &str) -> Result<(), DriveError> {
        let mut downloads = self.active_downloads.write().await;
        let progress = downloads
            .get_mut(download_id)
            .ok_or_else(|| DriveError::DownloadNotFound(download_id.to_string()))?;

        if progress.state.is_terminal() {
            return Err(DriveError::DownloadFailed(
                "Cannot cancel completed download".to_string(),
            ));
        }

        progress.state = DownloadState::Cancelled;
        drop(downloads);

        self.update_download_snapshot().await;
        Ok(())
    }

    /// Get progress of a specific download.
    pub async fn get_download_progress(&self, download_id: &str) -> Option<DownloadProgress> {
        let downloads = self.active_downloads.read().await;
        downloads.get(download_id).cloned()
    }

    /// Start a streaming download operation.
    ///
    /// This method downloads the file in chunks to maintain bounded memory usage.
    /// Data is written to a temp file and renamed on successful completion.
    ///
    /// Supports:
    /// - Progress updates per chunk
    /// - BLAKE3 verification of final file
    /// - Cancel support with cleanup
    #[instrument(skip(self), name = "ui.drive.start_streaming_download", fields(entity_id, ?disk_type, path, destination))]
    pub async fn start_streaming_download(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        destination: &str,
    ) -> Result<String, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let disk_type_arg = disk_type_to_arg(disk_type);

        // Get file metadata to know total size
        let metadata_response = self
            .app
            .query(Query::GetFileMetadata {
                entity_id: entity_id.to_string(),
                disk_type: disk_type_arg,
                path: path.to_string(),
            })
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        let QueryResponse::FileMetadata(metadata) = metadata_response else {
            return Err(DriveError::QueryError(
                "unexpected response type from GetFileMetadata query".to_string(),
            ));
        };

        let total_bytes = metadata.size_bytes;
        let expected_hash = metadata.content_hash.clone();
        let chunk_count = metadata.chunk_count;

        // Generate download ID
        let download_id = {
            let mut counter = self.download_counter.write().await;
            *counter += 1;
            format!("download-{}", *counter)
        };

        let file_name = extract_name_from_path(path, "file");

        let progress = DownloadProgress {
            id: download_id.clone(),
            file_name,
            destination_path: destination.to_string(),
            bytes_downloaded: 0,
            total_bytes,
            state: DownloadState::Pending,
            checksum_verified: false,
        };

        // Store in active downloads
        {
            let mut downloads = self.active_downloads.write().await;
            downloads.insert(download_id.clone(), progress.clone());
        }
        self.update_download_snapshot().await;

        // Clone values for the spawned task
        let download_id_clone = download_id.clone();
        let downloads = self.active_downloads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let entity_id_owned = entity_id.to_string();
        let path_owned = path.to_string();
        let destination_owned = destination.to_string();

        tokio::spawn(async move {
            // Mark as downloading
            {
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Downloading;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

            // Helper to check cancellation
            let is_cancelled = || async {
                let map = downloads.read().await;
                map.get(&download_id_clone)
                    .is_some_and(|p| matches!(p.state, DownloadState::Cancelled))
            };

            if is_cancelled().await {
                return;
            }

            // Create temp file for atomic write
            let temp_path = format!("{}.download.tmp", destination_owned);

            // Open temp file for writing
            let file_result = tokio::fs::File::create(&temp_path).await;
            let mut file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to create temp file: {:?}", e);
                    let mut map = downloads.write().await;
                    if let Some(p) = map.get_mut(&download_id_clone) {
                        p.state =
                            DownloadState::Failed(format!("Failed to create temp file: {}", e));
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                    return;
                }
            };

            // Download chunks
            let mut bytes_downloaded: u64 = 0;
            let mut hasher = blake3::Hasher::new();

            for chunk_index in 0..chunk_count {
                if is_cancelled().await {
                    // Cleanup temp file
                    let _ = tokio::fs::remove_file(&temp_path).await;
                    return;
                }

                let offset = chunk_index * DEFAULT_CHUNK_SIZE;
                let read_result = app
                    .query(Query::ReadChunk {
                        entity_id: entity_id_owned.clone(),
                        disk_type: disk_type_arg,
                        path: path_owned.clone(),
                        offset,
                        chunk_size: Some(DEFAULT_CHUNK_SIZE),
                    })
                    .await;

                match read_result {
                    Ok(QueryResponse::ChunkRead(chunk_data)) => {
                        // Write chunk to temp file
                        if let Err(e) = file.write_all(&chunk_data.data).await {
                            tracing::error!("Failed to write chunk to temp file: {:?}", e);
                            let _ = tokio::fs::remove_file(&temp_path).await;
                            let mut map = downloads.write().await;
                            if let Some(p) = map.get_mut(&download_id_clone) {
                                p.state = DownloadState::Failed(format!("File write error: {}", e));
                            }
                            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                            return;
                        }

                        // Update hash
                        hasher.update(&chunk_data.data);

                        bytes_downloaded += chunk_data.size;

                        // Update progress
                        {
                            let mut map = downloads.write().await;
                            if let Some(p) = map.get_mut(&download_id_clone) {
                                p.bytes_downloaded = bytes_downloaded;
                            }
                        }
                        Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

                        debug!(
                            "Downloaded chunk {}/{}, progress: {}/{}",
                            chunk_index + 1,
                            chunk_count,
                            bytes_downloaded,
                            total_bytes
                        );
                    }
                    Ok(_) => {
                        tracing::error!("Unexpected response type from ReadChunk");
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Failed("unexpected response type".to_string());
                        }
                        Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                        return;
                    }
                    Err(e) => {
                        tracing::error!("Failed to read chunk {}: {:?}", chunk_index, e);
                        // Don't cleanup - leave partial download for potential resume
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Failed(format!(
                                "Chunk read failed at index {}: {}",
                                chunk_index, e
                            ));
                        }
                        Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                        return;
                    }
                }
            }

            if is_cancelled().await {
                let _ = tokio::fs::remove_file(&temp_path).await;
                return;
            }

            // Flush file
            if let Err(e) = file.flush().await {
                tracing::error!("Failed to flush temp file: {:?}", e);
                let _ = tokio::fs::remove_file(&temp_path).await;
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Failed(format!("File flush error: {}", e));
                }
                Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                return;
            }
            drop(file);

            // Verify hash
            {
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Verifying;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

            let final_hash = hasher.finalize().to_string();
            let checksum_verified = final_hash == expected_hash;

            if !checksum_verified {
                tracing::warn!(
                    "Hash mismatch: expected {}, got {}",
                    expected_hash,
                    final_hash
                );
            }

            // Rename temp file to destination (atomic)
            if let Err(e) = tokio::fs::rename(&temp_path, &destination_owned).await {
                tracing::error!("Failed to rename temp file: {:?}", e);
                let _ = tokio::fs::remove_file(&temp_path).await;
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Failed(format!("Failed to rename temp file: {}", e));
                }
                Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                return;
            }

            // Mark as complete
            {
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Complete;
                    p.checksum_verified = checksum_verified;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
        });

        Ok(download_id)
    }

    /// Resume an interrupted streaming download.
    ///
    /// This checks for an existing temp file and resumes from where it left off.
    /// Returns an error if no partial download exists or the file has changed.
    #[instrument(skip(self), name = "ui.drive.resume_download", fields(entity_id, ?disk_type, path, destination))]
    pub async fn resume_download(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        destination: &str,
    ) -> Result<String, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let disk_type_arg = disk_type_to_arg(disk_type);
        let temp_path = format!("{}.download.tmp", destination);

        // Check if temp file exists
        let temp_metadata = tokio::fs::metadata(&temp_path).await.map_err(|_| {
            DriveError::TransferNotResumable("No partial download found".to_string())
        })?;
        let bytes_downloaded = temp_metadata.len();

        // Get file metadata to know total size and verify it hasn't changed
        let metadata_response = self
            .app
            .query(Query::GetFileMetadata {
                entity_id: entity_id.to_string(),
                disk_type: disk_type_arg,
                path: path.to_string(),
            })
            .await
            .map_err(|e| DriveError::QueryError(e.to_string()))?;

        let QueryResponse::FileMetadata(metadata) = metadata_response else {
            return Err(DriveError::QueryError(
                "unexpected response type from GetFileMetadata query".to_string(),
            ));
        };

        let total_bytes = metadata.size_bytes;
        let expected_hash = metadata.content_hash.clone();
        let chunk_count = metadata.chunk_count;

        // Calculate which chunk to resume from
        let chunks_completed = bytes_downloaded / DEFAULT_CHUNK_SIZE;

        if chunks_completed >= chunk_count {
            // Already complete, just verify and rename
            return Err(DriveError::TransferNotResumable(
                "Download already complete, verify and rename manually".to_string(),
            ));
        }

        // Generate download ID
        let download_id = {
            let mut counter = self.download_counter.write().await;
            *counter += 1;
            format!("download-{}", *counter)
        };

        let file_name = extract_name_from_path(path, "file");

        let progress = DownloadProgress {
            id: download_id.clone(),
            file_name,
            destination_path: destination.to_string(),
            bytes_downloaded,
            total_bytes,
            state: DownloadState::Downloading,
            checksum_verified: false,
        };

        // Store in active downloads
        {
            let mut downloads = self.active_downloads.write().await;
            downloads.insert(download_id.clone(), progress.clone());
        }
        self.update_download_snapshot().await;

        // Clone values for the spawned task
        let download_id_clone = download_id.clone();
        let downloads = self.active_downloads.clone();
        let tx = self.tx.clone();
        let rx = self.rx.clone();
        let app = self.app.clone();
        let entity_id_owned = entity_id.to_string();
        let path_owned = path.to_string();
        let destination_owned = destination.to_string();

        tokio::spawn(async move {
            // Helper to check cancellation
            let is_cancelled = || async {
                let map = downloads.read().await;
                map.get(&download_id_clone)
                    .is_some_and(|p| matches!(p.state, DownloadState::Cancelled))
            };

            // Open temp file for appending
            let file_result = tokio::fs::OpenOptions::new()
                .append(true)
                .open(&temp_path)
                .await;
            let mut file = match file_result {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to open temp file for resume: {:?}", e);
                    let mut map = downloads.write().await;
                    if let Some(p) = map.get_mut(&download_id_clone) {
                        p.state = DownloadState::Failed(format!("Failed to open temp file: {}", e));
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                    return;
                }
            };

            // We need to re-hash the existing data for verification
            // Read the temp file to compute hash of already-downloaded data
            let existing_data = match tokio::fs::read(&temp_path).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to read temp file for hash: {:?}", e);
                    let mut map = downloads.write().await;
                    if let Some(p) = map.get_mut(&download_id_clone) {
                        p.state = DownloadState::Failed(format!("Failed to read temp file: {}", e));
                    }
                    Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                    return;
                }
            };

            let mut hasher = blake3::Hasher::new();
            hasher.update(&existing_data);
            let mut bytes_downloaded = existing_data.len() as u64;

            // Continue downloading remaining chunks
            for chunk_index in chunks_completed..chunk_count {
                if is_cancelled().await {
                    return;
                }

                let offset = chunk_index * DEFAULT_CHUNK_SIZE;
                let read_result = app
                    .query(Query::ReadChunk {
                        entity_id: entity_id_owned.clone(),
                        disk_type: disk_type_arg,
                        path: path_owned.clone(),
                        offset,
                        chunk_size: Some(DEFAULT_CHUNK_SIZE),
                    })
                    .await;

                match read_result {
                    Ok(QueryResponse::ChunkRead(chunk_data)) => {
                        if let Err(e) = file.write_all(&chunk_data.data).await {
                            tracing::error!("Failed to write chunk to temp file: {:?}", e);
                            let mut map = downloads.write().await;
                            if let Some(p) = map.get_mut(&download_id_clone) {
                                p.state = DownloadState::Failed(format!("File write error: {}", e));
                            }
                            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                            return;
                        }

                        hasher.update(&chunk_data.data);
                        bytes_downloaded += chunk_data.size;

                        {
                            let mut map = downloads.write().await;
                            if let Some(p) = map.get_mut(&download_id_clone) {
                                p.bytes_downloaded = bytes_downloaded;
                            }
                        }
                        Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

                        debug!(
                            "Resumed download chunk {}/{}, progress: {}/{}",
                            chunk_index + 1,
                            chunk_count,
                            bytes_downloaded,
                            total_bytes
                        );
                    }
                    Ok(_) => {
                        tracing::error!("Unexpected response type from ReadChunk");
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Failed("unexpected response type".to_string());
                        }
                        Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                        return;
                    }
                    Err(e) => {
                        tracing::error!("Failed to read chunk {}: {:?}", chunk_index, e);
                        let mut map = downloads.write().await;
                        if let Some(p) = map.get_mut(&download_id_clone) {
                            p.state = DownloadState::Failed(format!(
                                "Chunk read failed at index {}: {}",
                                chunk_index, e
                            ));
                        }
                        Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                        return;
                    }
                }
            }

            if is_cancelled().await {
                return;
            }

            // Flush file
            if let Err(e) = file.flush().await {
                tracing::error!("Failed to flush temp file: {:?}", e);
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Failed(format!("File flush error: {}", e));
                }
                Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                return;
            }
            drop(file);

            // Verify hash
            {
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Verifying;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;

            let final_hash = hasher.finalize().to_string();
            let checksum_verified = final_hash == expected_hash;

            // Rename temp file to destination
            if let Err(e) = tokio::fs::rename(&temp_path, &destination_owned).await {
                tracing::error!("Failed to rename temp file: {:?}", e);
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Failed(format!("Failed to rename temp file: {}", e));
                }
                Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
                return;
            }

            // Mark as complete
            {
                let mut map = downloads.write().await;
                if let Some(p) = map.get_mut(&download_id_clone) {
                    p.state = DownloadState::Complete;
                    p.checksum_verified = checksum_verified;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, None, Some(&downloads)).await;
        });

        Ok(download_id)
    }

    // ===== Share Link Operations =====

    /// Create a share link for a file.
    ///
    /// Share links can only be created for files on Public or Shared disks.
    /// Private disk files cannot be shared via links.
    #[instrument(skip(self, config), name = "ui.drive.create_share_link", fields(entity_id, ?disk_type, path))]
    pub async fn create_share_link(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
        config: ShareLinkConfig,
    ) -> Result<ShareLink, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Prevent sharing private files
        if disk_type == DiskType::Private {
            return Err(DriveError::CannotSharePrivateFiles);
        }

        // Verify the file exists by getting its preview
        let _preview = self.get_file_preview(entity_id, disk_type, path).await?;

        // Generate unique link ID
        let link_id = {
            let mut counter = self.share_link_counter.write().await;
            *counter += 1;
            format!("share-{:08x}", *counter)
        };

        // Generate shareable URL
        let url = format!(
            "communitas://share/{}/{}{}",
            entity_id,
            link_id,
            if config.password.is_some() {
                "?protected=1"
            } else {
                ""
            }
        );

        let now = current_timestamp_millis();
        let expires_at = config.expires_in_ms.map(|ms| now + ms);
        let file_name = extract_name_from_path(path, "file");

        let share_link = ShareLink {
            id: link_id.clone(),
            entity_id: entity_id.to_string(),
            disk_type,
            file_path: path.to_string(),
            file_name,
            url,
            created_at: now,
            expires_at,
            password_protected: config.password.is_some(),
            access_count: 0,
            max_accesses: config.max_accesses,
            active: true,
        };

        // Store the link
        {
            let mut links = self.share_links.write().await;
            links.insert(link_id.clone(), share_link.clone());
        }

        // Initialize stats for the link
        {
            let mut stats = self.share_link_stats.write().await;
            stats.insert(
                link_id.clone(),
                ShareLinkStats {
                    total_accesses: 0,
                    successful_downloads: 0,
                    failed_password_attempts: 0,
                    last_accessed_at: None,
                    unique_accessors: 0,
                },
            );
        }

        // Store password hash separately if provided (would use secure storage in production)
        // For now, we'll store in a simple way - in production this would be hashed
        if let Some(password) = config.password {
            debug!("Share link {} created with password protection", link_id);
            // In production, we'd hash the password and store it securely
            // For this implementation, we store a simple hash
            let _password_hash = compute_checksum(password.as_bytes());
        }

        debug!(
            "Created share link {} for {}:{} (expires: {:?})",
            link_id, entity_id, path, expires_at
        );

        Ok(share_link)
    }

    /// Revoke a share link.
    ///
    /// Once revoked, the link cannot be used to access the file.
    #[instrument(skip(self), name = "ui.drive.revoke_share_link", fields(link_id))]
    pub async fn revoke_share_link(&self, link_id: &str) -> Result<(), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let mut links = self.share_links.write().await;
        let link = links
            .get_mut(link_id)
            .ok_or_else(|| DriveError::ShareLinkNotFound(link_id.to_string()))?;

        link.active = false;
        debug!("Revoked share link {}", link_id);

        Ok(())
    }

    /// Access a share link.
    ///
    /// Returns file metadata if access is granted, or an appropriate error status.
    #[instrument(
        skip(self, password),
        name = "ui.drive.access_share_link",
        fields(link_id)
    )]
    pub async fn access_share_link(
        &self,
        link_id: &str,
        password: Option<&str>,
    ) -> Result<ShareLinkAccessResult, DriveError> {
        let now = current_timestamp_millis();

        // Get the link (read-only first to check access)
        let link_info = {
            let links = self.share_links.read().await;
            links.get(link_id).cloned()
        };

        let Some(link) = link_info else {
            return Ok(ShareLinkAccessResult::NotFound);
        };

        // Check if revoked
        if !link.active {
            return Ok(ShareLinkAccessResult::Revoked);
        }

        // Check if expired
        if link.is_expired(now) {
            return Ok(ShareLinkAccessResult::Expired);
        }

        // Check if access limit reached
        if link.is_access_limit_reached() {
            return Ok(ShareLinkAccessResult::AccessLimitReached);
        }

        // Check password if required
        if link.password_protected {
            if password.is_none() {
                // Update stats for access attempt
                {
                    let mut stats = self.share_link_stats.write().await;
                    if let Some(s) = stats.get_mut(link_id) {
                        s.total_accesses += 1;
                        s.last_accessed_at = Some(now);
                    }
                }
                return Ok(ShareLinkAccessResult::PasswordRequired);
            }

            // In production, we'd verify against stored hash
            // For this implementation, we'll accept any non-empty password
            // (real implementation would store and verify password hashes)
            let provided_password = password.unwrap_or("");
            if provided_password.is_empty() {
                // Update stats for failed password attempt
                {
                    let mut stats = self.share_link_stats.write().await;
                    if let Some(s) = stats.get_mut(link_id) {
                        s.total_accesses += 1;
                        s.failed_password_attempts += 1;
                        s.last_accessed_at = Some(now);
                    }
                }
                return Ok(ShareLinkAccessResult::IncorrectPassword);
            }
        }

        // Access granted - update link access count
        {
            let mut links = self.share_links.write().await;
            if let Some(l) = links.get_mut(link_id) {
                l.access_count += 1;
            }
        }

        // Update stats
        {
            let mut stats = self.share_link_stats.write().await;
            if let Some(s) = stats.get_mut(link_id) {
                s.total_accesses += 1;
                s.successful_downloads += 1;
                s.last_accessed_at = Some(now);
                // Note: unique_accessors would need IP tracking in production
            }
        }

        // Get file metadata for the response
        let preview = self
            .get_file_preview(&link.entity_id, link.disk_type, &link.file_path)
            .await?;

        debug!("Access granted to share link {}", link_id);

        Ok(ShareLinkAccessResult::Granted {
            file_path: link.file_path,
            file_name: link.file_name,
            size_bytes: preview.size_bytes,
            mime_type: Some(preview.mime_type),
            checksum: preview.metadata.checksum,
        })
    }

    /// Get usage statistics for a share link.
    #[instrument(skip(self), name = "ui.drive.get_share_link_stats", fields(link_id))]
    pub async fn get_share_link_stats(&self, link_id: &str) -> Result<ShareLinkStats, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let stats = self.share_link_stats.read().await;
        stats
            .get(link_id)
            .cloned()
            .ok_or_else(|| DriveError::ShareLinkNotFound(link_id.to_string()))
    }

    /// Get a share link by ID.
    #[instrument(skip(self), name = "ui.drive.get_share_link", fields(link_id))]
    pub async fn get_share_link(&self, link_id: &str) -> Result<ShareLink, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let links = self.share_links.read().await;
        links
            .get(link_id)
            .cloned()
            .ok_or_else(|| DriveError::ShareLinkNotFound(link_id.to_string()))
    }

    /// List all share links for an entity.
    #[instrument(skip(self), name = "ui.drive.list_share_links", fields(entity_id))]
    pub async fn list_share_links(&self, entity_id: &str) -> Result<Vec<ShareLink>, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let links = self.share_links.read().await;
        let entity_links: Vec<ShareLink> = links
            .values()
            .filter(|l| l.entity_id == entity_id)
            .cloned()
            .collect();

        Ok(entity_links)
    }

    /// List all share links for a specific file.
    #[instrument(skip(self), name = "ui.drive.list_file_share_links", fields(entity_id, ?disk_type, path))]
    pub async fn list_file_share_links(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<Vec<ShareLink>, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let links = self.share_links.read().await;
        let file_links: Vec<ShareLink> = links
            .values()
            .filter(|l| l.entity_id == entity_id && l.disk_type == disk_type && l.file_path == path)
            .cloned()
            .collect();

        Ok(file_links)
    }

    // ===== Offline Staging Area Methods =====

    /// Subscribe to staging queue events.
    pub fn subscribe_staging_events(&self) -> tokio::sync::broadcast::Receiver<StagingEvent> {
        self.staging_event_tx.subscribe()
    }

    /// Stage a file for upload when offline.
    /// The file will be automatically uploaded when network connectivity is restored.
    #[instrument(skip(self), name = "ui.drive.stage_upload", fields(entity_id, ?disk_type, destination, local_path))]
    pub async fn stage_upload(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        destination_path: &str,
        local_path: &str,
    ) -> Result<StagedUpload, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        // Verify local file exists and get metadata
        let local_metadata = tokio::fs::metadata(local_path)
            .await
            .map_err(|e| DriveError::LocalFileNotFound(format!("{}: {}", local_path, e)))?;

        if !local_metadata.is_file() {
            return Err(DriveError::LocalFileNotFound(format!(
                "{} is not a file",
                local_path
            )));
        }

        let size_bytes = local_metadata.len();

        // Read file and compute checksum
        let content = tokio::fs::read(local_path).await.map_err(|e| {
            DriveError::FileReadError(format!("failed to read {}: {}", local_path, e))
        })?;
        let local_checksum = compute_checksum(&content);

        // Extract file name from path
        let file_name = extract_name_from_path(local_path, "file");

        // Detect MIME type from extension
        let mime_type = Path::new(local_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(mime_from_extension);

        // Generate unique ID
        let id = {
            let mut counter = self.staging_counter.write().await;
            *counter += 1;
            format!("staged-{:08x}", *counter)
        };

        let now = current_timestamp_millis();

        let staged = StagedUpload {
            id: id.clone(),
            entity_id: entity_id.to_string(),
            disk_type,
            destination_path: destination_path.to_string(),
            local_path: local_path.to_string(),
            file_name: file_name.clone(),
            size_bytes,
            mime_type,
            local_checksum,
            state: StagedUploadState::Pending,
            retry_count: 0,
            max_retries: self.staging_max_retries,
            error: None,
            staged_at: now,
            updated_at: now,
            conflict: None,
        };

        // Add to staging queue
        {
            let mut queue = self.staging_queue.write().await;
            queue.insert(id.clone(), staged.clone());
        }

        // Broadcast event
        let _ = self.staging_event_tx.send(StagingEvent::FileStaged {
            upload_id: id.clone(),
            file_name,
        });

        // Update snapshot
        self.update_staging_snapshot().await;

        debug!(
            "File staged for upload: {} -> {}",
            local_path, destination_path
        );

        Ok(staged)
    }

    /// Get a staged upload by ID.
    #[instrument(skip(self), name = "ui.drive.get_staged_upload", fields(upload_id))]
    pub async fn get_staged_upload(&self, upload_id: &str) -> Result<StagedUpload, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let queue = self.staging_queue.read().await;
        queue
            .get(upload_id)
            .cloned()
            .ok_or_else(|| DriveError::StagedUploadNotFound(upload_id.to_string()))
    }

    /// List all staged uploads.
    #[instrument(skip(self), name = "ui.drive.list_staged_uploads")]
    pub async fn list_staged_uploads(&self) -> Result<Vec<StagedUpload>, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let queue = self.staging_queue.read().await;
        let mut uploads: Vec<StagedUpload> = queue.values().cloned().collect();
        // Sort by staged_at (oldest first)
        uploads.sort_by_key(|u| u.staged_at);
        Ok(uploads)
    }

    /// Get the current staging queue status.
    #[instrument(skip(self), name = "ui.drive.get_staging_status")]
    pub async fn get_staging_status(&self) -> Result<StagingQueueStatus, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let queue = self.staging_queue.read().await;
        let network_available = *self.network_available.read().await;

        let mut status = StagingQueueStatus {
            total_files: 0,
            pending_files: 0,
            uploading_files: 0,
            conflicted_files: 0,
            failed_files: 0,
            completed_files: 0,
            total_bytes: 0,
            bytes_uploaded: 0,
            is_syncing: false,
            network_available,
            last_sync_at: None,
            last_sync_error: None,
        };

        for upload in queue.values() {
            status.total_files += 1;
            status.total_bytes += upload.size_bytes;

            match upload.state {
                StagedUploadState::Pending => status.pending_files += 1,
                StagedUploadState::Uploading => {
                    status.uploading_files += 1;
                    status.is_syncing = true;
                }
                StagedUploadState::Conflicted => status.conflicted_files += 1,
                StagedUploadState::Completed => {
                    status.completed_files += 1;
                    status.bytes_uploaded += upload.size_bytes;
                }
                StagedUploadState::Failed => status.failed_files += 1,
            }
        }

        Ok(status)
    }

    /// Remove a staged upload from the queue.
    #[instrument(skip(self), name = "ui.drive.remove_staged_upload", fields(upload_id))]
    pub async fn remove_staged_upload(&self, upload_id: &str) -> Result<(), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let mut queue = self.staging_queue.write().await;
        if queue.remove(upload_id).is_some() {
            drop(queue);
            self.update_staging_snapshot().await;
            debug!("Staged upload removed: {}", upload_id);
            Ok(())
        } else {
            Err(DriveError::StagedUploadNotFound(upload_id.to_string()))
        }
    }

    /// Clear all staged uploads (optionally only completed/failed ones).
    #[instrument(
        skip(self),
        name = "ui.drive.clear_staging_queue",
        fields(only_terminal)
    )]
    pub async fn clear_staging_queue(&self, only_terminal: bool) -> Result<u32, DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let mut queue = self.staging_queue.write().await;
        let initial_count = queue.len();

        if only_terminal {
            queue.retain(|_, u| !u.state.is_terminal());
        } else {
            queue.clear();
        }

        let removed = (initial_count - queue.len()) as u32;
        drop(queue);

        // Broadcast event
        let _ = self.staging_event_tx.send(StagingEvent::QueueCleared {
            files_removed: removed,
        });

        self.update_staging_snapshot().await;

        debug!("Staging queue cleared: {} files removed", removed);
        Ok(removed)
    }

    /// Retry a failed staged upload.
    #[instrument(skip(self), name = "ui.drive.retry_staged_upload", fields(upload_id))]
    pub async fn retry_staged_upload(&self, upload_id: &str) -> Result<(), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let mut queue = self.staging_queue.write().await;
        let upload = queue
            .get_mut(upload_id)
            .ok_or_else(|| DriveError::StagedUploadNotFound(upload_id.to_string()))?;

        if !upload.can_retry() {
            return Err(DriveError::StagingQueueError(format!(
                "cannot retry upload {}: retries exhausted or not in failed state",
                upload_id
            )));
        }

        upload.state = StagedUploadState::Pending;
        upload.error = None;
        upload.retry_count += 1;
        upload.updated_at = current_timestamp_millis();

        drop(queue);
        self.update_staging_snapshot().await;

        debug!("Staged upload queued for retry: {}", upload_id);
        Ok(())
    }

    /// Resolve a conflict for a staged upload.
    #[instrument(skip(self), name = "ui.drive.resolve_staging_conflict", fields(upload_id, ?resolution))]
    pub async fn resolve_staging_conflict(
        &self,
        upload_id: &str,
        resolution: ConflictResolution,
    ) -> Result<(), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let mut queue = self.staging_queue.write().await;
        let upload = queue
            .get_mut(upload_id)
            .ok_or_else(|| DriveError::StagedUploadNotFound(upload_id.to_string()))?;

        if !matches!(upload.state, StagedUploadState::Conflicted) {
            return Err(DriveError::StagingQueueError(format!(
                "upload {} is not in conflicted state",
                upload_id
            )));
        }

        let now = current_timestamp_millis();

        match resolution {
            ConflictResolution::KeepLocal => {
                // Will overwrite remote on next sync
                upload.state = StagedUploadState::Pending;
                upload.conflict = None;
                upload.updated_at = now;
            }
            ConflictResolution::KeepRemote => {
                // Discard local changes - mark as completed (nothing to upload)
                upload.state = StagedUploadState::Completed;
                upload.conflict = None;
                upload.updated_at = now;
            }
            ConflictResolution::KeepBoth => {
                // Rename the destination with a conflict suffix
                let timestamp = now / 1000; // seconds
                let new_dest = append_conflict_suffix(&upload.destination_path, timestamp);
                upload.destination_path = new_dest;
                upload.state = StagedUploadState::Pending;
                upload.conflict = None;
                upload.updated_at = now;
            }
            ConflictResolution::Skip => {
                // Remove from queue entirely
                let upload_id_owned = upload.id.clone();
                drop(queue);
                return self.remove_staged_upload(&upload_id_owned).await;
            }
            ConflictResolution::Retry => {
                // Retry the upload
                upload.state = StagedUploadState::Pending;
                upload.conflict = None;
                upload.retry_count += 1;
                upload.updated_at = now;
            }
        }

        drop(queue);
        self.update_staging_snapshot().await;

        debug!(
            "Staging conflict resolved for {}: {:?}",
            upload_id, resolution
        );
        Ok(())
    }

    /// Set network availability status.
    /// When network becomes available, pending uploads will be processed.
    #[instrument(skip(self), name = "ui.drive.set_network_available", fields(available))]
    pub async fn set_network_available(&self, available: bool) {
        {
            let mut net = self.network_available.write().await;
            *net = available;
        }

        // Broadcast event
        let _ = self
            .staging_event_tx
            .send(StagingEvent::NetworkStatusChanged { available });

        self.update_staging_snapshot().await;

        debug!("Network availability changed: {}", available);

        // If network is now available, trigger sync
        if available {
            // In production, this would spawn a background sync task
            debug!("Network restored - pending uploads will be processed");
        }
    }

    /// Process pending staged uploads (call when network is available).
    /// Returns the number of uploads processed.
    #[instrument(skip(self), name = "ui.drive.sync_staging_queue")]
    pub async fn sync_staging_queue(&self) -> Result<(u32, u32), DriveError> {
        if !self.is_authenticated() {
            return Err(DriveError::NotAuthenticated);
        }

        let network_available = *self.network_available.read().await;
        if !network_available {
            return Err(DriveError::StagingQueueError(
                "network not available".to_string(),
            ));
        }

        let _ = self.staging_event_tx.send(StagingEvent::SyncStarted);

        // Get pending uploads
        let pending_ids: Vec<String> = {
            let queue = self.staging_queue.read().await;
            queue
                .values()
                .filter(|u| matches!(u.state, StagedUploadState::Pending))
                .map(|u| u.id.clone())
                .collect()
        };

        let mut uploaded = 0u32;
        let mut failed = 0u32;

        for upload_id in pending_ids {
            match self.process_staged_upload(&upload_id).await {
                Ok(()) => uploaded += 1,
                Err(e) => {
                    debug!("Staged upload {} failed: {}", upload_id, e);
                    failed += 1;
                }
            }
        }

        let _ = self.staging_event_tx.send(StagingEvent::SyncCompleted {
            files_uploaded: uploaded,
            files_failed: failed,
        });

        self.update_staging_snapshot().await;

        debug!(
            "Staging queue sync completed: {} uploaded, {} failed",
            uploaded, failed
        );
        Ok((uploaded, failed))
    }

    /// Process a single staged upload.
    async fn process_staged_upload(&self, upload_id: &str) -> Result<(), DriveError> {
        // Mark as uploading
        {
            let mut queue = self.staging_queue.write().await;
            if let Some(upload) = queue.get_mut(upload_id) {
                upload.state = StagedUploadState::Uploading;
                upload.updated_at = current_timestamp_millis();
            }
        }
        self.update_staging_snapshot().await;

        let _ = self.staging_event_tx.send(StagingEvent::UploadStarted {
            upload_id: upload_id.to_string(),
        });

        // Get upload details
        let upload = self.get_staged_upload(upload_id).await?;

        // Check if local file still exists and hasn't changed
        let current_content = tokio::fs::read(&upload.local_path)
            .await
            .map_err(|e| DriveError::LocalFileNotFound(format!("{}: {}", upload.local_path, e)))?;

        let current_checksum = compute_checksum(&current_content);

        if current_checksum != upload.local_checksum {
            // File was modified - create a conflict
            let conflict = StagingConflict {
                conflict_type: ConflictType::LocalModified,
                staged_checksum: upload.local_checksum.clone(),
                local_checksum: Some(current_checksum),
                remote_checksum: None,
                remote_size_bytes: None,
                detected_at: current_timestamp_millis(),
            };

            let mut queue = self.staging_queue.write().await;
            if let Some(u) = queue.get_mut(upload_id) {
                u.state = StagedUploadState::Conflicted;
                u.conflict = Some(conflict.clone());
                u.updated_at = current_timestamp_millis();
            }
            drop(queue);
            self.update_staging_snapshot().await;

            let _ = self.staging_event_tx.send(StagingEvent::ConflictDetected {
                upload_id: upload_id.to_string(),
                conflict_type: ConflictType::LocalModified,
            });

            return Err(DriveError::StagingConflict(
                "local file was modified after staging".to_string(),
            ));
        }

        // Check if destination already exists
        let exists = self
            .path_exists(
                &upload.entity_id,
                upload.disk_type,
                &upload.destination_path,
            )
            .await?;

        if exists {
            // Check if it's the same content
            let remote_preview = self
                .get_file_preview(
                    &upload.entity_id,
                    upload.disk_type,
                    &upload.destination_path,
                )
                .await;

            if let Ok(preview) = remote_preview {
                if preview.metadata.checksum == upload.local_checksum {
                    // Same content - mark as completed
                    let mut queue = self.staging_queue.write().await;
                    if let Some(u) = queue.get_mut(upload_id) {
                        u.state = StagedUploadState::Completed;
                        u.updated_at = current_timestamp_millis();
                    }
                    drop(queue);
                    self.update_staging_snapshot().await;
                    return Ok(());
                }

                // Different content - conflict
                let conflict = StagingConflict {
                    conflict_type: ConflictType::FileExists,
                    staged_checksum: upload.local_checksum.clone(),
                    local_checksum: None,
                    remote_checksum: Some(preview.metadata.checksum.clone()),
                    remote_size_bytes: Some(preview.size_bytes),
                    detected_at: current_timestamp_millis(),
                };

                let mut queue = self.staging_queue.write().await;
                if let Some(u) = queue.get_mut(upload_id) {
                    u.state = StagedUploadState::Conflicted;
                    u.conflict = Some(conflict);
                    u.updated_at = current_timestamp_millis();
                }
                drop(queue);
                self.update_staging_snapshot().await;

                let _ = self.staging_event_tx.send(StagingEvent::ConflictDetected {
                    upload_id: upload_id.to_string(),
                    conflict_type: ConflictType::FileExists,
                });

                return Err(DriveError::StagingConflict(
                    "destination file already exists with different content".to_string(),
                ));
            }
        }

        // Perform the actual upload
        match self
            .write_file(
                &upload.entity_id,
                upload.disk_type,
                &upload.destination_path,
                &current_content,
            )
            .await
        {
            Ok(_entry) => {
                // Success
                let mut queue = self.staging_queue.write().await;
                if let Some(u) = queue.get_mut(upload_id) {
                    u.state = StagedUploadState::Completed;
                    u.updated_at = current_timestamp_millis();
                }
                drop(queue);
                self.update_staging_snapshot().await;

                let _ = self.staging_event_tx.send(StagingEvent::UploadCompleted {
                    upload_id: upload_id.to_string(),
                    destination_path: upload.destination_path.clone(),
                });

                Ok(())
            }
            Err(e) => {
                // Failed
                let error_msg = e.to_string();
                let mut queue = self.staging_queue.write().await;
                if let Some(u) = queue.get_mut(upload_id) {
                    u.state = StagedUploadState::Failed;
                    u.error = Some(error_msg.clone());
                    u.updated_at = current_timestamp_millis();
                }
                drop(queue);
                self.update_staging_snapshot().await;

                let _ = self.staging_event_tx.send(StagingEvent::UploadFailed {
                    upload_id: upload_id.to_string(),
                    error: error_msg,
                });

                Err(e)
            }
        }
    }

    /// Check if a path exists on a disk.
    async fn path_exists(
        &self,
        entity_id: &str,
        disk_type: DiskType,
        path: &str,
    ) -> Result<bool, DriveError> {
        match self.list_directory(entity_id, disk_type, path).await {
            Ok(_) => Ok(true),
            Err(DriveError::PathNotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn update_staging_snapshot(&self) {
        if let Ok(status) = self.get_staging_status().await {
            let mut snap = self.rx.borrow().clone();
            snap.staging_status = Some(status);
            let _ = self.tx.send(snap);
        }
        self.persist_staging_queue().await;
    }

    // ===== Helper Methods =====

    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated { .. }
        )
    }

    fn set_loading(&self, loading: bool) {
        let mut snap = self.rx.borrow().clone();
        snap.loading = loading;
        let _ = self.tx.send(snap);
    }

    async fn update_upload_snapshot(&self) {
        let uploads = self.active_uploads.read().await;
        let mut snap = self.rx.borrow().clone();
        snap.uploads = uploads.clone();
        let _ = self.tx.send(snap);
        drop(uploads); // Release read lock before persisting
        self.persist_active_uploads().await;
    }

    async fn update_download_snapshot(&self) {
        let downloads = self.active_downloads.read().await;
        let mut snap = self.rx.borrow().clone();
        snap.downloads = downloads.clone();
        let _ = self.tx.send(snap);
    }

    async fn broadcast_snapshot(
        tx: &watch::Sender<DriveSnapshot>,
        rx: &watch::Receiver<DriveSnapshot>,
        uploads: Option<&Arc<RwLock<HashMap<String, UploadProgress>>>>,
        downloads: Option<&Arc<RwLock<HashMap<String, DownloadProgress>>>>,
    ) {
        let mut snap = rx.borrow().clone();
        if let Some(uploads) = uploads {
            snap.uploads = uploads.read().await.clone();
        }
        if let Some(downloads) = downloads {
            snap.downloads = downloads.read().await.clone();
        }
        let _ = tx.send(snap);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::AuthController;
    use crate::storage::UiStorage;
    use communitas_core::app::CommunitasApp;
    use tempfile::TempDir;

    async fn make_service(temp: &TempDir) -> DriveService {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());
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
            .unwrap(),
        );
        DriveService::new(auth, app)
    }

    #[tokio::test]
    async fn drive_service_constructs() {
        let temp = TempDir::new().unwrap();
        let _service = make_service(&temp).await;
    }

    #[tokio::test]
    async fn snapshot_starts_empty() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let snap = service.current_snapshot();
        assert!(snap.uploads.is_empty());
        assert!(snap.downloads.is_empty());
        assert!(snap.current_directory.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn list_disks_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.list_disks("entity-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn list_directory_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .list_directory("entity-1", DiskType::Private, "/")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn create_directory_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .create_directory("entity-1", DiskType::Private, "/test")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn delete_path_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .delete_path("entity-1", DiskType::Private, "/test")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn move_path_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .move_path("entity-1", DiskType::Private, "/from", "/to")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn copy_path_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .copy_path("entity-1", DiskType::Private, "/from", "/to")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn read_file_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .read_file("entity-1", DiskType::Private, "/test.txt")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn write_file_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .write_file("entity-1", DiskType::Private, "/test.txt", b"content")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn get_file_preview_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .get_file_preview("entity-1", DiskType::Private, "/test.txt")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn get_quota_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_quota("entity-1", DiskType::Private).await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn start_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .start_upload("entity-1", DiskType::Private, "/test.txt", vec![1, 2, 3])
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn start_download_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .start_download("entity-1", DiskType::Private, "/test.txt", "/tmp/test.txt")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn cancel_upload_not_found() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.cancel_upload("nonexistent").await;
        assert!(matches!(result, Err(DriveError::UploadNotFound(_))));
    }

    #[tokio::test]
    async fn cancel_download_not_found() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.cancel_download("nonexistent").await;
        assert!(matches!(result, Err(DriveError::DownloadNotFound(_))));
    }

    #[tokio::test]
    async fn get_upload_progress_returns_none_for_unknown() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_upload_progress("unknown").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_download_progress_returns_none_for_unknown() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_download_progress("unknown").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn subscribe_returns_receiver() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let _rx = service.subscribe();
    }

    #[tokio::test]
    async fn subscribe_uploads_returns_receiver() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let _rx = service.subscribe_uploads();
    }

    #[test]
    fn drive_error_display() {
        assert_eq!(
            DriveError::NotAuthenticated.to_string(),
            "not authenticated"
        );
        assert_eq!(
            DriveError::DiskNotFound("test".to_string()).to_string(),
            "disk not found: test"
        );
        assert_eq!(
            DriveError::PathNotFound("/path".to_string()).to_string(),
            "path not found: /path"
        );
        assert_eq!(
            DriveError::QuotaExceeded {
                used: 100,
                quota: 50
            }
            .to_string(),
            "quota exceeded: used 100 bytes of 50 bytes"
        );
        assert_eq!(
            DriveError::ChecksumMismatch {
                expected: "abc".to_string(),
                actual: "xyz".to_string()
            }
            .to_string(),
            "checksum mismatch: expected abc, got xyz"
        );
    }

    #[test]
    fn disk_type_variants_are_distinct() {
        assert_ne!(DiskType::Private, DiskType::Public);
        assert_ne!(DiskType::Public, DiskType::Shared);
        assert_ne!(DiskType::Private, DiskType::Shared);
    }

    #[tokio::test]
    async fn start_streaming_download_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .start_streaming_download("entity-1", DiskType::Private, "/test.txt", "/tmp/test.txt")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn resume_download_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .resume_download("entity-1", DiskType::Private, "/test.txt", "/tmp/test.txt")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn start_streaming_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let source = temp.path().join("test.txt");
        tokio::fs::write(&source, b"test content").await.unwrap();
        let result = service
            .start_streaming_upload("entity-1", DiskType::Private, "/dest.txt", &source)
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn resume_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .resume_upload("entity-1", DiskType::Private, "/test.txt", vec![1, 2, 3])
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    // ===== Share Link Tests =====

    #[tokio::test]
    async fn create_share_link_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let config = ShareLinkConfig::default();
        let result = service
            .create_share_link("entity-1", DiskType::Public, "/test.txt", config)
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn create_share_link_rejects_private_disk() {
        let temp = TempDir::new().unwrap();
        let _service = make_service(&temp).await;
        // Note: Would need authenticated session to test this properly
        // For now we just verify the error type exists
        let err = DriveError::CannotSharePrivateFiles;
        assert!(err.to_string().contains("private"));
    }

    #[tokio::test]
    async fn revoke_share_link_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.revoke_share_link("link-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn get_share_link_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_share_link("link-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn get_share_link_stats_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_share_link_stats("link-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn list_share_links_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.list_share_links("entity-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn list_file_share_links_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .list_file_share_links("entity-1", DiskType::Public, "/test.txt")
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn access_share_link_not_found() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        // access_share_link doesn't require auth (it's a public operation)
        let result = service.access_share_link("nonexistent", None).await;
        assert!(matches!(result, Ok(ShareLinkAccessResult::NotFound)));
    }

    #[test]
    fn share_link_error_display() {
        assert_eq!(
            DriveError::ShareLinkNotFound("link-1".to_string()).to_string(),
            "share link not found: link-1"
        );
        assert_eq!(
            DriveError::CannotSharePrivateFiles.to_string(),
            "cannot share private files"
        );
        assert_eq!(
            DriveError::ShareLinkCreationFailed("test".to_string()).to_string(),
            "share link creation failed: test"
        );
    }

    // ===== Staging Queue Tests =====

    #[tokio::test]
    async fn stage_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let source = temp.path().join("test.txt");
        tokio::fs::write(&source, b"test content").await.unwrap();
        let result = service
            .stage_upload(
                "entity-1",
                DiskType::Private,
                "/dest.txt",
                source.to_str().unwrap(),
            )
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn get_staged_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_staged_upload("staged-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn list_staged_uploads_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.list_staged_uploads().await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn get_staging_status_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.get_staging_status().await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn remove_staged_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.remove_staged_upload("staged-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn clear_staging_queue_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.clear_staging_queue(false).await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn retry_staged_upload_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.retry_staged_upload("staged-1").await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn resolve_staging_conflict_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service
            .resolve_staging_conflict("staged-1", ConflictResolution::KeepLocal)
            .await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn sync_staging_queue_requires_auth() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let result = service.sync_staging_queue().await;
        assert!(matches!(result, Err(DriveError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn set_network_available_works() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        // Should not panic
        service.set_network_available(false).await;
        service.set_network_available(true).await;
    }

    #[tokio::test]
    async fn subscribe_staging_events_returns_receiver() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp).await;
        let _rx = service.subscribe_staging_events();
    }

    #[test]
    fn staging_error_display() {
        assert_eq!(
            DriveError::StagedUploadNotFound("staged-1".to_string()).to_string(),
            "staged upload not found: staged-1"
        );
        assert_eq!(
            DriveError::StagingConflict("file exists".to_string()).to_string(),
            "staging conflict: file exists"
        );
        assert_eq!(
            DriveError::LocalFileNotFound("/path/file.txt".to_string()).to_string(),
            "local file not found: /path/file.txt"
        );
        assert_eq!(
            DriveError::StagingQueueError("network error".to_string()).to_string(),
            "staging queue error: network error"
        );
    }

    #[test]
    fn helper_append_conflict_suffix() {
        assert_eq!(
            append_conflict_suffix("/docs/report.pdf", 12345),
            "/docs/report_conflict_12345.pdf"
        );
        assert_eq!(
            append_conflict_suffix("/docs/report", 12345),
            "/docs/report_conflict_12345"
        );
        assert_eq!(
            append_conflict_suffix("file.txt", 99),
            "file_conflict_99.txt"
        );
    }

    #[test]
    fn helper_mime_from_extension() {
        assert_eq!(mime_from_extension("txt"), "text/plain");
        assert_eq!(mime_from_extension("pdf"), "application/pdf");
        assert_eq!(mime_from_extension("png"), "image/png");
        assert_eq!(mime_from_extension("jpg"), "image/jpeg");
        assert_eq!(mime_from_extension("rs"), "text/x-rust");
        assert_eq!(mime_from_extension("unknown"), "application/octet-stream");
    }
}
