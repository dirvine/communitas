use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::{GetString, Map, Text, Transact};

/// Types of virtual disks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskType {
    /// Private shared disk: encrypted locally, full CRDT replication among members
    PrivateShared,
    /// Public web disk: accessible to anyone, only members can edit
    PublicWeb,
}

impl DiskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiskType::PrivateShared => "private_shared",
            DiskType::PublicWeb => "public_web",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "private_shared" => Ok(DiskType::PrivateShared),
            "public_web" => Ok(DiskType::PublicWeb),
            _ => Err(anyhow::anyhow!("Invalid disk type: {}", s)),
        }
    }
}

/// Virtual disk metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualDisk {
    pub id: String,
    pub entity_id: String,
    pub entity_type: String, // "organization", "channel", "group", "project", "member"
    pub disk_type: DiskType,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub crdt_doc_id: String,
}

/// File metadata in virtual disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskFile {
    pub id: String,
    pub disk_id: String,
    pub path: String,
    pub content_type: String, // "text/markdown", "text/plain", etc.
    pub size: u64,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub crdt_doc_id: Option<String>, // For collaborative editing
    pub is_encrypted: bool,
}

/// Directory listing entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub updated_at: Option<i64>,
}

/// Service for managing virtual disks (private shared + public web)
pub struct VirtualDiskService {
    crdt: Arc<CrdtManager>,
}

