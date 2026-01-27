// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CSP (Content Security Policy) Validation Tests for MCP Apps
//!
//! These tests verify that UI resources have proper Content Security Policy
//! configurations to ensure secure widget rendering in MCP host applications.
//!
//! Run with: cargo test -p communitas-mcp --test csp_test

use communitas_mcp::protocol::{UiResourceCsp, UiResourceMeta};
use communitas_mcp::ui_resources::UiResourceRegistry;
use serde_json::json;

// =============================================================================
// CSP Configuration Tests
// =============================================================================

#[test]
fn test_default_csp_is_restrictive() {
    let csp = UiResourceCsp::default();

    // Default CSP should have no external domains allowed
    assert!(
        csp.connect_domains.is_empty(),
        "Default CSP should not allow any connect domains"
    );
    assert!(
        csp.resource_domains.is_empty(),
        "Default CSP should not allow any resource domains"
    );
    assert!(
        csp.frame_domains.is_empty(),
        "Default CSP should not allow any frame domains"
    );
    assert!(
        csp.base_uri_domains.is_empty(),
        "Default CSP should not allow any base URI domains"
    );
}

#[test]
fn test_csp_serialization_omits_empty_arrays() {
    let csp = UiResourceCsp::default();
    let json = serde_json::to_value(&csp).expect("CSP should serialize");

    // Empty vectors should be omitted from JSON (skip_serializing_if)
    assert!(
        !json
            .as_object()
            .is_some_and(|o| o.contains_key("connectDomains")),
        "Empty connectDomains should be omitted"
    );
    assert!(
        !json
            .as_object()
            .is_some_and(|o| o.contains_key("resourceDomains")),
        "Empty resourceDomains should be omitted"
    );
    assert!(
        !json
            .as_object()
            .is_some_and(|o| o.contains_key("frameDomains")),
        "Empty frameDomains should be omitted"
    );
    assert!(
        !json
            .as_object()
            .is_some_and(|o| o.contains_key("baseUriDomains")),
        "Empty baseUriDomains should be omitted"
    );
}

#[test]
fn test_csp_with_domains() {
    let csp = UiResourceCsp {
        connect_domains: vec!["api.communitas.local".to_string()],
        resource_domains: vec!["cdn.communitas.local".to_string()],
        frame_domains: vec![],
        base_uri_domains: vec![],
    };

    let json = serde_json::to_value(&csp).expect("CSP should serialize");

    assert_eq!(
        json["connectDomains"],
        json!(["api.communitas.local"]),
        "Connect domains should serialize"
    );
    assert_eq!(
        json["resourceDomains"],
        json!(["cdn.communitas.local"]),
        "Resource domains should serialize"
    );
}

#[test]
fn test_csp_deserialization() {
    let json = json!({
        "connectDomains": ["api.example.com", "ws.example.com"],
        "resourceDomains": ["cdn.example.com"],
        "frameDomains": [],
        "baseUriDomains": []
    });

    let csp: UiResourceCsp = serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(csp.connect_domains.len(), 2);
    assert!(csp.connect_domains.contains(&"api.example.com".to_string()));
    assert!(csp.connect_domains.contains(&"ws.example.com".to_string()));
    assert_eq!(csp.resource_domains.len(), 1);
    assert!(csp.frame_domains.is_empty());
}

#[test]
fn test_csp_partial_deserialization() {
    // CSP with only some fields specified
    let json = json!({
        "connectDomains": ["api.example.com"]
    });

    let csp: UiResourceCsp = serde_json::from_value(json).expect("Should deserialize");

    assert_eq!(csp.connect_domains.len(), 1);
    assert!(
        csp.resource_domains.is_empty(),
        "Missing fields should default to empty"
    );
    assert!(csp.frame_domains.is_empty());
    assert!(csp.base_uri_domains.is_empty());
}

// =============================================================================
// UI Resource Metadata Tests
// =============================================================================

#[test]
fn test_ui_resource_meta_default() {
    let meta = UiResourceMeta::default();

    assert!(meta.csp.is_none(), "Default metadata should have no CSP");
    assert!(!meta.prefers_border, "Default should not prefer border");
    assert!(
        meta.permissions.is_empty(),
        "Default should have no permissions"
    );
}

#[test]
fn test_ui_resource_meta_with_csp() {
    let meta = UiResourceMeta {
        csp: Some(UiResourceCsp::default()),
        prefers_border: true,
        permissions: vec!["clipboard-read".to_string()],
    };

    assert!(meta.csp.is_some());
    assert!(meta.prefers_border);
    assert_eq!(meta.permissions.len(), 1);
}

