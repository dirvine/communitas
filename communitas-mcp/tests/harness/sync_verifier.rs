// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Sync Verification
//!
//! Provides utilities for verifying that CRDT data has synchronized
//! correctly between multiple nodes in P2P tests.

#![allow(dead_code)]

use super::p2p_client::P2pTestNode;
use super::test_config::{poll_interval, sync_timeout};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

/// Result of a sync verification
#[derive(Debug, Clone)]
pub enum SyncResult {
    /// Data synchronized successfully
    Synced,
    /// Sync timed out
    Timeout { waited: Duration },
    /// Sync failed with error
    Failed { reason: String },
    /// Partial sync (some nodes have data, some don't)
    Partial { synced_count: usize, total: usize },
}

impl SyncResult {
    /// Check if sync was successful
    pub fn is_synced(&self) -> bool {
        matches!(self, SyncResult::Synced)
    }

    /// Unwrap or panic with message
    pub fn expect(self, msg: &str) {
        match self {
            SyncResult::Synced => {}
            other => panic!("{}: {:?}", msg, other),
        }
    }
}

/// Verifier for CRDT synchronization between nodes
pub struct CrdtSyncVerifier<'a> {
    nodes: Vec<&'a P2pTestNode>,
    timeout: Duration,
    poll_interval: Duration,
}

impl<'a> CrdtSyncVerifier<'a> {
    /// Create a new sync verifier with configurable defaults from environment
    pub fn new(nodes: Vec<&'a P2pTestNode>) -> Self {
        Self {
            nodes,
            timeout: sync_timeout(),
            poll_interval: poll_interval(),
        }
    }

    /// Set the timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the poll interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Wait for an entity to be visible on all nodes
    pub async fn wait_for_entity_sync(&self, entity_id: &str) -> SyncResult {
        let start = std::time::Instant::now();

        while start.elapsed() < self.timeout {
            let mut synced_count = 0;

            for node in &self.nodes {
                let result = node
                    .call_tool(
                        "get_entity",
                        json!({
                            "entity_id": entity_id
                        }),
                    )
                    .await;

                if result.success {
                    synced_count += 1;
                }
            }

            if synced_count == self.nodes.len() {
                return SyncResult::Synced;
            }

            sleep(self.poll_interval).await;
        }

        // Check final state
        let mut synced_count = 0;
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "get_entity",
                    json!({
                        "entity_id": entity_id
                    }),
                )
                .await;

