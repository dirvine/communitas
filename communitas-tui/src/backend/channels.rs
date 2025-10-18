use super::Backend;
use anyhow::Result;
use communitas_core::crdt::EntityType;

/// Simple entity for tracking conversations
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub members: Vec<String>, // Four-word addresses of members
}

impl Backend {
    /// Create a new entity (contact, group, channel, etc.)
    ///
    /// Requires CoreContext to be initialized. Returns error if not initialized.
    pub async fn create_entity(
        &mut self,
        name: String,
        entity_type: communitas_core::crdt::EntityType,
        members: Vec<String>,
    ) -> Result<Entity> {
        // REQUIRE CoreContext - no fallback to legacy EntityManager
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot create entity. Call initialize_core_context() first."
            ));
        }

        let ctx = self.context()?;

        // Create entity via EntityService (CRDT-based)
        let core_entity = ctx.entity_service.create_entity(
            name.clone(),
            entity_type,
            None, // description
            ctx.four_words.clone(), // created_by
            members.clone(),
        ).await?;

        // Convert to TUI Entity type
        let entity = Entity {
            id: core_entity.id.clone(),
            name: core_entity.name.clone(),
            entity_type: core_entity.entity_type,
            members: core_entity.members.clone(),
        };

        // Publish EntityCreated event
        self.publish_event(super::events::BackendEvent::EntityCreated {
            entity_id: entity.id.clone(),
            entity_type: entity.entity_type,
            name: entity.name.clone(),
        })
        .await;

        Ok(entity)
    }

    /// Get list of entities
    ///
    /// Requires CoreContext to be initialized. Returns error if not initialized.
    pub async fn get_entities(&self) -> Result<Vec<Entity>> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot list entities. Call initialize_core_context() first."
            ));
        }

        let ctx = self.context()?;

        // Get entities from EntityService (CRDT-based)
        let core_entities = ctx.entity_service.list_entities().await
            .map_err(|e| anyhow::anyhow!("Failed to list entities: {}", e))?;

        // Convert to TUI Entity type
        let entities = core_entities.into_iter().map(|e| Entity {
            id: e.id,
            name: e.name,
            entity_type: e.entity_type,
            members: e.members,
        }).collect();

        Ok(entities)
    }

    /// Get entity by ID
    ///
    /// Requires CoreContext to be initialized. Returns error if not initialized.
    pub async fn get_entity(&self, entity_id: &str) -> Result<Entity> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot get entity. Call initialize_core_context() first."
            ));
        }

        let ctx = self.context()?;

        // Get entity from EntityService (CRDT-based)
        let core_entity = ctx.entity_service.get_entity(entity_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get entity {}: {}", entity_id, e))?;

        // Convert to TUI Entity type
        Ok(Entity {
            id: core_entity.id,
            name: core_entity.name,
            entity_type: core_entity.entity_type,
            members: core_entity.members,
        })
    }

    /// Add member to entity
    ///
    /// Requires CoreContext to be initialized. Returns error if not initialized.
    pub async fn add_entity_member(
        &mut self,
        entity_type: communitas_core::crdt::EntityType,
        entity_id: &str,
        member_four_words: String,
    ) -> Result<()> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot add member. Call initialize_core_context() first."
            ));
        }

        let ctx = self.context()?;

        // Add member via EntityService (CRDT-based)
        ctx.entity_service
            .add_member(entity_type, entity_id, &member_four_words, "member")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add member: {}", e))?;

        // Publish MemberAdded event
        self.publish_event(super::events::BackendEvent::MemberAdded {
            entity_id: entity_id.to_string(),
            entity_type,
            member_id: member_four_words.clone(),
        })
        .await;

        Ok(())
    }

    /// Remove member from entity
    ///
    /// Requires CoreContext to be initialized. Returns error if not initialized.
    pub async fn remove_entity_member(
        &mut self,
        entity_type: communitas_core::crdt::EntityType,
        entity_id: &str,
        member_four_words: String,
    ) -> Result<()> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot remove member. Call initialize_core_context() first."
            ));
        }

        let ctx = self.context()?;

        // Remove member via EntityService (CRDT-based)
        ctx.entity_service
            .remove_member(entity_type, entity_id, &member_four_words, "system")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to remove member: {}", e))?;

        // Publish MemberRemoved event
        self.publish_event(super::events::BackendEvent::MemberRemoved {
            entity_id: entity_id.to_string(),
            entity_type,
            member_id: member_four_words.clone(),
        })
        .await;

        Ok(())
    }

    // ========================================================================
    // Compatibility methods for existing handlers
    // ========================================================================

    /// Get channels (returns as entities)
    pub async fn get_channels(&mut self) -> Result<Vec<Entity>> {
        self.get_entities().await
    }
}
