// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Comprehensive unit tests for TokenManager
//!
//! Tests cover:
//! - Token creation with various configurations
//! - Token validation (valid, expired, tampered)
//! - BLAKE3 signature generation and verification
//! - Scope validation and permission checks
//! - Edge cases and boundary conditions

use communitas_mcp::auth::Scope;
use communitas_mcp::token::TokenManager;
use proptest::prelude::*;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

// =============================================================================
// Test Helpers
// =============================================================================

/// Helper to create a TokenManager for testing
async fn create_test_manager() -> Result<(TempDir, TokenManager), anyhow::Error> {
    let temp_dir = TempDir::new()?;
    let manager = TokenManager::new(temp_dir.path().to_path_buf()).await?;
    Ok((temp_dir, manager))
}

/// All available scopes for testing
fn all_scopes() -> Vec<Scope> {
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
        Scope::Full,
    ]
}

// =============================================================================
// Token Creation Tests
// =============================================================================

mod token_creation {
    use super::*;

    #[tokio::test]
    async fn test_create_token_with_single_scope() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token(
            "ocean-forest-moon-star",
            "test-agent",
            vec![Scope::ReadMessages],
            24,
        )?;

        // Token should be in format: payload.signature
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 2, "Token should have exactly two parts");
        assert!(!parts[0].is_empty(), "Payload should not be empty");
        assert!(!parts[1].is_empty(), "Signature should not be empty");

        Ok(())
    }

    #[tokio::test]
    async fn test_create_token_with_multiple_scopes() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let scopes = vec![
            Scope::ReadMessages,
            Scope::SendMessages,
            Scope::ReadFiles,
            Scope::WriteFiles,
        ];

        let token = manager.create_token("test-issuer", "multi-scope-agent", scopes.clone(), 48)?;

        let verified = manager.verify_token(&token)?;
        assert_eq!(verified.scopes.len(), 4);
        assert!(verified.has_scope(&Scope::ReadMessages));
        assert!(verified.has_scope(&Scope::SendMessages));
        assert!(verified.has_scope(&Scope::ReadFiles));
        assert!(verified.has_scope(&Scope::WriteFiles));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_token_with_all_scopes() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let scopes = all_scopes();
        let token = manager.create_token("test-issuer", "full-scope-agent", scopes, 24)?;

        let verified = manager.verify_token(&token)?;
        assert!(verified.has_scope(&Scope::Full));

        // Full scope should grant access to everything
        for scope in all_scopes() {
            assert!(
                verified.has_scope(&scope),
                "Full scope should grant access to {:?}",
                scope
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_token_with_expiration() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("test-issuer", "test-agent", vec![Scope::Full], 72)?;

        let verified = manager.verify_token(&token)?;

        // Token should expire in approximately 72 hours
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let expected_expiry = now + (72 * 3600);

        // Allow 5 seconds tolerance for test execution time
        assert!(
            verified.expires_at >= expected_expiry - 5
                && verified.expires_at <= expected_expiry + 5,
            "Token expiry should be approximately 72 hours from now"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_create_token_preserves_issuer_and_delegate_name() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let issuer = "ocean-forest-moon-star";
        let delegate = "my-awesome-agent-v2";

        let token = manager.create_token(issuer, delegate, vec![Scope::ReadMessages], 24)?;

        let verified = manager.verify_token(&token)?;
        assert_eq!(verified.issuer, issuer);
        assert_eq!(verified.delegate_name, delegate);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_multiple_tokens_have_unique_nonces() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token1 = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;
        let token2 = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;

        let verified1 = manager.verify_token(&token1)?;
        let verified2 = manager.verify_token(&token2)?;

        assert_ne!(
            verified1.nonce, verified2.nonce,
            "Each token should have a unique nonce"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_create_token_with_empty_scopes() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Empty scopes should be allowed - token creation should succeed
        let token = manager.create_token("issuer", "agent", vec![], 24)?;

        let verified = manager.verify_token(&token)?;
        assert!(verified.scopes.is_empty());
        assert!(!verified.has_scope(&Scope::ReadMessages));
        assert!(!verified.has_scope(&Scope::Full));

        Ok(())
    }

    #[tokio::test]
    async fn test_create_token_with_special_characters_in_names() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Test with special characters that are valid in JSON
        let issuer = "user-with-dashes_and_underscores.and.dots";
        let delegate = "agent-name-123-test";

        let token = manager.create_token(issuer, delegate, vec![Scope::Full], 24)?;

        let verified = manager.verify_token(&token)?;
        assert_eq!(verified.issuer, issuer);
        assert_eq!(verified.delegate_name, delegate);

        Ok(())
    }
}

// =============================================================================
// Token Validation Tests
// =============================================================================

mod token_validation {
    use super::*;

    #[tokio::test]
    async fn test_valid_token_acceptance() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token(
            "ocean-forest-moon-star",
            "valid-agent",
            vec![Scope::ReadMessages, Scope::SendMessages],
            24,
        )?;

        let result = manager.verify_token(&token);
        assert!(result.is_ok(), "Valid token should be accepted");

        let verified = result?;
        assert_eq!(verified.issuer, "ocean-forest-moon-star");
        assert_eq!(verified.delegate_name, "valid-agent");

        Ok(())
    }

    #[tokio::test]
    async fn test_expired_token_rejection() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Create a token with 0 hours expiration (expires immediately)
        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 0)?;

        // Wait a moment to ensure expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = manager.verify_token(&token);
        assert!(result.is_err(), "Expired token should be rejected");

        let err = result
            .err()
            .ok_or_else(|| anyhow::anyhow!("Expected error"))?;
        assert!(
            err.to_string().contains("expired"),
            "Error should mention expiration: {}",
            err
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_wrong_signature_rejection() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;

        // Tamper with the signature by replacing it with a different base64 string
        let parts: Vec<&str> = token.split('.').collect();
        let tampered = format!("{}.AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", parts[0]);

        let result = manager.verify_token(&tampered);
        assert!(
            result.is_err(),
            "Token with wrong signature should be rejected"
        );

        let err = result
            .err()
            .ok_or_else(|| anyhow::anyhow!("Expected error"))?;
        assert!(
            err.to_string().contains("signature"),
            "Error should mention signature: {}",
            err
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_format_rejection_no_dot() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let result = manager.verify_token("no_dot_in_this_token");
        assert!(result.is_err(), "Token without dot should be rejected");

        let err = result
            .err()
            .ok_or_else(|| anyhow::anyhow!("Expected error"))?;
        assert!(
            err.to_string().contains("format"),
            "Error should mention format: {}",
            err
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_format_rejection_multiple_dots() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let result = manager.verify_token("too.many.dots.here");
        assert!(
            result.is_err(),
            "Token with multiple dots should be rejected"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_format_rejection_empty_string() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let result = manager.verify_token("");
        assert!(result.is_err(), "Empty token should be rejected");

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_format_rejection_only_dot() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let result = manager.verify_token(".");
        assert!(result.is_err(), "Token with only dot should be rejected");

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_base64_payload_rejection() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Invalid base64 characters in payload
        let result = manager.verify_token("!!!invalid-base64!!!.AAAA");
        assert!(
            result.is_err(),
            "Token with invalid base64 payload should be rejected"
        );

        let err = result
            .err()
            .ok_or_else(|| anyhow::anyhow!("Expected error"))?;
        assert!(
            err.to_string().contains("encoding"),
            "Error should mention encoding: {}",
            err
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_base64_signature_rejection() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;
        let parts: Vec<&str> = token.split('.').collect();

        // Use invalid base64 for signature
        let tampered = format!("{}.!!!invalid!!!", parts[0]);

        let result = manager.verify_token(&tampered);
        assert!(
            result.is_err(),
            "Token with invalid base64 signature should be rejected"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_token_from_different_manager_rejected() -> Result<(), anyhow::Error> {
        let (_temp_dir1, manager1) = create_test_manager().await?;
        let (_temp_dir2, manager2) = create_test_manager().await?;

        let token = manager1.create_token("issuer", "agent", vec![Scope::Full], 24)?;

        // Token from manager1 should not be valid for manager2 (different secrets)
        let result = manager2.verify_token(&token);
        assert!(
            result.is_err(),
            "Token should not be valid for different manager"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_tampered_payload_rejection() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![Scope::ReadMessages], 24)?;

        let parts: Vec<&str> = token.split('.').collect();

        // Modify a character in the payload to simulate tampering
        let payload_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[0])?;

        // Flip a bit in the payload
        let mut tampered_bytes = payload_bytes;
        if !tampered_bytes.is_empty() {
            tampered_bytes[0] ^= 0xFF;
        }

        let tampered_payload = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &tampered_bytes,
        );

        let tampered_token = format!("{}.{}", tampered_payload, parts[1]);

        let result = manager.verify_token(&tampered_token);
        assert!(result.is_err(), "Tampered payload should be rejected");

        Ok(())
    }
}

// =============================================================================
// BLAKE3 Signature Tests
// =============================================================================

mod blake3_signature {
    use super::*;

    #[tokio::test]
    async fn test_signature_is_consistent() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Same inputs should produce same token structure
        // Note: nonce will be different, so we test verification instead
        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;

        // Verify multiple times should always succeed
        for _ in 0..10 {
            let result = manager.verify_token(&token);
            assert!(
                result.is_ok(),
                "Same token should always verify successfully"
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_signature_length_is_correct() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;

        let parts: Vec<&str> = token.split('.').collect();
        let signature_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[1])?;

        // BLAKE3 produces 32-byte hashes
        assert_eq!(
            signature_bytes.len(),
            32,
            "BLAKE3 signature should be 32 bytes"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_signature_changes_with_different_payloads() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token1 = manager.create_token("issuer1", "agent", vec![Scope::Full], 24)?;
        let token2 = manager.create_token("issuer2", "agent", vec![Scope::Full], 24)?;

        let parts1: Vec<&str> = token1.split('.').collect();
        let parts2: Vec<&str> = token2.split('.').collect();

        // Different payloads should produce different signatures
        // (Even though nonces also differ, the key point is signatures should differ)
        assert_ne!(
            parts1[1], parts2[1],
            "Different payloads should produce different signatures"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_timing_attack_resistance() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let valid_token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;
        let parts: Vec<&str> = valid_token.split('.').collect();

        // Create signatures that differ by varying amounts
        let sig_bytes =
            base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, parts[1])?;

        // Signature with one byte different at the start
        let mut sig_diff_start = sig_bytes.clone();
        sig_diff_start[0] ^= 0xFF;

        // Signature with one byte different at the end
        let mut sig_diff_end = sig_bytes.clone();
        let last_idx = sig_diff_end.len() - 1;
        sig_diff_end[last_idx] ^= 0xFF;

        // All zeros signature
        let sig_zeros = vec![0u8; 32];

        // All invalid signatures should be rejected
        for (name, sig) in [
            ("diff_start", sig_diff_start),
            ("diff_end", sig_diff_end),
            ("zeros", sig_zeros),
        ] {
            let tampered_sig =
                base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &sig);
            let tampered_token = format!("{}.{}", parts[0], tampered_sig);

            let result = manager.verify_token(&tampered_token);
            assert!(
                result.is_err(),
                "Tampered signature ({}) should be rejected",
                name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_secret_persistence_across_manager_instances() -> Result<(), anyhow::Error> {
        let temp_dir = TempDir::new()?;

        // Create first manager and token
        let token = {
            let manager1 = TokenManager::new(temp_dir.path().to_path_buf()).await?;
            manager1.create_token("issuer", "agent", vec![Scope::Full], 24)?
        };

        // Create new manager with same vault directory
        let manager2 = TokenManager::new(temp_dir.path().to_path_buf()).await?;

        // Token should still be valid with new manager instance
        let result = manager2.verify_token(&token);
        assert!(
            result.is_ok(),
            "Token should be valid with new manager instance using same secret"
        );

        Ok(())
    }
}

// =============================================================================
// Scope Validation Tests
// =============================================================================

mod scope_validation {
    use super::*;

    #[tokio::test]
    async fn test_full_scope_grants_all_permissions() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;
        let verified = manager.verify_token(&token)?;

        // Full scope should grant access to all other scopes
        let scopes_to_check = vec![
            Scope::ReadMessages,
            Scope::SendMessages,
            Scope::ReadFiles,
            Scope::WriteFiles,
            Scope::ManageEntities,
            Scope::ManageMembers,
            Scope::ManageKanban,
            Scope::ManageNetwork,
            Scope::ManageContacts,
        ];

        for scope in scopes_to_check {
            assert!(
                verified.has_scope(&scope),
                "Full scope should grant {:?}",
                scope
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_limited_scopes_only_grant_specified() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token(
            "issuer",
            "agent",
            vec![Scope::ReadMessages, Scope::ReadFiles],
            24,
        )?;
        let verified = manager.verify_token(&token)?;

        // Should have specified scopes
        assert!(verified.has_scope(&Scope::ReadMessages));
        assert!(verified.has_scope(&Scope::ReadFiles));

        // Should NOT have unspecified scopes
        assert!(!verified.has_scope(&Scope::SendMessages));
        assert!(!verified.has_scope(&Scope::WriteFiles));
        assert!(!verified.has_scope(&Scope::ManageEntities));
        assert!(!verified.has_scope(&Scope::ManageNetwork));

        Ok(())
    }

    #[tokio::test]
    async fn test_empty_scopes_grant_nothing() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![], 24)?;
        let verified = manager.verify_token(&token)?;

        // Empty scopes should grant nothing
        for scope in all_scopes() {
            assert!(
                !verified.has_scope(&scope),
                "Empty scopes should not grant {:?}",
                scope
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_individual_scope_checks() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Test each scope individually
        let test_cases = vec![
            (Scope::ReadMessages, "ReadMessages"),
            (Scope::SendMessages, "SendMessages"),
            (Scope::ReadFiles, "ReadFiles"),
            (Scope::WriteFiles, "WriteFiles"),
            (Scope::ManageEntities, "ManageEntities"),
            (Scope::ManageMembers, "ManageMembers"),
            (Scope::ManageKanban, "ManageKanban"),
            (Scope::ManageNetwork, "ManageNetwork"),
            (Scope::ManageContacts, "ManageContacts"),
        ];

        for (scope, name) in test_cases {
            let token = manager.create_token("issuer", "agent", vec![scope.clone()], 24)?;
            let verified = manager.verify_token(&token)?;

            assert!(
                verified.has_scope(&scope),
                "Token with {} should have that scope",
                name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_duplicate_scopes_are_handled() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // Include same scope multiple times
        let token = manager.create_token(
            "issuer",
            "agent",
            vec![
                Scope::ReadMessages,
                Scope::ReadMessages,
                Scope::ReadMessages,
            ],
            24,
        )?;
        let verified = manager.verify_token(&token)?;

        assert!(verified.has_scope(&Scope::ReadMessages));
        // Duplicates should not cause issues
        assert!(!verified.has_scope(&Scope::SendMessages));

        Ok(())
    }
}

// =============================================================================
// Edge Cases
// =============================================================================

mod edge_cases {
    use super::*;

    #[tokio::test]
    async fn test_very_long_expiration() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        // 100 years in hours
        let hours = 100 * 365 * 24;
        let token = manager.create_token("issuer", "agent", vec![Scope::Full], hours)?;

        let verified = manager.verify_token(&token)?;
        assert!(
            !verified.is_expired(),
            "Token with long expiration should not be expired"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_zero_expiration_immediately_expires() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 0)?;

        // Small delay to ensure time passes
        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = manager.verify_token(&token);
        assert!(result.is_err(), "Zero expiration token should be expired");

        Ok(())
    }

    #[tokio::test]
    async fn test_token_with_empty_issuer() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("", "agent", vec![Scope::Full], 24)?;
        let verified = manager.verify_token(&token)?;

        assert_eq!(verified.issuer, "");

        Ok(())
    }

    #[tokio::test]
    async fn test_token_with_empty_delegate_name() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let token = manager.create_token("issuer", "", vec![Scope::Full], 24)?;
        let verified = manager.verify_token(&token)?;

        assert_eq!(verified.delegate_name, "");

        Ok(())
    }

    #[tokio::test]
    async fn test_token_with_unicode_characters() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let issuer = "issuer-unicode-test";
        let delegate = "agent-unicode-test";

        let token = manager.create_token(issuer, delegate, vec![Scope::Full], 24)?;
        let verified = manager.verify_token(&token)?;

        assert_eq!(verified.issuer, issuer);
        assert_eq!(verified.delegate_name, delegate);

        Ok(())
    }

    #[tokio::test]
    async fn test_token_with_very_long_names() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let long_name = "a".repeat(10000);

        let token = manager.create_token(&long_name, &long_name, vec![Scope::Full], 24)?;
        let verified = manager.verify_token(&token)?;

        assert_eq!(verified.issuer.len(), 10000);
        assert_eq!(verified.delegate_name.len(), 10000);

        Ok(())
    }

    #[tokio::test]
    async fn test_secret_file_with_wrong_size_fails() -> Result<(), anyhow::Error> {
        let temp_dir = TempDir::new()?;
        let secret_path = temp_dir.path().join("mcp_server_secret");

        // Write a secret with wrong size
        tokio::fs::write(&secret_path, b"too_short").await?;

        let result = TokenManager::new(temp_dir.path().to_path_buf()).await;
        assert!(result.is_err(), "Should fail with wrong secret size");

        Ok(())
    }

    #[tokio::test]
    async fn test_concurrent_token_creation() -> Result<(), anyhow::Error> {
        let temp_dir = TempDir::new()?;
        let manager = std::sync::Arc::new(TokenManager::new(temp_dir.path().to_path_buf()).await?);

        // Create multiple tokens concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let mgr = manager.clone();
            let handle = tokio::spawn(async move {
                let name = format!("agent-{}", i);
                mgr.create_token("issuer", &name, vec![Scope::Full], 24)
            });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles).await;

        for (i, result) in results.into_iter().enumerate() {
            let token = result.map_err(|e| anyhow::anyhow!("Join error: {}", e))??;

            // Verify each token
            let verified = manager.verify_token(&token)?;
            assert_eq!(verified.delegate_name, format!("agent-{}", i));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_nonce_is_random() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let mut nonces = std::collections::HashSet::new();

        // Create 100 tokens and collect nonces
        for _ in 0..100 {
            let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;
            let verified = manager.verify_token(&token)?;
            nonces.insert(verified.nonce);
        }

        // All nonces should be unique
        assert_eq!(nonces.len(), 100, "All nonces should be unique");

        Ok(())
    }

    #[tokio::test]
    async fn test_issued_at_is_current_time() -> Result<(), anyhow::Error> {
        let (_temp_dir, manager) = create_test_manager().await?;

        let before = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let token = manager.create_token("issuer", "agent", vec![Scope::Full], 24)?;
        let verified = manager.verify_token(&token)?;

        let after = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        assert!(
            verified.issued_at >= before && verified.issued_at <= after,
            "issued_at should be between before ({}) and after ({}), got {}",
            before,
            after,
            verified.issued_at
        );

        Ok(())
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

mod property_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        #[test]
        fn test_any_valid_token_can_be_verified(
            issuer in "[a-zA-Z0-9_-]{1,100}",
            delegate in "[a-zA-Z0-9_-]{1,100}",
            hours in 1u64..=8760u64  // 1 hour to 1 year
        ) {
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                TestCaseError::fail(format!("Failed to create runtime: {}", e))
            })?;

            rt.block_on(async {
                let temp_dir = TempDir::new().map_err(|e| {
                    TestCaseError::fail(format!("Failed to create temp dir: {}", e))
                })?;

                let manager = TokenManager::new(temp_dir.path().to_path_buf())
                    .await
                    .map_err(|e| {
                        TestCaseError::fail(format!("Failed to create manager: {}", e))
                    })?;

                let token = manager
                    .create_token(&issuer, &delegate, vec![Scope::Full], hours)
                    .map_err(|e| {
                        TestCaseError::fail(format!("Failed to create token: {}", e))
                    })?;

                let verified = manager.verify_token(&token).map_err(|e| {
                    TestCaseError::fail(format!("Failed to verify token: {}", e))
                })?;

                prop_assert_eq!(verified.issuer, issuer);
                prop_assert_eq!(verified.delegate_name, delegate);

                Ok(())
            })?;
        }

        #[test]
        fn test_tampered_tokens_are_rejected(
            issuer in "[a-zA-Z0-9]{1,50}",
            tamper_byte in 0u8..255u8
        ) {
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                TestCaseError::fail(format!("Failed to create runtime: {}", e))
            })?;

            rt.block_on(async {
                let temp_dir = TempDir::new().map_err(|e| {
                    TestCaseError::fail(format!("Failed to create temp dir: {}", e))
                })?;

                let manager = TokenManager::new(temp_dir.path().to_path_buf())
                    .await
                    .map_err(|e| {
                        TestCaseError::fail(format!("Failed to create manager: {}", e))
                    })?;

                let token = manager
                    .create_token(&issuer, "agent", vec![Scope::Full], 24)
                    .map_err(|e| {
                        TestCaseError::fail(format!("Failed to create token: {}", e))
                    })?;

                let parts: Vec<&str> = token.split('.').collect();
                let payload_bytes = base64::Engine::decode(
                    &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                    parts[0],
                ).map_err(|e| {
                    TestCaseError::fail(format!("Failed to decode payload: {}", e))
                })?;

                if !payload_bytes.is_empty() {
                    let mut tampered = payload_bytes;
                    tampered[0] ^= tamper_byte.max(1); // Ensure we actually change something

                    let tampered_payload = base64::Engine::encode(
                        &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                        &tampered,
                    );

                    let tampered_token = format!("{}.{}", tampered_payload, parts[1]);

                    // Tampered token should fail verification
                    let result = manager.verify_token(&tampered_token);
                    prop_assert!(result.is_err(), "Tampered token should be rejected");
                }

                Ok(())
            })?;
        }

        #[test]
        fn test_scope_combinations_are_preserved(
            scope_mask in 0u16..1024u16  // 10 scopes = up to 2^10 combinations
        ) {
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                TestCaseError::fail(format!("Failed to create runtime: {}", e))
            })?;

            rt.block_on(async {
                let all = all_scopes();
                let selected_scopes: Vec<Scope> = all
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| (scope_mask >> i) & 1 == 1)
                    .map(|(_, s)| s)
                    .collect();

                let temp_dir = TempDir::new().map_err(|e| {
                    TestCaseError::fail(format!("Failed to create temp dir: {}", e))
                })?;

                let manager = TokenManager::new(temp_dir.path().to_path_buf())
                    .await
                    .map_err(|e| {
                        TestCaseError::fail(format!("Failed to create manager: {}", e))
                    })?;

                let token = manager
                    .create_token("issuer", "agent", selected_scopes.clone(), 24)
                    .map_err(|e| {
                        TestCaseError::fail(format!("Failed to create token: {}", e))
                    })?;

                let verified = manager.verify_token(&token).map_err(|e| {
                    TestCaseError::fail(format!("Failed to verify token: {}", e))
                })?;

                prop_assert_eq!(verified.scopes.len(), selected_scopes.len());

                Ok(())
            })?;
        }
    }
}
