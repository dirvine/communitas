// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Protocol types - JSON-RPC 2.0 implementation
//!
//! Implements the Model Context Protocol (MCP) specification for communication
//! between AI agents and the Communitas application.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }

    pub fn invalid_request(msg: &str) -> Self {
        Self {
            code: -32600,
            message: format!("Invalid Request: {msg}"),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> Self {
        Self {
            code: -32601,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    pub fn invalid_params(msg: &str) -> Self {
        Self {
            code: -32602,
            message: format!("Invalid params: {msg}"),
            data: None,
        }
    }

    pub fn internal_error(msg: &str) -> Self {
        Self {
            code: -32603,
            message: format!("Internal error: {msg}"),
            data: None,
        }
    }
}

// =============================================================================
// Security Validation Functions (MCP Apps Extension - SEP-1865)
// =============================================================================

/// Error type for MCP protocol validation errors
#[derive(Debug, Clone, thiserror::Error)]
#[allow(dead_code)] // Security infrastructure for future hardening phases
pub enum ProtocolError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid origin: {0}")]
    InvalidOrigin(String),

    #[error("Path traversal detected in: {0}")]
    PathTraversal(String),
}

/// Validates a string identifier (e.g., app_id, widget_id, resource_uri)
///
/// # Validation Rules
/// - Only alphanumeric characters, hyphens, underscores, and forward slashes
/// - Maximum 256 characters
/// - Cannot start or end with hyphen/underscore/slash
/// - Prevents path traversal (`..`) and shell injection
///
/// # Examples
/// ```
/// use communitas_mcp::protocol::validate_identifier;
/// assert!(validate_identifier("ui://communitas/contacts", "resource_uri").is_ok());
/// assert!(validate_identifier("../../../etc/passwd", "resource_uri").is_err());
/// ```
#[allow(dead_code)] // Security infrastructure for future hardening phases
pub fn validate_identifier(id: &str, identifier_type: &str) -> Result<(), ProtocolError> {
    if id.is_empty() {
        return Err(ProtocolError::InvalidInput(format!(
            "{identifier_type} cannot be empty"
        )));
    }

    if id.len() > 256 {
        return Err(ProtocolError::InvalidInput(format!(
            "{identifier_type} too long (max 256 characters)"
        )));
    }

    // Prevent path traversal
    if id.contains("..") {
        return Err(ProtocolError::PathTraversal(format!(
            "{identifier_type}: {}",
            id
        )));
    }

    // Prevent Windows path traversal
    if id.contains('\\') {
        return Err(ProtocolError::PathTraversal(format!(
            "{identifier_type}: {}",
            id
        )));
    }

    // Check for valid characters (alphanumeric, hyphen, underscore, slash, colon, dot)
    // Allows URIs like "ui://communitas/contacts"
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == ':' || c == '.')
    {
        return Err(ProtocolError::InvalidInput(format!(
            "{identifier_type} contains invalid characters"
        )));
    }

    // Prevent leading/trailing special characters (except for URI scheme prefixes)
    let trimmed = id.trim_start_matches("ui://").trim_start_matches("mcp://");
    let first = trimmed.chars().next();
    let last = trimmed.chars().last();
    if first == Some('/')
        || first == Some('.')
        || first == Some('-')
        || first == Some('_')
        || last == Some('/')
        || last == Some('.')
        || last == Some('-')
        || last == Some('_')
    {
        return Err(ProtocolError::InvalidInput(format!(
            "{identifier_type} cannot start or end with special character"
        )));
    }

    Ok(())
}

/// Validates an origin string for postMessage security
///
/// # Validation Rules
/// - Must be a valid HTTP/HTTPS origin
/// - Must not contain javascript: or data: schemes
///
/// # Examples
/// ```
/// use communitas_mcp::protocol::validate_origin;
/// assert!(validate_origin("https://localhost:8443").is_ok());
/// assert!(validate_origin("javascript:alert('xss')").is_err());
/// ```
#[allow(dead_code)] // Used in Phase 4 security hardening
pub fn validate_origin(origin: &str) -> Result<(), ProtocolError> {
    if origin.is_empty() {
        return Err(ProtocolError::InvalidOrigin(
            "Origin cannot be empty".to_string(),
        ));
    }

    // Prevent dangerous schemes
    let origin_lower = origin.to_lowercase();
    if origin_lower.starts_with("javascript:")
        || origin_lower.starts_with("data:")
        || origin_lower.starts_with("file:")
        || origin_lower.starts_with("ftp:")
    {
        return Err(ProtocolError::InvalidOrigin(format!(
            "Dangerous origin scheme: {origin}"
        )));
    }

    // Must start with http:// or https://
    if !origin_lower.starts_with("http://") && !origin_lower.starts_with("https://") {
        return Err(ProtocolError::InvalidOrigin(format!(
            "Origin must use HTTP or HTTPS scheme: {origin}"
        )));
    }

    Ok(())
}

/// MCP Initialize request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: ClientCapabilities,
    pub client_info: ClientInfo,
}

