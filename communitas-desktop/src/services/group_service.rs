use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub four_word_identity: String,
    pub org_id: Option<String>, // Optional for personal groups
    pub name: String,
    pub description: Option<String>,
    pub private_disk_id: String,
    pub public_disk_id: String,
    pub website_root: Option<String>,
    pub created_at: i64,
    pub created_by: String,
    pub group_type: String, // "organization" or "personal"
}

pub struct GroupService {
    crdt: Arc<CrdtManager>,
}

impl GroupService {
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self { crdt }
    }

    /// Create a new group
    pub async fn create_group(
        &self,
        org_id: Option<&str>,
        name: &str,
        description: Option<String>,
        created_by: &str,
        group_type: &str,
    ) -> Result<Group> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let doc_id = format!("group:{}", id);

        // Generate four-word identity for this group
        let four_word_identity = Self::generate_four_word_identity()?;

        // Generate disk IDs
        let private_disk_id = format!("disk:private:{}", Uuid::new_v4());
        let public_disk_id = format!("disk:public:{}", Uuid::new_v4());

        // Create CRDT document for group data (Map structure)
        let doc = yrs::Doc::new();
        let _group_data = doc.get_or_insert_map("group_data");

        self.crdt
            .save_document(&doc_id, "group", &id, &doc)
            .await?;

        // Save group metadata
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO groups (id, four_word_identity, org_id, name, description, private_disk_id, public_disk_id, crdt_doc_id, created_at, created_by, group_type)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id.clone(),
                four_word_identity.clone(),
                org_id,
                name,
                description.clone(),
                private_disk_id.clone(),
                public_disk_id.clone(),
                doc_id.clone(),
                now,
                created_by,
                group_type
            ],
        )
        .await
        .context("Failed to create group")?;

        Ok(Group {
            id,
            four_word_identity,
            org_id: org_id.map(|s| s.to_string()),
            name: name.to_string(),
            description,
            private_disk_id,
            public_disk_id,
            website_root: None,
            created_at: now,
            created_by: created_by.to_string(),
            group_type: group_type.to_string(),
        })
    }

    /// Get group by ID
    pub async fn get_group(&self, group_id: &str) -> Result<Option<Group>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, org_id, name, description, private_disk_id, public_disk_id, website_root, created_at, created_by, group_type
                 FROM groups WHERE id = ?",
                params![group_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Group {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                org_id: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                private_disk_id: row.get(5)?,
                public_disk_id: row.get(6)?,
                website_root: row.get(7)?,
                created_at: row.get(8)?,
                created_by: row.get(9)?,
                group_type: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all groups for an organization
    pub async fn list_org_groups(&self, org_id: &str) -> Result<Vec<Group>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, org_id, name, description, private_disk_id, public_disk_id, website_root, created_at, created_by, group_type
                 FROM groups WHERE org_id = ? AND group_type = 'organization'
                 ORDER BY created_at DESC",
                params![org_id],
            )
            .await?;

        let mut groups = Vec::new();
        while let Some(row) = rows.next().await? {
            groups.push(Group {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                org_id: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                private_disk_id: row.get(5)?,
                public_disk_id: row.get(6)?,
                website_root: row.get(7)?,
                created_at: row.get(8)?,
                created_by: row.get(9)?,
                group_type: row.get(10)?,
            });
        }

        Ok(groups)
    }

    /// List all personal groups for a user
    pub async fn list_personal_groups(&self, user_id: &str) -> Result<Vec<Group>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, org_id, name, description, private_disk_id, public_disk_id, website_root, created_at, created_by, group_type
                 FROM groups WHERE created_by = ? AND group_type = 'personal'
                 ORDER BY created_at DESC",
                params![user_id],
            )
            .await?;

        let mut groups = Vec::new();
        while let Some(row) = rows.next().await? {
            groups.push(Group {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                org_id: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                private_disk_id: row.get(5)?,
                public_disk_id: row.get(6)?,
                website_root: row.get(7)?,
                created_at: row.get(8)?,
                created_by: row.get(9)?,
                group_type: row.get(10)?,
            });
        }

        Ok(groups)
    }

    /// Add member to group
    pub async fn add_member(&self, group_id: &str, user_id: &str, role: &str) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        db.execute(
            "INSERT OR REPLACE INTO group_members (group_id, user_id, role, joined_at)
             VALUES (?, ?, ?, ?)",
            params![group_id, user_id, role, now],
        )
        .await
        .context("Failed to add group member")?;

        Ok(())
    }

    /// Remove member from group
    pub async fn remove_member(&self, group_id: &str, user_id: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        db.execute(
            "DELETE FROM group_members WHERE group_id = ? AND user_id = ?",
            params![group_id, user_id],
        )
        .await
        .context("Failed to remove group member")?;

        Ok(())
    }

    /// Get group members
    pub async fn get_members(&self, group_id: &str) -> Result<Vec<(String, String)>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT user_id, role FROM group_members WHERE group_id = ?",
                params![group_id],
            )
            .await?;

        let mut members = Vec::new();
        while let Some(row) = rows.next().await? {
            members.push((row.get(0)?, row.get(1)?));
        }

        Ok(members)
    }

    /// Update group details
    pub async fn update_group(
        &self,
        group_id: &str,
        name: Option<&str>,
        description: Option<String>,
    ) -> Result<()> {
        let db = self.crdt.connection()?;

        if let Some(name) = name {
            db.execute(
                "UPDATE groups SET name = ? WHERE id = ?",
                params![name, group_id],
            )
            .await
            .context("Failed to update group name")?;
        }

        if let Some(desc) = description {
            db.execute(
                "UPDATE groups SET description = ? WHERE id = ?",
                params![desc, group_id],
            )
            .await
            .context("Failed to update group description")?;
        }

        Ok(())
    }

    /// Delete group
    pub async fn delete_group(&self, group_id: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        // Remove all members first
        db.execute(
            "DELETE FROM group_members WHERE group_id = ?",
            params![group_id],
        )
        .await
        .context("Failed to remove group members")?;

        // Delete the group
        db.execute("DELETE FROM groups WHERE id = ?", params![group_id])
            .await
            .context("Failed to delete group")?;

        Ok(())
    }

    /// Get CRDT update for sync
    pub async fn get_group_update(&self, group_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("group:{}", group_id);
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
    pub async fn apply_group_update(&self, group_id: &str, update: &[u8]) -> Result<()> {
        let doc_id = format!("group:{}", group_id);
        let doc = self.crdt.load_document(&doc_id).await?;

        {
            use yrs::Transact;
            use yrs::updates::decoder::Decode;
            let mut txn = doc.transact_mut();
            let update = yrs::Update::decode_v1(update)
                .map_err(|e| anyhow::anyhow!("Failed to decode update: {}", e))?;
            txn.apply_update(update);
        }

        self.crdt.save_document(&doc_id, "group", group_id, &doc).await?;
        Ok(())
    }

    /// Get group state vector for efficient sync
    pub async fn get_group_state_vector(&self, group_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("group:{}", group_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let sv = {
            use yrs::{ReadTxn, Transact};
            let txn = doc.transact();
            txn.state_vector()
        };
        use yrs::updates::encoder::Encode;
        Ok(sv.encode_v1())
    }

    /// Get group diff for efficient sync
    pub async fn get_group_diff(&self, group_id: &str, remote_sv: &[u8]) -> Result<Vec<u8>> {
        let doc_id = format!("group:{}", group_id);
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

    /// Apply group diff for efficient sync
    pub async fn apply_group_diff(&self, group_id: &str, diff: &[u8]) -> Result<()> {
        self.apply_group_update(group_id, diff).await
    }

    /// Set website root for DNS-free publishing
    pub async fn set_website_root(&self, group_id: &str, website_root: &str) -> Result<()> {
        let db = self.crdt.connection()?;
        db.execute(
            "UPDATE groups SET website_root = ? WHERE id = ?",
            params![website_root, group_id],
        )
        .await
        .context("Failed to set group website root")?;
        Ok(())
    }

    /// Generate a four-word identity from UUID
    fn generate_four_word_identity() -> Result<String> {
        // TODO: Integrate with four-word-networking crate properly
        // For now, generate a placeholder four-word identifier from UUID
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string().replace('-', "");
        let word1 = &uuid_str[0..8];
        let word2 = &uuid_str[8..16];
        let word3 = &uuid_str[16..24];
        let word4 = &uuid_str[24..32];
        Ok(format!("{}-{}-{}-{}", word1, word2, word3, word4))
    }
}
