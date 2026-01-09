//! Invite service for managing collaboration invites.
//!
//! This service handles the creation, acceptance, rejection, and revocation
//! of invites to join entities. It integrates with the permission system
//! to enforce access control and stores invites in CRDT documents for sync.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::crdt_manager::CrdtManager;
use crate::entity_service::EntityService;
use crate::invite::{Invite, InviteActionError, InviteStatus};
use crate::legacy_crdt::EntityType;
use crate::permissions::{MemberPermissions, ResourceType};

/// Request data for creating an invite.
///
/// This struct reduces the number of parameters to `create_invite`.
#[derive(Debug, Clone)]
pub struct InviteRequest {
    /// Four-word identity of the intended recipient.
    pub recipient_id: String,
    /// Type of entity to join.
    pub entity_type: EntityType,
    /// ID of the entity to join.
    pub entity_id: String,
    /// Role to grant on acceptance.
    pub role: String,
    /// Optional message to recipient.
    pub message: Option<String>,
    /// Optional expiration in hours.
    pub expires_in_hours: Option<u32>,
}

impl InviteRequest {
    /// Create a new invite request.
    pub fn new(
        recipient_id: impl Into<String>,
        entity_type: EntityType,
        entity_id: impl Into<String>,
        role: impl Into<String>,
    ) -> Self {
        Self {
            recipient_id: recipient_id.into(),
            entity_type,
            entity_id: entity_id.into(),
            role: role.into(),
            message: None,
            expires_in_hours: None,
        }
    }

    /// Set an optional message for the invite.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set expiration time in hours.
    pub fn with_expiration(mut self, hours: u32) -> Self {
        self.expires_in_hours = Some(hours);
        self
    }
}

/// Errors from invite service operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InviteServiceError {
    /// Entity not found.
    EntityNotFound(String),

    /// Member not found in entity.
    MemberNotFound {
        entity_id: String,
        member_id: String,
    },

    /// Invite not found.
    InviteNotFound(String),

    /// Permission denied for this operation.
    PermissionDenied(String),

    /// Invalid four-word identity format.
    InvalidFourWords(String),

    /// Invite has expired.
    InviteExpired,

    /// Invite already resolved (accepted, rejected, etc.).
    AlreadyResolved(InviteStatus),

    /// Cannot grant a role higher than your own.
    RoleEscalation {
        granter_role: String,
        target_role: String,
    },

    /// Recipient is already a member.
    AlreadyMember {
        entity_id: String,
        member_id: String,
    },

    /// CRDT operation failed.
    CrdtError(String),

    /// Entity service error.
    EntityServiceError(String),

    /// Invite action failed.
    InviteActionFailed(String),
}

impl std::fmt::Display for InviteServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityNotFound(id) => write!(f, "entity not found: {}", id),
            Self::MemberNotFound {
                entity_id,
                member_id,
            } => {
                write!(
                    f,
                    "member '{}' not found in entity '{}'",
                    member_id, entity_id
                )
            }
            Self::InviteNotFound(id) => write!(f, "invite not found: {}", id),
            Self::PermissionDenied(msg) => write!(f, "permission denied: {}", msg),
            Self::InvalidFourWords(id) => write!(f, "invalid four-word identity: {}", id),
            Self::InviteExpired => write!(f, "invite has expired"),
            Self::AlreadyResolved(status) => {
                write!(f, "invite already resolved with status: {}", status)
            }
            Self::RoleEscalation {
                granter_role,
                target_role,
            } => {
                write!(
                    f,
                    "cannot grant '{}' role when you have '{}' role",
                    target_role, granter_role
                )
            }
            Self::AlreadyMember {
                entity_id,
                member_id,
            } => {
                write!(f, "'{}' is already a member of '{}'", member_id, entity_id)
            }
            Self::CrdtError(msg) => write!(f, "CRDT error: {}", msg),
            Self::EntityServiceError(msg) => write!(f, "entity service error: {}", msg),
            Self::InviteActionFailed(msg) => write!(f, "invite action failed: {}", msg),
        }
    }
}

impl std::error::Error for InviteServiceError {}

impl From<InviteActionError> for InviteServiceError {
    fn from(err: InviteActionError) -> Self {
        match err {
            InviteActionError::AlreadyResolved(status) => Self::AlreadyResolved(status),
            InviteActionError::Expired => Self::InviteExpired,
            InviteActionError::NotRecipient { expected, actual } => {
                Self::PermissionDenied(format!(
                    "not the invite recipient: expected '{}', got '{}'",
                    expected, actual
                ))
            }
        }
    }
}

/// Result type for invite service operations.
pub type InviteServiceResult<T> = Result<T, InviteServiceError>;

/// Role hierarchy for permission checking.
///
/// Higher values indicate more privileges.
fn role_rank(role: &str) -> u8 {
    match role.to_lowercase().as_str() {
        "owner" => 5,
        "admin" => 4,
        "member" => 3,
        "viewer" => 2,
        "guest" => 1,
        _ => 0,
    }
}

/// Check if a granter can grant a target role.
///
/// Returns true if the granter's role is >= target role.
pub fn can_grant_role(granter_role: &str, target_role: &str) -> bool {
    role_rank(granter_role) >= role_rank(target_role)
}

/// Validate a four-word identity format.
///
/// Checks that the identity has exactly 4 words separated by dashes.
/// Does not validate against the dictionary (that's done elsewhere).
pub fn validate_four_words_format(identity: &str) -> bool {
    let parts: Vec<&str> = identity.split('-').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|w| !w.is_empty() && w.chars().all(|c| c.is_alphabetic()))
}

/// Service for managing collaboration invites.
///
/// Invites allow entity members to invite external collaborators
/// via their four-word identity. The service enforces:
/// - Permission checks (must have Edit on Members)
/// - Role hierarchy (can't grant higher role than own)
/// - CRDT persistence for offline-first sync
pub struct InviteService {
    /// CRDT manager for storing invites.
    crdt_manager: Arc<CrdtManager>,

    /// Entity service for member operations.
    entity_service: Arc<EntityService>,

    /// In-memory invite cache (indexed by invite ID).
    /// This is used for quick lookups before falling back to CRDT.
    invite_cache: RwLock<HashMap<String, Invite>>,

    /// Index of invites by recipient (for list_pending_invites).
    recipient_index: RwLock<HashMap<String, Vec<String>>>,

    /// Index of invites by entity (for list_entity_invites).
    entity_index: RwLock<HashMap<String, Vec<String>>>,
}

impl InviteService {
    /// Create a new invite service.
    pub fn new(crdt_manager: Arc<CrdtManager>, entity_service: Arc<EntityService>) -> Self {
        Self {
            crdt_manager,
            entity_service,
            invite_cache: RwLock::new(HashMap::new()),
            recipient_index: RwLock::new(HashMap::new()),
            entity_index: RwLock::new(HashMap::new()),
        }
    }

    /// Create an invite to join an entity.
    ///
    /// # Arguments
    ///
    /// * `creator_id` - Four-word identity of the invite creator
    /// * `request` - The invite request containing recipient, entity, and role details
    ///
    /// # Errors
    ///
    /// * `PermissionDenied` - Creator doesn't have Edit on Members
    /// * `RoleEscalation` - Trying to grant higher role than own
    /// * `InvalidFourWords` - Invalid recipient identity format
    /// * `AlreadyMember` - Recipient is already a member
    pub async fn create_invite(
        &self,
        creator_id: &str,
        request: InviteRequest,
    ) -> InviteServiceResult<Invite> {
        // 1. Validate recipient format
        if !validate_four_words_format(&request.recipient_id) {
            return Err(InviteServiceError::InvalidFourWords(
                request.recipient_id.clone(),
            ));
        }

        // 2. Get creator's permissions
        let creator_perms = self
            .get_member_permissions(request.entity_type, &request.entity_id, creator_id)
            .await?;

        // 3. Check creator has Edit on Members
        if !creator_perms.can_edit(ResourceType::Members) {
            return Err(InviteServiceError::PermissionDenied(
                "must have Edit access to Members to create invites".to_string(),
            ));
        }

        // 4. Check role hierarchy
        if !can_grant_role(&creator_perms.role, &request.role) {
            return Err(InviteServiceError::RoleEscalation {
                granter_role: creator_perms.role.clone(),
                target_role: request.role.clone(),
            });
        }

        // 5. Check recipient is not already a member
        if self
            .is_member(
                request.entity_type,
                &request.entity_id,
                &request.recipient_id,
            )
            .await
        {
            return Err(InviteServiceError::AlreadyMember {
                entity_id: request.entity_id.clone(),
                member_id: request.recipient_id.clone(),
            });
        }

        // 6. Create the invite
        let invite = Invite::new(
            creator_id.to_string(),
            request.recipient_id,
            request.entity_id,
            request.entity_type,
            request.role,
            request.message,
            request.expires_in_hours,
        );

        // 7. Store in CRDT and cache
        self.store_invite(&invite).await?;

        Ok(invite)
    }

    /// Accept a pending invite.
    ///
    /// This adds the recipient as a member with the specified role.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - Four-word identity of the recipient accepting
    /// * `invite_id` - ID of the invite to accept
    ///
    /// # Errors
    ///
    /// * `InviteNotFound` - Invite doesn't exist
    /// * `PermissionDenied` - Actor is not the recipient
    /// * `InviteExpired` - Invite has expired
    /// * `AlreadyResolved` - Invite already accepted/rejected/revoked
    pub async fn accept_invite(
        &self,
        recipient_id: &str,
        invite_id: &str,
    ) -> InviteServiceResult<()> {
        // 1. Get the invite
        let mut invite = self.get_invite(invite_id).await?;

        // 2. Accept (validates recipient and state)
        invite.accept(recipient_id)?;

        // 3. Add recipient as member
        self.entity_service
            .add_member(
                invite.entity_type,
                &invite.entity_id,
                recipient_id,
                &invite.role,
            )
            .await
            .map_err(|e| InviteServiceError::EntityServiceError(e.to_string()))?;

        // 4. Update invite in storage
        self.update_invite(&invite).await?;

        Ok(())
    }

    /// Reject a pending invite.
    ///
    /// # Arguments
    ///
    /// * `recipient_id` - Four-word identity of the recipient rejecting
    /// * `invite_id` - ID of the invite to reject
    ///
    /// # Errors
    ///
    /// * `InviteNotFound` - Invite doesn't exist
    /// * `PermissionDenied` - Actor is not the recipient
    /// * `AlreadyResolved` - Invite already accepted/rejected/revoked
    pub async fn reject_invite(
        &self,
        recipient_id: &str,
        invite_id: &str,
    ) -> InviteServiceResult<()> {
        let mut invite = self.get_invite(invite_id).await?;
        invite.reject(recipient_id)?;
        self.update_invite(&invite).await?;
        Ok(())
    }