/// MCP Client capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub roots: Option<RootsCapability>,
    #[serde(default)]
    pub sampling: Option<SamplingCapability>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootsCapability {
    #[serde(default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplingCapability {}

/// MCP Client info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Initialize response result (basic version without extensions)
#[allow(dead_code)] // Kept for backwards compatibility; use InitializeResultWithExtensions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(default)]
    pub list_changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCapability {
    #[serde(default)]
    pub subscribe: bool,
    #[serde(default)]
    pub list_changed: bool,
}

/// MCP Server info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP Tool list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolListResult {
    pub tools: Vec<Tool>,
}

/// MCP Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

/// MCP Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(default, rename = "isError")]
    pub is_error: bool,
}

/// MCP Tool content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub _meta: Option<ResourceMeta>,
}

/// MCP Resource list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceListResult {
    pub resources: Vec<Resource>,
}

/// MCP Resource read parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadParams {
    pub uri: String,
}

/// MCP Resource read result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReadResult {
    pub contents: Vec<ResourceContent>,
}

/// MCP Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

// =============================================================================
// MCP Apps Extension Types (SEP-1865)
// =============================================================================
//
// These types implement the MCP Apps UI extension for interactive rendering in
// MCP hosts (Claude Desktop, ChatGPT, VS Code). They are used by:
// - ui_resources.rs (Phase 1.2) - UI resource registry and routing
// - tools.rs (Phase 2.x) - Adding _meta.ui to tool responses
// - http.rs (Phase 1.2) - HTTP transport UI resource handling
//
// Note: Some types may appear unused until Phase 1.2 completes UI resource integration.

/// UI metadata for tools (MCP Apps extension)
///
/// Tools can include UI metadata to indicate they support interactive rendering.
/// The `resource_uri` points to a UI resource that hosts can render in a sandboxed iframe.
#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiToolMeta {
    /// URI of the UI resource to render (e.g., "ui://communitas/contacts")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,

    /// Visibility scopes for this tool's UI
    /// - "model": Visible to the AI model
    /// - "app": Visible to the MCP host application
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visibility: Vec<String>,
}

/// Metadata wrapper for tool results (MCP Apps extension)
#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMeta {
    /// UI metadata for interactive rendering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<McpUiToolMeta>,
}

/// Extended tool call result with _meta support (MCP Apps extension)
#[allow(dead_code)] // Used by tools.rs in Phase 2.x
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResultWithMeta {
    pub content: Vec<ToolContent>,
    #[serde(default, rename = "isError")]
    pub is_error: bool,
    /// MCP Apps metadata for UI rendering
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<ToolResultMeta>,
}

#[allow(dead_code)] // Used by tools.rs in Phase 2.x
impl ToolCallResultWithMeta {
    /// Create a successful result without UI metadata
    pub fn success(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: false,
            _meta: None,
        }
    }

    /// Create a successful result with UI metadata
    pub fn success_with_ui(text: String, resource_uri: &str) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: false,
            _meta: Some(ToolResultMeta {
                ui: Some(McpUiToolMeta {
                    resource_uri: Some(resource_uri.to_string()),
                    visibility: vec!["model".to_string(), "app".to_string()],
                }),
            }),
        }
    }

    /// Create an error result
    pub fn error(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: true,
            _meta: None,
        }
    }

    /// Convert from basic ToolCallResult
    pub fn from_basic(result: ToolCallResult) -> Self {
        Self {
            content: result.content,
            is_error: result.is_error,
            _meta: None,
        }
    }
}

/// Tool definition with _meta support (MCP Apps extension)
#[allow(dead_code)] // Used by tools.rs in Phase 2.x
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolWithMeta {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    /// MCP Apps metadata for UI-enabled tools
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<ToolDefinitionMeta>,
}

/// Metadata for tool definitions (MCP Apps extension)
#[allow(dead_code)] // Used by tools.rs in Phase 2.x
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinitionMeta {
    /// UI configuration for this tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<McpUiToolMeta>,
}

impl From<Tool> for ToolWithMeta {
    fn from(tool: Tool) -> Self {
        Self {
            name: tool.name,
            description: tool.description,
            input_schema: tool.input_schema,
            _meta: None,
        }
    }
}

/// Content Security Policy configuration for UI resources (MCP Apps extension)
#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiResourceCsp {
    /// Allowed domains for network requests (fetch, XHR)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connect_domains: Vec<String>,

    /// Allowed domains for static resources (images, fonts, scripts)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub resource_domains: Vec<String>,

    /// Allowed domains for nested iframes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frame_domains: Vec<String>,

    /// Allowed domains for base URI
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_uri_domains: Vec<String>,
}

/// Metadata for UI resources (MCP Apps extension)
#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiResourceMeta {
    /// Content Security Policy configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<UiResourceCsp>,

    /// Whether the UI prefers to have a visible border
    #[serde(default)]
    pub prefers_border: bool,

    /// Requested permissions (e.g., "camera", "microphone")
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
}

/// Resource definition with _meta support (MCP Apps extension)
#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceWithMeta {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// MCP Apps metadata for UI resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<ResourceMeta>,
}

/// Metadata for resource definitions (MCP Apps extension)
#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMeta {
    /// UI-specific metadata for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiResourceMeta>,
}

#[allow(dead_code)] // Used by ui_resources.rs in Phase 1.2
impl ResourceMeta {
    /// Create UI resource metadata with default CSP
    pub fn ui_default() -> Self {
        Self {
            ui: Some(UiResourceMeta::default()),
        }
    }
}

