// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! HTTP/HTTPS transport for MCP server
//!
//! Provides an HTTP-based JSON-RPC 2.0 endpoint as an alternative to stdio.
//! Supports both plain HTTP (for development) and HTTPS with RFC 7250 Raw Public Keys.

use crate::Args;
use crate::protocol::{
    InitializeParams, InitializeResultWithExtensions, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, Resource, ResourceContent, ResourceListResult, ResourceReadParams,
    ResourceReadResult, ResourcesCapability, ServerInfo, ToolCallParams, ToolListResult,
    ToolsCapability,
};
use crate::tls::{ServerTlsConfig, ServerTlsConfigBuilder, TlsConfigError};
use crate::tools;
use crate::ui_resources::UiResourceRegistry;
use anyhow::Result;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::post,
};
use base64::Engine;
use communitas_core::app::CommunitasApp;
use communitas_core::identity::generate_id_words;
use communitas_mcp::auth::{
    AuthState, AuthenticatedSession, DelegateSession, DemoSession, requires_auth,
};
use communitas_mcp::token::TokenManager;
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

/// HTTP server state
pub struct HttpServerState {
    /// The Communitas application instance (Arc for sharing with UiServices)
    app: RwLock<Option<Arc<CommunitasApp>>>,
    /// UI services for MCP-Dioxus parity
    services: RwLock<Option<UiServices>>,
    /// Authentication state
    auth_state: RwLock<AuthState>,
    /// CLI arguments
    args: Args,
    /// Whether MCP protocol has been initialized
    protocol_initialized: RwLock<bool>,
    /// Token manager for delegate authentication
    token_manager: RwLock<Option<TokenManager>>,
    /// UI resource registry for MCP Apps extension
    ui_resources: UiResourceRegistry,
}

impl HttpServerState {
    pub fn new(args: Args) -> Self {
        Self {
            app: RwLock::new(None),
            services: RwLock::new(None),
            auth_state: RwLock::new(AuthState::Unauthenticated),
            args,
            protocol_initialized: RwLock::new(false),
            token_manager: RwLock::new(None),
            ui_resources: UiResourceRegistry::with_standard_widgets(),
        }
    }

    async fn is_authenticated(&self) -> bool {
        let auth_state = self.auth_state.read().await;
        matches!(
            *auth_state,
            AuthState::Authenticated(_) | AuthState::DemoMode(_) | AuthState::Delegate(_)
        )
    }

