// SPDX-License-Identifier: MIT OR Apache-2.0

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

//! Integration tests for identity recovery flows.
//!
//! These tests verify the full recovery lifecycle including:
//! - Create identity and recover with same mnemonic
//! - Deterministic key derivation (same mnemonic = same identity)
//! - Passphrase support (different passphrase = different identity)
//! - Invalid mnemonic handling
//! - Recovery when vault already exists
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use communitas_core::app::CommunitasApp;
use communitas_core::recovery::{RecoveryConfig, generate_recovery_mnemonic, mnemonic_to_words};
use communitas_core::ui_core::{preview_identity_from_mnemonic, validate_recovery_mnemonic};
use communitas_ui_service::UiServices;
use communitas_ui_service::auth::AuthService;
use communitas_ui_service::storage::UiStorage;
use std::sync::Arc;
use tempfile::TempDir;

/// Stack size for test threads (8MB) to handle large async state machines.
const TEST_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Run a test with a larger stack size to avoid overflow.
fn run_with_large_stack<F>(test_fn: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::Builder::new()
        .stack_size(TEST_STACK_SIZE)
        .spawn(test_fn)
        .expect("Failed to spawn test thread")
        .join()
        .expect("Test thread panicked");
}

/// Helper to create UiServices with a dummy app for testing.
async fn make_services(temp: &TempDir) -> UiServices {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let app = Arc::new(
        CommunitasApp::new(
            "test-word-word-word".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            temp.path()
                .join("app_storage")
                .to_string_lossy()
                .to_string(),
        )
        .await
        .unwrap(),
    );
    UiServices::new(storage, app).unwrap()
}

// =============================================================================
// Test 1: Create -> Recover cycle with same mnemonic
// =============================================================================

/// Test that recovering with the same mnemonic produces the same four_words identity.
#[test]
fn test_create_recover_cycle_same_identity() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_create_recover_cycle_same_identity_inner());
    });
}

async fn test_create_recover_cycle_same_identity_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap();
    let mnemonic_str = mnemonic.to_string();

    // Preview the identity first (deterministic derivation)
    let preview = preview_identity_from_mnemonic(mnemonic_str.clone(), None).unwrap();
    let expected_four_words = preview.four_words.clone();

    // Now do recovery via AuthService
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let session = auth
        .recover_identity(&mnemonic_str, None, "TestUser", None, false)
        .await
        .expect("Recovery should succeed");

    // Verify the recovered identity matches the preview
    assert_eq!(
        session.four_words, expected_four_words,
        "Recovered four_words should match preview"
    );
}

// =============================================================================
// Test 2: Recovery produces same public key (determinism via preview)
// =============================================================================

/// Test that the same mnemonic always produces the same public key.
#[test]
fn test_recovery_deterministic_pubkey() {
    // Use a fixed test mnemonic (12 words for simplicity)
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Preview identity twice - should produce identical results
    let preview1 = preview_identity_from_mnemonic(test_mnemonic.to_string(), None).unwrap();
    let preview2 = preview_identity_from_mnemonic(test_mnemonic.to_string(), None).unwrap();

    assert_eq!(
        preview1.four_words, preview2.four_words,
        "Same mnemonic should produce same four_words"
    );
    assert_eq!(
        preview1.pubkey_hex, preview2.pubkey_hex,
        "Same mnemonic should produce same pubkey"
    );

    // Verify pubkey is non-empty (BLAKE3-derived seed = 32 bytes = 64 hex chars)
    assert!(
        !preview1.pubkey_hex.is_empty(),
        "Pubkey should not be empty"
    );
    assert!(
        preview1.pubkey_hex.len() >= 64,
        "Pubkey should be at least 64 hex chars (32-byte BLAKE3 seed derivation), got {}",
        preview1.pubkey_hex.len()
    );
}

/// Test that AuthService recovery produces same four_words as preview.
#[test]
fn test_recovery_deterministic_four_words() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recovery_deterministic_four_words_inner());
    });
}

async fn test_recovery_deterministic_four_words_inner() {
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Get expected four_words from preview
    let preview = preview_identity_from_mnemonic(test_mnemonic.to_string(), None).unwrap();

    // Recover via AuthService and verify
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let session = auth
        .recover_identity(test_mnemonic, None, "TestUser", None, false)
        .await
        .expect("Recovery should succeed");

    assert_eq!(
        session.four_words, preview.four_words,
        "AuthService recovery should produce same four_words as preview"
    );
}

