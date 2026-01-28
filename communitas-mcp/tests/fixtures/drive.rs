// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Drive test fixtures

#![allow(dead_code)]

use serde_json::{Value, json};

/// File fixture
pub fn file_fixture(name: &str, content: &str) -> Value {
    json!({
        "name": name,
        "content": content
    })
}

/// File with path
pub fn file_with_path(path: &str, content: &str) -> Value {
    json!({
        "path": path,
        "content": content
    })
}

/// Directory fixture
pub fn directory_fixture(path: &str) -> Value {
    json!({
        "path": path
    })
}

/// List files fixture
pub fn list_files_fixture(path: &str, limit: u32) -> Value {
    json!({
        "path": path,
        "limit": limit
    })
}

/// File move fixture
pub fn file_move_fixture(from: &str, to: &str) -> Value {
    json!({
        "from": from,
        "to": to
    })
}

/// File copy fixture
pub fn file_copy_fixture(from: &str, to: &str) -> Value {
    json!({
        "from": from,
        "to": to
    })
}

/// Upload fixture
pub fn upload_fixture(path: &str, content: &str) -> Value {
    json!({
        "path": path,
        "content": content
    })
}

/// Streaming upload fixture
pub fn streaming_upload_fixture(path: &str, total_size: u64) -> Value {
    json!({
        "path": path,
        "total_size": total_size
    })
}

/// Download fixture
pub fn download_fixture(path: &str) -> Value {
    json!({
        "path": path
    })
}

/// Share link fixture
pub fn share_link_fixture(file_id: &str, expires_in_hours: u32) -> Value {
    json!({
        "file_id": file_id,
        "expires_in_hours": expires_in_hours
    })
}

/// Sample text content for testing
pub fn sample_text_content() -> &'static str {
    "This is test file content for MCP drive testing."
}

/// Sample binary content (PNG header) for testing
pub fn sample_binary_content() -> Vec<u8> {
    vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] // PNG header
}

/// Disk types
pub enum DiskType {
    Private,
    Public,
    Shared,
}

impl DiskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiskType::Private => "private",
            DiskType::Public => "public",
            DiskType::Shared => "shared",
        }
    }
}

/// List disks fixture
pub fn list_disks_fixture() -> Value {
    json!({})
}

/// Staging conflict resolution fixture
pub fn resolve_conflict_fixture(upload_id: &str, resolution: &str) -> Value {
    json!({
        "upload_id": upload_id,
        "resolution": resolution
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_fixture() {
        let file = file_fixture("test.txt", "Hello");
        assert_eq!(file["name"], "test.txt");
        assert_eq!(file["content"], "Hello");
    }

    #[test]
    fn test_disk_types() {
        assert_eq!(DiskType::Private.as_str(), "private");
        assert_eq!(DiskType::Public.as_str(), "public");
        assert_eq!(DiskType::Shared.as_str(), "shared");
    }

    #[test]
    fn test_binary_content() {
        let content = sample_binary_content();
        assert!(!content.is_empty());
        // PNG header starts with 0x89 0x50 0x4E 0x47
        assert_eq!(content[0], 0x89);
    }
}
