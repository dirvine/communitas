// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! WebRTC Identity Wrapper
//!
//! Implements the `PeerIdentity` trait from saorsa-webrtc for Communitas
//! four-word addresses.

use anyhow::Result;
use saorsa_webrtc_core::identity::PeerIdentity;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Communitas identity wrapper for WebRTC
///
/// Wraps a four-word address (e.g., "ocean-forest-moon-star") to implement
/// the `PeerIdentity` trait required by saorsa-webrtc.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommunitasIdentity {
    /// Four-word address (e.g., "ocean-forest-moon-star")
    pub four_words: String,
}

impl CommunitasIdentity {
    /// Create a new identity from a four-word address
    pub fn new(four_words: String) -> Result<Self> {
        // Validate the four-word format
        if !crate::identity::validate_id_words(&four_words) {
            return Err(anyhow::anyhow!("Invalid four-word address: {}", four_words));
        }

        Ok(Self { four_words })
    }

    /// Creates a placeholder identity for group calls.
    ///
    /// Used when starting a group call where there's no initial target peer.
    /// The placeholder value is a valid four-word address but serves as a
    /// sentinel value until real peers join.
    pub fn placeholder() -> Self {
        // Use a well-known placeholder that's a valid four-word format
        Self {
            four_words: "group-call-room-host".to_string(),
        }
    }

    /// Returns true if this is a placeholder identity.
    pub fn is_placeholder(&self) -> bool {
        self.four_words == "group-call-room-host"
    }

    /// Get the four-word address
    pub fn four_words(&self) -> &str {
        &self.four_words
    }
}

impl PeerIdentity for CommunitasIdentity {
    fn to_string_repr(&self) -> String {
        self.four_words.clone()
    }

    fn from_string_repr(s: &str) -> Result<Self> {
        Self::new(s.to_string())
    }

    fn unique_id(&self) -> String {
        // Use the four-word address as the unique identifier
        self.four_words.clone()
    }
}

impl fmt::Display for CommunitasIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.four_words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_creation() {
        let identity = CommunitasIdentity::new("ocean-forest-moon-star".to_string());
        assert!(identity.is_ok());

        let identity = identity.expect("valid identity");
        assert_eq!(identity.four_words(), "ocean-forest-moon-star");
        assert_eq!(identity.unique_id(), "ocean-forest-moon-star");
        assert_eq!(identity.to_string(), "ocean-forest-moon-star");
    }

    #[test]
    fn test_peer_identity_trait() {
        let identity =
            CommunitasIdentity::new("ocean-forest-moon-star".to_string()).expect("valid identity");

        // Test to_string_repr
        assert_eq!(identity.to_string_repr(), "ocean-forest-moon-star");

        // Test from_string_repr
        let identity2 = CommunitasIdentity::from_string_repr("ocean-forest-moon-star")
            .expect("valid from_string_repr");
        assert_eq!(identity, identity2);

        // Test unique_id
        assert_eq!(identity.unique_id(), "ocean-forest-moon-star");
    }

    #[test]
    fn test_serialization() {
        let identity =
            CommunitasIdentity::new("ocean-forest-moon-star".to_string()).expect("valid identity");

        // Test JSON serialization
        let json = serde_json::to_string(&identity).expect("serialize");
        let deserialized: CommunitasIdentity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(identity, deserialized);
    }

    #[test]
    fn test_placeholder_identity() {
        let placeholder = CommunitasIdentity::placeholder();

        assert!(placeholder.is_placeholder());
        assert_eq!(placeholder.four_words(), "group-call-room-host");
    }

    #[test]
    fn test_non_placeholder_identity() {
        let identity =
            CommunitasIdentity::new("ocean-forest-moon-star".to_string()).expect("valid identity");

        assert!(!identity.is_placeholder());
    }
}
