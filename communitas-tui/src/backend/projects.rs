use super::Backend;
use anyhow::Result;
use communitas_core::crdt::EntityType;
use serde_json::json;

impl Backend {
    /// Create a new project entity
    ///
    /// Projects are first-class entities in the CRDT system.
    /// They can contain issues (as messages) and have members.
    pub async fn create_project(
        &mut self,
        org_id: String,
        name: String,
        description: Option<String>,
    ) -> Result<String> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot create project"
            ));
        }

        let ctx = self.context()?;

        // Create project entity via EntityService
        let project = ctx
            .entity_service
            .create_entity(
                name.clone(),
                EntityType::Project,
                description.clone(),
                ctx.four_words.clone(),
                vec![], // Initial members (empty for now, can be added later)
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create project: {}", e))?;

        let project_id = project.id.clone();

        // Publish ProjectCreated event
        self.publish_event(super::events::BackendEvent::EntityCreated {
            entity_id: project_id.clone(),
            entity_type: EntityType::Project,
            name: name.clone(),
        })
        .await;

        tracing::info!(
            "Created project '{}' (id: {}) in org {}",
            name,
            project_id,
            org_id
        );

        Ok(project_id)
    }

    /// List all projects
    ///
    /// Note: Currently returns all Project entities. In a full implementation,
    /// this would filter by org_id. For now, org_id is accepted but not used.
    pub async fn list_projects(&mut self, _org_id: String) -> Result<Vec<serde_json::Value>> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot list projects"
            ));
        }

        let ctx = self.context()?;

        // Get all entities
        let entities = ctx
            .entity_service
            .list_entities()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list entities: {}", e))?;

        // Filter for Project entities and convert to JSON
        let projects: Vec<serde_json::Value> = entities
            .into_iter()
            .filter(|e| e.entity_type == EntityType::Project)
            .map(|project| {
                json!({
                    "id": project.id,
                    "name": project.name,
                    "description": project.description,
                    "entity_type": "Project",
                    "members": project.members,
                    "created_at": project.created_at,
                    "created_by": project.created_by,
                })
            })
            .collect();

        Ok(projects)
    }

    /// Get a specific project by ID
    pub async fn get_project(&self, project_id: &str) -> Result<serde_json::Value> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot get project"
            ));
        }

        let ctx = self.context()?;

        // Get project entity
        let project = ctx
            .entity_service
            .get_entity(project_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get project {}: {}", project_id, e))?;

        // Verify it's a project
        if project.entity_type != EntityType::Project {
            return Err(anyhow::anyhow!(
                "Entity {} is not a project (type: {:?})",
                project_id,
                project.entity_type
            ));
        }

        // Convert to JSON
        Ok(json!({
            "id": project.id,
            "name": project.name,
            "description": project.description,
            "entity_type": "Project",
            "members": project.members,
            "created_at": project.created_at,
            "created_by": project.created_by,
        }))
    }

    /// Add a member to a project
    pub async fn add_project_member(
        &mut self,
        project_id: String,
        member_four_words: String,
    ) -> Result<()> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot add member"
            ));
        }

        let ctx = self.context()?;

        // Add member via EntityService
        ctx.entity_service
            .add_member(
                EntityType::Project,
                &project_id,
                &member_four_words,
                "member",
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to add member: {}", e))?;

        // Publish MemberAdded event
        self.publish_event(super::events::BackendEvent::MemberAdded {
            entity_id: project_id,
            entity_type: EntityType::Project,
            member_id: member_four_words,
        })
        .await;

        Ok(())
    }
}
