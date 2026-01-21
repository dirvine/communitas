//! Authentication middleware for core command adapters
//!
//! This module provides:
//! - Session-based authentication
//! - Role-based access control
//! - Secure session management
//! - Protection against unauthorized command execution
//! - Failed login tracking with exponential backoff
//! - Temporary lockout after repeated failed attempts

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Maximum session duration (1 hour)
pub const MAX_SESSION_DURATION: Duration = Duration::from_secs(3600);

/// Session cleanup interval (5 minutes)
pub const SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

/// Maximum failed login attempts before lockout
pub const MAX_FAILED_ATTEMPTS: u32 = 5;

/// Lockout duration after max failed attempts (15 minutes)
pub const LOCKOUT_DURATION: Duration = Duration::from_secs(900);

/// Base delay for exponential backoff (1 second)
pub const BASE_BACKOFF_DELAY: Duration = Duration::from_secs(1);

/// Maximum backoff delay (30 seconds)
pub const MAX_BACKOFF_DELAY: Duration = Duration::from_secs(30);

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
}

/// Tracks failed login attempts for an identity
#[derive(Debug, Clone)]
pub struct LoginAttemptRecord {
    /// Number of consecutive failed attempts
    pub failed_count: u32,

    /// Time of the first failed attempt in this sequence
    pub first_failed_at: Instant,

    /// Time of the most recent failed attempt
    pub last_failed_at: Instant,

    /// Whether this identity is currently locked out
    pub locked_out: bool,

    /// When the lockout expires (if locked)
    pub lockout_expires_at: Option<Instant>,
}

impl LoginAttemptRecord {
    /// Create a new empty record (call record_failure to register first failure)
    fn new() -> Self {
        let now = Instant::now();
        Self {
            failed_count: 0,
            first_failed_at: now,
            last_failed_at: now,
            locked_out: false,
            lockout_expires_at: None,
        }
    }

    /// Record a failed attempt, potentially triggering lockout
    fn record_failure(&mut self) {
        let now = Instant::now();

        // Set first_failed_at only on the first failure
        if self.failed_count == 0 {
            self.first_failed_at = now;
        }

        self.failed_count += 1;
        self.last_failed_at = now;

        // Check if lockout threshold reached
        if self.failed_count >= MAX_FAILED_ATTEMPTS && !self.locked_out {
            self.locked_out = true;
            self.lockout_expires_at = Some(now + LOCKOUT_DURATION);
            warn!(
                "Identity locked out after {} failed attempts",
                self.failed_count
            );
        }
    }

    /// Calculate the current backoff delay based on failed attempts
    ///
    /// Uses exponential backoff: 1s, 2s, 4s, 8s, 16s, capped at 30s
    pub fn backoff_delay(&self) -> Duration {
        if self.failed_count == 0 {
            return Duration::ZERO;
        }

        // Calculate exponential delay: base * 2^(attempts-1)
        let multiplier = 2u64.saturating_pow(self.failed_count.saturating_sub(1));
        let delay_secs = BASE_BACKOFF_DELAY.as_secs().saturating_mul(multiplier);

        Duration::from_secs(delay_secs.min(MAX_BACKOFF_DELAY.as_secs()))
    }

    /// Check if this record is currently locked out
    pub fn is_locked_out(&self) -> bool {
        if !self.locked_out {
            return false;
        }

        // Check if lockout has expired
        if let Some(expires) = self.lockout_expires_at
            && Instant::now() >= expires
        {
            return false;
        }

        true
    }

    /// Get remaining lockout time, if any
    pub fn lockout_remaining(&self) -> Option<Duration> {
        if !self.is_locked_out() {
            return None;
        }

        self.lockout_expires_at.map(|expires| {
            let now = Instant::now();
            if now < expires {
                expires - now
            } else {
                Duration::ZERO
            }
        })
    }

    /// Reset after successful login
    fn reset(&mut self) {
        self.failed_count = 0;
        self.locked_out = false;
        self.lockout_expires_at = None;
    }
}

/// Tracks failed login attempts across all identities
#[derive(Debug, Clone)]
pub struct LoginAttemptTracker {
    /// Map from identity (four words) to attempt record
    attempts: Arc<RwLock<HashMap<String, LoginAttemptRecord>>>,

