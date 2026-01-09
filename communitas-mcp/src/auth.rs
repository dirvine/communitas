// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Authentication and session management for MCP server
//!
//! Provides a state machine for handling authentication before exposing tools.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// MCP Server authentication state
/// Session data stored for future get_session API
#[allow(dead_code)]
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

    Delegate(DelegateSession),
}

/// Authenticated session with user credentials
/// Fields stored for future get_session API and session management
#[allow(dead_code)]
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

/// Demo mode session with auto-generated identity
/// Fields stored for future get_session API and session management
#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DelegateSession {
    pub issuer_four_words: String,
    pub delegate_name: String,
    pub scopes: Vec<Scope>,
    pub started_at: SystemTime,
    pub storage_dir: String,
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

#[allow(dead_code)]
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

#[allow(dead_code)]
impl Scope {
    /// Parse scope from string
    pub fn from_str(s: &str) -> Option<Self> {
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
/// Used for delegate token scope checking (Phase 2)
#[allow(dead_code)]
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

    #[test]
    fn test_pre_auth_tools() {
        assert!(!requires_auth("authenticate"));
        assert!(!requires_auth("create_vault"));
        assert!(!requires_auth("authenticate_token"));
        assert!(requires_auth("send_message"));
        assert!(requires_auth("create_entity"));
    }

    #[test]
    fn test_scope_parsing() {
        assert_eq!(Scope::from_str("full"), Some(Scope::Full));
        assert_eq!(Scope::from_str("read_messages"), Some(Scope::ReadMessages));
        assert_eq!(Scope::from_str("FULL"), Some(Scope::Full));
        assert_eq!(Scope::from_str("invalid"), None);
    }

    #[test]
    fn test_delegate_token_scope() {
        let token = DelegateToken {
            issuer: "test.four.word.id".to_string(),
            delegate_name: "test-agent".to_string(),
            scopes: vec![Scope::ReadMessages, Scope::SendMessages],
            issued_at: 0,
            expires_at: u64::MAX,
            nonce: "test".to_string(),
        };

        assert!(token.has_scope(&Scope::ReadMessages));
        assert!(token.has_scope(&Scope::SendMessages));
        assert!(!token.has_scope(&Scope::WriteFiles));
    }

    #[test]
    fn test_full_scope_grants_all() {
        let token = DelegateToken {
            issuer: "test.four.word.id".to_string(),
            delegate_name: "test-agent".to_string(),
            scopes: vec![Scope::Full],
            issued_at: 0,
            expires_at: u64::MAX,
            nonce: "test".to_string(),
        };

        assert!(token.has_scope(&Scope::ReadMessages));
        assert!(token.has_scope(&Scope::WriteFiles));
        assert!(token.has_scope(&Scope::ManageNetwork));
    }
}
