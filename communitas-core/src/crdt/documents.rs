// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Document Type Definitions
//!
//! Defines the structure of CRDT documents for all entity types in Communitas.
//! Each entity type (Channel, Organization, Member, etc.) has a corresponding
//! document structure that uses Yrs Y.Map for conflict-free replication.

use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use yrs::{
    Any, ArrayPrelim, Doc, Map as YrsMap, MapPrelim, MapRef, ReadTxn, Transact, TransactionMut,
};

/// Trait for entities that can be stored as CRDT documents
pub trait CrdtDocument: Sized {
    /// Get the document ID prefix (e.g., "channel", "member", "org")
    fn document_type() -> &'static str;

    /// Create a new Yrs document for this entity
    fn create_document(id: &str) -> Result<Doc>;

    /// Load entity from Yrs document
    fn from_document(doc: &Doc) -> Result<Self>;

    /// Update Yrs document with entity data
    fn update_document(&self, doc: &Doc) -> Result<()>;

    /// Get the entity ID
    fn entity_id(&self) -> &str;

    /// Get full document ID (prefix:id)
    fn document_id(&self) -> String {
        format!("{}:{}", Self::document_type(), self.entity_id())
    }
}

/// Helper to get or create a nested map in a Yrs document
pub fn get_or_create_map(txn: &mut TransactionMut, parent: &MapRef, key: &str) -> Result<MapRef> {
    if let Some(existing) = parent.get(txn, key) {
        MapRef::try_from(existing)
            .map_err(|e| anyhow::anyhow!("Failed to convert to MapRef: {:?}", e))
    } else {
        // Create empty map using temporary key-value pair
        let empty_prelim: MapPrelim = MapPrelim::from([("_", Any::Null)]);
        let map = parent.insert(txn, key, empty_prelim);
        // Remove the temporary key
        map.remove(txn, "_");
        Ok(map)
    }
}

/// Helper to set a string value in a map with proper error handling
pub fn set_map_string(
    txn: &mut TransactionMut,
    map: &MapRef,
    key: &str,
    value: impl Into<String>,
) -> Result<()> {
    map.insert(txn, key, value.into());
    Ok(())
}

/// Helper to get a string value from a map
pub fn get_map_string(txn: &impl ReadTxn, map: &MapRef, key: &str) -> Result<Option<String>> {
    match map.get(txn, key) {
        Some(value) => Ok(Some(value.to_string(txn))),
        None => Ok(None),
    }
}

/// Helper to set an i64 value in a map
pub fn set_map_i64(txn: &mut TransactionMut, map: &MapRef, key: &str, value: i64) -> Result<()> {
    map.insert(txn, key, value);
    Ok(())
}

/// Helper to get an i64 value from a map
pub fn get_map_i64(txn: &impl ReadTxn, map: &MapRef, key: &str) -> Result<Option<i64>> {
    match map.get(txn, key) {
        Some(value) => Ok(i64::try_from(value).ok()),
        None => Ok(None),
    }
}

/// Helper to set a boolean value in a map
pub fn set_map_bool(txn: &mut TransactionMut, map: &MapRef, key: &str, value: bool) -> Result<()> {
    map.insert(txn, key, value);
    Ok(())
}

/// Helper to get a boolean value from a map
pub fn get_map_bool(txn: &impl ReadTxn, map: &MapRef, key: &str) -> Result<Option<bool>> {
    match map.get(txn, key) {
        Some(value) => Ok(bool::try_from(value).ok()),
        None => Ok(None),
    }
}

/// Member/User document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberDocument {
    pub id: String,
    /// Hex-encoded ML-DSA-65 public key (THE identity)
    pub pubkey_hex: String,
    /// User-chosen display name (shown in UI)
    pub display_name: String,
    pub email: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: i64,
    pub personal_disk_id: String,
    pub website_root: Option<String>,
}