    fn get_vault_dir(&self) -> std::path::PathBuf {
        self.args
            .storage_dir
            .as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("communitas-mcp-http"))
    }

    fn get_storage_dir(&self, four_words: &str) -> String {
        self.get_vault_dir()
            .join(four_words)
            .to_string_lossy()
            .to_string()
    }

    async fn get_token_manager(
        &self,
    ) -> Result<tokio::sync::RwLockReadGuard<'_, Option<TokenManager>>, JsonRpcError> {
        {
            let existing = self.token_manager.read().await;
            if existing.is_some() {
                return Ok(existing);
            }
        }

        let vault_dir = self.get_vault_dir();
        let manager = TokenManager::new(vault_dir).await.map_err(|e| {
            JsonRpcError::internal_error(&format!("Failed to initialize token manager: {e}"))
        })?;

        let mut write_guard = self.token_manager.write().await;
        *write_guard = Some(manager);
        drop(write_guard);

        Ok(self.token_manager.read().await)
    }

    /// Initialize UiServices from storage path and CommunitasApp.
    ///
    /// This ensures MCP tools use the same code path as Dioxus UI components,
    /// guaranteeing feature parity between automation and interactive use.
    async fn init_services(
        &self,
        storage_dir: &str,
        app: CommunitasApp,
    ) -> Result<(), JsonRpcError> {
        let app = Arc::new(app);

        let storage = UiStorage::from_path(storage_dir).map_err(|e| {
            error!("HTTP: Failed to create UiStorage: {}", e);
            JsonRpcError::internal_error(&format!("Failed to create UiStorage: {e}"))
        })?;

        let services = UiServices::new(storage, app.clone()).map_err(|e| {
            error!("HTTP: Failed to create UiServices: {}", e);
            JsonRpcError::internal_error(&format!("Failed to create UiServices: {e}"))
        })?;

        // Enable demo mode authentication on the services auth controller
        // so that UI service operations work without interactive login
        services.auth().enable_demo_mode();

        // Store the Arc<CommunitasApp> (shared with UiServices)
        {
            let mut app_lock = self.app.write().await;
            *app_lock = Some(app);
        }

        // Store the services
        {
            let mut services_lock = self.services.write().await;
            *services_lock = Some(services);
        }

        Ok(())
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
    axum_server::bind_rustls(
        addr,
        axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(rustls_config)),
    )
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
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
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
    body: Bytes,
) -> (StatusCode, Json<JsonRpcResponse>) {
    let value: Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(JsonRpcResponse::error(None, JsonRpcError::parse_error())),
            );
        }
    };

    let request: JsonRpcRequest = match serde_json::from_value(value) {
        Ok(request) => request,
        Err(err) => {
            return (
                StatusCode::OK,
                Json(JsonRpcResponse::error(
                    None,
                    JsonRpcError::invalid_request(&err.to_string()),
                )),
            );
        }
    };

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
        "resources/list" => handle_resources_list(state).await,
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
        None => {
            generate_id_words().map_err(|e| anyhow::anyhow!("Failed to generate identity: {e}"))?
        }
    };

    let display_name = state.args.display_name.clone();
    let device_name = "mcp-http-demo".to_string();
    let storage_dir = state.args.storage_dir.clone().unwrap_or_else(|| {
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
    .map_err(|e| anyhow::anyhow!("Failed to initialize app: {e}"))?;

    state
        .init_services(&storage_dir, app)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize services: {:?}", e))?;

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
// Helper Functions
// =============================================================================

/// Validates and normalizes a four-word identity string.
/// Accepts both dot-separated (word1.word2.word3.word4) and dash-separated (word1-word2-word3-word4) formats.
/// Returns the dash-separated format on success.
fn validate_four_words(four_words: &str) -> Result<String, JsonRpcError> {
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
    Ok(words.join("-"))
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

    // Use extended initialize result with MCP Apps UI extension support
    let result = InitializeResultWithExtensions::with_ui_support(
        "2024-11-05",
        ServerInfo {
            name: "communitas-mcp-http".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
        Some(ToolsCapability { list_changed: false }),
        Some(ResourcesCapability {
            subscribe: false,
            list_changed: false,
        }),
    );

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

    let services_lock = state.services.read().await;
    let services = services_lock
        .as_ref()
        .ok_or_else(|| JsonRpcError::internal_error("Services not initialized"))?;

    let result = tools::call_tool(app, services, &params.name, params.arguments).await;
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
                "content": [{"type": "text", "text": serde_json::to_string(&serde_json::json!({"initialized": initialized})).map_err(|e| JsonRpcError::internal_error(&format!("JSON serialization failed: {e}")))?}],
                "isError": false
            });
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        "authenticate" => handle_authenticate(state, args).await,
        "create_vault" => handle_create_vault(state, args).await,
        "authenticate_token" => handle_authenticate_token(state, args).await,
        "list_vaults" => handle_list_vaults(state).await,
        _ => Err(JsonRpcError::method_not_found(name)),
    }
}

