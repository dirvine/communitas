// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for the ContactsService.
//!
//! These tests verify the contacts service layer works correctly
//! when connected to the core API.

use std::sync::Arc;

use communitas_core::app::CommunitasApp;
use communitas_ui_service::auth::AuthController;
use communitas_ui_service::contacts::{ContactsError, ContactsService};
use communitas_ui_service::storage::UiStorage;
use tempfile::TempDir;

/// Helper to create a contacts service with no authentication.
fn make_contacts_service(temp: &TempDir) -> ContactsService {
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let auth = Arc::new(AuthController::new(storage).unwrap());
    ContactsService::new(auth)
}

#[tokio::test]
async fn test_add_contact_not_authenticated() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Should fail because we're not authenticated
    let result = service.add_contact("ocean-forest-moon-star", "Alice").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::NotAuthenticated => {} // expected
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test]
async fn test_add_contact_invalid_four_words_wrong_count() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Three words - invalid
    let result = service.add_contact("ocean-forest-moon", "Alice").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::InvalidFourWords(_) => {} // expected
        other => panic!("expected InvalidFourWords, got {other:?}"),
    }
}

#[tokio::test]
async fn test_add_contact_invalid_four_words_empty_word() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Empty word in the middle - invalid
    let result = service.add_contact("ocean--moon-star", "Alice").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::InvalidFourWords(_) => {} // expected
        other => panic!("expected InvalidFourWords, got {other:?}"),
    }
}

#[tokio::test]
async fn test_add_contact_invalid_four_words_numbers() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Numbers in words - invalid
    let result = service
        .add_contact("ocean-forest123-moon-star", "Alice")
        .await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::InvalidFourWords(_) => {} // expected
        other => panic!("expected InvalidFourWords, got {other:?}"),
    }
}

#[tokio::test]
async fn test_delete_contact_not_authenticated() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Should fail because we're not authenticated
    let result = service.delete_contact("some-contact-id").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::NotAuthenticated => {} // expected
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test]
async fn test_update_contact_not_authenticated() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Should fail because we're not authenticated
    let result = service
        .update_contact("some-contact-id", Some("New Name"), None)
        .await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::NotAuthenticated => {} // expected
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test]
async fn test_toggle_favourite_not_authenticated() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Should fail because we're not authenticated
    let result = service.toggle_favourite("some-contact-id", true).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::NotAuthenticated => {} // expected
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test]
async fn test_link_contact_not_authenticated() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Should fail because we're not authenticated
    let result = service
        .link_contact("some-contact-id", "ocean-forest-moon-star")
        .await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::NotAuthenticated => {} // expected
        other => panic!("expected NotAuthenticated, got {other:?}"),
    }
}

#[tokio::test]
async fn test_link_contact_invalid_four_words() {
    let temp = TempDir::new().unwrap();
    let service = make_contacts_service(&temp);

    // Should fail due to invalid four-words format before even checking auth
    let result = service.link_contact("some-contact-id", "invalid").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        ContactsError::InvalidFourWords(_) => {} // expected
        other => panic!("expected InvalidFourWords, got {other:?}"),
    }
}

// ============================================================================
// Integration tests with authenticated state
// ============================================================================

/// Helper to create authenticated services for integration testing.
#[allow(dead_code)]
async fn make_authenticated_services(temp: &TempDir) -> (ContactsService, Arc<CommunitasApp>) {
    // Set up storage
    let storage = UiStorage::from_path(temp.path()).unwrap();
    let storage_path = temp
        .path()
        .join("app_storage")
        .to_string_lossy()
        .to_string();

    // Create a real CommunitasApp instance
    let app = Arc::new(
        CommunitasApp::new(
            "ocean-forest-moon-star".to_string(),
            "TestUser".to_string(),
            "TestDevice".to_string(),
            storage_path,
        )
        .await
        .unwrap(),
    );

    // Create auth controller and contacts service
    let auth = Arc::new(AuthController::new(storage).unwrap());
    let contacts = ContactsService::new(auth);

    (contacts, app)
}

// Note: Full integration tests with authenticated state require starting
// the networking layer, which involves async coordination. These tests
// are better suited for end-to-end testing scenarios.
//
// The unit tests above verify:
// 1. Four-words validation works correctly
// 2. All operations require authentication
// 3. Error types are properly returned
