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
        let api = self
            .auth
            .api_async()
            .await
            .ok_or(DirectoryError::NotAuthenticated)?;
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

    /// Async-compatible snapshot (safe to call from within async context).
    pub async fn snapshot(&self) -> DirectorySnapshot {
        self.inner.read().await.clone()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use tempfile::TempDir;

    fn make_directory_service(temp: &TempDir) -> DirectoryService {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());
        DirectoryService::new(auth)
    }

    #[test]
    fn directory_service_creates_with_empty_snapshot() {
        let temp = TempDir::new().unwrap();
        let service = make_directory_service(&temp);
        let snap = service.current_snapshot();
        assert!(snap.identity.is_none());
        assert!(snap.entities.is_empty());
        assert!(snap.contacts.is_empty());
    }

    #[test]
    fn subscribe_returns_empty_snapshot_initially() {
        let temp = TempDir::new().unwrap();
        let service = make_directory_service(&temp);
        let rx = service.subscribe();

        let snap = rx.borrow().clone();
        assert!(snap.identity.is_none());
        assert!(snap.entities.is_empty());
    }

    #[test]
    fn resolve_org_category_community_keywords() {
        assert_eq!(
            resolve_org_category("Rust Community", ""),
            OrganizationCategory::Community
        );
        assert_eq!(
            resolve_org_category("Acme", "nonprofit org"),
            OrganizationCategory::Community
        );
        assert_eq!(
            resolve_org_category("Mozilla Foundation", ""),
            OrganizationCategory::Community
        );
        assert_eq!(
            resolve_org_category("Dev Collective", ""),
            OrganizationCategory::Community
        );
        assert_eq!(
            resolve_org_category("Non-Profit Helpers", ""),
            OrganizationCategory::Community
        );
    }

    #[test]
    fn resolve_org_category_default_organization() {
        assert_eq!(
            resolve_org_category("Acme Corp", "A software company"),
            OrganizationCategory::Organization
        );
        assert_eq!(
            resolve_org_category("Startup Inc", ""),
            OrganizationCategory::Organization
        );
    }

    fn make_entity(
        id: &str,
        entity_type: UiEntityType,
        name: &str,
        description: Option<&str>,
        member_count: u64,
        parent_org_id: Option<&str>,
    ) -> UiEntity {
        UiEntity {
            id: id.to_string(),
            entity_type,
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            member_count,
            parent_org_id: parent_org_id.map(|s| s.to_string()),
            is_local_only: false,
            created_at: 1704067200, // 2024-01-01 00:00:00 UTC
            created_by: "test-user".to_string(),
            network_four_words: None,
        }
    }

    fn make_contact(
        id: &str,
        display_name: &str,
        four_words: Option<&str>,
        is_online: bool,
    ) -> UiContact {
        UiContact {
            id: id.to_string(),
            display_name: display_name.to_string(),
            four_words: four_words.map(|s| s.to_string()),
            is_online,
            is_favourite: false,
            last_seen: None,
            created_at: 1704067200, // 2024-01-01 00:00:00 UTC
        }
    }

    #[test]
    fn map_entity_organization_with_category() {
        let entity = make_entity(
            "org-1",
            UiEntityType::Organisation,
            "Dev Community",
            Some("A developer community"),
            100,
            None,
        );

        let unified = DirectoryService::map_entity(entity);
        assert_eq!(unified.id, "org-1");
        assert_eq!(unified.entity_type, UnifiedEntityType::Organization);
        assert_eq!(unified.name, "Dev Community");
        assert_eq!(unified.category, Some(OrganizationCategory::Community));
    }

    #[test]
    fn map_entity_project_no_category() {
        let entity = make_entity(
            "proj-1",
            UiEntityType::Project,
            "My Project",
            None,
            5,
            Some("org-1"),
        );

        let unified = DirectoryService::map_entity(entity);
        assert_eq!(unified.entity_type, UnifiedEntityType::Project);
        assert_eq!(unified.parent_id, Some("org-1".to_string()));
        assert_eq!(unified.category, None);
    }

    #[test]
    fn map_entity_types() {
        let types = [
            (UiEntityType::Organisation, UnifiedEntityType::Organization),
            (UiEntityType::Project, UnifiedEntityType::Project),
            (UiEntityType::Group, UnifiedEntityType::Group),
            (UiEntityType::Channel, UnifiedEntityType::Channel),
            (UiEntityType::Person, UnifiedEntityType::Person),
        ];

        for (ui_type, unified_type) in types {
            let entity = make_entity("test", ui_type, "Test", None, 0, None);
            let result = DirectoryService::map_entity(entity);
            assert_eq!(result.entity_type, unified_type);
        }
    }

    #[test]
    fn map_contact_with_display_name() {
        let contact = make_contact("contact-1", "Alice", Some("alpha-beta-gamma-delta"), true);

        let unified = DirectoryService::map_contact(contact);
        assert_eq!(unified.id, "contact-1");
        assert_eq!(unified.display_name, "Alice");
        assert_eq!(unified.status, "online");
    }

    #[test]
    fn map_contact_fallback_to_four_words() {
        let contact = make_contact("contact-2", "", Some("one-two-three-four"), false);

        let unified = DirectoryService::map_contact(contact);
        assert_eq!(unified.display_name, "one-two-three-four");
        assert_eq!(unified.status, "offline");
    }

    #[test]
    fn map_contact_fallback_to_id() {
        let contact = make_contact("contact-3", "", None, false);

        let unified = DirectoryService::map_contact(contact);
        assert_eq!(unified.display_name, "contact-3");
    }

    // Phase 2.2: Directory watcher / auth integration tests

    #[tokio::test]
    async fn refresh_all_fails_when_not_authenticated() {
        let temp = TempDir::new().unwrap();
        let service = make_directory_service(&temp);

        // Auth is not logged in, so refresh_all should fail with NotAuthenticated
        let result = service.refresh_all().await;
        assert!(result.is_err());

        match result.unwrap_err() {
            DirectoryError::NotAuthenticated => {} // expected
            other => panic!("expected NotAuthenticated, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn refresh_all_does_not_modify_snapshot_on_auth_error() {
        let temp = TempDir::new().unwrap();
        let service = make_directory_service(&temp);

        // Get initial empty snapshot
        let initial = service.snapshot().await;
        assert!(initial.identity.is_none());

        // Attempt refresh (will fail)
        let _ = service.refresh_all().await;

        // Snapshot should remain unchanged
        let after = service.snapshot().await;
        assert!(after.identity.is_none());
        assert!(after.entities.is_empty());
        assert!(after.contacts.is_empty());
    }

    #[tokio::test]
    async fn subscriber_not_notified_on_failed_refresh() {
        let temp = TempDir::new().unwrap();
        let service = make_directory_service(&temp);
        let mut rx = service.subscribe();

        // Initial state - use borrow_and_update to mark as seen
        let initial = rx.borrow_and_update().clone();
        assert!(initial.identity.is_none());

        // Attempt refresh (will fail due to no auth)
        let _ = service.refresh_all().await;

        // Should NOT have changed since refresh failed before publishing
        assert!(!rx.has_changed().unwrap_or(false));
    }

    #[test]
    fn auth_api_returns_none_when_not_logged_in() {
        let temp = TempDir::new().unwrap();
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let auth = Arc::new(AuthController::new(storage).unwrap());

        // AuthController.api() should return None when not authenticated
        assert!(auth.api().is_none());
    }
}