impl From<Resource> for ResourceWithMeta {
    fn from(resource: Resource) -> Self {
        Self {
            uri: resource.uri,
            name: resource.name,
            description: resource.description,
            mime_type: resource.mime_type,
            _meta: resource._meta,
        }
    }
}

/// MCP Apps UI extension capability
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiExtensionCapability {
    /// Supported MIME types for UI resources
    #[serde(default)]
    pub mime_types: Vec<String>,
}

impl Default for UiExtensionCapability {
    fn default() -> Self {
        Self {
            mime_types: vec!["text/html;profile=mcp-app".to_string()],
        }
    }
}

#[allow(dead_code)] // Standard constructor pattern
impl UiExtensionCapability {
    /// Create with standard MCP Apps MIME type
    pub fn new() -> Self {
        Self::default()
    }
}

/// Server extensions including MCP Apps UI support
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerExtensions {
    /// MCP Apps UI extension
    #[serde(
        rename = "io.modelcontextprotocol/ui",
        skip_serializing_if = "Option::is_none"
    )]
    pub ui: Option<UiExtensionCapability>,
}

impl ServerExtensions {
    /// Create with MCP Apps UI extension enabled
    pub fn with_ui() -> Self {
        Self {
            ui: Some(UiExtensionCapability::default()),
        }
    }
}

/// Extended initialize result with extensions support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResultWithExtensions {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ServerExtensions>,
    pub server_info: ServerInfo,
}

impl InitializeResultWithExtensions {
    /// Create initialize result with MCP Apps UI extension enabled
    pub fn with_ui_support(
        protocol_version: impl Into<String>,
        server_info: ServerInfo,
        tools: Option<ToolsCapability>,
        resources: Option<ResourcesCapability>,
    ) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            capabilities: ServerCapabilities { tools, resources },
            extensions: Some(ServerExtensions::with_ui()),
            server_info,
        }
    }
}

// =============================================================================
// AI Context Types (Phase 9.3)
// =============================================================================
//
// These types provide context hints to AI hosts about the current UI state,
// enabling the AI to understand what the user is viewing, what's selected,
// pending changes, and any error states. This allows for more contextual
// assistance without requiring clarifying questions.
//
// Context is included in tool responses via `_meta.ui.context`.

/// AI context hints for tool responses
///
/// Provides contextual information about the UI state to help AI hosts
/// understand the user's current situation and provide better assistance.
///
/// # Example
/// ```json
/// {
///   "_meta": {
///     "ui": {
///       "resourceUri": "ui://communitas/kanban",
///       "context": {
///         "current_view": { "widget": "kanban", "view_id": "board-123" },
///         "selection_state": { "selected_ids": ["card-1"], "selection_type": "card" }
///       }
///     }
///   }
/// }
/// ```
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AiContext {
    /// Current view state - what widget and view is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_view: Option<CurrentView>,

    /// Selection state - what items are selected
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_state: Option<SelectionState>,

    /// Pending unsaved actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_actions: Option<PendingActions>,

    /// Current error state for troubleshooting
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_state: Option<ErrorState>,
}

#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
impl AiContext {
    /// Create an empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context with just current view
    pub fn with_view(widget: impl Into<String>) -> Self {
        Self {
            current_view: Some(CurrentView {
                widget: widget.into(),
                view_id: None,
                view_mode: None,
                filter: None,
            }),
            ..Default::default()
        }
    }

    /// Add selection state to context
    pub fn with_selection(mut self, selection: SelectionState) -> Self {
        self.selection_state = Some(selection);
        self
    }

    /// Add pending actions to context
    pub fn with_pending(mut self, pending: PendingActions) -> Self {
        self.pending_actions = Some(pending);
        self
    }

    /// Add error state to context
    pub fn with_error(mut self, error: ErrorState) -> Self {
        self.error_state = Some(error);
        self
    }

    /// Check if context has any meaningful data
    pub fn is_empty(&self) -> bool {
        self.current_view.is_none()
            && self.selection_state.is_none()
            && self.pending_actions.is_none()
            && self.error_state.is_none()
    }
}

