// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! HTTP/HTTPS transport for MCP server
//!
//! Provides an HTTP-based JSON-RPC 2.0 endpoint as an alternative to stdio.
//! Supports both plain HTTP (for development) and HTTPS with RFC 7250 Raw Public Keys.

use base64::Engine;
use crate::auth::{requires_auth, AuthState, AuthenticatedSession, DemoSession};
use crate::protocol::{
    InitializeParams, InitializeResult, JsonRpcError, JsonRpcRequest, JsonRpcResponse, Resource,
    ResourceContent, ResourceListResult, ResourceReadParams, ResourceReadResult,
    ResourcesCapability, ServerCapabilities, ServerInfo, ToolCallParams, ToolListResult,
    ToolsCapability,
};
use crate::tls::{ServerTlsConfig, ServerTlsConfigBuilder, TlsConfigError};
use crate::tools;
use crate::Args;
use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::post,
};
use communitas_core::app::CommunitasApp;
use communitas_core::identity::generate_id_words;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

/// HTTP server state
pub struct HttpServerState {
    /// The Communitas application instance
    app: RwLock<Option<CommunitasApp>>,
    /// Authentication state
    auth_state: RwLock<AuthState>,
    /// Whether running in demo mode (for future get_session API)
    #[allow(dead_code)]
    demo_mode: bool,
    /// CLI arguments
    args: Args,
    /// Whether MCP protocol has been initialized
    protocol_initialized: RwLock<bool>,
}

impl HttpServerState {
    /// Create a new HTTP server state
    pub fn new(args: Args) -> Self {
        Self {
            app: RwLock::new(None),
            auth_state: RwLock::new(AuthState::Unauthenticated),
            demo_mode: args.demo,
            args,
            protocol_initialized: RwLock::new(false),
        }
    }

    /// Check if authenticated
    async fn is_authenticated(&self) -> bool {
        let auth_state = self.auth_state.read().await;
        matches!(
            *auth_state,
            AuthState::Authenticated(_) | AuthState::DemoMode(_)
        )
    }
}

/// Run HTTP server (plain HTTP for development)
pub async fn run_http(args: Args) -> Result<()> {
    let addr: SocketAddr = args
        .listen
        .clone()
        .unwrap_or_else(|| "127.0.0.1:3040".to_string())
        .parse()?;

    let state = Arc::new(HttpServerState::new(args.clone()));

    // Initialize demo mode if enabled
    if args.demo {
        initialize_demo_mode(&state).await?;
    }

    let app = create_router(state);

    info!("MCP HTTP server listening on http://{}", addr);
    warn!("Running in plain HTTP mode - use --tls for secure connections");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Run HTTPS server with RFC 7250 Raw Public Keys
pub async fn run_https(args: Args, tls_config: ServerTlsConfig) -> Result<()> {
    let addr: SocketAddr = args
        .listen
        .clone()
        .unwrap_or_else(|| "127.0.0.1:3040".to_string())
        .parse()?;

    let state = Arc::new(HttpServerState::new(args.clone()));

    // Initialize demo mode if enabled
    if args.demo {
        initialize_demo_mode(&state).await?;
    }

    let app = create_router(state);

    info!("MCP HTTPS server listening on https://{}", addr);
    info!("Using RFC 7250 Raw Public Keys with ML-DSA-65");

    // Create axum-server with TLS
    let rustls_config = tls_config.into_inner();
    axum_server::bind_rustls(addr, axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(rustls_config)))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Create TLS configuration for the server
pub fn create_tls_config(args: &Args) -> Result<ServerTlsConfig, TlsConfigError> {
    // Generate or load keypair
    let (public_key, secret_key) = crate::tls::generate_keypair()?;

    // Log the server's public key for client configuration
    let spki = crate::tls::create_spki(&public_key)?;
    let spki_base64 = base64::engine::general_purpose::STANDARD.encode(&spki);
    info!("Server public key (SPKI base64): {}", spki_base64);

    let mut builder = ServerTlsConfigBuilder::new().with_keypair(secret_key, public_key);

    // In demo mode, allow any client
    if args.demo {
        warn!("Demo mode: accepting any client key (insecure)");
        builder = builder.allow_any_client();
    }

    // If no-client-auth is specified, disable mutual TLS
    if args.no_client_auth {
        builder = builder.no_client_auth();
    }

    builder.build()
}

/// Create the Axum router
fn create_router(state: Arc<HttpServerState>) -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/health", axum::routing::get(handle_health))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

/// Handle MCP JSON-RPC request
async fn handle_mcp_request(
    State(state): State<Arc<HttpServerState>>,
    Json(request): Json<JsonRpcRequest>,
) -> (StatusCode, Json<JsonRpcResponse>) {
    match handle_request_inner(&state, request).await {
        Ok(Some(response)) => (StatusCode::OK, Json(response)),
        Ok(None) => {
            // Notification - return empty success
            (
                StatusCode::OK,
                Json(JsonRpcResponse::success(None, Value::Null)),
            )
        }
        Err(error) => (
            StatusCode::OK, // JSON-RPC errors use 200 status
            Json(JsonRpcResponse::error(None, error)),
        ),
    }
}

/// Handle health check
async fn handle_health() -> &'static str {
    "OK"
}

