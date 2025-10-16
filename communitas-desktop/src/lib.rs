// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

// Security: Enforce no-panic policy in production code
#![cfg_attr(
    not(test),
    forbid(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]
// Allow these in tests for convenience
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

// Re-export modules that need to be accessible from integration tests
pub mod core_commands;
pub mod crdt_error;
pub mod crdt_manager;
pub mod gossip_commands;
pub mod member_commands;
pub mod member_manager;
pub mod update_manager;
pub mod webrtc_commands;
