// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Authentication and session management for MCP server
//!
//! Provides a state machine for handling authentication before exposing tools.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use thiserror::Error;

/// Errors that can occur during authentication operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// Attempted an invalid state transition
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },

    /// User is not authenticated
    #[error("Not authenticated")]
    NotAuthenticated,

    /// Already authenticated
    #[error("Already authenticated")]
    AlreadyAuthenticated,

    /// Invalid four-word format
    #[error("Invalid four-word format: {0}")]
    InvalidFourWords(String),

    /// Session has expired
    #[error("Session has expired")]
    SessionExpired,

    /// Invalid credentials provided
    #[error("Invalid credentials")]
    InvalidCredentials,
}

/// Result type for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// MCP Server authentication state
/// Session data stored for future get_session API
#[derive(Debug, Clone, Default)]
pub enum AuthState {
    /// Server running, waiting for authentication
    /// Only pre-auth tools available: authenticate, create_vault, authenticate_token
    #[default]
    Unauthenticated,

    /// User authenticated with full access
    Authenticated(AuthenticatedSession),

    /// Demo mode - auto-initialized with temporary identity
    /// All tools available without explicit authentication
    DemoMode(DemoSession),

    /// Delegate session with scoped access
    Delegate(DelegateSession),
}

impl AuthState {
    /// Check if the state is authenticated (any variant except Unauthenticated)
    pub fn is_authenticated(&self) -> bool {
        !matches!(self, AuthState::Unauthenticated)
    }

    /// Get the state name for error messages
    pub fn state_name(&self) -> &'static str {
        match self {
            AuthState::Unauthenticated => "Unauthenticated",
            AuthState::Authenticated(_) => "Authenticated",
            AuthState::DemoMode(_) => "DemoMode",
            AuthState::Delegate(_) => "Delegate",
        }
    }

    /// Transition to authenticated state from unauthenticated
    pub fn authenticate(&mut self, session: AuthenticatedSession) -> AuthResult<()> {
        match self {
            AuthState::Unauthenticated => {
                *self = AuthState::Authenticated(session);
                Ok(())
            }
            _ => Err(AuthError::InvalidTransition {
                from: self.state_name().to_string(),
                to: "Authenticated".to_string(),
            }),
        }
    }

    /// Transition to demo mode from unauthenticated
    pub fn start_demo(&mut self, session: DemoSession) -> AuthResult<()> {
        match self {
            AuthState::Unauthenticated => {
                *self = AuthState::DemoMode(session);
                Ok(())
            }
            _ => Err(AuthError::InvalidTransition {
                from: self.state_name().to_string(),
                to: "DemoMode".to_string(),
            }),
        }
    }

    /// Transition to delegate mode from unauthenticated
    pub fn delegate(&mut self, session: DelegateSession) -> AuthResult<()> {
        match self {
            AuthState::Unauthenticated => {
                *self = AuthState::Delegate(session);
                Ok(())
            }
            _ => Err(AuthError::InvalidTransition {
                from: self.state_name().to_string(),
                to: "Delegate".to_string(),
            }),
        }
    }

    /// Revoke authentication and return to unauthenticated state
    pub fn revoke(&mut self) -> AuthResult<()> {
        match self {
            AuthState::Unauthenticated => Err(AuthError::NotAuthenticated),
            _ => {
                *self = AuthState::Unauthenticated;
                Ok(())
            }
        }
    }

    /// Get the four-word identity for the current session
    pub fn four_words(&self) -> Option<&str> {
        match self {
            AuthState::Authenticated(session) => Some(&session.four_words),
            AuthState::DemoMode(session) => Some(&session.four_words),
            AuthState::Delegate(session) => Some(&session.issuer_four_words),
            AuthState::Unauthenticated => None,
        }
    }

    /// Get session start time if authenticated
    pub fn started_at(&self) -> Option<SystemTime> {
        match self {
            AuthState::Authenticated(session) => Some(session.started_at),
            AuthState::DemoMode(session) => Some(session.started_at),
            AuthState::Delegate(session) => Some(session.started_at),
            AuthState::Unauthenticated => None,
        }
    }
}
/// Authenticated session with user credentials
/// Fields stored for future get_session API and session management
#[derive(Debug, Clone)]
pub struct AuthenticatedSession {
    /// User's four-word identity
    pub four_words: String,
    /// Display name
    pub display_name: String,
    /// Device name for this session
    pub device_name: String,
    /// Session start time
    pub started_at: SystemTime,
    /// Storage directory for this user
    pub storage_dir: String,
}

