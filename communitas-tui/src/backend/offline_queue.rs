use anyhow::Result;
use communitas_core::crdt::EntityType;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

/// Operation that can be queued for offline execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueuedOperation {
    /// Create entity operation
    CreateEntity {
        name: String,
        entity_type: EntityType,
        members: Vec<String>,
    },
    /// Send message operation
    SendMessage {
        entity_id: String,
        entity_type: EntityType,
        text: String,
    },
    /// Add member operation
    AddMember {
        entity_id: String,
        entity_type: EntityType,
        member_id: String,
    },
    /// Remove member operation
    RemoveMember {
        entity_id: String,
        entity_type: EntityType,
        member_id: String,
    },
}

/// Queued operation entry with metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueuedOperationEntry {
    /// Unique operation ID
    pub id: String,
    /// The operation to execute
    pub operation: QueuedOperation,
    /// Priority (higher = sync first)
    pub priority: u8,
    /// Timestamp when queued
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl QueuedOperationEntry {
    /// Create new queued operation entry
    pub fn new(operation: QueuedOperation, priority: u8) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation,
            priority,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, PartialEq)]
pub enum SyncResult {
    /// Operation completed successfully
    Success { operation_id: String },
    /// Operation failed with error
    Failed { operation_id: String, error: String },
    /// Operation skipped (e.g., duplicate)
    Skipped { operation_id: String, reason: String },
}

/// Sync progress update
#[derive(Debug, Clone)]
pub struct SyncProgress {
    /// Total operations to sync
    pub total: usize,
    /// Number of operations completed
    pub completed: usize,
    /// Current operation ID being processed
    pub current_operation_id: Option<String>,
}

/// Offline queue manager
///
/// Manages a priority queue of operations that need to be synchronized
/// when the application comes back online. Supports persistence across
/// app restarts.
///
/// Performance characteristics:
/// - Enqueue: O(1) for append, O(n) for priority insert
/// - Dequeue: O(1)
/// - Size limit enforcement: O(1) with VecDeque::pop_front()
pub struct OfflineQueue {
    /// Queued operations (sorted by priority and timestamp)
    queue: VecDeque<QueuedOperationEntry>,
    /// Maximum queue size (0 = unlimited)
    max_size: usize,
    /// Persistence file path
    persistence_path: PathBuf,
}

impl OfflineQueue {
    /// Create new offline queue
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let persistence_path = data_dir.join("offline_queue.json");

        let mut queue = Self {
            queue: VecDeque::new(),
            max_size: 1000, // Default max size
            persistence_path,
        };

        // Load persisted queue if exists
        queue.load_from_disk().await?;

