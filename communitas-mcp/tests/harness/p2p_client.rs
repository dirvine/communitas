// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! P2P-enabled MCP Test Client
//!
//! Provides test infrastructure for multi-node P2P testing scenarios.
//! Wraps McpTestNode with additional P2P capabilities.

#![allow(dead_code)]

use super::client::{McpTestNode, ToolResult};
use super::test_config::bootstrap_nodes as get_bootstrap_nodes;
use serde_json::{Value, json};
use std::time::Duration;
use tokio::time::sleep;

/// Bootstrap nodes for P2P testing (legacy constant for compatibility)
pub const BOOTSTRAP_NODES: &[&str] = &[
    "142.93.199.50:11000",   // saorsa-2 (NYC1)
    "147.182.234.192:11000", // saorsa-3 (SFO3)
];

/// Get bootstrap nodes with CI fallback support
/// Uses localhost fallback when MCP_TEST_LOCALHOST_FALLBACK=true
pub fn bootstrap_nodes_with_fallback() -> Vec<String> {
    get_bootstrap_nodes()
}

/// P2P-enabled test node with additional networking capabilities
pub struct P2pTestNode {
    inner: McpTestNode,
    four_words: Option<String>,
    connected_peers: Vec<String>,
}

impl P2pTestNode {
    /// Create a new P2P test node
    pub async fn new(name: &str) -> Self {
        let inner = McpTestNode::start(name).await;
        Self {
            inner,
            four_words: None,
            connected_peers: Vec::new(),
        }
    }

    /// Create a P2P node and start networking
    pub async fn start_connected(name: &str) -> Result<Self, String> {
        let mut node = Self::new(name).await;
        node.inner.initialize().await;

        // Start network
        let result = node.call_tool("network_start", json!({})).await;
        if !result.success {
            return Err(format!("Failed to start network: {}", result.content));
        }

        // Get our connection words (fallback to connection identity)
        let result = node.call_tool("get_connection_words", json!({})).await;
        if result.success
            && let Some(words) = result.get_str("connection_words")
        {
            node.four_words = Some(words.to_string());
        }
        if node.four_words.is_none() {
            let status = node.call_tool("network_status", json!({})).await;
            if status.success
                && let Some(identity) = status.get_str("connection_identity")
            {
                node.four_words = Some(identity.to_string());
            }
        }

        Ok(node)
    }

    /// Connect to bootstrap nodes (with CI fallback support)
    ///
    /// Tries the real bootstrap nodes first (saorsa-2/3), then falls back
    /// to localhost if configured via MCP_TEST_LOCALHOST_FALLBACK=true.
    pub async fn connect_to_bootstrap(&mut self) -> Result<(), String> {
        let nodes = bootstrap_nodes_with_fallback();

        for bootstrap in &nodes {
            let result = self
                .call_tool(
                    "network_connect",
                    json!({
                        "address": bootstrap
                    }),
                )
                .await;

            if result.success {
                return Ok(());
            }
        }

        Err(format!(
            "Failed to connect to any bootstrap node. Tried: {:?}",
            nodes
        ))
    }

    /// Connect to bootstrap nodes using only real nodes (no fallback)
    pub async fn connect_to_bootstrap_no_fallback(&mut self) -> Result<(), String> {
        for bootstrap in BOOTSTRAP_NODES {
            let result = self
                .call_tool(
                    "network_connect",
                    json!({
                        "address": bootstrap
                    }),
                )
                .await;

            if result.success {
                return Ok(());
            }
        }

        Err("Failed to connect to any real bootstrap node".to_string())
    }

    /// Connect to another P2P node by four words
    pub async fn connect_to(&mut self, other: &P2pTestNode) -> Result<(), String> {
        let other_words = other
            .four_words()
            .ok_or_else(|| "Other node has no four words".to_string())?;

        let result = self
            .call_tool(
                "connect_by_words",
                json!({
                    "words": other_words
                }),
            )
            .await;

        if result.success {
            self.connected_peers.push(other_words.to_string());
            Ok(())
        } else {
            Err(format!("Failed to connect: {}", result.content))
        }
    }

    /// Wait for a peer to be discovered
    pub async fn wait_for_peer(&self, peer_words: &str, timeout: Duration) -> Result<(), String> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            let result = self.call_tool("network_peers", json!({})).await;

