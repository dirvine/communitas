// Copyright (c) 2025 Saorsa Labs Limited
// Licensed under the AGPL-3.0 license

//! Entity-aware storage system for Communitas
//!
//! Provides standardized storage structure for all entity types:
//! Person, Organization, Project, Group, Channel

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::local_storage::LocalStorageManager;

/// Entity types supported by the storage system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EntityType {
    Person,
    Organization,
    Project,
    Group,
    Channel,
}

/// Four-word identity for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FourWordIdentity {
    pub words: [String; 4],
    pub public_key: Vec<u8>,
    pub private_key: Option<Vec<u8>>,
}

impl FourWordIdentity {
    pub fn four_words(&self) -> String {
        self.words.join("-")
    }

    pub fn from_words(words: &str) -> Result<Self> {
        let parts: Vec<&str> = words.split('-').collect();
        if parts.len() != 4 {
            anyhow::bail!("Four-word address must have exactly 4 words separated by hyphens");
        }

        Ok(Self {
            words: [
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].to_string(),
            ],
            public_key: vec![], // TODO: Generate actual keys
            private_key: None,
        })
    }
}

/// Entity metadata and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    pub id: String,
    pub entity_type: EntityType,
    pub four_word_address: String,
    pub display_name: String,
    pub description: Option<String>,
    pub identity: FourWordIdentity,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub storage_quota: u64,
    pub used_storage: u64,
}

/// Standardized directory structure for all entities
#[derive(Debug, Clone)]
pub struct EntityStorageStructure {
    pub root: PathBuf,
    pub web: PathBuf,      // Public markdown website
    pub shared: PathBuf,   // Collaborative files
    pub metadata: PathBuf, // Hidden DHT metadata
    pub private: PathBuf,  // Private files (owner only)
}

impl EntityStorageStructure {
    pub fn new<P: AsRef<Path>>(entity_root: P) -> Self {
        let root = entity_root.as_ref().to_path_buf();
        Self {
            web: root.join("web"),
            shared: root.join("shared"),
            metadata: root.join(".metadata"),
            private: root.join("private"),
            root,
        }
    }

    pub async fn create_directories(&self) -> Result<()> {
        let directories = [
            &self.root,
            &self.web,
            &self.shared,
            &self.metadata,
            &self.private,
        ];

        for dir in directories.iter() {
            tokio::fs::create_dir_all(dir)
                .await
                .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
        }

        // Create subdirectories
        self.create_web_subdirectories().await?;
        self.create_shared_subdirectories().await?;
        self.create_metadata_subdirectories().await?;
        self.create_private_subdirectories().await?;

        debug!(
            "Created entity storage structure at {}",
            self.root.display()
        );
        Ok(())
    }

    async fn create_web_subdirectories(&self) -> Result<()> {
        let subdirs = ["assets", "pages"];
        for subdir in &subdirs {
            tokio::fs::create_dir_all(self.web.join(subdir))
                .await
                .with_context(|| format!("Failed to create web subdirectory: {}", subdir))?;
        }
        Ok(())
    }

    async fn create_shared_subdirectories(&self) -> Result<()> {
        let subdirs = ["documents", "projects", "resources"];
        for subdir in &subdirs {
            tokio::fs::create_dir_all(self.shared.join(subdir))
                .await
                .with_context(|| format!("Failed to create shared subdirectory: {}", subdir))?;
        }
        Ok(())
    }

    async fn create_metadata_subdirectories(&self) -> Result<()> {
        // Metadata files are stored at root level of .metadata/
        Ok(())
    }

    async fn create_private_subdirectories(&self) -> Result<()> {
        let subdirs = ["personal"];
        for subdir in &subdirs {
            tokio::fs::create_dir_all(self.private.join(subdir))
                .await
                .with_context(|| format!("Failed to create private subdirectory: {}", subdir))?;
        }
        Ok(())
    }

    /// Initialize the home.md file in the web directory with entity-specific content
    pub async fn initialize_home_markdown(&self, entity_info: &EntityInfo) -> Result<()> {
        let home_file = self.web.join("home.md");

        // Only create if it doesn't exist (don't overwrite existing content)
        if !home_file.exists() {
            let default_content = self.generate_default_markdown(entity_info);

            tokio::fs::write(&home_file, default_content)
                .await
                .with_context(|| {
                    format!("Failed to create home.md file: {}", home_file.display())
                })?;

            debug!(
                "Created default home.md file for {} at {}",
                entity_info.four_word_address,
                home_file.display()
            );
        }

        Ok(())
    }

