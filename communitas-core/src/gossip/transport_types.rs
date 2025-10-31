// Copyright (c) 2025 Saorsa Labs Limited
//
// Shared transport type aliases to avoid Clone/Box/RwLock complexity

use saorsa_gossip_transport::GossipTransport;
use std::sync::Arc;

/// Shared transport type used across Sites components
///
/// This avoids the complexity of Arc<RwLock<Box<dyn GossipTransport>>>
/// and the Clone requirements. All Sites components use this type.
pub type SharedTransport = Arc<dyn GossipTransport + Send + Sync>;
