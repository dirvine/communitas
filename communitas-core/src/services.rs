// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Domain services for Communitas
//!
//! This module provides high-level service abstractions that wrap CRDT operations
//! for specific business entities (channels, groups, issues, etc.).

use crate::{CrdtManager, CrdtResult};
use std::sync::Arc;

/// Consolidated service container for all domain services
pub struct CoreServices {
    crdt_manager: Arc<CrdtManager>,
}

impl CoreServices {
    /// Bootstrap services with a database path
    pub async fn bootstrap(db_path: impl AsRef<std::path::Path>) -> CrdtResult<Self> {
        let crdt_manager = Arc::new(CrdtManager::new(db_path).await?);

        Ok(Self { crdt_manager })
    }

    /// Get a reference to the CRDT manager
    pub fn crdt_manager(&self) -> &Arc<CrdtManager> {
        &self.crdt_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_bootstrap_services() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let services = CoreServices::bootstrap(&db_path).await.unwrap();
        assert!(Arc::strong_count(services.crdt_manager()) >= 1);
    }
}
