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

//! Tauri Commands for Contact Management
//!
//! This module provides the Tauri command interface for the contact management system

use crate::contacts::{ContactInvitation, ContactManager, ContactProfile, ContactStatus};
use anyhow::Result;

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

pub type ContactManagerState = Arc<RwLock<ContactManager>>;

#[tauri::command]
pub async fn add_contact(
    manager: State<'_, ContactManagerState>,
    four_word_address: String,
    display_name: String,
    bio: Option<String>,
) -> Result<ContactProfile, String> {
    let manager = manager.read().await;
    manager
        .add_contact(four_word_address, display_name, bio)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_contact(
    manager: State<'_, ContactManagerState>,
    contact_id: String,
) -> Result<Option<ContactProfile>, String> {
    let manager = manager.read().await;
    Ok(manager.get_contact(&contact_id).await)
}

#[tauri::command]
pub async fn get_contact_by_address(
    manager: State<'_, ContactManagerState>,
    address: String,
) -> Result<Option<ContactProfile>, String> {
    let manager = manager.read().await;
    Ok(manager.get_contact_by_address(&address).await)
}

#[tauri::command]
pub async fn list_contacts(
    manager: State<'_, ContactManagerState>,
    status_filter: Option<String>,
) -> Result<Vec<ContactProfile>, String> {
    let manager = manager.read().await;

    let status = match status_filter.as_deref() {
        Some("pending") => Some(ContactStatus::Pending),
        Some("active") => Some(ContactStatus::Active),
        Some("offline") => Some(ContactStatus::Offline),
        Some("blocked") => Some(ContactStatus::Blocked),
        Some("removed") => Some(ContactStatus::Removed),
        _ => None,
    };

    Ok(manager.list_contacts(status).await)
}

#[tauri::command]
pub async fn search_contacts(
    manager: State<'_, ContactManagerState>,
    query: String,
) -> Result<Vec<ContactProfile>, String> {
    let manager = manager.read().await;
    Ok(manager.search_contacts(&query).await)
}

#[tauri::command]
pub async fn create_invitation(
    manager: State<'_, ContactManagerState>,
    inviter_address: String,
    inviter_name: String,
    message: Option<String>,
    expires_in_hours: u64,
    max_usage: u32,
) -> Result<ContactInvitation, String> {
    let manager = manager.read().await;
    manager
        .create_invitation(
            inviter_address,
            inviter_name,
            message,
            expires_in_hours,
            max_usage,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn accept_invitation(
    manager: State<'_, ContactManagerState>,
    token: String,
    accepter_address: String,
    accepter_name: String,
) -> Result<ContactProfile, String> {
    let manager = manager.read().await;
    manager
        .accept_invitation(&token, accepter_address, accepter_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_contact_status(
    manager: State<'_, ContactManagerState>,
    contact_id: String,
    status: String,
) -> Result<(), String> {
    let manager = manager.read().await;

    let contact_status = match status.as_str() {
        "pending" => ContactStatus::Pending,
        "active" => ContactStatus::Active,
        "offline" => ContactStatus::Offline,
        "blocked" => ContactStatus::Blocked,
        "removed" => ContactStatus::Removed,
        _ => return Err("Invalid status".to_string()),
    };

    manager
        .update_contact_status(&contact_id, contact_status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_contact_file_system_path(
    manager: State<'_, ContactManagerState>,
    contact_id: String,
) -> Result<String, String> {
    let manager = manager.read().await;
    let path = manager.get_contact_file_system_path(&contact_id);
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn generate_four_word_address() -> Result<String, String> {
    // Generate encoding for a typical local address using core encoder
    let socket: SocketAddr = "127.0.0.1:9000"
        .parse()
        .or_else(|_| "127.0.0.1:0".parse())
        .map_err(|e| format!("Failed to parse socket address: {}", e))?;
    let net = saorsa_core::NetworkAddress::from(socket);
    Ok(net.four_words().unwrap_or("").to_string())
}

/// Initialize the contact manager with the given data directory
pub async fn init_contact_manager(data_dir: std::path::PathBuf) -> Result<ContactManager> {
    let contacts_dir = data_dir.join("contacts");
    ContactManager::new(contacts_dir).await
}

/// Encode a socket address into a four-word representation using saorsa-core
#[tauri::command]
pub async fn four_word_encode_address(addr: String) -> Result<String, String> {
    let socket: SocketAddr = addr
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    let net = saorsa_core::NetworkAddress::from(socket);
    Ok(net.four_words().unwrap_or("").to_string())
}

/// Decode a four-word representation back to a socket address string
#[tauri::command]
pub async fn four_word_decode_address(words: String) -> Result<String, String> {
    // Try parse directly via four-word decoding
    match saorsa_core::NetworkAddress::from_four_words(&words) {
        Ok(net) => Ok(net.socket_addr().to_string()),
        Err(_) => match saorsa_core::NetworkAddress::from_str(&words) {
            Ok(net) => Ok(net.socket_addr().to_string()),
            Err(e) => Err(format!("Failed to decode: {}", e)),
        },
    }
}
