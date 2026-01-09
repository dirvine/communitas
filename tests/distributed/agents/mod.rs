use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpNode {
    pub name: String,
    pub url: String,
    pub four_words: Option<String>,
    pub session_token: Option<String>,
}

impl McpNode {
    pub fn new(name: &str, host: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            url: format!("http://{}:{}/mcp", host, port),
            four_words: None,
            session_token: None,
        }
    }

    pub fn with_four_words(mut self, four_words: &str) -> Self {
        self.four_words = Some(four_words.to_string());
        self
    }
}

#[derive(Clone)]
pub struct McpClient {
    client: Client,
    node: McpNode,
    request_id: Arc<RwLock<u64>>,
}

impl McpClient {
    pub fn new(node: McpNode) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            node,
            request_id: Arc::new(RwLock::new(0)),
        }
    }

    pub fn node_name(&self) -> &str {
        &self.node.name
    }

    pub fn four_words(&self) -> Option<&str> {
        self.node.four_words.as_deref()
    }

    async fn next_id(&self) -> u64 {
        let mut id = self.request_id.write().await;
        *id += 1;
        *id
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id().await;
        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&self.node.url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send request")?;

        let body: Value = response.json().await.context("Failed to parse response")?;

        if let Some(error) = body.get("error") {
            anyhow::bail!("RPC Error: {}", error);
        }

        Ok(body)
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolResult> {
        let response = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments
                }),
            )
            .await?;

        let result = response.get("result").cloned().unwrap_or(json!(null));
        let is_error = result.get("isError").and_then(|v| v.as_bool()).unwrap_or(false);
        let content = result
            .get("content")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();

        let parsed: Option<Value> = serde_json::from_str(&content).ok();

        Ok(ToolResult {
            tool: name.to_string(),
            success: !is_error,
            content,
            parsed,
            raw: result,
        })
    }

    pub async fn initialize(&self) -> Result<Value> {
        self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": format!("test-{}", self.node.name),
                    "version": "1.0.0"
                }
            }),
        )
        .await
    }

    pub async fn list_tools(&self) -> Result<Vec<String>> {
        let response = self.request("tools/list", json!({})).await?;
        let tools = response
            .get("result")
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        Ok(tools)
    }

    pub async fn health_check(&self) -> Result<bool> {
        match self.call_tool("health_check", json!({})).await {
            Ok(result) => Ok(result.success),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool: String,
    pub success: bool,
    pub content: String,
    pub parsed: Option<Value>,
    pub raw: Value,
}

impl ToolResult {
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.parsed.as_ref().and_then(|p| p.get(key))
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    pub fn get_id(&self) -> Option<String> {
        self.get_string("id")
            .or_else(|| self.get_string("entity_id"))
            .or_else(|| self.get_string("board_id"))
            .or_else(|| self.get_string("card_id"))
            .or_else(|| self.get_string("column_id"))
            .or_else(|| self.get_string("message_id"))
    }
}

pub struct TestContext {
    pub clients: HashMap<String, McpClient>,
    pub shared_state: Arc<RwLock<SharedState>>,
}

#[derive(Default)]
pub struct SharedState {
    pub entities: HashMap<String, EntityInfo>,
    pub boards: HashMap<String, BoardInfo>,
    pub messages: Vec<MessageInfo>,
    pub files: Vec<FileInfo>,
    pub tool_coverage: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct EntityInfo {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub created_by: String,
}

#[derive(Debug, Clone)]
pub struct BoardInfo {
    pub id: String,
    pub entity_id: String,
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub cards: Vec<CardInfo>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub id: String,
    pub name: String,
    pub position: u32,
}

#[derive(Debug, Clone)]
pub struct CardInfo {
    pub id: String,
    pub column_id: String,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub id: String,
    pub entity_id: String,
    pub author: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub entity_id: String,
    pub path: String,
    pub disk_type: String,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            shared_state: Arc::new(RwLock::new(SharedState::default())),
        }
    }

    pub fn add_client(&mut self, node: McpNode) {
        let name = node.name.clone();
        self.clients.insert(name, McpClient::new(node));
    }

    pub fn get_client(&self, name: &str) -> Option<&McpClient> {
        self.clients.get(name)
    }

    pub async fn track_tool(&self, tool: &str) {
        let mut state = self.shared_state.write().await;
        *state.tool_coverage.entry(tool.to_string()).or_insert(0) += 1;
    }

    pub async fn get_coverage(&self) -> HashMap<String, usize> {
        self.shared_state.read().await.tool_coverage.clone()
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}
