// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! WebRTC Integration Module
//!
//! Provides real-time voice, video, and screen sharing capabilities using
//! saorsa-webrtc over the Communitas gossip overlay network.
//!
//! ## Architecture
//!
//! - **SignalingTransport**: Gossip-based signaling using PubSub and Rendezvous
//! - **PeerIdentity**: Four-word addresses as peer identifiers
//! - **Transport**: ant-quic QUIC transport with native NAT traversal
//! - **Media**: RTP-to-QUIC bridge for audio/video/screen share
//!
//! ## Key Components
//!
//! - `GossipSignalingTransport`: Implements WebRTC signaling over gossip
//! - `CommunitasIdentity`: Four-word address wrapper for `PeerIdentity` trait
//! - `WebRtcService`: High-level service for call management
//!
//! ## Usage
//!
//! ```no_run
//! use communitas_core::webrtc::{GossipSignalingTransport, CommunitasIdentity};
//! use saorsa_webrtc_core::service::WebRtcService;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create signaling transport
//! let signaling = GossipSignalingTransport::new(gossip_context)?;
//!
//! // Initialize WebRTC service
//! let webrtc = WebRtcService::<CommunitasIdentity, GossipSignalingTransport>::builder()
//!     .with_identity(my_identity)
//!     .with_transport(signaling)
//!     .build()
//!     .await?;
//!
//! // Start the service
//! webrtc.start().await?;
//! # Ok(())
//! # }
//! ```
//!
//! See `docs/architecture/webrtc-multimedia.md` for complete architecture documentation.

pub mod gossip_signaling;
pub mod identity;
pub mod recorder;
pub mod service;

// Re-export key types
pub use gossip_signaling::GossipSignalingTransport;
pub use identity::CommunitasIdentity;
pub use recorder::{
    AudioQuality, RecordingConfig, RecordingError, RecordingResult, RecordingSession, VideoQuality,
};
pub use service::{CallState as CommunitasCallState, CommunitasWebRtcService, MediaDevice};

// Re-export saorsa-webrtc-core types for convenience
pub use saorsa_webrtc_core::{
    call::CallManager,
    service::WebRtcService,
    signaling::SignalingMessage,
    types::{
        CallEvent, CallId, CallQualityMetrics, CallState, IceCandidate, MediaConstraints, MediaType,
    },
};
