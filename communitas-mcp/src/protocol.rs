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
            _meta: None,
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

/// Extended server capabilities with extensions support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilitiesWithExtensions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// MCP protocol extensions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<ServerExtensions>,
}

impl From<ServerCapabilities> for ServerCapabilitiesWithExtensions {
    fn from(caps: ServerCapabilities) -> Self {
        Self {
            tools: caps.tools,
            resources: caps.resources,
            extensions: None,
        }
    }
}

impl ServerCapabilitiesWithExtensions {
    /// Create capabilities with MCP Apps UI extension enabled
    pub fn with_ui_extension(
        tools: Option<ToolsCapability>,
        resources: Option<ResourcesCapability>,
    ) -> Self {
        Self {
            tools,
            resources,
            extensions: Some(ServerExtensions::with_ui()),
        }
    }
}

/// Extended initialize result with extensions support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResultWithExtensions {
    pub protocol_version: String,
    pub capabilities: ServerCapabilitiesWithExtensions,
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
            capabilities: ServerCapabilitiesWithExtensions::with_ui_extension(tools, resources),
            server_info,
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

    #[test]
    fn test_capabilities_with_extensions() {
        let caps = ServerCapabilitiesWithExtensions {
            tools: Some(ToolsCapability { list_changed: false }),
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
            extensions: Some(ServerExtensions {
                ui: Some(UiExtensionCapability::default()),
            }),
        };

        let json = serde_json::to_string(&caps).unwrap();
        assert!(json.contains("tools"));
        assert!(json.contains("resources"));
        assert!(json.contains("extensions"));
        assert!(json.contains("io.modelcontextprotocol/ui"));
    }
}