/// Current view state - tracks what widget and view is active
///
/// # Fields
/// - `widget`: The active widget name (e.g., "kanban", "contacts", "drive")
/// - `view_id`: Optional specific view/board/folder ID
/// - `view_mode`: Optional view mode (e.g., "board", "list", "grid")
/// - `filter`: Optional active filter description
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct CurrentView {
    /// Widget name (e.g., "kanban", "contacts", "drive")
    pub widget: String,

    /// Specific view/board/folder ID being viewed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_id: Option<String>,

    /// View mode (e.g., "board", "list", "grid", "detail")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,

    /// Active filter description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
impl CurrentView {
    /// Create a basic view for a widget
    pub fn new(widget: impl Into<String>) -> Self {
        Self {
            widget: widget.into(),
            view_id: None,
            view_mode: None,
            filter: None,
        }
    }

    /// Create a view with ID
    pub fn with_id(widget: impl Into<String>, view_id: impl Into<String>) -> Self {
        Self {
            widget: widget.into(),
            view_id: Some(view_id.into()),
            view_mode: None,
            filter: None,
        }
    }

    /// Set the view mode
    pub fn mode(mut self, mode: impl Into<String>) -> Self {
        self.view_mode = Some(mode.into());
        self
    }

    /// Set the filter
    pub fn filtered(mut self, filter: impl Into<String>) -> Self {
        self.filter = Some(filter.into());
        self
    }
}

/// Selection state - tracks what items are selected
///
/// # Fields
/// - `selected_ids`: List of selected item IDs
/// - `selection_type`: Type of items selected (e.g., "card", "contact", "file")
/// - `count`: Number of selected items (for convenience)
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SelectionState {
    /// IDs of selected items
    #[serde(default)]
    pub selected_ids: Vec<String>,

    /// Type of items selected (e.g., "card", "contact", "file")
    pub selection_type: String,

    /// Number of selected items
    #[serde(default)]
    pub count: usize,
}

#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
impl SelectionState {
    /// Create selection state for given type and IDs
    pub fn new(selection_type: impl Into<String>, ids: Vec<String>) -> Self {
        let count = ids.len();
        Self {
            selected_ids: ids,
            selection_type: selection_type.into(),
            count,
        }
    }

    /// Create single selection
    pub fn single(selection_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            selected_ids: vec![id.into()],
            selection_type: selection_type.into(),
            count: 1,
        }
    }

    /// Create empty selection (nothing selected)
    pub fn none(selection_type: impl Into<String>) -> Self {
        Self {
            selected_ids: vec![],
            selection_type: selection_type.into(),
            count: 0,
        }
    }
}

/// Pending actions - tracks unsaved changes
///
/// # Fields
/// - `has_unsaved`: Whether there are unsaved changes
/// - `unsaved_items`: List of item IDs with unsaved changes
/// - `action_type`: Type of pending action (edit, create, delete, move)
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PendingActions {
    /// Whether there are unsaved changes
    pub has_unsaved: bool,

    /// IDs of items with unsaved changes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unsaved_items: Vec<String>,

    /// Type of pending action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_type: Option<PendingActionType>,
}

/// Types of pending actions
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PendingActionType {
    /// Item being edited
    Edit,
    /// New item not yet saved
    Create,
    /// Pending deletion confirmation
    Delete,
    /// Item being relocated
    Move,
    /// Draft message/content
    Draft,
}

#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
impl PendingActions {
    /// Create pending action state
    pub fn new(action_type: PendingActionType, items: Vec<String>) -> Self {
        Self {
            has_unsaved: !items.is_empty(),
            unsaved_items: items,
            action_type: Some(action_type),
        }
    }

    /// Create state with no pending actions
    pub fn none() -> Self {
        Self {
            has_unsaved: false,
            unsaved_items: vec![],
            action_type: None,
        }
    }

    /// Create edit pending state for single item
    pub fn editing(id: impl Into<String>) -> Self {
        Self::new(PendingActionType::Edit, vec![id.into()])
    }

    /// Create create pending state (new unsaved item)
    pub fn creating() -> Self {
        Self {
            has_unsaved: true,
            unsaved_items: vec![],
            action_type: Some(PendingActionType::Create),
        }
    }

    /// Create draft pending state
    pub fn draft() -> Self {
        Self {
            has_unsaved: true,
            unsaved_items: vec![],
            action_type: Some(PendingActionType::Draft),
        }
    }
}

/// Error state - tracks current errors for troubleshooting
///
/// # Fields
/// - `has_error`: Whether an error is currently present
/// - `error_type`: Category of error
/// - `error_message`: Human-readable error message
/// - `recoverable`: Whether the error can be recovered from
/// - `recovery_hint`: Optional hint for how to recover
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ErrorState {
    /// Whether an error is currently present
    pub has_error: bool,

    /// Category of error
    pub error_type: ErrorType,

    /// Human-readable error message
    pub error_message: String,

    /// Whether the error can be recovered from
    #[serde(default = "default_recoverable")]
    pub recoverable: bool,

    /// Optional hint for how to recover
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_hint: Option<String>,
}

fn default_recoverable() -> bool {
    true
}

/// Types of errors
#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Network/connection error
    Network,
    /// Invalid input/validation error
    Validation,
    /// Access denied/permission error
    Permission,
    /// Request timed out
    Timeout,
    /// Server/internal error
    Internal,
    /// Resource not found
    NotFound,
    /// Quota/limit exceeded
    QuotaExceeded,
    /// Sync/conflict error
    Sync,
}

#[allow(dead_code)] // Used by server.rs and widgets in Tasks 2-10
impl ErrorState {
    /// Create error state
    pub fn new(error_type: ErrorType, message: impl Into<String>, recoverable: bool) -> Self {
        Self {
            has_error: true,
            error_type,
            error_message: message.into(),
            recoverable,
            recovery_hint: None,
        }
    }

    /// Create network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Network, message, true)
            .with_hint("Check your internet connection and try again")
    }

    /// Create validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Validation, message, true)
    }

    /// Create permission error
    pub fn permission(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Permission, message, false)
            .with_hint("You may need to request access from the owner")
    }

    /// Create timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(ErrorType::Timeout, message, true)
            .with_hint("The operation took too long. Try again")
    }

    /// Create no error state
    pub fn none() -> Self {
        Self {
            has_error: false,
            error_type: ErrorType::Internal,
            error_message: String::new(),
            recoverable: true,
            recovery_hint: None,
        }
    }

    /// Add recovery hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hint = Some(hint.into());
        self
    }
}

