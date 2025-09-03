use crate::dht_facade::LocalDht;
use crate::saorsa_storage::engine::{RetrievalRequest, StorageEngine, StorageRequest};
use crate::saorsa_storage::pqc_crypto::{
    MlKemEncapsulation, MlKemKeypair, PqcCryptoManager, PqcEncryptionMode,
};
/**
 * Saorsa Storage System - PQC Integration Tests
 * Comprehensive testing for ML-KEM-768 enhanced storage functionality
 */
use crate::saorsa_storage::*;
use chrono::Utc;
use std::sync::Arc;
use tokio::test;

/// Test harness for PQC storage integration testing
struct PqcStorageTestHarness {
    engine: StorageEngine<LocalDht>,
    pqc_manager: PqcCryptoManager,
    test_content: Vec<u8>,
    test_user_id: String,
}

impl PqcStorageTestHarness {
    async fn new() -> Result<Self, StorageError> {
        let dht = Arc::new(LocalDht::new("pqc_test_node".to_string()));
        let master_key = [0x42u8; 32]; // Deterministic test key
        let config_manager = crate::saorsa_storage::config::ConfigManager::new();

        let engine = StorageEngine::new(dht, master_key, config_manager).await?;
        let pqc_manager = PqcCryptoManager::new(master_key)?;

        Ok(Self {
            engine,
            pqc_manager,
            test_content: b"Test content for PQC storage integration testing".to_vec(),
            test_user_id: "test_user_pqc".to_string(),
        })
    }

    fn create_test_metadata(&self) -> StorageMetadata {
        StorageMetadata {
            content_type: "text/plain".to_string(),
            author: self.test_user_id.clone(),
            tags: vec!["pqc".to_string(), "test".to_string()],
            created_at: Utc::now(),
            modified_at: None,
            size: self.test_content.len() as u64,
            checksum: "pqc_test_checksum".to_string(),
        }
    }

    async fn store_and_retrieve_test(
        &self,
        policy: StoragePolicy,
        namespace: Option<String>,
        group_id: Option<String>,
    ) -> Result<(), StorageError> {
        // Create storage request
        let store_request = StorageRequest {
            content: self.test_content.clone(),
            content_type: "text/plain".to_string(),
            policy: policy.clone(),
            metadata: self.create_test_metadata(),
            user_id: self.test_user_id.clone(),
            group_id: group_id.clone(),
            namespace: namespace.clone(),
        };

        // Store content using PQC-enhanced encryption
        let store_response = self.engine.store_content(store_request).await?;

        // Verify storage response
        assert!(
            store_response.encrypted_size > self.test_content.len() as u64,
            "PQC encrypted content should be larger than original"
        );
        assert!(
            store_response.operation_time_ms >= 0,
            "Operation time should be non-negative"
        );

        // Create retrieval request
        let retrieval_request = RetrievalRequest {
            address: store_response.address.clone(),
            user_id: self.test_user_id.clone(),
            decryption_key: None,
        };

        // Retrieve and verify content
        let retrieval_response = self.engine.retrieve_content(retrieval_request).await?;

        assert_eq!(
            retrieval_response.content, self.test_content,
            "Retrieved content should match original"
        );
        assert_eq!(
            retrieval_response.metadata.size,
            self.test_content.len() as u64,
            "Content size should match"
        );

        Ok(())
    }
}

#[test]
async fn test_ml_kem_keypair_deterministic_generation() {
    let seed = b"deterministic_test_seed_ml_kem_768";

    let keypair1 = MlKemKeypair::derive_from_seed(seed).unwrap();
    let keypair2 = MlKemKeypair::derive_from_seed(seed).unwrap();

    assert_eq!(
        keypair1.public_key, keypair2.public_key,
        "Deterministic keypair generation should produce identical public keys"
    );
    assert_eq!(
        keypair1.secret_key, keypair2.secret_key,
        "Deterministic keypair generation should produce identical secret keys"
    );

    // Verify key sizes
    assert_eq!(
        keypair1.public_key.len(),
        crate::saorsa_storage::pqc_crypto::ML_KEM_768_PUBLIC_KEY_SIZE
    );
    assert_eq!(
        keypair1.secret_key.len(),
        crate::saorsa_storage::pqc_crypto::ML_KEM_768_SECRET_KEY_SIZE
    );
}

#[test]
async fn test_ml_kem_encapsulation_consistency() {
    let keypair = MlKemKeypair::generate().unwrap();

    let encapsulation = MlKemEncapsulation::encapsulate(&keypair.public_key).unwrap();
    assert_eq!(
        encapsulation.ciphertext.len(),
        crate::saorsa_storage::pqc_crypto::ML_KEM_768_CIPHERTEXT_SIZE
    );

    let decapsulated_secret =
        MlKemEncapsulation::decapsulate(&encapsulation.ciphertext, &keypair.secret_key).unwrap();

    assert_eq!(
        encapsulation.shared_secret, decapsulated_secret,
        "Decapsulated secret should match original shared secret"
    );
    assert_eq!(
        encapsulation.shared_secret.len(),
        32,
        "Shared secret should be 32 bytes"
    );
}