    fn generate_default_markdown(&self, entity_info: &EntityInfo) -> String {
        match entity_info.entity_type {
            EntityType::Person => format!(
                r#"# Welcome to {}'s Digital Space
Address: {}

## About Me
{}

## My Content
- [Personal Blog](./pages/blog.md)
- [Projects](./pages/projects.md)
- [Resources](./pages/resources.md)

## Connect With Me
- Send a direct message
- Join my conversations
- Collaborate on shared files

## About This Space
This is my personal corner of the markdown internet. All content here is:
- Stored across trusted peers using Reed-Solomon encoding
- Cryptographically signed with my four-word identity
- Accessible through decentralized routing
- Self-hosted without traditional servers

Start exploring or [contact me directly]({}/contact)!
"#,
                entity_info.display_name,
                entity_info.four_word_address,
                entity_info
                    .description
                    .as_deref()
                    .unwrap_or("Welcome to my digital space."),
                entity_info.four_word_address
            ),

            EntityType::Organization => format!(
                r#"# {}
Address: {}

## About Our Organization
{}

## What We Do
- [Our Mission](./pages/mission.md)
- [Team Members](./pages/team.md)
- [Current Projects](./pages/projects.md)

## Get Involved
- [Join Our Team](./pages/careers.md)
- [Partner With Us](./pages/partnerships.md)
- [Contact Information](./pages/contact.md)

## Organization Resources
- [Public Documents](./shared/documents/)
- [Project Archives](./shared/projects/)
- [Resource Library](./shared/resources/)

## Connect With Us
Visit us at [{}]({}) for more information.
"#,
                entity_info.display_name,
                entity_info.four_word_address,
                entity_info
                    .description
                    .as_deref()
                    .unwrap_or("Welcome to our organization."),
                entity_info.four_word_address,
                entity_info.four_word_address
            ),

            EntityType::Project => format!(
                r#"# Project: {}
Address: {}

## Project Overview
{}

## Project Resources
- [Documentation](./pages/documentation.md)
- [Progress Updates](./pages/updates.md)
- [Team Information](./pages/team.md)

## Collaboration
- [Shared Documents](./shared/documents/)
- [Project Files](./shared/projects/)
- [Resources](./shared/resources/)

## Get Involved
- [Contributing Guidelines](./pages/contributing.md)
- [Issue Tracker](./pages/issues.md)
- [Discussion Forum](./pages/discussions.md)

## Project Details
This project space provides collaborative tools for team coordination and resource sharing.
All project data is distributed across team members for resilience and availability.

Visit [{}]({}) for the latest updates.
"#,
                entity_info.display_name,
                entity_info.four_word_address,
                entity_info
                    .description
                    .as_deref()
                    .unwrap_or("Welcome to our project space."),
                entity_info.four_word_address,
                entity_info.four_word_address
            ),

            EntityType::Group => format!(
                r#"# Group: {}
Address: {}

## About This Group
{}

## Group Activities
- [Discussions](./pages/discussions.md)
- [Shared Resources](./shared/resources/)
- [Group Events](./pages/events.md)

## Member Area
- [Member Directory](./pages/members.md)
- [Group Guidelines](./pages/guidelines.md)
- [Shared Files](./shared/documents/)

## Communication
- Group chat and messaging
- Collaborative document editing
- File sharing with Reed-Solomon distribution

## Join Us
This group uses four-word identity addressing for secure, decentralized communication.
All group data is distributed across member devices for privacy and resilience.

Contact us at [{}]({}) to learn more.
"#,
                entity_info.display_name,
                entity_info.four_word_address,
                entity_info
                    .description
                    .as_deref()
                    .unwrap_or("Welcome to our group space."),
                entity_info.four_word_address,
                entity_info.four_word_address
            ),

            EntityType::Channel => format!(
                r#"# Channel: {}
Address: {}

## Channel Information
{}

## Channel Content
- [Recent Discussions](./pages/recent.md)
- [Channel Archive](./pages/archive.md)
- [Shared Resources](./shared/resources/)

## Participation
- [Channel Guidelines](./pages/guidelines.md)
- [How to Contribute](./pages/contributing.md)
- [Moderation Policy](./pages/moderation.md)

## Channel Features
- Real-time messaging with end-to-end encryption
- File sharing with automatic distribution
- Collaborative document editing
- Voice and video communication

## Access
Join the conversation at [{}]({}) using your four-word identity.
All channel data is cryptographically secured and distributed for privacy.
"#,
                entity_info.display_name,
                entity_info.four_word_address,
                entity_info
                    .description
                    .as_deref()
                    .unwrap_or("Welcome to this communication channel."),
                entity_info.four_word_address,
                entity_info.four_word_address
            ),
        }
    }
}

/// Entity storage manager that integrates with LocalStorageManager
#[derive(Debug)]
pub struct EntityStorageManager {
    local_storage: Arc<LocalStorageManager>,
    entities: Arc<RwLock<HashMap<String, EntityInfo>>>,
    storage_root: PathBuf,
}

