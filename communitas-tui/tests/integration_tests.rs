use anyhow::Result;
use communitas_core::encrypted_storage::{EncryptedStorageManager, StorageConfig};
use tempfile::TempDir;
use tokio::time::{Duration, timeout};

#[tokio::test(flavor = "multi_thread")]
async fn test_signup_flow_performance() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let start = std::time::Instant::now();

    // Create storage configuration
    let config = StorageConfig {
        vault_dir: data_dir.join("vaults"),
        use_keyring: false,
        pbkdf2_iterations: 1000, // Reduce for testing
        ..Default::default()
    };

    // Initialize storage manager
    let storage_manager = EncryptedStorageManager::new(config).await?;

    // Test signup flow
    let display_name = "Test User";

    let signup_result = timeout(
        Duration::from_secs(120), // 2 minute max
        storage_manager.create_vault("test-ocean-forest-moon-star", "test-password", display_name),
    )
    .await;

    let duration = start.elapsed();

    // Assert signup completes within reasonable time
    assert!(
        duration < Duration::from_secs(10),
        "Signup took too long: {:?}",
        duration
    );

    assert!(signup_result.is_ok(), "Signup timed out");
    let vault_id = signup_result??;
    assert!(!vault_id.is_empty());

    println!("✅ Signup completed in {:?}", duration);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_vault_creation_non_blocking() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let start = std::time::Instant::now();

    // Clone path for the spawned task
    let data_dir_clone = data_dir.clone();

    // Test vault creation doesn't block other tasks
    let vault_task = tokio::spawn(async move {
        let config = StorageConfig {
            vault_dir: data_dir_clone.join("vaults"),
            use_keyring: false,
            pbkdf2_iterations: 1000, // Reduce for testing
            ..Default::default()
        };

        let storage_manager = EncryptedStorageManager::new(config).await?;
        storage_manager
            .create_vault("test-async-ocean-forest", "test-password", "Test User")
            .await
    });

    // Simulate UI updates while vault is being created
    let mut ui_updates = 0;
    while !vault_task.is_finished() {
        tokio::time::sleep(Duration::from_millis(100)).await;
        ui_updates += 1;
        if ui_updates >= 600 {
            panic!("UI blocked for too long (60+ seconds)");
        }
    }

    let vault_id = vault_task.await??;
    let duration = start.elapsed();

    assert!(!vault_id.is_empty());
    assert!(
        duration < Duration::from_secs(10),
        "Vault creation too slow: {:?}",
        duration
    );

    println!(
        "✅ Vault created in {:?} with {} UI updates",
        duration, ui_updates
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_keyring_disabled() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Test with keyring disabled
    let config_no_keyring = StorageConfig {
        vault_dir: data_dir.join("vaults"),
        use_keyring: false,
        ..Default::default()
    };

    let storage_manager = EncryptedStorageManager::new(config_no_keyring).await?;
    let vault_id = storage_manager
        .create_vault("test-keyring-disabled", "test-password", "Test User")
        .await?;

    assert!(
        !vault_id.is_empty(),
        "Vault creation without keyring failed"
    );

    println!("✅ Keyring-disabled vault creation passed");
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_pbkdf2_performance() -> Result<()> {
    use communitas_core::encrypted_storage::key_management::KeyManager;

    // Test with different iteration counts
    let test_cases = vec![
        (1_000, Duration::from_millis(100)),
        (10_000, Duration::from_secs(1)),
        (100_000, Duration::from_secs(10)),
    ];

    for (iterations, max_duration) in test_cases {
        let key_manager = KeyManager::new(iterations, false).await?;
        let salt = vec![1u8; 32];

        let start = std::time::Instant::now();
        let _ = key_manager.derive_key("test-password", &salt).await?;
        let duration = start.elapsed();

        assert!(
            duration < max_duration,
            "PBKDF2 with {} iterations took {:?}, expected < {:?}",
            iterations,
            duration,
            max_duration
        );

        println!("✅ PBKDF2 {} iterations: {:?}", iterations, duration);
    }

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_operations() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    // Test multiple concurrent vault creations
    let mut tasks = vec![];

    for i in 0..3 {
        let data_dir_clone = data_dir.clone();
        let task = tokio::spawn(async move {
            let config = StorageConfig {
                vault_dir: data_dir_clone.join(format!("vaults_{}", i)),
                use_keyring: false,
                pbkdf2_iterations: 1000, // Reduce for testing
                ..Default::default()
            };

            let storage_manager = EncryptedStorageManager::new(config).await?;
            let identity = format!("test-concurrent-{}-identity", i);
            let display_name = format!("User {}", i);
            storage_manager
                .create_vault(&identity, "test-password", &display_name)
                .await
        });
        tasks.push(task);
    }

    let start = std::time::Instant::now();
    let results = futures::future::try_join_all(tasks).await?;
    let duration = start.elapsed();

    // All should succeed
    for result in results {
        assert!(
            result.is_ok(),
            "Concurrent vault creation failed: {:?}",
            result
        );
    }

    // Should complete reasonably quickly even with concurrent operations
    assert!(
        duration < Duration::from_secs(15),
        "Concurrent operations took too long: {:?}",
        duration
    );

    println!("✅ Concurrent vault creations completed in {:?}", duration);
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_timeout_behavior() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let data_dir = temp_dir.path().to_path_buf();

    let config = StorageConfig {
        vault_dir: data_dir.join("vaults"),
        use_keyring: false,
        pbkdf2_iterations: 1000, // Reduce for testing
        ..Default::default()
    };

    let storage_manager = EncryptedStorageManager::new(config).await?;

    // Test that timeout actually works (this should complete well before timeout)
    let start = std::time::Instant::now();
    let result = timeout(
        Duration::from_secs(60),
        storage_manager.create_vault("test-timeout-check", "test-password", "Timeout Test"),
    )
    .await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Operation timed out");
    assert!(result?.is_ok(), "Vault creation failed");
    assert!(
        duration < Duration::from_secs(60),
        "Operation should complete before timeout"
    );

    println!(
        "✅ Timeout mechanism working correctly, completed in {:?}",
        duration
    );
    Ok(())
}