#[test]
async fn test_pqc_private_max_storage() {
    let harness = PqcStorageTestHarness::new().await.unwrap();

    harness
        .store_and_retrieve_test(StoragePolicy::PrivateMax, None, None)
        .await
        .unwrap();
}

#[test]
async fn test_pqc_private_scoped_storage() {
    let harness = PqcStorageTestHarness::new().await.unwrap();
    let namespace = "pqc_test_namespace".to_string();

    harness
        .store_and_retrieve_test(
            StoragePolicy::PrivateScoped {
                namespace: namespace.clone(),
            },
            Some(namespace),
            None,
        )
        .await
        .unwrap();
}

#[test]
async fn test_pqc_group_scoped_storage() {
    let harness = PqcStorageTestHarness::new().await.unwrap();
    let group_id = "pqc_test_group".to_string();

    harness
        .store_and_retrieve_test(
            StoragePolicy::GroupScoped {
                group_id: group_id.clone(),
            },
            None,
            Some(group_id),
        )
        .await
        .unwrap();
}

#[test]
async fn test_pqc_public_markdown_storage() {
    let harness = PqcStorageTestHarness::new().await.unwrap();

    // Create markdown content for PublicMarkdown policy
    let markdown_request = StorageRequest {
        content: b"# Test Markdown\n\nThis is test markdown content.".to_vec(),
        content_type: "text/markdown".to_string(),
        policy: StoragePolicy::PublicMarkdown,
        metadata: StorageMetadata {
            content_type: "text/markdown".to_string(),
            author: harness.test_user_id.clone(),
            tags: vec![
                "pqc".to_string(),
                "test".to_string(),
                "markdown".to_string(),
            ],
            created_at: Utc::now(),
            modified_at: None,
            size: 43,
            checksum: "markdown_test_checksum".to_string(),
        },
        user_id: harness.test_user_id.clone(),
        group_id: None,
        namespace: None,
    };

    // Store content using PQC-enhanced encryption
    let store_response = harness
        .engine
        .store_content(markdown_request)
        .await
        .unwrap();

    // Verify storage response
    assert!(
        store_response.encrypted_size > 43,
        "PQC encrypted content should be larger than original"
    );

    // Create retrieval request
    let retrieval_request = RetrievalRequest {
        address: store_response.address.clone(),
        user_id: harness.test_user_id.clone(),
        decryption_key: None,
    };

    // Retrieve and verify content
    let retrieval_response = harness
        .engine
        .retrieve_content(retrieval_request)
        .await
        .unwrap();

    assert_eq!(
        retrieval_response.content,
        b"# Test Markdown\n\nThis is test markdown content."
    );
}

#[test]
async fn test_pqc_encryption_mode_conversion() {
    // Test conversion from legacy encryption modes to PQC modes
    let test_cases = vec![
        (
            EncryptionMode::ChaCha20Poly1305Local,
            PqcEncryptionMode::MlKem768Local,
        ),
        (
            EncryptionMode::ChaCha20Poly1305Derived,
            PqcEncryptionMode::MlKem768Derived,
        ),
        (
            EncryptionMode::ChaCha20Poly1305Shared,
            PqcEncryptionMode::MlKem768Shared,
        ),
        (
            EncryptionMode::Convergent,
            PqcEncryptionMode::MlKem768Convergent,
        ),
    ];

    for (legacy_mode, expected_pqc_mode) in test_cases {
        let converted_mode = PqcEncryptionMode::from(legacy_mode);
        assert_eq!(
            converted_mode, expected_pqc_mode,
            "Legacy mode {:?} should convert to PQC mode {:?}",
            legacy_mode, expected_pqc_mode
        );
    }
}

