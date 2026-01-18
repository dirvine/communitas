use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{RwLock, watch};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NavigationPersistence {
    recent_entities: Vec<EntityNavigationKey>,
    recent_contacts: Vec<String>,
    starred_entities: Vec<EntityNavigationKey>,
    starred_contacts: Vec<String>,
    #[serde(default)]
    last_updated_epoch_ms: u128,
}

impl Default for NavigationPersistence {
    fn default() -> Self {
        Self {
            recent_entities: Vec::new(),
            recent_contacts: Vec::new(),
            starred_entities: Vec::new(),
            starred_contacts: Vec::new(),
            last_updated_epoch_ms: 0,
        }
    }
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
}

#[async_trait]
impl NavigationService for NavigationStore {
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
