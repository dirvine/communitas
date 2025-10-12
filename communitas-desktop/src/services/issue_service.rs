use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::{Map, Transact};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: i64,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: IssueStatus,
    pub priority: IssuePriority,
    pub assignee_id: Option<String>,
    pub reporter_id: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueStatus {
    Backlog,
    Todo,
    InProgress,
    Done,
    Canceled,
}

impl IssueStatus {
    pub fn as_str(&self) -> &str {
        match self {
            IssueStatus::Backlog => "backlog",
            IssueStatus::Todo => "todo",
            IssueStatus::InProgress => "in-progress",
            IssueStatus::Done => "done",
            IssueStatus::Canceled => "canceled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "backlog" => Ok(IssueStatus::Backlog),
            "todo" => Ok(IssueStatus::Todo),
            "in-progress" => Ok(IssueStatus::InProgress),
            "done" => Ok(IssueStatus::Done),
            "canceled" => Ok(IssueStatus::Canceled),
            _ => anyhow::bail!("Invalid issue status: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssuePriority {
    Urgent,
    High,
    Medium,
    Low,
}

impl IssuePriority {
    pub fn as_str(&self) -> &str {
        match self {
            IssuePriority::Urgent => "urgent",
            IssuePriority::High => "high",
            IssuePriority::Medium => "medium",
            IssuePriority::Low => "low",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "urgent" => Ok(IssuePriority::Urgent),
            "high" => Ok(IssuePriority::High),
            "medium" => Ok(IssuePriority::Medium),
            "low" => Ok(IssuePriority::Low),
            _ => anyhow::bail!("Invalid priority: {}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: String,
    pub issue_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

pub struct IssueService {
    crdt: Arc<CrdtManager>,
}

impl IssueService {
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self { crdt }
    }

    /// Create a new project
    pub async fn create_project(
        &self,
        org_id: &str,
        name: &str,
        description: Option<String>,
        icon: Option<String>,
        color: Option<String>,
        created_by: &str,
    ) -> Result<Project> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO projects (id, org_id, name, description, icon, color, created_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![id.clone(), org_id, name, description.clone(), icon.clone(), color.clone(), now, created_by],
        )
        .await
        .context("Failed to create project")?;

        Ok(Project {
            id,
            org_id: org_id.to_string(),
            name: name.to_string(),
            description,
            icon,
            color,
            created_at: now,
            created_by: created_by.to_string(),
        })
    }

    /// Get project by ID
    pub async fn get_project(&self, project_id: &str) -> Result<Option<Project>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, org_id, name, description, icon, color, created_at, created_by
                 FROM projects WHERE id = ?",
                params![project_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Project {
                id: row.get(0)?,
                org_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                created_by: row.get(7)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all projects in an organization
    pub async fn list_projects(&self, org_id: &str) -> Result<Vec<Project>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, org_id, name, description, icon, color, created_at, created_by
                 FROM projects WHERE org_id = ? ORDER BY created_at DESC",
                params![org_id],
            )
            .await?;

        let mut projects = Vec::new();
        while let Some(row) = rows.next().await? {
            projects.push(Project {
                id: row.get(0)?,
                org_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                icon: row.get(4)?,
                color: row.get(5)?,
                created_at: row.get(6)?,
                created_by: row.get(7)?,
            });
        }

        Ok(projects)
    }

    /// Create an issue
    pub async fn create_issue(
        &self,
        project_id: &str,
        title: &str,
        description: Option<String>,
        priority: IssuePriority,
        reporter_id: &str,
    ) -> Result<Issue> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let doc_id = format!("issue:{}", id);

        // Create CRDT document for issue - scope to drop MapRef before await
        let doc = yrs::Doc::new();
        {
            let issue_map = doc.get_or_insert_map("issue");
            let mut txn = doc.transact_mut();
            issue_map.insert(&mut txn, "title", title);
            issue_map.insert(&mut txn, "status", "backlog");
            issue_map.insert(&mut txn, "priority", priority.as_str());
            if let Some(ref desc) = description {
                issue_map.insert(&mut txn, "description", desc.clone());
            }
        }

        self.crdt.save_document(&doc_id, "issue", &id, &doc).await?;

        // Save issue metadata
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO issues (id, project_id, title, description, status, priority, reporter_id, crdt_doc_id, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                id.clone(),
                project_id,
                title,
                description.clone(),
                "backlog",
                priority.as_str(),
                reporter_id,
                doc_id.clone(),
                now
            ],
        )
        .await
        .context("Failed to create issue")?;

        Ok(Issue {
            id,
            project_id: project_id.to_string(),
            title: title.to_string(),
            description,
            status: IssueStatus::Backlog,
            priority,
            assignee_id: None,
            reporter_id: reporter_id.to_string(),
            created_at: now,
            updated_at: None,
        })
    }

    /// Update issue status
    pub async fn update_status(&self, issue_id: &str, new_status: IssueStatus) -> Result<()> {
        let doc_id = format!("issue:{}", issue_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let now = Utc::now().timestamp();

        // Update CRDT - scope to drop MapRef before await
        {
            let issue_map = doc.get_or_insert_map("issue");
            let mut txn = doc.transact_mut();
            issue_map.insert(&mut txn, "status", new_status.as_str());
        }

        self.crdt
            .save_document(&doc_id, "issue", issue_id, &doc)
            .await?;

        // Update SQL
        let db = self.crdt.connection()?;
        db.execute(
            "UPDATE issues SET status = ?, updated_at = ? WHERE id = ?",
            params![new_status.as_str(), now, issue_id],
        )
        .await
        .context("Failed to update issue status")?;

        Ok(())
    }

    /// Assign issue to user
    pub async fn assign_issue(&self, issue_id: &str, assignee_id: &str) -> Result<()> {
        let doc_id = format!("issue:{}", issue_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let now = Utc::now().timestamp();

        // Update CRDT - scope to drop MapRef before await
        {
            let issue_map = doc.get_or_insert_map("issue");
            let mut txn = doc.transact_mut();
            issue_map.insert(&mut txn, "assignee_id", assignee_id);
        }

        self.crdt
            .save_document(&doc_id, "issue", issue_id, &doc)
            .await?;

        // Update SQL
        let db = self.crdt.connection()?;
        db.execute(
            "UPDATE issues SET assignee_id = ?, updated_at = ? WHERE id = ?",
            params![assignee_id, now, issue_id],
        )
        .await
        .context("Failed to assign issue")?;

        Ok(())
    }

    /// Update issue priority
    pub async fn update_priority(&self, issue_id: &str, priority: IssuePriority) -> Result<()> {
        let doc_id = format!("issue:{}", issue_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let now = Utc::now().timestamp();

        // Update CRDT - scope to drop MapRef before await
        {
            let issue_map = doc.get_or_insert_map("issue");
            let mut txn = doc.transact_mut();
            issue_map.insert(&mut txn, "priority", priority.as_str());
        }

        self.crdt
            .save_document(&doc_id, "issue", issue_id, &doc)
            .await?;

        // Update SQL
        let db = self.crdt.connection()?;
        db.execute(
            "UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?",
            params![priority.as_str(), now, issue_id],
        )
        .await
        .context("Failed to update issue priority")?;

        Ok(())
    }

    /// Get issue by ID
    pub async fn get_issue(&self, issue_id: &str) -> Result<Option<Issue>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, project_id, title, description, status, priority, assignee_id, reporter_id, created_at, updated_at
                 FROM issues WHERE id = ?",
                params![issue_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            let status_str: String = row.get(4)?;
            let priority_str: String = row.get(5)?;

            Ok(Some(Issue {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: IssueStatus::from_str(&status_str)?,
                priority: IssuePriority::from_str(&priority_str)?,
                assignee_id: row.get(6)?,
                reporter_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List issues by project
    pub async fn list_issues(&self, project_id: &str) -> Result<Vec<Issue>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, project_id, title, description, status, priority, assignee_id, reporter_id, created_at, updated_at
                 FROM issues WHERE project_id = ? ORDER BY created_at DESC",
                params![project_id],
            )
            .await?;

        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            let status_str: String = row.get(4)?;
            let priority_str: String = row.get(5)?;

            issues.push(Issue {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: IssueStatus::from_str(&status_str)?,
                priority: IssuePriority::from_str(&priority_str)?,
                assignee_id: row.get(6)?,
                reporter_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            });
        }

        Ok(issues)
    }

    /// List issues by status
    pub async fn list_issues_by_status(
        &self,
        project_id: &str,
        status: IssueStatus,
    ) -> Result<Vec<Issue>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, project_id, title, description, status, priority, assignee_id, reporter_id, created_at, updated_at
                 FROM issues WHERE project_id = ? AND status = ? ORDER BY created_at DESC",
                params![project_id, status.as_str()],
            )
            .await?;

        let mut issues = Vec::new();
        while let Some(row) = rows.next().await? {
            let status_str: String = row.get(4)?;
            let priority_str: String = row.get(5)?;

            issues.push(Issue {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: IssueStatus::from_str(&status_str)?,
                priority: IssuePriority::from_str(&priority_str)?,
                assignee_id: row.get(6)?,
                reporter_id: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            });
        }

        Ok(issues)
    }

    /// Add comment to issue
    pub async fn add_comment(
        &self,
        issue_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<IssueComment> {
        let comment_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO issue_comments (id, issue_id, author_id, content, created_at)
             VALUES (?, ?, ?, ?, ?)",
            params![comment_id.clone(), issue_id, author_id, content, now],
        )
        .await
        .context("Failed to add issue comment")?;

        Ok(IssueComment {
            id: comment_id,
            issue_id: issue_id.to_string(),
            author_id: author_id.to_string(),
            content: content.to_string(),
            created_at: now,
            updated_at: None,
        })
    }

    /// Get comments for an issue
    pub async fn get_comments(&self, issue_id: &str) -> Result<Vec<IssueComment>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, issue_id, author_id, content, created_at, updated_at
                 FROM issue_comments WHERE issue_id = ? ORDER BY created_at ASC",
                params![issue_id],
            )
            .await?;

        let mut comments = Vec::new();
        while let Some(row) = rows.next().await? {
            comments.push(IssueComment {
                id: row.get(0)?,
                issue_id: row.get(1)?,
                author_id: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            });
        }

        Ok(comments)
    }

    /// Get CRDT update for sync
    pub async fn get_issue_update(&self, issue_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("issue:{}", issue_id);
        let doc = self.crdt.load_document(&doc_id).await?;
        let update = {
            use yrs::ReadTxn;
            let sv = yrs::StateVector::default();
            let txn = doc.transact();
            txn.encode_diff_v1(&sv)
        };
        Ok(update)
    }

    /// Apply CRDT update from peer
    pub async fn apply_issue_update(&self, issue_id: &str, update: &[u8]) -> Result<()> {
        let doc_id = format!("issue:{}", issue_id);
        self.crdt
            .merge_update(&doc_id, "issue", issue_id, update)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_project_and_issue() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = IssueService::new(crdt);

        let project = service
            .create_project("org-1", "Test Project", None, None, None, "user-1")
            .await
            .unwrap();

        let issue = service
            .create_issue(
                &project.id,
                "Test Issue",
                None,
                IssuePriority::Medium,
                "user-1",
            )
            .await
            .unwrap();

        assert_eq!(issue.title, "Test Issue");
        assert_eq!(issue.status, IssueStatus::Backlog);
    }

    #[tokio::test]
    async fn test_update_issue_status() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = IssueService::new(crdt);

        let project = service
            .create_project("org-1", "Test", None, None, None, "user-1")
            .await
            .unwrap();

        let issue = service
            .create_issue(&project.id, "Test", None, IssuePriority::High, "user-1")
            .await
            .unwrap();

        service
            .update_status(&issue.id, IssueStatus::InProgress)
            .await
            .unwrap();

        let updated = service.get_issue(&issue.id).await.unwrap().unwrap();
        assert_eq!(updated.status, IssueStatus::InProgress);
    }
}
