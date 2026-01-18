//! Presence service for tracking contact online status with reactive subscriptions.

use std::collections::HashMap;
use std::sync::Arc;

use communitas_ui_api::{ContactWithPresence, PresenceStatus};
use thiserror::Error;
use tokio::sync::watch;
use tracing::instrument;

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use crate::directory::DirectoryService;

/// Errors returned by the presence service.
#[derive(Debug, Error)]
pub enum PresenceError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("contact not found: {0}")]
    ContactNotFound(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// Snapshot of presence state for reactive UI updates.
#[derive(Debug, Clone, Default)]
pub struct PresenceSnapshot {
    /// Mapping from contact_id to presence status.
    pub statuses: HashMap<String, PresenceStatus>,
    /// Mapping from contact_id to last seen timestamp (Unix millis).
    pub last_seen: HashMap<String, u64>,
}

/// Service for tracking contact presence status.
pub struct PresenceService {
    auth: Arc<AuthController>,
    directory: Arc<DirectoryService>,
    tx: watch::Sender<PresenceSnapshot>,
    rx: watch::Receiver<PresenceSnapshot>,
}

impl PresenceService {
    /// Create a new presence service linked to auth and directory services.
    pub fn new(auth: Arc<AuthController>, directory: Arc<DirectoryService>) -> Self {
        let (tx, rx) = watch::channel(PresenceSnapshot::default());
        Self {
            auth,
            directory,
            tx,
            rx,
        }
    }

    /// Subscribe to presence state updates.
    pub fn subscribe(&self) -> watch::Receiver<PresenceSnapshot> {
        self.rx.clone()
    }

    /// Get the current presence snapshot without subscribing.
    pub fn current_snapshot(&self) -> PresenceSnapshot {
        self.rx.borrow().clone()
    }

    /// Get presence status for a specific contact.
    #[instrument(skip(self), name = "ui.presence.get_status", fields(contact_id))]
    pub fn get_status(&self, contact_id: &str) -> PresenceStatus {
        self.rx
            .borrow()
            .statuses
            .get(contact_id)
            .copied()
            .unwrap_or_default()
    }

    /// Get all contacts with their presence information.
    #[instrument(skip(self), name = "ui.presence.get_contacts_with_presence")]
    pub fn get_contacts_with_presence(&self) -> Vec<ContactWithPresence> {
        let dir_snap = self.directory.current_snapshot();
        let pres_snap = self.rx.borrow();

        dir_snap
            .contacts
            .into_iter()
            .map(|contact| {
                let presence = pres_snap
                    .statuses
                    .get(&contact.id)
                    .copied()
                    .unwrap_or_default();
                let last_seen = pres_snap.last_seen.get(&contact.id).copied();
                ContactWithPresence {
                    contact,
                    presence,
                    last_seen,
                }
            })
            .collect()
    }

    /// Update presence for a contact (called by core events/gossip).
    #[instrument(skip(self), name = "ui.presence.update", fields(contact_id, ?status))]
    pub fn update_presence(&self, contact_id: &str, status: PresenceStatus) {
        let mut snap = self.rx.borrow().clone();
        snap.statuses.insert(contact_id.to_string(), status);

        // Update last_seen for non-offline statuses
        if status != PresenceStatus::Offline {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            snap.last_seen.insert(contact_id.to_string(), now);
        }

        // Ignore send errors (no receivers)
        let _ = self.tx.send(snap);
    }

    /// Batch update presence for multiple contacts.
    #[instrument(skip(self, updates), name = "ui.presence.batch_update", fields(count = updates.len()))]
    pub fn batch_update_presence(&self, updates: Vec<(String, PresenceStatus)>) {
        let mut snap = self.rx.borrow().clone();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        for (contact_id, status) in updates {
            snap.statuses.insert(contact_id.clone(), status);
            if status != PresenceStatus::Offline {
                snap.last_seen.insert(contact_id, now);
            }
        }

        let _ = self.tx.send(snap);
    }

    /// Clear all presence data (e.g., on logout).
    #[instrument(skip(self), name = "ui.presence.clear")]
    pub fn clear(&self) {
        let _ = self.tx.send(PresenceSnapshot::default());
    }

    /// Check if currently authenticated.
    #[allow(dead_code)]
    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use tempfile::TempDir;

    fn make_service(temp: &TempDir) -> PresenceService {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());
        let directory = Arc::new(DirectoryService::new(auth.clone()));
        PresenceService::new(auth, directory)
    }

    #[test]
    fn presence_service_starts_empty() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let snap = service.current_snapshot();
        assert!(snap.statuses.is_empty());
        assert!(snap.last_seen.is_empty());
    }

    #[test]
    fn get_status_returns_unknown_for_unknown_contact() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let status = service.get_status("unknown-contact");
        assert_eq!(status, PresenceStatus::Unknown);
    }

    #[test]
    fn update_presence_updates_subscribers() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        service.update_presence("alice", PresenceStatus::Online);

        let snap = rx.borrow().clone();
        assert_eq!(snap.statuses.get("alice"), Some(&PresenceStatus::Online));
        assert!(snap.last_seen.contains_key("alice"));
    }

    #[test]
    fn update_presence_does_not_update_last_seen_for_offline() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        // First set to online (sets last_seen)
        service.update_presence("alice", PresenceStatus::Online);
        let last_seen_online = rx.borrow().last_seen.get("alice").copied();

        // Then set to offline (should not update last_seen)
        service.update_presence("alice", PresenceStatus::Offline);
        let last_seen_offline = rx.borrow().last_seen.get("alice").copied();

        // last_seen should remain from when they were online
        assert_eq!(last_seen_online, last_seen_offline);
    }

    #[test]
    fn batch_update_presence_updates_multiple() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        service.batch_update_presence(vec![
            ("alice".to_string(), PresenceStatus::Online),
            ("bob".to_string(), PresenceStatus::Away),
            ("charlie".to_string(), PresenceStatus::Offline),
        ]);

        let snap = rx.borrow().clone();
        assert_eq!(snap.statuses.get("alice"), Some(&PresenceStatus::Online));
        assert_eq!(snap.statuses.get("bob"), Some(&PresenceStatus::Away));
        assert_eq!(snap.statuses.get("charlie"), Some(&PresenceStatus::Offline));
    }

    #[test]
    fn clear_resets_all_presence() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        // Add some presence data
        service.update_presence("alice", PresenceStatus::Online);
        service.update_presence("bob", PresenceStatus::Away);

        // Clear
        service.clear();

        let snap = rx.borrow().clone();
        assert!(snap.statuses.is_empty());
        assert!(snap.last_seen.is_empty());
    }

    #[test]
    fn get_contacts_with_presence_merges_directory_and_presence() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);

        // Update presence for a contact
        service.update_presence("alice-id", PresenceStatus::Online);

        // Get contacts with presence (directory is empty, so result is empty)
        let contacts = service.get_contacts_with_presence();
        assert!(contacts.is_empty());

        // The test verifies the method works; full integration requires
        // populating the directory which needs authentication
    }

    #[test]
    fn subscribe_returns_receiver() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();
        let snap = rx.borrow().clone();
        assert!(snap.statuses.is_empty());
    }
}
