// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Website Storage and Markdown Rendering Tests
//!
//! Tests for:
//! - Website creation and storage
//! - Collaborative markdown editing with CRDT
//! - Publishing/unpublishing websites
//! - 4-word address resolution
//! - Markdown rendering and sanitization

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

use communitas_core::crdt_manager::CrdtManager;

use communitas_core::website::{MarkdownPage, WebsiteManager, WebsiteMetadata};
use std::sync::Arc;
use tempfile::tempdir;

/// Test creating and storing a simple markdown page
#[tokio::test]
async fn test_create_markdown_page() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let page = MarkdownPage {
        path: "home.md".to_string(),
        content: "# Welcome\n\nThis is my homepage".to_string(),
        title: Some("Home".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page("alice-dev-coder-pro", &page)
        .await
        .expect("Failed to save page");

    // Load and verify
    let loaded = website_manager
        .load_page("alice-dev-coder-pro", "home.md")
        .await
        .expect("Failed to load page");

    assert_eq!(loaded.path, "home.md");
    assert_eq!(loaded.content, "# Welcome\n\nThis is my homepage");
    assert_eq!(loaded.title, Some("Home".to_string()));
}

/// Test creating a website with multiple pages
#[tokio::test]
async fn test_create_multi_page_website() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "alice-dev-coder-pro";

    // Create home page
    let home = MarkdownPage {
        path: "home.md".to_string(),
        content: "# Welcome to My Site\n\n[About Me](about.md)".to_string(),
        title: Some("Home".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    // Create about page
    let about = MarkdownPage {
        path: "about.md".to_string(),
        content: "# About Me\n\nI'm a developer.".to_string(),
        title: Some("About".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &home)
        .await
        .expect("Save home");
    website_manager
        .save_page(four_words, &about)
        .await
        .expect("Save about");

    // List all pages
    let pages = website_manager
        .list_pages(four_words)
        .await
        .expect("List pages");

    assert_eq!(pages.len(), 2);
    assert!(pages.contains(&"home.md".to_string()));
    assert!(pages.contains(&"about.md".to_string()));
}

/// Test collaborative markdown editing with two peers
#[tokio::test]
async fn test_collaborative_markdown_editing() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "collab-edit-test-site";
    let path = "shared.md";

    // Peer A creates initial content
    let initial = MarkdownPage {
        path: path.to_string(),
        content: "# Collaborative Document\n\n".to_string(),
        title: Some("Shared".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &initial)
        .await
        .expect("Save initial");

    // Peer A and Peer B both load the document
    let doc_a = website_manager
        .load_page_doc(four_words, path)
        .await
        .expect("Load doc A");
    let doc_b = website_manager
        .load_page_doc(four_words, path)
        .await
        .expect("Load doc B");

    // Peer A adds a section
    website_manager
        .append_text(&doc_a, "## Section A\n\nContent from peer A\n\n")
        .expect("Peer A append");

    // Peer B adds a different section
    website_manager
        .append_text(&doc_b, "## Section B\n\nContent from peer B\n\n")
        .expect("Peer B append");

    // Simulate sync: merge both documents
    let merged = website_manager
        .merge_page_docs(vec![doc_a, doc_b])
        .await
        .expect("Merge docs");

    // Both sections should be present
    let content = website_manager
        .extract_content(&merged)
        .expect("Extract content");

    assert!(content.contains("Section A"));
    assert!(content.contains("Content from peer A"));
    assert!(content.contains("Section B"));
    assert!(content.contains("Content from peer B"));
}

/// Test publishing and unpublishing websites
#[tokio::test]
async fn test_publish_unpublish_website() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "publish-test-site-demo";

    // Create a page
    let page = MarkdownPage {
        path: "home.md".to_string(),
        content: "# My Published Site".to_string(),
        title: Some("Home".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &page)
        .await
        .expect("Save page");

    // Initially not published
    assert!(
        !website_manager
            .is_published(four_words)
            .await
            .expect("Check published")
    );

    // Publish the website
    website_manager
        .publish(four_words, "publisher-id")
        .await
        .expect("Publish website");

    // Should now be published
    assert!(
        website_manager
            .is_published(four_words)
            .await
            .expect("Check published")
    );

    // Verify metadata
    let metadata = website_manager
        .get_metadata(four_words)
        .await
        .expect("Get metadata");

    assert!(metadata.published);
    assert_eq!(metadata.four_word_address, four_words);

    // Unpublish
    website_manager
        .unpublish(four_words)
        .await
        .expect("Unpublish");

    // Should no longer be published
    assert!(
        !website_manager
            .is_published(four_words)
            .await
            .expect("Check published")
    );
}

/// Test 4-word address resolution
#[tokio::test]
async fn test_four_word_address_resolution() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    // Create websites for two different addresses
    let alice = "alice-dev-coder-pro";
    let bob = "bob-test-user-demo";

    let alice_page = MarkdownPage {
        path: "home.md".to_string(),
        content: "# Alice's Site".to_string(),
        title: Some("Alice".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    let bob_page = MarkdownPage {
        path: "home.md".to_string(),
        content: "# Bob's Site".to_string(),
        title: Some("Bob".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(alice, &alice_page)
        .await
        .expect("Save Alice's page");
    website_manager
        .save_page(bob, &bob_page)
        .await
        .expect("Save Bob's page");

    // Resolve addresses
    let alice_content = website_manager
        .resolve_address(alice, "home.md")
        .await
        .expect("Resolve Alice");
    let bob_content = website_manager
        .resolve_address(bob, "home.md")
        .await
        .expect("Resolve Bob");

    assert_eq!(alice_content.content, "# Alice's Site");
    assert_eq!(bob_content.content, "# Bob's Site");
}

/// Test markdown sanitization (XSS prevention)
#[tokio::test]
async fn test_markdown_sanitization() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "sanitize-test-site";

    // Try to inject XSS
    let malicious = MarkdownPage {
        path: "malicious.md".to_string(),
        content: r#"# Test
        
<script>alert('XSS')</script>

<img src=x onerror="alert('XSS')">

Normal **markdown** content.
"#
        .to_string(),
        title: Some("Test".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &malicious)
        .await
        .expect("Save page");

    // Render to HTML with sanitization
    let html = website_manager
        .render_to_html(four_words, "malicious.md")
        .await
        .expect("Render HTML");

    // Should NOT contain script tags
    assert!(!html.contains("<script"));
    assert!(!html.contains("onerror"));

    // Should still contain safe markdown
    assert!(html.contains("<strong>markdown</strong>") || html.contains("<b>markdown</b>"));
}

/// Test website metadata management
#[tokio::test]
async fn test_website_metadata() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "metadata-test-site";

    // Initialize with metadata
    let metadata = WebsiteMetadata {
        four_word_address: four_words.to_string(),
        title: "My Awesome Site".to_string(),
        description: Some("A test website".to_string()),
        home_page: "home.md".to_string(),
        published: false,
        published_at: None,
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_metadata(four_words, &metadata)
        .await
        .expect("Save metadata");

    // Load and verify
    let loaded = website_manager
        .get_metadata(four_words)
        .await
        .expect("Load metadata");

    assert_eq!(loaded.title, "My Awesome Site");
    assert_eq!(loaded.description, Some("A test website".to_string()));
    assert_eq!(loaded.home_page, "home.md");
}

/// Test concurrent edits to different parts of the same document
#[tokio::test]
async fn test_concurrent_section_edits() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "concurrent-edit-test";
    let path = "doc.md";

    // Create initial structured document
    let initial = MarkdownPage {
        path: path.to_string(),
        content: "# Document\n\n## Section 1\n\nContent 1\n\n## Section 2\n\nContent 2\n\n"
            .to_string(),
        title: Some("Doc".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &initial)
        .await
        .expect("Save initial");

    // Three peers load the document
    let doc_a = website_manager
        .load_page_doc(four_words, path)
        .await
        .expect("Load A");
    let doc_b = website_manager
        .load_page_doc(four_words, path)
        .await
        .expect("Load B");
    let doc_c = website_manager
        .load_page_doc(four_words, path)
        .await
        .expect("Load C");

    // Each peer edits a different section
    website_manager
        .insert_text_at(&doc_a, 15, "More details for section 1.\n")
        .expect("Edit A");
    website_manager
        .insert_text_at(&doc_b, 50, "Additional info for section 2.\n")
        .expect("Edit B");
    website_manager
        .append_text(&doc_c, "## Section 3\n\nNew section from peer C\n")
        .expect("Edit C");

    // Merge all changes
    let merged = website_manager
        .merge_page_docs(vec![doc_a, doc_b, doc_c])
        .await
        .expect("Merge");

    let content = website_manager.extract_content(&merged).expect("Extract");

    // All edits should be present
    assert!(content.contains("More details for section 1"));
    assert!(content.contains("Additional info for section 2"));
    assert!(content.contains("Section 3"));
    assert!(content.contains("New section from peer C"));
}

/// Test deleting a page from a website
#[tokio::test]
async fn test_delete_page() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "delete-page-test";

    let page = MarkdownPage {
        path: "temp.md".to_string(),
        content: "# Temporary Page".to_string(),
        title: Some("Temp".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &page)
        .await
        .expect("Save page");

    // Verify it exists
    let pages = website_manager
        .list_pages(four_words)
        .await
        .expect("List pages");
    assert!(pages.contains(&"temp.md".to_string()));

    // Delete the page
    website_manager
        .delete_page(four_words, "temp.md")
        .await
        .expect("Delete page");

    // Should no longer exist
    let pages_after = website_manager
        .list_pages(four_words)
        .await
        .expect("List pages after delete");
    assert!(!pages_after.contains(&"temp.md".to_string()));
}

/// Test rendering markdown with internal links to other pages
#[tokio::test]
async fn test_internal_links() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "internal-links-test";

    let home = MarkdownPage {
        path: "home.md".to_string(),
        content: "# Home\n\n[Go to About](about.md)\n[External](https://example.com)".to_string(),
        title: Some("Home".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &home)
        .await
        .expect("Save home");

    let html = website_manager
        .render_to_html(four_words, "home.md")
        .await
        .expect("Render HTML");

    // Should contain internal link (relative)
    assert!(html.contains("about.md") || html.contains("href=\"about.md\""));

    // Should contain external link
    assert!(html.contains("https://example.com"));
}

/// Test rendering markdown with images
#[tokio::test]
async fn test_markdown_with_images() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "images-test-site";

    let page = MarkdownPage {
        path: "gallery.md".to_string(),
        content:
            "# Gallery\n\n![My Image](images/photo.jpg)\n\n![External](https://example.com/pic.png)"
                .to_string(),
        title: Some("Gallery".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    website_manager
        .save_page(four_words, &page)
        .await
        .expect("Save page");

    let html = website_manager
        .render_to_html(four_words, "gallery.md")
        .await
        .expect("Render HTML");

    // Should contain image tags
    assert!(html.contains("<img"));
    assert!(html.contains("images/photo.jpg") || html.contains("src=\"images/photo.jpg\""));
}

/// Test path traversal attack prevention
#[tokio::test]
async fn test_path_traversal_attacks() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "security-test-site";

    // Test various path traversal attempts
    let malicious_paths = vec![
        "../../../etc/passwd",
        "../../sensitive.md",
        "../outside.md",
        "....//..//test.md",
    ];

    for path in malicious_paths {
        let page = MarkdownPage {
            path: path.to_string(),
            content: "# Malicious Content".to_string(),
            title: Some("Bad".to_string()),
            created_at: 1000,
            updated_at: 1000,
        };

        let result = website_manager.save_page(four_words, &page).await;
        assert!(
            result.is_err(),
            "Path '{}' should be rejected but was accepted",
            path
        );
    }
}

/// Test invalid path component rejection
#[tokio::test]
async fn test_invalid_path_components() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "invalid-path-test";

    // Test path with only dots
    let page = MarkdownPage {
        path: "a..b.md".to_string(),
        content: "# Content".to_string(),
        title: Some("Test".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    let result = website_manager.save_page(four_words, &page).await;
    assert!(
        result.is_err(),
        "Path with multiple consecutive dots should be rejected"
    );
}

/// Test valid path acceptance
#[tokio::test]
async fn test_valid_paths_accepted() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "valid-path-test";

    // Test various valid paths
    let valid_paths = vec![
        "home.md",
        "blog/post-1.md",
        "docs/api/v1.0.md",
        "my_file.md",
        "test-page_2.md",
    ];

    for path in valid_paths {
        let page = MarkdownPage {
            path: path.to_string(),
            content: "# Valid Content".to_string(),
            title: Some("Valid".to_string()),
            created_at: 1000,
            updated_at: 1000,
        };

        let result = website_manager.save_page(four_words, &page).await;
        assert!(
            result.is_ok(),
            "Valid path '{}' should be accepted but was rejected: {:?}",
            path,
            result.err()
        );
    }
}

/// Test max path length rejection
#[tokio::test]
async fn test_path_length_limit() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "xyz";

    // Create a path longer than 255 characters
    let long_path = "a".repeat(256) + ".md";

    let page = MarkdownPage {
        path: long_path.clone(),
        content: "# Content".to_string(),
        title: Some("Test".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    let result = website_manager.save_page(four_words, &page).await;
    assert!(
        result.is_err(),
        "Path longer than 255 characters should be rejected"
    );

    // Test exactly 255 characters - should be accepted by our validation
    // (though the filesystem may have stricter limits for the full doc_id)
    // Use a reasonable length that will pass both our validation and filesystem limits
    let max_path = "a".repeat(100) + "-x.md"; // 105 characters, well within limits

    let page_max = MarkdownPage {
        path: max_path,
        content: "# Content".to_string(),
        title: Some("Test".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    let result_max = website_manager.save_page(four_words, &page_max).await;
    assert!(
        result_max.is_ok(),
        "Valid path within length limits should be accepted: {:?}",
        result_max.err()
    );
}

/// Test empty path rejection
#[tokio::test]
async fn test_empty_path_rejection() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let manager = Arc::new(
        CrdtManager::new(temp_dir.path())
            .await
            .expect("Failed to create manager"),
    );
    let website_manager = WebsiteManager::new(manager);

    let four_words = "empty-path-test";

    let page = MarkdownPage {
        path: "".to_string(),
        content: "# Content".to_string(),
        title: Some("Test".to_string()),
        created_at: 1000,
        updated_at: 1000,
    };

    let result = website_manager.save_page(four_words, &page).await;
    assert!(result.is_err(), "Empty path should be rejected");
}
