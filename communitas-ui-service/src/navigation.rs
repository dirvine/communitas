use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{RwLock, watch};
use tracing::instrument;

use crate::storage::{JsonFile, StorageError, UiStorage};

const MAX_RECENT: usize = 20;

/// Structured navigation snapshot replicated across UI surfaces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavigationStateSnapshot {
    pub recent_entities: Vec<EntityNavigationKey>,
    pub recent_contacts: Vec<String>,
    pub starred_entities: Vec<EntityNavigationKey>,
    pub starred_contacts: Vec<String>,
}

/// Unique identifier for an entity within navigation state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityNavigationKey {
    pub entity_type: String,
    pub entity_id: String,
}

impl EntityNavigationKey {
    pub fn new(entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        }
    }

    pub fn as_composite(&self) -> String {
        format!("{}:{}", self.entity_type, self.entity_id)
    }
}

#[derive(Debug, Error)]
pub enum NavigationError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
}

#[async_trait]
pub trait NavigationService: Send + Sync {
    async fn record_entity(&self, key: EntityNavigationKey) -> Result<(), NavigationError>;
    async fn record_contact(&self, contact_id: String) -> Result<(), NavigationError>;
    async fn toggle_star_entity(&self, key: EntityNavigationKey) -> Result<bool, NavigationError>;
    async fn toggle_star_contact(&self, contact_id: String) -> Result<bool, NavigationError>;
    async fn clear(&self) -> Result<(), NavigationError>;
    fn subscribe(&self) -> watch::Receiver<NavigationStateSnapshot>;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct NavigationPersistence {
    recent_entities: Vec<EntityNavigationKey>,
    recent_contacts: Vec<String>,
    starred_entities: Vec<EntityNavigationKey>,
    starred_contacts: Vec<String>,
    #[serde(default)]
    last_updated_epoch_ms: u128,
}

impl NavigationPersistence {
    fn truncate(list: &mut Vec<String>) {
        if list.len() > MAX_RECENT {
            list.truncate(MAX_RECENT);
        }
    }

    fn truncate_entities(list: &mut Vec<EntityNavigationKey>) {
        if list.len() > MAX_RECENT {
            list.truncate(MAX_RECENT);
        }
    }

    fn touch(&mut self) {
        self.last_updated_epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
    }

    fn snapshot(&self) -> NavigationStateSnapshot {
        NavigationStateSnapshot {
            recent_entities: self.recent_entities.clone(),
            recent_contacts: self.recent_contacts.clone(),
            starred_entities: self.starred_entities.clone(),
            starred_contacts: self.starred_contacts.clone(),
        }
    }
}

/// Persistent navigation state manager.
pub struct NavigationStore {
    storage: UiStorage,
    inner: RwLock<NavigationPersistence>,
    tx: watch::Sender<NavigationStateSnapshot>,
}

impl NavigationStore {
    pub fn new(storage: UiStorage) -> Result<Self, NavigationError> {
        let persisted = JsonFile::<NavigationPersistence>::load(&storage.navigation_state_file())?
            .unwrap_or_default();
        let snapshot = persisted.snapshot();
        let (tx, _) = watch::channel(snapshot);
        Ok(Self {
            storage,
            inner: RwLock::new(persisted),
            tx,
        })
    }

    async fn persist(&self, inner: &NavigationPersistence) -> Result<(), NavigationError> {
        JsonFile::save(&self.storage.navigation_state_file(), inner)?;
        Ok(())
    }

    fn publish(&self, snapshot: NavigationStateSnapshot) {
        let _ = self.tx.send(snapshot);
    }

    pub fn current_snapshot(&self) -> NavigationStateSnapshot {
        self.inner.blocking_read().snapshot()
    }

    /// Async-compatible snapshot (safe to call from within async context)
    pub async fn snapshot(&self) -> NavigationStateSnapshot {
        self.inner.read().await.snapshot()
    }
}

#[async_trait]
impl NavigationService for NavigationStore {
    #[instrument(name = "ui.nav.record_entity", skip(self), fields(entity_type = %key.entity_type, entity_id = %key.entity_id))]
    async fn record_entity(&self, key: EntityNavigationKey) -> Result<(), NavigationError> {
        let mut inner = self.inner.write().await;
        inner.recent_entities.retain(|existing| existing != &key);
        inner.recent_entities.insert(0, key);
        NavigationPersistence::truncate_entities(&mut inner.recent_entities);
        inner.touch();
        let snapshot = inner.snapshot();
        self.persist(&inner).await?;
        self.publish(snapshot);
        Ok(())
    }