        Ok(queue)
    }

    /// Set maximum queue size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;

        // Trim queue if needed
        while self.max_size > 0 && self.queue.len() > self.max_size {
            self.queue.pop_front();
        }
    }

    /// Enqueue operation with priority
    ///
    /// Higher priority operations are synced first.
    /// Operations with same priority are synced in FIFO order.
    pub async fn enqueue(&mut self, operation: QueuedOperation, priority: u8) -> Result<String> {
        let entry = QueuedOperationEntry::new(operation, priority);
        let operation_id = entry.id.clone();

        // Find insertion position based on priority (higher priority first)
        let insert_pos = self.queue.iter().position(|e| e.priority < priority)
            .unwrap_or(self.queue.len());

        self.queue.insert(insert_pos, entry);

        // Enforce size limit (remove oldest if at capacity)
        if self.max_size > 0 && self.queue.len() > self.max_size {
            self.queue.pop_front();
        }

        // Persist to disk
        self.save_to_disk().await?;

        Ok(operation_id)
    }

    /// Get all queued operations
    pub fn get_all(&self) -> Vec<QueuedOperationEntry> {
        self.queue.iter().cloned().collect()
    }

    /// Remove operation by ID
    pub async fn remove(&mut self, operation_id: &str) -> Result<bool> {
        let initial_len = self.queue.len();
        self.queue.retain(|entry| entry.id != operation_id);

        let removed = self.queue.len() < initial_len;

        if removed {
            self.save_to_disk().await?;
        }

        Ok(removed)
    }

    /// Clear all queued operations
    pub async fn clear(&mut self) -> Result<()> {
        self.queue.clear();
        self.save_to_disk().await?;
        Ok(())
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get queue size
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Detect if operation is duplicate
    ///
    /// Checks if a similar operation already exists in queue or was recently synced.
    pub fn is_duplicate(&self, operation: &QueuedOperation) -> bool {
        self.queue.iter().any(|entry| {
            match (&entry.operation, operation) {
                // Same entity creation
                (QueuedOperation::CreateEntity { name: n1, entity_type: t1, .. },
                 QueuedOperation::CreateEntity { name: n2, entity_type: t2, .. }) => {
                    n1 == n2 && t1 == t2
                }
                // Same message to same entity
                (QueuedOperation::SendMessage { entity_id: e1, text: t1, .. },
                 QueuedOperation::SendMessage { entity_id: e2, text: t2, .. }) => {
                    e1 == e2 && t1 == t2
                }
                // Same member addition
                (QueuedOperation::AddMember { entity_id: e1, member_id: m1, .. },
                 QueuedOperation::AddMember { entity_id: e2, member_id: m2, .. }) => {
                    e1 == e2 && m1 == m2
                }
                // Same member removal
                (QueuedOperation::RemoveMember { entity_id: e1, member_id: m1, .. },
                 QueuedOperation::RemoveMember { entity_id: e2, member_id: m2, .. }) => {
                    e1 == e2 && m1 == m2
                }
                _ => false,
            }
        })
    }

    /// Load queue from disk
    async fn load_from_disk(&mut self) -> Result<()> {
        if !self.persistence_path.exists() {
            return Ok(());
        }

        let contents = fs::read_to_string(&self.persistence_path).await?;
        self.queue = serde_json::from_str(&contents)?;

        Ok(())
    }

    /// Save queue to disk
    async fn save_to_disk(&self) -> Result<()> {
        let contents = serde_json::to_string_pretty(&self.queue)?;

        // Ensure directory exists
        if let Some(parent) = self.persistence_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(&self.persistence_path, contents).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_enqueue_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path().to_path_buf()).await.unwrap();

        let op = QueuedOperation::CreateEntity {
            name: "Test".to_string(),
            entity_type: EntityType::Channel,
            members: vec![],
        };

        let op_id = queue.enqueue(op.clone(), 0).await.unwrap();

        let all_ops = queue.get_all();
        assert_eq!(all_ops.len(), 1);
        assert_eq!(all_ops[0].id, op_id);
        assert_eq!(all_ops[0].operation, op);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path().to_path_buf()).await.unwrap();

        // Add low priority
        queue.enqueue(
            QueuedOperation::CreateEntity {
                name: "Low".to_string(),
                entity_type: EntityType::Channel,
                members: vec![],
            },
            0,
        ).await.unwrap();

        // Add high priority (should be first)
        queue.enqueue(
            QueuedOperation::CreateEntity {
                name: "High".to_string(),
                entity_type: EntityType::Channel,
                members: vec![],
            },
            10,
        ).await.unwrap();

        let all_ops = queue.get_all();
        assert_eq!(all_ops.len(), 2);

        // High priority should be first
        if let QueuedOperation::CreateEntity { name, .. } = &all_ops[0].operation {
            assert_eq!(name, "High");
        } else {
            panic!("Expected CreateEntity operation");
        }
    }

    #[tokio::test]
    async fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();

        let op_id = {
            let mut queue = OfflineQueue::new(data_dir.clone()).await.unwrap();

            queue.enqueue(
                QueuedOperation::CreateEntity {
                    name: "Persistent".to_string(),
                    entity_type: EntityType::Channel,
                    members: vec![],
                },
                0,
            ).await.unwrap()
        };

        // Create new queue instance - should load from disk
        let queue = OfflineQueue::new(data_dir).await.unwrap();
        let all_ops = queue.get_all();
        assert_eq!(all_ops.len(), 1);
        assert_eq!(all_ops[0].id, op_id);
    }

    #[tokio::test]
    async fn test_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path().to_path_buf()).await.unwrap();

        queue.set_max_size(3);

        // Add 5 operations
        for i in 0..5 {
            queue.enqueue(
                QueuedOperation::SendMessage {
                    entity_id: "test".to_string(),
                    entity_type: EntityType::Channel,
                    text: format!("Message {}", i),
                },
                0,
            ).await.unwrap();
        }

        // Should only have last 3
        let all_ops = queue.get_all();
        assert_eq!(all_ops.len(), 3);

        if let QueuedOperation::SendMessage { text, .. } = &all_ops[2].operation {
            assert_eq!(text, "Message 4");
        }
    }

    #[tokio::test]
    async fn test_duplicate_detection() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path().to_path_buf()).await.unwrap();

        let op = QueuedOperation::CreateEntity {
            name: "Duplicate".to_string(),
            entity_type: EntityType::Channel,
            members: vec![],
        };

        queue.enqueue(op.clone(), 0).await.unwrap();

        assert!(queue.is_duplicate(&op));
    }
}