    /// Revoke a pending invite.
    ///
    /// Can be done by the creator or anyone with Edit on Members.
    ///
    /// # Arguments
    ///
    /// * `revoker_id` - Four-word identity of the person revoking
    /// * `invite_id` - ID of the invite to revoke
    ///
    /// # Errors
    ///
    /// * `InviteNotFound` - Invite doesn't exist
    /// * `PermissionDenied` - Actor is not creator and lacks permissions
    /// * `AlreadyResolved` - Invite already resolved
    pub async fn revoke_invite(
        &self,
        revoker_id: &str,
        invite_id: &str,
    ) -> InviteServiceResult<()> {
        let mut invite = self.get_invite(invite_id).await?;

        // Check authorization: must be creator OR have Edit on Members
        if invite.creator_id != revoker_id {
            let perms = self
                .get_member_permissions(invite.entity_type, &invite.entity_id, revoker_id)
                .await?;

            if !perms.can_edit(ResourceType::Members) {
                return Err(InviteServiceError::PermissionDenied(
                    "only creator or admin can revoke invites".to_string(),
                ));
            }
        }

        invite.revoke(revoker_id)?;
        self.update_invite(&invite).await?;
        Ok(())
    }

    /// List pending invites for a recipient.
    ///
    /// Returns all invites where the recipient matches and status is Pending.
    pub async fn list_pending_invites(
        &self,
        recipient_id: &str,
    ) -> InviteServiceResult<Vec<Invite>> {
        self.list_invites_for_recipient(recipient_id, Some(InviteStatus::Pending))
            .await
    }

    /// List all invites for a recipient (including resolved).
    pub async fn list_all_invites_for_recipient(
        &self,
        recipient_id: &str,
    ) -> InviteServiceResult<Vec<Invite>> {
        self.list_invites_for_recipient(recipient_id, None).await
    }

    /// List invites for an entity (admin view).
    ///
    /// Requires at least ReadOnly on Members to view.
    ///
    /// # Arguments
    ///
    /// * `requester_id` - Four-word identity of the person requesting
    /// * `entity_type` - Type of entity
    /// * `entity_id` - ID of the entity
    ///
    /// # Errors
    ///
    /// * `PermissionDenied` - Requester lacks access to Members
    pub async fn list_entity_invites(
        &self,
        requester_id: &str,
        entity_type: EntityType,
        entity_id: &str,
    ) -> InviteServiceResult<Vec<Invite>> {
        // Check permissions
        let perms = self
            .get_member_permissions(entity_type, entity_id, requester_id)
            .await?;

        if !perms.can_view(ResourceType::Members) {
            return Err(InviteServiceError::PermissionDenied(
                "must have access to Members to view invites".to_string(),
            ));
        }

        self.get_invites_for_entity(entity_type, entity_id).await
    }

    /// Get a single invite by ID.
    pub async fn get_invite(&self, invite_id: &str) -> InviteServiceResult<Invite> {
        // Check cache first
        {
            let cache = self
                .invite_cache
                .read()
                .map_err(|_| InviteServiceError::CrdtError("cache lock poisoned".to_string()))?;
            if let Some(invite) = cache.get(invite_id) {
                return Ok(invite.clone());
            }
        }

        // Load from CRDT
        self.load_invite_from_crdt(invite_id).await
    }

    // ========================================
    // Private helper methods
    // ========================================

    /// Get member permissions for an entity.
    async fn get_member_permissions(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        member_id: &str,
    ) -> InviteServiceResult<MemberPermissions> {
        // Get role from entity service
        let role = self
            .entity_service
            .get_member_role(entity_type, entity_id, member_id)
            .await
            .map_err(|e| {
                // Check if it's a "member not found" error
                let err_str = e.to_string();
                if err_str.contains("not found") || err_str.contains("not a member") {
                    InviteServiceError::MemberNotFound {
                        entity_id: entity_id.to_string(),
                        member_id: member_id.to_string(),
                    }
                } else {
                    InviteServiceError::EntityServiceError(err_str)
                }
            })?;

        // Get permission overrides
        let overrides = self
            .entity_service
            .get_permission_overrides(entity_type, entity_id, member_id)
            .await
            .unwrap_or_default();

        // Build MemberPermissions
        let mut perms = MemberPermissions::new(member_id.to_string(), role);
        for (resource_str, level_str) in overrides {
            if let (Ok(resource), Ok(level)) = (resource_str.parse(), level_str.parse()) {
                perms.overrides.insert(resource, level);
            }
        }

        Ok(perms)
    }

    /// Check if someone is already a member of an entity.
    async fn is_member(&self, entity_type: EntityType, entity_id: &str, member_id: &str) -> bool {
        self.entity_service
            .get_member_role(entity_type, entity_id, member_id)
            .await
            .is_ok()
    }

    /// Store an invite in CRDT and update indices.
    async fn store_invite(&self, invite: &Invite) -> InviteServiceResult<()> {
        // Store in cache
        {
            let mut cache = self
                .invite_cache
                .write()
                .map_err(|_| InviteServiceError::CrdtError("cache lock poisoned".to_string()))?;
            cache.insert(invite.id.clone(), invite.clone());
        }

        // Update recipient index
        {
            let mut index = self
                .recipient_index
                .write()
                .map_err(|_| InviteServiceError::CrdtError("index lock poisoned".to_string()))?;
            index
                .entry(invite.recipient_id.clone())
                .or_default()
                .push(invite.id.clone());
        }

        // Update entity index
        {
            let entity_key = format!("{:?}:{}", invite.entity_type, invite.entity_id);
            let mut index = self
                .entity_index
                .write()
                .map_err(|_| InviteServiceError::CrdtError("index lock poisoned".to_string()))?;
            index.entry(entity_key).or_default().push(invite.id.clone());
        }

        // Store in CRDT
        self.store_invite_in_crdt(invite).await
    }

    /// Update an invite in storage.
    async fn update_invite(&self, invite: &Invite) -> InviteServiceResult<()> {
        // Update cache
        {
            let mut cache = self
                .invite_cache
                .write()
                .map_err(|_| InviteServiceError::CrdtError("cache lock poisoned".to_string()))?;
            cache.insert(invite.id.clone(), invite.clone());
        }

        // Update CRDT
        self.store_invite_in_crdt(invite).await
    }

    /// Store invite in CRDT document.
    ///
    /// Stores the invite in a per-entity CRDT document using the format:
    /// `{entity_type}:{entity_id}:invites`
    ///
    /// Also updates the global index mapping invite_id -> entity_doc_id
    /// for efficient lookup by invite ID alone.
    ///
    /// The invite is stored as a JSON blob in a YMap keyed by invite_id.
    async fn store_invite_in_crdt(&self, invite: &Invite) -> InviteServiceResult<()> {
        use yrs::{Doc, Map, Transact};

        let doc_id = self.entity_invite_doc_id(invite.entity_type, &invite.entity_id);
        let entity_type_str = format!("{:?}", invite.entity_type).to_lowercase();

        // Load existing doc or create new one
        let doc = match self.crdt_manager.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(crate::crdt_manager::CrdtError::DocumentNotFound(_)) => Doc::new(),
            Err(e) => return Err(InviteServiceError::CrdtError(e.to_string())),
        };

        // Serialize invite to JSON
        let invite_json = serde_json::to_string(invite).map_err(|e| {
            InviteServiceError::CrdtError(format!("Failed to serialize invite: {}", e))
        })?;

        // Store in YMap
        {
            let invites_map = doc.get_or_insert_map("invites");
            let mut txn = doc.transact_mut();
            invites_map.insert(&mut txn, invite.id.clone(), invite_json);
        }

        // Save the entity document
        self.crdt_manager
            .save_document(&doc_id, &entity_type_str, &invite.entity_id, &doc)
            .await
            .map_err(|e| InviteServiceError::CrdtError(e.to_string()))?;

        // Update global index for invite_id -> entity_doc_id lookup
        self.update_global_invite_index(&invite.id, &doc_id).await?;

