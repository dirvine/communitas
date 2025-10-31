// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Gossip Overlay System
//!
//! This module implements the saorsa-gossip based overlay network that replaces
//! DHT-based discovery. It provides:
//!
//! - Post-quantum secure gossip protocols (HyParView + SWIM + Plumtree)
//! - MLS group management for channels/projects/orgs
//! - Presence beacons for online/offline detection
//! - CRDT-based message synchronization
//! - Favourite contacts backup system
//!
//! ## Architecture (per SPEC.md)
//!
//! | Communitas Object | Gossip Mapping |
//! |-------------------|----------------|
//! | User identity (four-word) | ML-DSA identity + alias |
//! | Contact | Overlay edge and seed |
//! | Channel/Project/Org | MLS group + gossip topic |
//! | Presence | MLS-encrypted rotating beacons |
//! | Backup | Favourite contacts hold encrypted replicas |

// Gossip is now the default networking layer (no longer feature-gated)

pub mod backup;
pub mod block_cache;
pub mod boot;
pub mod context;
pub mod coordinator;
pub mod discovery;
pub mod name_record;
pub mod peer_cache;
pub mod port_manager;
pub mod presence;
pub mod rendezvous;
pub mod signed_provider;
pub mod sites;
pub mod sites_dispatcher;
pub mod sites_listener;
pub mod telemetry;
pub mod transport_types;

// Re-export key types
pub use backup::BackupManager;
pub use block_cache::{BlockCache, CacheStats};
pub use boot::GossipBootSequence;
pub use context::GossipContext;
pub use coordinator::CoordinatorClient;
pub use discovery::{FoafDiscovery, IntroducerConfig, cold_start_discovery};
pub use name_record::{NameRecord, NameRegistry};
pub use peer_cache::{NatClass, PeerCache, PeerCacheEntry};
pub use port_manager::PortManager;
pub use presence::PresenceWrapper;
pub use rendezvous::RendezvousClient;
pub use saorsa_gossip_coordinator::{CoordinatorRoles, NatClass as CoordinatorNatClass};
pub use saorsa_gossip_rendezvous::{Capability, ProviderSummary};
pub use signed_provider::{CollectorStats, ProviderCollector, ProviderRateLimiter};
pub use sites::{
    Block, MAX_BLOCK_SIZE, SiteFetcher, SiteId, SiteManifest, SitePublisher, SiteRequest,
    SiteResponse, SitesWire, chunk_content,
};
pub use sites_listener::SitesListener;
