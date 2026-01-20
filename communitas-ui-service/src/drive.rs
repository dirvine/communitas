//! Drive service for virtual disk operations with progress tracking.
//!
//! Provides a UI-friendly abstraction over the storage system with:
//! - Virtual disk management (Private/Public/Shared per entity)
//! - Directory and file operations
//! - Upload/download progress tracking via watch channels
//! - Checksum verification

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use communitas_ui_api::drive::{
    DirectoryEntry, DiskInfo, DiskType, DownloadProgress, DownloadState, FileMetadata, FilePreview,
    QuotaInfo, UploadProgress, UploadState,
};
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::instrument;

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, DiskTypeArg, Event, Query, QueryResponse};

/// Get current timestamp in milliseconds since Unix epoch.
fn current_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

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
}

impl DriveService {
    /// Create a new drive service linked to the auth controller.
    pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        let (tx, rx) = watch::channel(DriveSnapshot::default());
        Self {
            auth,
            app,
            tx,
            rx,
            active_uploads: Arc::new(RwLock::new(HashMap::new())),
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            upload_counter: Arc::new(RwLock::new(0)),
            download_counter: Arc::new(RwLock::new(0)),
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

    /// Start an upload operation and return an upload ID for tracking.
    ///
    /// This method performs real file I/O via CommunitasApp with progress tracking.
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

        tokio::spawn(async move {
            // Mark as uploading
            {
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    p.state = UploadState::Uploading;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

            // Check for cancellation before starting
            {
                let map = uploads.read().await;
                if map
                    .get(&upload_id_clone)
                    .is_some_and(|p| matches!(p.state, UploadState::Cancelled))
                {
                    return;
                }
            }

            // Perform the actual write operation
            let cmd = Command::WriteFile {
                entity_id: entity_id_owned.clone(),
                disk_type: disk_type_to_arg(disk_type),
                path: path_owned.clone(),
                data: content.clone(),
            };

            // Update progress to show we're uploading (since write is atomic, show full progress)
            {
                let mut map = uploads.write().await;
                if let Some(p) = map.get_mut(&upload_id_clone) {
                    if matches!(p.state, UploadState::Cancelled) {
                        return;
                    }
                    p.bytes_uploaded = total_bytes;
                }
            }
            Self::broadcast_snapshot(&tx, &rx, Some(&uploads), None).await;

            let write_result = app.execute(cmd).await;

            // Check for cancellation after write
            {
                let map = uploads.read().await;
                if map
                    .get(&upload_id_clone)
                    .is_some_and(|p| matches!(p.state, UploadState::Cancelled))
                {
                    return;
                }
            }

            match write_result {
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
                            disk_type: disk_type_to_arg(disk_type),
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
                    // Mark as failed
                    tracing::error!("Upload failed: {:?}", e);
                    {
                        let mut map = uploads.write().await;
                        if let Some(p) = map.get_mut(&upload_id_clone) {
                            p.state = UploadState::Failed(e.message);
                        }
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

    // ===== Helper Methods =====

    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated(_)
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
}
