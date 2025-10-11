//! Storage module for Communitas Tauri backend
//!
//! This module provides storage-related functionality including
//! DHT storage, local storage, Reed-Solomon encoding, and metrics.

pub mod dht_storage;
pub mod local_storage;
pub mod metrics;
pub mod reed_solomon_manager;

// Re-export commonly used types
pub use dht_storage::*;
pub use local_storage::*;
pub use reed_solomon_manager::*;
