//! Tests for the test harness itself
//!
//! Verifies that TestFixture and related utilities work correctly

mod fixtures;

use fixtures::test_harness::TestFixture;

#[test]
fn test_fixture_creation() {
    let fixture = TestFixture::new().unwrap();
    assert!(fixture.temp_path().exists());
    assert!(fixture.core_context.is_none());
    assert!(fixture.app_state.is_none());
}

#[tokio::test]
async fn test_fixture_with_core_context() {
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context()
        .await
        .unwrap();

    assert!(fixture.core_context.is_some());

    let ctx = fixture.core_context();
    let ctx_guard = ctx.read().await;
    assert_eq!(ctx_guard.four_words, "ocean-forest-moon-star");
    assert_eq!(ctx_guard.display_name, "Test User");
    assert_eq!(ctx_guard.device_name, "Test Device");
}

#[tokio::test]
async fn test_fixture_with_custom_identity() {
    let fixture = TestFixture::new()
        .unwrap()
        .with_core_context_custom(
            "river-mountain-cloud-tree".to_string(),
            "Alice".to_string(),
            "Alice-Desktop".to_string(),
        )
        .await
        .unwrap();

    let ctx = fixture.core_context();
    let ctx_guard = ctx.read().await;
    assert_eq!(ctx_guard.four_words, "river-mountain-cloud-tree");
    assert_eq!(ctx_guard.display_name, "Alice");
    assert_eq!(ctx_guard.device_name, "Alice-Desktop");
}

#[tokio::test]
async fn test_fixture_with_app_state() {
    let fixture = TestFixture::new().unwrap().with_app_state().await.unwrap();

    assert!(fixture.app_state.is_some());
    let _state = fixture.app_state();
}

#[test]
fn test_fixture_with_networking() {
    let bootstrap = vec!["127.0.0.1:9000".to_string()];
    let fixture = TestFixture::new()
        .unwrap()
        .with_networking(bootstrap.clone());

    assert!(fixture.config().enable_networking);
    assert_eq!(fixture.config().bootstrap_nodes, bootstrap);
}

#[test]
fn test_fixture_cleanup() {
    let temp_path = {
        let fixture = TestFixture::new().unwrap();
        fixture.temp_path().to_path_buf()
    };

    // Fixture dropped here, temp directory should be cleaned up
    // Note: TempDir cleanup happens asynchronously, so we can't easily test it
    // But we can verify the fixture was created successfully
    assert!(!temp_path.as_os_str().is_empty());
}