    #[instrument(name = "ui.nav.record_contact", skip(self), fields(contact_id = %contact_id))]
    async fn record_contact(&self, contact_id: String) -> Result<(), NavigationError> {
        let mut inner = self.inner.write().await;
        inner.recent_contacts.retain(|value| value != &contact_id);
        inner.recent_contacts.insert(0, contact_id);
        NavigationPersistence::truncate(&mut inner.recent_contacts);
        inner.touch();
        let snapshot = inner.snapshot();
        self.persist(&inner).await?;
        self.publish(snapshot);
        Ok(())
    }

    #[instrument(name = "ui.nav.toggle_star_entity", skip(self), fields(entity_type = %key.entity_type, entity_id = %key.entity_id))]
    async fn toggle_star_entity(&self, key: EntityNavigationKey) -> Result<bool, NavigationError> {
        let mut inner = self.inner.write().await;
        let mut added = false;
        if let Some(idx) = inner
            .starred_entities
            .iter()
            .position(|existing| existing == &key)
        {
            inner.starred_entities.remove(idx);
        } else {
            inner.starred_entities.push(key);
            added = true;
        }
        inner.touch();
        let snapshot = inner.snapshot();
        self.persist(&inner).await?;
        self.publish(snapshot);
        Ok(added)
    }

    #[instrument(name = "ui.nav.toggle_star_contact", skip(self), fields(contact_id = %contact_id))]
    async fn toggle_star_contact(&self, contact_id: String) -> Result<bool, NavigationError> {
        let mut inner = self.inner.write().await;
        let mut added = false;
        if let Some(idx) = inner
            .starred_contacts
            .iter()
            .position(|existing| existing == &contact_id)
        {
            inner.starred_contacts.remove(idx);
        } else {
            inner.starred_contacts.push(contact_id);
            added = true;
        }
        inner.touch();
        let snapshot = inner.snapshot();
        self.persist(&inner).await?;
        self.publish(snapshot);
        Ok(added)
    }

    #[instrument(name = "ui.nav.clear", skip(self))]
    async fn clear(&self) -> Result<(), NavigationError> {
        let mut inner = self.inner.write().await;
        *inner = NavigationPersistence::default();
        let snapshot = inner.snapshot();
        self.persist(&inner).await?;
        self.publish(snapshot);
        Ok(())
    }

    fn subscribe(&self) -> watch::Receiver<NavigationStateSnapshot> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store(temp: &TempDir) -> NavigationStore {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        NavigationStore::new(storage).unwrap()
    }

    fn entity(t: &str, id: &str) -> EntityNavigationKey {
        EntityNavigationKey::new(t, id)
    }

    #[test]
    fn entity_navigation_key_composite() {
        let key = entity("channel", "abc123");
        assert_eq!(key.as_composite(), "channel:abc123");
    }

