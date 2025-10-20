use super::Backend;
use anyhow::Result;
use communitas_core::crdt::{EntityType, MessageContent};
use serde_json::json;

/// Issue representation (stored as messages with structured content)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Issue {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub assignee_id: Option<String>,
    pub reporter_id: String,
    pub created_at: u64,
}

impl Backend {
    /// Create an issue as a message within a project entity
    ///
    /// Issues are implemented as messages with structured JSON content.
    /// This allows them to be synced via the MessageService CRDT.
    pub async fn create_issue(
        &mut self,
        project_id: String,
        title: String,
        description: Option<String>,
        priority: String,
    ) -> Result<String> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot create issue"
            ));
        }

        let ctx = self.context()?;

        // Create issue data structure
        let issue = Issue {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.clone(),
            title: title.clone(),
            description: description.clone(),
            status: "backlog".to_string(),
            priority,
            assignee_id: None,
            reporter_id: ctx.four_words.clone(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        // Serialize issue as JSON
        let issue_json = serde_json::to_string(&issue)
            .map_err(|e| anyhow::anyhow!("Failed to serialize issue: {}", e))?;

        // Create message content with issue data
        let content = MessageContent {
            text: issue_json,
            author: ctx.display_name.clone(),
            attachments: None,
        };

        // Send issue as message to project entity
        let _message = ctx
            .message_service
            .send_message(project_id.clone(), EntityType::Project, content, None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create issue: {}", e))?;

        let issue_id = issue.id.clone();

        // Publish IssueCreated event
        self.publish_event(super::events::BackendEvent::EntityCreated {
            entity_id: issue_id.clone(),
            entity_type: EntityType::Project,
            name: title,
        })
        .await;

        Ok(issue_id)
    }

    /// List issues for a project
    ///
    /// Retrieves messages from the project and parses those that are issues.
    pub async fn list_issues(&mut self, project_id: String) -> Result<Vec<serde_json::Value>> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot list issues"
            ));
        }

        let ctx = self.context_mut()?;

        // Get all messages for the project
        let sync_response = ctx
            .message_service
            .get_entity_messages(project_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get project messages: {}", e))?;

        // Parse messages that are issues
        let mut issues = Vec::new();
        for message in sync_response.messages {
            // Try to parse message content as Issue
            if let Ok(issue) = serde_json::from_str::<Issue>(&message.content.text) {
                // Convert to JSON value for flexibility
                issues.push(json!({
                    "id": issue.id,
                    "project_id": issue.project_id,
                    "title": issue.title,
                    "description": issue.description,
                    "status": issue.status,
                    "priority": issue.priority,
                    "assignee_id": issue.assignee_id,
                    "reporter_id": issue.reporter_id,
                    "created_at": issue.created_at,
                }));
            }
        }

        Ok(issues)
    }

    /// Update issue status
    ///
    /// Finds the issue message and updates its status field.
    /// This is done by sending a new message with updated issue data.
    pub async fn update_issue_status(&mut self, issue_id: String, _status: String) -> Result<()> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot update issue"
            ));
        }

        let _ctx = self.context_mut()?;

        // First, we need to find the project this issue belongs to
        // This requires searching through all project messages
        // For now, return an error with a message about the limitation
        // In a full implementation, we'd maintain an issue index

        Err(anyhow::anyhow!(
            "Issue status update requires project_id. Use update_issue_status_in_project() instead. Issue ID: {}",
            issue_id
        ))
    }

    /// Update issue status within a known project
    ///
    /// This is the preferred method when the project_id is known.
    pub async fn update_issue_status_in_project(
        &mut self,
        project_id: String,
        issue_id: String,
        new_status: String,
    ) -> Result<()> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot update issue"
            ));
        }

        let ctx = self.context_mut()?;

        // Get all messages for the project
        let sync_response = ctx
            .message_service
            .get_entity_messages(project_id.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get project messages: {}", e))?;

        // Find the issue
        for message in sync_response.messages {
            if let Ok(mut issue) = serde_json::from_str::<Issue>(&message.content.text)
                && issue.id == issue_id
            {
                // Update status
                issue.status = new_status.clone();

                // Serialize updated issue
                let issue_json = serde_json::to_string(&issue)
                    .map_err(|e| anyhow::anyhow!("Failed to serialize issue: {}", e))?;

                // Create new message with updated issue
                let content = MessageContent {
                    text: issue_json,
                    author: ctx.display_name.clone(),
                    attachments: None,
                };

                // Send updated issue
                ctx.message_service
                    .send_message(
                        project_id,
                        EntityType::Project,
                        content,
                        Some(message.metadata.id),
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to update issue: {}", e))?;

                return Ok(());
            }
        }

        Err(anyhow::anyhow!("Issue not found: {}", issue_id))
    }
}
