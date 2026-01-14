//! Integration tests for authentication state machine
//!
//! Tests concurrent access, race conditions, and full lifecycle scenarios.

use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use communitas_mcp::auth::{
    AuthError, AuthState, AuthenticatedSession, DelegateSession, DelegateToken, DemoSession,
    FourWordChallenge, Scope,
};

// =============================================
// Concurrent Access Tests
// =============================================

mod concurrent_access {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_reads_do_not_block() {
        let state = Arc::new(RwLock::new(AuthState::Authenticated(
            AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            },
        )));

        let mut handles = vec![];

        for i in 0..10 {
            let state_clone = Arc::clone(&state);
            handles.push(tokio::spawn(async move {
                let guard = state_clone.read().await;
                assert!(guard.is_authenticated());
                assert_eq!(guard.four_words(), Some("ocean-forest-moon-star"));
                i
            }));
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_sequential_state_transitions() {
        let state = Arc::new(RwLock::new(AuthState::Unauthenticated));

        // Transition to authenticated
        {
            let mut guard = state.write().await;
            let session = AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            };
            let result = guard.authenticate(session);
            assert!(result.is_ok());
        }

        // Verify authenticated
        {
            let guard = state.read().await;
            assert!(guard.is_authenticated());
            assert_eq!(guard.state_name(), "Authenticated");
        }

        // Revoke
        {
            let mut guard = state.write().await;
            let result = guard.revoke();
            assert!(result.is_ok());
        }

        // Verify unauthenticated
        {
            let guard = state.read().await;
            assert!(!guard.is_authenticated());
            assert_eq!(guard.state_name(), "Unauthenticated");
        }
    }

