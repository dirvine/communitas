// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Communitas MCP Server Library
//!
//! This module exports the internal types needed for testing and external integration.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

pub mod auth;
pub mod presence;
pub mod protocol;
pub mod token;
pub mod tools;
pub mod ui_resources;

// Re-export MCP Apps types for external use
pub use protocol::{
    // MCP Apps Extension Types (SEP-1865)
    InitializeResultWithExtensions, McpUiToolMeta, ResourceMeta, ResourceWithMeta,
    ServerCapabilitiesWithExtensions, ServerExtensions, ToolCallResultWithMeta,
    ToolDefinitionMeta, ToolResultMeta, ToolWithMeta, UiExtensionCapability, UiResourceCsp,
    UiResourceMeta,
};

// Re-export UI resource registry
pub use ui_resources::{SharedUiResourceRegistry, UiContent, UiResourceEntry, UiResourceRegistry};
