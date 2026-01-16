//! Storage module for Communitas core
//!
//! This module provides storage-related functionality including
//! local storage and metrics.

pub mod local_storage;
pub mod metrics;
// pub mod reed_solomon_manager; // Removed: FEC not used in RC1b (gossip-based architecture)

// Re-export commonly used types
pub use local_storage::*;
// pub use reed_solomon_manager::*; // Removed: FEC not used in RC1b