/// Internal request handler (shared logic with stdio server)
async fn handle_request_inner(
    state: &HttpServerState,
    request: JsonRpcRequest,
) -> Result<Option<JsonRpcResponse>, JsonRpcError> {
    let is_notification = request.id.is_none();

    let result = match request.method.as_str() {
        "initialize" => handle_initialize(state, request.params).await,
        "initialized" => {
            info!("Client confirmed initialization");
            return Ok(None);
        }
        "shutdown" => {
            info!("Shutdown requested");
            Ok(Value::Null)
        }
        "tools/list" => handle_tools_list(state).await,
        "tools/call" => handle_tools_call(state, request.params).await,
        "resources/list" => handle_resources_list().await,
        "resources/read" => handle_resources_read(state, request.params).await,
        "ping" => Ok(Value::Object(serde_json::Map::new())),
        method => {
            warn!("Unknown method: {}", method);
            Err(JsonRpcError::method_not_found(method))
        }
    };

    if is_notification {
        return Ok(None);
    }

    Ok(Some(match result {
        Ok(value) => JsonRpcResponse::success(request.id, value),
        Err(error) => JsonRpcResponse::error(request.id, error),
    }))
}

/// Initialize demo mode
async fn initialize_demo_mode(state: &HttpServerState) -> Result<()> {
    let four_words = match &state.args.four_words {
        Some(fw) => fw.clone(),
        None => generate_id_words().map_err(|e| anyhow::anyhow!("Failed to generate identity: {}", e))?,
    };

    let display_name = state.args.display_name.clone();
    let device_name = "mcp-http-demo".to_string();
    let storage_dir = state
        .args
        .storage_dir
        .clone()
        .unwrap_or_else(|| {
            std::env::temp_dir()
                .join("communitas-mcp-http-demo")
                .to_string_lossy()
                .to_string()
        });

    info!(
        "HTTP Demo mode: initializing with identity {} in {}",
        four_words, storage_dir
    );

    let app = CommunitasApp::new(
        four_words.clone(),
        display_name.clone(),
        device_name,
        storage_dir.clone(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to initialize app: {}", e))?;

    let mut app_lock = state.app.write().await;
    *app_lock = Some(app);

    let mut auth_lock = state.auth_state.write().await;
    *auth_lock = AuthState::DemoMode(DemoSession {
        four_words,
        display_name,
        started_at: SystemTime::now(),
        storage_dir,
    });

    let mut init_lock = state.protocol_initialized.write().await;
    *init_lock = true;

    info!("HTTP Demo mode initialized successfully");
    Ok(())
}

// =============================================================================
// Request Handlers (similar to stdio server)
// =============================================================================

async fn handle_initialize(
    state: &HttpServerState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
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

    let mut init_lock = state.protocol_initialized.write().await;
    *init_lock = true;

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
            name: "communitas-mcp-http".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
}

async fn handle_tools_list(state: &HttpServerState) -> Result<Value, JsonRpcError> {
    let result = ToolListResult {
        tools: tools::list_tools(state.is_authenticated().await),
    };
    serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
}

async fn handle_tools_call(
    state: &HttpServerState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let is_initialized = *state.protocol_initialized.read().await;
    if !is_initialized {
        return Err(JsonRpcError::invalid_request("Server not initialized"));
    }

    let params: ToolCallParams = params
        .map(|p| {
            serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
        })
        .transpose()?
        .ok_or_else(|| JsonRpcError::invalid_params("Missing tool call params"))?;

    let tool_requires_auth = requires_auth(&params.name);

    // Handle pre-auth tools
    if !tool_requires_auth {
        return handle_pre_auth_tool(state, &params.name, params.arguments).await;
    }

    if !state.is_authenticated().await {
        return Err(JsonRpcError::invalid_request(
            "Authentication required. Use 'authenticate', 'create_vault', or restart with --demo flag.",
        ));
    }

    let app_lock = state.app.read().await;
    let app = app_lock
        .as_ref()
        .ok_or_else(|| JsonRpcError::internal_error("App not initialized"))?;

    let result = tools::call_tool(app, &params.name, params.arguments).await;
    serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
}

async fn handle_pre_auth_tool(
    state: &HttpServerState,
    name: &str,
    args: Option<Value>,
) -> Result<Value, JsonRpcError> {
    let args = args.unwrap_or(serde_json::json!({}));

    match name {
        "health_check" => {
            let result = tools::success_result("MCP HTTP service is healthy");
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        "core_status" => {
            let initialized = state.is_authenticated().await;
            let result = serde_json::json!({
                "content": [{"type": "text", "text": serde_json::to_string(&serde_json::json!({"initialized": initialized})).unwrap_or_default()}],
                "isError": false
            });
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        "authenticate" => handle_authenticate(state, args).await,
        "create_vault" => handle_create_vault(state, args).await,
        "authenticate_token" => {
            Err(JsonRpcError::internal_error(
                "Delegate token authentication not yet implemented",
            ))
        }
        _ => Err(JsonRpcError::method_not_found(name)),
    }
}

async fn handle_authenticate(
    state: &HttpServerState,
    args: Value,
) -> Result<Value, JsonRpcError> {
    let four_words = args["four_words"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?
        .to_string();
    let _password = args["password"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?;
    let device_name = args["device_name"]
        .as_str()
        .unwrap_or("mcp-http-client")
        .to_string();

    // Validate four-word format (accepts dots or dashes as separator)
    let words: Vec<&str> = if four_words.contains('.') {
        four_words.split('.').collect()
    } else {
        four_words.split('-').collect()
    };
    if words.len() != 4 {
        return Err(JsonRpcError::invalid_params(
            "Invalid four-word format. Expected: word1.word2.word3.word4 or word1-word2-word3-word4",
        ));
    }

    // Convert to dash-separated format for core (which expects dashes)
    let four_words_dashed = words.join("-");

    let storage_dir = std::env::temp_dir()
        .join("communitas-mcp-http")
        .join(&four_words_dashed)
        .to_string_lossy()
        .to_string();

    info!("HTTP: Authenticating user: {}", four_words_dashed);

    match CommunitasApp::new(
        four_words_dashed.clone(),
        four_words_dashed.clone(),
        device_name.clone(),
        storage_dir.clone(),
    )
    .await
    {
        Ok(app) => {
            let mut app_lock = state.app.write().await;
            *app_lock = Some(app);

            let mut auth_lock = state.auth_state.write().await;
            *auth_lock = AuthState::Authenticated(AuthenticatedSession {
                four_words: four_words_dashed.clone(),
                display_name: four_words_dashed.clone(),
                device_name,
                started_at: SystemTime::now(),
                storage_dir,
            });

            info!("HTTP: User authenticated successfully: {}", four_words_dashed);

            let result = tools::success_result("Authentication successful");
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        Err(e) => {
            error!("HTTP: Authentication failed: {}", e);
            Err(JsonRpcError::internal_error(&format!(
                "Authentication failed: {}",
                e
            )))
        }
    }
}

async fn handle_create_vault(
    state: &HttpServerState,
    args: Value,
) -> Result<Value, JsonRpcError> {
    let four_words = args["four_words"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?
        .to_string();
    let _password = args["password"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?;
    let display_name = args["display_name"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing display_name"))?
        .to_string();

    // Validate four-word format (accepts dots or dashes as separator)
    let words: Vec<&str> = if four_words.contains('.') {
        four_words.split('.').collect()
    } else {
        four_words.split('-').collect()
    };
    if words.len() != 4 {
        return Err(JsonRpcError::invalid_params(
            "Invalid four-word format. Expected: word1.word2.word3.word4 or word1-word2-word3-word4",
        ));
    }

    // Convert to dash-separated format for core (which expects dashes)
    let four_words_dashed = words.join("-");

    let device_name = "mcp-http-client".to_string();
    let storage_dir = std::env::temp_dir()
        .join("communitas-mcp-http")
        .join(&four_words_dashed)
        .to_string_lossy()
        .to_string();

    info!("HTTP: Creating new vault for: {}", four_words_dashed);

    match CommunitasApp::new(
        four_words_dashed.clone(),
        display_name.clone(),
        device_name.clone(),
        storage_dir.clone(),
    )
    .await
    {
        Ok(app) => {
            let mut app_lock = state.app.write().await;
            *app_lock = Some(app);

            let mut auth_lock = state.auth_state.write().await;
            *auth_lock = AuthState::Authenticated(AuthenticatedSession {
                four_words: four_words_dashed.clone(),
                display_name,
                device_name,
                started_at: SystemTime::now(),
                storage_dir,
            });

            info!("HTTP: Vault created successfully for: {}", four_words_dashed);

            let result = tools::success_result("Vault created and authenticated successfully");
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        Err(e) => {
            error!("HTTP: Failed to create vault: {}", e);
            Err(JsonRpcError::internal_error(&format!(
                "Failed to create vault: {}",
                e
            )))
        }
    }
}

async fn handle_resources_list() -> Result<Value, JsonRpcError> {
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
        Resource {
            uri: "communitas://contacts".to_string(),
            name: "Contacts".to_string(),
            description: Some("All contacts".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "communitas://network".to_string(),
            name: "Network Status".to_string(),
            description: Some("P2P network status and connected peers".to_string()),
            mime_type: Some("application/json".to_string()),
        },
    ];

    let result = ResourceListResult { resources };
    serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
}

async fn handle_resources_read(
    state: &HttpServerState,
    params: Option<Value>,
) -> Result<Value, JsonRpcError> {
    if !state.is_authenticated().await {
        return Err(JsonRpcError::invalid_request(
            "Authentication required to read resources",
        ));
    }

    let params: ResourceReadParams = params
        .map(|p| {
            serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
        })
        .transpose()?
        .ok_or_else(|| JsonRpcError::invalid_params("Missing resource read params"))?;

    let app_lock = state.app.read().await;
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
        "communitas://entities" | "communitas://chats" => {
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
        "communitas://contacts" => serde_json::json!({"contacts": []}).to_string(),
        "communitas://network" => serde_json::json!({
            "status": "not_started",
            "peers": [],
            "connection_info": null
        })
        .to_string(),
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