        Ok(())
    }

    /// Update the global invite index with a mapping from invite_id to entity_doc_id.
    async fn update_global_invite_index(
        &self,
        invite_id: &str,
        entity_doc_id: &str,
    ) -> InviteServiceResult<()> {
        use yrs::{Doc, Map, Transact};

        let index_doc_id = self.global_invite_index_doc_id();

        // Load existing index doc or create new one
        let index_doc = match self.crdt_manager.load_document(&index_doc_id).await {
            Ok(doc) => doc,
            Err(crate::crdt_manager::CrdtError::DocumentNotFound(_)) => Doc::new(),
            Err(e) => return Err(InviteServiceError::CrdtError(e.to_string())),
        };

        // Store mapping from invite_id to entity_doc_id
        {
            let index_map = index_doc.get_or_insert_map("invite_to_entity");
            let mut txn = index_doc.transact_mut();
            index_map.insert(&mut txn, invite_id, entity_doc_id);
        }

        // Save the global index document
        // Format: invites:global:index
        self.crdt_manager
            .save_document(&index_doc_id, "invites", "global", &index_doc)
            .await
            .map_err(|e| InviteServiceError::CrdtError(e.to_string()))?;

        Ok(())
    }

    /// Load invite from CRDT.
    ///
    /// Loads an invite by searching for it in entity documents.
    /// Uses a global invite index for efficient lookup.
    async fn load_invite_from_crdt(&self, invite_id: &str) -> InviteServiceResult<Invite> {
        use yrs::{Map, Transact};

        // First check in-memory cache
        {
            let cache = self
                .invite_cache
                .read()
                .map_err(|_| InviteServiceError::CrdtError("cache lock poisoned".to_string()))?;
            if let Some(invite) = cache.get(invite_id) {
                return Ok(invite.clone());
            }
        }

        // Load from global invite index
        let index_doc_id = self.global_invite_index_doc_id();
        let index_doc = match self.crdt_manager.load_document(&index_doc_id).await {
            Ok(doc) => doc,
            Err(crate::crdt_manager::CrdtError::DocumentNotFound(_)) => {
                return Err(InviteServiceError::InviteNotFound(invite_id.to_string()));
            }
            Err(e) => return Err(InviteServiceError::CrdtError(e.to_string())),
        };

        // Find the entity doc containing this invite
        let entity_doc_id = {
            let index_map = index_doc.get_or_insert_map("invite_to_entity");
            let txn = index_doc.transact();
            match index_map.get(&txn, invite_id) {
                Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
                _ => return Err(InviteServiceError::InviteNotFound(invite_id.to_string())),
            }
        };

        // Load the entity document
        let entity_doc = self
            .crdt_manager
            .load_document(&entity_doc_id)
            .await
            .map_err(|e| InviteServiceError::CrdtError(e.to_string()))?;

        // Get the invite from the entity document
        let invites_map = entity_doc.get_or_insert_map("invites");
        let txn = entity_doc.transact();
        let invite_json = match invites_map.get(&txn, invite_id) {
            Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
            _ => return Err(InviteServiceError::InviteNotFound(invite_id.to_string())),
        };

        let invite: Invite = serde_json::from_str(&invite_json).map_err(|e| {
            InviteServiceError::CrdtError(format!("Failed to deserialize invite: {}", e))
        })?;

        Ok(invite)
    }

    /// Load all invites for an entity from CRDT.
    ///
    /// This is currently only used in tests but will be used when we integrate
    /// CRDT-based storage with the list_entity_invites public API.
    #[allow(dead_code)]
    async fn load_invites_for_entity_from_crdt(
        &self,
        entity_type: EntityType,
        entity_id: &str,
    ) -> InviteServiceResult<Vec<Invite>> {
        use yrs::{Map, Transact};

        let doc_id = self.entity_invite_doc_id(entity_type, entity_id);

        // Load the entity document
        let doc = match self.crdt_manager.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(crate::crdt_manager::CrdtError::DocumentNotFound(_)) => {
                return Ok(Vec::new()); // No invites for this entity
            }
            Err(e) => return Err(InviteServiceError::CrdtError(e.to_string())),
        };

        // Get all invites from the document
        let invites_map = doc.get_or_insert_map("invites");
        let txn = doc.transact();

        let mut invites = Vec::new();
        for (key, value) in invites_map.iter(&txn) {
            if let yrs::Out::Any(yrs::Any::String(invite_json)) = value {
                let invite: Invite = serde_json::from_str(invite_json.as_ref()).map_err(|e| {
                    InviteServiceError::CrdtError(format!(
                        "Failed to deserialize invite {}: {}",
                        key, e
                    ))
                })?;
                invites.push(invite);
            }
        }

        Ok(invites)
    }

    /// Store recipient index in CRDT.
    ///
    /// The recipient index maps recipient IDs to lists of invite IDs.
    /// Used to quickly look up all invites for a given recipient.
    ///
    /// This is currently only used in tests but will be used when we integrate
    /// CRDT-based storage with the list_pending_invites public API.
    #[allow(dead_code)]
    async fn store_recipient_index_in_crdt(
        &self,
        recipient_id: &str,
        invite_ids: &[String],
    ) -> InviteServiceResult<()> {
        use yrs::{Doc, Map, Transact};

        let doc_id = self.recipient_index_doc_id(recipient_id);

        // Load existing doc or create new one
        let doc = match self.crdt_manager.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(crate::crdt_manager::CrdtError::DocumentNotFound(_)) => Doc::new(),
            Err(e) => return Err(InviteServiceError::CrdtError(e.to_string())),
        };

        // Serialize invite IDs to JSON array
        let invite_ids_json = serde_json::to_string(invite_ids).map_err(|e| {
            InviteServiceError::CrdtError(format!("Failed to serialize invite IDs: {}", e))
        })?;

        // Store in YMap
        {
            let index_map = doc.get_or_insert_map("recipient_invites");
            let mut txn = doc.transact_mut();
            index_map.insert(&mut txn, recipient_id, invite_ids_json);
        }

        // Save the document
        self.crdt_manager
            .save_document(&doc_id, "invites", recipient_id, &doc)
            .await
            .map_err(|e| InviteServiceError::CrdtError(e.to_string()))?;

        Ok(())
    }

    /// Load recipient index from CRDT.
    ///
    /// This is currently only used in tests but will be used when we integrate
    /// CRDT-based storage with the list_pending_invites public API.
    #[allow(dead_code)]
    async fn load_recipient_index_from_crdt(
        &self,
        recipient_id: &str,
    ) -> InviteServiceResult<Vec<String>> {
        use yrs::{Map, Transact};

        let doc_id = self.recipient_index_doc_id(recipient_id);

        // Load the document
        let doc = match self.crdt_manager.load_document(&doc_id).await {
            Ok(doc) => doc,
            Err(crate::crdt_manager::CrdtError::DocumentNotFound(_)) => {
                return Ok(Vec::new()); // No index for this recipient
            }
            Err(e) => return Err(InviteServiceError::CrdtError(e.to_string())),
        };

        // Get the invite IDs
        let index_map = doc.get_or_insert_map("recipient_invites");
        let txn = doc.transact();

        let invite_ids_json = match index_map.get(&txn, recipient_id) {
            Some(yrs::Out::Any(yrs::Any::String(s))) => s.to_string(),
            _ => return Ok(Vec::new()),
        };

        let invite_ids: Vec<String> = serde_json::from_str(&invite_ids_json).map_err(|e| {
            InviteServiceError::CrdtError(format!("Failed to deserialize invite IDs: {}", e))
        })?;

        Ok(invite_ids)
    }

    /// Generate doc ID for entity invite document.
    ///
    /// Format: `{entity_type}:{entity_id}:invites`
    /// e.g., `organisation:org-123:invites`, `group:grp-456:invites`
    fn entity_invite_doc_id(&self, entity_type: EntityType, entity_id: &str) -> String {
        let entity_type_str = format!("{:?}", entity_type).to_lowercase();
        format!("{}:{}:invites", entity_type_str, entity_id)
    }

    /// Generate doc ID for recipient index document.
    ///
    /// Format: `invites:{recipient_id}:index`
    ///
    /// This is currently only used in tests but will be used when we integrate
    /// CRDT-based storage with the list_pending_invites public API.
    #[allow(dead_code)]
    fn recipient_index_doc_id(&self, recipient_id: &str) -> String {
        format!("invites:{}:index", recipient_id)
    }

    /// Generate doc ID for global invite index.
    ///
    /// Format: `invites:global:index`
    fn global_invite_index_doc_id(&self) -> String {
        "invites:global:index".to_string()
    }

    /// List invites for a recipient with optional status filter.
    async fn list_invites_for_recipient(
        &self,
        recipient_id: &str,
        status_filter: Option<InviteStatus>,
    ) -> InviteServiceResult<Vec<Invite>> {
        let invite_ids = {
            let index = self
                .recipient_index
                .read()
                .map_err(|_| InviteServiceError::CrdtError("index lock poisoned".to_string()))?;
            index.get(recipient_id).cloned().unwrap_or_default()
        };

        let cache = self
            .invite_cache
            .read()
            .map_err(|_| InviteServiceError::CrdtError("cache lock poisoned".to_string()))?;
        let mut result = Vec::new();

        for invite_id in invite_ids {
            if let Some(invite) = cache.get(&invite_id) {
                // Apply status filter
                if let Some(filter) = status_filter {
                    if invite.status == filter {
                        // Also check if expired (for pending filter)
                        if filter == InviteStatus::Pending && invite.is_expired() {
                            continue;
                        }
                        result.push(invite.clone());
                    }
                } else {
                    result.push(invite.clone());
                }
            }
        }

        Ok(result)
    }

    /// Get invites for an entity.
    async fn get_invites_for_entity(
        &self,
        entity_type: EntityType,
        entity_id: &str,
    ) -> InviteServiceResult<Vec<Invite>> {
        let entity_key = format!("{:?}:{}", entity_type, entity_id);
        let invite_ids = {
            let index = self
                .entity_index
                .read()
                .map_err(|_| InviteServiceError::CrdtError("lock poisoned".to_string()))?;
            index.get(&entity_key).cloned().unwrap_or_default()
        };

        let cache = self
            .invite_cache
            .read()
            .map_err(|_| InviteServiceError::CrdtError("cache lock poisoned".to_string()))?;
        let mut result = Vec::new();

        for invite_id in invite_ids {
            if let Some(invite) = cache.get(&invite_id) {
                result.push(invite.clone());
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper: mock EntityService that tracks members
    // Note: This is a placeholder - full integration tests will use the real EntityService
    #[allow(dead_code)]
    struct MockEntityService {
        members: RwLock<HashMap<String, HashMap<String, String>>>, // entity_key -> member_id -> role
        permission_overrides: RwLock<HashMap<String, Vec<(String, String)>>>, // member_key -> [(resource, level)]
    }

    #[allow(dead_code)]
    impl MockEntityService {
        fn new() -> Self {
            Self {
                members: RwLock::new(HashMap::new()),
                permission_overrides: RwLock::new(HashMap::new()),
            }
        }

        fn add_member_sync(
            &self,
            entity_type: EntityType,
            entity_id: &str,
            member_id: &str,
            role: &str,
        ) {
            let entity_key = format!("{:?}:{}", entity_type, entity_id);
            let mut members = self.members.write().unwrap();
            members
                .entry(entity_key)
                .or_default()
                .insert(member_id.to_string(), role.to_string());
        }

        fn get_role(
            &self,
            entity_type: EntityType,
            entity_id: &str,
            member_id: &str,
        ) -> Option<String> {
            let entity_key = format!("{:?}:{}", entity_type, entity_id);
            let members = self.members.read().unwrap();
            members
                .get(&entity_key)
                .and_then(|m| m.get(member_id).cloned())
        }

        fn set_permission_override(
            &self,
            entity_type: EntityType,
            entity_id: &str,
            member_id: &str,
            resource: &str,
            level: &str,
        ) {
            let key = format!("{:?}:{}:{}", entity_type, entity_id, member_id);
            let mut overrides = self.permission_overrides.write().unwrap();
            overrides
                .entry(key)
                .or_default()
                .push((resource.to_string(), level.to_string()));
        }

        fn get_overrides(
            &self,
            entity_type: EntityType,
            entity_id: &str,
            member_id: &str,
        ) -> Vec<(String, String)> {
            let key = format!("{:?}:{}:{}", entity_type, entity_id, member_id);
            let overrides = self.permission_overrides.read().unwrap();
            overrides.get(&key).cloned().unwrap_or_default()
        }
    }

    // ============================================
    // Helper Function Tests
    // ============================================

    #[test]
    fn test_role_rank() {
        assert_eq!(role_rank("owner"), 5);
        assert_eq!(role_rank("admin"), 4);
        assert_eq!(role_rank("member"), 3);
        assert_eq!(role_rank("viewer"), 2);
        assert_eq!(role_rank("guest"), 1);
        assert_eq!(role_rank("unknown"), 0);
    }

    #[test]
    fn test_role_rank_case_insensitive() {
        assert_eq!(role_rank("OWNER"), 5);
        assert_eq!(role_rank("Owner"), 5);
        assert_eq!(role_rank("ADMIN"), 4);
    }

    #[test]
    fn test_can_grant_role_same_level() {
        assert!(can_grant_role("member", "member"));
        assert!(can_grant_role("admin", "admin"));
        assert!(can_grant_role("owner", "owner"));
    }

    #[test]
    fn test_can_grant_role_higher_to_lower() {
        assert!(can_grant_role("owner", "admin"));
        assert!(can_grant_role("owner", "member"));
        assert!(can_grant_role("owner", "viewer"));
        assert!(can_grant_role("owner", "guest"));
        assert!(can_grant_role("admin", "member"));
        assert!(can_grant_role("member", "viewer"));
    }

    #[test]
    fn test_can_grant_role_lower_to_higher_fails() {
        assert!(!can_grant_role("guest", "viewer"));
        assert!(!can_grant_role("viewer", "member"));
        assert!(!can_grant_role("member", "admin"));
        assert!(!can_grant_role("admin", "owner"));
    }

    #[test]
    fn test_validate_four_words_format_valid() {
        assert!(validate_four_words_format("alice-bob-carol-dave"));
        assert!(validate_four_words_format("ocean-forest-moon-star"));
        assert!(validate_four_words_format("a-b-c-d"));
    }

    #[test]
    fn test_validate_four_words_format_invalid() {
        assert!(!validate_four_words_format("alice-bob-carol")); // 3 words
        assert!(!validate_four_words_format("alice-bob-carol-dave-eve")); // 5 words
        assert!(!validate_four_words_format("alice--bob-carol")); // empty word
        assert!(!validate_four_words_format("alice-bob123-carol-dave")); // numbers
        assert!(!validate_four_words_format("")); // empty
        assert!(!validate_four_words_format("----")); // all empty
    }

    // ============================================
    // Error Display Tests
    // ============================================

    #[test]
    fn test_error_display() {
        let err = InviteServiceError::EntityNotFound("entity-123".to_string());
        assert!(err.to_string().contains("entity-123"));

        let err = InviteServiceError::MemberNotFound {
            entity_id: "entity-1".to_string(),
            member_id: "alice-a-b-c".to_string(),
        };
        assert!(err.to_string().contains("entity-1"));
        assert!(err.to_string().contains("alice-a-b-c"));

        let err = InviteServiceError::RoleEscalation {
            granter_role: "member".to_string(),
            target_role: "admin".to_string(),
        };
        assert!(err.to_string().contains("member"));
        assert!(err.to_string().contains("admin"));

        let err = InviteServiceError::AlreadyMember {
            entity_id: "entity-1".to_string(),
            member_id: "bob-b-c-d".to_string(),
        };
        assert!(err.to_string().contains("already a member"));
    }

    #[test]
    fn test_error_from_invite_action_error() {
        let action_err = InviteActionError::Expired;
        let service_err: InviteServiceError = action_err.into();
        assert!(matches!(service_err, InviteServiceError::InviteExpired));

        let action_err = InviteActionError::AlreadyResolved(InviteStatus::Accepted);
        let service_err: InviteServiceError = action_err.into();
        assert!(matches!(
            service_err,
            InviteServiceError::AlreadyResolved(InviteStatus::Accepted)
        ));

        let action_err = InviteActionError::NotRecipient {
            expected: "alice-a-b-c".to_string(),
            actual: "bob-d-e-f".to_string(),
        };
        let service_err: InviteServiceError = action_err.into();
        assert!(matches!(
            service_err,
            InviteServiceError::PermissionDenied(_)
        ));
    }

    // ============================================
    // InviteRequest Builder Tests
    // ============================================

    #[test]
    fn test_invite_request_new_sets_required_fields() {
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            "org-123",
            "member",
        );

        assert_eq!(request.recipient_id, "alice-bob-carol-dave");
        assert_eq!(request.entity_type, EntityType::Organisation);
        assert_eq!(request.entity_id, "org-123");
        assert_eq!(request.role, "member");
        assert!(request.message.is_none());
        assert!(request.expires_in_hours.is_none());
    }

    #[test]
    fn test_invite_request_new_accepts_string_types() {
        // Test with &str
        let req1 = InviteRequest::new("alice-a-b-c", EntityType::Group, "group-1", "viewer");
        assert_eq!(req1.recipient_id, "alice-a-b-c");

        // Test with String
        let req2 = InviteRequest::new(
            String::from("bob-d-e-f"),
            EntityType::Channel,
            String::from("channel-1"),
            String::from("member"),
        );
        assert_eq!(req2.recipient_id, "bob-d-e-f");
        assert_eq!(req2.entity_id, "channel-1");
        assert_eq!(req2.role, "member");
    }

    #[test]
    fn test_invite_request_with_message() {
        let request = InviteRequest::new("alice-a-b-c", EntityType::Project, "proj-1", "admin")
            .with_message("Welcome to the team!");

        assert_eq!(request.message, Some("Welcome to the team!".to_string()));
        // Other fields unchanged
        assert_eq!(request.recipient_id, "alice-a-b-c");
        assert_eq!(request.role, "admin");
        assert!(request.expires_in_hours.is_none());
    }

    #[test]
    fn test_invite_request_with_expiration() {
        let request = InviteRequest::new("bob-x-y-z", EntityType::Organisation, "org-1", "member")
            .with_expiration(48);

        assert_eq!(request.expires_in_hours, Some(48));
        // Other fields unchanged
        assert_eq!(request.recipient_id, "bob-x-y-z");
        assert!(request.message.is_none());
    }

    #[test]
    fn test_invite_request_builder_chaining() {
        let request = InviteRequest::new("carol-m-n-o", EntityType::Group, "group-42", "viewer")
            .with_message("Join our group!")
            .with_expiration(24);

        assert_eq!(request.recipient_id, "carol-m-n-o");
        assert_eq!(request.entity_type, EntityType::Group);
        assert_eq!(request.entity_id, "group-42");
        assert_eq!(request.role, "viewer");
        assert_eq!(request.message, Some("Join our group!".to_string()));
        assert_eq!(request.expires_in_hours, Some(24));
    }

    #[test]
    fn test_invite_request_builder_order_independence() {
        // Expiration first, then message
        let req1 = InviteRequest::new("a-b-c-d", EntityType::Channel, "ch-1", "member")
            .with_expiration(12)
            .with_message("Hello");

        // Message first, then expiration
        let req2 = InviteRequest::new("a-b-c-d", EntityType::Channel, "ch-1", "member")
            .with_message("Hello")
            .with_expiration(12);

        assert_eq!(req1.message, req2.message);
        assert_eq!(req1.expires_in_hours, req2.expires_in_hours);
        assert_eq!(req1.recipient_id, req2.recipient_id);
        assert_eq!(req1.role, req2.role);
    }

    #[test]
    fn test_invite_request_with_all_entity_types() {
        let entity_types = [
            EntityType::Organisation,
            EntityType::Group,
            EntityType::Channel,
            EntityType::Project,
        ];

        for entity_type in entity_types {
            let request = InviteRequest::new("test-a-b-c", entity_type, "entity-id", "member");
            assert_eq!(request.entity_type, entity_type);
        }
    }

    #[test]
    fn test_invite_request_with_all_roles() {
        let roles = ["owner", "admin", "member", "viewer", "guest"];

        for role in roles {
            let request = InviteRequest::new("test-a-b-c", EntityType::Group, "group-1", role);
            assert_eq!(request.role, role);
        }
    }

    #[test]
    fn test_invite_request_empty_message_allowed() {
        let request =
            InviteRequest::new("alice-a-b-c", EntityType::Organisation, "org-1", "member")
                .with_message("");

        assert_eq!(request.message, Some(String::new()));
    }

    #[test]
    fn test_invite_request_zero_expiration_allowed() {
        // Zero hours means immediate expiration (edge case)
        let request = InviteRequest::new("alice-a-b-c", EntityType::Group, "group-1", "viewer")
            .with_expiration(0);

        assert_eq!(request.expires_in_hours, Some(0));
    }

    #[test]
    fn test_invite_request_long_expiration() {
        // 30 days in hours
        let request = InviteRequest::new("alice-a-b-c", EntityType::Project, "proj-1", "member")
            .with_expiration(720);

        assert_eq!(request.expires_in_hours, Some(720));
    }

    // ============================================
    // Integration tests (async with real services)
    // ============================================

    use crate::crdt_manager::CrdtManager;
    use tempfile::tempdir;

    /// Create a test InviteService with real EntityService.
    async fn create_test_invite_service() -> (InviteService, Arc<EntityService>, String) {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let crdt_manager = Arc::new(CrdtManager::new(&db_path).await.unwrap());
        let entity_service = Arc::new(EntityService::new(crdt_manager.clone()));
        let invite_service = InviteService::new(crdt_manager, entity_service.clone());
        // Keep temp_dir to prevent cleanup during test
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        std::mem::forget(temp_dir); // Prevent cleanup
        (invite_service, entity_service, temp_path)
    }

    #[tokio::test]
    async fn test_create_invite_invalid_recipient_format() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Create an organization
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A test org".to_string()),
                "owner-abc-def-ghi".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        // Try to create invite with invalid recipient format
        let request = InviteRequest::new(
            "invalid-format", // Only 2 words instead of 4
            EntityType::Organisation,
            &org.id,
            "member",
        );

        let result = invite_service
            .create_invite("owner-abc-def-ghi", request)
            .await;

        assert!(matches!(
            result,
            Err(InviteServiceError::InvalidFourWords(_))
        ));
    }

    #[tokio::test]
    async fn test_create_invite_creator_not_member() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Create an organization
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A test org".to_string()),
                "owner-abc-def-ghi".to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        // Try to create invite as non-member
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "member",
        );

        let result = invite_service
            .create_invite("other-user-not-member", request)
            .await;

        // Should fail because creator is not a member
        assert!(matches!(
            result,
            Err(InviteServiceError::MemberNotFound { .. })
        ));
    }

    #[tokio::test]
    async fn test_create_invite_success() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Create organization with owner
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A test org".to_string()),
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        // Set owner role for creator
        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create invite
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "member",
        )
        .with_message("Welcome to the team!")
        .with_expiration(48);

        let invite = invite_service
            .create_invite(creator_id, request)
            .await
            .expect("Failed to create invite");

        assert_eq!(invite.creator_id, creator_id);
        assert_eq!(invite.recipient_id, "alice-bob-carol-dave");
        assert_eq!(invite.entity_id, org.id);
        assert_eq!(invite.role, "member");
        assert_eq!(invite.message, Some("Welcome to the team!".to_string()));
        assert_eq!(invite.status, InviteStatus::Pending);
        assert!(invite.is_valid());
    }

    #[tokio::test]
    async fn test_create_invite_role_escalation_blocked() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Create organization with owner
        let owner_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A test org".to_string()),
                owner_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        // Set member role for a member
        let member_id = "member-xyz-uvw-rst";
        entity_service
            .add_member(EntityType::Organisation, &org.id, member_id, "member")
            .await
            .expect("Failed to add member");

        // Member tries to create invite with admin role
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "admin", // Higher than member's role
        );

        let result = invite_service.create_invite(member_id, request).await;

        // Member with "member" role doesn't have Edit on Members, so expect PermissionDenied
        // not RoleEscalation (which only happens after permissions check passes)
        assert!(
            matches!(result, Err(InviteServiceError::PermissionDenied(_))),
            "Expected PermissionDenied, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_create_invite_role_escalation_by_admin() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Create organization with owner
        let owner_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A test org".to_string()),
                owner_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        // Add admin with "admin" role (can create invites)
        let admin_id = "admin-xyz-uvw-rst";
        entity_service
            .add_member(EntityType::Organisation, &org.id, admin_id, "admin")
            .await
            .expect("Failed to add admin");

        // Admin tries to create invite with owner role (higher than admin's)
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "owner", // Higher than admin's role
        );

        let result = invite_service.create_invite(admin_id, request).await;

        assert!(
            matches!(result, Err(InviteServiceError::RoleEscalation { .. })),
            "Expected RoleEscalation, got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_get_invite() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Create organization with owner
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create invite
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "member",
        );

        let created_invite = invite_service
            .create_invite(creator_id, request)
            .await
            .expect("Failed to create invite");

        // Get invite
        let retrieved_invite = invite_service
            .get_invite(&created_invite.id)
            .await
            .expect("Failed to get invite");

        assert_eq!(retrieved_invite.id, created_invite.id);
        assert_eq!(retrieved_invite.creator_id, created_invite.creator_id);
        assert_eq!(retrieved_invite.recipient_id, created_invite.recipient_id);
    }

    #[tokio::test]
    async fn test_get_invite_not_found() {
        let (invite_service, _entity_service, _temp) = create_test_invite_service().await;

        let result = invite_service.get_invite("nonexistent-id").await;
        assert!(matches!(result, Err(InviteServiceError::InviteNotFound(_))));
    }

    #[tokio::test]
    async fn test_accept_invite_success() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup: Create org with owner
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create invite
        let recipient_id = "alice-bob-carol-dave";
        let request = InviteRequest::new(recipient_id, EntityType::Organisation, &org.id, "member");

        let invite = invite_service
            .create_invite(creator_id, request)
            .await
            .expect("Failed to create invite");

        // Accept invite
        invite_service
            .accept_invite(recipient_id, &invite.id)
            .await
            .expect("Failed to accept invite");

        // Verify invite is accepted
        let accepted_invite = invite_service
            .get_invite(&invite.id)
            .await
            .expect("Failed to get invite");

        assert_eq!(accepted_invite.status, InviteStatus::Accepted);

        // Verify recipient is now a member
        let role = entity_service
            .get_member_role(EntityType::Organisation, &org.id, recipient_id)
            .await
            .expect("Failed to get role");

        assert_eq!(role, "member");
    }

    #[tokio::test]
    async fn test_accept_invite_wrong_recipient() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create invite for Alice
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "member",
        );

        let invite = invite_service
            .create_invite(creator_id, request)
            .await
            .expect("Failed to create invite");

        // Bob tries to accept Alice's invite
        let result = invite_service
            .accept_invite("bob-xyz-uvw-rst", &invite.id)
            .await;

        assert!(matches!(
            result,
            Err(InviteServiceError::PermissionDenied(_))
        ));
    }

    #[tokio::test]
    async fn test_reject_invite_success() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create invite
        let recipient_id = "alice-bob-carol-dave";
        let request = InviteRequest::new(recipient_id, EntityType::Organisation, &org.id, "member");

        let invite = invite_service
            .create_invite(creator_id, request)
            .await
            .expect("Failed to create invite");

        // Reject invite
        invite_service
            .reject_invite(recipient_id, &invite.id)
            .await
            .expect("Failed to reject invite");

        // Verify invite is rejected
        let rejected_invite = invite_service
            .get_invite(&invite.id)
            .await
            .expect("Failed to get invite");

        assert_eq!(rejected_invite.status, InviteStatus::Rejected);

        // Verify recipient is NOT a member
        let role_result = entity_service
            .get_member_role(EntityType::Organisation, &org.id, recipient_id)
            .await;

        assert!(role_result.is_err());
    }

    #[tokio::test]
    async fn test_revoke_invite_by_creator() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create invite
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "member",
        );

        let invite = invite_service
            .create_invite(creator_id, request)
            .await
            .expect("Failed to create invite");

        // Creator revokes the invite
        invite_service
            .revoke_invite(creator_id, &invite.id)
            .await
            .expect("Failed to revoke invite");

        // Verify invite is revoked
        let revoked_invite = invite_service
            .get_invite(&invite.id)
            .await
            .expect("Failed to get invite");

        assert_eq!(revoked_invite.status, InviteStatus::Revoked);
    }

    #[tokio::test]
    async fn test_revoke_invite_by_admin() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup: Owner creates org, Admin also exists
        let owner_id = "owner-abc-def-ghi";
        let admin_id = "admin-xyz-uvw-rst";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                owner_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, owner_id, "owner")
            .await
            .expect("Failed to set owner role");

        entity_service
            .add_member(EntityType::Organisation, &org.id, admin_id, "admin")
            .await
            .expect("Failed to add admin");

        // Owner creates invite
        let request = InviteRequest::new(
            "alice-bob-carol-dave",
            EntityType::Organisation,
            &org.id,
            "member",
        );

        let invite = invite_service
            .create_invite(owner_id, request)
            .await
            .expect("Failed to create invite");

        // Admin revokes the invite
        invite_service
            .revoke_invite(admin_id, &invite.id)
            .await
            .expect("Admin should be able to revoke");

        let revoked_invite = invite_service
            .get_invite(&invite.id)
            .await
            .expect("Failed to get invite");

        assert_eq!(revoked_invite.status, InviteStatus::Revoked);
    }

    #[tokio::test]
    async fn test_list_pending_invites() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup
        let creator_id = "owner-abc-def-ghi";
        let recipient_id = "alice-bob-carol-dave";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create multiple invites for same recipient
        for i in 0..3 {
            let new_org = entity_service
                .create_entity(
                    format!("Org {}", i),
                    EntityType::Organisation,
                    None,
                    creator_id.to_string(),
                    vec![],
                )
                .await
                .expect("Failed to create org");

            entity_service
                .set_member_role(EntityType::Organisation, &new_org.id, creator_id, "owner")
                .await
                .expect("Failed to set role");

            let request = InviteRequest::new(
                recipient_id,
                EntityType::Organisation,
                &new_org.id,
                "member",
            );

            invite_service
                .create_invite(creator_id, request)
                .await
                .expect("Failed to create invite");
        }

        // List pending invites
        let pending = invite_service
            .list_pending_invites(recipient_id)
            .await
            .expect("Failed to list invites");

        assert_eq!(pending.len(), 3);
        for invite in pending {
            assert_eq!(invite.status, InviteStatus::Pending);
            assert_eq!(invite.recipient_id, recipient_id);
        }
    }

    #[tokio::test]
    async fn test_list_entity_invites() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup
        let creator_id = "owner-abc-def-ghi";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Create multiple invites to same org
        let recipients = [
            "alice-bob-carol-dave",
            "bob-xyz-uvw-rst",
            "carol-mno-pqr-stu",
        ];
        for recipient_id in recipients {
            let request =
                InviteRequest::new(recipient_id, EntityType::Organisation, &org.id, "member");

            invite_service
                .create_invite(creator_id, request)
                .await
                .expect("Failed to create invite");
        }

        // List invites for entity (requester must be a member)
        let entity_invites = invite_service
            .list_entity_invites(creator_id, EntityType::Organisation, &org.id)
            .await
            .expect("Failed to list entity invites");

        assert_eq!(entity_invites.len(), 3);
        for invite in entity_invites {
            assert_eq!(invite.entity_id, org.id);
        }
    }

    #[tokio::test]
    async fn test_already_member_blocked() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // Setup
        let creator_id = "owner-abc-def-ghi";
        let existing_member = "existing-mem-ber-xyz";
        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                None,
                creator_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, creator_id, "owner")
            .await
            .expect("Failed to set role");

        // Add existing member
        entity_service
            .add_member(EntityType::Organisation, &org.id, existing_member, "member")
            .await
            .expect("Failed to add member");

        // Try to create invite for existing member
        let request =
            InviteRequest::new(existing_member, EntityType::Organisation, &org.id, "member");

        let result = invite_service.create_invite(creator_id, request).await;

        assert!(matches!(
            result,
            Err(InviteServiceError::AlreadyMember { .. })
        ));
    }

    #[tokio::test]
    async fn test_full_invite_lifecycle() {
        let (invite_service, entity_service, _temp) = create_test_invite_service().await;

        // 1. Create organization
        let owner_id = "owner-abc-def-ghi";
        let recipient_id = "alice-bob-carol-dave";

        let org = entity_service
            .create_entity(
                "Test Org".to_string(),
                EntityType::Organisation,
                Some("A collaborative org".to_string()),
                owner_id.to_string(),
                vec![],
            )
            .await
            .expect("Failed to create org");

        entity_service
            .set_member_role(EntityType::Organisation, &org.id, owner_id, "owner")
            .await
            .expect("Failed to set role");

        // 2. Create invite
        let request = InviteRequest::new(recipient_id, EntityType::Organisation, &org.id, "member")
            .with_message("Welcome!")
            .with_expiration(24);

        let invite = invite_service
            .create_invite(owner_id, request)
            .await
            .expect("Failed to create invite");

        assert_eq!(invite.status, InviteStatus::Pending);
        assert!(invite.is_valid());

        // 3. List pending invites for recipient
        let pending = invite_service
            .list_pending_invites(recipient_id)
            .await
            .expect("Failed to list");

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, invite.id);

        // 4. Accept invite
        invite_service
            .accept_invite(recipient_id, &invite.id)
            .await
            .expect("Failed to accept");

        // 5. Verify invite status changed
        let accepted = invite_service
            .get_invite(&invite.id)
            .await
            .expect("Failed to get");

        assert_eq!(accepted.status, InviteStatus::Accepted);
        assert!(!accepted.is_valid()); // No longer valid after acceptance

        // 6. Verify member was added
        let role = entity_service
            .get_member_role(EntityType::Organisation, &org.id, recipient_id)
            .await
            .expect("Failed to get role");

        assert_eq!(role, "member");

        // 7. Verify pending list is now empty
        let pending_after = invite_service
            .list_pending_invites(recipient_id)
            .await
            .expect("Failed to list");

        assert!(pending_after.is_empty());
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating roles
    fn role_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("owner".to_string()),
            Just("admin".to_string()),
            Just("member".to_string()),
            Just("viewer".to_string()),
            Just("guest".to_string()),
        ]
    }

    // Strategy for generating four-word identities
    fn four_word_identity() -> impl Strategy<Value = String> {
        proptest::collection::vec("[a-z]{3,8}", 4).prop_map(|words| words.join("-"))
    }

    proptest! {
        /// Property: role_rank is monotonic with the role hierarchy.
        #[test]
        fn prop_role_rank_monotonic(
            role1 in role_strategy(),
            role2 in role_strategy(),
        ) {
            let rank1 = role_rank(&role1);
            let rank2 = role_rank(&role2);

            // If rank1 >= rank2, then can_grant_role should be true
            prop_assert_eq!(can_grant_role(&role1, &role2), rank1 >= rank2);
        }

        /// Property: can_grant_role is reflexive (can always grant own role).
        #[test]
        fn prop_can_grant_role_reflexive(role in role_strategy()) {
            prop_assert!(can_grant_role(&role, &role));
        }

        /// Property: can_grant_role is transitive.
        #[test]
        fn prop_can_grant_role_transitive(
            role_a in role_strategy(),
            role_b in role_strategy(),
            role_c in role_strategy(),
        ) {
            // If A can grant B and B can grant C, then A can grant C
            if can_grant_role(&role_a, &role_b) && can_grant_role(&role_b, &role_c) {
                prop_assert!(can_grant_role(&role_a, &role_c));
            }
        }

        /// Property: validate_four_words_format accepts valid four-word identities.
        #[test]
        fn prop_valid_four_words_accepted(identity in four_word_identity()) {
            prop_assert!(validate_four_words_format(&identity));
        }

        /// Property: validate_four_words_format rejects identities with wrong word count.
        #[test]
        fn prop_wrong_word_count_rejected(
            word_count in (1usize..10).prop_filter("not 4", |&c| c != 4),
        ) {
            let words: Vec<String> = (0..word_count).map(|i| format!("word{}", i)).collect();
            let identity = words.join("-");
            prop_assert!(!validate_four_words_format(&identity));
        }

        /// Property: role_rank returns 0 for unknown roles.
        #[test]
        fn prop_unknown_role_rank_zero(
            unknown in "[a-z]{5,10}".prop_filter("not standard", |s| {
                !["owner", "admin", "member", "viewer", "guest"].contains(&s.as_str())
            }),
        ) {
            prop_assert_eq!(role_rank(&unknown), 0);
        }

        /// Property: InviteRequest builder preserves required fields through chaining.
        #[test]
        fn prop_invite_request_preserves_required_fields(
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{8,16}",
            role in role_strategy(),
            message in proptest::option::of("[a-zA-Z0-9 ]{0,50}"),
            hours in proptest::option::of(0u32..1000),
        ) {
            let mut request = InviteRequest::new(
                recipient.clone(),
                EntityType::Organisation,
                entity_id.clone(),
                role.clone(),
            );

            if let Some(msg) = &message {
                request = request.with_message(msg.clone());
            }
            if let Some(h) = hours {
                request = request.with_expiration(h);
            }

            // Required fields must be preserved
            prop_assert_eq!(request.recipient_id, recipient);
            prop_assert_eq!(request.entity_id, entity_id);
            prop_assert_eq!(request.role, role);
            prop_assert_eq!(request.entity_type, EntityType::Organisation);
        }

        /// Property: Builder order does not affect final state.
        #[test]
        fn prop_builder_order_independence(
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{8,16}",
            role in role_strategy(),
            message in "[a-zA-Z0-9 ]{1,30}",
            hours in 1u32..500,
        ) {
            // Message first, then expiration
            let req1 = InviteRequest::new(
                recipient.clone(),
                EntityType::Group,
                entity_id.clone(),
                role.clone(),
            )
            .with_message(message.clone())
            .with_expiration(hours);

            // Expiration first, then message
            let req2 = InviteRequest::new(
                recipient,
                EntityType::Group,
                entity_id,
                role,
            )
            .with_expiration(hours)
            .with_message(message);

            prop_assert_eq!(req1.recipient_id, req2.recipient_id);
            prop_assert_eq!(req1.entity_id, req2.entity_id);
            prop_assert_eq!(req1.role, req2.role);
            prop_assert_eq!(req1.message, req2.message);
            prop_assert_eq!(req1.expires_in_hours, req2.expires_in_hours);
        }
    }
}