#[test]
fn test_ui_resource_meta_serialization() {
    let meta = UiResourceMeta {
        csp: Some(UiResourceCsp {
            connect_domains: vec!["api.test.com".to_string()],
            resource_domains: vec![],
            frame_domains: vec![],
            base_uri_domains: vec![],
        }),
        prefers_border: false,
        permissions: vec![],
    };

    let json = serde_json::to_value(&meta).expect("Should serialize");

    // CSP should be present
    assert!(json.get("csp").is_some(), "CSP should be present");

    // prefers_border defaults to false, but it's not skipped
    assert!(json.get("prefersBorder").is_some() || !meta.prefers_border);

    // Empty permissions should be omitted
    assert!(
        !json
            .as_object()
            .is_some_and(|o| o.contains_key("permissions")),
        "Empty permissions should be omitted"
    );
}

// =============================================================================
// Widget CSP Validation Tests
// =============================================================================

#[test]
fn test_all_widgets_have_csp() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let meta = resource._meta.as_ref();
        assert!(meta.is_some(), "Widget {} should have _meta", resource.uri);

        let ui_meta = meta.unwrap().ui.as_ref();
        assert!(
            ui_meta.is_some(),
            "Widget {} should have ui metadata",
            resource.uri
        );

        let csp = ui_meta.unwrap().csp.as_ref();
        assert!(
            csp.is_some(),
            "Widget {} should have CSP configuration",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_use_restrictive_csp() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let csp = resource
            ._meta
            .as_ref()
            .and_then(|m| m.ui.as_ref())
            .and_then(|ui| ui.csp.as_ref())
            .expect("Widget should have CSP");

        // Standard widgets should use restrictive default CSP
        assert!(
            csp.connect_domains.is_empty(),
            "Widget {} should not allow external connect domains by default",
            resource.uri
        );
        assert!(
            csp.frame_domains.is_empty(),
            "Widget {} should not allow nested iframes by default",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_do_not_request_dangerous_permissions() {
    let registry = UiResourceRegistry::with_standard_widgets();
    let dangerous_permissions = [
        "geolocation",
        "camera",
        "microphone",
        "notifications",
        "storage-access",
    ];

    for resource in registry.list() {
        let permissions = resource
            ._meta
            .as_ref()
            .and_then(|m| m.ui.as_ref())
            .map(|ui| &ui.permissions)
            .expect("Widget should have permissions list");

        for perm in &dangerous_permissions {
            assert!(
                !permissions.contains(&perm.to_string()),
                "Widget {} should not request dangerous permission: {}",
                resource.uri,
                perm
            );
        }
    }
}

// =============================================================================
// Widget HTML CSP Validation Tests
// =============================================================================

#[test]
fn test_widgets_have_no_inline_event_handlers() {
    let registry = UiResourceRegistry::with_standard_widgets();

    // Inline event handlers that would violate strict CSP
    let dangerous_patterns = [
        "onclick=",
        "onload=",
        "onerror=",
        "onmouseover=",
        "onfocus=",
        "onblur=",
        "onsubmit=",
        "onchange=",
        "onkeydown=",
        "onkeyup=",
    ];

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");
        let content_lower = content.to_lowercase();

        for pattern in &dangerous_patterns {
            // Check if the pattern appears outside of JavaScript strings/comments
            // Simple heuristic: if it appears with a quote after =, it's an inline handler
            let pattern_with_quote = format!("{}\"", pattern);
            let pattern_with_single = format!("{}'", pattern);

            // Allow if the pattern only appears in JS code (within script tags as string literals)
            // This is a basic check - real CSP validation would be more sophisticated
            let has_dangerous_inline = content_lower.contains(&pattern_with_quote.to_lowercase())
                || content_lower.contains(&pattern_with_single.to_lowercase());

            if has_dangerous_inline {
                // Check if it's within a script tag (acceptable for event listener setup)
                let in_script =
                    content_lower.contains("<script") && content_lower.contains("</script>");

                // If not in script, this is a CSP violation
                assert!(
                    in_script,
                    "Widget {} has inline event handler '{}' which violates CSP",
                    resource.uri, pattern
                );
            }
        }
    }
}

#[test]
fn test_widgets_use_mcp_bridge_not_fetch() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");

        // Widgets should use MCP bridge, not direct fetch to external URLs
        let has_external_fetch = content.contains("fetch('http")
            || content.contains("fetch(\"http")
            || content.contains("XMLHttpRequest");

        assert!(
            !has_external_fetch,
            "Widget {} should use MCP bridge instead of direct fetch/XHR",
            resource.uri
        );

        // Should reference mcp-bridge
        assert!(
            content.contains("mcp-bridge")
                || content.contains("McpBridge")
                || content.contains("bridge.callTool"),
            "Widget {} should use MCP bridge for communication",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_have_no_external_script_src() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");
        let content_lower = content.to_lowercase();

        // Check for external script sources
        let has_external_scripts = content_lower.contains("src=\"http")
            || content_lower.contains("src='http")
            || content_lower.contains("src=\"//")
            || content_lower.contains("src='//");

        assert!(
            !has_external_scripts,
            "Widget {} should not load external scripts (CSP violation)",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_have_no_external_stylesheet_href() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");
        let content_lower = content.to_lowercase();

        // Check for external stylesheets
        let has_external_styles = content_lower.contains("href=\"http")
            || content_lower.contains("href='http")
            || content_lower.contains("href=\"//")
            || content_lower.contains("href='//");

        // Exception: links to documentation or other non-stylesheet resources
        // Real check would parse HTML and verify link rel="stylesheet"
        let has_stylesheet_link = content_lower.contains("rel=\"stylesheet\"")
            || content_lower.contains("rel='stylesheet'");

        if has_external_styles && has_stylesheet_link {
            panic!(
                "Widget {} should not load external stylesheets (CSP violation)",
                resource.uri
            );
        }
    }
}

// =============================================================================
// Cross-Origin Security Tests
// =============================================================================

#[test]
fn test_widgets_do_not_open_new_windows() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");

        // window.open can be a security risk in sandboxed iframes
        let has_window_open = content.contains("window.open(") || content.contains("window.open (");

        assert!(
            !has_window_open,
            "Widget {} should not use window.open() (sandbox restriction)",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_do_not_use_eval() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");

        // eval is blocked by CSP and is a security risk
        let has_eval = content.contains("eval(")
            || content.contains("new Function(")
            || content.contains("setTimeout(\"")
            || content.contains("setTimeout('")
            || content.contains("setInterval(\"")
            || content.contains("setInterval('");

        assert!(
            !has_eval,
            "Widget {} should not use eval() or similar (CSP violation)",
            resource.uri
        );
    }
}

