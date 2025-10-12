// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Saorsa P2P network.
//
// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Communitas Core - Business logic for the Communitas P2P collaboration platform
//!
//! This crate contains all the core functionality shared between the desktop app
//! and the headless node/CLI, without any UI dependencies.

// Security: Enforce no-panic policy in production code
#![cfg_attr(
    not(test),
    forbid(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Allow these in tests for convenience
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod auth_service;
// pub mod bootstrap_integration; // TODO: Reimplement in Sprint 2 with gossip-based bootstrap
pub mod core_context;
pub mod crdt;
// pub mod dht_identity; // Removed: DHT not used in RC1b (gossip-based architecture)
// pub mod dht_schemas; // Removed: DHT not used in RC1b (gossip-based architecture)
pub mod doc_replicator;
pub mod encrypted_storage;
pub mod error;
pub mod identity;
pub mod keystore;
pub mod message_sync;
// pub mod messaging; // TODO: Refactor in Sprint 3 for gossip pubsub
pub mod presence_service;
pub mod security;
pub mod storage;
pub mod test_harness;
pub mod types;

// Gossip overlay system (now default, no longer feature-gated)
pub mod gossip;

// Re-export commonly used types
pub use auth_service::{AuthService, SessionInfo};
pub use core_context::CoreContext;
pub use error::{AppError, AppResult as Result};

// Re-export gossip context (now default, no longer feature-gated)
pub use gossip::GossipContext;

// Re-export identity helpers for convenience
pub use identity::{
    IdentityError, IdentityResult, conn_from_words, conn_words, generate_id_words,
    identity_to_seed, validate_id_words,
};
