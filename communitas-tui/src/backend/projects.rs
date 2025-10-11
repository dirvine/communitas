use super::Backend;
use anyhow::Result;

impl Backend {
    /// Create a new project (stub - requires org_commands integration)
    pub async fn create_project(
        &mut self,
        _org_id: String,
        _name: String,
        _description: Option<String>,
    ) -> Result<String> {
        // TODO: Implement project creation when org_commands are integrated into CoreContext
        Err(anyhow::anyhow!(
            "Project creation not yet implemented in CoreContext"
        ))
    }

    /// List projects (stub - requires org_commands integration)
    pub async fn list_projects(&mut self, _org_id: String) -> Result<Vec<serde_json::Value>> {
        // TODO: Implement project listing when org_commands are integrated
        Ok(Vec::new())
    }
}
