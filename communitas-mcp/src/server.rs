// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Server implementation
//!
//! Handles JSON-RPC 2.0 communication over stdio, routing requests to the
//! CommunitasApp for processing.

use crate::protocol::{
    InitializeParams, InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, Resource,
    ResourceContent, ResourceListResult, ResourceReadParams, ResourceReadResult,
    ResourcesCapability, ServerCapabilities, ServerInfo, ToolCallParams, ToolListResult,
    ToolsCapability,
};
use crate::tools;
use anyhow::Result;
use communitas_core::app::CommunitasApp;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// MCP Server state
pub struct McpServer {
    /// The Communitas application instance
    app: Arc<RwLock<Option<CommunitasApp>>>,
    /// Whether the server has been initialized
    initialized: bool,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new() -> Self {
        Self {
            app: Arc::new(RwLock::new(None)),
            initialized: false,
        }
    }

    /// Handle a JSON-RPC request
    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        debug!(
            "Handling request: {} (id: {:?})",
            request.method, request.id
        );

        // Notifications (no id) don't get responses
        let is_notification = request.id.is_none();

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await,
            "initialized" => {
                // Notification - no response needed
                info!("Client confirmed initialization");
                return None;
            }
            "shutdown" => {
                info!("Shutdown requested");
                Ok(Value::Null)
            }
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => self.handle_tools_call(request.params).await,
            "resources/list" => self.handle_resources_list().await,
            "resources/read" => self.handle_resources_read(request.params).await,
            "ping" => Ok(Value::Object(serde_json::Map::new())),
            method => {
                warn!("Unknown method: {}", method);
                Err(JsonRpcError::method_not_found(method))
            }
        };

        // Notifications don't get responses
        if is_notification {
            return None;
        }

        Some(match result {
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(error) => JsonRpcResponse::error(request.id, error),
        })
    }

    /// Handle initialize request
    async fn handle_initialize(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: InitializeParams = params
            .map(|p| {
                serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
            })
            .transpose()?
            .ok_or_else(|| JsonRpcError::invalid_params("Missing initialize params"))?;

        info!(
            "Initialize request from {} v{} (protocol: {})",
            params.client_info.name, params.client_info.version, params.protocol_version
        );

        // Generate temporary identity for MCP session
        // TODO: Allow client to provide identity in params
        let four_words = format!(
            "mcp-{}-{}-{}",
            params.client_info.name.to_lowercase().replace(' ', "-"),
            uuid::Uuid::new_v4()
                .to_string()
                .split('-')
                .next()
                .unwrap_or("0000"),
            "agent"
        );
        let display_name = format!("MCP Agent ({})", params.client_info.name);
        let device_name = "mcp-server".to_string();
        let storage_dir = std::env::temp_dir()
            .join("communitas-mcp")
            .to_string_lossy()
            .to_string();

        // Initialize the CommunitasApp
        match CommunitasApp::new(four_words, display_name, device_name, storage_dir).await {
            Ok(app) => {
                let mut app_lock = self.app.write().await;
                *app_lock = Some(app);
                self.initialized = true;
                info!("CommunitasApp initialized successfully");
            }
            Err(e) => {
                error!("Failed to initialize CommunitasApp: {}", e);
                return Err(JsonRpcError::internal_error(&format!(
                    "Failed to initialize: {}",
                    e
                )));
            }
        }

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
                resources: Some(ResourcesCapability {
                    subscribe: false,
                    list_changed: false,
                }),
            },
            server_info: ServerInfo {
                name: "communitas-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let result = ToolListResult {
            tools: tools::list_tools(),
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let params: ToolCallParams = params
            .map(|p| {
                serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
            })
            .transpose()?
            .ok_or_else(|| JsonRpcError::invalid_params("Missing tool call params"))?;

        let app_lock = self.app.read().await;
        let app = app_lock
            .as_ref()
            .ok_or_else(|| JsonRpcError::internal_error("App not initialized"))?;

        let result = tools::call_tool(app, &params.name, params.arguments).await;

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self) -> Result<Value, JsonRpcError> {
        // Expose application state as resources
        let resources = vec![
            Resource {
                uri: "communitas://identity".to_string(),
                name: "Current Identity".to_string(),
                description: Some("The current user's identity and four-word address".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "communitas://entities".to_string(),
                name: "Entities".to_string(),
                description: Some(
                    "List of all entities (orgs, groups, channels, projects)".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "communitas://chats".to_string(),
                name: "Chats".to_string(),
                description: Some("List of all chat conversations".to_string()),
                mime_type: Some("application/json".to_string()),
            },
            Resource {
                uri: "communitas://invites".to_string(),
                name: "Invites".to_string(),
                description: Some("Pending invitations".to_string()),
                mime_type: Some("application/json".to_string()),
            },
        ];

        let result = ResourceListResult { resources };
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        if !self.initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let params: ResourceReadParams = params
            .map(|p| {
                serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
            })
            .transpose()?
            .ok_or_else(|| JsonRpcError::invalid_params("Missing resource read params"))?;

        let app_lock = self.app.read().await;
        let app = app_lock
            .as_ref()
            .ok_or_else(|| JsonRpcError::internal_error("App not initialized"))?;

        let content = match params.uri.as_str() {
            "communitas://identity" => {
                let response = app
                    .query(communitas_core::command::Query::GetProfile)
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.message))?;
                serde_json::to_string_pretty(&response)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
            }
            "communitas://entities" => {
                let response = app
                    .query(communitas_core::command::Query::ListEntities)
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.message))?;
                serde_json::to_string_pretty(&response)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
            }
            "communitas://chats" => {
                // Chats are entities of type Channel or Group - list all entities
                let response = app
                    .query(communitas_core::command::Query::ListEntities)
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.message))?;
                serde_json::to_string_pretty(&response)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
            }
            "communitas://invites" => {
                let response = app
                    .query(communitas_core::command::Query::ListPendingInvites)
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.message))?;
                serde_json::to_string_pretty(&response)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
            }
            uri => {
                return Err(JsonRpcError::invalid_params(&format!(
                    "Unknown resource: {}",
                    uri
                )));
            }
        };

        let result = ResourceReadResult {
            contents: vec![ResourceContent {
                uri: params.uri,
                mime_type: Some("application/json".to_string()),
                text: Some(content),
                blob: None,
            }],
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the MCP server
pub async fn run() -> Result<()> {
    let mut server = McpServer::new();

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    info!("MCP Server ready, waiting for requests on stdin");

    loop {
        line.clear();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF - client closed connection
                info!("Client disconnected (EOF)");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                debug!("Received: {}", trimmed);

                // Parse JSON-RPC request
                let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
                    Ok(req) => req,
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());
                        let json = serde_json::to_string(&response)?;
                        stdout.write_all(json.as_bytes()).await?;
                        stdout.write_all(b"\n").await?;
                        stdout.flush().await?;
                        continue;
                    }
                };

                // Handle the request
                if let Some(response) = server.handle_request(request).await {
                    let json = serde_json::to_string(&response)?;
                    debug!("Sending: {}", json);
                    stdout.write_all(json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    info!("MCP Server shutting down");
    Ok(())
}
