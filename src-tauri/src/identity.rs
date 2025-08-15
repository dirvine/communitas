// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


// Copyright 2024 P2P Foundation
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Identity management module for Communitas
//!
//! Integrates with the four-word address system from saorsa-core

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// User identity with four-word address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub id: String,
    pub four_word_address: String,
    pub display_name: String,
    pub bio: String,
    pub avatar_hash: Option<String>,
    pub created_at: i64,
}

/// Identity manager for the Communitas app
#[derive(Debug)]
pub struct IdentityManager {
    user_profile: Arc<RwLock<Option<UserIdentity>>>,
}

impl IdentityManager {
    /// Create a new identity manager
    pub fn new() -> Self {
        Self {
            user_profile: Arc::new(RwLock::new(None)),
        }
    }

    /// Create or load user identity using four-word-networking for the address
    pub async fn create_or_load_identity(&self, _passphrase: String) -> Result<UserIdentity> {
        // Derive a stable local address for identity display; in production this should
        // map to the node's public reachable address(es)
        let default_socket: std::net::SocketAddr = "127.0.0.1:9000".parse().unwrap();
        let net = saorsa_core::NetworkAddress::from(default_socket);
        let four_word_address = net.four_words().unwrap_or("").to_string();

        let user_identity = UserIdentity {
            id: uuid::Uuid::new_v4().to_string(),
            four_word_address,
            display_name: "Anonymous".to_string(),
            bio: "P2P Network User".to_string(),
            avatar_hash: None,
            created_at: chrono::Utc::now().timestamp(),
        };

        *self.user_profile.write().await = Some(user_identity.clone());
        Ok(user_identity)
    }

    /// Get current user identity
    pub async fn get_identity(&self) -> Result<UserIdentity> {
        self.user_profile
            .read()
            .await
            .clone()
            .context("No identity loaded")
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        display_name: Option<String>,
        bio: Option<String>,
        avatar_hash: Option<String>,
    ) -> Result<UserIdentity> {
        let mut profile = self.user_profile.write().await;
        let user = profile.as_mut().context("No identity loaded")?;

        if let Some(name) = display_name {
            user.display_name = name;
        }
        if let Some(bio_text) = bio {
            user.bio = bio_text;
        }
        if let Some(avatar) = avatar_hash {
            user.avatar_hash = Some(avatar);
        }

        // TODO: Update in DHT using P2P node when real implementation is ready
        Ok(user.clone())
    }

    /// Lookup identity by four-word address (validate via decoder)
    pub async fn lookup_identity(&self, address: String) -> Result<UserIdentity> {
        // Validate and normalize address using four-word decoder
        let _sock = saorsa_core::NetworkAddress::from_four_words(&address)?;
        Ok(UserIdentity {
            id: format!("lookup-{}", address),
            four_word_address: address.clone(),
            display_name: format!("User {}", address),
            bio: "P2P user".to_string(),
            avatar_hash: None,
            created_at: chrono::Utc::now().timestamp(),
        })
    }
}
