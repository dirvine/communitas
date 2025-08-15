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


//! Communitas - P2P Diagnostic Chat Application
//!
//! A comprehensive diagnostic chat application built on the P2P Foundation network,
//! providing real-time visibility into all network layers while functioning as a
//! fully-featured group chat application.

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod chat;
pub mod diagnostics;
pub mod network;
pub mod testing;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main application state
#[derive(Clone)]
pub struct CommuniasApp {
    /// Core chat service
    pub chat_service: Arc<chat::ChatService>,
    /// Diagnostics engine
    pub diagnostics: Arc<diagnostics::DiagnosticsEngine>,
    /// Test harness (optional)
    #[cfg(feature = "test-harness")]
    pub test_harness: Arc<testing::TestHarness>,
    /// Network integration
    pub network: Arc<network::NetworkIntegration>,
    /// Application state
    pub state: Arc<RwLock<AppState>>,
}

/// Application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Current user identity
    pub identity: Identity,
    /// Active tab in UI
    pub current_tab: Tab,
    /// Connection status
    pub connected: bool,
    /// Bootstrap node
    pub bootstrap_node: String,
}

/// User identity with 4-word address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    /// Four-word network address
    pub four_word_address: String,
    /// Public key bytes
    pub public_key: Vec<u8>,
    /// Display name (optional)
    pub display_name: Option<String>,
}

/// UI tabs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Tab {
    /// Overview of network health
    Overview,
    /// Chat messages
    Messages,
    /// Network diagnostics
    Network,
    /// Storage diagnostics
    Storage,
    /// Advanced packet inspection
    Advanced,
}

impl std::fmt::Debug for CommuniasApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommuniasApp")
            .field("chat_service", &"Arc<ChatService>")
            .field("diagnostics", &"Arc<DiagnosticsEngine>")
            .field("network", &"Arc<NetworkIntegration>")
            .field("state", &"Arc<RwLock<AppState>>")
            .finish()
    }
}

impl CommuniasApp {
    /// Create a new Communitas application instance
    pub async fn new(bootstrap_node: String) -> Result<Self> {
        // Initialize components
        let network = Arc::new(network::NetworkIntegration::new(bootstrap_node.clone()).await?);
        let chat_service = Arc::new(chat::ChatService::new(network.clone()).await?);
        let diagnostics = Arc::new(diagnostics::DiagnosticsEngine::new(network.clone()));

        // Load or create identity
        let identity = network.get_or_create_identity().await?;

        let state = Arc::new(RwLock::new(AppState {
            identity,
            current_tab: Tab::Overview,
            connected: false,
            bootstrap_node,
        }));

        Ok(Self {
            chat_service,
            diagnostics,
            #[cfg(feature = "test-harness")]
            test_harness: Arc::new(testing::TestHarness::new()),
            network,
            state,
        })
    }

    /// Connect to the P2P network
    pub async fn connect(&self) -> Result<()> {
        self.network.connect_to_bootstrap().await?;

        // Update state
        let mut state = self.state.write().await;
        state.connected = true;

        // Start diagnostics collection
        self.diagnostics.start_collection();

        Ok(())
    }

    /// Get current network health
    pub async fn get_network_health(&self) -> diagnostics::NetworkHealth {
        self.diagnostics.get_network_health().await
    }
}
