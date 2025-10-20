// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Offline Operation Queue
//!
//! Queues CRDT operations when offline and applies them when connection is restored.
//! Provides automatic retry, conflict detection, and operation ordering.

use super::operations::LamportTimestamp;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A queued CRDT operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedOperation {
    /// Unique operation ID
    pub id: String,
    /// Entity type (channel, member, org, etc.)
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// Operation type
    pub operation: OperationType,
    /// When this operation was created
    pub created_at: i64,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retries before giving up
    pub max_retries: u32,
    /// Timestamp for conflict resolution
    pub timestamp: LamportTimestamp,
}

/// Types of CRDT operations that can be queued
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Create a new entity
    Create {
        doc_type: String,
        data: serde_json::Value,
    },
    /// Update an existing entity
    Update {
        field: String,
        value: serde_json::Value,
    },
    /// Delete an entity (soft delete with tombstone)
    Delete,
    /// Add to a set (e.g., add member to channel)
    SetAdd {
        collection: String,
        element_id: String,
        operation_id: String,
    },
    /// Remove from a set
    SetRemove {
        collection: String,
        element_id: String,
        operation_id: String,
    },
    /// Increment a counter
    CounterIncrement { field: String, amount: i64 },
}

/// Status of an operation in the queue
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OperationStatus {
    /// Waiting to be applied
    Pending,
    /// Currently being applied
    InProgress,
    /// Successfully applied
    Completed,
    /// Failed after max retries
    Failed,
}

/// Offline operation queue
pub struct OfflineQueue {
    /// Queue of pending operations
    queue: VecDeque<QueuedOperation>,
    /// Completed operations (kept for a while for auditing)
    completed: Vec<QueuedOperation>,
    /// Failed operations
    failed: Vec<QueuedOperation>,
    /// Maximum number of completed operations to keep
    max_completed: usize,
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            completed: Vec::new(),
            failed: Vec::new(),
            max_completed: 1000,
        }
    }

    /// Enqueue a new operation
    pub fn enqueue(&mut self, operation: QueuedOperation) {
        self.queue.push_back(operation);
    }

    /// Get the next operation to process (dequeues from front)
    pub fn dequeue(&mut self) -> Option<QueuedOperation> {
        self.queue.pop_front()
    }

    /// Mark an operation as completed
    pub fn mark_completed(&mut self, operation: QueuedOperation) {
        self.completed.push(operation);

        // Trim completed list if too large
        if self.completed.len() > self.max_completed {
            self.completed
                .drain(0..(self.completed.len() - self.max_completed));
        }
    }

    /// Mark an operation as failed (re-queue if retries remaining)
    pub fn mark_failed(&mut self, mut operation: QueuedOperation) {
        operation.retry_count += 1;

        if operation.retry_count < operation.max_retries {
            // Re-queue for retry
            self.queue.push_back(operation);
        } else {
            // Max retries reached, move to failed
            self.failed.push(operation);
        }
    }

    /// Get number of pending operations
    pub fn pending_count(&self) -> usize {
        self.queue.len()
    }

    /// Get number of failed operations
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Get all pending operations (for inspection)
    pub fn pending_operations(&self) -> Vec<&QueuedOperation> {
        self.queue.iter().collect()
    }

    /// Get all failed operations
    pub fn failed_operations(&self) -> &[QueuedOperation] {
        &self.failed
    }

    /// Clear all failed operations
    pub fn clear_failed(&mut self) {
        self.failed.clear();
    }

    /// Retry all failed operations
    pub fn retry_failed(&mut self) {
        for mut op in self.failed.drain(..) {
            op.retry_count = 0;
            self.queue.push_back(op);
        }
    }

    /// Persist queue to disk (JSON format)
    pub fn save_to_disk(&self, path: &std::path::Path) -> Result<()> {
        let data = serde_json::json!({
            "queue": self.queue,
            "completed": self.completed,
            "failed": self.failed,
        });

        std::fs::write(path, serde_json::to_string_pretty(&data)?)
            .context("Failed to write queue to disk")?;

        Ok(())
    }

    /// Load queue from disk
    pub fn load_from_disk(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("Failed to read queue from disk")?;

        let data: serde_json::Value = serde_json::from_str(&contents)?;

        let queue =
            serde_json::from_value(data["queue"].clone()).context("Failed to parse queue")?;
        let completed = serde_json::from_value(data["completed"].clone())
            .context("Failed to parse completed")?;
        let failed =
            serde_json::from_value(data["failed"].clone()).context("Failed to parse failed")?;

        Ok(Self {
            queue,
            completed,
            failed,
            max_completed: 1000,
        })
    }
}

/// Builder for creating queued operations
pub struct OperationBuilder {
    entity_type: String,
    entity_id: String,
    peer_id: String,
}

