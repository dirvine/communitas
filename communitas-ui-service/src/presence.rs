// SPDX-License-Identifier: MIT OR Apache-2.0

//! Presence service for tracking contact online status with reactive subscriptions.

use std::collections::HashMap;
use std::sync::Arc;

use communitas_ui_api::{ContactWithPresence, PresenceStatus};
use thiserror::Error;
use tokio::sync::watch;
use tracing::instrument;

use crate::auth::AuthController;
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
    /// Set of contact_ids that are currently in a call.
    pub in_call: HashMap<String, bool>,
    /// Mapping from contact_id to call entity name (channel/group name).
    pub call_entity_names: HashMap<String, String>,
    /// Set of contact_ids that are currently screen sharing (presenting).
    pub is_screen_sharing: HashMap<String, bool>,
}

/// Service for tracking contact presence status.
pub struct PresenceService {
    directory: Arc<DirectoryService>,
    tx: watch::Sender<PresenceSnapshot>,
    rx: watch::Receiver<PresenceSnapshot>,
}

impl PresenceService {
    /// Create a new presence service linked to auth and directory services.
    pub fn new(_auth: Arc<AuthController>, directory: Arc<DirectoryService>) -> Self {
        let (tx, rx) = watch::channel(PresenceSnapshot::default());
        Self { directory, tx, rx }
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
                let is_in_call = pres_snap.in_call.get(&contact.id).copied().unwrap_or(false);
                let call_entity_name = pres_snap.call_entity_names.get(&contact.id).cloned();
                let is_screen_sharing = pres_snap
                    .is_screen_sharing
                    .get(&contact.id)
                    .copied()
                    .unwrap_or(false);
                ContactWithPresence {
                    contact,
                    presence,
                    last_seen,
                    is_in_call,
                    call_entity_name,
                    is_screen_sharing,
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

    /// Update call status for a contact.
    #[instrument(
        skip(self),
        name = "ui.presence.update_call_status",
        fields(contact_id, is_in_call)
    )]
    pub fn update_call_status(
        &self,
        contact_id: &str,
        is_in_call: bool,
        call_entity_name: Option<String>,
    ) {
        let mut snap = self.rx.borrow().clone();

        if is_in_call {
            snap.in_call.insert(contact_id.to_string(), true);
            if let Some(name) = call_entity_name {
                snap.call_entity_names.insert(contact_id.to_string(), name);
            }
        } else {
            snap.in_call.remove(contact_id);
            snap.call_entity_names.remove(contact_id);
        }

        let _ = self.tx.send(snap);
    }

    /// Batch update call status for multiple contacts (e.g., when call participants change).
    #[instrument(skip(self, updates), name = "ui.presence.batch_update_call_status", fields(count = updates.len()))]
    pub fn batch_update_call_status(&self, updates: Vec<(String, bool, Option<String>)>) {
        let mut snap = self.rx.borrow().clone();

        for (contact_id, is_in_call, call_entity_name) in updates {
            if is_in_call {
                snap.in_call.insert(contact_id.clone(), true);
                if let Some(name) = call_entity_name {
                    snap.call_entity_names.insert(contact_id, name);
                }
            } else {
                snap.in_call.remove(&contact_id);
                snap.call_entity_names.remove(&contact_id);
            }
        }

        let _ = self.tx.send(snap);
    }

    /// Clear call status for all contacts (e.g., when leaving a call).
    #[instrument(skip(self), name = "ui.presence.clear_all_call_status")]
    pub fn clear_all_call_status(&self) {
        let mut snap = self.rx.borrow().clone();
        snap.in_call.clear();
        snap.call_entity_names.clear();
        snap.is_screen_sharing.clear();
        let _ = self.tx.send(snap);
    }

    /// Update screen sharing status for a contact.
    #[instrument(
        skip(self),
        name = "ui.presence.update_screen_sharing",
        fields(contact_id, is_sharing)
    )]
    pub fn update_screen_sharing(&self, contact_id: &str, is_sharing: bool) {
        let mut snap = self.rx.borrow().clone();

        if is_sharing {
            snap.is_screen_sharing.insert(contact_id.to_string(), true);
        } else {
            snap.is_screen_sharing.remove(contact_id);
        }

        let _ = self.tx.send(snap);
    }

    /// Batch update screen sharing status for multiple contacts.
    #[instrument(skip(self, updates), name = "ui.presence.batch_update_screen_sharing", fields(count = updates.len()))]
    pub fn batch_update_screen_sharing(&self, updates: Vec<(String, bool)>) {
        let mut snap = self.rx.borrow().clone();

        for (contact_id, is_sharing) in updates {
            if is_sharing {
                snap.is_screen_sharing.insert(contact_id, true);
            } else {
                snap.is_screen_sharing.remove(&contact_id);
            }
        }

        let _ = self.tx.send(snap);
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

    #[test]
    fn update_call_status_sets_in_call() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        service.update_call_status("alice", true, Some("Team Standup".to_string()));

        let snap = rx.borrow().clone();
        assert_eq!(snap.in_call.get("alice"), Some(&true));
        assert_eq!(
            snap.call_entity_names.get("alice"),
            Some(&"Team Standup".to_string())
        );
    }

    #[test]
    fn update_call_status_clears_when_leaving_call() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        // Join call
        service.update_call_status("alice", true, Some("Team Standup".to_string()));
        assert!(rx.borrow().in_call.contains_key("alice"));

        // Leave call
        service.update_call_status("alice", false, None);

        let snap = rx.borrow().clone();
        assert!(!snap.in_call.contains_key("alice"));
        assert!(!snap.call_entity_names.contains_key("alice"));
    }

    #[test]
    fn batch_update_call_status_updates_multiple() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        service.batch_update_call_status(vec![
            ("alice".to_string(), true, Some("Call A".to_string())),
            ("bob".to_string(), true, Some("Call B".to_string())),
            ("charlie".to_string(), false, None),
        ]);

        let snap = rx.borrow().clone();
        assert!(snap.in_call.get("alice").copied().unwrap_or(false));
        assert!(snap.in_call.get("bob").copied().unwrap_or(false));
        assert!(!snap.in_call.contains_key("charlie"));
        assert_eq!(
            snap.call_entity_names.get("alice"),
            Some(&"Call A".to_string())
        );
        assert_eq!(
            snap.call_entity_names.get("bob"),
            Some(&"Call B".to_string())
        );
    }

    #[test]
    fn clear_all_call_status_removes_all() {
        let temp = TempDir::new().unwrap();
        let service = make_service(&temp);
        let rx = service.subscribe();

        // Add some call status
        service.update_call_status("alice", true, Some("Call A".to_string()));
        service.update_call_status("bob", true, Some("Call B".to_string()));
        assert!(!rx.borrow().in_call.is_empty());

        // Clear all
        service.clear_all_call_status();

        let snap = rx.borrow().clone();
        assert!(snap.in_call.is_empty());
        assert!(snap.call_entity_names.is_empty());
    }

    #[test]
    fn presence_snapshot_default_includes_empty_call_fields() {
        let snap = PresenceSnapshot::default();
        assert!(snap.in_call.is_empty());
        assert!(snap.call_entity_names.is_empty());
    }
}
