// SPDX-License-Identifier: MIT OR Apache-2.0

//! Invite model for cross-organization collaboration.
//!
//! This module defines the invite structure and status tracking for
//! inviting external collaborators to entities via four-word identities.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crdt::EntityType;

/// Status of an invite throughout its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteStatus {
    /// Invite is awaiting recipient action.
    Pending,
    /// Invite was accepted by recipient.
    Accepted,
    /// Invite was rejected by recipient.
    Rejected,
    /// Invite expired before any action.
    Expired,
    /// Invite was revoked by creator or admin.
    Revoked,
}

impl InviteStatus {
    /// Check if this status represents a terminal state.
    pub fn is_terminal(self) -> bool {
        !matches!(self, InviteStatus::Pending)
    }

    /// Get all possible status values.
    pub fn all() -> &'static [InviteStatus] {
        &[
            InviteStatus::Pending,
            InviteStatus::Accepted,
            InviteStatus::Rejected,
            InviteStatus::Expired,
            InviteStatus::Revoked,
        ]
    }
}

impl std::fmt::Display for InviteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InviteStatus::Pending => write!(f, "pending"),
            InviteStatus::Accepted => write!(f, "accepted"),
            InviteStatus::Rejected => write!(f, "rejected"),
            InviteStatus::Expired => write!(f, "expired"),
            InviteStatus::Revoked => write!(f, "revoked"),
        }
    }
}

impl std::str::FromStr for InviteStatus {
    type Err = InviteParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(InviteStatus::Pending),
            "accepted" => Ok(InviteStatus::Accepted),
            "rejected" => Ok(InviteStatus::Rejected),
            "expired" => Ok(InviteStatus::Expired),
            "revoked" => Ok(InviteStatus::Revoked),
            _ => Err(InviteParseError::InvalidStatus(s.to_string())),
        }
    }
}

/// Errors when parsing invite data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InviteParseError {
    /// Invalid status string.
    InvalidStatus(String),
    /// Invalid four-word identity format.
    InvalidFourWords(String),
}

impl std::fmt::Display for InviteParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InviteParseError::InvalidStatus(s) => {
                write!(
                    f,
                    "invalid invite status '{}': expected pending, accepted, rejected, expired, or revoked",
                    s
                )
            }
            InviteParseError::InvalidFourWords(s) => {
                write!(
                    f,
                    "invalid four-word identity '{}': expected format 'word-word-word-word'",
                    s
                )
            }
        }
    }
}

impl std::error::Error for InviteParseError {}

/// An invitation to join an entity.
///
/// Invites are created by entity members with appropriate permissions
/// and target a specific four-word identity. On acceptance, the recipient
/// becomes a member with the specified role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    /// Unique identifier for this invite.
    pub id: String,

    /// Four-word identity of the invite creator.
    pub creator_id: String,

    /// Four-word identity of the intended recipient.
    pub recipient_id: String,

    /// Target entity ID.
    pub entity_id: String,

    /// Target entity type.
    pub entity_type: EntityType,

    /// Role to grant on acceptance.
    pub role: String,

    /// Optional message from creator to recipient.
    pub message: Option<String>,

    /// Current status of the invite.
    pub status: InviteStatus,

    /// When the invite was created (Unix timestamp ms).
    pub created_at: i64,

    /// When the invite expires, if set (Unix timestamp ms).
    pub expires_at: Option<i64>,

    /// When the invite was resolved (accepted/rejected/revoked).
    pub resolved_at: Option<i64>,

    /// Who resolved the invite (recipient for accept/reject, admin for revoke).
    pub resolved_by: Option<String>,
}

impl Invite {
    /// Create a new pending invite.
    ///
    /// # Arguments
    ///
    /// * `creator_id` - Four-word identity of the creator
    /// * `recipient_id` - Four-word identity of the recipient
    /// * `entity_id` - ID of the entity to join
    /// * `entity_type` - Type of the entity
    /// * `role` - Role to grant on acceptance
    /// * `message` - Optional message to recipient
    /// * `expires_in_hours` - Optional expiration in hours from now
    ///
    /// # Example
    ///
    /// ```
    /// use communitas_core::invite::Invite;
    /// use communitas_core::crdt::EntityType;
    ///
    /// let invite = Invite::new(
    ///     "alice-brave-cloud-dawn".to_string(),
    ///     "bob-calm-river-east".to_string(),
    ///     "project-123".to_string(),
    ///     EntityType::Project,
    ///     "member".to_string(),
    ///     Some("Join our project!".to_string()),
    ///     Some(168), // 1 week
    /// );
    ///
    /// assert!(invite.is_pending());
    /// assert!(invite.is_valid());
    /// ```
    pub fn new(
        creator_id: String,
        recipient_id: String,
        entity_id: String,
        entity_type: EntityType,
        role: String,
        message: Option<String>,
        expires_in_hours: Option<u32>,
    ) -> Self {
        let now = Utc::now();
        let created_at = now.timestamp_millis();
        let expires_at =
            expires_in_hours.map(|h| (now + Duration::hours(i64::from(h))).timestamp_millis());

        Self {
            id: Uuid::new_v4().to_string(),
            creator_id,
            recipient_id,
            entity_id,
            entity_type,
            role,
            message,
            status: InviteStatus::Pending,
            created_at,
            expires_at,
            resolved_at: None,
            resolved_by: None,
        }
    }

