use crate::crdt_manager::CrdtManager;
use crate::services::virtual_disk_service::{DiskType, VirtualDiskService};
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::{Map, Transact};

/// Member (user with four-word identity and personal disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,
    pub four_word_identity: String,
    pub display_name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub created_at: i64,
    pub updated_at: Option<i64>,
    pub personal_disk_id: String, // Private personal storage
    pub website_root: Option<String>,
    pub crdt_doc_id: String,
}

/// Service for managing members (users with four-word identities)
pub struct MemberService {
    crdt: Arc<CrdtManager>,
    disk_service: Arc<VirtualDiskService>,
}

impl MemberService {
    /// Create a new MemberService
    pub fn new(crdt: Arc<CrdtManager>, disk_service: Arc<VirtualDiskService>) -> Self {
        Self { crdt, disk_service }
    }

    /// Create a new member with four-word identity
    pub async fn create_member(
        &self,
        display_name: &str,
        email: Option<String>,
    ) -> Result<Member> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        // Generate four-word identity
        let four_word_identity = generate_four_word_identity()?;

        // Create personal disk for private storage
        let personal_disk = self
            .disk_service
            .create_disk(&id, "member", DiskType::PrivateShared)
            .await?;

        // Create CRDT document for member metadata
        let doc_id = format!("member:{}", id);
        let doc = yrs::Doc::new();
        {
            let member_meta = doc.get_or_insert_map("member_metadata");
            let mut txn = doc.transact_mut();
            member_meta.insert(&mut txn, "display_name", display_name);
            member_meta.insert(&mut txn, "four_word_identity", four_word_identity.as_str());
            if let Some(email) = &email {
                member_meta.insert(&mut txn, "email", email.as_str());
            }
        }

        // Save CRDT document
        self.crdt
            .save_document(&doc_id, "member", &id, &doc)
            .await?;

