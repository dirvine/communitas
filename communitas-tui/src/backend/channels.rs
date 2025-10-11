use super::Backend;
use anyhow::Result;
use communitas_core::crdt::EntityType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Simple entity for tracking conversations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub members: Vec<String>, // Four-word addresses of members
}

/// Simple entity manager with persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityManager {
    entities: HashMap<String, Entity>,
    #[serde(skip)]
    storage_path: PathBuf,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            storage_path: PathBuf::new(),
        }
    }

    /// Load entity manager from storage
    pub async fn load(data_dir: &Path) -> Result<Self> {
        let storage_path = data_dir.join("entities.json");

        if storage_path.exists() {
            let data = fs::read_to_string(&storage_path).await?;
            let mut manager: EntityManager = serde_json::from_str(&data)?;
            manager.storage_path = storage_path;
            tracing::info!("Loaded {} entities from storage", manager.entities.len());
            Ok(manager)
        } else {
            tracing::info!("No existing entities, starting with empty EntityManager");
            Ok(Self {
                entities: HashMap::new(),
                storage_path,
            })
        }
    }

    /// Save entity manager to storage
    pub async fn save(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self)?;
        let mut file = fs::File::create(&self.storage_path).await?;
        file.write_all(data.as_bytes()).await?;
        file.sync_all().await?;
        tracing::debug!("Saved {} entities to storage", self.entities.len());
        Ok(())
    }

    pub fn create_entity(&mut self, name: String, entity_type: EntityType, members: Vec<String>) -> Entity {
        let id = uuid::Uuid::new_v4().to_string();
        let entity = Entity {
            id: id.clone(),
            name,
            entity_type,
            members,
        };
        self.entities.insert(id, entity.clone());
        entity
    }

    pub fn get_entity(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    pub fn list_entities(&self) -> Vec<Entity> {
        self.entities.values().cloned().collect()
    }

    pub fn add_member(&mut self, entity_id: &str, member_four_words: String) -> Result<()> {
        let entity = self
            .entities
            .get_mut(entity_id)
            .ok_or_else(|| anyhow::anyhow!("Entity not found"))?;

        if !entity.members.contains(&member_four_words) {
            entity.members.push(member_four_words);
        }
        Ok(())
    }
}

impl Backend {
    /// Create a new entity (contact, group, channel, etc.)
    pub fn create_entity(
        &mut self,
        name: String,
        entity_type: EntityType,
        members: Vec<String>,
    ) -> Result<Entity> {
        let entity = self.entity_manager.create_entity(name, entity_type, members);

        // Save to disk asynchronously (fire and forget for now)
        let manager = self.entity_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.save().await {
                tracing::error!("Failed to save entities: {}", e);
            }
        });

        Ok(entity)
    }

    /// Get list of entities
    pub fn get_entities(&self) -> Result<Vec<Entity>> {
        Ok(self.entity_manager.list_entities())
    }

    /// Get entity by ID
    pub fn get_entity(&self, entity_id: &str) -> Result<Entity> {
        self.entity_manager
            .get_entity(entity_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Entity not found: {}", entity_id))
    }

    /// Add member to entity
    pub fn add_entity_member(&mut self, entity_id: &str, member_four_words: String) -> Result<()> {
        self.entity_manager.add_member(entity_id, member_four_words)?;

        // Save to disk asynchronously
        let manager = self.entity_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = manager.save().await {
                tracing::error!("Failed to save entities: {}", e);
            }
        });

        Ok(())
    }

    // ========================================================================
    // Compatibility methods for existing handlers
    // ========================================================================

    /// Get channels (returns as entities)
    pub async fn get_channels(&mut self) -> Result<Vec<Entity>> {
        self.get_entities()
    }
}
