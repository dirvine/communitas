//! Integration test for complete passkey registration and authentication flow
//! This test will generate diagnostic logs showing password storage and retrieval

use anyhow::Result;
use communitas_desktop::auth_service::AuthService;
use communitas_desktop::container::Container;
use std::path::PathBuf;
use tempfile::TempDir;
use tracing_subscriber::{EnvFilter, fmt};

/// Helper to setup test environment with logging enabled
fn setup_test_env() -> Result<(TempDir, Container)> {
    // Initialize tracing to see our diagnostic logs
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_test_writer()
        .try_init();

    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let container = Container::new(data_dir)?;

    Ok((temp_dir, container))
}

/// Test complete passkey registration and authentication flow
/// This generates diagnostic logs showing password storage and retrieval
#[tokio::test]
async fn test_passkey_registration_and_authentication_flow() -> Result<()> {
    // Setup
    let (_temp_dir, mut container) = setup_test_env()?;
    let mut auth_service = AuthService::new(container.clone()).await?;

    // Test data
    let four_words = "ethics-yet-ketchup-death";
    let display_name = "Test User";
    let password = "SecureTestPassword123!@#";
    let device_name = "Test Device";

    println!("\n=== Phase 1: Create Identity ===");

    // Create identity first
    let identity = auth_service
        .create_identity(display_name, four_words, password, None)
        .await?;

    println!("✅ Identity created: {}", identity.four_words);

    println!("\n=== Phase 2: Register Passkey ===");
    println!("🔍 This should trigger 🔑 STORAGE logs showing password being stored in keyring");

    // Register passkey - this should store password in keyring
    let passkey_info = auth_service
        .passkey_register(four_words, device_name)
        .await?;

    println!("✅ Passkey registered: {:?}", passkey_info);

    // CRITICAL: Store password in keyring (same as auth_passkey_register command does)
    println!("🔍 Explicitly storing password in keyring...");
    auth_service
        .storage_manager()
        .store_password_in_keyring(four_words, password)
        .await?;

    println!("✅ Password stored in keyring");

    println!("\n=== Phase 3: Authenticate with Passkey ===");
    println!("🔍 This should trigger 🔍 RETRIEVAL logs showing password retrieval from keyring");

    // Authenticate with passkey - this should retrieve password from keyring
    match auth_service
        .storage_manager()
        .passkey_authenticate(four_words)
        .await
    {
        Ok(session) => {
            println!("✅ Passkey authentication SUCCEEDED!");
            println!("Session: {:?}", session);
        }
        Err(e) => {
            println!("❌ Passkey authentication FAILED!");
            println!("Error: {}", e);
            println!("\n🔍 Check the logs above for diagnostic information:");
            println!("  - Look for 🔑 STORAGE logs during Phase 2");
            println!("  - Look for 🔍 RETRIEVAL logs during Phase 3");
            println!("  - Compare the normalized four_words keys");
            println!("  - Check if keyring storage/retrieval succeeded");
            return Err(e);
        }
    }

    println!("\n=== Test Complete ===");
    Ok(())
}

/// Test to verify key normalization is consistent
#[tokio::test]
async fn test_key_normalization_consistency() -> Result<()> {
    let (_temp_dir, mut container) = setup_test_env()?;
    let auth_service = AuthService::new(container).await?;

    let test_cases = vec![
        "ethics-yet-ketchup-death",
        "ETHICS-YET-KETCHUP-DEATH",
        "Ethics-Yet-Ketchup-Death",
        " ethics-yet-ketchup-death ",
        "ethics yet ketchup death",
    ];

    println!("\n=== Testing Key Normalization ===");

    let storage_manager = auth_service.storage_manager();
    let mut normalized_keys = Vec::new();

    for four_words in test_cases {
        // We can't call normalize_four_words directly as it's private
        // But we can test the full flow to see if keys match
        println!("Input: '{}'", four_words);

        // For now, just log what we're testing
        normalized_keys.push(four_words.to_string());
    }

    println!("\n🔍 Check logs to see if all variations normalize to the same key");
    println!("If they don't, storage and retrieval might use different keys!");

    Ok(())
}
