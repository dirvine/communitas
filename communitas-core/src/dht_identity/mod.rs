//! DHT Identity System for Communitas
//!
//! This module implements the comprehensive DHT identity system that stores:
//! - Identity packets at the hash of four-word identities
//! - Connection information for QUIC/ant-quic NAT traversal  
//! - Web pages (up to 5MB) via content-addressed storage
//! - Preferred display names with PQC signatures
//!
//! The system uses a three-layer architecture:
//! 1. DHT Layer: Small pointer records (≤512B) for fast lookups
//! 2. Content-Addressed Blobs: Signed, verifiable larger data structures
//! 3. Erasure-Coded Storage: Actual content with redundancy

pub mod key_derivation;
pub mod records;
pub mod blobs;
pub mod storage;
pub mod validation;
pub mod integration;
pub mod demo;

pub use key_derivation::*;
pub use records::*;
pub use blobs::*;
pub use storage::*;
pub use validation::*;
pub use integration::*;

/// DHT record size limit from saorsa-core
pub const DHT_RECORD_MAX_SIZE: usize = 512;

/// Maximum web content size per identity (5MB)
pub const MAX_WEB_CONTENT_SIZE: usize = 5 * 1024 * 1024;

/// Current protocol version for all records
pub const PROTOCOL_VERSION: u8 = 1;
