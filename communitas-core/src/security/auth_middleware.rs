//! Authentication middleware for Tauri commands
//!
//! This module provides:
//! - Session-based authentication
//! - Role-based access control
//! - Secure session management
//! - Protection against unauthorized command execution

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use uuid::Uuid;

/// Maximum session duration (1 hour)
pub const MAX_SESSION_DURATION: Duration = Duration::from_secs(3600);

/// Session cleanup interval (5 minutes)
pub const SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

/// Authentication session information
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub session_id: String,
    pub user_id: String,
    pub four_words_identity: String,
    pub permissions: Vec<Permission>,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub expires_at: Instant,
}

impl AuthSession {
    /// Create a new authentication session
    pub fn new(user_id: String, four_words_identity: String, permissions: Vec<Permission>) -> Self {
        let now = Instant::now();
        Self {
            session_id: Uuid::new_v4().to_string(),
            user_id,
            four_words_identity,
            permissions,
            created_at: now,
            last_accessed: now,
            expires_at: now + MAX_SESSION_DURATION,
        }
    }

    /// Check if the session is still valid
    pub fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }

    /// Update the last accessed timestamp
    pub fn refresh(&mut self) {
        self.last_accessed = Instant::now();
    }

    /// Check if the session has the required permission
    pub fn has_permission(&self, required: &Permission) -> bool {
        self.permissions.iter().any(|p| p.allows(required))
    }
}

/// Permission system for role-based access control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub scope: PermissionScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionScope {
    Own,    // Only own resources
    Shared, // Shared resources with appropriate access
    All,    // All resources (admin level)
}

impl Permission {
    pub fn new(resource: &str, action: &str, scope: PermissionScope) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
            scope,
        }
    }

    /// Check if this permission allows the required permission
    pub fn allows(&self, required: &Permission) -> bool {
        // Resource must match (or this permission is for all resources)
        let resource_match = self.resource == "*" || self.resource == required.resource;

        // Action must match (or this permission allows all actions)
        let action_match = self.action == "*" || self.action == required.action;

        // Scope must be sufficient
        let scope_match = matches!(
            (&self.scope, &required.scope),
            (PermissionScope::All, _)
                | (PermissionScope::Shared, PermissionScope::Own)
                | (PermissionScope::Shared, PermissionScope::Shared)
                | (PermissionScope::Own, PermissionScope::Own)
        );

        resource_match && action_match && scope_match
    }
}

/// Authentication middleware for managing sessions
#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    sessions: Arc<RwLock<HashMap<String, AuthSession>>>,
    last_cleanup: Arc<RwLock<Instant>>,
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthMiddleware {
    /// Create a new authentication middleware
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Create a new authenticated session
    pub fn create_session(
        &self,
        user_id: String,
        four_words_identity: String,
        permissions: Vec<Permission>,
    ) -> Result<String> {
        let session = AuthSession::new(user_id, four_words_identity, permissions);
        let session_id = session.session_id.clone();

        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire sessions lock"))?;
            sessions.insert(session_id.clone(), session);
        }

        // Trigger cleanup if needed
        self.cleanup_expired_sessions()?;

        Ok(session_id)
    }

    /// Validate a session and return the session information
    pub fn validate_session(&self, session_id: &str) -> Result<AuthSession> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire sessions lock"))?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow::anyhow!("Invalid session ID"))?;

        if !session.is_valid() {
            sessions.remove(session_id);
            return Err(anyhow::anyhow!("Session expired"));
        }

        session.refresh();
        Ok(session.clone())
    }

    /// Check if a session has the required permission
    pub fn check_permission(
        &self,
        session_id: &str,
        required_permission: &Permission,
    ) -> Result<bool> {
        let session = self.validate_session(session_id)?;
        Ok(session.has_permission(required_permission))
    }

    /// Require a specific permission for a session (returns error if not authorized)
    pub fn require_permission(
        &self,
        session_id: &str,
        required_permission: &Permission,
    ) -> Result<AuthSession> {
        let session = self.validate_session(session_id)?;

        if !session.has_permission(required_permission) {
            return Err(anyhow::anyhow!(
                "Insufficient permissions. Required: {:?}",
                required_permission
            ));
        }

        Ok(session)
    }

    /// End a session (logout)
    pub fn end_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self
            .sessions
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire sessions lock"))?;

        sessions.remove(session_id);
        Ok(())
    }

    /// Get all active sessions (admin function)
    pub fn get_active_sessions(&self) -> Result<Vec<AuthSession>> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire sessions lock"))?;

        let active_sessions: Vec<AuthSession> = sessions
            .values()
            .filter(|session| session.is_valid())
            .cloned()
            .collect();

        Ok(active_sessions)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> Result<()> {
        let now = Instant::now();

        // Check if cleanup is needed
        {
            let last_cleanup = self
                .last_cleanup
                .read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cleanup lock"))?;

            if now.duration_since(*last_cleanup) < SESSION_CLEANUP_INTERVAL {
                return Ok(()); // Cleanup not needed yet
            }
        }

        // Perform cleanup
        {
            let mut sessions = self
                .sessions
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire sessions lock"))?;

            sessions.retain(|_, session| session.is_valid());
        }

        // Update last cleanup time
        {
            let mut last_cleanup = self
                .last_cleanup
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cleanup lock"))?;
            *last_cleanup = now;
        }

        Ok(())
    }

    /// Get session statistics
    pub fn get_stats(&self) -> Result<SessionStats> {
        let sessions = self
            .sessions
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire sessions lock"))?;

        let total_sessions = sessions.len();
        let active_sessions = sessions.values().filter(|s| s.is_valid()).count();
        let expired_sessions = total_sessions - active_sessions;

        Ok(SessionStats {
            total_sessions,
            active_sessions,
            expired_sessions,
        })
    }
}

