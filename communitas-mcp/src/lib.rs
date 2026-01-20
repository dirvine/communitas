// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Communitas MCP Server Library
//!
//! This module exports the internal types needed for testing.

// Alias communitas_bindings (the actual lib name) as communitas_core
extern crate communitas_bindings as communitas_core;

pub mod auth;
pub mod presence;
pub mod protocol;
pub mod token;
pub mod tools;