impl OperationBuilder {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        peer_id: impl Into<String>,
    ) -> Self {
        Self {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            peer_id: peer_id.into(),
        }
    }

    /// Build a create operation
    pub fn create(self, doc_type: String, data: serde_json::Value) -> QueuedOperation {
        QueuedOperation {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            operation: OperationType::Create { doc_type, data },
            created_at: Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 5,
            timestamp: LamportTimestamp::now(&self.peer_id),
        }
    }

    /// Build an update operation
    pub fn update(self, field: String, value: serde_json::Value) -> QueuedOperation {
        QueuedOperation {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            operation: OperationType::Update { field, value },
            created_at: Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 5,
            timestamp: LamportTimestamp::now(&self.peer_id),
        }
    }

    /// Build a delete operation
    pub fn delete(self) -> QueuedOperation {
        QueuedOperation {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            operation: OperationType::Delete,
            created_at: Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 5,
            timestamp: LamportTimestamp::now(&self.peer_id),
        }
    }

    /// Build a set add operation
    pub fn set_add(self, collection: String, element_id: String) -> QueuedOperation {
        let operation_id = uuid::Uuid::new_v4().to_string();
        QueuedOperation {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            operation: OperationType::SetAdd {
                collection,
                element_id,
                operation_id,
            },
            created_at: Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 5,
            timestamp: LamportTimestamp::now(&self.peer_id),
        }
    }

    /// Build a set remove operation
    pub fn set_remove(self, collection: String, element_id: String) -> QueuedOperation {
        let operation_id = uuid::Uuid::new_v4().to_string();
        QueuedOperation {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            operation: OperationType::SetRemove {
                collection,
                element_id,
                operation_id,
            },
            created_at: Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 5,
            timestamp: LamportTimestamp::now(&self.peer_id),
        }
    }

    /// Build a counter increment operation
    pub fn counter_increment(self, field: String, amount: i64) -> QueuedOperation {
        QueuedOperation {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            operation: OperationType::CounterIncrement { field, amount },
            created_at: Utc::now().timestamp(),
            retry_count: 0,
            max_retries: 5,
            timestamp: LamportTimestamp::now(&self.peer_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_queue_operations() {
        let mut queue = OfflineQueue::new();
        assert!(queue.is_empty());

        let op = OperationBuilder::new("channel", "channel-123", "peer1")
            .update("name".to_string(), serde_json::json!("New Name"));

        queue.enqueue(op.clone());
        assert_eq!(queue.pending_count(), 1);

        let next = queue.dequeue().unwrap();
        assert_eq!(next.entity_id, op.entity_id);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_retry_logic() {
        let mut queue = OfflineQueue::new();

        let op = OperationBuilder::new("channel", "channel-123", "peer1")
            .update("name".to_string(), serde_json::json!("New Name"));

        queue.enqueue(op.clone());
        let mut next = queue.dequeue().unwrap();

        // Mark as failed (should re-queue)
        queue.mark_failed(next.clone());
        assert_eq!(queue.pending_count(), 1);
        assert_eq!(queue.failed_count(), 0);

        // Fail multiple times
        for _ in 0..4 {
            next = queue.dequeue().unwrap();
            queue.mark_failed(next.clone());
        }

        // After max retries, should move to failed
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.failed_count(), 1);
    }

    #[test]
    fn test_persistence() {
        let dir = tempdir().unwrap();
        let queue_path = dir.path().join("queue.json");

        let mut queue = OfflineQueue::new();
        let op = OperationBuilder::new("channel", "channel-123", "peer1")
            .update("name".to_string(), serde_json::json!("New Name"));

        queue.enqueue(op);
        queue.save_to_disk(&queue_path).unwrap();

        let loaded = OfflineQueue::load_from_disk(&queue_path).unwrap();
        assert_eq!(loaded.pending_count(), 1);
    }

    #[test]
    fn test_operation_builder() {
        let op = OperationBuilder::new("channel", "channel-123", "peer1").create(
            "channel".to_string(),
            serde_json::json!({"name": "General"}),
        );

        assert_eq!(op.entity_type, "channel");
        assert_eq!(op.entity_id, "channel-123");
        match op.operation {
            OperationType::Create { doc_type, .. } => {
                assert_eq!(doc_type, "channel");
            }
            _ => panic!("Expected Create operation"),
        }
    }

    #[test]
    fn test_retry_failed() {
        let mut queue = OfflineQueue::new();

        let op1 = OperationBuilder::new("channel", "channel-123", "peer1")
            .update("name".to_string(), serde_json::json!("Name1"));
        let op2 = OperationBuilder::new("channel", "channel-456", "peer1")
            .update("name".to_string(), serde_json::json!("Name2"));

        queue.enqueue(op1);
        queue.enqueue(op2);

        // Fail both operations max times
        for _ in 0..2 {
            for _ in 0..5 {
                let op = queue.dequeue().unwrap();
                queue.mark_failed(op);
            }
        }

        assert_eq!(queue.failed_count(), 2);
        assert_eq!(queue.pending_count(), 0);

        // Retry all failed
        queue.retry_failed();
        assert_eq!(queue.failed_count(), 0);
        assert_eq!(queue.pending_count(), 2);
    }
}