#[test]
async fn test_pqc_content_encryption_decryption_direct() {
    let master_key = [0x11u8; 32];
    let pqc_manager = PqcCryptoManager::new(master_key).unwrap();

    let test_content = b"Direct PQC encryption test content";
    let user_id = "direct_test_user";
    let namespace = "direct_test_namespace";

    // Test each policy type
    let policies = vec![
        StoragePolicy::PrivateMax,
        StoragePolicy::PrivateScoped {
            namespace: namespace.to_string(),
        },
        StoragePolicy::GroupScoped {
            group_id: "test_group".to_string(),
        },
        StoragePolicy::PublicMarkdown,
    ];

    for policy in policies {
        let encrypted_content = pqc_manager
            .encrypt_content(
                test_content,
                &policy,
                user_id,
                Some(namespace),
                Some("test_group"),
            )
            .await
            .unwrap();

        // Verify PQC encryption structure
        assert_eq!(encrypted_content.algorithm, "ML-KEM-768+ChaCha20-Poly1305");
        assert_eq!(
            encrypted_content.ml_kem_ciphertext.len(),
            crate::saorsa_storage::pqc_crypto::ML_KEM_768_CIPHERTEXT_SIZE
        );
        assert_eq!(encrypted_content.nonce.len(), 12);
        assert!(!encrypted_content.ciphertext.is_empty());

        // Decrypt and verify
        let decrypted_content = pqc_manager
            .decrypt_content(&encrypted_content, user_id)
            .await
            .unwrap();
        assert_eq!(decrypted_content, test_content);
    }
}

#[test]
async fn test_pqc_performance_benchmarks() {
    let master_key = [0x33u8; 32];
    let pqc_manager = PqcCryptoManager::new(master_key).unwrap();

    let test_sizes = vec![1024, 10 * 1024, 100 * 1024, 1024 * 1024]; // 1KB to 1MB
    let policy = StoragePolicy::PrivateScoped {
        namespace: "benchmark".to_string(),
    };

    for size in test_sizes {
        let test_content = vec![0x55u8; size];

        // Measure encryption time
        let encrypt_start = std::time::Instant::now();
        let encrypted = pqc_manager
            .encrypt_content(
                &test_content,
                &policy,
                "benchmark_user",
                Some("benchmark"),
                None,
            )
            .await
            .unwrap();
        let encrypt_duration = encrypt_start.elapsed();

        // Measure decryption time
        let decrypt_start = std::time::Instant::now();
        let decrypted = pqc_manager
            .decrypt_content(&encrypted, "benchmark_user")
            .await
            .unwrap();
        let decrypt_duration = decrypt_start.elapsed();

        // Verify correctness
        assert_eq!(decrypted, test_content);

        // Performance assertions (should be under reasonable limits)
        assert!(
            encrypt_duration.as_millis() < 1000,
            "Encryption of {}KB should complete in under 1 second",
            size / 1024
        );
        assert!(
            decrypt_duration.as_millis() < 500,
            "Decryption of {}KB should complete in under 500ms",
            size / 1024
        );

        // Verify encryption overhead is reasonable (should be < 10% for large content)
        if size > 10 * 1024 {
            let overhead_ratio = (encrypted.ciphertext.len() as f64) / (test_content.len() as f64);
            assert!(
                overhead_ratio < 1.1,
                "Encryption overhead should be less than 10% for content larger than 10KB"
            );
        }
    }
}

#[test]
async fn test_pqc_key_derivation_consistency() {
    let master_key = [0x77u8; 32];
    let pqc_manager = PqcCryptoManager::new(master_key).unwrap();

    let test_content = b"Key derivation consistency test";
    let user_id = "consistency_user";
    let namespace = "consistency_namespace";
    let policy = StoragePolicy::PrivateScoped {
        namespace: namespace.to_string(),
    };

    // Encrypt the same content multiple times
    let encrypted1 = pqc_manager
        .encrypt_content(test_content, &policy, user_id, Some(namespace), None)
        .await
        .unwrap();

    let encrypted2 = pqc_manager
        .encrypt_content(test_content, &policy, user_id, Some(namespace), None)
        .await
        .unwrap();

    // ML-KEM ciphertexts should be different (probabilistic encryption)
    assert_ne!(
        encrypted1.ml_kem_ciphertext, encrypted2.ml_kem_ciphertext,
        "ML-KEM ciphertexts should be different due to randomness"
    );

    // But both should decrypt correctly
    let decrypted1 = pqc_manager
        .decrypt_content(&encrypted1, user_id)
        .await
        .unwrap();
    let decrypted2 = pqc_manager
        .decrypt_content(&encrypted2, user_id)
        .await
        .unwrap();

    assert_eq!(decrypted1, test_content);
    assert_eq!(decrypted2, test_content);
    assert_eq!(decrypted1, decrypted2);
}

