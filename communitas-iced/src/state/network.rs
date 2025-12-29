// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Network state for P2P connectivity.

/// Information about a connected peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer's four-word identity.
    pub four_words: String,
    /// Peer's display name (if known).
    pub display_name: Option<String>,
    /// Connection endpoint.
    pub endpoint: String,
    /// Last seen timestamp.
    pub last_seen: i64,
    /// Whether this is a bootstrap node.
    pub is_bootstrap: bool,
}

impl PeerInfo {
    /// Get the short identity display.
    #[must_use]
    pub fn short_identity(&self) -> String {
        let words: Vec<&str> = self.four_words.split('-').collect();
        if words.len() >= 2 {
            format!("{}..{}", words[0], words[words.len() - 1])
        } else {
            self.four_words.clone()
        }
    }

    /// Get the display label.
    #[must_use]
    pub fn display_label(&self) -> String {
        self.display_name
            .clone()
            .unwrap_or_else(|| self.short_identity())
    }
}

/// Network information.
#[derive(Debug, Clone, Default)]
pub struct NetworkInfo {
    /// Whether networking is active.
    pub is_networking: bool,
    /// Local listen address.
    pub listen_address: Option<String>,
    /// External/public address (NAT-reflected).
    pub external_address: Option<String>,
    /// Connected peers.
    pub peers: Vec<PeerInfo>,
    /// Bootstrap nodes.
    pub bootstrap_nodes: Vec<BootstrapNode>,
    /// Last connection error (if any).
    pub last_error: Option<String>,
}

/// A bootstrap node.
#[derive(Debug, Clone)]
pub struct BootstrapNode {
    /// Node name/label.
    pub name: String,
    /// Node address (ip:port).
    pub address: String,
    /// Whether we're connected to this node.
    pub is_connected: bool,
}

impl BootstrapNode {
    /// Default bootstrap nodes.
    #[must_use]
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "Droplet 2064413".to_string(),
                address: "167.71.188.131:50000".to_string(),
                is_connected: false,
            },
            Self {
                name: "communitas-bootstrap-1".to_string(),
                address: "138.197.29.195:50000".to_string(),
                is_connected: false,
            },
        ]
    }
}

impl NetworkInfo {
    /// Create a new network info with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self {
            is_networking: false,
            listen_address: None,
            external_address: None,
            peers: Vec::new(),
            bootstrap_nodes: BootstrapNode::defaults(),
            last_error: None,
        }
    }

    /// Get the number of connected peers.
    #[must_use]
    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }
}