impl AuthenticatedSession {
    /// Create a new authenticated session
    pub fn new(
        four_words: String,
        display_name: String,
        device_name: String,
        storage_dir: String,
    ) -> AuthResult<Self> {
        FourWordChallenge::validate(&four_words)?;
        Ok(Self {
            four_words,
            display_name,
            device_name,
            started_at: SystemTime::now(),
            storage_dir,
        })
    }

    /// Check if the session has expired (sessions expire after 24 hours by default)
    pub fn is_expired(&self, max_duration_secs: u64) -> bool {
        self.started_at
            .elapsed()
            .map(|d| d.as_secs() > max_duration_secs)
            .unwrap_or(true)
    }
}

/// Demo mode session with auto-generated identity
/// Fields stored for future get_session API and session management
#[derive(Debug, Clone)]
pub struct DemoSession {
    /// Auto-generated four-word identity
    pub four_words: String,
    /// Default display name
    pub display_name: String,
    /// Session start time
    pub started_at: SystemTime,
    /// Temporary storage directory
    pub storage_dir: String,
}

impl DemoSession {
    /// Create a new demo session
    pub fn new(four_words: String, display_name: String, storage_dir: String) -> Self {
        Self {
            four_words,
            display_name,
            started_at: SystemTime::now(),
            storage_dir,
        }
    }
}

/// Delegate session with scoped access
#[derive(Debug, Clone)]
pub struct DelegateSession {
    /// Issuer's four-word identity
    pub issuer_four_words: String,
    /// Name of the delegate
    pub delegate_name: String,
    /// Scopes granted to this delegate
    pub scopes: Vec<Scope>,
    /// Session start time
    pub started_at: SystemTime,
    /// Storage directory
    pub storage_dir: String,
}

impl DelegateSession {
    /// Create a new delegate session
    pub fn new(
        issuer_four_words: String,
        delegate_name: String,
        scopes: Vec<Scope>,
        storage_dir: String,
    ) -> AuthResult<Self> {
        FourWordChallenge::validate(&issuer_four_words)?;
        Ok(Self {
            issuer_four_words,
            delegate_name,
            scopes,
            started_at: SystemTime::now(),
            storage_dir,
        })
    }

    /// Check if the delegate has a specific scope
    pub fn has_scope(&self, scope: &Scope) -> bool {
        self.scopes.contains(&Scope::Full) || self.scopes.contains(scope)
    }
}

/// Delegate token for AI agents with scoped access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateToken {
    /// Issuer's four-word identity
    pub issuer: String,
    /// Name for this delegate (e.g., "my-claude-agent")
    pub delegate_name: String,
    /// Scopes granted to this token
    pub scopes: Vec<Scope>,
    /// Token creation timestamp
    pub issued_at: u64,
    /// Token expiration timestamp
    pub expires_at: u64,
    /// Random nonce for uniqueness
    pub nonce: String,
}

impl DelegateToken {
    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now >= self.expires_at
    }

    /// Check if the token has a specific scope
    pub fn has_scope(&self, scope: &Scope) -> bool {
        self.scopes.contains(&Scope::Full) || self.scopes.contains(scope)
    }

    /// Check if the token is valid (not expired and has required scope)
    pub fn is_valid_for(&self, scope: &Scope) -> bool {
        !self.is_expired() && self.has_scope(scope)
    }
}
/// Access scopes for delegate tokens
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Read messages in entities
    ReadMessages,
    /// Send messages to entities
    SendMessages,
    /// Read files from virtual disks
    ReadFiles,
    /// Write files to virtual disks
    WriteFiles,
    /// Manage entities (create, update, delete)
    ManageEntities,
    /// Manage members (add, remove)
    ManageMembers,
    /// Manage Kanban boards and cards
    ManageKanban,
    /// Network operations (start, connect, disconnect)
    ManageNetwork,
    /// Manage contacts
    ManageContacts,
    /// Full access (includes all scopes)
    Full,
}

