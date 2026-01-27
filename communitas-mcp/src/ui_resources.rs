// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! UI Resource Registry for MCP Apps Extension (SEP-1865)
//!
//! This module implements the registry for UI resources that can be rendered
//! in MCP host applications (Claude Desktop, ChatGPT, VS Code, etc.).
//!
//! UI resources are served via the `ui://` URI scheme and contain HTML bundles
//! that communicate with the MCP server via postMessage JSON-RPC.

use crate::protocol::{ResourceMeta, ResourceWithMeta, UiResourceCsp, UiResourceMeta};
use std::collections::HashMap;
use std::sync::Arc;

/// Content source for UI resources
#[derive(Debug, Clone)]
pub enum UiContent {
    /// Embedded HTML content (compiled into binary via include_str!)
    Inline(String),
    /// Reference to an external HTML file (for development)
    #[allow(dead_code)] // Will be used for dev mode in Phase 4
    File(std::path::PathBuf),
}

/// Entry in the UI resource registry
#[derive(Debug, Clone)]
pub struct UiResourceEntry {
    /// Resource metadata (name, description, MIME type)
    pub resource: ResourceWithMeta,
    /// The actual HTML content
    pub content: UiContent,
}

impl UiResourceEntry {
    /// Create a new UI resource entry with inline content
    pub fn new_inline(
        uri: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        html: impl Into<String>,
    ) -> Self {
        let uri = uri.into();
        Self {
            resource: ResourceWithMeta {
                uri: uri.clone(),
                name: name.into(),
                description: Some(description.into()),
                mime_type: Some("text/html;profile=mcp-app".to_string()),
                _meta: Some(ResourceMeta {
                    ui: Some(UiResourceMeta {
                        csp: Some(UiResourceCsp::default()),
                        prefers_border: false,
                        permissions: vec![],
                    }),
                }),
            },
            content: UiContent::Inline(html.into()),
        }
    }

    /// Get the HTML content of this resource
    pub fn get_content(&self) -> std::io::Result<String> {
        match &self.content {
            UiContent::Inline(html) => Ok(html.clone()),
            UiContent::File(path) => std::fs::read_to_string(path),
        }
    }
}

/// Registry of UI resources for MCP Apps
#[derive(Debug, Clone, Default)]
pub struct UiResourceRegistry {
    /// Map of URI -> UiResourceEntry
    resources: HashMap<String, UiResourceEntry>,
}

impl UiResourceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Create a registry with standard Communitas UI resources
    pub fn with_standard_widgets() -> Self {
        let mut registry = Self::new();
        registry.register_standard_widgets();
        registry
    }

    /// Register a UI resource
    pub fn register(&mut self, entry: UiResourceEntry) {
        self.resources.insert(entry.resource.uri.clone(), entry);
    }

    /// List all registered UI resources
    pub fn list(&self) -> Vec<ResourceWithMeta> {
        self.resources
            .values()
            .map(|entry| entry.resource.clone())
            .collect()
    }

    /// Read a UI resource by URI
    pub fn read(&self, uri: &str) -> Option<(String, String)> {
        self.resources.get(uri).and_then(|entry| {
            entry.get_content().ok().map(|content| {
                let mime_type = entry
                    .resource
                    .mime_type
                    .clone()
                    .unwrap_or_else(|| "text/html".to_string());
                (content, mime_type)
            })
        })
    }

    /// Check if a URI is a UI resource
    #[allow(dead_code)] // Used by http.rs in Phase 1.2
    pub fn is_ui_resource(&self, uri: &str) -> bool {
        uri.starts_with("ui://")
    }

    /// Get a resource entry by URI
    #[allow(dead_code)] // Used for detailed resource inspection
    pub fn get(&self, uri: &str) -> Option<&UiResourceEntry> {
        self.resources.get(uri)
    }

    /// Register the standard Communitas UI widgets
    fn register_standard_widgets(&mut self) {
        // Contacts widget
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/contacts",
            "Contacts",
            "Interactive contact list with search and favorites",
            include_str!("../ui-bundles/contacts/index.html"),
        ));

        // Messages widget
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/messages",
            "Messages",
            "Thread navigation and message composition",
            include_str!("../ui-bundles/messages/index.html"),
        ));

        // Kanban widget
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/kanban",
            "Kanban",
            "Interactive Kanban board with drag-drop",
            include_str!("../ui-bundles/kanban/index.html"),
        ));

        // Drive widget
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/drive",
            "Drive",
            "File browser with upload and preview",
            include_str!("../ui-bundles/drive/index.html"),
        ));

        // Canvas widget
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/canvas",
            "Canvas",
            "Collaborative whiteboard viewer",
            include_str!("../ui-bundles/canvas/index.html"),
        ));

        // Settings widget (Phase 9.2)
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/settings",
            "Settings",
            "User preferences, theme, and notification settings",
            include_str!("../ui-bundles/settings/index.html"),
        ));

        // Search widget (Phase 9.2)
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/search",
            "Search",
            "Global search across contacts, messages, files, and more",
            include_str!("../ui-bundles/search/index.html"),
        ));

        // Notifications widget (Phase 9.2)
        self.register(UiResourceEntry::new_inline(
            "ui://communitas/notifications",
            "Notifications",
            "Notification center with filters and mark-as-read",
            include_str!("../ui-bundles/notifications/index.html"),
        ));
    }
}

