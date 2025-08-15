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


//! Identity management module for Communitas
//!
//! This module provides secure cross-platform identity management with:
//! - Platform-specific secure storage (Keychain, Credential Manager, Secret Service)
//! - Encrypted file storage as fallback
//! - Integration with saorsa-core identity system
//! - 4-word address generation and management

pub mod secure_storage;
pub mod encrypted_file_storage;
pub mod macos_keychain;
pub mod windows_credential_manager;
pub mod linux_secret_service;
pub mod identity_manager;

// Re-export main types
pub use secure_storage::{
    SecureStorage, 
    SecureStorageFactory, 
    KeyEntry, 
    KeyMetadata, 
    StorageInfo
};

pub use identity_manager::CommunidentityManager;
