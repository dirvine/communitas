//! End-to-end tests for authentication flows.
//!
//! These tests verify complete authentication workflows through the AuthService layer,
//! ensuring that login, logout, session management, identity switching, and recovery
//! flows work correctly.
//!
//! Note: These tests use larger stack sizes (8MB) to avoid stack overflow
//! from the large async state machines in CommunitasApp.

use communitas_core::app::CommunitasApp;
use communitas_core::recovery::{RecoveryConfig, generate_recovery_mnemonic};
use communitas_core::ui_core::preview_identity_from_mnemonic;
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

/// Helper to create a test identity via recovery and return the services.
async fn create_identity_via_recovery(
    temp: &TempDir,
    mnemonic: &str,
    display_name: &str,
) -> (UiServices, String) {
    let services = make_services(temp).await;
    let auth = services.auth();

    let session = auth
        .recover_identity(mnemonic, None, display_name, None, false)
        .await
        .expect("Recovery should succeed");

    (services, session.four_words)
}

// =============================================================================
// Test 1: Login with valid credentials (via recovery flow)
// =============================================================================

#[test]
fn test_login_with_valid_credentials() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_login_with_valid_credentials_inner());
    });
}

async fn test_login_with_valid_credentials_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create identity via recovery
    let temp = TempDir::new().unwrap();
    let display_name = "TestUser";
    let (services, four_words) = create_identity_via_recovery(&temp, &mnemonic, display_name).await;
    let auth = services.auth();

    // Logout first
    auth.logout().await.expect("Logout should succeed");

    // Test: Login with valid four_words
    let session = auth.login(&four_words).await.expect("Login should succeed");

    // Verify session is active
    assert_eq!(session.four_words, four_words);
    assert_eq!(session.display_name, display_name);

    // Verify current state shows authenticated
    let state = auth.subscribe().borrow().clone();
    match state {
        communitas_ui_service::auth::AuthStateSnapshot::Authenticated { session: s, .. } => {
            assert_eq!(s.four_words, four_words);
        }
        _ => panic!("Expected authenticated state after login"),
    }
}

#[test]
fn test_login_with_invalid_four_words() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_login_with_invalid_four_words_inner());
    });
}

async fn test_login_with_invalid_four_words_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create identity
    let temp = TempDir::new().unwrap();
    let (services, _) = create_identity_via_recovery(&temp, &mnemonic, "TestUser").await;
    let auth = services.auth();

    // Logout first
    auth.logout().await.expect("Logout should succeed");

    // Test: Login with non-existent four_words
    let result = auth.login("nonexistent-word-word-word").await;

    // Verify: Should fail
    assert!(result.is_err(), "Login with invalid four_words should fail");
}

// =============================================================================
// Test 3: Logout and session cleanup
// =============================================================================

#[test]
fn test_logout_and_session_cleanup() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_logout_and_session_cleanup_inner());
    });
}

async fn test_logout_and_session_cleanup_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create identity
    let temp = TempDir::new().unwrap();
    let (services, _) = create_identity_via_recovery(&temp, &mnemonic, "TestUser").await;
    let auth = services.auth();

    // Verify session exists (created by recovery)
    let state_before = auth.subscribe().borrow().clone();
    assert!(
        matches!(
            state_before,
            communitas_ui_service::auth::AuthStateSnapshot::Authenticated { .. }
        ),
        "Should be authenticated after recovery"
    );

    // Test: Logout
    auth.logout().await.expect("Logout should succeed");

    // Verify: Session is cleared
    let state_after = auth.subscribe().borrow().clone();
    assert!(
        matches!(
            state_after,
            communitas_ui_service::auth::AuthStateSnapshot::LoggedOut
        ),
        "Should be logged out after logout"
    );
}

// =============================================================================
// Test 4: Identity switch between multiple identities
// =============================================================================

#[test]
fn test_identity_switch_between_multiple_identities() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_identity_switch_inner());
    });
}

