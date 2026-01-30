// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Server implementation
//!
//! Handles JSON-RPC 2.0 communication over stdio, routing requests to the
//! CommunitasApp for processing. Supports both authenticated and demo modes.

use crate::Args;
use crate::protocol::{
    AiContext, InitializeParams, InitializeResultWithExtensions, JsonRpcError, JsonRpcRequest,
    JsonRpcResponse, Resource, ResourceContent, ResourceListResult, ResourceReadParams,
    ResourceReadResult, ResourcesCapability, ServerInfo, ToolCallParams, ToolCallResultWithContext,
    ToolListResult, ToolsCapability,
};
use crate::tools;
use crate::ui_resources::UiResourceRegistry;
use crate::ui_session::UiSessionStore;
use anyhow::Result;
use communitas_core::app::CommunitasApp;
use communitas_core::auth_service::AuthService;
use communitas_core::encrypted_storage::{EncryptedStorageManager, StorageConfig};
use communitas_core::identity::generate_id_words;
use communitas_mcp::auth::{
    AuthState, AuthenticatedSession, DelegateSession, DemoSession, Scope, requires_auth,
};
use communitas_mcp::token::TokenManager;
use communitas_ui_service::UiServices;
use communitas_ui_service::storage::UiStorage;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct McpServer {
    app: Arc<RwLock<Option<Arc<CommunitasApp>>>>,
    services: Arc<RwLock<Option<UiServices>>>,
    auth_state: AuthState,
    protocol_initialized: bool,
    demo_mode: bool,
    args: Args,
    token_manager: Option<TokenManager>,
    /// UI resource registry for MCP Apps extension
    ui_resources: UiResourceRegistry,
    /// Current UI context for AI context hints (Phase 9.3)
    ui_context: AiContext,
    /// UI session tokens for widget communication
    ui_sessions: UiSessionStore,
}

impl McpServer {
    pub fn new(args: Args) -> Self {
        let demo_mode = args.demo;
        Self {
            app: Arc::new(RwLock::new(None)),
            services: Arc::new(RwLock::new(None)),
            auth_state: AuthState::Unauthenticated,
            protocol_initialized: false,
            demo_mode,
            args,
            token_manager: None,
            ui_resources: UiResourceRegistry::with_standard_widgets(),
            ui_context: AiContext::default(),
            ui_sessions: UiSessionStore::with_default_ttl(),
        }
    }