            if result.success
                && let Some(peers) = result.get_array("peers")
            {
                for peer in peers {
                    if let Some(words) = peer.get("four_words").and_then(|v| v.as_str())
                        && words == peer_words
                    {
                        return Ok(());
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        Err(format!("Peer {} not found after {:?}", peer_words, timeout))
    }

    /// Get our four words
    pub fn four_words(&self) -> Option<&str> {
        self.four_words.as_deref()
    }

    /// Get the inner node
    pub fn inner(&self) -> &McpTestNode {
        &self.inner
    }

    /// Get the node name
    pub fn name(&self) -> &str {
        self.inner.name()
    }

    /// Get the node port
    pub fn port(&self) -> u16 {
        self.inner.port()
    }

    /// Call a tool on the node
    pub async fn call_tool(&self, name: &str, arguments: Value) -> ToolResult {
        self.inner.call_tool(name, arguments).await
    }

    /// Initialize the MCP connection
    pub async fn initialize(&self) -> Value {
        self.inner.initialize().await
    }

    /// Send a raw JSON-RPC request
    pub async fn request(&self, method: &str, params: Value) -> Value {
        self.inner.request(method, params).await
    }

    /// List available tools
    pub async fn list_tools(&self) -> Vec<String> {
        self.inner.list_tools().await
    }

    /// Announce presence to the network
    pub async fn announce_presence(&self, status: &str) -> Result<(), String> {
        let result = self
            .call_tool(
                "announce_presence",
                json!({
                    "status": status
                }),
            )
            .await;

        if result.success {
            Ok(())
        } else {
            Err(format!("Failed to announce presence: {}", result.content))
        }
    }

    /// Query presence of other peers
    pub async fn query_presence(&self, four_words_list: &[&str]) -> Result<Vec<Value>, String> {
        let result = self
            .call_tool(
                "query_presence",
                json!({
                    "four_words_list": four_words_list
                }),
            )
            .await;

        if result.success {
            Ok(result.get_array("presences").cloned().unwrap_or_default())
        } else {
            Err(format!("Failed to query presence: {}", result.content))
        }
    }

    /// Get network status
    pub async fn network_status(&self) -> Result<Value, String> {
        let result = self.call_tool("network_status", json!({})).await;

        if result.success {
            Ok(result.parsed.unwrap_or(json!({})))
        } else {
            Err(format!("Failed to get network status: {}", result.content))
        }
    }

    /// Stop the network
    pub async fn stop_network(&self) -> Result<(), String> {
        let result = self.call_tool("network_stop", json!({})).await;

        if result.success {
            Ok(())
        } else {
            Err(format!("Failed to stop network: {}", result.content))
        }
    }
}

/// Builder for creating P2P test scenarios
pub struct P2pTestScenario {
    nodes: Vec<P2pTestNode>,
}

impl P2pTestScenario {
    /// Create a new test scenario
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Add a node to the scenario
    pub async fn add_node(&mut self, name: &str) -> Result<usize, String> {
        let node = P2pTestNode::start_connected(name).await?;
        let index = self.nodes.len();
        self.nodes.push(node);
        Ok(index)
    }

    /// Connect all nodes to each other
    pub async fn mesh_connect(&mut self) -> Result<(), String> {
        let words: Vec<Option<String>> = self
            .nodes
            .iter()
            .map(|n| n.four_words().map(|s| s.to_string()))
            .collect();

        for i in 0..self.nodes.len() {
            for (j, word_opt) in words.iter().enumerate().skip(i + 1) {
                if let Some(target_words) = word_opt {
                    let result = self.nodes[i]
                        .call_tool(
                            "connect_by_words",
                            json!({
                                "words": target_words
                            }),
                        )
                        .await;

                    if !result.success {
                        return Err(format!(
                            "Failed to connect node {} to node {}: {}",
                            i, j, result.content
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a reference to a node
    pub fn node(&self, index: usize) -> Option<&P2pTestNode> {
        self.nodes.get(index)
    }

    /// Get a mutable reference to a node
    pub fn node_mut(&mut self, index: usize) -> Option<&mut P2pTestNode> {
        self.nodes.get_mut(index)
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[P2pTestNode] {
        &self.nodes
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for P2pTestScenario {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_p2p_node_creation() {
        let node = P2pTestNode::new("test-node").await;
        assert_eq!(node.name(), "test-node");
    }

    #[tokio::test]
    async fn test_scenario_builder() {
        let scenario = P2pTestScenario::new();
        assert_eq!(scenario.node_count(), 0);
    }
}
