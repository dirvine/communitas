//! Diagnostic test to verify login function's keyring storage behavior
//! This test will show if keyring storage is failing silently during login

use anyhow::Result;
use tempfile::TempDir;
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_login_keyring_diagnostic() -> Result<()> {
    // Initialize tracing to see diagnostic logs
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_test_writer()
        .try_init();

    println!("\n=== Testing Login with Keyring Storage ===\n");

    // Create temporary storage
    let temp_dir = TempDir::new()?;
    let vault_dir = temp_dir.path().join("vaults");
    std::fs::create_dir_all(&vault_dir)?;

    // Create EncryptedStorageManager directly with keyring enabled
    use communitas_core::encrypted_storage::{EncryptedStorageManager, StorageConfig};

    let config = StorageConfig {
        vault_dir,
        pbkdf2_iterations: 100_000,
        enable_fec: true,
        fec_redundancy: 1.5,
        max_vault_size: 0,
        use_keyring: true, // Keyring enabled - critical for this test!
        cache_timeout: 3600,
    };

    let storage = EncryptedStorageManager::new(config).await?;

    // Test data
    let four_words = "test-login-keyring-storage";
    let display_name = "Test User";
    let password = "SecureTestPassword123!";

    println!("🔑 Step 1: Creating vault for '{}'", four_words);

    // Create vault
    storage
        .create_vault(four_words, display_name, password)
        .await?;

    println!("✅ Vault created successfully\n");

    println!("🔑 Step 2: Logging in (should trigger keyring storage)");
    println!("🔍 Watch for 🔑 LOGIN logs showing keyring storage attempt\n");

    // Login - this should trigger the diagnostic logging we added
    let session = storage.login(four_words, password, None).await?;

    println!("\n✅ Login successful: {:?}", session);

    println!("\n=== Check logs above for: ===");
    println!("  - '🔑 LOGIN: Attempting to store password in keyring'");
    println!("  - '✅ LOGIN: Password stored in keyring successfully'");
    println!("  OR");
    println!("  - '❌ LOGIN: Failed to store password in keyring'");
    println!("  - '⚠️ LOGIN: Keyring storage skipped'");

    Ok(())
}