/// Property-based tests for CRDT-relevant invite operations.
///
/// These tests verify properties that are essential for CRDT storage:
/// - Terminal status transitions are irreversible
/// - Operations are idempotent
/// - State is deterministic based on timestamps
/// - Serialization preserves all fields
#[cfg(test)]
mod crdt_proptests {
    use super::*;
    use crate::invite::{Invite, InviteActionError, InviteStatus};
    use chrono::{Duration, Utc};
    use proptest::prelude::*;

    // Strategy for generating four-word identities
    fn four_word_identity() -> impl Strategy<Value = String> {
        proptest::collection::vec("[a-z]{3,8}", 4).prop_map(|words| words.join("-"))
    }

    // Strategy for generating roles
    fn role_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            Just("owner".to_string()),
            Just("admin".to_string()),
            Just("member".to_string()),
            Just("viewer".to_string()),
            Just("guest".to_string()),
        ]
    }

    // Strategy for generating entity types
    fn entity_type_strategy() -> impl Strategy<Value = EntityType> {
        prop_oneof![
            Just(EntityType::Organisation),
            Just(EntityType::Group),
            Just(EntityType::Channel),
            Just(EntityType::Project),
        ]
    }

    // Strategy for generating terminal status values
    fn terminal_status_strategy() -> impl Strategy<Value = InviteStatus> {
        prop_oneof![
            Just(InviteStatus::Accepted),
            Just(InviteStatus::Rejected),
            Just(InviteStatus::Expired),
            Just(InviteStatus::Revoked),
        ]
    }

    // Strategy for creating a valid invite
    fn invite_strategy() -> impl Strategy<Value = Invite> {
        (
            four_word_identity(),
            four_word_identity(),
            "[a-z0-9-]{8,16}",
            entity_type_strategy(),
            role_strategy(),
            proptest::option::of("[a-zA-Z0-9 ]{1,50}"),
            proptest::option::of(1u32..1000),
        )
            .prop_map(
                |(creator, recipient, entity_id, entity_type, role, message, expires)| {
                    Invite::new(
                        creator,
                        recipient,
                        entity_id,
                        entity_type,
                        role,
                        message,
                        expires,
                    )
                },
            )
    }

    proptest! {
        /// Property: Terminal status is permanent - once an invite is resolved, it cannot change.
        ///
        /// This is crucial for CRDT semantics: terminal states must be stable regardless
        /// of operation order when replaying on different nodes.
        #[test]
        fn prop_terminal_status_is_permanent(
            invite in invite_strategy(),
            terminal_status in terminal_status_strategy(),
            actor in four_word_identity(),
        ) {
            let mut invite = invite;
            let recipient = invite.recipient_id.clone();
            // Force to terminal state
            invite.status = terminal_status;
            invite.resolved_at = Some(Utc::now().timestamp_millis());

            // Try accept - should fail
            let accept_result = invite.accept(&recipient);
            prop_assert!(accept_result.is_err());
            if let Err(InviteActionError::AlreadyResolved(s)) = accept_result {
                prop_assert_eq!(s, terminal_status);
            } else {
                prop_assert!(false, "Expected AlreadyResolved error");
            }
            prop_assert_eq!(invite.status, terminal_status);

            // Try reject - should fail
            let reject_result = invite.reject(&recipient);
            prop_assert!(reject_result.is_err());
            prop_assert_eq!(invite.status, terminal_status);

            // Try revoke - should fail
            let revoke_result = invite.revoke(&actor);
            prop_assert!(revoke_result.is_err());
            prop_assert_eq!(invite.status, terminal_status);

            // Try mark_expired - should fail
            let expire_result = invite.mark_expired();
            prop_assert!(expire_result.is_err());
            prop_assert_eq!(invite.status, terminal_status);
        }

        /// Property: Accept operation is idempotent on the error case.
        ///
        /// Once accepted, repeated accept calls return the same error.
        #[test]
        fn prop_accept_idempotent_after_success(
            invite in invite_strategy(),
        ) {
            let mut invite = invite;
            let recipient = invite.recipient_id.clone();

            // First accept succeeds
            let result1 = invite.accept(&recipient);
            prop_assert!(result1.is_ok());
            prop_assert_eq!(invite.status, InviteStatus::Accepted);

            let resolved_at1 = invite.resolved_at;

            // Second accept fails with AlreadyResolved
            let result2 = invite.accept(&recipient);
            prop_assert!(matches!(result2, Err(InviteActionError::AlreadyResolved(InviteStatus::Accepted))));

            // State unchanged
            prop_assert_eq!(invite.status, InviteStatus::Accepted);
            prop_assert_eq!(invite.resolved_at, resolved_at1);
        }

        /// Property: Reject operation is idempotent on the error case.
        #[test]
        fn prop_reject_idempotent_after_success(
            invite in invite_strategy(),
        ) {
            let mut invite = invite;
            let recipient = invite.recipient_id.clone();

            // First reject succeeds
            let result1 = invite.reject(&recipient);
            prop_assert!(result1.is_ok());
            prop_assert_eq!(invite.status, InviteStatus::Rejected);

            let resolved_at1 = invite.resolved_at;

            // Second reject fails with AlreadyResolved
            let result2 = invite.reject(&recipient);
            prop_assert!(matches!(result2, Err(InviteActionError::AlreadyResolved(InviteStatus::Rejected))));

            // State unchanged
            prop_assert_eq!(invite.status, InviteStatus::Rejected);
            prop_assert_eq!(invite.resolved_at, resolved_at1);
        }

        /// Property: Revoke operation is idempotent on the error case.
        #[test]
        fn prop_revoke_idempotent_after_success(
            invite in invite_strategy(),
            revoker in four_word_identity(),
        ) {
            let mut invite = invite;

            // First revoke succeeds
            let result1 = invite.revoke(&revoker);
            prop_assert!(result1.is_ok());
            prop_assert_eq!(invite.status, InviteStatus::Revoked);

            let resolved_at1 = invite.resolved_at;

            // Second revoke fails with AlreadyResolved
            let result2 = invite.revoke(&revoker);
            prop_assert!(matches!(result2, Err(InviteActionError::AlreadyResolved(InviteStatus::Revoked))));

            // State unchanged
            prop_assert_eq!(invite.status, InviteStatus::Revoked);
            prop_assert_eq!(invite.resolved_at, resolved_at1);
        }

        /// Property: Expiration is deterministic based on timestamps.
        ///
        /// The same invite checked at the same time always gives the same result.
        #[test]
        fn prop_expiration_deterministic(
            invite in invite_strategy(),
            hours_offset in -100i64..100,
        ) {
            let check_time = Utc::now() + Duration::hours(hours_offset);

            // Check multiple times at the same timestamp
            let expired1 = invite.is_expired_at(check_time);
            let expired2 = invite.is_expired_at(check_time);
            let expired3 = invite.is_expired_at(check_time);

            prop_assert_eq!(expired1, expired2);
            prop_assert_eq!(expired2, expired3);
        }

        /// Property: Expiration monotonicity - once expired, always expired (at same or later time).
        #[test]
        fn prop_expiration_monotonic(
            invite in invite_strategy(),
        ) {
            if let Some(expires_at) = invite.expires_at {
                // Time after expiration
                let after_expiry = chrono::DateTime::from_timestamp_millis(expires_at + 1000)
                    .unwrap_or_else(Utc::now);
                let later_still = after_expiry + Duration::hours(1);

                // Should be expired at both times
                prop_assert!(invite.is_expired_at(after_expiry));
                prop_assert!(invite.is_expired_at(later_still));
            }
        }

        /// Property: is_valid is consistent with is_pending and is_expired.
        #[test]
        fn prop_is_valid_consistent(
            invite in invite_strategy(),
            hours_offset in -100i64..100,
        ) {
            let check_time = Utc::now() + Duration::hours(hours_offset);

            let is_valid = invite.is_valid_at(check_time);
            let is_pending = invite.is_pending();
            let is_expired = invite.is_expired_at(check_time);

            // is_valid should be: pending AND NOT expired
            prop_assert_eq!(is_valid, is_pending && !is_expired);
        }

        /// Property: Only recipient can accept or reject.
        #[test]
        fn prop_only_recipient_can_accept_or_reject(
            invite in invite_strategy(),
            wrong_actor in four_word_identity(),
        ) {
            // Skip if wrong_actor happens to match recipient (very unlikely)
            prop_assume!(wrong_actor != invite.recipient_id);

            let mut invite_for_accept = invite.clone();
            let mut invite_for_reject = invite;
            let expected_recipient_accept = invite_for_accept.recipient_id.clone();
            let expected_recipient_reject = invite_for_reject.recipient_id.clone();

            // Accept with wrong actor fails
            let accept_result = invite_for_accept.accept(&wrong_actor);
            prop_assert!(accept_result.is_err());
            if let Err(InviteActionError::NotRecipient { expected, actual }) = accept_result {
                prop_assert_eq!(expected, expected_recipient_accept);
                prop_assert_eq!(actual, wrong_actor.clone());
            } else {
                prop_assert!(false, "Expected NotRecipient error");
            }
            prop_assert!(invite_for_accept.is_pending()); // Status unchanged

            // Reject with wrong actor fails
            let reject_result = invite_for_reject.reject(&wrong_actor);
            prop_assert!(reject_result.is_err());
            if let Err(InviteActionError::NotRecipient { expected, actual }) = reject_result {
                prop_assert_eq!(expected, expected_recipient_reject);
                prop_assert_eq!(actual, wrong_actor);
            } else {
                prop_assert!(false, "Expected NotRecipient error");
            }
            prop_assert!(invite_for_reject.is_pending()); // Status unchanged
        }

        /// Property: Revoke can be done by anyone (authorization is at service level).
        ///
        /// The Invite model itself doesn't check who can revoke - that's
        /// done by InviteService which checks permissions.
        #[test]
        fn prop_revoke_accepts_any_actor(
            invite in invite_strategy(),
            any_actor in four_word_identity(),
        ) {
            let mut invite = invite;

            // Any actor can revoke at the model level
            let result = invite.revoke(&any_actor);
            prop_assert!(result.is_ok());
            prop_assert_eq!(invite.status, InviteStatus::Revoked);
            prop_assert_eq!(invite.resolved_by, Some(any_actor));
        }

        /// Property: Serialization roundtrip preserves all fields.
        ///
        /// Essential for CRDT storage: data must survive serialization.
        #[test]
        fn prop_serialization_roundtrip(
            invite in invite_strategy(),
        ) {
            // Serialize to JSON
            let json = serde_json::to_string(&invite)
                .expect("serialization should succeed");

            // Deserialize back
            let restored: Invite = serde_json::from_str(&json)
                .expect("deserialization should succeed");

            // All fields preserved
            prop_assert_eq!(invite.id, restored.id);
            prop_assert_eq!(invite.creator_id, restored.creator_id);
            prop_assert_eq!(invite.recipient_id, restored.recipient_id);
            prop_assert_eq!(invite.entity_id, restored.entity_id);
            prop_assert_eq!(invite.entity_type, restored.entity_type);
            prop_assert_eq!(invite.role, restored.role);
            prop_assert_eq!(invite.message, restored.message);
            prop_assert_eq!(invite.status, restored.status);
            prop_assert_eq!(invite.created_at, restored.created_at);
            prop_assert_eq!(invite.expires_at, restored.expires_at);
            prop_assert_eq!(invite.resolved_at, restored.resolved_at);
            prop_assert_eq!(invite.resolved_by, restored.resolved_by);
        }

        /// Property: InviteStatus serialization roundtrip for all statuses.
        #[test]
        fn prop_invite_status_serialization(
            status in prop_oneof![
                Just(InviteStatus::Pending),
                Just(InviteStatus::Accepted),
                Just(InviteStatus::Rejected),
                Just(InviteStatus::Expired),
                Just(InviteStatus::Revoked),
            ],
        ) {
            // Serialize
            let json = serde_json::to_string(&status)
                .expect("status serialization should succeed");

            // Deserialize
            let restored: InviteStatus = serde_json::from_str(&json)
                .expect("status deserialization should succeed");

            prop_assert_eq!(status, restored);

            // Also test Display/FromStr roundtrip
            let display = status.to_string();
            let parsed: InviteStatus = display.parse()
                .expect("status parse should succeed");
            prop_assert_eq!(status, parsed);
        }

        /// Property: Resolved invites record resolution metadata.
        #[test]
        fn prop_resolution_records_metadata(
            invite in invite_strategy(),
            actor in four_word_identity(),
        ) {
            // Test accept
            let mut invite_accept = invite.clone();
            let recipient = invite_accept.recipient_id.clone();
            if invite_accept.accept(&recipient).is_ok() {
                prop_assert!(invite_accept.resolved_at.is_some());
                prop_assert_eq!(invite_accept.resolved_by, Some(recipient.clone()));
            }

            // Test reject
            let mut invite_reject = invite.clone();
            let recipient = invite_reject.recipient_id.clone();
            if invite_reject.reject(&recipient).is_ok() {
                prop_assert!(invite_reject.resolved_at.is_some());
                prop_assert_eq!(invite_reject.resolved_by, Some(recipient.clone()));
            }

            // Test revoke
            let mut invite_revoke = invite;
            if invite_revoke.revoke(&actor).is_ok() {
                prop_assert!(invite_revoke.resolved_at.is_some());
                prop_assert_eq!(invite_revoke.resolved_by, Some(actor));
            }
        }

        /// Property: New invites start in pending status.
        #[test]
        fn prop_new_invite_pending(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{8,16}",
            entity_type in entity_type_strategy(),
            role in role_strategy(),
        ) {
            let invite = Invite::new(
                creator,
                recipient,
                entity_id,
                entity_type,
                role,
                None,
                None,
            );

            prop_assert!(invite.is_pending());
            prop_assert!(!invite.is_resolved());
            prop_assert!(invite.resolved_at.is_none());
            prop_assert!(invite.resolved_by.is_none());
        }

        /// Property: Invite without expiration never expires.
        #[test]
        fn prop_no_expiration_never_expires(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{8,16}",
            hours_in_future in 0i64..100000,
        ) {
            let invite = Invite::new(
                creator,
                recipient,
                entity_id,
                EntityType::Organisation,
                "member".to_string(),
                None,
                None, // No expiration
            );

            // Check at any future time
            let future_time = Utc::now() + Duration::hours(hours_in_future);
            prop_assert!(!invite.is_expired_at(future_time));
        }

        /// Property: First action on pending invite determines final state.
        ///
        /// This simulates CRDT conflict resolution: whichever action happens
        /// first (by timestamp) wins and subsequent actions fail.
        #[test]
        fn prop_first_action_wins(
            invite in invite_strategy(),
            action_type in 0u8..3,
            revoker in four_word_identity(),
        ) {
            let mut invite = invite;
            let recipient = invite.recipient_id.clone();

            // Perform first action based on type
            let first_status = match action_type {
                0 => {
                    let _ = invite.accept(&recipient);
                    InviteStatus::Accepted
                }
                1 => {
                    let _ = invite.reject(&recipient);
                    InviteStatus::Rejected
                }
                _ => {
                    let _ = invite.revoke(&revoker);
                    InviteStatus::Revoked
                }
            };

            // Now the invite should be in the terminal state
            prop_assert_eq!(invite.status, first_status);
            prop_assert!(invite.is_resolved());

            // All subsequent actions should fail
            prop_assert!(invite.accept(&recipient).is_err());
            prop_assert!(invite.reject(&recipient).is_err());
            prop_assert!(invite.revoke(&revoker).is_err());
        }
    }
}

