// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Website Manager - Handles website storage and operations

use super::markdown::{MarkdownRenderer, render_and_sanitize};
use super::types::{MarkdownPage, WebsiteError, WebsiteMetadata, WebsiteResult};
use crate::crdt_manager::CrdtManager;
use std::sync::Arc;
use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, Map, ReadTxn, Text, Transact, WriteTxn};

/// Manager for website storage and operations
pub struct WebsiteManager {
    crdt_manager: Arc<CrdtManager>,
    renderer: MarkdownRenderer,
}

impl WebsiteManager {
    /// Create a new website manager
    pub fn new(crdt_manager: Arc<CrdtManager>) -> Self {
        Self {
            crdt_manager,
            renderer: MarkdownRenderer::new(),
        }
    }

    /// Get the document ID for a website page
    fn page_doc_id(four_word_address: &str, path: &str) -> String {
        // Sanitize path to prevent directory traversal
        let safe_path = path.replace("..", "").replace("//", "/");
        format!("website:{}:page:{}", four_word_address, safe_path)
    }

    /// Get the document ID for website metadata
    fn metadata_doc_id(four_word_address: &str) -> String {
        format!("website:{}:metadata", four_word_address)
    }

    /// Save a markdown page
    pub async fn save_page(
        &self,
        four_word_address: &str,
        page: &MarkdownPage,
    ) -> WebsiteResult<()> {
        let doc_id = Self::page_doc_id(four_word_address, &page.path);
        let doc = Doc::new();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");

            // Store content as Text for collaborative editing
            root.insert(&mut txn, "content", yrs::TextPrelim::new(&page.content));

            // Store metadata
            CrdtManager::set_map_string(&root, &mut txn, "path", &page.path);
            if let Some(ref title) = page.title {
                CrdtManager::set_map_string(&root, &mut txn, "title", title);
            }
            CrdtManager::set_map_i64(&root, &mut txn, "created_at", page.created_at);
            CrdtManager::set_map_i64(&root, &mut txn, "updated_at", page.updated_at);
        }

        self.crdt_manager
            .save_document(&doc_id, "website", four_word_address, &doc)
            .await?;