    async fn get_token_manager(&mut self) -> Result<&TokenManager, JsonRpcError> {
        if self.token_manager.is_none() {
            let vault_dir = self.get_vault_dir();
            let manager = TokenManager::new(vault_dir).await.map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to initialize token manager: {e}"))
            })?;
            self.token_manager = Some(manager);
        }
        self.token_manager
            .as_ref()
            .ok_or_else(|| JsonRpcError::internal_error("token_manager not initialized"))
    }

    fn is_authenticated(&self) -> bool {
        matches!(
            self.auth_state,
            AuthState::Authenticated(_) | AuthState::DemoMode(_) | AuthState::Delegate(_)
        )
    }

    fn get_vault_dir(&self) -> PathBuf {
        self.args
            .storage_dir
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("communitas-mcp"))
    }

    fn get_storage_dir(&self, four_words: &str) -> String {
        self.get_vault_dir()
            .join(four_words)
            .to_string_lossy()
            .to_string()
    }

    /// Initialize UiServices from storage path and CommunitasApp.
    ///
    /// This ensures MCP tools use the same code path as Dioxus UI components,
    /// guaranteeing feature parity between automation and interactive use.
    async fn init_services(
        &mut self,
        storage_dir: &str,
        app: CommunitasApp,
    ) -> Result<(), JsonRpcError> {
        let app = Arc::new(app);

        let storage = UiStorage::from_path(storage_dir).map_err(|e| {
            error!("Failed to create UiStorage: {}", e);
            JsonRpcError::internal_error(&format!("Failed to create UiStorage: {e}"))
        })?;

        let services = UiServices::new(storage, app.clone()).map_err(|e| {
            error!("Failed to create UiServices: {}", e);
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

    pub async fn handle_request(&mut self, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
        debug!(
            "Handling request: {} (id: {:?})",
            request.method, request.id
        );

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
            // UI context methods (Phase 9.3 - AI Context Integration)
            "ui/context" => self.handle_ui_context(request.params).await,
            "ui/message" => self.handle_ui_message(request.params).await,
            "ui/initialize" => self.handle_ui_initialize(request.params).await,
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

        self.protocol_initialized = true;

        // In demo mode, auto-initialize with temporary identity
        if self.demo_mode {
            self.initialize_demo_mode().await?;
        }

        // Use extended initialize result with MCP Apps UI extension support
        let result = InitializeResultWithExtensions::with_ui_support(
            "2024-11-05",
            ServerInfo {
                name: "communitas-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            Some(ToolsCapability {
                list_changed: false,
            }),
            Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
        );

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Initialize demo mode with temporary identity
    async fn initialize_demo_mode(&mut self) -> Result<(), JsonRpcError> {
        // Use provided four-words or generate new ones
        let four_words = match &self.args.four_words {
            Some(fw) => fw.clone(),
            None => generate_id_words().map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to generate four-word identity: {e}"))
            })?,
        };

        let display_name = self.args.display_name.clone();
        let device_name = "mcp-demo".to_string();
        let storage_dir = self.args.storage_dir.clone().unwrap_or_else(|| {
            std::env::temp_dir()
                .join("communitas-mcp-demo")
                .to_string_lossy()
                .to_string()
        });

        info!(
            "Demo mode: initializing with identity {} in {}",
            four_words, storage_dir
        );

        // Initialize the CommunitasApp and UiServices
        let app = CommunitasApp::new(
            four_words.clone(),
            display_name.clone(),
            device_name,
            storage_dir.clone(),
        )
        .await
        .map_err(|e| {
            error!("Failed to initialize demo mode: {}", e);
            JsonRpcError::internal_error(&format!("Failed to initialize demo mode: {e}"))
        })?;

        self.init_services(&storage_dir, app).await?;

        self.auth_state = AuthState::DemoMode(DemoSession {
            four_words,
            display_name,
            started_at: SystemTime::now(),
            storage_dir,
        });

        info!("Demo mode initialized successfully");
        Ok(())
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        // Return all tools - auth checking happens on call
        let result = ToolListResult {
            tools: tools::list_tools(self.is_authenticated()),
        };

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle tools/call request
    async fn handle_tools_call(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        if !self.protocol_initialized {
            return Err(JsonRpcError::invalid_request("Server not initialized"));
        }

        let params: ToolCallParams = params
            .map(|p| {
                serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
            })
            .transpose()?
            .ok_or_else(|| JsonRpcError::invalid_params("Missing tool call params"))?;

        // Check if this is a pre-auth tool
        let tool_requires_auth = requires_auth(&params.name);

        // Handle pre-auth tools
        if !tool_requires_auth {
            return self
                .handle_pre_auth_tool(&params.name, params.arguments)
                .await;
        }

        // For tools requiring auth, check if we're authenticated
        if !self.is_authenticated() {
            return Err(JsonRpcError::invalid_request(
                "Authentication required. Use 'authenticate', 'create_vault', or restart with --demo flag.",
            ));
        }

        if params.name == "logout" {
            self.auth_state = AuthState::Unauthenticated;
            let result = tools::success_result("Logged out successfully");
            return serde_json::to_value(result)
                .map_err(|e| JsonRpcError::internal_error(&e.to_string()));
        }

        if params.name == "create_delegate_token" {
            return self
                .handle_create_delegate_token(params.arguments.unwrap_or(serde_json::json!({})))
                .await;
        }

        if params.name == "export_vault" {
            return self
                .handle_export_vault(params.arguments.unwrap_or(serde_json::json!({})))
                .await;
        }

        let app_lock = self.app.read().await;
        let app = app_lock
            .as_ref()
            .ok_or_else(|| JsonRpcError::internal_error("App not initialized"))?;

        let services_lock = self.services.read().await;
        let services = services_lock
            .as_ref()
            .ok_or_else(|| JsonRpcError::internal_error("Services not initialized"))?;

        let result = tools::call_tool(app, services, &params.name, params.arguments).await;

        // Add AI context to tool response (Phase 9.3)
        // This helps the AI host understand current widget state
        let resource_uri = get_tool_resource_uri(&params.name);
        let result_with_context = ToolCallResultWithContext::from_basic_with_context(
            result,
            resource_uri,
            self.ui_context.clone(),
        );

        serde_json::to_value(result_with_context)
            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle pre-authentication tools
    async fn handle_pre_auth_tool(
        &mut self,
        name: &str,
        args: Option<Value>,
    ) -> Result<Value, JsonRpcError> {
        let args = args.unwrap_or(serde_json::json!({}));

        match name {
            "authenticate" => self.handle_authenticate(args).await,
            "create_vault" => self.handle_create_vault(args).await,
            "authenticate_token" => self.handle_authenticate_token(args).await,
            "health_check" => {
                let result = tools::success_result("MCP service is healthy");
                serde_json::to_value(result)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))
            }
            "core_status" => {
                let initialized = self.is_authenticated();
                let result = serde_json::json!({
                    "content": [{"type": "text", "text": serde_json::to_string(&serde_json::json!({"initialized": initialized})).map_err(|e| JsonRpcError::internal_error(&format!("JSON serialization failed: {e}")))?}],
                    "isError": false
                });
                serde_json::to_value(result)
                    .map_err(|e| JsonRpcError::internal_error(&e.to_string()))
            }
            "list_vaults" => self.handle_list_vaults().await,
            "delete_vault" => self.handle_delete_vault(args).await,
            "import_vault" => self.handle_import_vault(args).await,
            _ => Err(JsonRpcError::method_not_found(name)),
        }
    }

    /// Handle authenticate tool
    async fn handle_authenticate(&mut self, args: Value) -> Result<Value, JsonRpcError> {
        let four_words = args["four_words"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?
            .to_string();
        let password = args["password"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?
            .to_string();
        let device_name = args["device_name"]
            .as_str()
            .unwrap_or("mcp-client")
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

        let four_words_dashed = words.join("-");
        let storage_dir = self.get_storage_dir(&four_words_dashed);

        info!("Authenticating user: {}", four_words_dashed);

        let storage_config = StorageConfig {
            vault_dir: self.get_vault_dir(),
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(storage_config)
            .await
            .map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to initialize storage: {e}"))
            })?;

        let mut auth_service = AuthService::new(storage_manager);

        let session_info = auth_service
            .login(&four_words_dashed, &password, Some(&device_name))
            .await
            .map_err(|e| {
                error!("Authentication failed: {}", e);
                JsonRpcError::invalid_request(&format!(
                    "Authentication failed: {e}. Make sure vault exists and password is correct."
                ))
            })?;
        let app = CommunitasApp::new(
            four_words_dashed.clone(),
            session_info.display_name.clone(),
            device_name.clone(),
            storage_dir.clone(),
        )
        .await
        .map_err(|e| {
            error!("Failed to initialize app after authentication: {}", e);
            JsonRpcError::internal_error(&format!("Failed to initialize app: {e}"))
        })?;

        self.init_services(&storage_dir, app).await?;

        self.auth_state = AuthState::Authenticated(AuthenticatedSession {
            four_words: four_words_dashed.clone(),
            display_name: session_info.display_name,
            device_name,
            started_at: SystemTime::now(),
            storage_dir,
        });

        info!("User authenticated successfully: {}", four_words_dashed);

        let result = tools::success_result("Authentication successful");
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_create_vault(&mut self, args: Value) -> Result<Value, JsonRpcError> {
        let four_words = args["four_words"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?
            .to_string();
        let password = args["password"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?
            .to_string();
        let display_name = args["display_name"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing display_name"))?
            .to_string();

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

        let four_words_dashed = words.join("-");
        let device_name = "mcp-client".to_string();
        let storage_dir = self.get_storage_dir(&four_words_dashed);

        info!("Creating new vault for: {}", four_words_dashed);

        let storage_config = StorageConfig {
            vault_dir: self.get_vault_dir(),
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(storage_config)
            .await
            .map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to initialize storage: {e}"))
            })?;

        let mut auth_service = AuthService::new(storage_manager);

        auth_service
            .create_vault(&four_words_dashed, &password, &display_name)
            .await
            .map_err(|e| {
                error!("Failed to create vault: {}", e);
                JsonRpcError::internal_error(&format!("Failed to create vault: {e}"))
            })?;

        let session_info = auth_service
            .login(&four_words_dashed, &password, Some(&device_name))
            .await
            .map_err(|e| {
                error!("Failed to login after vault creation: {}", e);
                JsonRpcError::internal_error(&format!("Vault created but login failed: {e}"))
            })?;

        let app = CommunitasApp::new(
            four_words_dashed.clone(),
            session_info.display_name.clone(),
            device_name.clone(),
            storage_dir.clone(),
        )
        .await
        .map_err(|e| {
            error!("Failed to initialize app after vault creation: {}", e);
            JsonRpcError::internal_error(&format!(
                "Vault created but app initialization failed: {e}"
            ))
        })?;

        self.init_services(&storage_dir, app).await?;

        self.auth_state = AuthState::Authenticated(AuthenticatedSession {
            four_words: four_words_dashed.clone(),
            display_name: session_info.display_name,
            device_name,
            started_at: SystemTime::now(),
            storage_dir,
        });

        info!("Vault created successfully for: {}", four_words_dashed);

        let result = tools::success_result("Vault created and authenticated successfully");
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_authenticate_token(&mut self, args: Value) -> Result<Value, JsonRpcError> {
        let token_str = args["token"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing token"))?;

        let token_manager = self.get_token_manager().await?;
        let delegate_token = token_manager.verify_token(token_str).map_err(|e| {
            error!("Token verification failed: {}", e);
            JsonRpcError::invalid_request(&format!("Invalid token: {e}"))
        })?;

        let storage_dir = self.get_storage_dir(&delegate_token.issuer);

        let app = CommunitasApp::new(
            delegate_token.issuer.clone(),
            delegate_token.delegate_name.clone(),
            "delegate-session".to_string(),
            storage_dir.clone(),
        )
        .await
        .map_err(|e| {
            error!("Failed to initialize app for delegate: {}", e);
            JsonRpcError::internal_error(&format!("Failed to initialize delegate session: {e}"))
        })?;

        self.init_services(&storage_dir, app).await?;

        self.auth_state = AuthState::Delegate(DelegateSession {
            issuer_four_words: delegate_token.issuer.clone(),
            delegate_name: delegate_token.delegate_name,
            scopes: delegate_token.scopes,
            started_at: SystemTime::now(),
            storage_dir,
        });

        info!(
            "Delegate authenticated successfully for issuer: {}",
            delegate_token.issuer
        );

        let result = tools::success_result("Delegate token authentication successful");
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_create_delegate_token(&mut self, args: Value) -> Result<Value, JsonRpcError> {
        let delegate_name = args["delegate_name"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing delegate_name"))?
            .to_string();

        let expires_in_hours = args["expires_in_hours"].as_u64().unwrap_or(24);

        let scopes: Vec<Scope> = match args.get("scopes") {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .filter_map(Scope::parse)
                .collect(),
            _ => vec![Scope::Full],
        };

        let issuer = match &self.auth_state {
            AuthState::Authenticated(session) => session.four_words.clone(),
            AuthState::DemoMode(session) => session.four_words.clone(),
            AuthState::Delegate(_) => {
                return Err(JsonRpcError::invalid_request(
                    "Delegate sessions cannot create new tokens",
                ));
            }
            AuthState::Unauthenticated => {
                return Err(JsonRpcError::invalid_request("Not authenticated"));
            }
        };

        let token_manager = self.get_token_manager().await?;
        let token = token_manager
            .create_token(&issuer, &delegate_name, scopes.clone(), expires_in_hours)
            .map_err(|e| JsonRpcError::internal_error(&format!("Failed to create token: {e}")))?;

        info!(
            "Created delegate token for {} with {} scopes, expires in {} hours",
            delegate_name,
            scopes.len(),
            expires_in_hours
        );

        let result = serde_json::json!({
            "content": [{
                "type": "text",
                "text": serde_json::json!({
                    "token": token,
                    "delegate_name": delegate_name,
                    "scopes": scopes,
                    "expires_in_hours": expires_in_hours
                }).to_string()
            }],
            "isError": false
        });

        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_list_vaults(&self) -> Result<Value, JsonRpcError> {
        let storage_config = StorageConfig {
            vault_dir: self.get_vault_dir(),
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(storage_config)
            .await
            .map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to initialize storage: {e}"))
            })?;

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

    async fn handle_delete_vault(&mut self, args: Value) -> Result<Value, JsonRpcError> {
        let four_words = args["four_words"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing four_words"))?
            .to_string();
        let password = args["password"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?
            .to_string();

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

        let four_words_dashed = words.join("-");

        let storage_config = StorageConfig {
            vault_dir: self.get_vault_dir(),
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(storage_config)
            .await
            .map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to initialize storage: {e}"))
            })?;

        let mut auth_service = AuthService::new(storage_manager);

        auth_service
            .delete_vault(&four_words_dashed, &password)
            .await
            .map_err(|e| JsonRpcError::invalid_request(&format!("Failed to delete vault: {e}")))?;

        info!("Vault deleted: {}", four_words_dashed);

        let result = tools::success_result("Vault deleted successfully");
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_import_vault(&mut self, args: Value) -> Result<Value, JsonRpcError> {
        let backup_data_base64 = args["backup_data"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing backup_data"))?;
        let password = args["password"]
            .as_str()
            .ok_or_else(|| JsonRpcError::invalid_params("Missing password"))?
            .to_string();

        use base64::Engine;
        let backup_data = base64::engine::general_purpose::STANDARD
            .decode(backup_data_base64)
            .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid base64 data: {e}")))?;

        let storage_config = StorageConfig {
            vault_dir: self.get_vault_dir(),
            use_keyring: false,
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(storage_config)
            .await
            .map_err(|e| {
                JsonRpcError::internal_error(&format!("Failed to initialize storage: {e}"))
            })?;

        let four_words = storage_manager
            .import_vault(&backup_data, &password)
            .await
            .map_err(|e| JsonRpcError::invalid_request(&format!("Failed to import vault: {e}")))?;

        info!("Vault imported: {}", four_words);

        let result = serde_json::json!({
            "content": [{"type": "text", "text": serde_json::to_string(&serde_json::json!({"success": true, "four_words": four_words, "message": "Vault imported successfully"})).map_err(|e| JsonRpcError::internal_error(&format!("JSON serialization failed: {e}")))?}],
            "isError": false
        });
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    async fn handle_export_vault(&self, _args: Value) -> Result<Value, JsonRpcError> {
        if let AuthState::Unauthenticated = &self.auth_state {
            return Err(JsonRpcError::invalid_request("Not authenticated"));
        }

        Err(JsonRpcError::internal_error(
            "Vault export not yet implemented - requires session-based storage access. Use the desktop app for vault backup.",
        ))
    }

    /// Handle resources/list request
    async fn handle_resources_list(&self) -> Result<Value, JsonRpcError> {
        // Expose application state as resources
        let mut resources = vec![
            Resource {
                uri: "communitas://identity".to_string(),
                name: "Current Identity".to_string(),
                description: Some("The current user's identity and four-word address".to_string()),
                mime_type: Some("application/json".to_string()),
                _meta: None,
            },
            Resource {
                uri: "communitas://entities".to_string(),
                name: "Entities".to_string(),
                description: Some(
                    "List of all entities (orgs, groups, channels, projects)".to_string(),
                ),
                mime_type: Some("application/json".to_string()),
                _meta: None,
            },
            Resource {
                uri: "communitas://chats".to_string(),
                name: "Chats".to_string(),
                description: Some("List of all chat conversations".to_string()),
                mime_type: Some("application/json".to_string()),
                _meta: None,
            },
            Resource {
                uri: "communitas://invites".to_string(),
                name: "Invites".to_string(),
                description: Some("Pending invitations".to_string()),
                mime_type: Some("application/json".to_string()),
                _meta: None,
            },
        ];

        // Add new resources
        resources.push(Resource {
            uri: "communitas://contacts".to_string(),
            name: "Contacts".to_string(),
            description: Some("All contacts".to_string()),
            mime_type: Some("application/json".to_string()),
            _meta: None,
        });

        resources.push(Resource {
            uri: "communitas://network".to_string(),
            name: "Network Status".to_string(),
            description: Some("P2P network status and connected peers".to_string()),
            mime_type: Some("application/json".to_string()),
            _meta: None,
        });

        // Add UI resources from the MCP Apps registry
        for ui_resource in self.ui_resources.list() {
            resources.push(Resource {
                uri: ui_resource.uri,
                name: ui_resource.name,
                description: ui_resource.description,
                mime_type: ui_resource.mime_type,
                _meta: ui_resource._meta,
            });
        }

        let result = ResourceListResult { resources };
        serde_json::to_value(result).map_err(|e| JsonRpcError::internal_error(&e.to_string()))
    }

    /// Handle resources/read request
    async fn handle_resources_read(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params: ResourceReadParams = params
            .map(|p| {
                serde_json::from_value(p).map_err(|e| JsonRpcError::invalid_params(&e.to_string()))
            })
            .transpose()?
            .ok_or_else(|| JsonRpcError::invalid_params("Missing resource read params"))?;

        if params.uri.starts_with("ui://") {
            let (content, mime_type) = self.ui_resources.read(&params.uri).ok_or_else(|| {
                JsonRpcError::invalid_params(&format!("Unknown UI resource: {}", params.uri))
            })?;

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

        if !self.is_authenticated() {
            return Err(JsonRpcError::invalid_request(
                "Authentication required to read resources",
            ));
        }

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
            "communitas://contacts" => {
                let response = app
                    .query(communitas_core::command::Query::ListContacts)
                    .await
                    .map_err(|e| JsonRpcError::internal_error(&e.message))?;
                match response {
                    communitas_core::command::QueryResponse::ContactList(contacts) => {
                        let list: Vec<serde_json::Value> = contacts
                            .into_iter()
                            .map(|c| {
                                serde_json::json!({
                                    "id": c.id,
                                    "display_name": c.display_name,
                                    "four_words": c.four_words,
                                    "is_favourite": c.is_favourite,
                                    "is_online": c.is_online,
                                    "created_at": c.created_at,
                                    "last_seen": c.last_seen
                                })
                            })
                            .collect();
                        serde_json::to_string_pretty(&serde_json::json!({ "contacts": list }))
                            .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
                    }
                    _ => {
                        return Err(JsonRpcError::internal_error(
                            "Unexpected response type for contacts",
                        ));
                    }
                }
            }
            "communitas://network" => {
                let is_active = match app
                    .query(communitas_core::command::Query::IsNetworkingActive)
                    .await
                {
                    Ok(communitas_core::command::QueryResponse::Bool(active)) => active,
                    _ => false,
                };
                let connection_identity = match app
                    .query(communitas_core::command::Query::GetConnectionIdentity)
                    .await
                {
                    Ok(communitas_core::command::QueryResponse::OptionalString(value)) => value,
                    _ => None,
                };
                let connection_words = match app
                    .query(communitas_core::command::Query::GetConnectionWords)
                    .await
                {
                    Ok(communitas_core::command::QueryResponse::OptionalString(value)) => value,
                    _ => None,
                };
                let external_address = match app
                    .query(communitas_core::command::Query::GetExternalAddress)
                    .await
                {
                    Ok(communitas_core::command::QueryResponse::OptionalString(value)) => value,
                    _ => None,
                };
                let peers = match app
                    .query(communitas_core::command::Query::ListOnlinePeers)
                    .await
                {
                    Ok(communitas_core::command::QueryResponse::PeerList(list)) => list,
                    _ => Vec::new(),
                };
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": if is_active { "active" } else { "inactive" },
                    "connection_identity": connection_identity,
                    "connection_words": connection_words,
                    "external_address": external_address,
                    "peers": peers,
                    "peer_count": peers.len()
                }))
                .map_err(|e| JsonRpcError::internal_error(&e.to_string()))?
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

    fn ui_session_token_from_params(params: &Value) -> Option<&str> {
        params
            .get("sessionToken")
            .and_then(|value| value.as_str())
            .or_else(|| params.get("session_token").and_then(|value| value.as_str()))
    }

    fn validate_ui_session(&mut self, params: &Value) -> Result<(), JsonRpcError> {
        let token = Self::ui_session_token_from_params(params)
            .ok_or_else(|| JsonRpcError::invalid_request("Missing ui session token"))?;

        if !self.ui_sessions.validate(token) {
            return Err(JsonRpcError::invalid_request(
                "UI session expired or invalid",
            ));
        }

        Ok(())
    }

    // ==========================================================================
    // UI Context Methods (Phase 9.3 - AI Context Integration)
    // ==========================================================================

    /// Handle ui/context - receive context updates from widgets
    async fn handle_ui_context(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        // Parse context update from widget
        let params =
            params.ok_or_else(|| JsonRpcError::invalid_params("Missing ui/context params"))?;

        self.validate_ui_session(&params)?;

        // Extract context and changed field from params
        let context_value = params.get("context");
        let changed_field = params
            .get("changed")
            .and_then(|v| v.as_str())
            .map(String::from);

        if let Some(ctx) = context_value {
            // Parse the context
            let context: AiContext = serde_json::from_value(ctx.clone())
                .map_err(|e| JsonRpcError::invalid_params(&format!("Invalid context: {e}")))?;

            // Update our stored context
            self.ui_context = context;

            debug!(
                "UI context updated: changed={:?}, current_view={:?}",
                changed_field,
                self.ui_context.current_view.as_ref().map(|v| &v.widget)
            );
        }

        // Return success acknowledgment
        Ok(serde_json::json!({
            "success": true,
            "changed": changed_field
        }))
    }

    /// Handle ui/message - receive messages with optional context from widgets
    async fn handle_ui_message(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params =
            params.ok_or_else(|| JsonRpcError::invalid_params("Missing ui/message params"))?;

        self.validate_ui_session(&params)?;

        let content = params
            .get("content")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| JsonRpcError::invalid_params("Missing content in ui/message"))?;

        // Optional context included with message
        let context = params.get("context").cloned();

        debug!(
            "UI message received: content_len={}, has_context={}",
            content.len(),
            context.is_some()
        );

        // The message content can be used by the AI host to understand widget state
        // For now, we acknowledge receipt. Future: could emit as notification to host.
        Ok(serde_json::json!({
            "success": true,
            "received": true,
            "content_length": content.len(),
            "has_context": context.is_some()
        }))
    }

    /// Handle ui/initialize - widget handshake
    async fn handle_ui_initialize(&mut self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params =
            params.ok_or_else(|| JsonRpcError::invalid_params("Missing ui/initialize params"))?;

        // Parse widget capabilities
        let capabilities = params.get("capabilities").cloned().unwrap_or_default();

        debug!(
            "UI widget initialized with capabilities: {:?}",
            capabilities
        );

        let session = self.ui_sessions.issue();
        let expires_in = session.expires_in(SystemTime::now());

        // Return acknowledgment with server capabilities
        Ok(serde_json::json!({
            "success": true,
            "sessionToken": session.token,
            "expiresInSec": expires_in,
            "server_capabilities": {
                "context_tracking": true,
                "messaging": true,
                "tool_calls": true,
                "resource_reads": true
            }
        }))
    }

    /// Get the current UI context for including in tool responses
    #[allow(dead_code)] // Will be used in Task 2 completion
    pub fn get_ui_context(&self) -> &AiContext {
        &self.ui_context
    }
}

/// Run the MCP server
pub async fn run(args: Args) -> Result<()> {
    let mut server = McpServer::new(args);

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

/// Map tool names to their corresponding UI resource URIs
///
/// Tools related to a specific widget return that widget's resource URI,
/// allowing the AI host to understand which UI is relevant to the tool result.
fn get_tool_resource_uri(tool_name: &str) -> Option<&'static str> {
    match tool_name {
        // Contacts widget tools
        "list_contacts" | "get_contact" | "create_contact" | "update_contact"
        | "delete_contact" | "set_favourite" => Some("ui://communitas/contacts"),

        // Messages widget tools
        "list_threads" | "get_thread" | "create_thread" | "list_messages" | "get_message"
        | "send_message" | "delete_message" | "mark_read" => Some("ui://communitas/messages"),

        // Kanban widget tools
        "list_kanban_boards"
        | "get_kanban_board"
        | "create_kanban_board"
        | "delete_kanban_board"
        | "create_kanban_column"
        | "update_kanban_column"
        | "delete_kanban_column"
        | "create_kanban_card"
        | "update_kanban_card"
        | "delete_kanban_card"
        | "move_kanban_card"
        | "get_kanban_card" => Some("ui://communitas/kanban"),

        // Drive widget tools
        "list_files" | "list_disks" | "get_file_preview" | "upload_file" | "download_file"
        | "delete_file" | "create_folder" | "move_file" | "copy_file" | "get_quota" => {
            Some("ui://communitas/drive")
        }

        // Canvas widget tools
        "canvas_get_snapshot"
        | "canvas_get_history"
        | "canvas_undo"
        | "canvas_redo"
        | "canvas_add_object"
        | "canvas_remove_object"
        | "canvas_update_object"
        | "canvas_list_layers"
        | "canvas_toggle_layer" => Some("ui://communitas/canvas"),

        // Settings widget tools
        "get_settings" | "update_settings" | "get_profile" | "update_profile" => {
            Some("ui://communitas/settings")
        }

        // Search widget tools
        "search" | "search_contacts" | "search_messages" | "search_files" => {
            Some("ui://communitas/search")
        }

        // Notifications widget tools
        "list_notifications" | "mark_notification_read" | "clear_notifications" => {
            Some("ui://communitas/notifications")
        }

        // Tools without a specific UI widget
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{ClientCapabilities, ClientInfo};

    fn test_args() -> Args {
        Args {
            demo: false,
            storage_dir: None,
            four_words: None,
            display_name: "Test".to_string(),
            http: false,
            tls: false,
            listen: None,
            no_client_auth: true,
        }
    }

    fn init_request(id: i64) -> JsonRpcRequest {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: ClientInfo {
                name: "test".to_string(),
                version: "0.1".to_string(),
            },
        };
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(id)),
            method: "initialize".to_string(),
            params: Some(serde_json::to_value(params).unwrap()),
        }
    }

    fn tool_call_request(id: i64, name: &str) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(id)),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": name,
                "arguments": {}
            })),
        }
    }

    #[tokio::test]
    async fn test_tools_call_requires_initialize() {
        let mut server = McpServer::new(test_args());
        let response = server
            .handle_request(tool_call_request(1, "get_profile"))
            .await
            .expect("Expected response");
        let error = response.error.expect("Expected error response");
        assert_eq!(error.code, -32600);
    }

    #[tokio::test]
    async fn test_tools_call_requires_auth_after_init() {
        let mut server = McpServer::new(test_args());
        let _ = server.handle_request(init_request(1)).await;

        let response = server
            .handle_request(tool_call_request(2, "get_profile"))
            .await
            .expect("Expected response");
        let error = response.error.expect("Expected error response");
        assert_eq!(error.code, -32600);
        assert!(
            error.message.contains("Authentication required"),
            "Unexpected error message: {}",
            error.message
        );
    }

    #[test]
    fn test_get_tool_resource_uri_contacts() {
        assert_eq!(
            get_tool_resource_uri("list_contacts"),
            Some("ui://communitas/contacts")
        );
        assert_eq!(
            get_tool_resource_uri("create_contact"),
            Some("ui://communitas/contacts")
        );
    }

    #[test]
    fn test_get_tool_resource_uri_messages() {
        assert_eq!(
            get_tool_resource_uri("list_threads"),
            Some("ui://communitas/messages")
        );
        assert_eq!(
            get_tool_resource_uri("send_message"),
            Some("ui://communitas/messages")
        );
    }

    #[test]
    fn test_get_tool_resource_uri_kanban() {
        assert_eq!(
            get_tool_resource_uri("list_kanban_boards"),
            Some("ui://communitas/kanban")
        );
        assert_eq!(
            get_tool_resource_uri("create_kanban_card"),
            Some("ui://communitas/kanban")
        );
    }

    #[test]
    fn test_get_tool_resource_uri_drive() {
        assert_eq!(
            get_tool_resource_uri("list_files"),
            Some("ui://communitas/drive")
        );
        assert_eq!(
            get_tool_resource_uri("upload_file"),
            Some("ui://communitas/drive")
        );
    }

    #[test]
    fn test_get_tool_resource_uri_canvas() {
        assert_eq!(
            get_tool_resource_uri("canvas_get_snapshot"),
            Some("ui://communitas/canvas")
        );
        assert_eq!(
            get_tool_resource_uri("canvas_add_object"),
            Some("ui://communitas/canvas")
        );
    }

    #[test]
    fn test_get_tool_resource_uri_new_widgets() {
        // Settings widget (Phase 9.2)
        assert_eq!(
            get_tool_resource_uri("get_settings"),
            Some("ui://communitas/settings")
        );
        // Search widget (Phase 9.2)
        assert_eq!(
            get_tool_resource_uri("search"),
            Some("ui://communitas/search")
        );
        // Notifications widget (Phase 9.2)
        assert_eq!(
            get_tool_resource_uri("list_notifications"),
            Some("ui://communitas/notifications")
        );
    }

    #[test]
    fn test_get_tool_resource_uri_unknown() {
        assert_eq!(get_tool_resource_uri("authenticate"), None);
        assert_eq!(get_tool_resource_uri("health_check"), None);
        assert_eq!(get_tool_resource_uri("unknown_tool"), None);
    }
}
