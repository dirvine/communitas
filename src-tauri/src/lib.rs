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


//! Communitas P2P Collaboration Platform Library
//!
//! This library provides the core functionality for the Communitas platform,
//! including bootstrap node capabilities, contact management, and P2P networking.

pub mod bootstrap;
pub mod contact_commands;
pub mod contacts;
pub mod files;
pub mod groups;
pub mod identity;
pub mod storage;
pub mod stores;

// Re-export main components
pub use bootstrap::{run_bootstrap_node, BootstrapConfig, BootstrapNode, NodeStats};
pub use contact_commands::{init_contact_manager, ContactManagerState};
pub use contacts::{
    ContactInvitation, ContactManager, ContactPreferences, ContactProfile, ContactStatus,
};
// Store API re-exports
pub use stores::{
    init_local_stores, get_metadata, create_organization, create_group_local as create_group, create_project,
    add_contact_local, list_markdown, read_markdown_file, write_markdown_file,
    create_markdown, Metadata, ScopePath, MarkdownFileInfo,
};
