// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


// Copyright 2024 P2P Foundation
// SPDX-License-Identifier: AGPL-3.0-or-later

//! File sharing module for Communitas
//! Implements Dropbox-like file sharing with DHT storage

use anyhow::{Context, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: String,
    pub hash: String, // BLAKE3 hash
    pub filename: String,
    pub size: u64,
    pub mime_type: String,
    pub owner: String,            // Four-word address
    pub group_id: Option<String>, // If shared in a group
    pub created_at: i64,
    pub chunks: Vec<String>,       // Chunk hashes
    pub thumbnail: Option<String>, // For images/videos
}

/// File chunk for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    pub hash: String,
    pub data: Vec<u8>,
    pub sequence: u32,
    pub total_chunks: u32,
}

/// Upload progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadProgress {
    pub file_id: String,
    pub filename: String,
    pub total_bytes: u64,
    pub uploaded_bytes: u64,
    pub percentage: f32,
    pub status: UploadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UploadStatus {
    Preparing,
    Uploading,
    Processing,
    Complete,
    Failed(String),
}

/// File manager
#[derive(Debug)]
pub struct FileManager {
    files: Arc<RwLock<HashMap<String, FileMetadata>>>,
    chunks: Arc<RwLock<HashMap<String, FileChunk>>>,
    uploads: Arc<RwLock<HashMap<String, UploadProgress>>>,
    user_quota: Arc<RwLock<HashMap<String, u64>>>, // user -> bytes used
    max_file_size: u64,                            // 100MB
    max_user_quota: u64,                           // 10GB
    chunk_size: usize,                             // 1MB
}

