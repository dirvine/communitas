// Licensed under the AGPL-3.0 license - see LICENSE file for details

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

const TYPING_TIMEOUT_SECS: u64 = 5;
const PRESENCE_STALE_SECS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PresenceStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "away")]
    Away,
    #[serde(rename = "busy")]
    Busy,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "offline")]
    #[default]
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingIndicator {
    pub entity_id: String,
    pub message_preview: Option<String>,
    pub started_at: u64,
}

impl TypingIndicator {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now > self.started_at + TYPING_TIMEOUT_SECS
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub user_id: String,
    pub status: PresenceStatus,
    pub last_seen: u64,
    pub current_entity: Option<String>,
    pub typing_in: Vec<TypingIndicator>,
    pub updated_at: u64,
}

impl Presence {
    pub fn new(user_id: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            user_id,
            status: PresenceStatus::Offline,
            last_seen: now,
            current_entity: None,
            typing_in: Vec::new(),
            updated_at: now,
        }
    }

    pub fn is_stale(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now > self.updated_at + PRESENCE_STALE_SECS
    }

    fn clean_expired_typing(&mut self) {
        self.typing_in.retain(|t| !t.is_expired());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdate {
    pub status: Option<PresenceStatus>,
    pub current_entity: Option<String>,
    pub typing: Option<TypingIndicator>,
    pub clear_typing: Option<String>,
}

impl PresenceUpdate {
    pub fn status_only(status: PresenceStatus) -> Self {
        Self {
            status: Some(status),
            current_entity: None,
            typing: None,
            clear_typing: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceSubscription {
    pub entity_ids: Vec<String>,
    pub include_self: bool,
}

pub struct PresenceStore {
    presences: RwLock<HashMap<String, Presence>>,
    subscriptions: RwLock<HashMap<String, PresenceSubscription>>,
}

impl Default for PresenceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceStore {
    pub fn new() -> Self {
        Self {
            presences: RwLock::new(HashMap::new()),
            subscriptions: RwLock::new(HashMap::new()),
        }
    }

    pub async fn update_presence(&self, user_id: &str, update: PresenceUpdate) -> Presence {
        let mut presences = self.presences.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let presence = presences
            .entry(user_id.to_string())
            .or_insert_with(|| Presence::new(user_id.to_string()));

        if let Some(status) = update.status {
            presence.status = status;
        }

        if let Some(entity) = update.current_entity {
            presence.current_entity = Some(entity);
        }

        if let Some(typing) = update.typing {
            presence
                .typing_in
                .retain(|t| t.entity_id != typing.entity_id);
            presence.typing_in.push(typing);
        }

        if let Some(entity_id) = update.clear_typing {
            presence.typing_in.retain(|t| t.entity_id != entity_id);
        }

        presence.last_seen = now;
        presence.updated_at = now;
        presence.clean_expired_typing();

        presence.clone()
    }

    /// Transforms a stored presence for return: cleans expired typing and marks stale as offline
    fn prepare_presence(p: &Presence) -> Presence {
        let mut p = p.clone();
        p.clean_expired_typing();
        if p.is_stale() && p.status != PresenceStatus::Offline {
            p.status = PresenceStatus::Offline;
        }
        p
    }

    pub async fn get_users_presence(&self, user_ids: Vec<String>) -> Vec<Presence> {
        let presences = self.presences.read().await;
        user_ids
            .into_iter()
            .map(|user_id| {
                presences
                    .get(&user_id)
                    .map(Self::prepare_presence)
                    .unwrap_or_else(|| Presence::new(user_id))
            })
            .collect()
    }

    pub async fn subscribe(&self, subscription: PresenceSubscription) -> String {
        let sub_id = format!("presence_sub_{}", uuid::Uuid::new_v4());
        let mut subs = self.subscriptions.write().await;
        subs.insert(sub_id.clone(), subscription);
        sub_id
    }
}

lazy_static::lazy_static! {
    static ref GLOBAL_PRESENCE_STORE: Arc<PresenceStore> = Arc::new(PresenceStore::new());
}

pub fn get_presence_store() -> Arc<PresenceStore> {
    GLOBAL_PRESENCE_STORE.clone()
}

pub struct PresenceOperations;

impl PresenceOperations {
    pub async fn update_presence(
        _app: &communitas_core::app::CommunitasApp,
        user_id: String,
        update: PresenceUpdate,
    ) -> Result<Presence, Box<dyn std::error::Error + Send + Sync>> {
        let store = get_presence_store();
        let presence = store.update_presence(&user_id, update).await;
        tracing::debug!("Updated presence for user: {}", user_id);
        Ok(presence)
    }

    pub async fn get_users_presence(
        _app: &communitas_core::app::CommunitasApp,
        user_ids: Vec<String>,
    ) -> Result<Vec<Presence>, Box<dyn std::error::Error + Send + Sync>> {
        let store = get_presence_store();
        Ok(store.get_users_presence(user_ids).await)
    }

    pub async fn subscribe_to_presence(
        _app: &communitas_core::app::CommunitasApp,
        subscription: PresenceSubscription,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let store = get_presence_store();
        let sub_id = store.subscribe(subscription.clone()).await;
        tracing::info!(
            "Created presence subscription {} for entities: {:?}",
            sub_id,
            subscription.entity_ids
        );
        Ok(sub_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_status_serialization() {
        let status = PresenceStatus::Online;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"online\""));
    }

    #[test]
    fn test_presence_update_creation() {
        let update = PresenceUpdate::status_only(PresenceStatus::Online);
        assert!(update.status.is_some());
        assert!(update.current_entity.is_none());
        assert!(update.typing.is_none());
    }

    #[tokio::test]
    async fn test_presence_store_update() {
        let store = PresenceStore::new();

        let presence = store
            .update_presence(
                "user-1",
                PresenceUpdate::status_only(PresenceStatus::Online),
            )
            .await;

        assert_eq!(presence.user_id, "user-1");
        assert_eq!(presence.status, PresenceStatus::Online);
    }

    /// Test helper to get a single user's presence
    async fn get_single_presence(store: &PresenceStore, user_id: &str) -> Presence {
        store
            .get_users_presence(vec![user_id.to_string()])
            .await
            .into_iter()
            .next()
            .unwrap()
    }

    #[tokio::test]
    async fn test_presence_store_get() {
        let store = PresenceStore::new();

        store
            .update_presence("user-1", PresenceUpdate::status_only(PresenceStatus::Away))
            .await;

        let presence = get_single_presence(&store, "user-1").await;
        assert_eq!(presence.status, PresenceStatus::Away);
    }

    fn typing_update(entity_id: &str) -> PresenceUpdate {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        PresenceUpdate {
            status: None,
            current_entity: None,
            typing: Some(TypingIndicator {
                entity_id: entity_id.to_string(),
                message_preview: None,
                started_at: now,
            }),
            clear_typing: None,
        }
    }

    #[tokio::test]
    async fn test_typing_indicator() {
        let store = PresenceStore::new();

        store
            .update_presence("user-1", typing_update("channel-1"))
            .await;

        let presence = get_single_presence(&store, "user-1").await;
        assert_eq!(presence.typing_in.len(), 1);
        assert_eq!(presence.typing_in[0].entity_id, "channel-1");
    }

    #[tokio::test]
    async fn test_clear_typing() {
        let store = PresenceStore::new();

        store
            .update_presence("user-1", typing_update("channel-1"))
            .await;

        store
            .update_presence(
                "user-1",
                PresenceUpdate {
                    status: None,
                    current_entity: None,
                    typing: None,
                    clear_typing: Some("channel-1".to_string()),
                },
            )
            .await;

        let presence = get_single_presence(&store, "user-1").await;
        assert!(presence.typing_in.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_users_presence() {
        let store = PresenceStore::new();

        store
            .update_presence(
                "user-1",
                PresenceUpdate::status_only(PresenceStatus::Online),
            )
            .await;
        store
            .update_presence("user-2", PresenceUpdate::status_only(PresenceStatus::Busy))
            .await;

        let presences = store
            .get_users_presence(vec!["user-1".to_string(), "user-2".to_string()])
            .await;

        assert_eq!(presences.len(), 2);
        assert_eq!(presences[0].status, PresenceStatus::Online);
        assert_eq!(presences[1].status, PresenceStatus::Busy);
    }
}
