// Copyright (c) 2025 Saorsa Labs Limited
// SPDX-License-Identifier: AGPL-3.0-or-later OR Commercial

//! Services Module
//!
//! Provides wrappers around communitas-core functionality for the UI layer.

mod core_service;

pub use core_service::CoreService;

// Re-export types needed by screens
pub use communitas_core::crdt::EntityType;
pub use communitas_core::entity_service::Entity;