impl FileManager {
    /// Create a new file manager
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            chunks: Arc::new(RwLock::new(HashMap::new())),
            uploads: Arc::new(RwLock::new(HashMap::new())),
            user_quota: Arc::new(RwLock::new(HashMap::new())),
            max_file_size: 100 * 1024 * 1024,        // 100MB
            max_user_quota: 10 * 1024 * 1024 * 1024, // 10GB
            chunk_size: 1024 * 1024,                 // 1MB
        }
    }

    /// Upload a file
    pub async fn upload_file(
        &self,
        path: String,
        owner: String,
        group_id: Option<String>,
    ) -> Result<FileMetadata> {
        let file_path = Path::new(&path);
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename")?
            .to_string();

        // Check file size
        let metadata = fs::metadata(&path).await?;
        if metadata.len() > self.max_file_size {
            anyhow::bail!("File exceeds maximum size of 100MB");
        }

        // Check user quota
        let mut user_quota = self.user_quota.write().await;
        let used = user_quota.entry(owner.clone()).or_insert(0);
        if *used + metadata.len() > self.max_user_quota {
            anyhow::bail!("User quota exceeded (10GB limit)");
        }

        // Create upload progress
        let file_id = Uuid::new_v4().to_string();
        let progress = UploadProgress {
            file_id: file_id.clone(),
            filename: filename.clone(),
            total_bytes: metadata.len(),
            uploaded_bytes: 0,
            percentage: 0.0,
            status: UploadStatus::Preparing,
        };
        self.uploads
            .write()
            .await
            .insert(file_id.clone(), progress.clone());

        // Read file and calculate hash
        let file_data = fs::read(&path).await?;
        let mut hasher = Hasher::new();
        hasher.update(&file_data);
        let file_hash = hasher.finalize().to_hex().to_string();

        // Update progress
        self.update_upload_progress(&file_id, UploadStatus::Uploading, 0)
            .await?;

        // Chunk the file
        let mut chunk_hashes = Vec::new();
        let total_chunks = ((file_data.len() + self.chunk_size - 1) / self.chunk_size) as u32;

        for (i, chunk_data) in file_data.chunks(self.chunk_size).enumerate() {
            let mut chunk_hasher = Hasher::new();
            chunk_hasher.update(chunk_data);
            let chunk_hash = chunk_hasher.finalize().to_hex().to_string();

            let chunk = FileChunk {
                hash: chunk_hash.clone(),
                data: chunk_data.to_vec(),
                sequence: i as u32,
                total_chunks,
            };

            // Store chunk
            self.chunks.write().await.insert(chunk_hash.clone(), chunk);
            chunk_hashes.push(chunk_hash);

            // Update progress
            let uploaded = (i + 1) * self.chunk_size;
            self.update_upload_progress(&file_id, UploadStatus::Uploading, uploaded as u64)
                .await?;
        }

        // Detect mime type
        let mime_type = mime_guess::from_path(&path)
            .first_or_octet_stream()
            .to_string();

        // Generate thumbnail for images/videos
        let thumbnail = if mime_type.starts_with("image/") || mime_type.starts_with("video/") {
            Some(self.generate_thumbnail(&file_data, &mime_type).await?)
        } else {
            None
        };

        // Create file metadata
        let file_metadata = FileMetadata {
            id: file_id.clone(),
            hash: file_hash,
            filename,
            size: metadata.len(),
            mime_type,
            owner: owner.clone(),
            group_id,
            created_at: chrono::Utc::now().timestamp(),
            chunks: chunk_hashes,
            thumbnail,
        };

        // Store file metadata
        self.files
            .write()
            .await
            .insert(file_id.clone(), file_metadata.clone());

        // Update user quota
        *used += metadata.len();

        // Mark upload complete
        self.update_upload_progress(&file_id, UploadStatus::Complete, metadata.len())
            .await?;

        Ok(file_metadata)
    }

    /// Download a file
    pub async fn download_file(&self, file_hash: String, destination: String) -> Result<()> {
        let files = self.files.read().await;
        let file_metadata = files
            .values()
            .find(|f| f.hash == file_hash)
            .context("File not found")?;

        // Reassemble chunks
        let mut file_data = Vec::with_capacity(file_metadata.size as usize);
        let chunks = self.chunks.read().await;

        for chunk_hash in &file_metadata.chunks {
            let chunk = chunks.get(chunk_hash).context("Chunk not found")?;
            file_data.extend_from_slice(&chunk.data);
        }

        // Write to destination
        fs::write(&destination, file_data).await?;

        Ok(())
    }

    /// Get file metadata
    pub async fn get_file_info(&self, file_id: String) -> Result<FileMetadata> {
        let files = self.files.read().await;
        files.get(&file_id).cloned().context("File not found")
    }

    /// List files for a user
    pub async fn list_user_files(&self, user: String) -> Result<Vec<FileMetadata>> {
        let files = self.files.read().await;
        let user_files: Vec<FileMetadata> = files
            .values()
            .filter(|f| f.owner == user)
            .cloned()
            .collect();
        Ok(user_files)
    }

    /// List files in a group
    pub async fn list_group_files(&self, group_id: String) -> Result<Vec<FileMetadata>> {
        let files = self.files.read().await;
        let group_files: Vec<FileMetadata> = files
            .values()
            .filter(|f| f.group_id.as_ref() == Some(&group_id))
            .cloned()
            .collect();
        Ok(group_files)
    }

    /// Delete a file
    pub async fn delete_file(&self, file_id: String, user: String) -> Result<()> {
        let mut files = self.files.write().await;
        let file = files.get(&file_id).context("File not found")?;

        if file.owner != user {
            anyhow::bail!("Only the owner can delete this file");
        }

        let file_size = file.size;
        let chunk_hashes = file.chunks.clone();

        // Remove file metadata
        files.remove(&file_id);

        // Remove chunks
        let mut chunks = self.chunks.write().await;
        for chunk_hash in chunk_hashes {
            chunks.remove(&chunk_hash);
        }

        // Update user quota
        let mut user_quota = self.user_quota.write().await;
        if let Some(used) = user_quota.get_mut(&user) {
            *used = used.saturating_sub(file_size);
        }

        Ok(())
    }

    /// Get user's storage usage
    pub async fn get_user_storage(&self, user: String) -> Result<(u64, u64)> {
        let user_quota = self.user_quota.read().await;
        let used = user_quota.get(&user).copied().unwrap_or(0);
        Ok((used, self.max_user_quota))
    }

    /// Update upload progress
    async fn update_upload_progress(
        &self,
        file_id: &str,
        status: UploadStatus,
        uploaded_bytes: u64,
    ) -> Result<()> {
        let mut uploads = self.uploads.write().await;
        if let Some(progress) = uploads.get_mut(file_id) {
            progress.status = status;
            progress.uploaded_bytes = uploaded_bytes;
            progress.percentage = (uploaded_bytes as f32 / progress.total_bytes as f32) * 100.0;
        }
        Ok(())
    }

    /// Generate thumbnail for media files
    async fn generate_thumbnail(&self, _data: &[u8], _mime_type: &str) -> Result<String> {
        // In a real implementation, this would use image processing libraries
        // to generate actual thumbnails
        Ok("mock-thumbnail-hash".to_string())
    }

    /// Get upload progress
    pub async fn get_upload_progress(&self, file_id: String) -> Result<UploadProgress> {
        let uploads = self.uploads.read().await;
        uploads.get(&file_id).cloned().context("Upload not found")
    }
}