    #[tokio::test]
    async fn test_multiple_authenticate_attempts_only_first_succeeds() {
        let state = Arc::new(RwLock::new(AuthState::Unauthenticated));
        let mut handles = vec![];

        for i in 0..5 {
            let state_clone = Arc::clone(&state);
            handles.push(tokio::spawn(async move {
                let mut guard = state_clone.write().await;
                let session = AuthenticatedSession {
                    four_words: format!("word{}-word-word-word", i),
                    display_name: format!("User {}", i),
                    device_name: format!("Device {}", i),
                    started_at: SystemTime::now(),
                    storage_dir: format!("/tmp/test{}", i),
                };
                guard.authenticate(session)
            }));
        }

        let mut success_count = 0;
        let mut fail_count = 0;

        for handle in handles {
            match handle.await.expect("Task should complete") {
                Ok(()) => success_count += 1,
                Err(AuthError::InvalidTransition { .. }) => fail_count += 1,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        assert_eq!(success_count, 1);
        assert_eq!(fail_count, 4);

        let guard = state.read().await;
        assert!(guard.is_authenticated());
    }

    #[tokio::test]
    async fn test_state_consistency_under_load() {
        let state = Arc::new(RwLock::new(AuthState::Unauthenticated));
        let mut handles = vec![];

        {
            let mut guard = state.write().await;
            let session = AuthenticatedSession {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: "Test User".to_string(),
                device_name: "Test Device".to_string(),
                started_at: SystemTime::now(),
                storage_dir: "/tmp/test".to_string(),
            };
            guard.authenticate(session).expect("Should authenticate");
        }

        for _ in 0..100 {
            let state_clone = Arc::clone(&state);
            handles.push(tokio::spawn(async move {
                let guard = state_clone.read().await;
                guard.is_authenticated()
            }));
        }

        for handle in handles {
            let is_auth = handle.await.expect("Task should complete");
            assert!(is_auth);
        }
    }
}

// =============================================
// Full Lifecycle Tests
// =============================================

mod lifecycle {
    use super::*;

    #[test]
    fn test_full_auth_lifecycle() {
        let mut state = AuthState::default();

        assert!(!state.is_authenticated());
        assert_eq!(state.state_name(), "Unauthenticated");
        assert!(state.four_words().is_none());
        assert!(state.started_at().is_none());

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
        assert!(state.started_at().is_some());

        let new_session = AuthenticatedSession {
            four_words: "another-four-word-id".to_string(),
            display_name: "Another User".to_string(),
            device_name: "Another Device".to_string(),
            started_at: SystemTime::now(),
            storage_dir: "/tmp/test2".to_string(),
        };
        let result = state.authenticate(new_session);
        assert!(result.is_err());

        let result = state.revoke();
        assert!(result.is_ok());

        assert!(!state.is_authenticated());
        assert_eq!(state.state_name(), "Unauthenticated");
        assert!(state.four_words().is_none());

        let session = AuthenticatedSession {
            four_words: "new-four-word-identity".to_string(),
            display_name: "New User".to_string(),
            device_name: "New Device".to_string(),
            started_at: SystemTime::now(),
            storage_dir: "/tmp/test3".to_string(),
        };
        let result = state.authenticate(session);
        assert!(result.is_ok());
        assert!(state.is_authenticated());
    }

    #[test]
    fn test_demo_mode_lifecycle() {
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

        let session = DemoSession::new(
            "another-demo-four-word".to_string(),
            "Another Demo".to_string(),
            "/tmp/demo2".to_string(),
        );
        let result = state.start_demo(session);
        assert!(result.is_err());

        state.revoke().expect("Should revoke");
        let session = DemoSession::new(
            "new-demo-four-word".to_string(),
            "New Demo".to_string(),
            "/tmp/demo3".to_string(),
        );
        let result = state.start_demo(session);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delegate_lifecycle() {
        let mut state = AuthState::default();

        let session = DelegateSession {
            issuer_four_words: "ocean-forest-moon-star".to_string(),
            delegate_name: "test-agent".to_string(),
            scopes: vec![Scope::ReadMessages, Scope::SendMessages],
            started_at: SystemTime::now(),
            storage_dir: "/tmp/delegate".to_string(),
        };
        let result = state.delegate(session);
        assert!(result.is_ok());
        assert!(state.is_authenticated());
        assert_eq!(state.state_name(), "Delegate");

        assert_eq!(state.four_words(), Some("ocean-forest-moon-star"));

        state.revoke().expect("Should revoke");
        assert!(!state.is_authenticated());
    }
}

// =============================================
// Four-Word Validation Edge Cases
// =============================================

mod four_word_edge_cases {
    use super::*;

    #[test]
    fn test_empty_words() {
        let result = FourWordChallenge::validate("ocean--moon-star");
        assert!(result.is_err());
    }

    #[test]
    fn test_leading_separator() {
        let result = FourWordChallenge::validate("-ocean-forest-moon");
        assert!(result.is_err());
    }

    #[test]
    fn test_trailing_separator() {
        let result = FourWordChallenge::validate("ocean-forest-moon-");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_only() {
        let result = FourWordChallenge::validate("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_mixed_valid_invalid_words() {
        let result = FourWordChallenge::validate("ocean-FOREST-moon-star");
        assert!(result.is_err());
    }

    #[test]
    fn test_normalize_preserves_valid_input() {
        let input = "ocean-forest-moon-star";
        let result = FourWordChallenge::normalize(input);
        assert!(result.is_ok());
        assert_eq!(result.ok(), Some(input.to_string()));
    }

    #[test]
    fn test_all_same_separator() {
        let result = FourWordChallenge::validate("ocean.forest.moon.star");
        assert!(result.is_ok());

        let result = FourWordChallenge::validate("ocean forest moon star");
        assert!(result.is_ok());
    }
}

// =============================================
// Token Edge Cases
// =============================================

mod token_edge_cases {
    use super::*;

    #[test]
    fn test_token_with_empty_scopes() {
        let token = DelegateToken {
            issuer: "ocean-forest-moon-star".to_string(),
            delegate_name: "test-agent".to_string(),
            scopes: vec![],
            issued_at: 0,
            expires_at: u64::MAX,
            nonce: "test".to_string(),
        };

        assert!(!token.has_scope(&Scope::ReadMessages));
        assert!(!token.has_scope(&Scope::Full));
    }

    #[test]
    fn test_token_boundary_expiration() {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let token = DelegateToken {
            issuer: "ocean-forest-moon-star".to_string(),
            delegate_name: "test-agent".to_string(),
            scopes: vec![Scope::Full],
            issued_at: now - 1,
            expires_at: now,
            nonce: "test".to_string(),
        };

        assert!(token.is_expired());
    }

    #[test]
    fn test_delegate_session_scope_combinations() {
        let session = DelegateSession {
            issuer_four_words: "ocean-forest-moon-star".to_string(),
            delegate_name: "test-agent".to_string(),
            scopes: vec![Scope::ReadMessages, Scope::SendMessages, Scope::ReadFiles],
            started_at: SystemTime::now(),
            storage_dir: "/tmp/delegate".to_string(),
        };

        assert!(session.has_scope(&Scope::ReadMessages));
        assert!(session.has_scope(&Scope::SendMessages));
        assert!(session.has_scope(&Scope::ReadFiles));
        assert!(!session.has_scope(&Scope::WriteFiles));
        assert!(!session.has_scope(&Scope::ManageNetwork));
    }
}

// =============================================
// Error Handling Tests
// =============================================

mod error_handling {
    use super::*;

    #[test]
    fn test_double_revoke_fails() {
        let mut state = AuthState::Authenticated(AuthenticatedSession {
            four_words: "ocean-forest-moon-star".to_string(),
            display_name: "Test User".to_string(),
            device_name: "Test Device".to_string(),
            started_at: SystemTime::now(),
            storage_dir: "/tmp/test".to_string(),
        });

        let result = state.revoke();
        assert!(result.is_ok());

        let result = state.revoke();
        assert!(result.is_err());
        match result {
            Err(AuthError::NotAuthenticated) => {}
            _ => panic!("Expected NotAuthenticated error"),
        }
    }

    #[test]
    fn test_authenticated_session_with_invalid_four_words() {
        let result = AuthenticatedSession::new(
            "not-valid".to_string(),
            "Test User".to_string(),
            "Test Device".to_string(),
            "/tmp/test".to_string(),
        );

        assert!(result.is_err());
        match result {
            Err(AuthError::InvalidFourWords(msg)) => {
                assert!(msg.contains("Expected 4 words"));
            }
            _ => panic!("Expected InvalidFourWords error"),
        }
    }

    #[test]
    fn test_delegate_session_with_invalid_issuer() {
        let result = DelegateSession::new(
            "INVALID".to_string(),
            "test-agent".to_string(),
            vec![Scope::Full],
            "/tmp/delegate".to_string(),
        );

        assert!(result.is_err());
        match result {
            Err(AuthError::InvalidFourWords(msg)) => {
                assert!(
                    msg.contains("Expected 4 words") || msg.contains("invalid characters"),
                    "Expected message about word count or invalid characters, got: {}",
                    msg
                );
            }
            _ => panic!("Expected InvalidFourWords error"),
        }
    }
}