impl Scope {
    /// Parse scope from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read_messages" => Some(Self::ReadMessages),
            "send_messages" => Some(Self::SendMessages),
            "read_files" => Some(Self::ReadFiles),
            "write_files" => Some(Self::WriteFiles),
            "manage_entities" => Some(Self::ManageEntities),
            "manage_members" => Some(Self::ManageMembers),
            "manage_kanban" => Some(Self::ManageKanban),
            "manage_network" => Some(Self::ManageNetwork),
            "manage_contacts" => Some(Self::ManageContacts),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    /// Get the string representation of the scope
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ReadMessages => "read_messages",
            Self::SendMessages => "send_messages",
            Self::ReadFiles => "read_files",
            Self::WriteFiles => "write_files",
            Self::ManageEntities => "manage_entities",
            Self::ManageMembers => "manage_members",
            Self::ManageKanban => "manage_kanban",
            Self::ManageNetwork => "manage_network",
            Self::ManageContacts => "manage_contacts",
            Self::Full => "full",
        }
    }
}

/// Four-word challenge validation
pub struct FourWordChallenge;

impl FourWordChallenge {
    /// Minimum word length
    const MIN_WORD_LENGTH: usize = 2;

    /// Maximum word length
    const MAX_WORD_LENGTH: usize = 20;

    /// Validate a four-word identity string
    pub fn validate(input: &str) -> AuthResult<()> {
        let words: Vec<&str> = input.split(['-', '.', ' ']).collect();

        // Must have exactly 4 words
        if words.len() != 4 {
            return Err(AuthError::InvalidFourWords(format!(
                "Expected 4 words, got {}",
                words.len()
            )));
        }

        for (i, word) in words.iter().enumerate() {
            // Check word length
            if word.len() < Self::MIN_WORD_LENGTH {
                return Err(AuthError::InvalidFourWords(format!(
                    "Word {} '{}' is too short (min {} chars)",
                    i + 1,
                    word,
                    Self::MIN_WORD_LENGTH
                )));
            }

            if word.len() > Self::MAX_WORD_LENGTH {
                return Err(AuthError::InvalidFourWords(format!(
                    "Word {} '{}' is too long (max {} chars)",
                    i + 1,
                    word,
                    Self::MAX_WORD_LENGTH
                )));
            }

            // Check for valid characters (lowercase alpha only)
            if !word.chars().all(|c| c.is_ascii_lowercase()) {
                return Err(AuthError::InvalidFourWords(format!(
                    "Word {} '{}' contains invalid characters (only lowercase a-z allowed)",
                    i + 1,
                    word
                )));
            }
        }

        Ok(())
    }

    /// Normalize a four-word string to canonical format (lowercase, hyphen-separated)
    pub fn normalize(input: &str) -> AuthResult<String> {
        let normalized = input.to_lowercase();
        let words: Vec<&str> = normalized.split(['-', '.', ' ']).collect();

        if words.len() != 4 {
            return Err(AuthError::InvalidFourWords(format!(
                "Expected 4 words, got {}",
                words.len()
            )));
        }

        let result = words.join("-");
        Self::validate(&result)?;
        Ok(result)
    }
}
/// Tools that are available before authentication
pub const PRE_AUTH_TOOLS: &[&str] = &[
    "authenticate",
    "create_vault",
    "authenticate_token",
    "health_check",
    "core_status",
    "list_vaults",
    "delete_vault",
    "import_vault",
];

/// Check if a tool requires authentication
pub fn requires_auth(tool_name: &str) -> bool {
    !PRE_AUTH_TOOLS.contains(&tool_name)
}

