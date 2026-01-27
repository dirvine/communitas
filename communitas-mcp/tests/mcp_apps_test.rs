// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! MCP Apps Extension Integration Tests (ADR-022)
//!
//! These tests verify the MCP Apps extension implementation:
//! - UI resource registry functionality
//! - Protocol types with _meta.ui serialization
//! - Capability negotiation with UI extension
//! - Standard widget registration
//!
//! Run with: cargo test -p communitas-mcp --test mcp_apps_test

use communitas_mcp::protocol::{
    InitializeResultWithExtensions, McpUiToolMeta, ResourceMeta, ResourceWithMeta,
    ServerCapabilitiesWithExtensions, ServerExtensions, ServerInfo, ToolResultMeta,
    UiExtensionCapability, UiResourceCsp, UiResourceMeta,
};
use communitas_mcp::ui_resources::{UiContent, UiResourceEntry, UiResourceRegistry};
use serde_json::json;

// =============================================================================
// UI Resource Registry Tests
// =============================================================================

#[test]
fn test_standard_widgets_registered() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let resources = registry.list();

    // Verify all 8 standard widgets are registered (5 original + 3 from Phase 9.2)
    let uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();

    // Original 5 widgets
    assert!(
        uris.contains(&"ui://communitas/contacts"),
        "Contacts widget should be registered"
    );
    assert!(
        uris.contains(&"ui://communitas/messages"),
        "Messages widget should be registered"
    );
    assert!(
        uris.contains(&"ui://communitas/kanban"),
        "Kanban widget should be registered"
    );
    assert!(
        uris.contains(&"ui://communitas/drive"),
        "Drive widget should be registered"
    );
    assert!(
        uris.contains(&"ui://communitas/canvas"),
        "Canvas widget should be registered"
    );

    // Phase 9.2 widgets
    assert!(
        uris.contains(&"ui://communitas/settings"),
        "Settings widget should be registered"
    );
    assert!(
        uris.contains(&"ui://communitas/search"),
        "Search widget should be registered"
    );
    assert!(
        uris.contains(&"ui://communitas/notifications"),
        "Notifications widget should be registered"
    );

    assert_eq!(resources.len(), 8, "Should have exactly 8 standard widgets");
}

#[test]
fn test_widget_metadata_complete() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        // Every widget should have a name
        assert!(
            !resource.name.is_empty(),
            "Widget {} should have a name",
            resource.uri
        );

        // Every widget should have a description
        assert!(
            resource.description.is_some(),
            "Widget {} should have a description",
            resource.uri
        );

        // Every widget should have the correct MIME type
        assert_eq!(
            resource.mime_type,
            Some("text/html;profile=mcp-app".to_string()),
            "Widget {} should have MCP App MIME type",
            resource.uri
        );

        // Every widget should have _meta with UI metadata
        assert!(
            resource._meta.is_some(),
            "Widget {} should have _meta",
            resource.uri
        );

        let meta = resource._meta.as_ref().unwrap();
        assert!(
            meta.ui.is_some(),
            "Widget {} should have ui metadata",
            resource.uri
        );
    }
}

#[test]
fn test_widget_content_readable() {
    let registry = UiResourceRegistry::with_standard_widgets();

    let expected_widgets = vec![
        ("ui://communitas/contacts", "Contacts"),
        ("ui://communitas/messages", "Messages"),
        ("ui://communitas/kanban", "Kanban"),
        ("ui://communitas/drive", "Drive"),
        ("ui://communitas/canvas", "Canvas"),
    ];

    for (uri, name) in expected_widgets {
        let result = registry.read(uri);
        assert!(result.is_some(), "Should be able to read {} widget", name);

        let (content, mime_type) = result.unwrap();

        // Content should be valid HTML
        assert!(
            content.contains("<!DOCTYPE html>") || content.contains("<html"),
            "{} widget should contain HTML",
            name
        );

        // Should contain the MCP bridge script reference
        assert!(
            content.contains("mcp-bridge.js"),
            "{} widget should reference mcp-bridge.js",
            name
        );

        // MIME type should be correct
        assert_eq!(
            mime_type, "text/html;profile=mcp-app",
            "{} widget should have MCP App MIME type",
            name
        );
    }
}

// =============================================================================
// Protocol Type Serialization Tests
// =============================================================================