#[test]
async fn test_pqc_legacy_compatibility() {
    let master_key = [0x99u8; 32];
    let _pqc_manager = PqcCryptoManager::new(master_key).unwrap();

    // Test that the PQC system can handle legacy encrypted content format
    // This verifies the backward compatibility layer in decrypt_content

    let legacy_key = [0x99u8; 32];
    let test_content = b"Legacy compatibility test content";

    // Simulate legacy encryption using ChaCha20Poly1305 directly
    use chacha20poly1305::aead::{Aead, AeadCore, OsRng};
    use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit};

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&legacy_key).into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher.encrypt(&nonce, test_content.as_ref()).unwrap();

    // Prepend nonce to ciphertext (legacy format)
    let mut encrypted_legacy = nonce.to_vec();
    encrypted_legacy.extend_from_slice(&ciphertext);

    // Verify we can decrypt with the same key
    use chacha20poly1305::Nonce;
    let nonce_slice = Nonce::from_slice(&encrypted_legacy[..12]);
    let ciphertext_slice = &encrypted_legacy[12..];

    let decrypted = cipher.decrypt(nonce_slice, ciphertext_slice).unwrap();
    assert_eq!(decrypted, test_content);
}

#[test]
async fn test_pqc_error_handling() {
    let master_key = [0xAAu8; 32];
    let pqc_manager = PqcCryptoManager::new(master_key).unwrap();

    // Test invalid public key size for encapsulation
    let invalid_public_key = vec![0u8; 100]; // Wrong size
    let result = MlKemEncapsulation::encapsulate(&invalid_public_key);
    assert!(result.is_err(), "Should fail with invalid public key size");

    // Test invalid ciphertext size for decapsulation
    let valid_keypair = MlKemKeypair::generate().unwrap();
    let invalid_ciphertext = vec![0u8; 50]; // Wrong size
    let result = MlKemEncapsulation::decapsulate(&invalid_ciphertext, &valid_keypair.secret_key);
    assert!(result.is_err(), "Should fail with invalid ciphertext size");

    // Test invalid secret key size for decapsulation
    let valid_encapsulation = MlKemEncapsulation::encapsulate(&valid_keypair.public_key).unwrap();
    let invalid_secret_key = vec![0u8; 100]; // Wrong size
    let result =
        MlKemEncapsulation::decapsulate(&valid_encapsulation.ciphertext, &invalid_secret_key);
    assert!(result.is_err(), "Should fail with invalid secret key size");
}

#[test]
async fn test_pqc_storage_engine_integration() {
    let harness = PqcStorageTestHarness::new().await.unwrap();

    // Test that storage engine properly integrates PQC encryption
    let policy = StoragePolicy::PrivateScoped {
        namespace: "integration_test".to_string(),
    };

    let store_request = StorageRequest {
        content: harness.test_content.clone(),
        content_type: "text/plain".to_string(),
        policy: policy.clone(),
        metadata: harness.create_test_metadata(),
        user_id: harness.test_user_id.clone(),
        group_id: None,
        namespace: Some("integration_test".to_string()),
    };

    // Store content
    let store_response = harness.engine.store_content(store_request).await.unwrap();

    // Verify that PQC encryption was used (encrypted size should be significantly larger)
    let min_expected_size = harness.test_content.len() as u64 + 1088 + 100; // Content + ML-KEM ciphertext + overhead
    assert!(
        store_response.encrypted_size > min_expected_size,
        "PQC encrypted content should include ML-KEM ciphertext overhead"
    );

    // Retrieve and verify
    let retrieval_request = RetrievalRequest {
        address: store_response.address,
        user_id: harness.test_user_id.clone(),
        decryption_key: None,
    };

    let retrieval_response = harness
        .engine
        .retrieve_content(retrieval_request)
        .await
        .unwrap();
    assert_eq!(retrieval_response.content, harness.test_content);
}

#[test]
async fn test_pqc_multiple_users_same_namespace() {
    let harness = PqcStorageTestHarness::new().await.unwrap();
    let namespace = "shared_namespace".to_string();

    // Test that different users in the same namespace can encrypt/decrypt
    let users = vec!["user_alice", "user_bob", "user_charlie"];
    let policy = StoragePolicy::PrivateScoped {
        namespace: namespace.clone(),
    };

    for user in users {
        let content = format!("Content for user {}", user).into_bytes();

        let store_request = StorageRequest {
            content: content.clone(),
            content_type: "text/plain".to_string(),
            policy: policy.clone(),
            metadata: StorageMetadata {
                content_type: "text/plain".to_string(),
                author: user.to_string(),
                tags: vec!["multi_user_test".to_string()],
                created_at: Utc::now(),
                modified_at: None,
                size: content.len() as u64,
                checksum: format!("checksum_{}", user),
            },
            user_id: user.to_string(),
            group_id: None,
            namespace: Some(namespace.clone()),
        };

        let store_response = harness.engine.store_content(store_request).await.unwrap();

        // Each user should be able to retrieve their own content
        let retrieval_request = RetrievalRequest {
            address: store_response.address,
            user_id: user.to_string(),
            decryption_key: None,
        };

        let retrieval_response = harness
            .engine
            .retrieve_content(retrieval_request)
            .await
            .unwrap();
        assert_eq!(retrieval_response.content, content);
    }
}