impl CrdtDocument for MemberDocument {
    fn document_type() -> &'static str {
        "member"
    }

    fn create_document(_id: &str) -> Result<Doc> {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();

        // Create main structure
        let empty_map: MapPrelim = MapPrelim::from([("_", Any::Null)]);

        let metadata = root.insert(&mut txn, "metadata", empty_map.clone());
        metadata.remove(&mut txn, "_");

        let organizations = root.insert(&mut txn, "organizations", empty_map.clone());
        organizations.remove(&mut txn, "_");

        let channels = root.insert(&mut txn, "channels", empty_map.clone());
        channels.remove(&mut txn, "_");

        let groups = root.insert(&mut txn, "groups", empty_map);
        groups.remove(&mut txn, "_");

        drop(txn);
        Ok(doc)
    }

    fn from_document(doc: &Doc) -> Result<Self> {
        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        let metadata = root
            .get(&txn, "metadata")
            .context("No metadata in member document")?;
        let metadata_map = MapRef::try_from(metadata)
            .map_err(|e| anyhow::anyhow!("Invalid metadata structure: {:?}", e))?;

        let id = get_map_string(&txn, &metadata_map, "id")?.context("Missing id")?;
        let pubkey_hex =
            get_map_string(&txn, &metadata_map, "pubkey_hex")?.context("Missing pubkey_hex")?;
        let display_name =
            get_map_string(&txn, &metadata_map, "display_name")?.context("Missing display_name")?;
        let email = get_map_string(&txn, &metadata_map, "email")?;
        let bio = get_map_string(&txn, &metadata_map, "bio")?;
        let avatar_url = get_map_string(&txn, &metadata_map, "avatar_url")?;
        let created_at =
            get_map_i64(&txn, &metadata_map, "created_at")?.context("Missing created_at")?;
        let personal_disk_id = get_map_string(&txn, &metadata_map, "personal_disk_id")?
            .context("Missing personal_disk_id")?;
        let website_root = get_map_string(&txn, &metadata_map, "website_root")?;

        drop(txn);
        Ok(Self {
            id,
            pubkey_hex,
            display_name,
            email,
            bio,
            avatar_url,
            created_at,
            personal_disk_id,
            website_root,
        })
    }

    fn update_document(&self, doc: &Doc) -> Result<()> {
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();

        let metadata = get_or_create_map(&mut txn, &root, "metadata")?;

        set_map_string(&mut txn, &metadata, "id", &self.id)?;
        set_map_string(&mut txn, &metadata, "pubkey_hex", &self.pubkey_hex)?;
        set_map_string(&mut txn, &metadata, "display_name", &self.display_name)?;

        if let Some(ref email) = self.email {
            set_map_string(&mut txn, &metadata, "email", email)?;
        }
        if let Some(ref bio) = self.bio {
            set_map_string(&mut txn, &metadata, "bio", bio)?;
        }
        if let Some(ref avatar_url) = self.avatar_url {
            set_map_string(&mut txn, &metadata, "avatar_url", avatar_url)?;
        }

        set_map_i64(&mut txn, &metadata, "created_at", self.created_at)?;
        set_map_string(
            &mut txn,
            &metadata,
            "personal_disk_id",
            &self.personal_disk_id,
        )?;

        if let Some(ref website_root) = self.website_root {
            set_map_string(&mut txn, &metadata, "website_root", website_root)?;
        }

        drop(txn);
        Ok(())
    }

    fn entity_id(&self) -> &str {
        &self.id
    }
}

/// Organization document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationDocument {
    pub id: String,
    /// Hex-encoded ML-DSA-65 public key (THE identity)
    pub pubkey_hex: String,
    /// Organization display name (shown in UI)
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub private_disk_id: String,
    pub public_disk_id: String,
    pub website_root: Option<String>,
}