    /// Time of last cleanup
    last_cleanup: Arc<RwLock<Instant>>,
}

impl Default for LoginAttemptTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LoginAttemptTracker {
    /// Create a new login attempt tracker
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(RwLock::new(HashMap::new())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Record a failed login attempt for an identity
    ///
    /// Returns the current attempt record including backoff delay
    pub fn record_failed_attempt(&self, identity: &str) -> Result<LoginAttemptRecord> {
        let mut attempts = self
            .attempts
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        let record = attempts
            .entry(identity.to_string())
            .or_insert_with(LoginAttemptRecord::new);

        // Always record the failure (new records start with count 0)
        record.record_failure();

        debug!(
            identity = identity,
            failed_count = record.failed_count,
            backoff_secs = record.backoff_delay().as_secs(),
            "Recorded failed login attempt"
        );

        Ok(record.clone())
    }

    /// Record a successful login, clearing the failure record
    pub fn record_successful_login(&self, identity: &str) -> Result<()> {
        let mut attempts = self
            .attempts
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        if let Some(record) = attempts.get_mut(identity) {
            if record.failed_count > 0 {
                info!(
                    identity = identity,
                    previous_failures = record.failed_count,
                    "Login successful, clearing failed attempt record"
                );
            }
            record.reset();
        }

        Ok(())
    }

    /// Check if an identity is currently locked out
    ///
    /// Returns Ok(None) if not locked out, or Ok(Some(remaining_duration)) if locked
    pub fn check_lockout(&self, identity: &str) -> Result<Option<Duration>> {
        let mut attempts = self
            .attempts
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        if let Some(record) = attempts.get_mut(identity) {
            // Check if lockout has expired
            if record.locked_out
                && let Some(expires) = record.lockout_expires_at
                && Instant::now() >= expires
            {
                // Lockout expired, reset the record
                info!(
                    identity = identity,
                    "Lockout expired, resetting attempt record"
                );
                record.reset();
                return Ok(None);
            }

            return Ok(record.lockout_remaining());
        }

        Ok(None)
    }

    /// Get the current backoff delay for an identity
    pub fn get_backoff_delay(&self, identity: &str) -> Result<Duration> {
        let attempts = self
            .attempts
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        Ok(attempts
            .get(identity)
            .map(|r| r.backoff_delay())
            .unwrap_or(Duration::ZERO))
    }

    /// Get the current attempt record for an identity
    pub fn get_attempt_record(&self, identity: &str) -> Result<Option<LoginAttemptRecord>> {
        let attempts = self
            .attempts
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        Ok(attempts.get(identity).cloned())
    }

    /// Get statistics about failed login tracking
    pub fn get_stats(&self) -> Result<LoginAttemptStats> {
        let attempts = self
            .attempts
            .read()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        let total_tracked = attempts.len();
        let currently_locked = attempts.values().filter(|r| r.is_locked_out()).count();
        let with_failures = attempts.values().filter(|r| r.failed_count > 0).count();

        Ok(LoginAttemptStats {
            total_tracked,
            currently_locked,
            with_failures,
        })
    }

    /// Clean up old records that haven't had activity in a while
    pub fn cleanup_old_records(&self, max_age: Duration) -> Result<usize> {
        let now = Instant::now();

        // Check if cleanup is needed (same interval as session cleanup)
        {
            let last_cleanup = self
                .last_cleanup
                .read()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cleanup lock"))?;

            if now.duration_since(*last_cleanup) < SESSION_CLEANUP_INTERVAL {
                return Ok(0);
            }
        }

        let mut attempts = self
            .attempts
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        let before_count = attempts.len();

        // Remove records that:
        // 1. Have no failures AND
        // 2. Last activity was more than max_age ago
        attempts.retain(|_, record| {
            let age = now.duration_since(record.last_failed_at);
            // Keep if: has recent activity OR is currently locked out
            age < max_age || record.is_locked_out()
        });

        let removed = before_count - attempts.len();

        // Update last cleanup time
        {
            let mut last_cleanup = self
                .last_cleanup
                .write()
                .map_err(|_| anyhow::anyhow!("Failed to acquire cleanup lock"))?;
            *last_cleanup = now;
        }

        if removed > 0 {
            debug!(removed = removed, "Cleaned up old login attempt records");
        }

        Ok(removed)
    }

    /// Manually unlock an identity (admin function)
    pub fn unlock_identity(&self, identity: &str) -> Result<bool> {
        let mut attempts = self
            .attempts
            .write()
            .map_err(|_| anyhow::anyhow!("Failed to acquire attempts lock"))?;

        if let Some(record) = attempts.get_mut(identity)
            && record.locked_out
        {
            info!(identity = identity, "Manually unlocking identity");
            record.reset();
            return Ok(true);
        }

        Ok(false)
    }
}

/// Statistics about login attempt tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginAttemptStats {
    /// Total number of identities being tracked
    pub total_tracked: usize,