/// Tests for CRDT storage implementation.
///
/// These tests verify that invites are correctly stored and retrieved
/// from Yrs CRDT documents, enabling offline-first sync.
#[cfg(test)]
mod crdt_storage_tests {
    use super::*;
    use crate::crdt_manager::CrdtManager;
    use crate::entity_service::EntityService;
    use crate::invite::{Invite, InviteStatus};
    use crate::legacy_crdt::EntityType;
    use std::sync::Arc;
    use tempfile::TempDir;

    /// Helper to create a test CrdtManager with temporary storage.
    async fn create_test_crdt_manager() -> (Arc<CrdtManager>, TempDir) {
        let temp_dir = TempDir::new().expect("failed to create temp dir");
        let crdt_manager = CrdtManager::new(temp_dir.path())
            .await
            .expect("failed to create CrdtManager");
        (Arc::new(crdt_manager), temp_dir)
    }

    /// Helper to create a test EntityService with a shared CrdtManager.
    fn create_test_entity_service(crdt_manager: Arc<CrdtManager>) -> Arc<EntityService> {
        Arc::new(EntityService::new(crdt_manager))
    }

    /// Helper to create test InviteService with real CRDT manager.
    async fn create_test_invite_service() -> (InviteService, TempDir) {
        let (crdt_manager, temp_dir) = create_test_crdt_manager().await;
        let entity_service = create_test_entity_service(crdt_manager.clone());
        let service = InviteService::new(crdt_manager, entity_service);
        (service, temp_dir)
    }