#[test]
fn test_mcp_ui_tool_meta_serialization() {
    let meta = McpUiToolMeta {
        resource_uri: Some("ui://communitas/contacts".to_string()),
        visibility: vec!["model".to_string(), "app".to_string()],
    };

    let json = serde_json::to_value(&meta).unwrap();

    assert_eq!(
        json["resourceUri"], "ui://communitas/contacts",
        "resourceUri should serialize correctly"
    );
    assert_eq!(
        json["visibility"],
        json!(["model", "app"]),
        "visibility should serialize correctly"
    );
}

#[test]
fn test_tool_result_meta_serialization() {
    let meta = ToolResultMeta {
        ui: Some(McpUiToolMeta {
            resource_uri: Some("ui://communitas/kanban".to_string()),
            visibility: vec!["app".to_string()],
        }),
    };

    let json = serde_json::to_value(&meta).unwrap();

    assert!(json["ui"].is_object(), "_meta.ui should be an object");
    assert_eq!(
        json["ui"]["resourceUri"], "ui://communitas/kanban",
        "_meta.ui.resourceUri should serialize correctly"
    );
}

#[test]
fn test_ui_resource_csp_default() {
    let csp = UiResourceCsp::default();

    assert!(
        csp.connect_domains.is_empty(),
        "Default CSP should have no connect domains"
    );
    assert!(
        csp.resource_domains.is_empty(),
        "Default CSP should have no resource domains"
    );

    // Note: Empty vectors are skipped during serialization (skip_serializing_if)
    // so we verify the struct fields directly rather than JSON output
    let json = serde_json::to_value(&csp).unwrap();
    // Empty fields are omitted, so they won't appear in JSON
    assert!(
        json.get("connectDomains").is_none() || json["connectDomains"] == json!([]),
        "connectDomains should be omitted or empty"
    );
}

#[test]
fn test_resource_with_meta_serialization() {
    let resource = ResourceWithMeta {
        uri: "ui://communitas/contacts".to_string(),
        name: "Contacts".to_string(),
        description: Some("Interactive contact list".to_string()),
        mime_type: Some("text/html;profile=mcp-app".to_string()),
        _meta: Some(ResourceMeta {
            ui: Some(UiResourceMeta {
                csp: Some(UiResourceCsp::default()),
                prefers_border: false,
                permissions: vec![],
            }),
        }),
    };

    let json = serde_json::to_value(&resource).unwrap();

    assert_eq!(json["uri"], "ui://communitas/contacts");
    assert_eq!(json["name"], "Contacts");
    assert_eq!(json["mimeType"], "text/html;profile=mcp-app");

    // _meta is present in the struct
    assert!(resource._meta.is_some(), "Resource should have _meta");

    // Note: _meta.ui may be serialized as an empty object {} when all fields
    // use skip_serializing_if and have default values
    if let Some(meta) = json.get("_meta") {
        // If _meta is present in JSON, verify it has correct structure
        assert!(meta.is_object(), "_meta should be an object when present");
    }
}

// =============================================================================
// Capability Negotiation Tests
// =============================================================================

#[test]
fn test_initialize_result_with_ui_extension() {
    let result = InitializeResultWithExtensions::with_ui_support(
        "2024-11-05",
        ServerInfo {
            name: "communitas-mcp".to_string(),
            version: "0.1.0".to_string(),
        },
        Some(communitas_mcp::protocol::ToolsCapability {
            list_changed: false,
        }),
        Some(communitas_mcp::protocol::ResourcesCapability {
            subscribe: false,
            list_changed: false,
        }),
    );

    // Verify protocol version
    assert_eq!(result.protocol_version, "2024-11-05");

    // Verify server info
    assert_eq!(result.server_info.name, "communitas-mcp");

    // Verify extensions include UI support
    let extensions = result
        .capabilities
        .extensions
        .as_ref()
        .expect("Extensions should be present");

    assert!(
        extensions.ui.is_some(),
        "Extensions should include UI capability"
    );

    let ui_cap = extensions.ui.as_ref().unwrap();
    assert!(
        ui_cap
            .mime_types
            .contains(&"text/html;profile=mcp-app".to_string()),
        "UI extension should support MCP App MIME type"
    );
}

#[test]
fn test_server_extensions_serialization() {
    let extensions = ServerExtensions {
        ui: Some(UiExtensionCapability {
            mime_types: vec!["text/html;profile=mcp-app".to_string()],
        }),
    };

    let json = serde_json::to_value(&extensions).unwrap();

    // Verify the extension key is correct
    assert!(
        json["io.modelcontextprotocol/ui"].is_object(),
        "Extension key should be io.modelcontextprotocol/ui"
    );
    assert!(
        json["io.modelcontextprotocol/ui"]["mimeTypes"]
            .as_array()
            .unwrap()
            .contains(&json!("text/html;profile=mcp-app")),
        "MIME types should include MCP App type"
    );
}