/// Session statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
}

/// Common permission definitions
pub mod permissions {
    use super::{Permission, PermissionScope};

    pub fn read_messages() -> Permission {
        Permission::new("messages", "read", PermissionScope::Shared)
    }

    pub fn send_messages() -> Permission {
        Permission::new("messages", "write", PermissionScope::Shared)
    }

    pub fn manage_contacts() -> Permission {
        Permission::new("contacts", "*", PermissionScope::Own)
    }

    pub fn dht_operations() -> Permission {
        Permission::new("dht", "*", PermissionScope::Shared)
    }

    pub fn admin_operations() -> Permission {
        Permission::new("*", "*", PermissionScope::All)
    }

    pub fn identity_management() -> Permission {
        Permission::new("identity", "*", PermissionScope::Own)
    }

    pub fn file_storage() -> Permission {
        Permission::new("storage", "*", PermissionScope::Own)
    }
}

/// Macro for protecting Tauri commands with authentication
#[macro_export]
macro_rules! require_auth {
    ($auth_middleware:expr, $session_id:expr, $permission:expr) => {
        match $auth_middleware.require_permission($session_id, &$permission) {
            Ok(session) => session,
            Err(e) => return Err(format!("Authentication failed: {}", e)),
        }
    };
}

/// Macro for protecting Tauri commands with session validation only
#[macro_export]
macro_rules! require_session {
    ($auth_middleware:expr, $session_id:expr) => {
        match $auth_middleware.validate_session($session_id) {
            Ok(session) => session,
            Err(e) => return Err(format!("Session validation failed: {}", e)),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_matching() {
        let admin_perm = Permission::new("*", "*", PermissionScope::All);
        let read_messages = Permission::new("messages", "read", PermissionScope::Shared);

        assert!(admin_perm.allows(&read_messages));
        assert!(!read_messages.allows(&admin_perm));
    }

    #[test]
    fn test_session_creation_and_validation() {
        let auth = AuthMiddleware::new();
        let permissions = vec![permissions::read_messages(), permissions::send_messages()];

        let session_id = auth
            .create_session(
                "test_user".to_string(),
                "hello-world-test-net".to_string(),
                permissions,
            )
            .unwrap();

        let session = auth.validate_session(&session_id).unwrap();
        assert_eq!(session.user_id, "test_user");
        assert!(session.has_permission(&permissions::read_messages()));
    }

    #[test]
    fn test_permission_checking() {
        let auth = AuthMiddleware::new();
        let permissions = vec![permissions::read_messages()];

        let session_id = auth
            .create_session(
                "test_user".to_string(),
                "hello-world-test-net".to_string(),
                permissions,
            )
            .unwrap();

        assert!(
            auth.check_permission(&session_id, &permissions::read_messages())
                .unwrap()
        );
        assert!(
            !auth
                .check_permission(&session_id, &permissions::admin_operations())
                .unwrap()
        );
    }

    #[test]
    fn test_session_expiry() {
        let auth = AuthMiddleware::new();
        let permissions = vec![permissions::read_messages()];

        let session_id = auth
            .create_session(
                "test_user".to_string(),
                "hello-world-test-net".to_string(),
                permissions,
            )
            .unwrap();

        // Session should be valid initially
        assert!(auth.validate_session(&session_id).is_ok());

        // Manually expire the session for testing
        {
            let mut sessions = auth.sessions.write().unwrap();
            if let Some(session) = sessions.get_mut(&session_id) {
                session.expires_at = Instant::now() - Duration::from_secs(1);
            }
        }

        // Session should now be invalid
        assert!(auth.validate_session(&session_id).is_err());
    }
}
