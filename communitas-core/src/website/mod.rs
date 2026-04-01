// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Website Storage and Markdown Management
//!
//! This module provides functionality for:
//! - Storing and managing markdown-based websites
//! - Collaborative editing with CRDT
//! - Publishing/unpublishing websites via 4-word addresses
//! - Markdown rendering with sanitization
//! - Page management (create, read, update, delete)

mod manager;
mod markdown;
mod types;

pub use manager::WebsiteManager;
pub use markdown::{MarkdownRenderer, render_markdown, sanitize_html};
pub use types::{MarkdownPage, WebsiteError, WebsiteMetadata, WebsiteResult};
