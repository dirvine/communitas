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

pub mod core_context;
pub mod dht_schemas;
pub mod error;
pub mod keystore;
pub mod security;
pub mod storage;
pub mod test_harness;

// Re-export commonly used types
pub use core_context::CoreContext;
pub use error::{AppError, AppResult as Result};

// Re-export saorsa-core for convenience
pub use saorsa_core;