    #[test]
    fn current_snapshot_outside_async() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);
        let snap = store.current_snapshot();
        assert!(snap.recent_entities.is_empty());
    }

    #[tokio::test]
    async fn record_entity_adds_to_front() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        store.record_entity(entity("channel", "ch1")).await.unwrap();
        store.record_entity(entity("group", "g1")).await.unwrap();

        let snap = store.snapshot().await;
        assert_eq!(snap.recent_entities.len(), 2);
        assert_eq!(snap.recent_entities[0], entity("group", "g1"));
        assert_eq!(snap.recent_entities[1], entity("channel", "ch1"));
    }

    #[tokio::test]
    async fn record_entity_dedups_and_moves_to_front() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        store.record_entity(entity("channel", "ch1")).await.unwrap();
        store.record_entity(entity("channel", "ch2")).await.unwrap();
        store.record_entity(entity("channel", "ch1")).await.unwrap(); // duplicate

        let snap = store.snapshot().await;
        assert_eq!(snap.recent_entities.len(), 2);
        assert_eq!(snap.recent_entities[0], entity("channel", "ch1"));
        assert_eq!(snap.recent_entities[1], entity("channel", "ch2"));
    }

    #[tokio::test]
    async fn record_entity_truncates_at_max() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        for i in 0..25 {
            store
                .record_entity(entity("channel", &format!("ch{i}")))
                .await
                .unwrap();
        }

        let snap = store.snapshot().await;
        assert_eq!(snap.recent_entities.len(), MAX_RECENT);
    }

    #[tokio::test]
    async fn record_contact_adds_to_front() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        store.record_contact("alice".to_string()).await.unwrap();
        store.record_contact("bob".to_string()).await.unwrap();

        let snap = store.snapshot().await;
        assert_eq!(snap.recent_contacts.len(), 2);
        assert_eq!(snap.recent_contacts[0], "bob");
        assert_eq!(snap.recent_contacts[1], "alice");
    }

    #[tokio::test]
    async fn record_contact_dedups_and_moves_to_front() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        store.record_contact("alice".to_string()).await.unwrap();
        store.record_contact("bob".to_string()).await.unwrap();
        store.record_contact("alice".to_string()).await.unwrap();

        let snap = store.snapshot().await;
        assert_eq!(snap.recent_contacts.len(), 2);
        assert_eq!(snap.recent_contacts[0], "alice");
        assert_eq!(snap.recent_contacts[1], "bob");
    }

    #[tokio::test]
    async fn toggle_star_entity_adds_then_removes() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        let key = entity("channel", "ch1");

        let added = store.toggle_star_entity(key.clone()).await.unwrap();
        assert!(added);
        assert_eq!(store.snapshot().await.starred_entities.len(), 1);

        let added = store.toggle_star_entity(key).await.unwrap();
        assert!(!added);
        assert_eq!(store.snapshot().await.starred_entities.len(), 0);
    }

    #[tokio::test]
    async fn toggle_star_contact_adds_then_removes() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        let added = store
            .toggle_star_contact("alice".to_string())
            .await
            .unwrap();
        assert!(added);
        assert_eq!(store.snapshot().await.starred_contacts.len(), 1);

        let added = store
            .toggle_star_contact("alice".to_string())
            .await
            .unwrap();
        assert!(!added);
        assert_eq!(store.snapshot().await.starred_contacts.len(), 0);
    }

    #[tokio::test]
    async fn clear_resets_all_state() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);

        store.record_entity(entity("channel", "ch1")).await.unwrap();
        store.record_contact("alice".to_string()).await.unwrap();
        store
            .toggle_star_entity(entity("group", "g1"))
            .await
            .unwrap();
        store.toggle_star_contact("bob".to_string()).await.unwrap();

        store.clear().await.unwrap();

        let snap = store.snapshot().await;
        assert!(snap.recent_entities.is_empty());
        assert!(snap.recent_contacts.is_empty());
        assert!(snap.starred_entities.is_empty());
        assert!(snap.starred_contacts.is_empty());
    }

    #[tokio::test]
    async fn subscribe_receives_updates() {
        let temp = TempDir::new().unwrap();
        let store = make_store(&temp);
        let mut rx = store.subscribe();

        store.record_contact("alice".to_string()).await.unwrap();
        rx.changed().await.unwrap();

        let snap = rx.borrow().clone();
        assert_eq!(snap.recent_contacts, vec!["alice".to_string()]);
    }

    #[test]
    fn persistence_roundtrip() {
        // Use blocking runtime for persistence test to avoid async/blocking_read conflict
        let temp = TempDir::new().unwrap();

        // Create runtime for first store
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let store = make_store(&temp);
            store.record_entity(entity("channel", "ch1")).await.unwrap();
            store.toggle_star_contact("bob".to_string()).await.unwrap();
        });

        // Re-open store from same storage path (outside async context)
        let store2 = make_store(&temp);
        let snap = store2.current_snapshot();
        assert_eq!(snap.recent_entities, vec![entity("channel", "ch1")]);
        assert_eq!(snap.starred_contacts, vec!["bob".to_string()]);
    }
}