/// Required scope for each tool (None means any authenticated session can use it)
pub fn required_scope(tool_name: &str) -> Option<Scope> {
    match tool_name {
        // Message tools
        "send_message" => Some(Scope::SendMessages),
        "get_messages" => Some(Scope::ReadMessages),

        // File tools
        "write_file" => Some(Scope::WriteFiles),
        "read_file" | "list_files" => Some(Scope::ReadFiles),
        "delete_file" | "move_file" => Some(Scope::WriteFiles),

        // Entity tools
        "create_entity" | "update_entity" | "delete_entity" => Some(Scope::ManageEntities),
        "get_entity" | "list_entities" | "join_entity" => None, // Read-only, any auth

        // Member tools
        "add_member" | "remove_member" => Some(Scope::ManageMembers),
        "list_members" => None, // Read-only

        // Kanban tools
        "create_kanban_board"
        | "update_kanban_board"
        | "delete_kanban_board"
        | "create_kanban_column"
        | "update_kanban_column"
        | "delete_kanban_column"
        | "move_kanban_column"
        | "create_kanban_card"
        | "update_kanban_card"
        | "delete_kanban_card"
        | "move_kanban_card"
        | "change_card_state"
        | "assign_card"
        | "unassign_card"
        | "create_tag"
        | "tag_card"
        | "untag_card"
        | "add_card_step"
        | "toggle_card_step"
        | "delete_card_step"
        | "add_card_comment"
        | "delete_card_comment" => Some(Scope::ManageKanban),
        "get_kanban_board" | "get_kanban_column" | "get_kanban_card" | "list_tags"
        | "list_card_steps" | "list_card_comments" => None, // Read-only

        // Network tools
        "network_start"
        | "network_stop"
        | "network_connect"
        | "network_disconnect"
        | "network_request_external_address" => Some(Scope::ManageNetwork),
        "network_status" | "network_peers" => None, // Read-only

        // Contact tools
        "create_contact" | "update_contact" | "delete_contact" | "link_contact" => {
            Some(Scope::ManageContacts)
        }
        "get_contact" | "list_contacts" => None, // Read-only

        // Invite tools
        "create_invite" | "accept_invite" => Some(Scope::ManageMembers),
        "list_pending_invites" => None, // Read-only

        // Profile tools
        "get_profile" | "update_profile" => None, // Always allowed

        // Session tools
        "get_session" | "logout" => None, // Always allowed
        "create_delegate_token" => Some(Scope::Full), // Only full access can create tokens

        _ => None, // Unknown tools - allow if authenticated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================
    // State Transition Tests
    // =============================================

    mod state_transitions {
        use super::*;

        #[test]
        fn test_initial_state_is_unauthenticated() {
            let state = AuthState::default();
            assert!(!state.is_authenticated());
            assert_eq!(state.state_name(), "Unauthenticated");
        }

        #[test]
        fn test_unauthenticated_to_authenticated() {
            let mut state = AuthState::default();
            let session = AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            };

            let result = state.authenticate(session);
            assert!(result.is_ok());
            assert!(state.is_authenticated());
            assert_eq!(state.state_name(), "Authenticated");
            assert_eq!(state.four_words(), Some("ocean-forest-moon-star"));
        }

        #[test]
        fn test_unauthenticated_to_demo_mode() {
            let mut state = AuthState::default();
            let session = DemoSession::new(
                "demo-four-word-test".to_string(),
                "Demo User".to_string(),
                "/tmp/demo".to_string(),
            );

            let result = state.start_demo(session);
            assert!(result.is_ok());
            assert!(state.is_authenticated());
            assert_eq!(state.state_name(), "DemoMode");
        }

        #[test]
        fn test_unauthenticated_to_delegate() {
            let mut state = AuthState::default();
            let session = DelegateSession {
                issuer_four_words: "ocean-forest-moon-star".to_string(),
                delegate_name: "test-agent".to_string(),
                scopes: vec![Scope::ReadMessages],
                started_at: SystemTime::now(),
                storage_dir: "/tmp/delegate".to_string(),
            };

            let result = state.delegate(session);
            assert!(result.is_ok());
            assert!(state.is_authenticated());
            assert_eq!(state.state_name(), "Delegate");
        }

        #[test]
        fn test_authenticated_to_unauthenticated_via_revoke() {
            let mut state = AuthState::Authenticated(AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            });

            let result = state.revoke();
            assert!(result.is_ok());
            assert!(!state.is_authenticated());
            assert_eq!(state.state_name(), "Unauthenticated");
        }

        #[test]
        fn test_invalid_transition_authenticated_to_authenticated() {
            let mut state = AuthState::Authenticated(AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            });

            let new_session = AuthenticatedSession {
                four_words: "another-four-word-id".to_string(),
                display_name: "Another User".to_string(),
                device_name: "Another Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test2".to_string(),
            };

            let result = state.authenticate(new_session);
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidTransition { from, to }) => {
                    assert_eq!(from, "Authenticated");
                    assert_eq!(to, "Authenticated");
                }
                _ => panic!("Expected InvalidTransition error"),
            }
        }

        #[test]
        fn test_revoke_unauthenticated_fails() {
            let mut state = AuthState::default();
            let result = state.revoke();
            assert!(result.is_err());
            match result {
                Err(AuthError::NotAuthenticated) => {}
                _ => panic!("Expected NotAuthenticated error"),
            }
        }
    }

    // =============================================
    // Four-Word Challenge Tests
    // =============================================

    mod four_word_challenge {
        use super::*;

        #[test]
        fn test_valid_four_word_hyphen_separated() {
            let result = FourWordChallenge::validate("ocean-forest-moon-star");
            assert!(result.is_ok());
        }

        #[test]
        fn test_valid_four_word_dot_separated() {
            let result = FourWordChallenge::validate("ocean.forest.moon.star");
            assert!(result.is_ok());
        }

        #[test]
        fn test_valid_four_word_space_separated() {
            let result = FourWordChallenge::validate("ocean forest moon star");
            assert!(result.is_ok());
        }

        #[test]
        fn test_invalid_three_words() {
            let result = FourWordChallenge::validate("ocean-forest-moon");
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidFourWords(msg)) => {
                    assert!(msg.contains("Expected 4 words, got 3"));
                }
                _ => panic!("Expected InvalidFourWords error"),
            }
        }

        #[test]
        fn test_invalid_five_words() {
            let result = FourWordChallenge::validate("ocean-forest-moon-star-extra");
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidFourWords(msg)) => {
                    assert!(msg.contains("Expected 4 words, got 5"));
                }
                _ => panic!("Expected InvalidFourWords error"),
            }
        }

        #[test]
        fn test_invalid_empty_string() {
            let result = FourWordChallenge::validate("");
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_single_word() {
            let result = FourWordChallenge::validate("ocean");
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_uppercase_characters() {
            let result = FourWordChallenge::validate("Ocean-Forest-Moon-Star");
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidFourWords(msg)) => {
                    assert!(msg.contains("invalid characters"));
                }
                _ => panic!("Expected InvalidFourWords error"),
            }
        }

        #[test]
        fn test_invalid_numeric_characters() {
            let result = FourWordChallenge::validate("ocean-forest-moon-star123");
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidFourWords(msg)) => {
                    assert!(msg.contains("invalid characters"));
                }
                _ => panic!("Expected InvalidFourWords error"),
            }
        }

        #[test]
        fn test_invalid_special_characters() {
            let result = FourWordChallenge::validate("ocean-forest-moon-star!");
            assert!(result.is_err());
        }

        #[test]
        fn test_invalid_word_too_short() {
            let result = FourWordChallenge::validate("a-forest-moon-star");
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidFourWords(msg)) => {
                    assert!(msg.contains("too short"));
                }
                _ => panic!("Expected InvalidFourWords error"),
            }
        }

        #[test]
        fn test_invalid_word_too_long() {
            let result =
                FourWordChallenge::validate("supercalifragilisticexpialidocious-forest-moon-star");
            assert!(result.is_err());
            match result {
                Err(AuthError::InvalidFourWords(msg)) => {
                    assert!(msg.contains("too long"));
                }
                _ => panic!("Expected InvalidFourWords error"),
            }
        }

        #[test]
        fn test_normalize_uppercase_to_lowercase() {
            let result = FourWordChallenge::normalize("Ocean-Forest-Moon-Star");
            assert!(result.is_ok());
            assert_eq!(result.ok(), Some("ocean-forest-moon-star".to_string()));
        }

        #[test]
        fn test_normalize_mixed_separators() {
            let result = FourWordChallenge::normalize("ocean.forest moon-star");
            assert!(result.is_ok());
            assert_eq!(result.ok(), Some("ocean-forest-moon-star".to_string()));
        }

        #[test]
        fn test_normalize_invalid_word_count() {
            let result = FourWordChallenge::normalize("ocean-forest-moon");
            assert!(result.is_err());
        }
    }

    // =============================================
    // Session Lifecycle Tests
    // =============================================

    mod session_lifecycle {
        use super::*;
        use std::time::Duration;

        #[test]
        fn test_authenticated_session_creation() {
            let result = AuthenticatedSession::new(
                "ocean-forest-moon-star".to_string(),
                "Test User".to_string(),
                "Test Device".to_string(),
                "/tmp/test".to_string(),
            );
            assert!(result.is_ok());
            let session = result.ok();
            assert!(session.is_some());
            let session = session.as_ref();
            assert_eq!(
                session.map(|s| s.four_words.as_str()),
                Some("ocean-forest-moon-star")
            );
        }

        #[test]
        fn test_authenticated_session_invalid_four_words() {
            let result = AuthenticatedSession::new(
                "invalid".to_string(),
                "Test User".to_string(),
                "Test Device".to_string(),
                "/tmp/test".to_string(),
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_session_expiration_not_expired() {
            let session = AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            };
            assert!(!session.is_expired(86400));
        }

        #[test]
        fn test_session_expiration_with_zero_duration() {
            let session = AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now() - Duration::from_secs(1),
                storage_dir: "/tmp/test".to_string(),
            };
            assert!(session.is_expired(0));
        }

        #[test]
        fn test_demo_session_creation() {
            let session = DemoSession::new(
                "demo-test-four-word".to_string(),
                "Demo User".to_string(),
                "/tmp/demo".to_string(),
            );
            assert_eq!(session.four_words, "demo-test-four-word");
            assert_eq!(session.display_name, "Demo User");
        }

        #[test]
        fn test_delegate_session_creation() {
            let result = DelegateSession::new(
                "ocean-forest-moon-star".to_string(),
                "test-agent".to_string(),
                vec![Scope::ReadMessages, Scope::SendMessages],
                "/tmp/delegate".to_string(),
            );
            assert!(result.is_ok());
        }

        #[test]
        fn test_delegate_session_invalid_issuer() {
            let result = DelegateSession::new(
                "invalid".to_string(),
                "test-agent".to_string(),
                vec![Scope::Full],
                "/tmp/delegate".to_string(),
            );
            assert!(result.is_err());
        }

        #[test]
        fn test_delegate_session_scope_check() {
            let session = DelegateSession {
                issuer_four_words: "ocean-forest-moon-star".to_string(),
                delegate_name: "test-agent".to_string(),
                scopes: vec![Scope::ReadMessages, Scope::SendMessages],
                started_at: SystemTime::now(),
                storage_dir: "/tmp/delegate".to_string(),
            };

            assert!(session.has_scope(&Scope::ReadMessages));
            assert!(session.has_scope(&Scope::SendMessages));
            assert!(!session.has_scope(&Scope::WriteFiles));
        }

        #[test]
        fn test_delegate_session_full_scope_grants_all() {
            let session = DelegateSession {
                issuer_four_words: "ocean-forest-moon-star".to_string(),
                delegate_name: "test-agent".to_string(),
                scopes: vec![Scope::Full],
                started_at: SystemTime::now(),
                storage_dir: "/tmp/delegate".to_string(),
            };

            assert!(session.has_scope(&Scope::ReadMessages));
            assert!(session.has_scope(&Scope::WriteFiles));
            assert!(session.has_scope(&Scope::ManageNetwork));
            assert!(session.has_scope(&Scope::Full));
        }

        #[test]
        fn test_auth_state_four_words_accessor() {
            let state = AuthState::Authenticated(AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test".to_string(),
                device_name: "Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp".to_string(),
            });
            assert_eq!(state.four_words(), Some("ocean-forest-moon-star"));

            let unauth = AuthState::Unauthenticated;
            assert_eq!(unauth.four_words(), None);
        }

        #[test]
        fn test_auth_state_started_at_accessor() {
            let now = SystemTime::now();
            let state = AuthState::Authenticated(AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test".to_string(),
                device_name: "Device".to_string(),
                started_at: now,
                storage_dir: "/tmp".to_string(),
            });
            assert!(state.started_at().is_some());

            let unauth = AuthState::Unauthenticated;
            assert!(unauth.started_at().is_none());
        }
    }

    // =============================================
    // Delegate Token Tests
    // =============================================

    mod delegate_token {
        use super::*;

        fn create_test_token(expires_at: u64, scopes: Vec<Scope>) -> DelegateToken {
            DelegateToken {
                issuer: "ocean-forest-moon-star".to_string(),
                delegate_name: "test-agent".to_string(),
                scopes,
                issued_at: 0,
                expires_at,
                nonce: "test-nonce".to_string(),
            }
        }

        #[test]
        fn test_token_not_expired() {
            let token = create_test_token(u64::MAX, vec![Scope::Full]);
            assert!(!token.is_expired());
        }

        #[test]
        fn test_token_expired() {
            let token = create_test_token(0, vec![Scope::Full]);
            assert!(token.is_expired());
        }

        #[test]
        fn test_token_has_specific_scope() {
            let token = create_test_token(u64::MAX, vec![Scope::ReadMessages, Scope::SendMessages]);
            assert!(token.has_scope(&Scope::ReadMessages));
            assert!(token.has_scope(&Scope::SendMessages));
            assert!(!token.has_scope(&Scope::WriteFiles));
        }

        #[test]
        fn test_token_full_scope_grants_all() {
            let token = create_test_token(u64::MAX, vec![Scope::Full]);
            assert!(token.has_scope(&Scope::ReadMessages));
            assert!(token.has_scope(&Scope::WriteFiles));
            assert!(token.has_scope(&Scope::ManageNetwork));
        }

        #[test]
        fn test_token_is_valid_for_scope() {
            let token = create_test_token(u64::MAX, vec![Scope::ReadMessages]);
            assert!(token.is_valid_for(&Scope::ReadMessages));
            assert!(!token.is_valid_for(&Scope::WriteFiles));
        }

        #[test]
        fn test_expired_token_not_valid() {
            let token = create_test_token(0, vec![Scope::Full]);
            assert!(!token.is_valid_for(&Scope::ReadMessages));
        }
    }

    // =============================================
    // Scope Tests
    // =============================================

    mod scope_tests {
        use super::*;

        #[test]
        fn test_scope_parse_all_variants() {
            assert_eq!(Scope::parse("read_messages"), Some(Scope::ReadMessages));
            assert_eq!(Scope::parse("send_messages"), Some(Scope::SendMessages));
            assert_eq!(Scope::parse("read_files"), Some(Scope::ReadFiles));
            assert_eq!(Scope::parse("write_files"), Some(Scope::WriteFiles));
            assert_eq!(Scope::parse("manage_entities"), Some(Scope::ManageEntities));
            assert_eq!(Scope::parse("manage_members"), Some(Scope::ManageMembers));
            assert_eq!(Scope::parse("manage_kanban"), Some(Scope::ManageKanban));
            assert_eq!(Scope::parse("manage_network"), Some(Scope::ManageNetwork));
            assert_eq!(Scope::parse("manage_contacts"), Some(Scope::ManageContacts));
            assert_eq!(Scope::parse("full"), Some(Scope::Full));
        }

        #[test]
        fn test_scope_parse_case_insensitive() {
            assert_eq!(Scope::parse("FULL"), Some(Scope::Full));
            assert_eq!(Scope::parse("Full"), Some(Scope::Full));
            assert_eq!(Scope::parse("READ_MESSAGES"), Some(Scope::ReadMessages));
        }

        #[test]
        fn test_scope_parse_invalid() {
            assert_eq!(Scope::parse("invalid"), None);
            assert_eq!(Scope::parse(""), None);
            assert_eq!(Scope::parse("readmessages"), None);
        }

        #[test]
        fn test_scope_as_str_roundtrip() {
            let scopes = [
                Scope::ReadMessages,
                Scope::SendMessages,
                Scope::ReadFiles,
                Scope::WriteFiles,
                Scope::ManageEntities,
                Scope::ManageMembers,
                Scope::ManageKanban,
                Scope::ManageNetwork,
                Scope::ManageContacts,
                Scope::Full,
            ];

            for scope in &scopes {
                let s = scope.as_str();
                let parsed = Scope::parse(s);
                assert_eq!(parsed, Some(scope.clone()));
            }
        }
    }

    // =============================================
    // Pre-Auth Tools Tests
    // =============================================

    mod pre_auth_tools {
        use super::*;

        #[test]
        fn test_pre_auth_tools_not_require_auth() {
            for tool in PRE_AUTH_TOOLS {
                assert!(
                    !requires_auth(tool),
                    "Tool '{}' should not require auth",
                    tool
                );
            }
        }

        #[test]
        fn test_authenticated_tools_require_auth() {
            let auth_tools = [
                "send_message",
                "get_messages",
                "create_entity",
                "write_file",
                "read_file",
            ];

            for tool in &auth_tools {
                assert!(requires_auth(tool), "Tool '{}' should require auth", tool);
            }
        }

        #[test]
        fn test_unknown_tool_requires_auth() {
            assert!(requires_auth("unknown_tool"));
        }
    }

    // =============================================
    // Required Scope Tests
    // =============================================

    mod required_scope_tests {
        use super::*;

        #[test]
        fn test_message_tools_scopes() {
            assert_eq!(required_scope("send_message"), Some(Scope::SendMessages));
            assert_eq!(required_scope("get_messages"), Some(Scope::ReadMessages));
        }

        #[test]
        fn test_file_tools_scopes() {
            assert_eq!(required_scope("write_file"), Some(Scope::WriteFiles));
            assert_eq!(required_scope("read_file"), Some(Scope::ReadFiles));
            assert_eq!(required_scope("list_files"), Some(Scope::ReadFiles));
            assert_eq!(required_scope("delete_file"), Some(Scope::WriteFiles));
        }

        #[test]
        fn test_entity_tools_scopes() {
            assert_eq!(required_scope("create_entity"), Some(Scope::ManageEntities));
            assert_eq!(required_scope("get_entity"), None); // Read-only
            assert_eq!(required_scope("list_entities"), None); // Read-only
        }

        #[test]
        fn test_kanban_tools_scopes() {
            assert_eq!(
                required_scope("create_kanban_board"),
                Some(Scope::ManageKanban)
            );
            assert_eq!(required_scope("get_kanban_board"), None); // Read-only
        }

        #[test]
        fn test_network_tools_scopes() {
            assert_eq!(required_scope("network_start"), Some(Scope::ManageNetwork));
            assert_eq!(required_scope("network_status"), None); // Read-only
        }

        #[test]
        fn test_session_tools_no_scope() {
            assert_eq!(required_scope("get_session"), None);
            assert_eq!(required_scope("logout"), None);
        }

        #[test]
        fn test_create_delegate_token_requires_full() {
            assert_eq!(required_scope("create_delegate_token"), Some(Scope::Full));
        }

        #[test]
        fn test_unknown_tool_no_required_scope() {
            assert_eq!(required_scope("unknown_tool"), None);
        }
    }

    // =============================================
    // Error Type Tests
    // =============================================

    mod error_tests {
        use super::*;

        #[test]
        fn test_auth_error_display() {
            let err = AuthError::NotAuthenticated;
            assert_eq!(format!("{}", err), "Not authenticated");

            let err = AuthError::InvalidTransition {
                from: "A".to_string(),
                to: "B".to_string(),
            };
            assert_eq!(format!("{}", err), "Invalid state transition from A to B");

            let err = AuthError::InvalidFourWords("test error".to_string());
            assert_eq!(format!("{}", err), "Invalid four-word format: test error");
        }

        #[test]
        fn test_auth_error_equality() {
            let err1 = AuthError::NotAuthenticated;
            let err2 = AuthError::NotAuthenticated;
            assert_eq!(err1, err2);

            let err3 = AuthError::AlreadyAuthenticated;
            assert_ne!(err1, err3);
        }

        #[test]
        fn test_auth_error_clone() {
            let err = AuthError::InvalidFourWords("test".to_string());
            let cloned = err.clone();
            assert_eq!(err, cloned);
        }
    }
}