/// Extended UI metadata with AI context support
///
/// Extends `McpUiToolMeta` to include AI context hints.
#[allow(dead_code)] // Used by server.rs in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpUiToolMetaWithContext {
    /// URI of the UI resource to render
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,

    /// Visibility scopes for this tool's UI
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub visibility: Vec<String>,

    /// AI context hints about current UI state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<AiContext>,
}

#[allow(dead_code)] // Used by server.rs in Tasks 2-10
impl McpUiToolMetaWithContext {
    /// Create UI metadata with context
    pub fn new(resource_uri: impl Into<String>, context: AiContext) -> Self {
        Self {
            resource_uri: Some(resource_uri.into()),
            visibility: vec!["model".to_string(), "app".to_string()],
            context: if context.is_empty() {
                None
            } else {
                Some(context)
            },
        }
    }

    /// Create UI metadata without context
    pub fn without_context(resource_uri: impl Into<String>) -> Self {
        Self {
            resource_uri: Some(resource_uri.into()),
            visibility: vec!["model".to_string(), "app".to_string()],
            context: None,
        }
    }
}

/// Extended tool result metadata with AI context support
#[allow(dead_code)] // Used by server.rs in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultMetaWithContext {
    /// UI metadata with context hints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<McpUiToolMetaWithContext>,
}

/// Extended tool call result with AI context in _meta
#[allow(dead_code)] // Used by server.rs in Tasks 2-10
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResultWithContext {
    pub content: Vec<ToolContent>,
    #[serde(default, rename = "isError")]
    pub is_error: bool,
    /// MCP Apps metadata with AI context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<ToolResultMetaWithContext>,
}

