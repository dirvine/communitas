use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::{Map, Transact};

/// Organization entity with four-word identity and dual virtual disks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    /// Unique identifier (UUID)
    pub id: String,
    /// Four-word network identity for this organization
    pub four_word_identity: String,
    /// Organization name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Creator's user ID
    pub created_by: String,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: Option<i64>,
    /// Private shared disk ID (encrypted, member-only, CRDT replicated)
    pub private_disk_id: String,
    /// Public web disk ID (public accessible, member-editable)
    pub public_disk_id: String,
    /// Optional website root hash for DNS-free publishing
    pub website_root: Option<String>,
    /// CRDT document ID for synchronization
    pub crdt_doc_id: String,
}

/// Organization member relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationMember {
    pub org_id: String,
    pub user_id: String,
    pub role: String, // "owner", "admin", "member"
    pub joined_at: i64,
}

/// Service for managing organizations with CRDT synchronization
pub struct OrganizationService {
    crdt: Arc<CrdtManager>,
}

impl OrganizationService {
    /// Create a new OrganizationService
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self { crdt }
    }

    /// Create a new organization with four-word identity and dual virtual disks
    pub async fn create_organization(
        &self,
        name: &str,
        description: Option<String>,
        created_by: &str,
    ) -> Result<Organization> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        // Generate four-word identity for this organization
        let four_word_identity = Self::generate_four_word_identity()?;

        // Generate disk IDs
        let private_disk_id = format!("disk:private:{}", Uuid::new_v4());
        let public_disk_id = format!("disk:public:{}", Uuid::new_v4());

        // Create CRDT document for organization data
        let doc_id = format!("organization:{}", id);
        let doc = yrs::Doc::new();
        {
            let org_data = doc.get_or_insert_map("org_data");
            let mut txn = doc.transact_mut();
            org_data.insert(&mut txn, "name", name);
            org_data.insert(&mut txn, "four_word_identity", four_word_identity.as_str());
            org_data.insert(&mut txn, "created_by", created_by);
            if let Some(desc) = &description {
                org_data.insert(&mut txn, "description", desc.as_str());
            }
        }

        // Save CRDT document
        self.crdt
            .save_document(&doc_id, "organization", &id, &doc)
            .await?;

        // Save organization metadata to database
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO organizations (
                id, four_word_identity, name, description, created_by, created_at,
                private_disk_id, public_disk_id, crdt_doc_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id.clone(),
                four_word_identity.clone(),
                name,
                description.clone(),
                created_by,
                now,
                private_disk_id.clone(),
                public_disk_id.clone(),
                doc_id.clone()
            ],
        )
        .await
        .context("Failed to create organization")?;

        // Add creator as owner
        self.add_member(&id, created_by, "owner").await?;

        Ok(Organization {
            id,
            four_word_identity,
            name: name.to_string(),
            description,
            created_by: created_by.to_string(),
            created_at: now,
            updated_at: None,
            private_disk_id,
            public_disk_id,
            website_root: None,
            crdt_doc_id: doc_id,
        })
    }

    /// Get organization by ID
    pub async fn get_organization(&self, org_id: &str) -> Result<Option<Organization>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, name, description, created_by, created_at,
                        updated_at, private_disk_id, public_disk_id, website_root, crdt_doc_id
                 FROM organizations WHERE id = ?",
                params![org_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Organization {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                private_disk_id: row.get(7)?,
                public_disk_id: row.get(8)?,
                website_root: row.get(9)?,
                crdt_doc_id: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get organization by four-word identity
    pub async fn get_organization_by_four_words(
        &self,
        four_words: &str,
    ) -> Result<Option<Organization>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, four_word_identity, name, description, created_by, created_at,
                        updated_at, private_disk_id, public_disk_id, website_root, crdt_doc_id
                 FROM organizations WHERE four_word_identity = ?",
                params![four_words],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Organization {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                private_disk_id: row.get(7)?,
                public_disk_id: row.get(8)?,
                website_root: row.get(9)?,
                crdt_doc_id: row.get(10)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all organizations for a user
    pub async fn list_user_organizations(&self, user_id: &str) -> Result<Vec<Organization>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT o.id, o.four_word_identity, o.name, o.description, o.created_by,
                        o.created_at, o.updated_at, o.private_disk_id, o.public_disk_id,
                        o.website_root, o.crdt_doc_id
                 FROM organizations o
                 INNER JOIN organization_members om ON o.id = om.org_id
                 WHERE om.user_id = ?
                 ORDER BY o.created_at DESC",
                params![user_id],
            )
            .await?;

        let mut orgs = Vec::new();
        while let Some(row) = rows.next().await? {
            orgs.push(Organization {
                id: row.get(0)?,
                four_word_identity: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
                private_disk_id: row.get(7)?,
                public_disk_id: row.get(8)?,
                website_root: row.get(9)?,
                crdt_doc_id: row.get(10)?,
            });
        }

        Ok(orgs)
    }

    /// Update organization details
    pub async fn update_organization(
        &self,
        org_id: &str,
        name: Option<&str>,
        description: Option<String>,
    ) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        // Update CRDT document
        let doc_id = format!("organization:{}", org_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        {
            let org_data = doc.get_or_insert_map("org_data");
            let mut txn = doc.transact_mut();

            if let Some(name) = name {
                org_data.insert(&mut txn, "name", name);
            }
            if let Some(desc) = &description {
                org_data.insert(&mut txn, "description", desc.as_str());
            }
            org_data.insert(&mut txn, "updated_at", now.to_string().as_str());
        }

        // Save updated CRDT document
        self.crdt
            .save_document(&doc_id, "organization", org_id, &doc)
            .await?;

        // Update database
        if let Some(name) = name {
            db.execute(
                "UPDATE organizations SET name = ?, updated_at = ? WHERE id = ?",
                params![name, now, org_id],
            )
            .await
            .context("Failed to update organization name")?;
        }

        if let Some(desc) = description {
            db.execute(
                "UPDATE organizations SET description = ?, updated_at = ? WHERE id = ?",
                params![desc, now, org_id],
            )
            .await
            .context("Failed to update organization description")?;
        }

        Ok(())
    }

    /// Set website root for organization
    pub async fn set_website_root(&self, org_id: &str, website_root_hash: &str) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        // Update CRDT document
        let doc_id = format!("organization:{}", org_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        {
            let org_data = doc.get_or_insert_map("org_data");
            let mut txn = doc.transact_mut();
            org_data.insert(&mut txn, "website_root", website_root_hash);
            org_data.insert(&mut txn, "updated_at", now.to_string().as_str());
        }

        self.crdt
            .save_document(&doc_id, "organization", org_id, &doc)
            .await?;

        // Update database
        db.execute(
            "UPDATE organizations SET website_root = ?, updated_at = ? WHERE id = ?",
            params![website_root_hash, now, org_id],
        )
        .await
        .context("Failed to set organization website root")?;

        Ok(())
    }

    /// Add member to organization
    pub async fn add_member(&self, org_id: &str, user_id: &str, role: &str) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        db.execute(
            "INSERT OR REPLACE INTO organization_members (org_id, user_id, role, joined_at)
             VALUES (?, ?, ?, ?)",
            params![org_id, user_id, role, now],
        )
        .await
        .context("Failed to add organization member")?;

        Ok(())
    }

    /// Remove member from organization
    pub async fn remove_member(&self, org_id: &str, user_id: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        db.execute(
            "DELETE FROM organization_members WHERE org_id = ? AND user_id = ?",
            params![org_id, user_id],
        )
        .await
        .context("Failed to remove organization member")?;

        Ok(())
    }

    /// Get organization members
    pub async fn get_members(&self, org_id: &str) -> Result<Vec<OrganizationMember>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT org_id, user_id, role, joined_at
                 FROM organization_members WHERE org_id = ?
                 ORDER BY joined_at ASC",
                params![org_id],
            )
            .await?;

        let mut members = Vec::new();
        while let Some(row) = rows.next().await? {
            members.push(OrganizationMember {
                org_id: row.get(0)?,
                user_id: row.get(1)?,
                role: row.get(2)?,
                joined_at: row.get(3)?,
            });
        }

        Ok(members)
    }

    /// Update member role
    pub async fn update_member_role(&self, org_id: &str, user_id: &str, role: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        db.execute(
            "UPDATE organization_members SET role = ? WHERE org_id = ? AND user_id = ?",
            params![role, org_id, user_id],
        )
        .await
        .context("Failed to update member role")?;

        Ok(())
    }

    /// Check if user is member of organization
    pub async fn is_member(&self, org_id: &str, user_id: &str) -> Result<bool> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT COUNT(*) FROM organization_members WHERE org_id = ? AND user_id = ?",
                params![org_id, user_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let count: i64 = row.get(0)?;
            Ok(count > 0)
        } else {
            Ok(false)
        }
    }

    /// Delete organization
    pub async fn delete_organization(&self, org_id: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        // Remove all members
        db.execute(
            "DELETE FROM organization_members WHERE org_id = ?",
            params![org_id],
        )
        .await
        .context("Failed to remove organization members")?;

        // Delete organization
        db.execute("DELETE FROM organizations WHERE id = ?", params![org_id])
            .await
            .context("Failed to delete organization")?;

        Ok(())
    }

    /// Get CRDT update for sync
    pub async fn get_organization_update(&self, org_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("organization:{}", org_id);
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
    pub async fn apply_organization_update(&self, org_id: &str, update: &[u8]) -> Result<()> {
        let doc_id = format!("organization:{}", org_id);
        let doc = self.crdt.load_document(&doc_id).await?;

        {
            use yrs::updates::decoder::Decode;
            use yrs::Transact;
            let mut txn = doc.transact_mut();
            let update = yrs::Update::decode_v1(update)
                .map_err(|e| anyhow::anyhow!("Failed to decode update: {}", e))?;
            txn.apply_update(update);
        }

        self.crdt
            .save_document(&doc_id, "organization", org_id, &doc)
            .await?;
        Ok(())
    }

    /// Get organization state vector for efficient sync
    pub async fn get_organization_state_vector(&self, org_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("organization:{}", org_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let sv = {
            use yrs::{ReadTxn, Transact};
            let txn = doc.transact();
            txn.state_vector()
        };
        use yrs::updates::encoder::Encode;
        Ok(sv.encode_v1())
    }

    /// Get organization diff for efficient sync
    pub async fn get_organization_diff(&self, org_id: &str, remote_sv: &[u8]) -> Result<Vec<u8>> {
        let doc_id = format!("organization:{}", org_id);
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

    /// Apply organization diff for efficient sync
    pub async fn apply_organization_diff(&self, org_id: &str, diff: &[u8]) -> Result<()> {
        self.apply_organization_update(org_id, diff).await
    }

    /// Generate a unique four-word identity for an organization
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
