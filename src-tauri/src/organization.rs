use anyhow::Result;
use chrono::{DateTime, Utc};
use saorsa_core::Key;
use saorsa_core::dht::DHT;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============= Core Types =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub user_id: String,
    pub display_name: String,
    pub four_word_address: String,
    pub email: Option<String>,
    pub role: Role,
    pub joined_at: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub profile_image: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    Owner,
    Admin,
    Member,
    Viewer,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageQuota {
    pub allocated_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Ownership and permissions
    pub owner_id: String,
    pub members: Vec<Member>,

    // Storage and file system
    pub has_file_system: bool,
    pub storage_quota: StorageQuota,

    // Settings
    pub settings: OrganizationSettings,

    // Child entity IDs (stored separately in DHT)
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSettings {
    pub visibility: Visibility,
    pub default_member_role: Role,
    pub allow_member_invitations: bool,
    pub require_approval_for_joins: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    InviteOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Ownership and permissions
    pub creator_id: String,
    pub members: Vec<Member>,

    // Chat-only functionality
    pub has_file_system: bool,
    pub chat_settings: ChatSettings,

    // Inherited permissions from organization
    pub inherited_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSettings {
    pub message_retention_days: u32,
    pub allow_file_sharing: bool,
    pub allow_voice_messages: bool,
    pub allow_video_calls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub organization_id: String,
    pub parent_group_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Ownership and permissions
    pub owner_id: String,
    pub members: Vec<Member>,

    // Project-specific file system
    pub has_file_system: bool,
    pub storage_quota: StorageQuota,

    // Project management
    pub status: ProjectStatus,
    pub deadline: Option<DateTime<Utc>>,
    pub priority: Priority,

    // Collaboration settings
    pub settings: ProjectSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub allow_external_collaborators: bool,
    pub require_approval_for_changes: bool,
    pub version_control_enabled: bool,
    pub backup_enabled: bool,
}

// ============= Request/Response Types =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
    pub visibility: Visibility,
    pub initial_storage_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub organization_id: String,
    pub initial_members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub organization_id: String,
    pub parent_group_id: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub initial_storage_gb: f64,
    pub initial_members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationHierarchy {
    pub organization: Organization,
    pub groups: Vec<Group>,
    pub projects: Vec<Project>,
    pub total_members: usize,
    pub total_storage_used_gb: f64,
}

// ============= Organization Manager =============

#[derive(Debug)]
pub struct OrganizationManager {
    dht: Arc<RwLock<DHT>>,
    cache: Arc<RwLock<HashMap<String, Organization>>>,
}

impl OrganizationManager {
    pub fn new(dht: Arc<RwLock<DHT>>) -> Self {
        Self {
            dht,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ============= Organization Operations =============

    pub async fn create_organization(
        &self,
        request: CreateOrganizationRequest,
        owner_id: String,
    ) -> Result<Organization> {
        let org_id = format!("org_{}", uuid::Uuid::new_v4());

        let organization = Organization {
            id: org_id.clone(),
            name: request.name,
            description: request.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner_id: owner_id.clone(),
            members: vec![Member {
                user_id: owner_id.clone(),
                display_name: "Owner".to_string(),
                four_word_address: format!("owner.{}", org_id),
                email: None,
                role: Role::Owner,
                joined_at: Utc::now(),
                last_active: Some(Utc::now()),
                profile_image: None,
            }],
            has_file_system: true,
            storage_quota: StorageQuota {
                allocated_gb: request.initial_storage_gb,
                used_gb: 0.0,
                available_gb: request.initial_storage_gb,
                last_updated: Utc::now(),
            },
            settings: OrganizationSettings {
                visibility: request.visibility,
                default_member_role: Role::Member,
                allow_member_invitations: true,
                require_approval_for_joins: false,
            },
            group_ids: vec![],
            project_ids: vec![],
        };

        // Store in DHT
        self.store_organization_in_dht(&organization).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(org_id.clone(), organization.clone());

        Ok(organization)
    }

    pub async fn get_organization(&self, org_id: &str) -> Result<Option<Organization>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(org) = cache.get(org_id) {
                return Ok(Some(org.clone()));
            }
        }

        // Fetch from DHT
        let dht_key = self.organization_dht_key(org_id);
        let dht = self.dht.read().await;

        if let Some(record) = dht.get(&dht_key).await {
            let org: Organization = serde_json::from_slice(&record.value)?;

            // Update cache
            let mut cache = self.cache.write().await;
            cache.insert(org_id.to_string(), org.clone());

            Ok(Some(org))
        } else {
            Ok(None)
        }
    }

    pub async fn update_organization(&self, organization: &Organization) -> Result<()> {
        // Update DHT
        self.store_organization_in_dht(organization).await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(organization.id.clone(), organization.clone());

        Ok(())
    }

    pub async fn get_user_organizations(&self, user_id: &str) -> Result<Vec<Organization>> {
        // In production, this would query an index in DHT
        // For now, we'll scan cached organizations
        let cache = self.cache.read().await;
        let orgs: Vec<Organization> = cache
            .values()
            .filter(|org| {
                org.owner_id == user_id || org.members.iter().any(|m| m.user_id == user_id)
            })
            .cloned()
            .collect();

        Ok(orgs)
    }

    // ============= Group Operations =============

    pub async fn create_group(
        &self,
        request: CreateGroupRequest,
        creator_id: String,
    ) -> Result<Group> {
        let group_id = format!("grp_{}", uuid::Uuid::new_v4());

        // Verify organization exists and user has permission
        let org = self
            .get_organization(&request.organization_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Organization not found"))?;

        if !self.user_has_permission(&org, &creator_id, "create_group") {
            return Err(anyhow::anyhow!("Permission denied"));
        }

        let group = Group {
            id: group_id.clone(),
            name: request.name,
            description: request.description,
            organization_id: request.organization_id.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            creator_id: creator_id.clone(),
            members: vec![],
            has_file_system: false,
            chat_settings: ChatSettings {
                message_retention_days: 30,
                allow_file_sharing: true,
                allow_voice_messages: true,
                allow_video_calls: true,
            },
            inherited_permissions: true,
        };

        // Store in DHT
        self.store_group_in_dht(&group).await?;

        // Update organization with new group
        let mut updated_org = org;
        updated_org.group_ids.push(group_id.clone());
        updated_org.updated_at = Utc::now();
        self.update_organization(&updated_org).await?;

        Ok(group)
    }

    pub async fn get_group(&self, group_id: &str) -> Result<Option<Group>> {
        let dht_key = self.group_dht_key(group_id);
        let dht = self.dht.read().await;

        if let Some(record) = dht.get(&dht_key).await {
            let group: Group = serde_json::from_slice(&record.value)?;
            Ok(Some(group))
        } else {
            Ok(None)
        }
    }

    // ============= Project Operations =============

    pub async fn create_project(
        &self,
        request: CreateProjectRequest,
        owner_id: String,
    ) -> Result<Project> {
        let project_id = format!("prj_{}", uuid::Uuid::new_v4());

        // Verify organization exists and user has permission
        let org = self
            .get_organization(&request.organization_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Organization not found"))?;

        if !self.user_has_permission(&org, &owner_id, "create_project") {
            return Err(anyhow::anyhow!("Permission denied"));
        }

        let project = Project {
            id: project_id.clone(),
            name: request.name,
            description: request.description,
            organization_id: request.organization_id.clone(),
            parent_group_id: request.parent_group_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            owner_id: owner_id.clone(),
            members: vec![],
            has_file_system: true,
            storage_quota: StorageQuota {
                allocated_gb: request.initial_storage_gb,
                used_gb: 0.0,
                available_gb: request.initial_storage_gb,
                last_updated: Utc::now(),
            },
            status: ProjectStatus::Active,
            deadline: request.deadline,
            priority: request.priority,
            settings: ProjectSettings {
                allow_external_collaborators: false,
                require_approval_for_changes: false,
                version_control_enabled: true,
                backup_enabled: true,
            },
        };

        // Store in DHT
        self.store_project_in_dht(&project).await?;

        // Update organization with new project
        let mut updated_org = org;
        updated_org.project_ids.push(project_id.clone());
        updated_org.updated_at = Utc::now();
        self.update_organization(&updated_org).await?;

        Ok(project)
    }

    pub async fn get_project(&self, project_id: &str) -> Result<Option<Project>> {
        let dht_key = self.project_dht_key(project_id);
        let dht = self.dht.read().await;

        if let Some(record) = dht.get(&dht_key).await {
            let project: Project = serde_json::from_slice(&record.value)?;
            Ok(Some(project))
        } else {
            Ok(None)
        }
    }

    // ============= Hierarchy Operations =============

    pub async fn get_organization_hierarchy(&self, org_id: &str) -> Result<OrganizationHierarchy> {
        let organization = self
            .get_organization(org_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Organization not found"))?;

        // Fetch all groups
        let mut groups = Vec::new();
        for group_id in &organization.group_ids {
            if let Some(group) = self.get_group(group_id).await? {
                groups.push(group);
            }
        }

        // Fetch all projects
        let mut projects = Vec::new();
        let mut total_storage_used = 0.0;
        for project_id in &organization.project_ids {
            if let Some(project) = self.get_project(project_id).await? {
                total_storage_used += project.storage_quota.used_gb;
                projects.push(project);
            }
        }

        // Add organization's storage to total
        total_storage_used += organization.storage_quota.used_gb;

        // Calculate total members (unique across all entities)
        let mut unique_members = std::collections::HashSet::new();
        for member in &organization.members {
            unique_members.insert(member.user_id.clone());
        }
        for group in &groups {
            for member in &group.members {
                unique_members.insert(member.user_id.clone());
            }
        }
        for project in &projects {
            for member in &project.members {
                unique_members.insert(member.user_id.clone());
            }
        }

        Ok(OrganizationHierarchy {
            organization,
            groups,
            projects,
            total_members: unique_members.len(),
            total_storage_used_gb: total_storage_used,
        })
    }

    // ============= DHT Storage Helpers =============

    async fn store_organization_in_dht(&self, org: &Organization) -> Result<()> {
        let dht_key = self.organization_dht_key(&org.id);
        let data = serde_json::to_vec(org)?;
        let dht = self.dht.write().await;
        dht.put(dht_key, data).await?;
        Ok(())
    }

    async fn store_group_in_dht(&self, group: &Group) -> Result<()> {
        let dht_key = self.group_dht_key(&group.id);
        let data = serde_json::to_vec(group)?;
        let dht = self.dht.write().await;
        dht.put(dht_key, data).await?;
        Ok(())
    }

    async fn store_project_in_dht(&self, project: &Project) -> Result<()> {
        let dht_key = self.project_dht_key(&project.id);
        let data = serde_json::to_vec(project)?;
        let dht = self.dht.write().await;
        dht.put(dht_key, data).await?;
        Ok(())
    }

    fn organization_dht_key(&self, org_id: &str) -> Key {
        Key::new(format!("organization:{}", org_id).as_bytes())
    }

    fn group_dht_key(&self, group_id: &str) -> Key {
        Key::new(format!("group:{}", group_id).as_bytes())
    }

    fn project_dht_key(&self, project_id: &str) -> Key {
        Key::new(format!("project:{}", project_id).as_bytes())
    }

    // ============= Permission Helpers =============

    fn user_has_permission(&self, org: &Organization, user_id: &str, _permission: &str) -> bool {
        // For now, check if user is owner or admin
        org.members
            .iter()
            .any(|m| m.user_id == user_id && (m.role == Role::Owner || m.role == Role::Admin))
    }
}

// ============= Call Management =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallRequest {
    pub entity_type: EntityType,
    pub entity_id: String,
    pub call_type: CallType,
    pub initiator_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Organization,
    Project,
    Group,
    Individual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallType {
    Voice,
    Video,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallSession {
    pub id: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub call_type: CallType,
    pub participants: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub webrtc_room_id: String,
}

impl OrganizationManager {
    pub async fn initiate_call(&self, request: CallRequest) -> Result<CallSession> {
        let session_id = format!("call_{}", uuid::Uuid::new_v4());
        let webrtc_room_id = format!("room_{}", uuid::Uuid::new_v4());

        // Get participants based on entity type
        let participants = match request.entity_type {
            EntityType::Organization => {
                if let Some(org) = self.get_organization(&request.entity_id).await? {
                    org.members.iter().map(|m| m.user_id.clone()).collect()
                } else {
                    vec![request.initiator_id.clone()]
                }
            }
            EntityType::Project => {
                if let Some(project) = self.get_project(&request.entity_id).await? {
                    project.members.iter().map(|m| m.user_id.clone()).collect()
                } else {
                    vec![request.initiator_id.clone()]
                }
            }
            EntityType::Group => {
                if let Some(group) = self.get_group(&request.entity_id).await? {
                    group.members.iter().map(|m| m.user_id.clone()).collect()
                } else {
                    vec![request.initiator_id.clone()]
                }
            }
            EntityType::Individual => {
                vec![request.initiator_id.clone(), request.entity_id.clone()]
            }
        };

        let session = CallSession {
            id: session_id,
            entity_type: request.entity_type,
            entity_id: request.entity_id,
            call_type: request.call_type,
            participants,
            started_at: Utc::now(),
            webrtc_room_id,
        };

        // Store call session in DHT for coordination
        let dht_key = Key::new(format!("call:{}", session.id).as_bytes());
        let data = serde_json::to_vec(&session)?;
        let dht = self.dht.write().await;
        dht.put(dht_key, data).await?;

        Ok(session)
    }
}