// =============================================================================
// Test 3: Recovery with optional passphrase
// =============================================================================

/// Test that recovery with passphrase produces different identity than without.
#[test]
fn test_recovery_with_passphrase() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recovery_with_passphrase_inner());
    });
}

async fn test_recovery_with_passphrase_inner() {
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Preview without passphrase
    let preview_no_pass = preview_identity_from_mnemonic(test_mnemonic.to_string(), None).unwrap();

    // Preview with passphrase
    let preview_with_pass =
        preview_identity_from_mnemonic(test_mnemonic.to_string(), Some("secret".to_string()))
            .unwrap();

    // Different passphrase should produce different identity
    assert_ne!(
        preview_no_pass.four_words, preview_with_pass.four_words,
        "Passphrase should produce different four_words"
    );
    assert_ne!(
        preview_no_pass.pubkey_hex, preview_with_pass.pubkey_hex,
        "Passphrase should produce different pubkey"
    );

    // Same mnemonic + same passphrase should produce same identity
    let preview_with_pass2 =
        preview_identity_from_mnemonic(test_mnemonic.to_string(), Some("secret".to_string()))
            .unwrap();

    assert_eq!(
        preview_with_pass.four_words, preview_with_pass2.four_words,
        "Same passphrase should produce same four_words"
    );
    assert_eq!(
        preview_with_pass.pubkey_hex, preview_with_pass2.pubkey_hex,
        "Same passphrase should produce same pubkey"
    );

    // Verify via AuthService
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let session = auth
        .recover_identity(test_mnemonic, Some("secret"), "TestUser", None, false)
        .await
        .expect("Recovery with passphrase should succeed");

    assert_eq!(
        session.four_words, preview_with_pass.four_words,
        "AuthService recovery should match preview with same passphrase"
    );
}

// =============================================================================
// Test 4: Invalid mnemonic handling
// =============================================================================

/// Test that invalid mnemonic words return error.
#[test]
fn test_invalid_mnemonic_word() {
    // Word not in BIP39 wordlist
    let invalid_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";

    let result = validate_recovery_mnemonic(invalid_mnemonic.to_string());
    assert!(
        result.is_ok(),
        "validate_recovery_mnemonic should return Ok"
    );
    assert!(
        !result.unwrap(),
        "Invalid word should return false (invalid mnemonic)"
    );
}

/// Test that wrong word count returns error.
#[test]
fn test_invalid_mnemonic_word_count() {
    // Only 5 words (must be 12, 15, 18, 21, or 24)
    let invalid_mnemonic = "abandon abandon abandon abandon about";

    let result = validate_recovery_mnemonic(invalid_mnemonic.to_string());
    assert!(
        result.is_ok(),
        "validate_recovery_mnemonic should return Ok"
    );
    assert!(
        !result.unwrap(),
        "Wrong word count should return false (invalid mnemonic)"
    );
}

/// Test that invalid checksum returns error.
#[test]
fn test_invalid_mnemonic_checksum() {
    // Valid words but wrong checksum (12 repeating words)
    let invalid_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";

    let result = validate_recovery_mnemonic(invalid_mnemonic.to_string());
    assert!(
        result.is_ok(),
        "validate_recovery_mnemonic should return Ok"
    );
    assert!(
        !result.unwrap(),
        "Invalid checksum should return false (invalid mnemonic)"
    );
}

/// Test that preview_identity_from_mnemonic fails for invalid mnemonic.
#[test]
fn test_preview_invalid_mnemonic_fails() {
    let invalid_mnemonic = "invalid words that are not in bip39 dictionary at all here test";

    let result = preview_identity_from_mnemonic(invalid_mnemonic.to_string(), None);
    assert!(
        result.is_err(),
        "preview_identity_from_mnemonic should fail for invalid mnemonic"
    );
}

/// Test that AuthService recover_identity fails for invalid mnemonic.
#[test]
fn test_auth_recover_invalid_mnemonic_fails() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_auth_recover_invalid_mnemonic_fails_inner());
    });
}

async fn test_auth_recover_invalid_mnemonic_fails_inner() {
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let invalid_mnemonic = "invalid words that are not in bip39 dictionary at all here test";

    let result = auth
        .recover_identity(invalid_mnemonic, None, "TestUser", None, false)
        .await;

    assert!(
        result.is_err(),
        "AuthService recover_identity should fail for invalid mnemonic"
    );
}