async fn handle_authenticate_token(
    state: &HttpServerState,
    args: Value,
) -> Result<Value, JsonRpcError> {
    let token_str = args["token"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing token"))?;

    let token_manager_guard = state.get_token_manager().await?;
    let token_manager = token_manager_guard
        .as_ref()
        .ok_or_else(|| JsonRpcError::internal_error("Token manager not initialized"))?;

    let delegate_token = token_manager.verify_token(token_str).map_err(|e| {
        error!("HTTP: Token verification failed: {}", e);
        JsonRpcError::invalid_request(&format!("Invalid token: {e}"))
    })?;

    let storage_dir = state.get_storage_dir(&delegate_token.issuer);

    match CommunitasApp::new(
        delegate_token.issuer.clone(),
        delegate_token.delegate_name.clone(),
        "delegate-session".to_string(),
        storage_dir.clone(),
    )
    .await
    {
        Ok(app) => {
            state.init_services(&storage_dir, app).await?;

            let mut auth_lock = state.auth_state.write().await;
            *auth_lock = AuthState::Delegate(DelegateSession {
                issuer_four_words: delegate_token.issuer.clone(),
                delegate_name: delegate_token.delegate_name,
                scopes: delegate_token.scopes,
                started_at: SystemTime::now(),
                storage_dir,
            });

            info!(
                "HTTP: Delegate authenticated successfully for issuer: {}",
                delegate_token.issuer
            );

            let result = tools::success_result("Delegate token authentication successful");
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        Err(e) => {
            error!("HTTP: Failed to initialize app for delegate: {}", e);
            Err(JsonRpcError::internal_error(&format!(
                "Failed to initialize delegate session: {e}"
            )))
        }
    }
}

async fn handle_list_vaults(state: &HttpServerState) -> Result<Value, JsonRpcError> {
    use communitas_core::auth_service::AuthService;
    use communitas_core::encrypted_storage::{EncryptedStorageManager, StorageConfig};

    let storage_config = StorageConfig {
        vault_dir: state.get_vault_dir(),
        use_keyring: false,
        ..Default::default()
    };

    let storage_manager = EncryptedStorageManager::new(storage_config)
        .await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to initialize storage: {e}")))?;

    let auth_service = AuthService::new(storage_manager);

    let vaults = auth_service
        .list_vaults()
        .await
        .map_err(|e| JsonRpcError::internal_error(&format!("Failed to list vaults: {e}")))?;

    let vault_list: Vec<Value> = vaults
        .iter()
        .map(|v| {
            serde_json::json!({
                "four_words": v.four_words,
                "display_name": v.display_name,
                "created_at": v.created_at,
                "last_accessed": v.last_accessed,
                "size_bytes": v.size_bytes
            })
        })
        .collect();

    let result = serde_json::json!({
        "content": [{"type": "text", "text": serde_json::to_string(&serde_json::json!({"vaults": vault_list, "count": vault_list.len()})).map_err(|e| JsonRpcError::internal_error(&format!("JSON serialization failed: {e}")))?}],
        "isError": false
    });
    serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
}

async fn handle_authenticate(state: &HttpServerState, args: Value) -> Result<Value, JsonRpcError> {
    let four_words_raw = args["four_words"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?;
    let _password = args["password"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?;
    let device_name = args["device_name"]
        .as_str()
        .unwrap_or("mcp-http-client")
        .to_string();

    let four_words_dashed = validate_four_words(four_words_raw)?;

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
            state.init_services(&storage_dir, app).await?;

            let mut auth_lock = state.auth_state.write().await;
            *auth_lock = AuthState::Authenticated(AuthenticatedSession {
                four_words: four_words_dashed.clone(),
                display_name: four_words_dashed.clone(),
                device_name,
                started_at: SystemTime::now(),
                storage_dir,
            });

            info!(
                "HTTP: User authenticated successfully: {}",
                four_words_dashed
            );

            let result = tools::success_result("Authentication successful");
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        Err(e) => {
            error!("HTTP: Authentication failed: {}", e);
            Err(JsonRpcError::internal_error(&format!(
                "Authentication failed: {e}"
            )))
        }
    }
}

async fn handle_create_vault(state: &HttpServerState, args: Value) -> Result<Value, JsonRpcError> {
    let four_words_raw = args["four_words"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?;
    let _password = args["password"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?;
    let display_name = args["display_name"]
        .as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing display_name"))?
        .to_string();

    let four_words_dashed = validate_four_words(four_words_raw)?;

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
            state.init_services(&storage_dir, app).await?;

            let mut auth_lock = state.auth_state.write().await;
            *auth_lock = AuthState::Authenticated(AuthenticatedSession {
                four_words: four_words_dashed.clone(),
                display_name,
                device_name,
                started_at: SystemTime::now(),
                storage_dir,
            });

            info!(
                "HTTP: Vault created successfully for: {}",
                four_words_dashed
            );

            let result = tools::success_result("Vault created and authenticated successfully");
            serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
        }
        Err(e) => {
            error!("HTTP: Failed to create vault: {}", e);
            Err(JsonRpcError::internal_error(&format!(
                "Failed to create vault: {e}"
            )))
        }
    }
}

async fn handle_resources_list(state: &HttpServerState) -> Result<Value, JsonRpcError> {
    let mut resources = vec![
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

    // Add UI resources from the MCP Apps registry
    for ui_resource in state.ui_resources.list() {
        resources.push(Resource {
            uri: ui_resource.uri,
            name: ui_resource.name,
            description: ui_resource.description,
            mime_type: ui_resource.mime_type,
        });
    }

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
        uri if uri.starts_with("ui://") => {
            // Handle MCP Apps UI resources
            drop(app_lock); // Release the app lock since we don't need it for UI resources

            let (content, mime_type) = state
                .ui_resources
                .read(uri)
                .ok_or_else(|| JsonRpcError::invalid_params(&format!("Unknown UI resource: {uri}")))?;

            let result = ResourceReadResult {
                contents: vec![ResourceContent {
                    uri: params.uri,
                    mime_type: Some(mime_type),
                    text: Some(content),
                    blob: None,
                }],
            };

            return serde_json::to_value(result)
                .map_err(|e| JsonRpcError::internal_error(&e.to_string()));
        }
        uri => {
            return Err(JsonRpcError::invalid_params(&format!(
                "Unknown resource: {uri}"
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_args() -> Args {
        Args {
            demo: false,
            storage_dir: None,
            four_words: None,
            display_name: "Test".to_string(),
            http: true,
            tls: false,
            listen: None,
            no_client_auth: true,
        }
    }

    async fn response_json(router: Router, body: &'static str) -> serde_json::Value {
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_parse_error_response() {
        let state = Arc::new(HttpServerState::new(test_args()));
        let router = create_router(state);
        let json = response_json(router, "{invalid json").await;
        assert_eq!(json["error"]["code"], -32700);
    }

    #[tokio::test]
    async fn test_invalid_request_response() {
        let state = Arc::new(HttpServerState::new(test_args()));
        let router = create_router(state);
        let json = response_json(router, r#"{"jsonrpc":"2.0","id":1}"#).await;
        assert_eq!(json["error"]["code"], -32600);
    }
}
