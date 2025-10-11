// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Core types for Communitas (replaces Saorsa Core types)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Device type classification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Laptop,
    Mobile,
    Server,
    Unknown,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Desktop => write!(f, "Desktop"),
            DeviceType::Laptop => write!(f, "Laptop"),
            DeviceType::Mobile => write!(f, "Mobile"),
            DeviceType::Server => write!(f, "Server"),
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl DeviceType {
    /// Parse device type from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "desktop" => DeviceType::Desktop,
            "laptop" => DeviceType::Laptop,
            "mobile" => DeviceType::Mobile,
            "server" => DeviceType::Server,
            _ => DeviceType::Unknown,
        }
    }
}

/// User profile with local identity and passkey support
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserProfile {
    /// Four-word user ID (derived from public key)
    pub id_fw: String,

    /// Display name for the user
    pub display_name: String,

    /// Ed25519 public key (32 bytes)
    pub pubkey: [u8; 32],

    /// Device type classification
    pub device_type: DeviceType,

    /// Connection identities (four-word encoded SocketAddr)
    pub connection_ids: Vec<String>,

    /// Passkey RP ID (e.g., "communitas.id")
    pub passkey_rpid: String,

    /// Passkey credential ID (if passkey is registered)
    pub passkey_cred_id: Option<Vec<u8>>,

    /// Passkey public key (if passkey is registered)
    pub passkey_pubkey: Option<Vec<u8>>,

    /// Local storage directory for this profile
    pub storage_dir: PathBuf,
}

impl UserProfile {
    /// Create a new user profile
    pub fn new(
        id_fw: String,
        display_name: String,
        pubkey: [u8; 32],
        device_type: DeviceType,
        storage_dir: PathBuf,
    ) -> Self {
        Self {
            id_fw,
            display_name,
            pubkey,
            device_type,
            connection_ids: Vec::new(),
            passkey_rpid: "communitas.id".to_string(),
            passkey_cred_id: None,
            passkey_pubkey: None,
            storage_dir,
        }
    }

    /// Check if passkey is registered
    pub fn has_passkey(&self) -> bool {
        self.passkey_cred_id.is_some() && self.passkey_pubkey.is_some()
    }

    /// Add a connection identity
    pub fn add_connection_id(&mut self, conn_id: String) {
        if !self.connection_ids.contains(&conn_id) {
            self.connection_ids.push(conn_id);
        }
    }

    /// Remove a connection identity
    pub fn remove_connection_id(&mut self, conn_id: &str) {
        self.connection_ids.retain(|id| id != conn_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_str() {
        assert_eq!(DeviceType::from_str("desktop"), DeviceType::Desktop);
        assert_eq!(DeviceType::from_str("Desktop"), DeviceType::Desktop);
        assert_eq!(DeviceType::from_str("LAPTOP"), DeviceType::Laptop);
        assert_eq!(DeviceType::from_str("mobile"), DeviceType::Mobile);
        assert_eq!(DeviceType::from_str("server"), DeviceType::Server);
        assert_eq!(DeviceType::from_str("unknown"), DeviceType::Unknown);
        assert_eq!(DeviceType::from_str("invalid"), DeviceType::Unknown);
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(DeviceType::Desktop.to_string(), "Desktop");
        assert_eq!(DeviceType::Laptop.to_string(), "Laptop");
        assert_eq!(DeviceType::Mobile.to_string(), "Mobile");
    }

    #[test]
    fn test_user_profile_creation() {
        let pubkey = [0u8; 32];
        let profile = UserProfile::new(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            pubkey,
            DeviceType::Desktop,
            PathBuf::from("/tmp/profiles/ocean-forest-moon-star"),
        );

        assert_eq!(profile.id_fw, "ocean-forest-moon-star");
        assert_eq!(profile.display_name, "Alice");
        assert_eq!(profile.device_type, DeviceType::Desktop);
        assert!(!profile.has_passkey());
        assert_eq!(profile.connection_ids.len(), 0);
    }

    #[test]
    fn test_connection_ids() {
        let pubkey = [0u8; 32];
        let mut profile = UserProfile::new(
            "ocean-forest-moon-star".to_string(),
            "Alice".to_string(),
            pubkey,
            DeviceType::Desktop,
            PathBuf::from("/tmp/profiles/ocean-forest-moon-star"),
        );

        profile.add_connection_id("river-mountain-sun-cloud".to_string());
        assert_eq!(profile.connection_ids.len(), 1);

        // Adding duplicate should not increase length
        profile.add_connection_id("river-mountain-sun-cloud".to_string());
        assert_eq!(profile.connection_ids.len(), 1);

        profile.add_connection_id("forest-ocean-star-moon".to_string());
        assert_eq!(profile.connection_ids.len(), 2);

        profile.remove_connection_id("river-mountain-sun-cloud");
        assert_eq!(profile.connection_ids.len(), 1);
        assert_eq!(profile.connection_ids[0], "forest-ocean-star-moon");
    }
}