        Ok(())
    }

    /// Load a markdown page
    pub async fn load_page(
        &self,
        four_word_address: &str,
        path: &str,
    ) -> WebsiteResult<MarkdownPage> {
        let doc_id = Self::page_doc_id(four_word_address, path);
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| WebsiteError::PageNotFound(path.to_string()))?;

        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        // Extract content from Text
        let content = if let Some(text_val) = root.get(&txn, "content") {
            if let Ok(text_ref) = yrs::TextRef::try_from(text_val) {
                text_ref.get_string(&txn)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let path =
            CrdtManager::get_map_string(&root, &txn, "path").unwrap_or_else(|| path.to_string());
        let title = CrdtManager::get_map_string(&root, &txn, "title");
        let created_at = CrdtManager::get_map_i64(&root, &txn, "created_at").unwrap_or(0);
        let updated_at = CrdtManager::get_map_i64(&root, &txn, "updated_at").unwrap_or(0);

        Ok(MarkdownPage {
            path,
            content,
            title,
            created_at,
            updated_at,
        })
    }

    /// Load a page as a Yrs document for collaborative editing
    pub async fn load_page_doc(&self, four_word_address: &str, path: &str) -> WebsiteResult<Doc> {
        let doc_id = Self::page_doc_id(four_word_address, path);
        self.crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| WebsiteError::PageNotFound(path.to_string()))
    }

    /// Append text to a page document (collaborative editing)
    pub fn append_text(&self, doc: &Doc, text: &str) -> WebsiteResult<()> {
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();

        if let Some(content_val) = root.get(&txn, "content") {
            if let Ok(content_text) = yrs::TextRef::try_from(content_val) {
                let len = content_text.len(&txn);
                content_text.insert(&mut txn, len, text);
                return Ok(());
            }
        }

        Err(WebsiteError::Rendering("No content text found".to_string()))
    }

    /// Insert text at a specific position (collaborative editing)
    pub fn insert_text_at(&self, doc: &Doc, index: u32, text: &str) -> WebsiteResult<()> {
        let root = doc.get_or_insert_map("root");
        let mut txn = doc.transact_mut();

        if let Some(content_val) = root.get(&txn, "content") {
            if let Ok(content_text) = yrs::TextRef::try_from(content_val) {
                content_text.insert(&mut txn, index, text);
                return Ok(());
            }
        }

        Err(WebsiteError::Rendering("No content text found".to_string()))
    }

    /// Extract content from a document
    pub fn extract_content(&self, doc: &Doc) -> WebsiteResult<String> {
        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        if let Some(content_val) = root.get(&txn, "content") {
            if let Ok(content_text) = yrs::TextRef::try_from(content_val) {
                return Ok(content_text.get_string(&txn));
            }
        }

        Err(WebsiteError::Rendering("No content found".to_string()))
    }

    /// Merge multiple page documents (collaborative editing)
    pub async fn merge_page_docs(&self, mut docs: Vec<Doc>) -> WebsiteResult<Doc> {
        if docs.is_empty() {
            return Err(WebsiteError::Rendering("No documents to merge".to_string()));
        }

        // Take the first document as base
        let base = docs.remove(0);

        // Apply updates from all other documents to the base
        for doc in docs {
            let update_bytes = doc
                .transact()
                .encode_state_as_update_v1(&yrs::StateVector::default());

            let update = yrs::Update::decode_v1(&update_bytes)
                .map_err(|e| WebsiteError::Rendering(format!("Failed to decode update: {}", e)))?;

            let mut txn = base.transact_mut();
            txn.apply_update(update);
        }

        Ok(base)
    }

    /// List all pages for a website
    pub async fn list_pages(&self, four_word_address: &str) -> WebsiteResult<Vec<String>> {
        let pages = self.crdt_manager.list_documents("website").await?;

        // Filter pages that belong to this address
        let prefix = format!("website:{}:page:", four_word_address);
        let mut result = Vec::new();

        for doc_id in pages {
            if doc_id.starts_with(&prefix) {
                // Extract path from doc_id
                if let Some(path) = doc_id.strip_prefix(&prefix) {
                    result.push(path.to_string());
                }
            }
        }

        Ok(result)
    }

    /// Delete a page
    pub async fn delete_page(&self, four_word_address: &str, path: &str) -> WebsiteResult<()> {
        let doc_id = Self::page_doc_id(four_word_address, path);
        self.crdt_manager.delete_document(&doc_id).await?;
        Ok(())
    }

    /// Save website metadata
    pub async fn save_metadata(
        &self,
        four_word_address: &str,
        metadata: &WebsiteMetadata,
    ) -> WebsiteResult<()> {
        let doc_id = Self::metadata_doc_id(four_word_address);
        let doc = Doc::new();

        {
            let mut txn = doc.transact_mut();
            let root = txn.get_or_insert_map("root");

            CrdtManager::set_map_string(
                &root,
                &mut txn,
                "four_word_address",
                &metadata.four_word_address,
            );
            CrdtManager::set_map_string(&root, &mut txn, "title", &metadata.title);

            if let Some(ref desc) = metadata.description {
                CrdtManager::set_map_string(&root, &mut txn, "description", desc);
            }

            CrdtManager::set_map_string(&root, &mut txn, "home_page", &metadata.home_page);
            CrdtManager::set_map_bool(&root, &mut txn, "published", metadata.published);

            if let Some(published_at) = metadata.published_at {
                CrdtManager::set_map_i64(&root, &mut txn, "published_at", published_at);
            }

            CrdtManager::set_map_i64(&root, &mut txn, "created_at", metadata.created_at);
            CrdtManager::set_map_i64(&root, &mut txn, "updated_at", metadata.updated_at);
        }

        self.crdt_manager
            .save_document(&doc_id, "website", four_word_address, &doc)
            .await?;

        Ok(())
    }

    /// Get website metadata
    pub async fn get_metadata(&self, four_word_address: &str) -> WebsiteResult<WebsiteMetadata> {
        let doc_id = Self::metadata_doc_id(four_word_address);
        let doc = self
            .crdt_manager
            .load_document(&doc_id)
            .await
            .map_err(|_| WebsiteError::WebsiteNotFound(four_word_address.to_string()))?;

        let root = doc.get_or_insert_map("root");
        let txn = doc.transact();

        let four_word_address = CrdtManager::get_map_string(&root, &txn, "four_word_address")
            .unwrap_or_else(|| four_word_address.to_string());
        let title = CrdtManager::get_map_string(&root, &txn, "title").unwrap_or_default();
        let description = CrdtManager::get_map_string(&root, &txn, "description");
        let home_page = CrdtManager::get_map_string(&root, &txn, "home_page")
            .unwrap_or_else(|| "home.md".to_string());
        let published = CrdtManager::get_map_bool(&root, &txn, "published").unwrap_or(false);
        let published_at = CrdtManager::get_map_i64(&root, &txn, "published_at");
        let created_at = CrdtManager::get_map_i64(&root, &txn, "created_at").unwrap_or(0);
        let updated_at = CrdtManager::get_map_i64(&root, &txn, "updated_at").unwrap_or(0);

        Ok(WebsiteMetadata {
            four_word_address,
            title,
            description,
            home_page,
            published,
            published_at,
            created_at,
            updated_at,
        })
    }

    /// Publish a website
    pub async fn publish(&self, four_word_address: &str, _publisher_id: &str) -> WebsiteResult<()> {
        let mut metadata = self
            .get_metadata(four_word_address)
            .await
            .unwrap_or_else(|_| WebsiteMetadata {
                four_word_address: four_word_address.to_string(),
                title: four_word_address.to_string(),
                ..Default::default()
            });

        metadata.published = true;
        metadata.published_at = Some(chrono::Utc::now().timestamp());
        metadata.updated_at = chrono::Utc::now().timestamp();

        self.save_metadata(four_word_address, &metadata).await?;
        Ok(())
    }

    /// Unpublish a website
    pub async fn unpublish(&self, four_word_address: &str) -> WebsiteResult<()> {
        let mut metadata = self.get_metadata(four_word_address).await?;

        metadata.published = false;
        metadata.updated_at = chrono::Utc::now().timestamp();

        self.save_metadata(four_word_address, &metadata).await?;
        Ok(())
    }

    /// Check if a website is published
    pub async fn is_published(&self, four_word_address: &str) -> WebsiteResult<bool> {
        match self.get_metadata(four_word_address).await {
            Ok(metadata) => Ok(metadata.published),
            Err(_) => Ok(false),
        }
    }

    /// Resolve a 4-word address to a page
    pub async fn resolve_address(
        &self,
        four_word_address: &str,
        path: &str,
    ) -> WebsiteResult<MarkdownPage> {
        self.load_page(four_word_address, path).await
    }

    /// Render a page to HTML
    pub async fn render_to_html(
        &self,
        four_word_address: &str,
        path: &str,
    ) -> WebsiteResult<String> {
        let page = self.load_page(four_word_address, path).await?;
        Ok(render_and_sanitize(&page.content))
    }
}