async fn test_identity_switch_inner() {
    // Generate two different mnemonics
    let config = RecoveryConfig::default();
    let mnemonic1 = generate_recovery_mnemonic(&config).unwrap().to_string();
    let mnemonic2 = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Get expected four_words from preview
    let preview1 = preview_identity_from_mnemonic(mnemonic1.clone(), None).unwrap();
    let preview2 = preview_identity_from_mnemonic(mnemonic2.clone(), None).unwrap();

    // Setup shared temp directory
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    // Create first identity
    let session1 = auth
        .recover_identity(&mnemonic1, None, "User1", None, false)
        .await
        .expect("Recovery 1 should succeed");
    assert_eq!(session1.four_words, preview1.four_words);

    // Create second identity
    let session2 = auth
        .recover_identity(&mnemonic2, None, "User2", None, false)
        .await
        .expect("Recovery 2 should succeed");
    assert_eq!(session2.four_words, preview2.four_words);

    // After second recovery, we're logged in as user2
    let state = auth.subscribe().borrow().clone();
    match state {
        communitas_ui_service::auth::AuthStateSnapshot::Authenticated { session: s, .. } => {
            assert_eq!(s.four_words, preview2.four_words);
        }
        _ => panic!("Should be authenticated as user2"),
    }

    // Test: Switch to first user
    let session1_again = auth
        .login(&preview1.four_words)
        .await
        .expect("Login as user1 should succeed");
    assert_eq!(session1_again.four_words, preview1.four_words);

    // Verify session switched to user1
    let state1 = auth.subscribe().borrow().clone();
    match state1 {
        communitas_ui_service::auth::AuthStateSnapshot::Authenticated { session: s, .. } => {
            assert_eq!(s.four_words, preview1.four_words);
            assert_eq!(s.display_name, "User1");
        }
        _ => panic!("Should be authenticated as user1"),
    }

    // Test: Switch back to second user
    let session2_again = auth
        .login(&preview2.four_words)
        .await
        .expect("Login as user2 should succeed");
    assert_eq!(session2_again.four_words, preview2.four_words);

    // Verify session switched to user2
    let state2 = auth.subscribe().borrow().clone();
    match state2 {
        communitas_ui_service::auth::AuthStateSnapshot::Authenticated { session: s, .. } => {
            assert_eq!(s.four_words, preview2.four_words);
            assert_eq!(s.display_name, "User2");
        }
        _ => panic!("Should be authenticated as user2"),
    }
}

// =============================================================================
// Test 5: Session persistence with remember_me (recent identities)
// =============================================================================

#[test]
fn test_session_persistence_recent_identities() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_session_persistence_inner());
    });
}

async fn test_session_persistence_inner() {
    // Generate two mnemonics to test recent identities
    let config = RecoveryConfig::default();
    let mnemonic1 = generate_recovery_mnemonic(&config).unwrap().to_string();
    let mnemonic2 = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Preview to get expected four_words
    let preview1 = preview_identity_from_mnemonic(mnemonic1.clone(), None).unwrap();
    let preview2 = preview_identity_from_mnemonic(mnemonic2.clone(), None).unwrap();

    // Create first identity
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    let session1 = auth
        .recover_identity(&mnemonic1, None, "User1", None, false)
        .await
        .expect("Recovery should succeed");
    assert_eq!(session1.four_words, preview1.four_words);

    // Create second identity (this should add first to recent list)
    let session2 = auth
        .recover_identity(&mnemonic2, None, "User2", None, false)
        .await
        .expect("Recovery should succeed");
    assert_eq!(session2.four_words, preview2.four_words);

    // While logged in as user2, check recent identities
    let recent = auth
        .list_recent_identities()
        .await
        .expect("Should get recent identities");

    // Verify user1 is in recent list (we switched away from it)
    let found = recent.iter().any(|r| r.four_words == preview1.four_words);
    assert!(found, "User1 identity should be in recent identities list");

    // Login as user1 using the four_words (switching back)
    let session1_again = auth
        .login(&preview1.four_words)
        .await
        .expect("Login should succeed");
    assert_eq!(session1_again.four_words, preview1.four_words);

    // Now user2 should be in recent list
    let recent2 = auth
        .list_recent_identities()
        .await
        .expect("Should get recent identities");
    let found2 = recent2.iter().any(|r| r.four_words == preview2.four_words);
    assert!(found2, "User2 identity should be in recent identities list");
}

