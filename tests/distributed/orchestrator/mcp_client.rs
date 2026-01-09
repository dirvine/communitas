// Copyright (c) 2025 Saorsa Labs Limited
//
// MCP Client
//
// HTTP client for communicating with communitas-mcp servers

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::debug;

/// MCP client for a specific node
#[allow(dead_code)]
pub struct McpClient {
    base_url: String,
    client: reqwest::Client,
}

impl McpClient {
    /// Create a new MCP client for a node
    pub fn new(host: &str, port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            base_url: format!("http://{}:{}", host, port),
            client,
        }
    }

    /// Check if the node is healthy
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/health", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send health check request")?;

        if !response.status().is_success() {
            anyhow::bail!("Health check failed with status: {}", response.status());
        }

        Ok(())
    }

    /// Call an MCP tool
    pub async fn call_tool(
        &self,
        tool: &str,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/mcp", self.base_url);

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/call".to_string(),
            params: McpCallParams {
                name: tool.to_string(),
                arguments: params.clone(),
            },
        };

        debug!("MCP request to {}: {:?}", url, request);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .with_context(|| format!("Failed to call tool '{}'", tool))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("MCP call failed ({}): {}", status, body);
        }

        let mcp_response: McpResponse = response
            .json()
            .await
            .with_context(|| format!("Failed to parse response from tool '{}'", tool))?;

        if let Some(error) = mcp_response.error {
            anyhow::bail!("MCP error ({}): {}", error.code, error.message);
        }

        Ok(mcp_response.result.unwrap_or(serde_json::Value::Null))
    }

    /// List all available tools
    #[allow(dead_code)]
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>> {
        let url = format!("{}/mcp", self.base_url);

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/list".to_string(),
            params: McpCallParams {
                name: String::new(),
                arguments: HashMap::new(),
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to list tools")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to list tools ({}): {}", status, body);
        }

        let mcp_response: McpResponse = response
            .json()
            .await
            .context("Failed to parse tools list response")?;

        let tools: ToolsListResult =
            serde_json::from_value(mcp_response.result.unwrap_or(serde_json::Value::Null))
                .context("Failed to parse tools list")?;

        Ok(tools.tools)
    }

    /// Get node status
    #[allow(dead_code)]
    pub async fn get_status(&self) -> Result<NodeStatus> {
        let params = HashMap::new();

        let result = self.call_tool("core_status", &params).await?;

        serde_json::from_value(result).context("Failed to parse node status")
    }

    /// Create a vault with authentication
    #[allow(dead_code)]
    pub async fn create_vault(
        &self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<AuthResponse> {
        let mut params = HashMap::new();
        params.insert("four_words".to_string(), serde_json::json!(four_words));
        params.insert("password".to_string(), serde_json::json!(password));
        params.insert("display_name".to_string(), serde_json::json!(display_name));

        let result = self.call_tool("create_vault", &params).await?;

        serde_json::from_value(result).context("Failed to parse create_vault response")
    }

    /// Authenticate with an existing vault
    #[allow(dead_code)]
    pub async fn authenticate(&self, four_words: &str, password: &str) -> Result<AuthResponse> {
        let mut params = HashMap::new();
        params.insert("four_words".to_string(), serde_json::json!(four_words));
        params.insert("password".to_string(), serde_json::json!(password));

        let result = self.call_tool("authenticate", &params).await?;

        serde_json::from_value(result).context("Failed to parse authenticate response")
    }

    /// Create an entity
    #[allow(dead_code)]
    pub async fn create_entity(&self, name: &str, entity_type: &str) -> Result<EntityResponse> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), serde_json::json!(name));
        params.insert("entity_type".to_string(), serde_json::json!(entity_type));

        let result = self.call_tool("create_entity", &params).await?;

        serde_json::from_value(result).context("Failed to parse create_entity response")
    }

    /// List entities
    #[allow(dead_code)]
    pub async fn list_entities(&self) -> Result<Vec<EntityResponse>> {
        let result = self.call_tool("list_entities", &HashMap::new()).await?;

        serde_json::from_value(result).context("Failed to parse list_entities response")
    }

    /// Send a message
    #[allow(dead_code)]
    pub async fn send_message(&self, entity_id: &str, text: &str) -> Result<MessageResponse> {
        let mut params = HashMap::new();
        params.insert("entity_id".to_string(), serde_json::json!(entity_id));
        params.insert("text".to_string(), serde_json::json!(text));

        let result = self.call_tool("send_message", &params).await?;

        serde_json::from_value(result).context("Failed to parse send_message response")
    }

    /// Get messages
    #[allow(dead_code)]
    pub async fn get_messages(&self, entity_id: &str) -> Result<Vec<MessageResponse>> {
        let mut params = HashMap::new();
        params.insert("entity_id".to_string(), serde_json::json!(entity_id));

        let result = self.call_tool("get_messages", &params).await?;

        serde_json::from_value(result).context("Failed to parse get_messages response")
    }

    /// Start the P2P network
    #[allow(dead_code)]
    pub async fn network_start(&self) -> Result<()> {
        self.call_tool("network_start", &HashMap::new()).await?;
        Ok(())
    }

    /// Stop the P2P network
    #[allow(dead_code)]
    pub async fn network_stop(&self) -> Result<()> {
        self.call_tool("network_stop", &HashMap::new()).await?;
        Ok(())
    }

    /// Get network peers
    #[allow(dead_code)]
    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>> {
        let result = self.call_tool("network_peers", &HashMap::new()).await?;

        serde_json::from_value(result).context("Failed to parse network_peers response")
    }
}

// MCP JSON-RPC request/response structures

#[derive(Debug, Serialize)]
struct McpRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: McpCallParams,
}

#[derive(Debug, Serialize)]
struct McpCallParams {
    name: String,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct McpResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<McpError>,
}

#[derive(Debug, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ToolsListResult {
    tools: Vec<ToolInfo>,
}

// Response types

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeStatus {
    pub authenticated: bool,
    pub network_active: bool,
    pub peer_count: usize,
    #[serde(default)]
    pub identity: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub identity: String,
    #[serde(default)]
    pub token: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    #[serde(default)]
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub text: String,
    pub sender: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_client_creation() {
        let client = McpClient::new("127.0.0.1", 3040);
        assert_eq!(client.base_url, "http://127.0.0.1:3040");
    }

    #[test]
    fn test_mcp_request_serialization() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), serde_json::json!("test"));

        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "tools/call".to_string(),
            params: McpCallParams {
                name: "create_entity".to_string(),
                arguments: args,
            },
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains("create_entity"));
        assert!(json.contains("test"));
    }
}