impl CrdtDocument for OrganizationDocument {
    fn document_type() -> &'static str {
        "org"
    }

    fn create_document(_id: &str) -> Result<Doc> {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();
        let empty_map: MapPrelim = MapPrelim::from([("_", Any::Null)]);

        let metadata = root.insert(&mut txn, "metadata", empty_map.clone());
        metadata.remove(&mut txn, "_");

        let members = root.insert(&mut txn, "members", empty_map);
        members.remove(&mut txn, "_");

        let _channels = root.insert(
            &mut txn,
            "channels",
            ArrayPrelim::from(vec![] as Vec<String>),
        );
        let _projects = root.insert(
            &mut txn,
            "projects",
            ArrayPrelim::from(vec![] as Vec<String>),
        );

        drop(txn);
        Ok(doc)
    }

    fn from_document(doc: &Doc) -> Result<Self> {
        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        let metadata = root
            .get(&txn, "metadata")
            .context("No metadata in org document")?;
        let metadata_map = MapRef::try_from(metadata)
            .map_err(|e| anyhow::anyhow!("Invalid metadata structure: {:?}", e))?;

        let id = get_map_string(&txn, &metadata_map, "id")?.context("Missing id")?;
        let pubkey_hex =
            get_map_string(&txn, &metadata_map, "pubkey_hex")?.context("Missing pubkey_hex")?;
        let name = get_map_string(&txn, &metadata_map, "name")?.context("Missing name")?;
        let description = get_map_string(&txn, &metadata_map, "description")?;
        let created_by =
            get_map_string(&txn, &metadata_map, "created_by")?.context("Missing created_by")?;
        let created_at =
            get_map_i64(&txn, &metadata_map, "created_at")?.context("Missing created_at")?;
        let private_disk_id = get_map_string(&txn, &metadata_map, "private_disk_id")?
            .context("Missing private_disk_id")?;
        let public_disk_id = get_map_string(&txn, &metadata_map, "public_disk_id")?
            .context("Missing public_disk_id")?;
        let website_root = get_map_string(&txn, &metadata_map, "website_root")?;

        drop(txn);
        Ok(Self {
            id,
            pubkey_hex,
            name,
            description,
            created_by,
            created_at,
            private_disk_id,
            public_disk_id,
            website_root,
        })
    }

    fn update_document(&self, doc: &Doc) -> Result<()> {
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();

        let metadata = get_or_create_map(&mut txn, &root, "metadata")?;

        set_map_string(&mut txn, &metadata, "id", &self.id)?;
        set_map_string(&mut txn, &metadata, "pubkey_hex", &self.pubkey_hex)?;
        set_map_string(&mut txn, &metadata, "name", &self.name)?;

        if let Some(ref desc) = self.description {
            set_map_string(&mut txn, &metadata, "description", desc)?;
        }

        set_map_string(&mut txn, &metadata, "created_by", &self.created_by)?;
        set_map_i64(&mut txn, &metadata, "created_at", self.created_at)?;
        set_map_string(
            &mut txn,
            &metadata,
            "private_disk_id",
            &self.private_disk_id,
        )?;
        set_map_string(&mut txn, &metadata, "public_disk_id", &self.public_disk_id)?;

        if let Some(ref website_root) = self.website_root {
            set_map_string(&mut txn, &metadata, "website_root", website_root)?;
        }

        drop(txn);
        Ok(())
    }

    fn entity_id(&self) -> &str {
        &self.id
    }
}

/// Channel document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelDocument {
    pub id: String,
    /// Hex-encoded ML-DSA-65 public key (THE identity)
    pub pubkey_hex: String,
    pub org_id: String,
    /// Channel display name (shown in UI)
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: i64,
    pub private_disk_id: String,
    pub public_disk_id: String,
    pub website_root: Option<String>,
}

