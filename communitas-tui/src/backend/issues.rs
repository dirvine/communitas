use super::Backend;
use anyhow::Result;

impl Backend {
    /// Create an issue (stub - requires org_commands integration)
    pub async fn create_issue(
        &mut self,
        _project_id: String,
        _title: String,
        _description: Option<String>,
        _priority: String,
    ) -> Result<String> {
        // TODO: Implement issue creation when org_commands are integrated
        Err(anyhow::anyhow!(
            "Issue creation not yet implemented in CoreContext"
        ))
    }

    /// List issues for a project (stub - requires org_commands integration)
    pub async fn list_issues(&mut self, _project_id: String) -> Result<Vec<serde_json::Value>> {
        // TODO: Implement issue listing when org_commands are integrated
        Ok(Vec::new())
    }

    /// Update issue status (stub - requires org_commands integration)
    pub async fn update_issue_status(
        &mut self,
        _issue_id: String,
        _status: String,
    ) -> Result<()> {
        // TODO: Implement issue status update when org_commands are integrated
        Err(anyhow::anyhow!(
            "Issue status update not yet implemented in CoreContext"
        ))
    }
}
