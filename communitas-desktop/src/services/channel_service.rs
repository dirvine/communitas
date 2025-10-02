use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::{Map, Transact};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: i64,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub thread_id: Option<String>,
    pub author_id: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub parent_message_id: String,
    pub channel_id: String,
    pub reply_count: i32,
    pub last_reply_at: Option<i64>,
}

pub struct ChannelService {
    crdt: Arc<CrdtManager>,
}

impl ChannelService {
    pub fn new(crdt: Arc<CrdtManager>) -> Self {
        Self { crdt }
    }

    /// Create a new channel
    pub async fn create_channel(
        &self,
        org_id: &str,
        name: &str,
        description: Option<String>,
        created_by: &str,
    ) -> Result<Channel> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let doc_id = format!("channel:{}", id);

        // Create CRDT document for channel messages
        let doc = yrs::Doc::new();
        let _messages = doc.get_or_insert_array("messages");

        self.crdt
            .save_document(&doc_id, "channel", &id, &doc)
            .await?;

        // Save channel metadata
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO channels (id, org_id, name, description, crdt_doc_id, created_at, created_by)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![id.clone(), org_id, name, description.clone(), doc_id.clone(), now, created_by],
        )
        .await
        .context("Failed to create channel")?;

        Ok(Channel {
            id,
            org_id: org_id.to_string(),
            name: name.to_string(),
            description,
            created_at: now,
            created_by: created_by.to_string(),
        })
    }

    /// Get channel by ID
    pub async fn get_channel(&self, channel_id: &str) -> Result<Option<Channel>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, org_id, name, description, created_at, created_by FROM channels WHERE id = ?",
                params![channel_id],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(Channel {
                id: row.get(0)?,
                org_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                created_by: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all channels in an organization
    pub async fn list_channels(&self, org_id: &str) -> Result<Vec<Channel>> {
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT id, org_id, name, description, created_at, created_by
                 FROM channels WHERE org_id = ? ORDER BY created_at DESC",
                params![org_id],
            )
            .await?;

        let mut channels = Vec::new();
        while let Some(row) = rows.next().await? {
            channels.push(Channel {
                id: row.get(0)?,
                org_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                created_by: row.get(5)?,
            });
        }

        Ok(channels)
    }

    /// Send a message to a channel
    pub async fn send_message(
        &self,
        channel_id: &str,
        author_id: &str,
        content: &str,
        thread_id: Option<String>,
    ) -> Result<Message> {
        let msg_id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();
        let doc_id = format!("channel:{}", channel_id);

        // Load channel CRDT document
        let doc = self.crdt.load_document(&doc_id).await?;

        // Create message object
        let message = Message {
            id: msg_id.clone(),
            channel_id: channel_id.to_string(),
            thread_id: thread_id.clone(),
            author_id: author_id.to_string(),
            content: content.to_string(),
            created_at: now,
            updated_at: None,
        };

        // Add message to CRDT array - scope to drop ArrayRef before await
        {
            let messages = doc.get_or_insert_array("messages");
            let mut txn = doc.transact_mut();

            // Create message map with data using From trait
            use yrs::{Array, MapPrelim};
            use std::collections::HashMap;

            let mut map_entries: HashMap<String, yrs::Any> = HashMap::new();
            map_entries.insert("id".to_string(), msg_id.clone().into());
            map_entries.insert("author_id".to_string(), author_id.to_string().into());
            map_entries.insert("content".to_string(), content.to_string().into());
            map_entries.insert("created_at".to_string(), now.to_string().into());
            if let Some(ref tid) = thread_id {
                map_entries.insert("thread_id".to_string(), tid.clone().into());
            }

            let msg_data = MapPrelim::from(map_entries);
            messages.push_back(&mut txn, msg_data);
        }

        // Save updated CRDT
        self.crdt
            .save_document(&doc_id, "channel", channel_id, &doc)
            .await?;

        // Save message metadata to SQL for indexing
        let db = self.crdt.connection()?;
        db.execute(
            "INSERT INTO messages (id, channel_id, thread_id, author_id, content, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![msg_id, channel_id, thread_id.clone(), author_id, content, now],
        )
        .await
        .context("Failed to save message")?;

        // Update thread reply count if this is a reply
        if let Some(tid) = thread_id {
            db.execute(
                "UPDATE threads SET reply_count = reply_count + 1, last_reply_at = ? WHERE id = ?",
                params![now, tid],
            )
            .await
            .ok(); // Non-critical if thread doesn't exist yet
        }

        Ok(message)
    }

    /// Get messages from a channel
    pub async fn get_messages(
        &self,
        channel_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Message>> {
        let db = self.crdt.connection()?;
        let limit = limit.unwrap_or(100);
        let offset = offset.unwrap_or(0);

        let mut rows = db
            .query(
                "SELECT id, channel_id, thread_id, author_id, content, created_at, updated_at
                 FROM messages
                 WHERE channel_id = ? AND thread_id IS NULL
                 ORDER BY created_at DESC
                 LIMIT ? OFFSET ?",
                params![channel_id, limit, offset],
            )
            .await?;

        let mut messages = Vec::new();
        while let Some(row) = rows.next().await? {
            messages.push(Message {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                thread_id: row.get(2)?,
                author_id: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            });
        }

        Ok(messages)
    }

    /// Create a thread from a message
    pub async fn create_thread(&self, parent_message_id: &str) -> Result<Thread> {
        let db = self.crdt.connection()?;

        // Get parent message channel
        let mut rows = db
            .query(
                "SELECT channel_id FROM messages WHERE id = ?",
                params![parent_message_id],
            )
            .await?;

        let channel_id: String = if let Some(row) = rows.next().await? {
            row.get(0)?
        } else {
            anyhow::bail!("Parent message not found");
        };

        let thread_id = Uuid::new_v4().to_string();

        // Create thread record
        db.execute(
            "INSERT INTO threads (id, parent_message_id, channel_id, reply_count, last_reply_at)
             VALUES (?, ?, ?, 0, NULL)",
            params![thread_id.clone(), parent_message_id, channel_id.clone()],
        )
        .await
        .context("Failed to create thread")?;

        Ok(Thread {
            id: thread_id,
            parent_message_id: parent_message_id.to_string(),
            channel_id,
            reply_count: 0,
            last_reply_at: None,
        })
    }

    /// Get thread replies
    pub async fn get_thread_replies(&self, thread_id: &str) -> Result<Vec<Message>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT id, channel_id, thread_id, author_id, content, created_at, updated_at
                 FROM messages
                 WHERE thread_id = ?
                 ORDER BY created_at ASC",
                params![thread_id],
            )
            .await?;

        let mut messages = Vec::new();
        while let Some(row) = rows.next().await? {
            messages.push(Message {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                thread_id: row.get(2)?,
                author_id: row.get(3)?,
                content: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            });
        }

        Ok(messages)
    }

    /// Add member to channel
    pub async fn add_member(
        &self,
        channel_id: &str,
        user_id: &str,
        role: &str,
    ) -> Result<()> {
        let db = self.crdt.connection()?;
        let now = Utc::now().timestamp();

        db.execute(
            "INSERT OR REPLACE INTO channel_members (channel_id, user_id, role, joined_at)
             VALUES (?, ?, ?, ?)",
            params![channel_id, user_id, role, now],
        )
        .await
        .context("Failed to add channel member")?;

        Ok(())
    }

    /// Remove member from channel
    pub async fn remove_member(&self, channel_id: &str, user_id: &str) -> Result<()> {
        let db = self.crdt.connection()?;

        db.execute(
            "DELETE FROM channel_members WHERE channel_id = ? AND user_id = ?",
            params![channel_id, user_id],
        )
        .await
        .context("Failed to remove channel member")?;

        Ok(())
    }

    /// Get channel members
    pub async fn get_members(&self, channel_id: &str) -> Result<Vec<(String, String)>> {
        let db = self.crdt.connection()?;

        let mut rows = db
            .query(
                "SELECT user_id, role FROM channel_members WHERE channel_id = ?",
                params![channel_id],
            )
            .await?;

        let mut members = Vec::new();
        while let Some(row) = rows.next().await? {
            members.push((row.get(0)?, row.get(1)?));
        }

        Ok(members)
    }

    /// Get CRDT update for sync
    pub async fn get_channel_update(&self, channel_id: &str) -> Result<Vec<u8>> {
        let doc_id = format!("channel:{}", channel_id);
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
    pub async fn apply_channel_update(&self, channel_id: &str, update: &[u8]) -> Result<()> {
        let doc_id = format!("channel:{}", channel_id);
        self.crdt
            .merge_update(&doc_id, "channel", channel_id, update)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_and_get_channel() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        let channel = service
            .create_channel("org-1", "general", Some("General chat".to_string()), "user-1")
            .await
            .unwrap();

        assert_eq!(channel.name, "general");
        assert_eq!(channel.org_id, "org-1");

        let retrieved = service.get_channel(&channel.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "general");
    }

    #[tokio::test]
    async fn test_send_and_get_messages() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        let channel = service
            .create_channel("org-1", "test", None, "user-1")
            .await
            .unwrap();

        let msg = service
            .send_message(&channel.id, "user-1", "Hello!", None)
            .await
            .unwrap();

        assert_eq!(msg.content, "Hello!");

        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello!");
    }
}