    /// Helper to create a test invite.
    fn create_test_invite(recipient: &str, entity_type: EntityType, entity_id: &str) -> Invite {
        Invite::new(
            "alice-test-four-word".to_string(),
            recipient.to_string(),
            entity_id.to_string(),
            entity_type,
            "member".to_string(),
            Some("Welcome!".to_string()),
            Some(24), // expires in 24 hours
        )
    }

    // ============================================
    // CRDT Storage Tests (TDD - write tests first)
    // ============================================

    #[tokio::test]
    async fn test_store_and_load_invite_from_crdt() {
        let (service, _temp_dir) = create_test_invite_service().await;

        let invite = create_test_invite("bob-test-four-word", EntityType::Organisation, "org-123");
        let invite_id = invite.id.clone();

        // Store the invite in CRDT
        service
            .store_invite_in_crdt(&invite)
            .await
            .expect("failed to store invite");

        // Load it back from CRDT
        let loaded = service
            .load_invite_from_crdt(&invite_id)
            .await
            .expect("failed to load invite");

        // Verify all fields
        assert_eq!(loaded.id, invite.id);
        assert_eq!(loaded.creator_id, invite.creator_id);
        assert_eq!(loaded.recipient_id, invite.recipient_id);
        assert_eq!(loaded.entity_id, invite.entity_id);
        assert_eq!(loaded.entity_type, invite.entity_type);
        assert_eq!(loaded.role, invite.role);
        assert_eq!(loaded.message, invite.message);
        assert_eq!(loaded.status, invite.status);
        assert_eq!(loaded.created_at, invite.created_at);
        assert_eq!(loaded.expires_at, invite.expires_at);
    }