impl EntityStorageManager {
    pub async fn new(
        local_storage: Arc<LocalStorageManager>,
        storage_root: PathBuf,
    ) -> Result<Self> {
        tokio::fs::create_dir_all(&storage_root)
            .await
            .context("Failed to create entity storage root")?;

        let manager = Self {
            local_storage,
            entities: Arc::new(RwLock::new(HashMap::new())),
            storage_root,
        };

        // Load existing entities
        manager.load_entities().await?;

        info!(
            "Entity storage manager initialized at {}",
            manager.storage_root.display()
        );
        Ok(manager)
    }

    /// Create a new entity with standardized storage structure
    pub async fn create_entity(
        &self,
        entity_type: EntityType,
        four_word_address: String,
        display_name: String,
        description: Option<String>,
    ) -> Result<EntityInfo> {
        let entity_id = Uuid::new_v4().to_string();
        let identity = FourWordIdentity::from_words(&four_word_address)?;

        let entity_info = EntityInfo {
            id: entity_id.clone(),
            entity_type: entity_type.clone(),
            four_word_address: four_word_address.clone(),
            display_name,
            description,
            identity,
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
            storage_quota: 1024 * 1024 * 1024, // 1GB default
            used_storage: 0,
        };

        // Create entity directory structure
        let entity_dir = self.storage_root.join(&entity_id);
        let structure = EntityStorageStructure::new(&entity_dir);
        structure.create_directories().await?;
        structure.initialize_home_markdown(&entity_info).await?;

        // Store entity metadata
        self.save_entity_metadata(&entity_info).await?;

        // Add to in-memory registry
        {
            let mut entities = self.entities.write().await;
            entities.insert(entity_id.clone(), entity_info.clone());
        }

        info!(
            "Created {} entity: {} ({})",
            entity_type_name(&entity_type),
            four_word_address,
            entity_id
        );

        Ok(entity_info)
    }

    /// Get entity by four-word address
    pub async fn get_entity_by_address(&self, four_word_address: &str) -> Option<EntityInfo> {
        let entities = self.entities.read().await;
        entities
            .values()
            .find(|entity| entity.four_word_address == four_word_address)
            .cloned()
    }

    /// Get entity by ID
    pub async fn get_entity_by_id(&self, entity_id: &str) -> Option<EntityInfo> {
        let entities = self.entities.read().await;
        entities.get(entity_id).cloned()
    }

    /// List entities by type
    pub async fn list_entities_by_type(&self, entity_type: &EntityType) -> Vec<EntityInfo> {
        let entities = self.entities.read().await;
        entities
            .values()
            .filter(|entity| &entity.entity_type == entity_type)
            .cloned()
            .collect()
    }

    /// Get storage structure for an entity
    pub fn get_entity_storage_structure(&self, entity_id: &str) -> EntityStorageStructure {
        let entity_dir = self.storage_root.join(entity_id);
        EntityStorageStructure::new(entity_dir)
    }

    /// Store a file in an entity's storage
    pub async fn store_entity_file(
        &self,
        entity_id: &str,
        relative_path: &str,
        content: &[u8],
    ) -> Result<()> {
        let structure = self.get_entity_storage_structure(entity_id);
        let file_path = structure.root.join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&file_path, content)
            .await
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        // Update used storage
        self.update_entity_storage_usage(entity_id, content.len() as i64)
            .await?;