            if result.success {
                synced_count += 1;
            }
        }

        if synced_count == self.nodes.len() {
            SyncResult::Synced
        } else if synced_count > 0 {
            SyncResult::Partial {
                synced_count,
                total: self.nodes.len(),
            }
        } else {
            SyncResult::Timeout {
                waited: self.timeout,
            }
        }
    }

    /// Verify that all nodes have the same message count for an entity
    pub async fn verify_message_count(&self, entity_id: &str, expected: usize) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "get_messages",
                    json!({
                        "entity_id": entity_id
                    }),
                )
                .await;

            if !result.success {
                return false;
            }

            let count = result.get_array("messages").map(|a| a.len()).unwrap_or(0);

            if count != expected {
                return false;
            }
        }

        true
    }

    /// Wait for message count to match expected on all nodes
    pub async fn wait_for_message_count(&self, entity_id: &str, expected: usize) -> SyncResult {
        let start = std::time::Instant::now();

        while start.elapsed() < self.timeout {
            if self.verify_message_count(entity_id, expected).await {
                return SyncResult::Synced;
            }

            sleep(self.poll_interval).await;
        }

        SyncResult::Timeout {
            waited: self.timeout,
        }
    }

    /// Verify that a Kanban board exists on all nodes
    pub async fn verify_kanban_board_sync(&self, board_id: &str) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "get_kanban_board",
                    json!({
                        "board_id": board_id
                    }),
                )
                .await;

            if !result.success {
                return false;
            }
        }

        true
    }

    /// Wait for a Kanban board to sync to all nodes
    pub async fn wait_for_kanban_board_sync(&self, board_id: &str) -> SyncResult {
        let start = std::time::Instant::now();

        while start.elapsed() < self.timeout {
            if self.verify_kanban_board_sync(board_id).await {
                return SyncResult::Synced;
            }

            sleep(self.poll_interval).await;
        }

        SyncResult::Timeout {
            waited: self.timeout,
        }
    }

    /// Verify that a Kanban card exists on all nodes
    pub async fn verify_kanban_card_sync(&self, board_id: &str, card_id: &str) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "get_kanban_card",
                    json!({
                        "board_id": board_id,
                        "card_id": card_id
                    }),
                )
                .await;

            if !result.success {
                return false;
            }
        }

        true
    }

    /// Verify that all nodes have the same card count on a board
    pub async fn verify_kanban_card_count(&self, board_id: &str, expected: usize) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "list_kanban_cards",
                    json!({
                        "board_id": board_id
                    }),
                )
                .await;

            if !result.success {
                return false;
            }

            let count = result.get_array("cards").map(|a| a.len()).unwrap_or(0);

            if count != expected {
                return false;
            }
        }

        true
    }

    /// Verify that a file exists on all nodes
    pub async fn verify_file_sync(&self, entity_id: &str, disk_type: &str, path: &str) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "read_file",
                    json!({
                        "entity_id": entity_id,
                        "disk_type": disk_type,
                        "path": path
                    }),
                )
                .await;

            if !result.success {
                return false;
            }
        }

        true
    }

    /// Wait for a file to sync to all nodes
    pub async fn wait_for_file_sync(
        &self,
        entity_id: &str,
        disk_type: &str,
        path: &str,
    ) -> SyncResult {
        let start = std::time::Instant::now();

        while start.elapsed() < self.timeout {
            if self.verify_file_sync(entity_id, disk_type, path).await {
                return SyncResult::Synced;
            }

            sleep(self.poll_interval).await;
        }

        SyncResult::Timeout {
            waited: self.timeout,
        }
    }

    /// Verify that file content matches on all nodes
    pub async fn verify_file_content(
        &self,
        entity_id: &str,
        disk_type: &str,
        path: &str,
        expected_content: &str,
    ) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "read_file",
                    json!({
                        "entity_id": entity_id,
                        "disk_type": disk_type,
                        "path": path
                    }),
                )
                .await;

            if !result.success {
                return false;
            }

            let content = result.get_str("content").unwrap_or("");
            if content != expected_content {
                return false;
            }
        }

        true
    }

    /// Verify that a contact exists on all nodes
    pub async fn verify_contact_sync(&self, contact_id: &str) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "get_contact",
                    json!({
                        "contact_id": contact_id
                    }),
                )
                .await;

            if !result.success {
                return false;
            }
        }

        true
    }

    /// Get the member list from a node
    pub async fn get_members(
        &self,
        node_index: usize,
        entity_id: &str,
        entity_type: &str,
    ) -> Option<Vec<Value>> {
        if node_index >= self.nodes.len() {
            return None;
        }

        let result = self.nodes[node_index]
            .call_tool(
                "list_members",
                json!({
                    "entity_id": entity_id,
                    "entity_type": entity_type
                }),
            )
            .await;

        if result.success {
            result.get_array("members").cloned()
        } else {
            None
        }
    }

    /// Verify member count matches on all nodes
    pub async fn verify_member_count(
        &self,
        entity_id: &str,
        entity_type: &str,
        expected: usize,
    ) -> bool {
        for node in &self.nodes {
            let result = node
                .call_tool(
                    "list_members",
                    json!({
                        "entity_id": entity_id,
                        "entity_type": entity_type
                    }),
                )
                .await;

            if !result.success {
                return false;
            }

            let count = result.get_array("members").map(|a| a.len()).unwrap_or(0);

            if count != expected {
                return false;
            }
        }

        true
    }
}

/// Helper for comparing CRDT states between nodes
pub struct CrdtStateComparator;

impl CrdtStateComparator {
    /// Compare entity states between two nodes
    pub async fn compare_entities(
        node_a: &P2pTestNode,
        node_b: &P2pTestNode,
        entity_type: &str,
    ) -> bool {
        let result_a = node_a
            .call_tool(
                "list_entities",
                json!({
                    "entity_type": entity_type
                }),
            )
            .await;

        let result_b = node_b
            .call_tool(
                "list_entities",
                json!({
                    "entity_type": entity_type
                }),
            )
            .await;

        if !result_a.success || !result_b.success {
            return false;
        }

        let entities_a = result_a.get_array("entities").map(|a| a.len()).unwrap_or(0);
        let entities_b = result_b.get_array("entities").map(|a| a.len()).unwrap_or(0);

        entities_a == entities_b
    }

    /// Compare Kanban board states between two nodes
    pub async fn compare_kanban_boards(
        node_a: &P2pTestNode,
        node_b: &P2pTestNode,
        entity_id: &str,
    ) -> bool {
        let result_a = node_a
            .call_tool(
                "list_kanban_boards",
                json!({
                    "entity_id": entity_id
                }),
            )
            .await;

        let result_b = node_b
            .call_tool(
                "list_kanban_boards",
                json!({
                    "entity_id": entity_id
                }),
            )
            .await;

        if !result_a.success || !result_b.success {
            return false;
        }

        let boards_a = result_a.get_array("boards").map(|a| a.len()).unwrap_or(0);
        let boards_b = result_b.get_array("boards").map(|a| a.len()).unwrap_or(0);

        boards_a == boards_b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result() {
        let synced = SyncResult::Synced;
        assert!(synced.is_synced());

        let timeout = SyncResult::Timeout {
            waited: Duration::from_secs(30),
        };
        assert!(!timeout.is_synced());

        let partial = SyncResult::Partial {
            synced_count: 1,
            total: 2,
        };
        assert!(!partial.is_synced());
    }
}