#[allow(dead_code)] // Used by server.rs in Tasks 2-10
impl ToolCallResultWithContext {
    /// Create a successful result without UI metadata
    pub fn success(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: false,
            _meta: None,
        }
    }

    /// Create a successful result with UI metadata and context
    pub fn success_with_context(text: String, resource_uri: &str, context: AiContext) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: false,
            _meta: Some(ToolResultMetaWithContext {
                ui: Some(McpUiToolMetaWithContext::new(resource_uri, context)),
            }),
        }
    }

    /// Create a successful result with UI but no context
    pub fn success_with_ui(text: String, resource_uri: &str) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: false,
            _meta: Some(ToolResultMetaWithContext {
                ui: Some(McpUiToolMetaWithContext::without_context(resource_uri)),
            }),
        }
    }

    /// Create an error result
    pub fn error(text: String) -> Self {
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: true,
            _meta: None,
        }
    }

    /// Create an error result with context for troubleshooting
    pub fn error_with_context(text: String, resource_uri: &str, error: ErrorState) -> Self {
        let context = AiContext::default().with_error(error);
        Self {
            content: vec![ToolContent::Text { text }],
            is_error: true,
            _meta: Some(ToolResultMetaWithContext {
                ui: Some(McpUiToolMetaWithContext::new(resource_uri, context)),
            }),
        }
    }

    /// Convert from basic ToolCallResult
    pub fn from_basic(result: ToolCallResult) -> Self {
        Self {
            content: result.content,
            is_error: result.is_error,
            _meta: None,
        }
    }

    /// Convert from ToolCallResultWithMeta
    pub fn from_with_meta(result: ToolCallResultWithMeta) -> Self {
        Self {
            content: result.content,
            is_error: result.is_error,
            _meta: result._meta.map(|m| ToolResultMetaWithContext {
                ui: m.ui.map(|u| McpUiToolMetaWithContext {
                    resource_uri: u.resource_uri,
                    visibility: u.visibility,
                    context: None,
                }),
            }),
        }
    }

    /// Convert from basic ToolCallResult with UI context
    ///
    /// This is used to add AI context hints to tool responses so the AI host
    /// can understand the current widget state (what's being viewed, selected, etc.)
    pub fn from_basic_with_context(
        result: ToolCallResult,
        resource_uri: Option<&str>,
        context: AiContext,
    ) -> Self {
        let has_context = context.current_view.is_some()
            || context.selection_state.is_some()
            || context.pending_actions.is_some()
            || context.error_state.is_some();

        let meta = if has_context || resource_uri.is_some() {
            Some(ToolResultMetaWithContext {
                ui: Some(McpUiToolMetaWithContext {
                    resource_uri: resource_uri.map(String::from),
                    visibility: vec!["model".to_string(), "app".to_string()],
                    context: if has_context { Some(context) } else { None },
                }),
            })
        } else {
            None
        };

        Self {
            content: result.content,
            is_error: result.is_error,
            _meta: meta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.method, "initialize");
    }

    #[test]
    fn test_serialize_response() {
        let response = JsonRpcResponse::success(
            Some(serde_json::json!(1)),
            serde_json::json!({"status": "ok"}),
        );
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("jsonrpc"));
        assert!(json.contains("result"));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcError::parse_error().code, -32700);
        assert_eq!(JsonRpcError::invalid_request("x").code, -32600);
        assert_eq!(JsonRpcError::method_not_found("x").code, -32601);
        assert_eq!(JsonRpcError::invalid_params("x").code, -32602);
        assert_eq!(JsonRpcError::internal_error("x").code, -32603);
    }

    // MCP Apps Extension Tests

    #[test]
    fn test_tool_result_with_meta_success() {
        let result = ToolCallResultWithMeta::success("Hello".to_string());
        assert!(!result.is_error);
        assert!(result._meta.is_none());
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_tool_result_with_ui_meta() {
        let result =
            ToolCallResultWithMeta::success_with_ui("Contacts".to_string(), "ui://contacts");
        assert!(!result.is_error);
        assert!(result._meta.is_some());

        let meta = result._meta.unwrap();
        let ui = meta.ui.unwrap();
        assert_eq!(ui.resource_uri, Some("ui://contacts".to_string()));
        assert!(ui.visibility.contains(&"model".to_string()));
        assert!(ui.visibility.contains(&"app".to_string()));
    }

    #[test]
    fn test_tool_result_with_meta_serialization() {
        let result =
            ToolCallResultWithMeta::success_with_ui("Data".to_string(), "ui://communitas/contacts");
        let json = serde_json::to_string(&result).unwrap();

        // Verify _meta field is present with correct structure
        assert!(json.contains("\"_meta\""));
        assert!(json.contains("\"resourceUri\""));
        assert!(json.contains("ui://communitas/contacts"));
    }

    #[test]
    fn test_tool_with_meta_from_basic_tool() {
        let basic_tool = Tool {
            name: "list_contacts".to_string(),
            description: "List all contacts".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
        };

        let with_meta: ToolWithMeta = basic_tool.into();
        assert_eq!(with_meta.name, "list_contacts");
        assert!(with_meta._meta.is_none());
    }

    #[test]
    fn test_ui_resource_csp_serialization() {
        let csp = UiResourceCsp {
            connect_domains: vec!["api.example.com".to_string()],
            resource_domains: vec![],
            frame_domains: vec![],
            base_uri_domains: vec![],
        };

        let json = serde_json::to_string(&csp).unwrap();
        assert!(json.contains("connectDomains"));
        assert!(json.contains("api.example.com"));
        // Empty arrays should be omitted
        assert!(!json.contains("resourceDomains"));
    }

    #[test]
    fn test_server_extensions_serialization() {
        let extensions = ServerExtensions {
            ui: Some(UiExtensionCapability::default()),
        };

        let json = serde_json::to_string(&extensions).unwrap();
        // Verify the extension key format
        assert!(json.contains("io.modelcontextprotocol/ui"));
        assert!(json.contains("mimeTypes"));
        assert!(json.contains("text/html;profile=mcp-app"));
    }

    #[test]
    fn test_resource_with_meta() {
        let resource = ResourceWithMeta {
            uri: "ui://communitas/contacts".to_string(),
            name: "Contacts UI".to_string(),
            description: Some("Interactive contact list".to_string()),
            mime_type: Some("text/html;profile=mcp-app".to_string()),
            _meta: Some(ResourceMeta {
                ui: Some(UiResourceMeta {
                    csp: Some(UiResourceCsp::default()),
                    prefers_border: true,
                    permissions: vec![],
                }),
            }),
        };

        let json = serde_json::to_string(&resource).unwrap();
        assert!(json.contains("ui://communitas/contacts"));
        assert!(json.contains("text/html;profile=mcp-app"));
        assert!(json.contains("prefersBorder"));
    }

    // Security validation tests

    #[test]
    fn test_validate_identifier_valid() {
        assert!(validate_identifier("ui://communitas/contacts", "resource_uri").is_ok());
        assert!(validate_identifier("app-name-123", "app_id").is_ok());
        assert!(validate_identifier("widget_name", "widget_id").is_ok());
    }

    #[test]
    fn test_validate_identifier_path_traversal() {
        assert!(validate_identifier("../../../etc/passwd", "resource_uri").is_err());
        assert!(validate_identifier("../escape", "app_id").is_err());
        assert!(validate_identifier("path/../to/file", "widget_id").is_err());
    }

    #[test]
    fn test_validate_identifier_backslash() {
        assert!(validate_identifier("path\\to\\file", "resource_uri").is_err());
        assert!(validate_identifier("C:\\Windows\\System32", "app_id").is_err());
    }

    #[test]
    fn test_validate_identifier_empty() {
        assert!(validate_identifier("", "app_id").is_err());
    }

    #[test]
    fn test_validate_identifier_too_long() {
        let too_long = "a".repeat(257);
        assert!(validate_identifier(&too_long, "app_id").is_err());
    }

    #[test]
    fn test_validate_identifier_invalid_characters() {
        assert!(validate_identifier("app<script>", "app_id").is_err());
        assert!(validate_identifier("app&name", "app_id").is_err());
        assert!(validate_identifier("app|name", "app_id").is_err());
        assert!(validate_identifier("app\x00null", "app_id").is_err());
    }

    // =============================================================================
    // AI Context Tests (Phase 9.3)
    // =============================================================================

    #[test]
    fn test_ai_context_empty() {
        let context = AiContext::new();
        assert!(context.is_empty());

        let json = serde_json::to_string(&context).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_ai_context_with_view() {
        let context = AiContext::with_view("kanban");
        assert!(!context.is_empty());
        assert!(context.current_view.is_some());

        let view = context.current_view.unwrap();
        assert_eq!(view.widget, "kanban");
    }

    #[test]
    fn test_ai_context_serialization_roundtrip() {
        let context = AiContext {
            current_view: Some(CurrentView {
                widget: "kanban".to_string(),
                view_id: Some("board-123".to_string()),
                view_mode: Some("board".to_string()),
                filter: None,
            }),
            selection_state: Some(SelectionState {
                selected_ids: vec!["card-1".to_string(), "card-2".to_string()],
                selection_type: "card".to_string(),
                count: 2,
            }),
            pending_actions: Some(PendingActions {
                has_unsaved: true,
                unsaved_items: vec!["card-1".to_string()],
                action_type: Some(PendingActionType::Edit),
            }),
            error_state: None,
        };

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: AiContext = serde_json::from_str(&json).unwrap();

        assert_eq!(context, deserialized);
    }

    #[test]
    fn test_current_view_builder() {
        let view = CurrentView::with_id("drive", "folder-abc")
            .mode("list")
            .filtered("*.pdf");

        assert_eq!(view.widget, "drive");
        assert_eq!(view.view_id, Some("folder-abc".to_string()));
        assert_eq!(view.view_mode, Some("list".to_string()));
        assert_eq!(view.filter, Some("*.pdf".to_string()));
    }

    #[test]
    fn test_current_view_serialization() {
        let view = CurrentView::new("contacts");
        let json = serde_json::to_string(&view).unwrap();

        assert!(json.contains("\"widget\":\"contacts\""));
        // Optional fields should be omitted
        assert!(!json.contains("view_id"));
        assert!(!json.contains("view_mode"));
    }

    #[test]
    fn test_selection_state_single() {
        let selection = SelectionState::single("contact", "contact-123");
        assert_eq!(selection.count, 1);
        assert_eq!(selection.selected_ids.len(), 1);
        assert_eq!(selection.selection_type, "contact");
    }

    #[test]
    fn test_selection_state_multiple() {
        let selection = SelectionState::new(
            "file",
            vec![
                "file-1".to_string(),
                "file-2".to_string(),
                "file-3".to_string(),
            ],
        );
        assert_eq!(selection.count, 3);
        assert_eq!(selection.selected_ids.len(), 3);
    }

    #[test]
    fn test_selection_state_none() {
        let selection = SelectionState::none("card");
        assert_eq!(selection.count, 0);
        assert!(selection.selected_ids.is_empty());
    }

    #[test]
    fn test_pending_actions_editing() {
        let pending = PendingActions::editing("card-1");
        assert!(pending.has_unsaved);
        assert_eq!(pending.action_type, Some(PendingActionType::Edit));
        assert_eq!(pending.unsaved_items, vec!["card-1".to_string()]);
    }

    #[test]
    fn test_pending_actions_draft() {
        let pending = PendingActions::draft();
        assert!(pending.has_unsaved);
        assert_eq!(pending.action_type, Some(PendingActionType::Draft));
    }

    #[test]
    fn test_pending_actions_none() {
        let pending = PendingActions::none();
        assert!(!pending.has_unsaved);
        assert!(pending.unsaved_items.is_empty());
        assert!(pending.action_type.is_none());
    }

    #[test]
    fn test_pending_action_type_serialization() {
        let action = PendingActionType::Edit;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"edit\"");

        let action = PendingActionType::Create;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"create\"");

        let action = PendingActionType::Delete;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"delete\"");

        let action = PendingActionType::Move;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"move\"");

        let action = PendingActionType::Draft;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"draft\"");
    }

    #[test]
    fn test_error_state_network() {
        let error = ErrorState::network("Connection failed");
        assert!(error.has_error);
        assert_eq!(error.error_type, ErrorType::Network);
        assert!(error.recoverable);
        assert!(error.recovery_hint.is_some());
    }

    #[test]
    fn test_error_state_permission() {
        let error = ErrorState::permission("Access denied");
        assert!(error.has_error);
        assert_eq!(error.error_type, ErrorType::Permission);
        assert!(!error.recoverable);
    }

    #[test]
    fn test_error_state_none() {
        let error = ErrorState::none();
        assert!(!error.has_error);
    }

    #[test]
    fn test_error_type_serialization() {
        let error_type = ErrorType::Network;
        let json = serde_json::to_string(&error_type).unwrap();
        assert_eq!(json, "\"network\"");

        let error_type = ErrorType::Validation;
        let json = serde_json::to_string(&error_type).unwrap();
        assert_eq!(json, "\"validation\"");

        let error_type = ErrorType::QuotaExceeded;
        let json = serde_json::to_string(&error_type).unwrap();
        assert_eq!(json, "\"quota_exceeded\"");
    }

    #[test]
    fn test_tool_result_with_context_success() {
        let result = ToolCallResultWithContext::success("Hello".to_string());
        assert!(!result.is_error);
        assert!(result._meta.is_none());
    }

    #[test]
    fn test_tool_result_with_context_full() {
        let context = AiContext::with_view("kanban")
            .with_selection(SelectionState::single("card", "card-1"))
            .with_pending(PendingActions::editing("card-1"));

        let result = ToolCallResultWithContext::success_with_context(
            "Card details".to_string(),
            "ui://communitas/kanban",
            context,
        );

        assert!(!result.is_error);
        assert!(result._meta.is_some());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"context\""));
        assert!(json.contains("\"current_view\""));
        assert!(json.contains("\"selection_state\""));
        assert!(json.contains("\"pending_actions\""));
    }

    #[test]
    fn test_tool_result_with_context_error() {
        let error = ErrorState::network("Connection timeout");
        let result = ToolCallResultWithContext::error_with_context(
            "Failed to load".to_string(),
            "ui://communitas/drive",
            error,
        );

        assert!(result.is_error);
        assert!(result._meta.is_some());

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"error_state\""));
        assert!(json.contains("\"network\""));
    }

    #[test]
    fn test_ai_context_json_structure() {
        // Verify the JSON structure matches the plan specification
        let context = AiContext {
            current_view: Some(CurrentView {
                widget: "kanban".to_string(),
                view_id: Some("board-123".to_string()),
                view_mode: Some("board".to_string()),
                filter: None,
            }),
            selection_state: Some(SelectionState {
                selected_ids: vec!["card-1".to_string(), "card-2".to_string()],
                selection_type: "card".to_string(),
                count: 2,
            }),
            pending_actions: Some(PendingActions {
                has_unsaved: true,
                unsaved_items: vec!["card-1".to_string()],
                action_type: Some(PendingActionType::Edit),
            }),
            error_state: None,
        };

        let json = serde_json::to_string_pretty(&context).unwrap();

        // Verify snake_case field names (as per plan)
        assert!(json.contains("\"current_view\""));
        assert!(json.contains("\"selection_state\""));
        assert!(json.contains("\"pending_actions\""));
        assert!(json.contains("\"selected_ids\""));
        assert!(json.contains("\"selection_type\""));
        assert!(json.contains("\"has_unsaved\""));
        assert!(json.contains("\"unsaved_items\""));
        assert!(json.contains("\"action_type\""));
    }

    #[test]
    fn test_mcp_ui_tool_meta_with_context() {
        let context = AiContext::with_view("contacts");
        let meta = McpUiToolMetaWithContext::new("ui://communitas/contacts", context);

        assert_eq!(
            meta.resource_uri,
            Some("ui://communitas/contacts".to_string())
        );
        assert!(meta.visibility.contains(&"model".to_string()));
        assert!(meta.context.is_some());
    }

    #[test]
    fn test_mcp_ui_tool_meta_without_context() {
        let meta = McpUiToolMetaWithContext::without_context("ui://communitas/drive");

        assert_eq!(meta.resource_uri, Some("ui://communitas/drive".to_string()));
        assert!(meta.context.is_none());
    }

    #[test]
    fn test_mcp_ui_tool_meta_empty_context_omitted() {
        let empty_context = AiContext::new();
        let meta = McpUiToolMetaWithContext::new("ui://communitas/settings", empty_context);

        // Empty context should be None (omitted in serialization)
        assert!(meta.context.is_none());
    }

    #[test]
    fn test_from_basic_with_context() {
        // Create a basic tool result
        let basic_result = ToolCallResult {
            content: vec![ToolContent::Text {
                text: "Contact list retrieved".to_string(),
            }],
            is_error: false,
        };

        // Create context with current view
        let context = AiContext::with_view("contacts");

        // Convert with context
        let result = ToolCallResultWithContext::from_basic_with_context(
            basic_result,
            Some("ui://communitas/contacts"),
            context,
        );

        assert!(!result.is_error);
        assert!(result._meta.is_some());

        let meta = result._meta.unwrap();
        let ui = meta.ui.unwrap();
        assert_eq!(
            ui.resource_uri,
            Some("ui://communitas/contacts".to_string())
        );
        assert!(ui.context.is_some());

        let ctx = ui.context.unwrap();
        assert!(ctx.current_view.is_some());
        assert_eq!(ctx.current_view.unwrap().widget, "contacts");
    }

    #[test]
    fn test_from_basic_with_empty_context_no_uri() {
        let basic_result = ToolCallResult {
            content: vec![ToolContent::Text {
                text: "Health check OK".to_string(),
            }],
            is_error: false,
        };

        // Empty context and no resource URI should result in no _meta
        let result = ToolCallResultWithContext::from_basic_with_context(
            basic_result,
            None,
            AiContext::new(),
        );

        assert!(!result.is_error);
        assert!(result._meta.is_none()); // No context, no URI = no meta
    }

    #[test]
    fn test_from_basic_with_context_preserves_error_state() {
        let basic_result = ToolCallResult {
            content: vec![ToolContent::Text {
                text: "Failed to load contacts".to_string(),
            }],
            is_error: true,
        };

        let error = ErrorState::network("Connection timeout");
        let context = AiContext::default().with_error(error);

        let result = ToolCallResultWithContext::from_basic_with_context(
            basic_result,
            Some("ui://communitas/contacts"),
            context,
        );

        assert!(result.is_error);
        assert!(result._meta.is_some());

        let ctx = result._meta.unwrap().ui.unwrap().context.unwrap();
        assert!(ctx.error_state.is_some());
    }
}
