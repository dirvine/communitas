use crate::crdt_manager::CrdtManager;
use anyhow::{Context, Result};
use chrono::Utc;
use libsql::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use yrs::Transact;

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

/// Result of applying a diff from another peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedDiffResult {
    /// Number of messages that were materialized to SQL
    pub messages_updated: usize,
    /// Total messages in the channel after sync
    pub total_messages: usize,
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

        // Create CRDT document for channel messages (Map of Maps structure)
        let doc = yrs::Doc::new();
        let _messages = doc.get_or_insert_map("messages");

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

    /// Send a message to a channel (CRDT-first implementation)
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

        tracing::debug!(
            msg_id = %msg_id,
            channel_id = %channel_id,
            author_id = %author_id,
            thread_id = ?thread_id,
            content_len = content.len(),
            "Sending message to channel"
        );

        // Load channel CRDT document
        tracing::trace!(doc_id = %doc_id, "Loading CRDT document");
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

        // Add message to CRDT Map of Maps structure (not Array!)
        // Pattern: Complete all CRDT operations before any await to avoid Send issues
        {
            use crate::crdt_manager::CrdtManager;

            tracing::trace!("Creating CRDT transaction for message insertion");

            // Get the messages Map - this returns a lightweight reference
            // We need to do this before the transaction since it may create one internally
            let messages_map = {
                // Temporary scope to drop any internal transaction immediately
                doc.get_or_insert_map("messages")
            };

            // Now create our main transaction for the actual data manipulation
            let mut txn = doc.transact_mut();

            // Create nested Map for this message
            let msg_map = CrdtManager::get_or_create_nested_map(&messages_map, &mut txn, &msg_id);

            // Set message fields
            CrdtManager::set_map_string(&msg_map, &mut txn, "id", &msg_id);
            CrdtManager::set_map_string(&msg_map, &mut txn, "author_id", author_id);
            CrdtManager::set_map_string(&msg_map, &mut txn, "content", content);
            CrdtManager::set_map_i64(&msg_map, &mut txn, "created_at", now);
            CrdtManager::set_map_bool(&msg_map, &mut txn, "deleted", false);

            if let Some(ref tid) = thread_id {
                CrdtManager::set_map_string(&msg_map, &mut txn, "thread_id", tid);
            }

            tracing::trace!(msg_id = %msg_id, fields_set = 6, "Message added to CRDT with all fields");
        } // Transaction and MapRef dropped here - changes committed, safe to await now

        // Save updated CRDT
        tracing::trace!(doc_id = %doc_id, "Saving CRDT document");
        self.crdt
            .save_document(&doc_id, "channel", channel_id, &doc)
            .await?;

        // Materialize to SQL (rebuilds SQL from CRDT)
        tracing::trace!(msg_id = %msg_id, "Materializing message to SQL");
        self.materialize_message_to_sql_from_doc(&doc, &msg_id, channel_id)
            .await?;

        tracing::info!(
            msg_id = %msg_id,
            channel_id = %channel_id,
            "Message sent successfully"
        );

        Ok(message)
    }

    /// Edit an existing message (CRDT-first implementation)
    pub async fn edit_message(&self, message_id: &str, new_content: &str) -> Result<Message> {
        let now = Utc::now().timestamp();

        tracing::debug!(
            message_id = %message_id,
            content_len = new_content.len(),
            "Editing message"
        );

        // First, get channel_id from SQL to locate CRDT document
        tracing::trace!(message_id = %message_id, "Looking up channel_id from SQL");
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT channel_id, thread_id, author_id, created_at FROM messages WHERE id = ?",
                params![message_id],
            )
            .await?;

        let row = rows.next().await?.context("Message not found")?;

        let channel_id: String = row.get(0)?;
        let thread_id: Option<String> = row.get(1)?;
        let author_id: String = row.get(2)?;
        let created_at: i64 = row.get(3)?;

        // Load CRDT document
        let doc_id = format!("channel:{}", channel_id);
        tracing::trace!(doc_id = %doc_id, "Loading CRDT document for edit");
        let doc = self.crdt.load_document(&doc_id).await?;

        // Update message in CRDT
        // Pattern: Complete all CRDT operations before any await to avoid Send issues
        {
            use crate::crdt_manager::CrdtManager;

            let messages = doc.get_or_insert_map("messages");

            tracing::trace!("Creating CRDT transaction for message update");
            let mut txn = doc.transact_mut();
            let msg_map = CrdtManager::get_nested_map(&messages, &txn, message_id)
                .context("Message not found in CRDT")?;

            // Update content and timestamp
            CrdtManager::set_map_string(&msg_map, &mut txn, "content", new_content);
            CrdtManager::set_map_i64(&msg_map, &mut txn, "updated_at", now);

            tracing::trace!(message_id = %message_id, updated_at = now, "Message updated in CRDT");
        } // Transaction and MapRef dropped here - safe to await now

        // Save updated CRDT
        tracing::trace!(doc_id = %doc_id, "Saving CRDT document");
        self.crdt
            .save_document(&doc_id, "channel", &channel_id, &doc)
            .await?;

        // Materialize to SQL
        tracing::trace!(message_id = %message_id, "Materializing updated message to SQL");
        self.materialize_message_to_sql_from_doc(&doc, message_id, &channel_id)
            .await?;

        tracing::info!(
            message_id = %message_id,
            channel_id = %channel_id,
            "Message edited successfully"
        );

        Ok(Message {
            id: message_id.to_string(),
            channel_id,
            thread_id,
            author_id,
            content: new_content.to_string(),
            created_at,
            updated_at: Some(now),
        })
    }

    /// Delete a message (CRDT-first tombstone deletion)
    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let now = Utc::now().timestamp();

        tracing::debug!(
            message_id = %message_id,
            "Deleting message (tombstone pattern)"
        );

        // Get channel_id from SQL to locate CRDT document
        tracing::trace!(message_id = %message_id, "Looking up channel_id from SQL");
        let db = self.crdt.connection()?;
        let mut rows = db
            .query(
                "SELECT channel_id FROM messages WHERE id = ?",
                params![message_id],
            )
            .await?;

        let channel_id: String = rows.next().await?.context("Message not found")?.get(0)?;

        // Load CRDT document
        let doc_id = format!("channel:{}", channel_id);
        tracing::trace!(doc_id = %doc_id, "Loading CRDT document for deletion");
        let doc = self.crdt.load_document(&doc_id).await?;

        // Mark message as deleted in CRDT (tombstone pattern)
        // Pattern: Complete all CRDT operations before any await to avoid Send issues
        {
            use crate::crdt_manager::CrdtManager;

            let messages = doc.get_or_insert_map("messages");

            tracing::trace!("Creating CRDT transaction for tombstone deletion");
            let mut txn = doc.transact_mut();
            let msg_map = CrdtManager::get_nested_map(&messages, &txn, message_id)
                .context("Message not found in CRDT")?;

            // Set tombstone flags
            CrdtManager::set_map_bool(&msg_map, &mut txn, "deleted", true);
            CrdtManager::set_map_i64(&msg_map, &mut txn, "deleted_at", now);

            tracing::trace!(
                message_id = %message_id,
                deleted_at = now,
                "Tombstone flags set in CRDT"
            );
        } // Transaction and MapRef dropped here - safe to await now

        // Save updated CRDT
        tracing::trace!(doc_id = %doc_id, "Saving CRDT document");
        self.crdt
            .save_document(&doc_id, "channel", &channel_id, &doc)
            .await?;

        // Materialize to SQL (will set deleted_at)
        tracing::trace!(message_id = %message_id, "Materializing deleted message to SQL");
        self.materialize_message_to_sql_from_doc(&doc, message_id, &channel_id)
            .await?;

        tracing::info!(
            message_id = %message_id,
            channel_id = %channel_id,
            "Message deleted successfully (tombstone)"
        );

        Ok(())
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
                 WHERE channel_id = ? AND thread_id IS NULL AND deleted_at IS NULL
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
    pub async fn add_member(&self, channel_id: &str, user_id: &str, role: &str) -> Result<()> {
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
            .await?;
        Ok(())
    }

    // =========================================================================
    // Private Helper Methods
    // =========================================================================

    /// Materialize a single message from CRDT to SQL using an already-loaded document
    /// This avoids reloading the document and ensures we materialize the latest in-memory state
    async fn materialize_message_to_sql_from_doc(
        &self,
        doc: &yrs::Doc,
        msg_id: &str,
        channel_id: &str,
    ) -> Result<()> {
        use crate::crdt_manager::CrdtManager;

        tracing::trace!(
            msg_id = %msg_id,
            channel_id = %channel_id,
            "Starting materialization from CRDT to SQL"
        );

        // Extract all data in a scope to drop Transaction and MapRef before await
        let (author_id, content, created_at, deleted, thread_id, updated_at, deleted_at) = {
            // Get messages Map BEFORE creating transaction to avoid conflict
            let messages = doc.get_or_insert_map("messages");
            let txn = doc.transact();

            // Get the specific message Map
            let msg_map = CrdtManager::get_nested_map(&messages, &txn, msg_id)
                .context("Message not found in CRDT")?;

            // Extract fields
            let author_id = CrdtManager::get_map_string(&msg_map, &txn, "author_id")
                .context("Missing author_id")?;
            let content = CrdtManager::get_map_string(&msg_map, &txn, "content")
                .context("Missing content")?;
            let created_at = CrdtManager::get_map_i64(&msg_map, &txn, "created_at")
                .context("Missing created_at")?;
            let deleted = CrdtManager::get_map_bool(&msg_map, &txn, "deleted").unwrap_or(false);
            let thread_id = CrdtManager::get_map_string(&msg_map, &txn, "thread_id");
            let updated_at = CrdtManager::get_map_i64(&msg_map, &txn, "updated_at");
            let deleted_at = CrdtManager::get_map_i64(&msg_map, &txn, "deleted_at");

            tracing::trace!(
                msg_id = %msg_id,
                author_id = %author_id,
                deleted = deleted,
                "Extracted message fields from CRDT"
            );

            (
                author_id, content, created_at, deleted, thread_id, updated_at, deleted_at,
            )
        }; // Transaction and MapRefs dropped here

        // Write to SQL (now safe to await)
        let db = self.crdt.connection()?;

        if deleted {
            // Soft delete: set deleted_at
            tracing::trace!(msg_id = %msg_id, deleted_at = ?deleted_at, "Writing deleted message to SQL");
            db.execute(
                "INSERT OR REPLACE INTO messages
                 (id, channel_id, thread_id, author_id, content, created_at, updated_at, deleted_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    msg_id, channel_id, thread_id, author_id, content, created_at, updated_at,
                    deleted_at
                ],
            )
            .await
            .context("Failed to materialize deleted message")?;
            tracing::trace!(msg_id = %msg_id, "Deleted message materialized to SQL");
        } else {
            // Normal message
            tracing::trace!(msg_id = %msg_id, "Writing normal message to SQL");
            db.execute(
                "INSERT OR REPLACE INTO messages
                 (id, channel_id, thread_id, author_id, content, created_at, updated_at, deleted_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, NULL)",
                params![
                    msg_id, channel_id, thread_id, author_id, content, created_at, updated_at
                ],
            )
            .await
            .context("Failed to materialize message")?;
            tracing::trace!(msg_id = %msg_id, "Normal message materialized to SQL");
        }

        tracing::debug!(
            msg_id = %msg_id,
            channel_id = %channel_id,
            deleted = deleted,
            "Message materialization completed"
        );

        Ok(())
    }

    // =========================================================================
    // Synchronization Methods (Phase 3)
    // =========================================================================

    /// Get the current state vector for a channel's CRDT document
    /// Used by peers to request only missing updates
    pub async fn get_channel_state_vector(&self, channel_id: &str) -> Result<Vec<u8>> {
        use yrs::updates::encoder::Encode;

        let doc_id = format!("channel:{}", channel_id);
        let sv = self.crdt._get_state_vector(&doc_id).await?;
        Ok(sv.encode_v1())
    }

    /// Generate a diff (update) containing only changes not in the remote state vector
    /// This is the core of efficient CRDT synchronization - only send what's missing
    pub async fn get_channel_diff(
        &self,
        channel_id: &str,
        remote_state_vector: &[u8],
    ) -> Result<Vec<u8>> {
        use yrs::updates::decoder::Decode;

        let doc_id = format!("channel:{}", channel_id);

        // Decode remote state vector
        let remote_sv = yrs::StateVector::decode_v1(remote_state_vector)
            .map_err(|e| anyhow::anyhow!("Failed to decode state vector: {}", e))?;

        // Generate diff containing only updates the remote peer doesn't have
        let diff = self.crdt._get_diff(&doc_id, &remote_sv).await?;
        Ok(diff)
    }

    /// Apply an incoming diff (update) from another peer
    /// Merges remote changes into our local CRDT, then re-materializes to SQL
    pub async fn apply_channel_diff(
        &self,
        channel_id: &str,
        diff: &[u8],
    ) -> Result<AppliedDiffResult> {
        use yrs::Map;

        let doc_id = format!("channel:{}", channel_id);

        // Apply the diff to our CRDT
        self.crdt
            .merge_update(&doc_id, "channel", channel_id, diff)
            .await?;

        // Load the updated document
        let doc = self.crdt.load_document(&doc_id).await?;

        // Count messages and rematerialize all to SQL
        // Extract message IDs in a scope to ensure MapRef is dropped before async operations
        let message_ids: Vec<String> = {
            let messages_map = doc.get_or_insert_map("messages");
            let txn = doc.transact();
            messages_map.keys(&txn).map(|k| k.to_string()).collect()
        };

        let total_messages = message_ids.len();
        let mut materialized_count = 0;

        for msg_id in message_ids {
            match self
                .materialize_message_to_sql_from_doc(&doc, &msg_id, channel_id)
                .await
            {
                Ok(_) => materialized_count += 1,
                Err(e) => {
                    tracing::warn!(
                        msg_id = %msg_id,
                        error = %e,
                        "Failed to materialize message"
                    );
                }
            }
        }

        Ok(AppliedDiffResult {
            messages_updated: materialized_count,
            total_messages,
        })
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
            .create_channel(
                "org-1",
                "general",
                Some("General chat".to_string()),
                "user-1",
            )
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

    #[tokio::test]
    async fn test_crdt_message_edit() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        // Create channel and send message
        let channel = service
            .create_channel("org-1", "test", None, "user-1")
            .await
            .unwrap();

        let msg = service
            .send_message(&channel.id, "user-1", "Original content", None)
            .await
            .unwrap();

        // Edit the message
        let edited = service
            .edit_message(&msg.id, "Updated content")
            .await
            .unwrap();

        assert_eq!(edited.content, "Updated content");
        assert!(edited.updated_at.is_some());

        // Verify edit persisted to SQL
        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Updated content");
        assert!(messages[0].updated_at.is_some());
    }

    #[tokio::test]
    async fn test_crdt_message_tombstone_deletion() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        // Create channel and send message
        let channel = service
            .create_channel("org-1", "test", None, "user-1")
            .await
            .unwrap();

        let msg = service
            .send_message(&channel.id, "user-1", "Test message", None)
            .await
            .unwrap();

        // Delete the message (tombstone pattern)
        service.delete_message(&msg.id).await.unwrap();

        // Verify message is filtered out by SQL query (deleted_at IS NULL)
        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 0);

        // Verify tombstone exists in SQL
        let db = service.crdt.connection().unwrap();
        let mut rows = db
            .query(
                "SELECT id, deleted_at FROM messages WHERE id = ?",
                params![msg.id.clone()],
            )
            .await
            .unwrap();

        let row = rows.next().await.unwrap().unwrap();
        let deleted_at: Option<i64> = row.get(1).unwrap();
        assert!(deleted_at.is_some());
    }

    #[tokio::test]
    async fn test_crdt_sql_consistency() {
        use crate::crdt_manager::CrdtManager as CrdtMgr;

        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt.clone());

        // Create channel and send message
        let channel = service
            .create_channel("org-1", "test", None, "user-1")
            .await
            .unwrap();

        let msg = service
            .send_message(&channel.id, "user-1", "Consistency test", None)
            .await
            .unwrap();

        // Load CRDT document and verify message exists
        let doc_id = format!("channel:{}", channel.id);
        let doc = crdt.load_document(&doc_id).await.unwrap();

        let (crdt_content, crdt_author, crdt_deleted) = {
            // Get messages Map BEFORE creating transaction to avoid conflict
            let messages = doc.get_or_insert_map("messages");
            let txn = doc.transact();
            let msg_map = CrdtMgr::get_nested_map(&messages, &txn, &msg.id).unwrap();

            let content = CrdtMgr::get_map_string(&msg_map, &txn, "content").unwrap();
            let author = CrdtMgr::get_map_string(&msg_map, &txn, "author_id").unwrap();
            let deleted = CrdtMgr::get_map_bool(&msg_map, &txn, "deleted").unwrap_or(false);

            (content, author, deleted)
        };

        // Verify CRDT matches SQL
        assert_eq!(crdt_content, "Consistency test");
        assert_eq!(crdt_author, "user-1");
        assert!(!crdt_deleted);

        // Verify SQL data
        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Consistency test");
        assert_eq!(messages[0].author_id, "user-1");
    }

    #[tokio::test]
    async fn test_crdt_multiple_edits() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        // Create channel and send message
        let channel = service
            .create_channel("org-1", "test", None, "user-1")
            .await
            .unwrap();

        let msg = service
            .send_message(&channel.id, "user-1", "Version 1", None)
            .await
            .unwrap();

        // Perform multiple edits
        service.edit_message(&msg.id, "Version 2").await.unwrap();
        service.edit_message(&msg.id, "Version 3").await.unwrap();
        let final_msg = service
            .edit_message(&msg.id, "Final version")
            .await
            .unwrap();

        assert_eq!(final_msg.content, "Final version");
        assert!(final_msg.updated_at.is_some());

        // Verify final state in SQL
        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Final version");
    }

    #[tokio::test]
    async fn test_crdt_thread_messages() {
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        // Create channel and parent message
        let channel = service
            .create_channel("org-1", "test", None, "user-1")
            .await
            .unwrap();

        let parent_msg = service
            .send_message(&channel.id, "user-1", "Parent message", None)
            .await
            .unwrap();

        // Send thread reply
        let thread_msg = service
            .send_message(
                &channel.id,
                "user-2",
                "Thread reply",
                Some(parent_msg.id.clone()),
            )
            .await
            .unwrap();

        assert_eq!(thread_msg.thread_id, Some(parent_msg.id.clone()));

        // Verify only parent shows in main channel list (thread_id IS NULL)
        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, parent_msg.id);
    }

    // =========================================================================
    // Phase 3: Network Synchronization Tests (TDD)
    // =========================================================================

    #[tokio::test]
    async fn test_get_channel_state_vector() {
        // Setup service and create channel with messages
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        let channel = service
            .create_channel("org-1", "sync-test", None, "user-1")
            .await
            .unwrap();

        // Send some messages to create CRDT state
        service
            .send_message(&channel.id, "user-1", "First message", None)
            .await
            .unwrap();

        service
            .send_message(&channel.id, "user-2", "Second message", None)
            .await
            .unwrap();

        // Test: Get state vector - should return non-empty encoded bytes
        let state_vector = service
            .get_channel_state_vector(&channel.id)
            .await
            .unwrap();

        // Verify state vector is encoded and non-empty
        assert!(!state_vector.is_empty(), "State vector should be non-empty");
        assert!(state_vector.len() > 10, "State vector should have reasonable size");
    }

    #[tokio::test]
    async fn test_get_channel_diff() {
        // Setup service with messages
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        // Create channel and send messages
        let channel = service
            .create_channel("org-1", "sync-test", None, "user-1")
            .await
            .unwrap();

        service
            .send_message(&channel.id, "user-1", "First message", None)
            .await
            .unwrap();

        service
            .send_message(&channel.id, "user-1", "Second message", None)
            .await
            .unwrap();

        // Simulate a remote peer with empty state (no messages received yet)
        // Empty state vector = "I have seen nothing, send me everything"
        use yrs::updates::encoder::Encode;
        let empty_state_vector = yrs::StateVector::default().encode_v1();

        // Generate diff for remote peer who has empty state
        let diff = service
            .get_channel_diff(&channel.id, &empty_state_vector)
            .await
            .unwrap();

        // Verify diff is non-empty (contains both messages)
        assert!(!diff.is_empty(), "Diff should contain all messages");
        assert!(diff.len() > 50, "Diff should have reasonable size for 2 messages");
    }

    #[tokio::test]
    async fn test_apply_channel_diff() {
        // Setup service
        let dir = tempdir().unwrap();
        let crdt = Arc::new(CrdtManager::new(dir.path().join("test.db")).await.unwrap());
        let service = ChannelService::new(crdt);

        // Create channel
        let channel = service
            .create_channel("org-1", "sync-test", None, "user-1")
            .await
            .unwrap();

        // Send messages
        service
            .send_message(&channel.id, "user-1", "Message one", None)
            .await
            .unwrap();

        service
            .send_message(&channel.id, "user-1", "Message two", None)
            .await
            .unwrap();

        // Get current state as diff (simulating what a remote peer would send)
        use yrs::updates::encoder::Encode;
        let empty_state = yrs::StateVector::default().encode_v1();
        let diff = service
            .get_channel_diff(&channel.id, &empty_state)
            .await
            .unwrap();

        // Apply diff (idempotent operation - applying same data twice should work)
        let result = service
            .apply_channel_diff(&channel.id, &diff)
            .await
            .unwrap();

        // Verify result stats
        assert_eq!(result.messages_updated, 2, "Should materialize 2 messages");
        assert_eq!(result.total_messages, 2, "Should have 2 total messages");

        // Verify messages still in SQL (INSERT OR REPLACE is idempotent)
        let messages = service.get_messages(&channel.id, None, None).await.unwrap();
        assert_eq!(messages.len(), 2, "Should have 2 messages");

        // Check messages exist (order may vary)
        let contents: Vec<&str> = messages.iter().map(|m| m.content.as_str()).collect();
        assert!(contents.contains(&"Message one"), "Should contain 'Message one'");
        assert!(contents.contains(&"Message two"), "Should contain 'Message two'");
    }

    #[tokio::test]
    async fn test_multi_peer_sync_integration() {
        // Setup two peers with separate databases
        let dir1 = tempdir().unwrap();
        let dir2 = tempdir().unwrap();

        let crdt1 = Arc::new(CrdtManager::new(dir1.path().join("peer1.db")).await.unwrap());
        let crdt2 = Arc::new(CrdtManager::new(dir2.path().join("peer2.db")).await.unwrap());

        let peer1 = ChannelService::new(crdt1);
        let peer2 = ChannelService::new(crdt2);

        // Both peers create the same channel (same ID via deterministic generation)
        let channel1 = peer1
            .create_channel("org-1", "test-channel", None, "user-1")
            .await
            .unwrap();

        let channel2 = peer2
            .create_channel("org-1", "test-channel", None, "user-1")
            .await
            .unwrap();

        // Peer 1 sends messages
        peer1
            .send_message(&channel1.id, "user-1", "Hello from peer 1", None)
            .await
            .unwrap();

        peer1
            .send_message(&channel1.id, "user-1", "Second from peer 1", None)
            .await
            .unwrap();

        // Peer 2 sends different messages
        peer2
            .send_message(&channel2.id, "user-2", "Hello from peer 2", None)
            .await
            .unwrap();

        // Verify initial state - each peer has only their own messages
        let peer1_messages = peer1.get_messages(&channel1.id, None, None).await.unwrap();
        let peer2_messages = peer2.get_messages(&channel2.id, None, None).await.unwrap();

        assert_eq!(peer1_messages.len(), 2, "Peer 1 should have 2 messages");
        assert_eq!(peer2_messages.len(), 1, "Peer 2 should have 1 message");

        // SYNC: Peer 2 requests state from Peer 1
        let peer2_state_vector = peer2.get_channel_state_vector(&channel2.id).await.unwrap();
        let diff_from_peer1 = peer1
            .get_channel_diff(&channel1.id, &peer2_state_vector)
            .await
            .unwrap();

        // Peer 2 applies diff from Peer 1
        let result = peer2
            .apply_channel_diff(&channel2.id, &diff_from_peer1)
            .await
            .unwrap();

        assert_eq!(result.total_messages, 3, "Peer 2 should now have 3 messages total");

        // SYNC: Peer 1 requests state from Peer 2
        let peer1_state_vector = peer1.get_channel_state_vector(&channel1.id).await.unwrap();
        let diff_from_peer2 = peer2
            .get_channel_diff(&channel2.id, &peer1_state_vector)
            .await
            .unwrap();

        // Peer 1 applies diff from Peer 2
        let result = peer1
            .apply_channel_diff(&channel1.id, &diff_from_peer2)
            .await
            .unwrap();

        assert_eq!(result.total_messages, 3, "Peer 1 should now have 3 messages total");

        // Verify both peers now have all 3 messages
        let final_peer1_messages = peer1.get_messages(&channel1.id, None, None).await.unwrap();
        let final_peer2_messages = peer2.get_messages(&channel2.id, None, None).await.unwrap();

        assert_eq!(final_peer1_messages.len(), 3, "Peer 1 should have all 3 messages");
        assert_eq!(final_peer2_messages.len(), 3, "Peer 2 should have all 3 messages");

        // Verify message contents match (CRDT convergence)
        let peer1_contents: Vec<&str> = final_peer1_messages.iter().map(|m| m.content.as_str()).collect();
        let peer2_contents: Vec<&str> = final_peer2_messages.iter().map(|m| m.content.as_str()).collect();

        assert!(peer1_contents.contains(&"Hello from peer 1"));
        assert!(peer1_contents.contains(&"Second from peer 1"));
        assert!(peer1_contents.contains(&"Hello from peer 2"));

        assert!(peer2_contents.contains(&"Hello from peer 1"));
        assert!(peer2_contents.contains(&"Second from peer 1"));
        assert!(peer2_contents.contains(&"Hello from peer 2"));
    }
}
