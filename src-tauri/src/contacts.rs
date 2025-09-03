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

//! Enhanced Contact Management System for Communitas v2

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

/// Contact profile with metadata and four-word address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactProfile {
    pub id: String,
    pub four_word_address: String,
    pub display_name: String,
    pub avatar_path: Option<String>,
    pub bio: Option<String>,
    pub status: ContactStatus,
    pub file_system_quota: u64,
    pub used_storage: u64,
    pub preferences: ContactPreferences,
    pub created_at: SystemTime,
    pub last_active: Option<SystemTime>,
    pub verified: bool,
}

/// Contact status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContactStatus {
    Pending,
    Active,
    Offline,
    Blocked,
    Removed,
}

/// Contact preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactPreferences {
    pub allow_file_sharing: bool,
    pub allow_calls: bool,
    pub allow_screen_sharing: bool,
    pub notifications_enabled: bool,
    pub auto_download_limit: u64,
}

impl Default for ContactPreferences {
    fn default() -> Self {
        Self {
            allow_file_sharing: true,
            allow_calls: true,
            allow_screen_sharing: false,
            notifications_enabled: true,
            auto_download_limit: 10 * 1024 * 1024,
        }
    }
}

/// Secure invitation for new contacts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInvitation {
    pub id: String,
    pub token: String,
    pub inviter_address: String,
    pub inviter_name: String,
    pub message: Option<String>,
    pub expires_at: SystemTime,
    pub qr_code: Option<String>,
    pub usage_count: u32,
    pub max_usage: u32,
    pub created_at: SystemTime,
}

impl ContactInvitation {
    pub fn generate(
        inviter_address: String,
        inviter_name: String,
        message: Option<String>,
        expires_in: Duration,
        max_usage: u32,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let token = Self::generate_secure_token();
        let expires_at = SystemTime::now() + expires_in;
        let created_at = SystemTime::now();

        Self {
            id,
            token,
            inviter_address,
            inviter_name,
            message,
            expires_at,
            qr_code: None,
            usage_count: 0,
            max_usage,
            created_at,
        }
    }

    fn generate_secure_token() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.r#gen();
        hex::encode(bytes)
    }

    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now();
        now < self.expires_at && self.usage_count < self.max_usage
    }

    pub fn generate_qr_code(&mut self) -> Result<String> {
        let invitation_data = serde_json::json!({
            "type": "communitas_invitation",
            "version": "2.0",
            "id": self.id,
            "token": self.token,
            "inviter": self.inviter_address,
            "name": self.inviter_name,
            "message": self.message,
            "expires_at": self.expires_at.duration_since(UNIX_EPOCH)?.as_secs()
        });

        let qr_data = serde_json::to_string(&invitation_data)?;
        self.qr_code = Some(qr_data.clone());
        Ok(qr_data)
    }
}

/// Contact management system
#[derive(Debug)]
pub struct ContactManager {
    contacts: Arc<RwLock<HashMap<String, ContactProfile>>>,
    invitations: Arc<RwLock<HashMap<String, ContactInvitation>>>,
    address_lookup: Arc<RwLock<HashMap<String, String>>>,
    file_system_root: PathBuf,
}

impl ContactManager {
    pub async fn new(file_system_root: PathBuf) -> Result<Self> {
        tokio::fs::create_dir_all(&file_system_root)
            .await
            .context("Failed to create file system root")?;

        Ok(Self {
            contacts: Arc::new(RwLock::new(HashMap::new())),
            invitations: Arc::new(RwLock::new(HashMap::new())),
            address_lookup: Arc::new(RwLock::new(HashMap::new())),
            file_system_root,
        })
    }

    pub async fn add_contact(
        &self,
        four_word_address: String,
        display_name: String,
        bio: Option<String>,
    ) -> Result<ContactProfile> {
        let contact_id = Uuid::new_v4().to_string();
        let contact = ContactProfile {
            id: contact_id.clone(),
            four_word_address: four_word_address.clone(),
            display_name,
            avatar_path: None,
            bio,
            status: ContactStatus::Pending,
            file_system_quota: 1024 * 1024 * 1024,
            used_storage: 0,
            preferences: ContactPreferences::default(),
            created_at: SystemTime::now(),
            last_active: None,
            verified: false,
        };

        self.create_contact_file_system(&contact_id).await?;

        let mut contacts = self.contacts.write().await;
        contacts.insert(contact_id.clone(), contact.clone());

        let mut address_lookup = self.address_lookup.write().await;
        address_lookup.insert(four_word_address, contact_id);

        info!(
            "Added new contact: {} ({})",
            contact.display_name, contact.four_word_address
        );
        Ok(contact)
    }