impl VirtualDiskService {
    /// Create a new VirtualDiskService
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self { crdt }
    }

    /// Create a new virtual disk for an entity
    pub async fn create_disk(
        &self,
        entity_id: &str,
        entity_type: &str,
        disk_type: DiskType,
    ) -> Result<VirtualDisk> {
        let id = format!("disk:{}:{}", disk_type.as_str(), Uuid::new_v4());
        let now = Utc::now().timestamp();

        // Create CRDT document for disk metadata
        let doc_id = format!("{}:metadata", id);
        let doc = yrs::Doc::new();
        {
            let disk_meta = doc.get_or_insert_map("disk_metadata");
            let mut txn = doc.transact_mut();
            disk_meta.insert(&mut txn, "entity_id", entity_id);
            disk_meta.insert(&mut txn, "entity_type", entity_type);
            disk_meta.insert(&mut txn, "disk_type", disk_type.as_str());
        }

        // Save CRDT document
        self.crdt
            .save_document(&doc_id, "disk", &id, &doc)
            .await?;

        // Save disk metadata to database
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO virtual_disks (id, entity_id, entity_type, disk_type, created_at, crdt_doc_id)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                id.clone(),
                entity_id,
                entity_type,
                disk_type.as_str(),
                now,
                doc_id.clone()
            ],
        )
        .await
        .context("Failed to create virtual disk")?;

        Ok(VirtualDisk {
            id,
            entity_id: entity_id.to_string(),
            entity_type: entity_type.to_string(),
            disk_type,
            created_at: now,
            updated_at: None,
            crdt_doc_id: doc_id,
        })
    }

    /// Get virtual disk by ID
    pub async fn get_disk(&self, disk_id: &str) -> Result<Option<VirtualDisk>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, entity_id, entity_type, disk_type, created_at, updated_at, crdt_doc_id
                 FROM virtual_disks WHERE id = ?",
                params![disk_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let disk_type_str: String = row.get(3)?;
            Ok(Some(VirtualDisk {
                id: row.get(0)?,
                entity_id: row.get(1)?,
                entity_type: row.get(2)?,
                disk_type: DiskType::from_str(&disk_type_str)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                crdt_doc_id: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Write file to disk
    pub async fn write_file(
        &self,
        disk_id: &str,
        path: &str,
        content: &[u8],
        content_type: &str,
        enable_crdt: bool, // Enable CRDT for collaborative editing
    ) -> Result<DiskFile> {
        let file_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        // Get disk to determine if encryption is needed
        let disk = self
            .get_disk(disk_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Disk not found"))?;

        let is_encrypted = disk.disk_type == DiskType::PrivateShared;

        // For collaborative documents, create CRDT document
        let crdt_doc_id = if enable_crdt && content_type.starts_with("text/") {
            let doc_id = format!("file:{}:{}", disk_id, file_id);
            let doc = yrs::Doc::new();
            {
                let text = doc.get_or_insert_text("content");
                let content_str = String::from_utf8_lossy(content);
                let mut txn = doc.transact_mut();
                text.insert(&mut txn, 0, &content_str);
            }

            self.crdt
                .save_document(&doc_id, "file", &file_id, &doc)
                .await?;
            Some(doc_id)
        } else {
            None
        };

        // Store file content (encrypted if private disk)
        let stored_content = if is_encrypted {
            // TODO: Integrate with vault encryption
            content.to_vec()
        } else {
            content.to_vec()
        };

        // Save file metadata and content
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT OR REPLACE INTO disk_files
             (id, disk_id, path, content, content_type, size, created_at, crdt_doc_id, is_encrypted)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                file_id.clone(),
                disk_id,
                path,
                stored_content,
                content_type,
                content.len() as i64,
                now,
                crdt_doc_id.clone(),
                is_encrypted
            ],
        )
        .await
        .context("Failed to write file")?;

        Ok(DiskFile {
            id: file_id,
            disk_id: disk_id.to_string(),
            path: path.to_string(),
            content_type: content_type.to_string(),
            size: content.len() as u64,
            created_at: now,
            updated_at: None,
            crdt_doc_id,
            is_encrypted,
        })
    }

    /// Read file from disk
    pub async fn read_file(&self, disk_id: &str, path: &str) -> Result<Vec<u8>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT content, is_encrypted FROM disk_files WHERE disk_id = ? AND path = ?",
                params![disk_id, path],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let content: Vec<u8> = row.get(0)?;
            let is_encrypted: bool = row.get(1)?;

            if is_encrypted {
                // TODO: Decrypt with vault
                Ok(content)
            } else {
                Ok(content)
            }
        } else {
            Err(anyhow::anyhow!("File not found: {}", path))
        }
    }

    /// Get file metadata
    pub async fn get_file(&self, disk_id: &str, path: &str) -> Result<Option<DiskFile>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, disk_id, path, content_type, size, created_at, updated_at, crdt_doc_id, is_encrypted
                 FROM disk_files WHERE disk_id = ? AND path = ?",
                params![disk_id, path],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(DiskFile {
                id: row.get(0)?,
                disk_id: row.get(1)?,
                path: row.get(2)?,
                content_type: row.get(3)?,
                size: row.get::<i64>(4)? as u64,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                crdt_doc_id: row.get(7)?,
                is_encrypted: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Delete file from disk
    pub async fn delete_file(&self, disk_id: &str, path: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        db.execute(
            "DELETE FROM disk_files WHERE disk_id = ? AND path = ?",
            params![disk_id, path],
        )
        .await
        .context("Failed to delete file")?;

        Ok(())
    }

    /// Check if file exists on disk
    pub async fn file_exists(&self, disk_id: &str, path: &str) -> Result<bool> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT COUNT(*) FROM disk_files WHERE disk_id = ? AND path = ?",
                params![disk_id, path],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// List files in directory
    pub async fn list_directory(&self, disk_id: &str, dir_path: &str) -> Result<Vec<DiskEntry>> {
        let db = self.crdt.connection()?;

        // Normalize directory path
        let normalized_dir = if dir_path.is_empty() || dir_path == "/" {
            "".to_string()
        } else {
            format!("{}/", dir_path.trim_matches('/'))
        };

        // Get all files under this directory
        let pattern = if normalized_dir.is_empty() {
            "%".to_string()
        } else {
            format!("{}%", normalized_dir)
        };

        let mut rows = db
            .query(
                "SELECT path, size, updated_at FROM disk_files
                 WHERE disk_id = ? AND path LIKE ? AND path != ?
                 ORDER BY path",
                params![disk_id, pattern, normalized_dir.clone()],
            )
            .await?;

        let mut entries = Vec::new();
        let mut seen_dirs = std::collections::HashSet::new();

        while let Some(row) = rows.next().await? {
            let full_path: String = row.get(0)?;

            // Get relative path from directory
            let rel_path = if normalized_dir.is_empty() {
                &full_path
            } else {
                full_path.strip_prefix(&normalized_dir).ok_or_else(|| {
                    anyhow::anyhow!("Path doesn't start with directory: {}", full_path)
                })?
            };

            // Check if this is a subdirectory or direct file
            if let Some(slash_pos) = rel_path.find('/') {
                // This is in a subdirectory
                let dir_name = &rel_path[..slash_pos];
                if !seen_dirs.contains(dir_name) {
                    seen_dirs.insert(dir_name.to_string());
                    entries.push(DiskEntry {
                        name: dir_name.to_string(),
                        path: if normalized_dir.is_empty() {
                            dir_name.to_string()
                        } else {
                            format!("{}{}", normalized_dir, dir_name)
                        },
                        is_directory: true,
                        size: None,
                        updated_at: None,
                    });
                }
            } else {
                // Direct file in this directory
                entries.push(DiskEntry {
                    name: rel_path.to_string(),
                    path: full_path,
                    is_directory: false,
                    size: Some(row.get::<i64>(1)? as u64),
                    updated_at: row.get(2)?,
                });
            }
        }

        Ok(entries)
    }

    /// Get CRDT update for file sync
    pub async fn get_file_update(&self, file_crdt_doc_id: &str) -> Result<Vec<u8>> {
        let doc = self.crdt.load_document(file_crdt_doc_id).await?;
        let update = {
            use yrs::{ReadTxn, Transact};
            let sv = yrs::StateVector::default();
            let txn = doc.transact();
            txn.encode_diff_v1(&sv)
        };
        Ok(update)
    }

    /// Apply CRDT update for file sync
    pub async fn apply_file_update(
        &self,
        file_crdt_doc_id: &str,
        update: &[u8],
    ) -> Result<String> {
        let doc = self.crdt.load_document(file_crdt_doc_id).await?;

        {
            use yrs::updates::decoder::Decode;
            use yrs::Transact;
            let mut txn = doc.transact_mut();
            let update = yrs::Update::decode_v1(update)
                .map_err(|e| anyhow::anyhow!("Failed to decode update: {}", e))?;
            txn.apply_update(update);
        }

        // Extract current text content
        let content = {
            let text = doc.get_or_insert_text("content");
            let txn = doc.transact();
            text.get_string(&txn)
        };

        // Save updated document
        // Extract file_id from doc_id format "file:{disk_id}:{file_id}"
        let parts: Vec<&str> = file_crdt_doc_id.split(':').collect();
        if parts.len() >= 3 {
            let file_id = parts[2];
            self.crdt
                .save_document(file_crdt_doc_id, "file", file_id, &doc)
                .await?;
        }

        Ok(content)
    }

    /// Get file state vector for efficient sync
    pub async fn get_file_state_vector(&self, file_crdt_doc_id: &str) -> Result<Vec<u8>> {
        let doc = self.crdt.load_document(file_crdt_doc_id).await?;
        let sv = {
            use yrs::{ReadTxn, Transact};
            let txn = doc.transact();
            txn.state_vector()
        };
        use yrs::updates::encoder::Encode;
        Ok(sv.encode_v1())
    }

    /// Get file diff for efficient sync
    pub async fn get_file_diff(&self, file_crdt_doc_id: &str, remote_sv: &[u8]) -> Result<Vec<u8>> {
        let doc = self.crdt.load_document(file_crdt_doc_id).await?;

        let sv = {
            use yrs::updates::decoder::Decode;
            yrs::StateVector::decode_v1(remote_sv)
                .map_err(|e| anyhow::anyhow!("Failed to decode state vector: {}", e))?
        };

        let diff = {
            use yrs::{ReadTxn, Transact};
            let txn = doc.transact();
            txn.encode_diff_v1(&sv)
        };

        Ok(diff)
    }

    /// Check if user has access to disk (member check for private disks)
    pub async fn has_access(
        &self,
        disk_id: &str,
        _user_id: &str,
        is_member: bool,
    ) -> Result<bool> {
        let disk = self
            .get_disk(disk_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Disk not found"))?;

        match disk.disk_type {
            DiskType::PrivateShared => {
                // Private disks require membership
                Ok(is_member)
            }
            DiskType::PublicWeb => {
                // Public disks are readable by anyone
                // Write access still requires membership
                Ok(true)
            }
        }
    }
}