    /// Create an invite with a specific ID (for deserialization/testing).
    ///
    /// This is primarily used for testing and deserialization where the ID
    /// is already known. For normal invite creation, use `Invite::new()`.
    #[cfg(test)]
    pub fn with_id(id: String, mut base: Self) -> Self {
        base.id = id;
        base
    }

    /// Set a custom ID on this invite (builder pattern).
    ///
    /// Returns self for method chaining.
    pub fn set_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }

    /// Check if the invite is still pending.
    pub fn is_pending(&self) -> bool {
        self.status == InviteStatus::Pending
    }

    /// Check if the invite has been accepted.
    pub fn is_accepted(&self) -> bool {
        self.status == InviteStatus::Accepted
    }

    /// Check if the invite is in a terminal state.
    pub fn is_resolved(&self) -> bool {
        self.status.is_terminal()
    }

    /// Check if the invite has expired based on current time.
    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Utc::now())
    }

    /// Check if the invite would be expired at a given time.
    pub fn is_expired_at(&self, at: DateTime<Utc>) -> bool {
        if let Some(expires_at) = self.expires_at {
            at.timestamp_millis() > expires_at
        } else {
            false
        }
    }

    /// Check if the invite is valid (pending and not expired).
    pub fn is_valid(&self) -> bool {
        self.is_valid_at(Utc::now())
    }

    /// Check if the invite would be valid at a given time.
    pub fn is_valid_at(&self, at: DateTime<Utc>) -> bool {
        self.is_pending() && !self.is_expired_at(at)
    }

    /// Accept the invite.
    ///
    /// Returns `Err` if invite is not in a valid state.
    pub fn accept(&mut self, acceptor_id: &str) -> Result<(), InviteActionError> {
        self.accept_at(acceptor_id, Utc::now())
    }

    /// Accept the invite at a specific time (for testing).
    pub fn accept_at(
        &mut self,
        acceptor_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), InviteActionError> {
        if self.recipient_id != acceptor_id {
            return Err(InviteActionError::NotRecipient {
                expected: self.recipient_id.clone(),
                actual: acceptor_id.to_string(),
            });
        }

        if !self.is_pending() {
            return Err(InviteActionError::AlreadyResolved(self.status));
        }

        if self.is_expired_at(at) {
            return Err(InviteActionError::Expired);
        }

        self.status = InviteStatus::Accepted;
        self.resolved_at = Some(at.timestamp_millis());
        self.resolved_by = Some(acceptor_id.to_string());
        Ok(())
    }

    /// Reject the invite.
    ///
    /// Returns `Err` if invite is not in a valid state.
    pub fn reject(&mut self, rejector_id: &str) -> Result<(), InviteActionError> {
        self.reject_at(rejector_id, Utc::now())
    }

    /// Reject the invite at a specific time (for testing).
    pub fn reject_at(
        &mut self,
        rejector_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), InviteActionError> {
        if self.recipient_id != rejector_id {
            return Err(InviteActionError::NotRecipient {
                expected: self.recipient_id.clone(),
                actual: rejector_id.to_string(),
            });
        }

        if !self.is_pending() {
            return Err(InviteActionError::AlreadyResolved(self.status));
        }

        // Note: Can reject even if expired (just marking intent)
        self.status = InviteStatus::Rejected;
        self.resolved_at = Some(at.timestamp_millis());
        self.resolved_by = Some(rejector_id.to_string());
        Ok(())
    }

    /// Revoke the invite (creator or admin action).
    ///
    /// Returns `Err` if invite is not pending.
    pub fn revoke(&mut self, revoker_id: &str) -> Result<(), InviteActionError> {
        self.revoke_at(revoker_id, Utc::now())
    }

    /// Revoke the invite at a specific time (for testing).
    pub fn revoke_at(
        &mut self,
        revoker_id: &str,
        at: DateTime<Utc>,
    ) -> Result<(), InviteActionError> {
        if !self.is_pending() {
            return Err(InviteActionError::AlreadyResolved(self.status));
        }

        self.status = InviteStatus::Revoked;
        self.resolved_at = Some(at.timestamp_millis());
        self.resolved_by = Some(revoker_id.to_string());
        Ok(())
    }

    /// Mark the invite as expired.
    ///
    /// This is typically called during cleanup when checking validity.
    pub fn mark_expired(&mut self) -> Result<(), InviteActionError> {
        self.mark_expired_at(Utc::now())
    }

    /// Mark the invite as expired at a specific time.
    pub fn mark_expired_at(&mut self, at: DateTime<Utc>) -> Result<(), InviteActionError> {
        if !self.is_pending() {
            return Err(InviteActionError::AlreadyResolved(self.status));
        }

        self.status = InviteStatus::Expired;
        self.resolved_at = Some(at.timestamp_millis());
        Ok(())
    }

    /// Get the created_at timestamp as DateTime.
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp_millis(self.created_at).unwrap_or_else(Utc::now)
    }

    /// Get the expires_at timestamp as DateTime, if set.
    pub fn expires_at_datetime(&self) -> Option<DateTime<Utc>> {
        self.expires_at.and_then(DateTime::from_timestamp_millis)
    }

    /// Get the resolved_at timestamp as DateTime, if set.
    pub fn resolved_at_datetime(&self) -> Option<DateTime<Utc>> {
        self.resolved_at.and_then(DateTime::from_timestamp_millis)
    }

    /// Get time remaining until expiration, if applicable.
    pub fn time_remaining(&self) -> Option<Duration> {
        self.time_remaining_at(Utc::now())
    }

    /// Get time remaining until expiration at a specific time.
    pub fn time_remaining_at(&self, at: DateTime<Utc>) -> Option<Duration> {
        self.expires_at_datetime().map(|expires| expires - at)
    }
}