    async fn create_contact_file_system(&self, contact_id: &str) -> Result<()> {
        let contact_dir = self.file_system_root.join(contact_id);
        tokio::fs::create_dir_all(&contact_dir)
            .await
            .context("Failed to create contact directory")?;

        let subdirs = ["shared", "received", "temp"];
        for subdir in &subdirs {
            tokio::fs::create_dir_all(contact_dir.join(subdir))
                .await
                .context("Failed to create contact subdirectory")?;
        }

        info!("Created file system for contact: {}", contact_id);
        Ok(())
    }

    pub async fn get_contact(&self, contact_id: &str) -> Option<ContactProfile> {
        let contacts = self.contacts.read().await;
        contacts.get(contact_id).cloned()
    }

    pub async fn get_contact_by_address(&self, address: &str) -> Option<ContactProfile> {
        let address_lookup = self.address_lookup.read().await;
        if let Some(contact_id) = address_lookup.get(address) {
            let contacts = self.contacts.read().await;
            contacts.get(contact_id).cloned()
        } else {
            None
        }
    }

    pub async fn list_contacts(&self, status_filter: Option<ContactStatus>) -> Vec<ContactProfile> {
        let contacts = self.contacts.read().await;
        contacts
            .values()
            .filter(|contact| {
                status_filter
                    .as_ref()
                    .map_or(true, |s| contact.status == *s)
            })
            .cloned()
            .collect()
    }

    pub async fn create_invitation(
        &self,
        inviter_address: String,
        inviter_name: String,
        message: Option<String>,
        expires_in_hours: u64,
        max_usage: u32,
    ) -> Result<ContactInvitation> {
        let expires_in = Duration::from_secs(expires_in_hours * 3600);
        let mut invitation = ContactInvitation::generate(
            inviter_address,
            inviter_name,
            message,
            expires_in,
            max_usage,
        );

        invitation.generate_qr_code()?;

        let mut invitations = self.invitations.write().await;
        invitations.insert(invitation.id.clone(), invitation.clone());

        info!(
            "Created invitation: {} (expires in {}h)",
            invitation.id, expires_in_hours
        );
        Ok(invitation)
    }

    pub fn get_contact_file_system_path(&self, contact_id: &str) -> PathBuf {
        self.file_system_root.join(contact_id)
    }

    pub async fn search_contacts(&self, query: &str) -> Vec<ContactProfile> {
        let contacts = self.contacts.read().await;
        let query_lower = query.to_lowercase();

        contacts
            .values()
            .filter(|contact| {
                contact.display_name.to_lowercase().contains(&query_lower)
                    || contact
                        .four_word_address
                        .to_lowercase()
                        .contains(&query_lower)
                    || contact
                        .bio
                        .as_ref()
                        .map_or(false, |bio| bio.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    pub async fn accept_invitation(
        &self,
        token: &str,
        accepter_address: String,
        accepter_name: String,
    ) -> Result<ContactProfile> {
        let mut invitations = self.invitations.write().await;

        // Find invitation by token
        let invitation = invitations
            .values_mut()
            .find(|inv| inv.token == token && inv.is_valid())
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired invitation"))?;

        // Increment usage count
        invitation.usage_count += 1;

        // Create contact from invitation
        let contact = self
            .add_contact(accepter_address, accepter_name, None)
            .await?;

        info!(
            "Accepted invitation: {} -> contact: {}",
            invitation.id, contact.id
        );
        Ok(contact)
    }

    pub async fn update_contact_status(
        &self,
        contact_id: &str,
        status: ContactStatus,
    ) -> Result<()> {
        let mut contacts = self.contacts.write().await;
        if let Some(contact) = contacts.get_mut(contact_id) {
            contact.status = status.clone();
            if status == ContactStatus::Active {
                contact.last_active = Some(SystemTime::now());
            }
            info!("Updated contact {} status to {:?}", contact_id, status);
        }
        Ok(())
    }
}
