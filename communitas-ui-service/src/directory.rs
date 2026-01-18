use std::sync::Arc;

use communitas_core::ui_core::{CommunitasApi, UiContact, UiEntity, UiEntityType};
use communitas_ui_api::{
    OrganizationCategory, UnifiedContact, UnifiedEntity, UnifiedEntityType, UnifiedIdentity,
};
use thiserror::Error;
use tokio::sync::{RwLock, watch};

use crate::auth::{AuthController, AuthError};

/// Aggregate snapshot for identity, entities, and contacts.
#[derive(Debug, Clone, Default)]
pub struct DirectorySnapshot {
    pub identity: Option<UnifiedIdentity>,
    pub entities: Vec<UnifiedEntity>,
    pub contacts: Vec<UnifiedContact>,
}

#[derive(Debug, Error)]
pub enum DirectoryError {
    #[error("not authenticated")]
    NotAuthenticated,
    #[error("core error: {0}")]
    Core(String),
    #[error("auth error: {0}")]
    Auth(#[from] AuthError),
}

impl From<String> for DirectoryError {
    fn from(value: String) -> Self {
        DirectoryError::Core(value)
    }
}

/// Directory service backed by Communitas core.
pub struct DirectoryService {
    auth: Arc<AuthController>,
    inner: RwLock<DirectorySnapshot>,
    tx: watch::Sender<DirectorySnapshot>,
}

impl DirectoryService {
    pub fn new(auth: Arc<AuthController>) -> Self {
        let initial = DirectorySnapshot::default();
        let (tx, _) = watch::channel(initial.clone());
        Self {
            auth,
            inner: RwLock::new(initial),
            tx,
        }
    }

    async fn fetch_snapshot(
        &self,
        api: CommunitasApi,
    ) -> Result<DirectorySnapshot, DirectoryError> {
        let (profile, entities, contacts) =
            tokio::try_join!(api.get_profile(), api.entity_list(), api.contacts_list())?;

        let identity = UnifiedIdentity {
            display_name: profile.display_name,
            four_words: profile.four_words,
        };

        let mapped_entities = entities
            .into_iter()
            .map(Self::map_entity)
            .collect::<Vec<_>>();

        let mapped_contacts = contacts
            .into_iter()
            .map(Self::map_contact)
            .collect::<Vec<_>>();

        Ok(DirectorySnapshot {
            identity: Some(identity),
            entities: mapped_entities,
            contacts: mapped_contacts,
        })
    }

    fn map_entity(entity: UiEntity) -> UnifiedEntity {
        let entity_type = match entity.entity_type {
            UiEntityType::Organisation => UnifiedEntityType::Organization,
            UiEntityType::Project => UnifiedEntityType::Project,
            UiEntityType::Group => UnifiedEntityType::Group,
            UiEntityType::Channel => UnifiedEntityType::Channel,
            UiEntityType::Person => UnifiedEntityType::Person,
        };

        let category = if matches!(entity_type, UnifiedEntityType::Organization) {
            Some(resolve_org_category(
                &entity.name,
                entity.description.as_deref().unwrap_or_default(),
            ))
        } else {
            None
        };

        UnifiedEntity {
            id: entity.id,
            entity_type,
            name: entity.name,
            description: entity.description.unwrap_or_default(),
            member_count: entity.member_count,
            parent_id: entity.parent_org_id,
            category,
        }
    }

    fn map_contact(contact: UiContact) -> UnifiedContact {
        let UiContact {
            id,
            display_name,
            four_words,
            is_online,
            ..
        } = contact;

        let resolved_name = if display_name.is_empty() {
            four_words.unwrap_or_else(|| id.clone())
        } else {
            display_name
        };

        UnifiedContact {
            id,
            display_name: resolved_name,
            status: if is_online {
                "online".to_string()
            } else {
                "offline".to_string()
            },
        }
    }

    pub async fn refresh_all(&self) -> Result<(), DirectoryError> {
        let api = self.auth.api().ok_or(DirectoryError::NotAuthenticated)?;
        let snapshot = self.fetch_snapshot(api).await?;
        {
            let mut inner = self.inner.write().await;
            *inner = snapshot.clone();
        }
        let _ = self.tx.send(snapshot);
        Ok(())
    }

    pub fn subscribe(&self) -> watch::Receiver<DirectorySnapshot> {
        self.tx.subscribe()
    }

    pub fn current_snapshot(&self) -> DirectorySnapshot {
        self.inner.blocking_read().clone()
    }
}

fn resolve_org_category(name: &str, description: &str) -> OrganizationCategory {
    let combined = format!("{name} {description}").to_lowercase();
    if combined.contains("community")
        || combined.contains("collective")
        || combined.contains("nonprofit")
        || combined.contains("non-profit")
        || combined.contains("foundation")
    {
        OrganizationCategory::Community
    } else {
        OrganizationCategory::Organization
    }
}