    #[tokio::test]
    async fn test_load_nonexistent_invite_returns_not_found() {
        let (service, _temp_dir) = create_test_invite_service().await;

        let result = service.load_invite_from_crdt("nonexistent-invite-id").await;

        assert!(matches!(result, Err(InviteServiceError::InviteNotFound(_))));
    }

    #[tokio::test]
    async fn test_store_invite_updates_existing() {
        let (service, _temp_dir) = create_test_invite_service().await;

        let mut invite =
            create_test_invite("bob-test-four-word", EntityType::Organisation, "org-123");
        let invite_id = invite.id.clone();

        // Store initial invite
        service
            .store_invite_in_crdt(&invite)
            .await
            .expect("failed to store invite");

        // Modify the invite (accept it)
        let recipient = invite.recipient_id.clone();
        invite.accept(&recipient).expect("failed to accept");

        // Store updated invite
        service
            .store_invite_in_crdt(&invite)
            .await
            .expect("failed to store updated invite");

        // Load and verify update was persisted
        let loaded = service
            .load_invite_from_crdt(&invite_id)
            .await
            .expect("failed to load invite");

        assert_eq!(loaded.status, InviteStatus::Accepted);
        assert!(loaded.resolved_at.is_some());
        assert_eq!(loaded.resolved_by, Some(recipient));
    }

