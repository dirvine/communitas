// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Manager - Manages Yrs documents with filesystem persistence
//!
//! This module provides the core CRDT functionality for collaborative editing
//! shared between desktop and headless nodes.

mod error;
mod manager;

pub use error::{CrdtError, CrdtResult};
pub use manager::{CompactionConfig, CompactionResult, CrdtManager};