// =============================================================================
// Test 6: Recovery flow via BIP39 mnemonic
// =============================================================================

#[test]
fn test_recovery_flow_via_bip39_mnemonic() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_recovery_flow_inner());
    });
}

async fn test_recovery_flow_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Preview to get expected identity
    let preview = preview_identity_from_mnemonic(mnemonic.clone(), None).unwrap();

    // First: Create identity
    let temp1 = TempDir::new().unwrap();
    let (services1, four_words) = create_identity_via_recovery(&temp1, &mnemonic, "TestUser").await;
    assert_eq!(four_words, preview.four_words);

    // Verify can login with original four_words
    let auth1 = services1.auth();
    auth1.logout().await.expect("Logout should succeed");
    auth1
        .login(&four_words)
        .await
        .expect("Login with original four_words should succeed");

    // Second: Recover to a fresh storage
    let temp2 = TempDir::new().unwrap();
    let (services2, recovered_four_words) =
        create_identity_via_recovery(&temp2, &mnemonic, "RecoveredUser").await;

    // Verify: Recovery produces same four_words (identity)
    assert_eq!(recovered_four_words, preview.four_words);

    // Verify: Can login with recovered four_words
    let auth2 = services2.auth();
    auth2.logout().await.expect("Logout should succeed");
    let session = auth2
        .login(&recovered_four_words)
        .await
        .expect("Login with recovered four_words should succeed");
    assert_eq!(session.four_words, preview.four_words);
}

// =============================================================================
// Test 7: Concurrent login detection (multiple service instances)
// =============================================================================

#[test]
fn test_concurrent_service_instances() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_concurrent_service_instances_inner());
    });
}

async fn test_concurrent_service_instances_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create identity
    let temp = TempDir::new().unwrap();
    let (services1, four_words) = create_identity_via_recovery(&temp, &mnemonic, "TestUser").await;
    let auth1 = services1.auth();

    // Verify first service is authenticated
    let state1_before = auth1.subscribe().borrow().clone();
    assert!(
        matches!(
            state1_before,
            communitas_ui_service::auth::AuthStateSnapshot::Authenticated { .. }
        ),
        "First service should be authenticated"
    );

    // Create a second service instance pointing to same storage
    let services2 = make_services(&temp).await;
    let auth2 = services2.auth();

    // Login with second service
    let session2 = auth2
        .login(&four_words)
        .await
        .expect("Login from second service should succeed");
    assert_eq!(session2.four_words, four_words);

    // Both services should have valid session state
    // (they share the same underlying storage)
    let state2 = auth2.subscribe().borrow().clone();
    assert!(
        matches!(
            state2,
            communitas_ui_service::auth::AuthStateSnapshot::Authenticated { .. }
        ),
        "Second service should be authenticated"
    );
}

// =============================================================================
// Test 8: Session expiration check
// =============================================================================

#[test]
fn test_session_expiration_check() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_session_expiration_check_inner());
    });
}

async fn test_session_expiration_check_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create identity
    let temp = TempDir::new().unwrap();
    let (services, _) = create_identity_via_recovery(&temp, &mnemonic, "TestUser").await;
    let auth = services.auth();

    // Get current session
    let state = auth.subscribe().borrow().clone();
    match state {
        communitas_ui_service::auth::AuthStateSnapshot::Authenticated {
            session,
            expires_soon,
        } => {
            // Verify session has valid expiration time
            assert!(
                session.expires_at > 0,
                "Session should have expiration time"
            );

            // Verify expires_soon is calculated correctly
            let time_remaining = session.time_remaining();
            let expected_expires_soon =
                time_remaining.as_secs() < communitas_ui_service::auth::SESSION_EXPIRY_WARNING_SECS;
            assert_eq!(expires_soon, expected_expires_soon);
        }
        _ => panic!("Should be authenticated"),
    }
}

// =============================================================================
// Test 9: State subscription works correctly
// =============================================================================

#[test]
fn test_auth_state_subscription() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_auth_state_subscription_inner());
    });
}