impl CrdtDocument for ChannelDocument {
    fn document_type() -> &'static str {
        "channel"
    }

    fn create_document(_id: &str) -> Result<Doc> {
        let doc = Doc::new();
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();
        let empty_map: MapPrelim = MapPrelim::from([("_", Any::Null)]);

        let metadata = root.insert(&mut txn, "metadata", empty_map.clone());
        metadata.remove(&mut txn, "_");

        let members = root.insert(&mut txn, "members", empty_map.clone());
        members.remove(&mut txn, "_");

        let messages = root.insert(&mut txn, "messages", empty_map.clone());
        messages.remove(&mut txn, "_");

        let threads = root.insert(&mut txn, "threads", empty_map);
        threads.remove(&mut txn, "_");

        drop(txn);
        Ok(doc)
    }

    fn from_document(doc: &Doc) -> Result<Self> {
        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        let metadata = root
            .get(&txn, "metadata")
            .context("No metadata in channel document")?;
        let metadata_map = MapRef::try_from(metadata)
            .map_err(|e| anyhow::anyhow!("Invalid metadata structure: {:?}", e))?;

        let id = get_map_string(&txn, &metadata_map, "id")?.context("Missing id")?;
        let pubkey_hex =
            get_map_string(&txn, &metadata_map, "pubkey_hex")?.context("Missing pubkey_hex")?;
        let org_id = get_map_string(&txn, &metadata_map, "org_id")?.context("Missing org_id")?;
        let name = get_map_string(&txn, &metadata_map, "name")?.context("Missing name")?;
        let description = get_map_string(&txn, &metadata_map, "description")?;
        let created_by =
            get_map_string(&txn, &metadata_map, "created_by")?.context("Missing created_by")?;
        let created_at =
            get_map_i64(&txn, &metadata_map, "created_at")?.context("Missing created_at")?;
        let private_disk_id = get_map_string(&txn, &metadata_map, "private_disk_id")?
            .context("Missing private_disk_id")?;
        let public_disk_id = get_map_string(&txn, &metadata_map, "public_disk_id")?
            .context("Missing public_disk_id")?;
        let website_root = get_map_string(&txn, &metadata_map, "website_root")?;

        drop(txn);
        Ok(Self {
            id,
            pubkey_hex,
            org_id,
            name,
            description,
            created_by,
            created_at,
            private_disk_id,
            public_disk_id,
            website_root,
        })
    }

    fn update_document(&self, doc: &Doc) -> Result<()> {
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();

        let metadata = get_or_create_map(&mut txn, &root, "metadata")?;

        set_map_string(&mut txn, &metadata, "id", &self.id)?;
        set_map_string(&mut txn, &metadata, "pubkey_hex", &self.pubkey_hex)?;
        set_map_string(&mut txn, &metadata, "org_id", &self.org_id)?;
        set_map_string(&mut txn, &metadata, "name", &self.name)?;

        if let Some(ref desc) = self.description {
            set_map_string(&mut txn, &metadata, "description", desc)?;
        }

        set_map_string(&mut txn, &metadata, "created_by", &self.created_by)?;
        set_map_i64(&mut txn, &metadata, "created_at", self.created_at)?;
        set_map_string(
            &mut txn,
            &metadata,
            "private_disk_id",
            &self.private_disk_id,
        )?;
        set_map_string(&mut txn, &metadata, "public_disk_id", &self.public_disk_id)?;

        if let Some(ref website_root) = self.website_root {
            set_map_string(&mut txn, &metadata, "website_root", website_root)?;
        }

        drop(txn);
        Ok(())
    }

    fn entity_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_document_roundtrip() {
        let member = MemberDocument {
            id: "member-123".to_string(),
            pubkey_hex: "abcd1234".to_string(),
            display_name: "Alice".to_string(),
            email: Some("alice@example.com".to_string()),
            bio: Some("Software engineer".to_string()),
            avatar_url: None,
            created_at: 1234567890,
            personal_disk_id: "disk-456".to_string(),
            website_root: None,
        };

        let doc = MemberDocument::create_document(&member.id).unwrap();
        member.update_document(&doc).unwrap();

        let loaded = MemberDocument::from_document(&doc).unwrap();

        assert_eq!(member.id, loaded.id);
        assert_eq!(member.pubkey_hex, loaded.pubkey_hex);
        assert_eq!(member.display_name, loaded.display_name);
        assert_eq!(member.email, loaded.email);
    }

    #[test]
    fn test_organization_document_roundtrip() {
        let org = OrganizationDocument {
            id: "org-123".to_string(),
            pubkey_hex: "ef567890".to_string(),
            name: "TechCorp".to_string(),
            description: Some("Our company".to_string()),
            created_by: "member-456".to_string(),
            created_at: 1234567890,
            private_disk_id: "disk-789".to_string(),
            public_disk_id: "disk-790".to_string(),
            website_root: None,
        };

        let doc = OrganizationDocument::create_document(&org.id).unwrap();
        org.update_document(&doc).unwrap();

        let loaded = OrganizationDocument::from_document(&doc).unwrap();

        assert_eq!(org.id, loaded.id);
        assert_eq!(org.pubkey_hex, loaded.pubkey_hex);
        assert_eq!(org.name, loaded.name);
        assert_eq!(org.created_by, loaded.created_by);
    }

    #[test]
    fn test_channel_document_roundtrip() {
        let channel = ChannelDocument {
            id: "channel-123".to_string(),
            pubkey_hex: "1234abcd".to_string(),
            org_id: "org-456".to_string(),
            name: "General".to_string(),
            description: Some("Main channel".to_string()),
            created_by: "member-789".to_string(),
            created_at: 1234567890,
            private_disk_id: "disk-111".to_string(),
            public_disk_id: "disk-222".to_string(),
            website_root: None,
        };

        let doc = ChannelDocument::create_document(&channel.id).unwrap();
        channel.update_document(&doc).unwrap();

        let loaded = ChannelDocument::from_document(&doc).unwrap();

        assert_eq!(channel.id, loaded.id);
        assert_eq!(channel.pubkey_hex, loaded.pubkey_hex);
        assert_eq!(channel.name, loaded.name);
        assert_eq!(channel.org_id, loaded.org_id);
    }
}
