// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use serde::{Deserialize, Serialize};

/// A single markdown page within a website
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkdownPage {
    /// Path to the page (e.g., "home.md", "blog/post1.md")
    pub path: String,

    /// Markdown content
    pub content: String,

    /// Optional page title (extracted from metadata or first heading)
    pub title: Option<String>,

    /// Creation timestamp
    pub created_at: i64,

    /// Last update timestamp
    pub updated_at: i64,
}

/// Website metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WebsiteMetadata {
    /// Four-word address of the website owner
    pub four_word_address: String,

    /// Website title
    pub title: String,

    /// Optional description
    pub description: Option<String>,

    /// Home page path (default: "home.md")
    pub home_page: String,

    /// Whether the website is published and accessible
    pub published: bool,

    /// When the website was published (if published)
    pub published_at: Option<i64>,

    /// Creation timestamp
    pub created_at: i64,

    /// Last update timestamp
    pub updated_at: i64,
}

impl Default for WebsiteMetadata {
    fn default() -> Self {
        Self {
            four_word_address: String::new(),
            title: String::new(),
            description: None,
            home_page: "home.md".to_string(),
            published: false,
            published_at: None,
            created_at: 0,
            updated_at: 0,
        }
    }
}

/// Website-related errors
#[derive(Debug, thiserror::Error)]
pub enum WebsiteError {
    #[error("Page not found: {0}")]
    PageNotFound(String),

    #[error("Website not found: {0}")]
    WebsiteNotFound(String),

    #[error("Website not published: {0}")]
    NotPublished(String),

    #[error("CRDT error: {0}")]
    Crdt(#[from] crate::crdt_manager::CrdtError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Markdown rendering error: {0}")]
    Rendering(String),
}

/// Result type for website operations
pub type WebsiteResult<T> = Result<T, WebsiteError>;