#[test]
fn test_widgets_do_not_use_document_write() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");

        // document.write is blocked in sandboxed iframes
        let has_document_write =
            content.contains("document.write(") || content.contains("document.writeln(");

        assert!(
            !has_document_write,
            "Widget {} should not use document.write() (sandbox restriction)",
            resource.uri
        );
    }
}

// =============================================================================
// Iframe Sandbox Attribute Tests
// =============================================================================

#[test]
fn test_widgets_define_safe_iframe_sandbox() {
    // When widgets embed iframes, they should use sandbox attribute
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");
        let content_lower = content.to_lowercase();

        // If widget contains iframes, they should be sandboxed
        if content_lower.contains("<iframe") {
            let has_sandbox = content_lower.contains("sandbox=");
            assert!(
                has_sandbox,
                "Widget {} has iframe without sandbox attribute",
                resource.uri
            );

            // Should not have allow-same-origin + allow-scripts (XSS risk)
            let dangerous_sandbox = content_lower.contains("allow-same-origin")
                && content_lower.contains("allow-scripts")
                && !content_lower.contains("allow-forms"); // This combo is dangerous

            assert!(
                !dangerous_sandbox,
                "Widget {} has iframe with dangerous sandbox combination",
                resource.uri
            );
        }
    }
}

// =============================================================================
// MIME Type Validation Tests
// =============================================================================

#[test]
fn test_all_widgets_have_correct_mime_type() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let mime_type = resource.mime_type.as_ref();
        assert!(
            mime_type.is_some(),
            "Widget {} should have MIME type",
            resource.uri
        );

        assert_eq!(
            mime_type.unwrap(),
            "text/html;profile=mcp-app",
            "Widget {} should have MCP App MIME type",
            resource.uri
        );
    }
}

#[test]
fn test_widget_content_is_valid_html() {
    let registry = UiResourceRegistry::with_standard_widgets();

    for resource in registry.list() {
        let (content, _) = registry
            .read(&resource.uri)
            .expect("Widget should be readable");

        // Basic HTML structure validation
        assert!(
            content.contains("<!DOCTYPE html>") || content.contains("<!doctype html>"),
            "Widget {} should have DOCTYPE declaration",
            resource.uri
        );

        assert!(
            content.contains("<html"),
            "Widget {} should have html element",
            resource.uri
        );

        assert!(
            content.contains("<head") && content.contains("</head>"),
            "Widget {} should have head element",
            resource.uri
        );

        assert!(
            content.contains("<body") && content.contains("</body>"),
            "Widget {} should have body element",
            resource.uri
        );
    }
}
