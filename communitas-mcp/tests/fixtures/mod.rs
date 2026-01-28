// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Test Data Fixtures for MCP Testing
//!
//! Provides reusable test data structures and builders for consistent testing.

pub mod contacts;
pub mod drive;
pub mod entities;
pub mod kanban;
pub mod messaging;

// Re-export common fixtures
pub use contacts::*;
pub use drive::*;
pub use entities::*;
pub use kanban::*;
pub use messaging::*;