    #[tokio::test]
    async fn test_load_invites_for_entity_from_crdt() {
        let (service, _temp_dir) = create_test_invite_service().await;

        let entity_type = EntityType::Organisation;
        let entity_id = "org-456";

        // Store multiple invites for the same entity
        let invite1 = create_test_invite("bob-one-two-three", entity_type, entity_id);
        let invite2 = create_test_invite("carol-four-five-six", entity_type, entity_id);
        let invite3 = create_test_invite("dave-seven-eight-nine", EntityType::Group, "group-789");

        service
            .store_invite_in_crdt(&invite1)
            .await
            .expect("store 1");
        service
            .store_invite_in_crdt(&invite2)
            .await
            .expect("store 2");
        service
            .store_invite_in_crdt(&invite3)
            .await
            .expect("store 3");

        // Load invites for the specific entity
        let invites = service
            .load_invites_for_entity_from_crdt(entity_type, entity_id)
            .await
            .expect("failed to load entity invites");

        // Should only get the 2 invites for org-456
        assert_eq!(invites.len(), 2);

        let invite_ids: Vec<&str> = invites.iter().map(|i| i.id.as_str()).collect();
        assert!(invite_ids.contains(&invite1.id.as_str()));
        assert!(invite_ids.contains(&invite2.id.as_str()));
        assert!(!invite_ids.contains(&invite3.id.as_str()));
    }

    #[tokio::test]
    async fn test_load_invites_for_empty_entity_returns_empty_vec() {
        let (service, _temp_dir) = create_test_invite_service().await;

        let invites = service
            .load_invites_for_entity_from_crdt(EntityType::Organisation, "nonexistent-entity")
            .await
            .expect("should return empty vec");

        assert!(invites.is_empty());
    }

    #[tokio::test]
    async fn test_crdt_storage_survives_service_restart() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        let invite_id: String;
        let original_recipient: String;

        // First service instance - store an invite
        {
            let crdt_manager = Arc::new(
                CrdtManager::new(temp_dir.path())
                    .await
                    .expect("failed to create CrdtManager"),
            );
            let entity_service = create_test_entity_service(crdt_manager.clone());
            let service = InviteService::new(crdt_manager, entity_service);

            let invite =
                create_test_invite("bob-test-four-word", EntityType::Organisation, "org-123");
            invite_id = invite.id.clone();
            original_recipient = invite.recipient_id.clone();

            service
                .store_invite_in_crdt(&invite)
                .await
                .expect("failed to store invite");
        }

        // Second service instance (simulates restart) - load the invite
        {
            let crdt_manager = Arc::new(
                CrdtManager::new(temp_dir.path())
                    .await
                    .expect("failed to create CrdtManager"),
            );
            let entity_service = create_test_entity_service(crdt_manager.clone());
            let service = InviteService::new(crdt_manager, entity_service);

            let loaded = service
                .load_invite_from_crdt(&invite_id)
                .await
                .expect("failed to load invite after restart");

            assert_eq!(loaded.id, invite_id);
            assert_eq!(loaded.recipient_id, original_recipient);
            assert_eq!(loaded.status, InviteStatus::Pending);
        }
    }

    #[tokio::test]
    async fn test_multiple_entities_have_separate_crdt_docs() {
        let (service, _temp_dir) = create_test_invite_service().await;

        // Store invites for different entity types
        let org_invite =
            create_test_invite("bob-org-four-word", EntityType::Organisation, "org-123");
        let group_invite =
            create_test_invite("carol-group-four-word", EntityType::Group, "group-456");
        let project_invite =
            create_test_invite("dave-project-four-word", EntityType::Project, "project-789");

        service
            .store_invite_in_crdt(&org_invite)
            .await
            .expect("store org");
        service
            .store_invite_in_crdt(&group_invite)
            .await
            .expect("store group");
        service
            .store_invite_in_crdt(&project_invite)
            .await
            .expect("store project");

        // Each entity should have exactly one invite
        let org_invites = service
            .load_invites_for_entity_from_crdt(EntityType::Organisation, "org-123")
            .await
            .expect("org invites");
        assert_eq!(org_invites.len(), 1);
        assert_eq!(org_invites[0].id, org_invite.id);

        let group_invites = service
            .load_invites_for_entity_from_crdt(EntityType::Group, "group-456")
            .await
            .expect("group invites");
        assert_eq!(group_invites.len(), 1);
        assert_eq!(group_invites[0].id, group_invite.id);

        let project_invites = service
            .load_invites_for_entity_from_crdt(EntityType::Project, "project-789")
            .await
            .expect("project invites");
        assert_eq!(project_invites.len(), 1);
        assert_eq!(project_invites[0].id, project_invite.id);
    }

    #[tokio::test]
    async fn test_store_invite_with_all_statuses() {
        let (service, _temp_dir) = create_test_invite_service().await;

        // Test each status can be stored and retrieved
        let statuses = [
            InviteStatus::Pending,
            InviteStatus::Accepted,
            InviteStatus::Rejected,
            InviteStatus::Revoked,
            InviteStatus::Expired,
        ];

        for (i, status) in statuses.iter().enumerate() {
            let mut invite = create_test_invite(
                &format!("bob-{}-four-word", i),
                EntityType::Organisation,
                &format!("org-status-{}", i),
            );

            // Manually set status for testing
            match status {
                InviteStatus::Pending => {}
                InviteStatus::Accepted => {
                    let recipient = invite.recipient_id.clone();
                    invite.accept(&recipient).expect("accept");
                }
                InviteStatus::Rejected => {
                    let recipient = invite.recipient_id.clone();
                    invite.reject(&recipient).expect("reject");
                }
                InviteStatus::Revoked => {
                    invite.revoke("admin-a-b-c").expect("revoke");
                }
                InviteStatus::Expired => {
                    invite.mark_expired().expect("mark expired");
                }
            }

            let invite_id = invite.id.clone();
            service
                .store_invite_in_crdt(&invite)
                .await
                .unwrap_or_else(|_| panic!("store {} failed", status));

            let loaded = service
                .load_invite_from_crdt(&invite_id)
                .await
                .unwrap_or_else(|_| panic!("load {} failed", status));

            assert_eq!(loaded.status, *status, "status mismatch for {}", status);
        }
    }

    #[tokio::test]
    async fn test_recipient_index_persisted_in_crdt() {
        let temp_dir = TempDir::new().expect("failed to create temp dir");

        let recipient = "bob-index-test-word";
        let invite1_id: String;
        let invite2_id: String;

        // First instance - store invites for same recipient
        {
            let crdt_manager = Arc::new(
                CrdtManager::new(temp_dir.path())
                    .await
                    .expect("failed to create CrdtManager"),
            );
            let entity_service = create_test_entity_service(crdt_manager.clone());
            let service = InviteService::new(crdt_manager, entity_service);

            let invite1 = create_test_invite(recipient, EntityType::Organisation, "org-1");
            let invite2 = create_test_invite(recipient, EntityType::Group, "group-1");
            invite1_id = invite1.id.clone();
            invite2_id = invite2.id.clone();

            service
                .store_invite_in_crdt(&invite1)
                .await
                .expect("store 1");
            service
                .store_invite_in_crdt(&invite2)
                .await
                .expect("store 2");

            // Also store in recipient index
            service
                .store_recipient_index_in_crdt(recipient, &[invite1_id.clone(), invite2_id.clone()])
                .await
                .expect("store index");
        }

        // Second instance - verify index persisted
        {
            let crdt_manager = Arc::new(
                CrdtManager::new(temp_dir.path())
                    .await
                    .expect("failed to create CrdtManager"),
            );
            let entity_service = create_test_entity_service(crdt_manager.clone());
            let service = InviteService::new(crdt_manager, entity_service);

            let invite_ids = service
                .load_recipient_index_from_crdt(recipient)
                .await
                .expect("load index");

            assert_eq!(invite_ids.len(), 2);
            assert!(invite_ids.contains(&invite1_id));
            assert!(invite_ids.contains(&invite2_id));
        }
    }
}