        debug!(
            "Stored file for entity {}: {} ({} bytes)",
            entity_id,
            relative_path,
            content.len()
        );
        Ok(())
    }

    /// Retrieve a file from an entity's storage
    pub async fn retrieve_entity_file(
        &self,
        entity_id: &str,
        relative_path: &str,
    ) -> Result<Vec<u8>> {
        let structure = self.get_entity_storage_structure(entity_id);
        let file_path = structure.root.join(relative_path);

        tokio::fs::read(&file_path)
            .await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))
    }

    /// Update entity metadata
    pub async fn update_entity_metadata(&self, entity_info: &EntityInfo) -> Result<()> {
        let mut updated_info = entity_info.clone();
        updated_info.last_modified = chrono::Utc::now();

        self.save_entity_metadata(&updated_info).await?;

        // Update in-memory registry
        {
            let mut entities = self.entities.write().await;
            entities.insert(entity_info.id.clone(), updated_info);
        }

        Ok(())
    }

    // Private helper methods

    async fn load_entities(&self) -> Result<()> {
        let metadata_dir = self.storage_root.join("metadata");
        if !metadata_dir.exists() {
            return Ok(());
        }

        let mut dir_reader = tokio::fs::read_dir(&metadata_dir).await?;
        let mut entities = self.entities.write().await;

        while let Some(entry) = dir_reader.next_entry().await? {
            if let Some(extension) = entry.path().extension() {
                if extension == "json" {
                    let content = tokio::fs::read_to_string(entry.path()).await?;
                    if let Ok(entity_info) = serde_json::from_str::<EntityInfo>(&content) {
                        entities.insert(entity_info.id.clone(), entity_info);
                    }
                }
            }
        }

        info!("Loaded {} entities from storage", entities.len());
        Ok(())
    }

    async fn save_entity_metadata(&self, entity_info: &EntityInfo) -> Result<()> {
        let metadata_dir = self.storage_root.join("metadata");
        tokio::fs::create_dir_all(&metadata_dir).await?;

        let metadata_file = metadata_dir.join(format!("{}.json", entity_info.id));
        let metadata_json = serde_json::to_string_pretty(entity_info)?;

        tokio::fs::write(&metadata_file, metadata_json)
            .await
            .with_context(|| {
                format!(
                    "Failed to save entity metadata: {}",
                    metadata_file.display()
                )
            })?;

        Ok(())
    }

    async fn update_entity_storage_usage(&self, entity_id: &str, size_delta: i64) -> Result<()> {
        let mut entities = self.entities.write().await;
        if let Some(entity) = entities.get_mut(entity_id) {
            if size_delta < 0 {
                entity.used_storage = entity.used_storage.saturating_sub((-size_delta) as u64);
            } else {
                entity.used_storage += size_delta as u64;
            }
            entity.last_modified = chrono::Utc::now();
        }
        Ok(())
    }
}

fn entity_type_name(entity_type: &EntityType) -> &'static str {
    match entity_type {
        EntityType::Person => "Person",
        EntityType::Organization => "Organization",
        EntityType::Project => "Project",
        EntityType::Group => "Group",
        EntityType::Channel => "Channel",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_entity_storage_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let local_storage =
            Arc::new(LocalStorageManager::new(temp_dir.path().join("local"), 1024 * 1024).await?);

        let entity_storage =
            EntityStorageManager::new(local_storage, temp_dir.path().join("entities")).await?;

        let entity = entity_storage
            .create_entity(
                EntityType::Person,
                "ocean-forest-mountain-star".to_string(),
                "Alice".to_string(),
                Some("Software developer".to_string()),
            )
            .await?;

        assert_eq!(entity.four_word_address, "ocean-forest-mountain-star");
        assert_eq!(entity.display_name, "Alice");
        assert_eq!(entity.entity_type, EntityType::Person);

        // Verify directory structure was created
        let structure = entity_storage.get_entity_storage_structure(&entity.id);
        assert!(structure.web.exists());
        assert!(structure.shared.exists());
        assert!(structure.metadata.exists());
        assert!(structure.private.exists());

        // Verify home.md was created
        let home_file = structure.web.join("home.md");
        assert!(home_file.exists());
        let content = tokio::fs::read_to_string(&home_file).await?;
        assert!(content.contains("Alice's Digital Space"));

        Ok(())
    }

    #[tokio::test]
    async fn test_entity_file_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let local_storage =
            Arc::new(LocalStorageManager::new(temp_dir.path().join("local"), 1024 * 1024).await?);

        let entity_storage =
            EntityStorageManager::new(local_storage, temp_dir.path().join("entities")).await?;

        let entity = entity_storage
            .create_entity(
                EntityType::Organization,
                "company-blue-tech-labs".to_string(),
                "Blue Tech Labs".to_string(),
                None,
            )
            .await?;

        // Store a file
        let file_content = b"Hello, world!";
        entity_storage
            .store_entity_file(&entity.id, "web/pages/about.md", file_content)
            .await?;

        // Retrieve the file
        let retrieved = entity_storage
            .retrieve_entity_file(&entity.id, "web/pages/about.md")
            .await?;

        assert_eq!(retrieved, file_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_entity_lookup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let local_storage =
            Arc::new(LocalStorageManager::new(temp_dir.path().join("local"), 1024 * 1024).await?);

        let entity_storage =
            EntityStorageManager::new(local_storage, temp_dir.path().join("entities")).await?;

        let entity = entity_storage
            .create_entity(
                EntityType::Project,
                "project-quantum-secure".to_string(),
                "Quantum Security Project".to_string(),
                Some("Post-quantum cryptography research".to_string()),
            )
            .await?;

        // Test lookup by address
        let found_by_address = entity_storage
            .get_entity_by_address("project-quantum-secure")
            .await;
        assert!(found_by_address.is_some());
        assert_eq!(found_by_address.unwrap().id, entity.id);

        // Test lookup by ID
        let found_by_id = entity_storage.get_entity_by_id(&entity.id).await;
        assert!(found_by_id.is_some());
        assert_eq!(
            found_by_id.unwrap().four_word_address,
            "project-quantum-secure"
        );

        Ok(())
    }
}