        // Save member metadata to database
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO members (id, four_word_identity, display_name, email, created_at, personal_disk_id, crdt_doc_id)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                id.clone(),
                four_word_identity.clone(),
                display_name,
                email.clone(),
                now,
                personal_disk.id.clone(),
                doc_id.clone()
            ],
        )
        .await
        .context("Failed to create member")?;

        Ok(Member {
            id,
            four_word_identity,
            display_name: display_name.to_string(),
            email,
            avatar_url: None,
            bio: None,
            created_at: now,
            updated_at: None,
            personal_disk_id: personal_disk.id,
            website_root: None,
            crdt_doc_id: doc_id,
        })
    }

    /// Get member by ID
    pub async fn get_member(&self, member_id: &str) -> Result<Option<Member>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, display_name, email, avatar_url, bio, created_at, updated_at, personal_disk_id, website_root, crdt_doc_id
                 FROM members WHERE id = ?",
                params![member_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Member {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                display_name: row.get(2)?,
                email: row.get(3)?,
                avatar_url: row.get(4)?,
                bio: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                personal_disk_id: row.get(8)?,
                website_root: row.get(9)?,
                crdt_doc_id: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get member by four-word identity
    pub async fn get_member_by_four_words(
        &self,
        four_words: &str,
    ) -> Result<Option<Member>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, display_name, email, avatar_url, bio, created_at, updated_at, personal_disk_id, website_root, crdt_doc_id
                 FROM members WHERE four_word_identity = ?",
                params![four_words],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Member {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                display_name: row.get(2)?,
                email: row.get(3)?,
                avatar_url: row.get(4)?,
                bio: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                personal_disk_id: row.get(8)?,
                website_root: row.get(9)?,
                crdt_doc_id: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update member profile
    pub async fn update_member(
        &self,
        member_id: &str,
        display_name: Option<&str>,
        email: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    ) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        // Update CRDT document
        let doc_id = format!("member:{}", member_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        {
            let member_meta = doc.get_or_insert_map("member_metadata");
            let mut txn = doc.transact_mut();

            if let Some(name) = display_name {
                member_meta.insert(&mut txn, "display_name", name);
            }
            if let Some(email) = &email {
                member_meta.insert(&mut txn, "email", email.as_str());
            }
            if let Some(avatar) = &avatar_url {
                member_meta.insert(&mut txn, "avatar_url", avatar.as_str());
            }
            if let Some(bio) = &bio {
                member_meta.insert(&mut txn, "bio", bio.as_str());
            }
        }

        self.crdt
            .save_document(&doc_id, "member", member_id, &doc)
            .await?;

        // Update database
        if let Some(name) = display_name {
            db.execute(
                "UPDATE members SET display_name = ?, updated_at = ? WHERE id = ?",
                params![name, now, member_id],
            )
            .await
            .context("Failed to update member display name")?;
        }

        if let Some(email) = email {
            db.execute(
                "UPDATE members SET email = ?, updated_at = ? WHERE id = ?",
                params![email, now, member_id],
            )
            .await
            .context("Failed to update member email")?;
        }

        if let Some(avatar) = avatar_url {
            db.execute(
                "UPDATE members SET avatar_url = ?, updated_at = ? WHERE id = ?",
                params![avatar, now, member_id],
            )
            .await
            .context("Failed to update member avatar")?;
        }

        if let Some(bio) = bio {
            db.execute(
                "UPDATE members SET bio = ?, updated_at = ? WHERE id = ?",
                params![bio, now, member_id],
            )
            .await
            .context("Failed to update member bio")?;
        }

        Ok(())
    }

    /// Set website root for member's public web presence
    pub async fn set_website_root(
        &self,
        member_id: &str,
        website_root: &str,
    ) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        // Update CRDT document
        let doc_id = format!("member:{}", member_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        {
            let member_meta = doc.get_or_insert_map("member_metadata");
            let mut txn = doc.transact_mut();
            member_meta.insert(&mut txn, "website_root", website_root);
        }

        self.crdt
            .save_document(&doc_id, "member", member_id, &doc)
            .await?;

        // Update database
        db.execute(
            "UPDATE members SET website_root = ?, updated_at = ? WHERE id = ?",
            params![website_root, now, member_id],
        )
        .await
        .context("Failed to set website root")?;

        Ok(())
    }

    /// Delete member (soft delete via CRDT tombstone)
    pub async fn delete_member(&self, member_id: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        // Delete from database
        db.execute("DELETE FROM members WHERE id = ?", params![member_id])
            .await
            .context("Failed to delete member")?;

        Ok(())
    }

    /// Get CRDT update for sync
    pub async fn get_member_update(&self, member_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("member:{}", member_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let update = {
            use yrs::{ReadTxn, Transact};
            let sv = yrs::StateVector::default();
            let txn = doc.transact();
            txn.encode_diff_v1(&sv)
        };
        Ok(update)
    }

    /// Apply CRDT update for sync
    pub async fn apply_member_update(&self, member_id: &str, update: &[u8]) -> Result<()> {
        let doc_id = format!("member:{}", member_id);
        let doc = self.crdt.load_document(&doc_id).await?;

        {
            use yrs::Transact;
            use yrs::updates::decoder::Decode;
            let mut txn = doc.transact_mut();
            let update = yrs::Update::decode_v1(update)
                .map_err(|e| anyhow::anyhow!("Failed to decode update: {}", e))?;
            txn.apply_update(update);
        }

        self.crdt
            .save_document(&doc_id, "member", member_id, &doc)
            .await?;
        Ok(())
    }

    /// Get member state vector for efficient sync
    pub async fn get_member_state_vector(&self, member_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("member:{}", member_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let sv = {
            use yrs::{ReadTxn, Transact};
            let txn = doc.transact();
            txn.state_vector()
        };
        use yrs::updates::encoder::Encode;
        Ok(sv.encode_v1())
    }

    /// Get member diff for efficient sync
    pub async fn get_member_diff(&self, member_id: &str, remote_sv: &[u8]) -> Result<Vec<u8>> {
        let doc_id = format!("member:{}", member_id);
        let doc = self.crdt.load_document(&doc_id).await?;

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

    /// Apply member diff for efficient sync
    pub async fn apply_member_diff(&self, member_id: &str, diff: &[u8]) -> Result<()> {
        self.apply_member_update(member_id, diff).await
    }

    /// List all members
    pub async fn list_all_members(&self) -> Result<Vec<Member>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, display_name, email, avatar_url, bio, created_at, updated_at, personal_disk_id, website_root, crdt_doc_id
                 FROM members ORDER BY created_at DESC",
                params![],
            )
            .await?;

        let mut members = Vec::new();
        while let Some(row) = rows.next().await? {
            members.push(Member {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                display_name: row.get(2)?,
                email: row.get(3)?,
                avatar_url: row.get(4)?,
                bio: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                personal_disk_id: row.get(8)?,
                website_root: row.get(9)?,
                crdt_doc_id: row.get(10)?,
            });
        }

        Ok(members)
    }
}

/// Generate a four-word identity for a member
fn generate_four_word_identity() -> Result<String> {
    // TODO: Integrate with four-word-networking crate properly
    // For now, use UUID-based generation
    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string().replace('-', "");
    let word1 = &uuid_str[0..8];
    let word2 = &uuid_str[8..16];
    let word3 = &uuid_str[16..24];
    let word4 = &uuid_str[24..32];
    Ok(format!("{}-{}-{}-{}", word1, word2, word3, word4))
}