    /// Number of identities currently locked out
    pub currently_locked: usize,

    /// Number of identities with at least one failed attempt
    pub with_failures: usize,
}

impl AuthSession {
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
    login_attempts: LoginAttemptTracker,
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
            login_attempts: LoginAttemptTracker::new(),
        }
    }

    /// Get reference to the login attempt tracker
    pub fn login_tracker(&self) -> &LoginAttemptTracker {
        &self.login_attempts
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

/// Macro for protecting adapter commands with authentication
#[macro_export]
macro_rules! require_auth {
    ($auth_middleware:expr, $session_id:expr, $permission:expr) => {
        match $auth_middleware.require_permission($session_id, &$permission) {
            Ok(session) => session,
            Err(e) => return Err(format!("Authentication failed: {}", e)),
        }
    };
}

/// Macro for protecting adapter commands with session validation only
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

    // ==========================================
    // Login Attempt Tracking Tests
    // ==========================================

    #[test]
    fn test_login_attempt_record_backoff() {
        let mut record = LoginAttemptRecord::new();

        // New record starts with 0 failures, 0 delay
        assert_eq!(record.backoff_delay(), Duration::ZERO);

        // First attempt: 1 second delay
        record.record_failure();
        assert_eq!(record.backoff_delay(), Duration::from_secs(1));

        // Second attempt: 2 seconds
        record.record_failure();
        assert_eq!(record.backoff_delay(), Duration::from_secs(2));

        // Third attempt: 4 seconds
        record.record_failure();
        assert_eq!(record.backoff_delay(), Duration::from_secs(4));

        // Fourth attempt: 8 seconds
        record.record_failure();
        assert_eq!(record.backoff_delay(), Duration::from_secs(8));

        // Fifth attempt: 16 seconds
        record.record_failure();
        assert_eq!(record.backoff_delay(), Duration::from_secs(16));

        // Sixth attempt: 30 seconds (capped at MAX_BACKOFF_DELAY)
        record.record_failure();
        assert_eq!(record.backoff_delay(), MAX_BACKOFF_DELAY);
    }

    #[test]
    fn test_login_attempt_lockout_trigger() {
        let mut record = LoginAttemptRecord::new();
        assert!(!record.locked_out);
        assert_eq!(record.failed_count, 0);

        // Record failures up to threshold (5 total)
        for i in 0..MAX_FAILED_ATTEMPTS {
            record.record_failure();
            if i < MAX_FAILED_ATTEMPTS - 1 {
                assert!(!record.locked_out);
            }
        }

        // After 5 failures, should be locked out
        assert_eq!(record.failed_count, MAX_FAILED_ATTEMPTS);
        assert!(record.locked_out);
        assert!(record.lockout_expires_at.is_some());
    }

    #[test]
    fn test_login_attempt_tracker_record_and_clear() {
        let tracker = LoginAttemptTracker::new();
        let identity = "test-user-identity";

        // First failure
        let record = tracker.record_failed_attempt(identity).unwrap();
        assert_eq!(record.failed_count, 1);

        // Second failure
        let record = tracker.record_failed_attempt(identity).unwrap();
        assert_eq!(record.failed_count, 2);

        // Successful login clears record
        tracker.record_successful_login(identity).unwrap();

        // New failure should start fresh
        let record = tracker.record_failed_attempt(identity).unwrap();
        assert_eq!(record.failed_count, 1);
    }

    #[test]
    fn test_login_attempt_tracker_lockout_check() {
        let tracker = LoginAttemptTracker::new();
        let identity = "attacker-identity";

        // Should not be locked out initially
        assert!(tracker.check_lockout(identity).unwrap().is_none());

        // Record failures to trigger lockout
        for _ in 0..MAX_FAILED_ATTEMPTS {
            tracker.record_failed_attempt(identity).unwrap();
        }

        // Should now be locked out
        let lockout = tracker.check_lockout(identity).unwrap();
        assert!(lockout.is_some());
        assert!(lockout.unwrap() > Duration::ZERO);
    }

    #[test]
    fn test_login_attempt_tracker_stats() {
        let tracker = LoginAttemptTracker::new();

        // Initial stats
        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.total_tracked, 0);
        assert_eq!(stats.currently_locked, 0);
        assert_eq!(stats.with_failures, 0);

        // Add some failures
        tracker.record_failed_attempt("user1").unwrap();
        tracker.record_failed_attempt("user2").unwrap();
        tracker.record_failed_attempt("user2").unwrap();

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.total_tracked, 2);
        assert_eq!(stats.with_failures, 2);

        // Lock out one user
        for _ in 0..MAX_FAILED_ATTEMPTS {
            tracker.record_failed_attempt("attacker").unwrap();
        }

        let stats = tracker.get_stats().unwrap();
        assert_eq!(stats.total_tracked, 3);
        assert_eq!(stats.currently_locked, 1);
    }

    #[test]
    fn test_login_attempt_tracker_manual_unlock() {
        let tracker = LoginAttemptTracker::new();
        let identity = "locked-user";

        // Lock the identity
        for _ in 0..MAX_FAILED_ATTEMPTS {
            tracker.record_failed_attempt(identity).unwrap();
        }

        assert!(tracker.check_lockout(identity).unwrap().is_some());

        // Manual unlock
        let unlocked = tracker.unlock_identity(identity).unwrap();
        assert!(unlocked);

        // Should no longer be locked
        assert!(tracker.check_lockout(identity).unwrap().is_none());

        // Unlocking already unlocked identity returns false
        let unlocked = tracker.unlock_identity(identity).unwrap();
        assert!(!unlocked);
    }

    #[test]
    fn test_auth_middleware_has_login_tracker() {
        let auth = AuthMiddleware::new();

        // Should be able to access the login tracker
        let tracker = auth.login_tracker();

        // Record a failed attempt
        tracker.record_failed_attempt("test-identity").unwrap();

        let record = tracker.get_attempt_record("test-identity").unwrap();
        assert!(record.is_some());
        assert_eq!(record.unwrap().failed_count, 1);
    }

    #[test]
    fn test_backoff_delay_with_zero_failures() {
        let tracker = LoginAttemptTracker::new();

        // Unknown identity should have zero delay
        let delay = tracker.get_backoff_delay("unknown-user").unwrap();
        assert_eq!(delay, Duration::ZERO);
    }

    #[test]
    fn test_login_attempt_record_reset() {
        let mut record = LoginAttemptRecord::new();

        // Record some failures
        for _ in 0..3 {
            record.record_failure();
        }

        assert_eq!(record.failed_count, 3);

        // Reset
        record.reset();

        assert_eq!(record.failed_count, 0);
        assert!(!record.locked_out);
        assert!(record.lockout_expires_at.is_none());
    }

    #[test]
    fn test_multiple_identities_tracked_independently() {
        let tracker = LoginAttemptTracker::new();

        // User A has 3 failures
        tracker.record_failed_attempt("user-a").unwrap();
        tracker.record_failed_attempt("user-a").unwrap();
        tracker.record_failed_attempt("user-a").unwrap();

        // User B has 1 failure
        tracker.record_failed_attempt("user-b").unwrap();

        // Verify independent tracking
        let record_a = tracker.get_attempt_record("user-a").unwrap().unwrap();
        let record_b = tracker.get_attempt_record("user-b").unwrap().unwrap();

        assert_eq!(record_a.failed_count, 3);
        assert_eq!(record_b.failed_count, 1);

        // Clearing user A doesn't affect user B
        tracker.record_successful_login("user-a").unwrap();

        let record_a = tracker.get_attempt_record("user-a").unwrap().unwrap();
        let record_b = tracker.get_attempt_record("user-b").unwrap().unwrap();

        assert_eq!(record_a.failed_count, 0);
        assert_eq!(record_b.failed_count, 1);
    }
}
