// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Token Test Helpers
//!
//! Provides utilities for creating various types of delegate tokens
//! for testing scope enforcement and authorization.

#![allow(dead_code)]

use communitas_mcp::auth::Scope;
use communitas_mcp::token::TokenManager;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper for creating test tokens with various scopes
pub struct TokenTestHelper {
    manager: TokenManager,
    _temp_dir: TempDir,
    issuer: String,
}

impl TokenTestHelper {
    /// Create a new token test helper
    pub async fn new() -> Result<Self, anyhow::Error> {
        let temp_dir = TempDir::new()?;
        let manager = TokenManager::new(temp_dir.path().to_path_buf()).await?;

        Ok(Self {
            manager,
            _temp_dir: temp_dir,
            issuer: "test-issuer-four-words".to_string(),
        })
    }

    /// Create from an existing vault path
    pub async fn from_path(path: PathBuf) -> Result<Self, anyhow::Error> {
        let temp_dir = TempDir::new()?;
        let manager = TokenManager::new(path).await?;

        Ok(Self {
            manager,
            _temp_dir: temp_dir,
            issuer: "test-issuer-four-words".to_string(),
        })
    }

    /// Set the issuer for tokens
    pub fn with_issuer(mut self, issuer: &str) -> Self {
        self.issuer = issuer.to_string();
        self
    }

    /// Create a read-only token (can only read messages and files)
    pub fn create_read_only_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ReadMessages, Scope::ReadFiles],
            24,
        )
    }

    /// Create a messaging token (can read and send messages)
    pub fn create_messaging_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ReadMessages, Scope::SendMessages],
            24,
        )
    }

    /// Create a kanban-only token
    pub fn create_kanban_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ManageKanban, Scope::ReadMessages],
            24,
        )
    }

    /// Create a files-only token (read and write files)
    pub fn create_files_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ReadFiles, Scope::WriteFiles],
            24,
        )
    }

    /// Create an entities token (can manage entities and members)
    pub fn create_entities_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ManageEntities, Scope::ManageMembers],
            24,
        )
    }

    /// Create a network token (can manage network operations)
    pub fn create_network_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ManageNetwork],
            24,
        )
    }

    /// Create a contacts token (can manage contacts)
    pub fn create_contacts_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::ManageContacts],
            24,
        )
    }

    /// Create a full-access token (all permissions)
    pub fn create_full_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager
            .create_token(&self.issuer, delegate_name, vec![Scope::Full], 24)
    }

    /// Create an expired token (0-hour expiration)
    pub fn create_expired_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager.create_token(
            &self.issuer,
            delegate_name,
            vec![Scope::Full],
            0, // Expires immediately
        )
    }

    /// Create a token with custom scopes
    pub fn create_custom_token(
        &self,
        delegate_name: &str,
        scopes: Vec<Scope>,
        hours: u64,
    ) -> Result<String, anyhow::Error> {
        self.manager
            .create_token(&self.issuer, delegate_name, scopes, hours)
    }

    /// Create a token with no scopes (should be rejected by all operations)
    pub fn create_empty_token(&self, delegate_name: &str) -> Result<String, anyhow::Error> {
        self.manager
            .create_token(&self.issuer, delegate_name, vec![], 24)
    }

    /// Verify a token
    pub fn verify_token(
        &self,
        token: &str,
    ) -> Result<communitas_mcp::auth::DelegateToken, anyhow::Error> {
        self.manager.verify_token(token)
    }

    /// Get a reference to the underlying manager
    pub fn manager(&self) -> &TokenManager {
        &self.manager
    }
}

/// Pre-defined token configurations for common test scenarios
pub struct TokenPresets;

impl TokenPresets {
    /// Scopes for a read-only agent
    pub fn read_only() -> Vec<Scope> {
        vec![Scope::ReadMessages, Scope::ReadFiles]
    }

    /// Scopes for a messaging agent
    pub fn messaging() -> Vec<Scope> {
        vec![Scope::ReadMessages, Scope::SendMessages]
    }

    /// Scopes for a kanban agent
    pub fn kanban() -> Vec<Scope> {
        vec![Scope::ManageKanban, Scope::ReadMessages]
    }

    /// Scopes for a full-featured agent
    pub fn full() -> Vec<Scope> {
        vec![Scope::Full]
    }

    /// Scopes for a file management agent
    pub fn files() -> Vec<Scope> {
        vec![Scope::ReadFiles, Scope::WriteFiles]
    }

    /// Scopes for an admin agent
    pub fn admin() -> Vec<Scope> {
        vec![Scope::Full]
    }

    /// All available scopes (for testing completeness)
    pub fn all_individual() -> Vec<Scope> {
        vec![
            Scope::ReadMessages,
            Scope::SendMessages,
            Scope::ReadFiles,
            Scope::WriteFiles,
            Scope::ManageEntities,
            Scope::ManageMembers,
            Scope::ManageKanban,
            Scope::ManageNetwork,
            Scope::ManageContacts,
        ]
    }
}

/// Scope assertions for validating token permissions
pub trait ScopeAssert {
    /// Assert that the token has a specific scope
    fn assert_has_scope(&self, scope: &Scope);
    /// Assert that the token does NOT have a specific scope
    fn assert_missing_scope(&self, scope: &Scope);
    /// Assert that the token is not expired
    fn assert_not_expired(&self);
}

impl ScopeAssert for communitas_mcp::auth::DelegateToken {
    fn assert_has_scope(&self, scope: &Scope) {
        assert!(
            self.has_scope(scope),
            "Expected token to have scope {:?}, but it doesn't. Scopes: {:?}",
            scope,
            self.scopes
        );
    }

    fn assert_missing_scope(&self, scope: &Scope) {
        // Full scope grants everything, so only check if Full is not present
        if !self.has_scope(&Scope::Full) {
            assert!(
                !self.has_scope(scope),
                "Expected token to NOT have scope {:?}, but it does. Scopes: {:?}",
                scope,
                self.scopes
            );
        }
    }

    fn assert_not_expired(&self) {
        assert!(
            !self.is_expired(),
            "Expected token to not be expired, but it is. Expires at: {}",
            self.expires_at
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_read_only_token() {
        let helper = TokenTestHelper::new().await.unwrap();
        let token = helper.create_read_only_token("test-agent").unwrap();

        let verified = helper.verify_token(&token).unwrap();
        verified.assert_has_scope(&Scope::ReadMessages);
        verified.assert_has_scope(&Scope::ReadFiles);
        verified.assert_not_expired();
    }

    #[tokio::test]
    async fn test_create_full_token() {
        let helper = TokenTestHelper::new().await.unwrap();
        let token = helper.create_full_token("test-agent").unwrap();

        let verified = helper.verify_token(&token).unwrap();
        verified.assert_has_scope(&Scope::Full);
        verified.assert_not_expired();
    }

    #[tokio::test]
    async fn test_expired_token() {
        let helper = TokenTestHelper::new().await.unwrap();
        let token = helper.create_expired_token("test-agent").unwrap();

        // Give it a moment to expire
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let result = helper.verify_token(&token);
        assert!(result.is_err(), "Expired token should fail verification");
    }

    #[tokio::test]
    async fn test_presets() {
        assert_eq!(TokenPresets::read_only().len(), 2);
        assert_eq!(TokenPresets::messaging().len(), 2);
        assert_eq!(TokenPresets::kanban().len(), 2);
        assert_eq!(TokenPresets::full().len(), 1);
    }
}