// =============================================================================
// Test 5: Recovery when vault already exists (should login, not duplicate)
// =============================================================================

/// Test that recovering when vault exists uses existing vault.
#[test]
fn test_recovery_existing_vault_login() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recovery_existing_vault_login_inner());
    });
}

async fn test_recovery_existing_vault_login_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap();
    let mnemonic_str = mnemonic.to_string();

    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let _password = "test-password-123";

    // First recovery - creates new vault
    let session1 = auth
        .recover_identity(&mnemonic_str, None, "OriginalUser", None, false)
        .await
        .expect("First recovery should succeed");

    // Logout
    auth.logout().await.expect("Logout should succeed");

    // Second recovery with same mnemonic - should use existing vault
    let session2 = auth
        .recover_identity(&mnemonic_str, None, "RecoveredUser", None, false)
        .await
        .expect("Second recovery should succeed (login to existing vault)");

    // Should produce the same identity
    assert_eq!(
        session1.four_words, session2.four_words,
        "Both recoveries should produce same four_words"
    );
}

// =============================================================================
// Additional tests: Mnemonic generation and validation
// =============================================================================

/// Test that generated mnemonic produces valid word list.
#[test]
fn test_mnemonic_generation_produces_valid_words() {
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap();
    let words = mnemonic_to_words(&mnemonic);

    // Default config produces 24 words
    assert_eq!(words.len(), 24, "Default config should produce 24 words");

    // All words should be non-empty lowercase
    for word in &words {
        assert!(!word.is_empty(), "Word should not be empty");
        assert!(
            word.chars().all(|c| c.is_ascii_lowercase()),
            "Word should be lowercase ASCII: {}",
            word
        );
    }

    // The mnemonic should be valid
    let mnemonic_str = words.join(" ");
    let is_valid = validate_recovery_mnemonic(mnemonic_str).unwrap();
    assert!(is_valid, "Generated mnemonic should be valid");
}

/// Test 12-word mnemonic configuration.
#[test]
fn test_mnemonic_12_word_config() {
    let config = RecoveryConfig::default().with_word_count(12);
    let mnemonic = generate_recovery_mnemonic(&config).unwrap();
    let words = mnemonic_to_words(&mnemonic);

    assert_eq!(words.len(), 12, "12-word config should produce 12 words");

    let mnemonic_str = words.join(" ");
    let is_valid = validate_recovery_mnemonic(mnemonic_str).unwrap();
    assert!(is_valid, "Generated 12-word mnemonic should be valid");
}

/// Test full create-recover roundtrip with newly generated mnemonic.
#[test]
fn test_full_create_recover_roundtrip() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_full_create_recover_roundtrip_inner());
    });
}

async fn test_full_create_recover_roundtrip_inner() {
    // 1. Generate new identity
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap();
    let mnemonic_str = mnemonic.to_string();
    let mnemonic_words = mnemonic_to_words(&mnemonic);

    // 2. Verify mnemonic is valid
    assert_eq!(mnemonic_words.len(), 24);
    let is_valid = validate_recovery_mnemonic(mnemonic_str.clone()).unwrap();
    assert!(is_valid, "Generated mnemonic should be valid");

    // 3. Preview identity
    let preview = preview_identity_from_mnemonic(mnemonic_str.clone(), None).unwrap();
    assert!(
        !preview.four_words.is_empty(),
        "Preview should produce non-empty four_words"
    );
    assert!(
        !preview.pubkey_hex.is_empty(),
        "Preview should produce non-empty pubkey"
    );

    // 4. Recover via AuthService
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let session = auth
        .recover_identity(&mnemonic_str, None, "TestUser", None, false)
        .await
        .expect("Recovery should succeed");

    // 5. Verify session matches preview
    assert_eq!(
        session.four_words, preview.four_words,
        "Session four_words should match preview"
    );

    // 6. Logout and recover again
    auth.logout().await.expect("Logout should succeed");

    let session2 = auth
        .recover_identity(&mnemonic_str, None, "RecoveredUser", None, false)
        .await
        .expect("Re-recovery should succeed");

    assert_eq!(
        session.four_words, session2.four_words,
        "Re-recovery should produce same identity"
    );
}