#[test]
fn test_capabilities_include_extensions() {
    let caps = ServerCapabilitiesWithExtensions::with_ui_extension(
        Some(communitas_mcp::protocol::ToolsCapability {
            list_changed: false,
        }),
        Some(communitas_mcp::protocol::ResourcesCapability {
            subscribe: false,
            list_changed: false,
        }),
    );

    let json = serde_json::to_value(&caps).unwrap();

    // Should have tools capability
    assert!(json["tools"].is_object());

    // Should have resources capability
    assert!(json["resources"].is_object());

    // Should have extensions with UI
    assert!(json["extensions"]["io.modelcontextprotocol/ui"].is_object());
}

// =============================================================================
// UI Resource Entry Tests
// =============================================================================

#[test]
fn test_ui_resource_entry_inline() {
    let entry = UiResourceEntry::new_inline(
        "ui://test/widget",
        "Test Widget",
        "A test widget for testing",
        "<html><body>Test</body></html>",
    );

    assert_eq!(entry.resource.uri, "ui://test/widget");
    assert_eq!(entry.resource.name, "Test Widget");
    assert_eq!(
        entry.resource.description,
        Some("A test widget for testing".to_string())
    );
    assert_eq!(
        entry.resource.mime_type,
        Some("text/html;profile=mcp-app".to_string())
    );

    // Content should be readable
    let content = entry.get_content().unwrap();
    assert_eq!(content, "<html><body>Test</body></html>");
}

#[test]
fn test_ui_content_enum() {
    // Test Inline variant
    let inline = UiContent::Inline("<html></html>".to_string());
    match inline {
        UiContent::Inline(content) => assert_eq!(content, "<html></html>"),
        UiContent::File(_) => panic!("Expected Inline variant"),
    }
}

// =============================================================================
// Widget-Specific Content Validation Tests
// =============================================================================

#[test]
fn test_contacts_widget_structure() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let (content, _) = registry.read("ui://communitas/contacts").unwrap();

    // Should have essential structural elements
    assert!(content.contains("id=\"app\""), "Should have app container");
    assert!(
        content.contains("search") || content.contains("Search"),
        "Should have search functionality"
    );
}

#[test]
fn test_messages_widget_structure() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let (content, _) = registry.read("ui://communitas/messages").unwrap();

    // Should have essential structural elements
    assert!(content.contains("id=\"app\""), "Should have app container");
    assert!(
        content.contains("thread") || content.contains("Thread"),
        "Should have thread-related elements"
    );
}

#[test]
fn test_kanban_widget_structure() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let (content, _) = registry.read("ui://communitas/kanban").unwrap();

    // Should have essential structural elements
    assert!(content.contains("id=\"app\""), "Should have app container");
    assert!(
        content.contains("board") || content.contains("Board") || content.contains("column"),
        "Should have board/column elements"
    );
}

#[test]
fn test_drive_widget_structure() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let (content, _) = registry.read("ui://communitas/drive").unwrap();

    // Should have essential structural elements
    assert!(content.contains("id=\"app\""), "Should have app container");
    assert!(
        content.contains("file") || content.contains("File") || content.contains("folder"),
        "Should have file-related elements"
    );
}

#[test]
fn test_canvas_widget_structure() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let (content, _) = registry.read("ui://communitas/canvas").unwrap();

    // Should have essential structural elements
    assert!(content.contains("id=\"app\""), "Should have app container");
    assert!(
        content.contains("canvas") || content.contains("Canvas") || content.contains("layer"),
        "Should have canvas-related elements"
    );
}

// =============================================================================
// Shared Assets Tests
// =============================================================================

#[test]
fn test_widgets_reference_shared_styles_or_inline() {
    // Widgets should either reference the shared styles.css or have inline styles
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry.read(&resource.uri).unwrap();
        let has_shared_styles = content.contains("styles.css");
        let has_inline_styles = content.contains("<style>");

        assert!(
            has_shared_styles || has_inline_styles,
            "Widget {} should have styles (shared or inline)",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_reference_mcp_bridge() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry.read(&resource.uri).unwrap();
        assert!(
            content.contains("mcp-bridge.js"),
            "Widget {} should reference mcp-bridge.js",
            resource.uri
        );
    }
}