async fn test_auth_state_subscription_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create fresh services (should start logged out)
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    // Initially should be logged out
    let initial_state = auth.subscribe().borrow().clone();
    assert!(
        matches!(
            initial_state,
            communitas_ui_service::auth::AuthStateSnapshot::LoggedOut
        ),
        "Should start logged out"
    );

    // Recover identity
    auth.recover_identity(&mnemonic, None, "TestUser", None, false)
        .await
        .expect("Recovery should succeed");

    // State should now be authenticated
    let authed_state = auth.subscribe().borrow().clone();
    assert!(
        matches!(
            authed_state,
            communitas_ui_service::auth::AuthStateSnapshot::Authenticated { .. }
        ),
        "Should be authenticated after recovery"
    );

    // Logout
    auth.logout().await.expect("Logout should succeed");

    // State should be logged out again
    let final_state = auth.subscribe().borrow().clone();
    assert!(
        matches!(
            final_state,
            communitas_ui_service::auth::AuthStateSnapshot::LoggedOut
        ),
        "Should be logged out after logout"
    );
}

// =============================================================================
// Test 10: Multiple identity isolation
// =============================================================================

#[test]
fn test_multiple_identity_isolation() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_multiple_identity_isolation_inner());
    });
}

async fn test_multiple_identity_isolation_inner() {
    let config = RecoveryConfig::default();

    // Create three separate identities with different mnemonics
    let identities = [
        (
            generate_recovery_mnemonic(&config).unwrap().to_string(),
            "Alice",
        ),
        (
            generate_recovery_mnemonic(&config).unwrap().to_string(),
            "Bob",
        ),
        (
            generate_recovery_mnemonic(&config).unwrap().to_string(),
            "Charlie",
        ),
    ];

    // Get previews
    let previews: Vec<_> = identities
        .iter()
        .map(|(m, _)| preview_identity_from_mnemonic(m.clone(), None).unwrap())
        .collect();

    // Setup shared temp directory
    let temp = TempDir::new().unwrap();
    let services = make_services(&temp).await;
    let auth = services.auth();

    // Create all identities
    for ((mnemonic, name), preview) in identities.iter().zip(previews.iter()) {
        let session = auth
            .recover_identity(mnemonic, None, name, None, false)
            .await
            .expect("Recovery should succeed");
        assert_eq!(session.four_words, preview.four_words);
    }

    // Test: Each identity can login independently with correct four_words
    for ((_, name), preview) in identities.iter().zip(previews.iter()) {
        let session = auth
            .login(&preview.four_words)
            .await
            .expect("Login should succeed");

        assert_eq!(session.four_words, preview.four_words);
        assert_eq!(session.display_name, *name);

        // Verify session
        let state = auth.subscribe().borrow().clone();
        match state {
            communitas_ui_service::auth::AuthStateSnapshot::Authenticated {
                session: s, ..
            } => {
                assert_eq!(s.four_words, preview.four_words);
                assert_eq!(s.display_name, *name);
            }
            _ => panic!("Should be authenticated"),
        }
    }
}

// =============================================================================
// Test 11: Session token uniqueness across logins
// =============================================================================

#[test]
fn test_session_persistence_across_logins() {
    run_with_large_stack(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(test_session_persistence_across_logins_inner());
    });
}

async fn test_session_persistence_across_logins_inner() {
    // Generate a fresh mnemonic
    let config = RecoveryConfig::default();
    let mnemonic = generate_recovery_mnemonic(&config).unwrap().to_string();

    // Create identity
    let temp = TempDir::new().unwrap();
    let (services, four_words) = create_identity_via_recovery(&temp, &mnemonic, "TestUser").await;
    let auth = services.auth();

    // Logout and login multiple times, tracking pubkey_hex as session identifier
    let mut session_ids = Vec::new();
    for _ in 0..5 {
        auth.logout().await.expect("Logout should succeed");

        let session = auth.login(&four_words).await.expect("Login should succeed");

        // The pubkey_hex should be consistent (same identity)
        session_ids.push(session.pubkey_hex.clone());
    }

    // All sessions should have the same pubkey_hex (same identity)
    let first_pubkey = &session_ids[0];
    for pubkey in &session_ids {
        assert_eq!(
            pubkey, first_pubkey,
            "All sessions should have same pubkey_hex"
        );
    }
}