/// Errors when performing actions on an invite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InviteActionError {
    /// The invite has already been resolved (accepted, rejected, etc.).
    AlreadyResolved(InviteStatus),
    /// The invite has expired.
    Expired,
    /// The actor is not the intended recipient.
    NotRecipient { expected: String, actual: String },
}

impl std::fmt::Display for InviteActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InviteActionError::AlreadyResolved(status) => {
                write!(f, "invite already resolved with status: {}", status)
            }
            InviteActionError::Expired => write!(f, "invite has expired"),
            InviteActionError::NotRecipient { expected, actual } => {
                write!(
                    f,
                    "not the invite recipient: expected '{}', got '{}'",
                    expected, actual
                )
            }
        }
    }
}

impl std::error::Error for InviteActionError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to create a basic invite
    fn create_test_invite() -> Invite {
        Invite::new(
            "alice-brave-cloud-dawn".to_string(),
            "bob-calm-river-east".to_string(),
            "project-123".to_string(),
            EntityType::Project,
            "member".to_string(),
            None,
            None,
        )
    }

    fn create_test_invite_with_expiry(hours: u32) -> Invite {
        Invite::new(
            "alice-brave-cloud-dawn".to_string(),
            "bob-calm-river-east".to_string(),
            "project-123".to_string(),
            EntityType::Project,
            "member".to_string(),
            None,
            Some(hours),
        )
    }

    // ============================================
    // Invite Creation Tests
    // ============================================

    #[test]
    fn test_new_invite_has_pending_status() {
        let invite = create_test_invite();
        assert_eq!(invite.status, InviteStatus::Pending);
        assert!(invite.is_pending());
        assert!(!invite.is_resolved());
    }

    #[test]
    fn test_new_invite_has_unique_id() {
        let invite1 = create_test_invite();
        let invite2 = create_test_invite();
        assert_ne!(invite1.id, invite2.id);
    }

    #[test]
    fn test_new_invite_stores_creator_and_recipient() {
        let invite = Invite::new(
            "creator-one-two-three".to_string(),
            "recipient-four-five-six".to_string(),
            "entity-1".to_string(),
            EntityType::Group,
            "viewer".to_string(),
            Some("Welcome!".to_string()),
            None,
        );

        assert_eq!(invite.creator_id, "creator-one-two-three");
        assert_eq!(invite.recipient_id, "recipient-four-five-six");
        assert_eq!(invite.entity_id, "entity-1");
        assert_eq!(invite.entity_type, EntityType::Group);
        assert_eq!(invite.role, "viewer");
        assert_eq!(invite.message, Some("Welcome!".to_string()));
    }

    #[test]
    fn test_new_invite_sets_created_at() {
        let before = Utc::now().timestamp_millis();
        let invite = create_test_invite();
        let after = Utc::now().timestamp_millis();

        assert!(invite.created_at >= before);
        assert!(invite.created_at <= after);
    }

    #[test]
    fn test_new_invite_without_expiry() {
        let invite = create_test_invite();
        assert!(invite.expires_at.is_none());
        assert!(!invite.is_expired());
    }

    #[test]
    fn test_new_invite_with_expiry() {
        let invite = create_test_invite_with_expiry(24);
        assert!(invite.expires_at.is_some());

        let expected_expiry = invite.created_at + (24 * 60 * 60 * 1000);
        assert_eq!(invite.expires_at.unwrap(), expected_expiry);
    }

    #[test]
    fn test_with_id_sets_custom_id() {
        let base = Invite::new(
            "creator-a-b-c".to_string(),
            "recipient-d-e-f".to_string(),
            "entity-1".to_string(),
            EntityType::Channel,
            "member".to_string(),
            None,
            None,
        );
        let invite = Invite::with_id("custom-id-123".to_string(), base);

        assert_eq!(invite.id, "custom-id-123");
    }

    #[test]
    fn test_set_id_builder_pattern() {
        let invite = Invite::new(
            "creator-a-b-c".to_string(),
            "recipient-d-e-f".to_string(),
            "entity-1".to_string(),
            EntityType::Channel,
            "member".to_string(),
            None,
            None,
        )
        .set_id("my-custom-id".to_string());

        assert_eq!(invite.id, "my-custom-id");
    }

    // ============================================
    // Expiration Tests
    // ============================================

    #[test]
    fn test_is_expired_before_expiry_time() {
        let invite = create_test_invite_with_expiry(24);

        // Check at creation time - should not be expired
        let at_creation = invite.created_at_datetime();
        assert!(!invite.is_expired_at(at_creation));
    }

    #[test]
    fn test_is_expired_after_expiry_time() {
        let invite = create_test_invite_with_expiry(1); // 1 hour

        // Check 2 hours after creation - should be expired
        let two_hours_later = invite.created_at_datetime() + Duration::hours(2);
        assert!(invite.is_expired_at(two_hours_later));
    }

    #[test]
    fn test_is_expired_exactly_at_expiry() {
        let invite = create_test_invite_with_expiry(1);

        // Check exactly at expiry - should not be expired (> not >=)
        let at_expiry = invite.expires_at_datetime().unwrap();
        assert!(!invite.is_expired_at(at_expiry));

        // 1ms after should be expired
        let just_after = at_expiry + Duration::milliseconds(1);
        assert!(invite.is_expired_at(just_after));
    }

    #[test]
    fn test_is_valid_checks_both_pending_and_expiry() {
        let invite = create_test_invite_with_expiry(1);

        // At creation: pending and not expired = valid
        let at_creation = invite.created_at_datetime();
        assert!(invite.is_valid_at(at_creation));

        // After expiry: pending but expired = not valid
        let after_expiry = invite.created_at_datetime() + Duration::hours(2);
        assert!(!invite.is_valid_at(after_expiry));
    }

    #[test]
    fn test_time_remaining() {
        let invite = create_test_invite_with_expiry(24);
        let at_creation = invite.created_at_datetime();

        let remaining = invite.time_remaining_at(at_creation).unwrap();
        // Should be approximately 24 hours (allow 1 second tolerance)
        assert!(remaining.num_hours() == 24 || remaining.num_hours() == 23);
    }

    #[test]
    fn test_time_remaining_none_without_expiry() {
        let invite = create_test_invite();
        assert!(invite.time_remaining().is_none());
    }

    // ============================================
    // Accept Tests
    // ============================================

    #[test]
    fn test_accept_by_recipient_succeeds() {
        let mut invite = create_test_invite();

        let result = invite.accept("bob-calm-river-east");
        assert!(result.is_ok());
        assert_eq!(invite.status, InviteStatus::Accepted);
        assert!(invite.is_accepted());
        assert!(invite.is_resolved());
        assert!(invite.resolved_at.is_some());
        assert_eq!(invite.resolved_by, Some("bob-calm-river-east".to_string()));
    }

    #[test]
    fn test_accept_by_non_recipient_fails() {
        let mut invite = create_test_invite();

        let result = invite.accept("charlie-wrong-person-here");
        assert!(result.is_err());

        match result {
            Err(InviteActionError::NotRecipient { expected, actual }) => {
                assert_eq!(expected, "bob-calm-river-east");
                assert_eq!(actual, "charlie-wrong-person-here");
            }
            _ => panic!("Expected NotRecipient error"),
        }

        // Invite should still be pending
        assert!(invite.is_pending());
    }

    #[test]
    fn test_accept_already_accepted_fails() {
        let mut invite = create_test_invite();
        invite.accept("bob-calm-river-east").unwrap();

        let result = invite.accept("bob-calm-river-east");
        assert!(matches!(
            result,
            Err(InviteActionError::AlreadyResolved(InviteStatus::Accepted))
        ));
    }

    #[test]
    fn test_accept_already_rejected_fails() {
        let mut invite = create_test_invite();
        invite.reject("bob-calm-river-east").unwrap();

        let result = invite.accept("bob-calm-river-east");
        assert!(matches!(
            result,
            Err(InviteActionError::AlreadyResolved(InviteStatus::Rejected))
        ));
    }

    #[test]
    fn test_accept_expired_invite_fails() {
        let mut invite = create_test_invite_with_expiry(1);
        let after_expiry = invite.created_at_datetime() + Duration::hours(2);

        let result = invite.accept_at("bob-calm-river-east", after_expiry);
        assert!(matches!(result, Err(InviteActionError::Expired)));

        // Invite should still be pending (not auto-marked expired)
        assert!(invite.is_pending());
    }

    // ============================================
    // Reject Tests
    // ============================================

    #[test]
    fn test_reject_by_recipient_succeeds() {
        let mut invite = create_test_invite();

        let result = invite.reject("bob-calm-river-east");
        assert!(result.is_ok());
        assert_eq!(invite.status, InviteStatus::Rejected);
        assert!(invite.is_resolved());
    }

    #[test]
    fn test_reject_by_non_recipient_fails() {
        let mut invite = create_test_invite();

        let result = invite.reject("charlie-wrong-person-here");
        assert!(matches!(
            result,
            Err(InviteActionError::NotRecipient { .. })
        ));
    }

    #[test]
    fn test_reject_already_resolved_fails() {
        let mut invite = create_test_invite();
        invite.accept("bob-calm-river-east").unwrap();

        let result = invite.reject("bob-calm-river-east");
        assert!(matches!(result, Err(InviteActionError::AlreadyResolved(_))));
    }

    #[test]
    fn test_reject_expired_invite_succeeds() {
        // You can reject an expired invite (just recording the rejection)
        let mut invite = create_test_invite_with_expiry(1);
        let after_expiry = invite.created_at_datetime() + Duration::hours(2);

        // This should succeed - rejection just marks intent
        let result = invite.reject_at("bob-calm-river-east", after_expiry);
        assert!(result.is_ok());
        assert_eq!(invite.status, InviteStatus::Rejected);
    }

    // ============================================
    // Revoke Tests
    // ============================================

    #[test]
    fn test_revoke_by_creator_succeeds() {
        let mut invite = create_test_invite();

        let result = invite.revoke("alice-brave-cloud-dawn");
        assert!(result.is_ok());
        assert_eq!(invite.status, InviteStatus::Revoked);
        assert!(invite.is_resolved());
        assert_eq!(
            invite.resolved_by,
            Some("alice-brave-cloud-dawn".to_string())
        );
    }

    #[test]
    fn test_revoke_by_admin_succeeds() {
        let mut invite = create_test_invite();

        // Admin (not creator) can also revoke
        let result = invite.revoke("admin-other-person-here");
        assert!(result.is_ok());
        assert_eq!(invite.status, InviteStatus::Revoked);
    }

    #[test]
    fn test_revoke_already_resolved_fails() {
        let mut invite = create_test_invite();
        invite.accept("bob-calm-river-east").unwrap();

        let result = invite.revoke("alice-brave-cloud-dawn");
        assert!(matches!(result, Err(InviteActionError::AlreadyResolved(_))));
    }

    // ============================================
    // Mark Expired Tests
    // ============================================

    #[test]
    fn test_mark_expired_succeeds() {
        let mut invite = create_test_invite();

        let result = invite.mark_expired();
        assert!(result.is_ok());
        assert_eq!(invite.status, InviteStatus::Expired);
        assert!(invite.is_resolved());
    }

    #[test]
    fn test_mark_expired_already_resolved_fails() {
        let mut invite = create_test_invite();
        invite.accept("bob-calm-river-east").unwrap();

        let result = invite.mark_expired();
        assert!(matches!(result, Err(InviteActionError::AlreadyResolved(_))));
    }

    // ============================================
    // InviteStatus Tests
    // ============================================

    #[test]
    fn test_status_is_terminal() {
        assert!(!InviteStatus::Pending.is_terminal());
        assert!(InviteStatus::Accepted.is_terminal());
        assert!(InviteStatus::Rejected.is_terminal());
        assert!(InviteStatus::Expired.is_terminal());
        assert!(InviteStatus::Revoked.is_terminal());
    }

    #[test]
    fn test_status_all() {
        let all = InviteStatus::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&InviteStatus::Pending));
        assert!(all.contains(&InviteStatus::Accepted));
        assert!(all.contains(&InviteStatus::Rejected));
        assert!(all.contains(&InviteStatus::Expired));
        assert!(all.contains(&InviteStatus::Revoked));
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", InviteStatus::Pending), "pending");
        assert_eq!(format!("{}", InviteStatus::Accepted), "accepted");
        assert_eq!(format!("{}", InviteStatus::Rejected), "rejected");
        assert_eq!(format!("{}", InviteStatus::Expired), "expired");
        assert_eq!(format!("{}", InviteStatus::Revoked), "revoked");
    }

    #[test]
    fn test_status_from_str() {
        assert_eq!(
            "pending".parse::<InviteStatus>().unwrap(),
            InviteStatus::Pending
        );
        assert_eq!(
            "ACCEPTED".parse::<InviteStatus>().unwrap(),
            InviteStatus::Accepted
        );
        assert_eq!(
            "Rejected".parse::<InviteStatus>().unwrap(),
            InviteStatus::Rejected
        );
        assert!("invalid".parse::<InviteStatus>().is_err());
    }

    #[test]
    fn test_status_roundtrip() {
        for status in InviteStatus::all() {
            let s = status.to_string();
            let parsed: InviteStatus = s.parse().unwrap();
            assert_eq!(*status, parsed);
        }
    }

    // ============================================
    // Serialization Tests
    // ============================================

    #[test]
    fn test_invite_serialization_roundtrip() {
        let invite = Invite::new(
            "creator-a-b-c".to_string(),
            "recipient-d-e-f".to_string(),
            "entity-123".to_string(),
            EntityType::Group,
            "member".to_string(),
            Some("Welcome to the group!".to_string()),
            Some(48),
        );

        let json = serde_json::to_string(&invite).unwrap();
        let parsed: Invite = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, invite.id);
        assert_eq!(parsed.creator_id, invite.creator_id);
        assert_eq!(parsed.recipient_id, invite.recipient_id);
        assert_eq!(parsed.entity_id, invite.entity_id);
        assert_eq!(parsed.entity_type, invite.entity_type);
        assert_eq!(parsed.role, invite.role);
        assert_eq!(parsed.message, invite.message);
        assert_eq!(parsed.status, invite.status);
        assert_eq!(parsed.created_at, invite.created_at);
        assert_eq!(parsed.expires_at, invite.expires_at);
    }

    #[test]
    fn test_status_serialization() {
        let status = InviteStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"pending\"");

        let parsed: InviteStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, status);
    }

    // ============================================
    // DateTime Helper Tests
    // ============================================

    #[test]
    fn test_created_at_datetime() {
        let invite = create_test_invite();
        let dt = invite.created_at_datetime();

        // Should be very close to now
        let diff = (Utc::now() - dt).num_seconds().abs();
        assert!(diff < 2);
    }

    #[test]
    fn test_expires_at_datetime() {
        let invite = create_test_invite_with_expiry(24);
        let expires = invite.expires_at_datetime().unwrap();
        let created = invite.created_at_datetime();

        let diff = (expires - created).num_hours();
        assert_eq!(diff, 24);
    }

    #[test]
    fn test_resolved_at_datetime_none_before_resolution() {
        let invite = create_test_invite();
        assert!(invite.resolved_at_datetime().is_none());
    }

    #[test]
    fn test_resolved_at_datetime_set_after_resolution() {
        let mut invite = create_test_invite();
        invite.accept("bob-calm-river-east").unwrap();

        let resolved = invite.resolved_at_datetime();
        assert!(resolved.is_some());
    }

    // ============================================
    // Error Display Tests
    // ============================================

    #[test]
    fn test_invite_action_error_display() {
        let err = InviteActionError::AlreadyResolved(InviteStatus::Accepted);
        assert!(err.to_string().contains("already resolved"));
        assert!(err.to_string().contains("accepted"));

        let err = InviteActionError::Expired;
        assert!(err.to_string().contains("expired"));

        let err = InviteActionError::NotRecipient {
            expected: "alice-a-b-c".to_string(),
            actual: "bob-d-e-f".to_string(),
        };
        assert!(err.to_string().contains("alice-a-b-c"));
        assert!(err.to_string().contains("bob-d-e-f"));
    }

    #[test]
    fn test_invite_parse_error_display() {
        let err = InviteParseError::InvalidStatus("unknown".to_string());
        assert!(err.to_string().contains("unknown"));
        assert!(err.to_string().contains("pending"));

        let err = InviteParseError::InvalidFourWords("bad-format".to_string());
        assert!(err.to_string().contains("bad-format"));
        assert!(err.to_string().contains("word-word-word-word"));
    }

    // ============================================
    // Entity Type Tests
    // ============================================

    #[test]
    fn test_invite_for_all_entity_types() {
        let entity_types = [
            EntityType::Group,
            EntityType::Channel,
            EntityType::Project,
            EntityType::Organisation,
            EntityType::Person,
        ];

        for entity_type in entity_types {
            let invite = Invite::new(
                "creator-a-b-c".to_string(),
                "recipient-d-e-f".to_string(),
                "entity-123".to_string(),
                entity_type,
                "member".to_string(),
                None,
                None,
            );

            assert_eq!(invite.entity_type, entity_type);
            assert!(invite.is_valid());
        }
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating valid four-word identities
    fn four_word_identity() -> impl Strategy<Value = String> {
        // Generate 4 words of 3-8 lowercase letters each
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
            Just(EntityType::Group),
            Just(EntityType::Channel),
            Just(EntityType::Project),
            Just(EntityType::Organisation),
            Just(EntityType::Person),
        ]
    }

    // Strategy for optional message
    fn optional_message() -> impl Strategy<Value = Option<String>> {
        prop_oneof![Just(None), "[a-zA-Z0-9 ]{0,100}".prop_map(Some),]
    }

    // Strategy for optional expiry (0-168 hours)
    fn optional_expiry() -> impl Strategy<Value = Option<u32>> {
        prop_oneof![Just(None), (1u32..168).prop_map(Some),]
    }

    proptest! {
        /// Property: New invites are always pending and valid.
        #[test]
        fn prop_new_invite_is_pending_and_valid(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
            entity_type in entity_type_strategy(),
            role in role_strategy(),
            message in optional_message(),
            expires in optional_expiry(),
        ) {
            let invite = Invite::new(
                creator,
                recipient,
                entity_id,
                entity_type,
                role,
                message,
                expires,
            );

            prop_assert!(invite.is_pending());
            prop_assert!(invite.is_valid());
            prop_assert!(!invite.is_resolved());
        }

        /// Property: Accepting an invite by recipient always succeeds for valid invite.
        #[test]
        fn prop_accept_by_recipient_succeeds(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
        ) {
            let mut invite = Invite::new(
                creator,
                recipient.clone(),
                entity_id,
                EntityType::Group,
                "member".to_string(),
                None,
                None,
            );

            let result = invite.accept(&recipient);
            prop_assert!(result.is_ok());
            prop_assert!(invite.is_accepted());
            prop_assert!(invite.is_resolved());
        }

        /// Property: Accepting by non-recipient always fails.
        #[test]
        fn prop_accept_by_non_recipient_fails(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            wrong_person in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
        ) {
            prop_assume!(recipient != wrong_person);

            let mut invite = Invite::new(
                creator,
                recipient,
                entity_id,
                EntityType::Group,
                "member".to_string(),
                None,
                None,
            );

            let result = invite.accept(&wrong_person);
            prop_assert!(result.is_err());
            prop_assert!(invite.is_pending());
        }

        /// Property: Cannot accept/reject/revoke an already resolved invite.
        #[test]
        fn prop_resolved_invite_cannot_be_changed(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
            action in prop_oneof![Just("accept"), Just("reject"), Just("revoke")],
        ) {
            let mut invite = Invite::new(
                creator.clone(),
                recipient.clone(),
                entity_id,
                EntityType::Group,
                "member".to_string(),
                None,
                None,
            );

            // First action - should succeed
            match action {
                "accept" => { invite.accept(&recipient).unwrap(); }
                "reject" => { invite.reject(&recipient).unwrap(); }
                "revoke" => { invite.revoke(&creator).unwrap(); }
                _ => unreachable!(),
            }

            // Second action - should fail
            let result1 = invite.accept(&recipient);
            let result2 = invite.reject(&recipient);
            let result3 = invite.revoke(&creator);

            prop_assert!(result1.is_err());
            prop_assert!(result2.is_err());
            prop_assert!(result3.is_err());
        }

        /// Property: Invite with expiry becomes invalid after expiry time.
        #[test]
        fn prop_expired_invite_invalid(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
            hours in 1u32..100,
        ) {
            let invite = Invite::new(
                creator,
                recipient,
                entity_id,
                EntityType::Project,
                "member".to_string(),
                None,
                Some(hours),
            );

            // At creation - valid
            let at_creation = invite.created_at_datetime();
            prop_assert!(invite.is_valid_at(at_creation));

            // After expiry - invalid
            let after_expiry = at_creation + Duration::hours(i64::from(hours) + 1);
            prop_assert!(!invite.is_valid_at(after_expiry));
        }

        /// Property: Invite without expiry never expires.
        #[test]
        fn prop_no_expiry_never_expires(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
            years_in_future in 1i64..100,
        ) {
            let invite = Invite::new(
                creator,
                recipient,
                entity_id,
                EntityType::Channel,
                "viewer".to_string(),
                None,
                None, // No expiry
            );

            // Far in the future - still valid
            let future = invite.created_at_datetime() + Duration::days(years_in_future * 365);
            prop_assert!(!invite.is_expired_at(future));
            prop_assert!(invite.is_valid_at(future));
        }

        /// Property: Serialization roundtrip preserves all fields.
        #[test]
        fn prop_serialization_roundtrip(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
            entity_type in entity_type_strategy(),
            role in role_strategy(),
            message in optional_message(),
            expires in optional_expiry(),
        ) {
            let invite = Invite::new(
                creator,
                recipient,
                entity_id,
                entity_type,
                role,
                message,
                expires,
            );

            let json = serde_json::to_string(&invite).unwrap();
            let parsed: Invite = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(invite.id, parsed.id);
            prop_assert_eq!(invite.creator_id, parsed.creator_id);
            prop_assert_eq!(invite.recipient_id, parsed.recipient_id);
            prop_assert_eq!(invite.entity_id, parsed.entity_id);
            prop_assert_eq!(invite.entity_type, parsed.entity_type);
            prop_assert_eq!(invite.role, parsed.role);
            prop_assert_eq!(invite.message, parsed.message);
            prop_assert_eq!(invite.status, parsed.status);
            prop_assert_eq!(invite.created_at, parsed.created_at);
            prop_assert_eq!(invite.expires_at, parsed.expires_at);
        }

        /// Property: Status roundtrip through string always works.
        #[test]
        fn prop_status_string_roundtrip(
            status_idx in 0usize..5,
        ) {
            let status = InviteStatus::all()[status_idx];
            let s = status.to_string();
            let parsed: InviteStatus = s.parse().unwrap();
            prop_assert_eq!(status, parsed);
        }

        /// Property: Each invite gets a unique ID.
        #[test]
        fn prop_unique_ids(
            count in 2usize..10,
            creator in four_word_identity(),
            recipient in four_word_identity(),
        ) {
            let invites: Vec<Invite> = (0..count)
                .map(|i| Invite::new(
                    creator.clone(),
                    recipient.clone(),
                    format!("entity-{}", i),
                    EntityType::Group,
                    "member".to_string(),
                    None,
                    None,
                ))
                .collect();

            let ids: std::collections::HashSet<_> = invites.iter().map(|i| &i.id).collect();
            prop_assert_eq!(ids.len(), count);
        }

        /// Property: resolved_by is set correctly after resolution.
        #[test]
        fn prop_resolved_by_tracks_resolver(
            creator in four_word_identity(),
            recipient in four_word_identity(),
            entity_id in "[a-z0-9-]{5,20}",
        ) {
            // Test accept
            let mut invite1 = Invite::new(
                creator.clone(),
                recipient.clone(),
                entity_id.clone(),
                EntityType::Group,
                "member".to_string(),
                None,
                None,
            );
            invite1.accept(&recipient).unwrap();
            prop_assert_eq!(invite1.resolved_by, Some(recipient.clone()));

            // Test reject
            let mut invite2 = Invite::new(
                creator.clone(),
                recipient.clone(),
                entity_id.clone(),
                EntityType::Group,
                "member".to_string(),
                None,
                None,
            );
            invite2.reject(&recipient).unwrap();
            prop_assert_eq!(invite2.resolved_by, Some(recipient.clone()));

            // Test revoke
            let mut invite3 = Invite::new(
                creator.clone(),
                recipient,
                entity_id,
                EntityType::Group,
                "member".to_string(),
                None,
                None,
            );
            invite3.revoke(&creator).unwrap();
            prop_assert_eq!(invite3.resolved_by, Some(creator));
        }
    }
}