/// Thread-safe wrapper for the UI resource registry
#[allow(dead_code)] // Available for multi-threaded scenarios
pub type SharedUiResourceRegistry = Arc<UiResourceRegistry>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_registry() {
        let registry = UiResourceRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_register_and_list() {
        let mut registry = UiResourceRegistry::new();
        registry.register(UiResourceEntry::new_inline(
            "ui://test/widget",
            "Test Widget",
            "A test widget",
            "<html><body>Hello</body></html>",
        ));

        let resources = registry.list();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "ui://test/widget");
        assert_eq!(resources[0].name, "Test Widget");
    }

    #[test]
    fn test_read_resource() {
        let mut registry = UiResourceRegistry::new();
        let html = "<html><body>Test Content</body></html>";
        registry.register(UiResourceEntry::new_inline(
            "ui://test/widget",
            "Test Widget",
            "A test widget",
            html,
        ));

        let (content, mime_type) = registry.read("ui://test/widget").unwrap();
        assert_eq!(content, html);
        assert_eq!(mime_type, "text/html;profile=mcp-app");
    }

    #[test]
    fn test_read_nonexistent_resource() {
        let registry = UiResourceRegistry::new();
        assert!(registry.read("ui://nonexistent").is_none());
    }

    #[test]
    fn test_is_ui_resource() {
        let registry = UiResourceRegistry::new();
        assert!(registry.is_ui_resource("ui://communitas/contacts"));
        assert!(registry.is_ui_resource("ui://test/anything"));
        assert!(!registry.is_ui_resource("file:///path/to/file"));
        assert!(!registry.is_ui_resource("https://example.com"));
    }

    #[test]
    fn test_resource_metadata() {
        let mut registry = UiResourceRegistry::new();
        registry.register(UiResourceEntry::new_inline(
            "ui://test/widget",
            "Test Widget",
            "A test widget",
            "<html></html>",
        ));

        let entry = registry.get("ui://test/widget").unwrap();
        assert!(entry.resource._meta.is_some());

        let meta = entry.resource._meta.as_ref().unwrap();
        assert!(meta.ui.is_some());

        let ui_meta = meta.ui.as_ref().unwrap();
        assert!(ui_meta.csp.is_some());
        assert!(!ui_meta.prefers_border);
    }

    #[test]
    fn test_standard_widgets_registered() {
        let registry = UiResourceRegistry::with_standard_widgets();
        let resources = registry.list();

        // Verify all 8 widgets are registered (5 original + 3 new from Phase 9.2)
        assert_eq!(resources.len(), 8);

        // Check original widgets
        assert!(registry.read("ui://communitas/contacts").is_some());
        assert!(registry.read("ui://communitas/messages").is_some());
        assert!(registry.read("ui://communitas/kanban").is_some());
        assert!(registry.read("ui://communitas/drive").is_some());
        assert!(registry.read("ui://communitas/canvas").is_some());

        // Check new Phase 9.2 widgets
        assert!(registry.read("ui://communitas/settings").is_some());
        assert!(registry.read("ui://communitas/search").is_some());
        assert!(registry.read("ui://communitas/notifications").is_some());
    }
}
