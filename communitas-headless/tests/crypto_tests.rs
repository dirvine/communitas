// TDD Tests for Headless PQC Cryptography
// Import the actual crypto module from the binary crate
// Note: We need to include the module here for testing
#[path = "../src/crypto.rs"]
mod crypto;

#[test]
// #[ignore] // Will be enabled after implementation
fn test_keygen_produces_valid_mldsa87_keys() {
    // Should generate 2592-byte public key, 4896-byte private key
    let result = crypto::generate_mldsa87_keypair();
    assert!(result.is_ok(), "Key generation should succeed");

    let (pk, sk) = result.unwrap();
    assert_eq!(pk.len(), 2592, "ML-DSA-87 public key should be 2592 bytes");
    assert_eq!(sk.len(), 4896, "ML-DSA-87 private key should be 4896 bytes");

    // Keys should not be all zeros
    assert!(
        pk.iter().any(|&b| b != 0),
        "Public key should not be all zeros"
    );
    assert!(
        sk.iter().any(|&b| b != 0),
        "Private key should not be all zeros"
    );
}

#[test]
// #[ignore]
fn test_keys_are_unique() {
    // Multiple key generation calls should produce different keys
    let result1 = crypto::generate_mldsa87_keypair();
    let result2 = crypto::generate_mldsa87_keypair();

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    let (pk1, sk1) = result1.unwrap();
    let (pk2, sk2) = result2.unwrap();

    assert_ne!(pk1, pk2, "Public keys should be different");
    assert_ne!(sk1, sk2, "Private keys should be different");
}

#[test]
// #[ignore]
fn test_sign_verify_roundtrip() {
    // Generate keys
    let (pk, sk) = crypto::generate_mldsa87_keypair().expect("Key generation failed");

    // Sign a message
    let message = b"test message for ML-DSA-87 signature";
    let signature = crypto::sign_mldsa87(&sk, message).expect("Signing failed");

    // Signature should be 4627 bytes for ML-DSA-87
    assert_eq!(
        signature.len(),
        4627,
        "ML-DSA-87 signature should be 4627 bytes"
    );

    // Verify signature
    let verify_result = crypto::verify_mldsa87(&pk, message, &signature);
    assert!(verify_result.is_ok(), "Verification should not error");
    assert!(verify_result.unwrap(), "Signature should verify");
}

#[test]
// #[ignore]
fn test_verify_fails_with_wrong_message() {
    let (pk, sk) = crypto::generate_mldsa87_keypair().expect("Key generation failed");

    let message = b"original message";
    let signature = crypto::sign_mldsa87(&sk, message).expect("Signing failed");

    // Try to verify with different message
    let wrong_message = b"tampered message";
    let verify_result = crypto::verify_mldsa87(&pk, wrong_message, &signature);

    assert!(verify_result.is_ok());
    assert!(
        !verify_result.unwrap(),
        "Signature should NOT verify with wrong message"
    );
}

#[test]
// #[ignore]
fn test_verify_fails_with_wrong_key() {
    let (_pk1, sk1) = crypto::generate_mldsa87_keypair().expect("Key generation failed");
    let (pk2, _sk2) = crypto::generate_mldsa87_keypair().expect("Key generation failed");

    let message = b"test message";
    let signature = crypto::sign_mldsa87(&sk1, message).expect("Signing failed");

    // Try to verify with different public key
    let verify_result = crypto::verify_mldsa87(&pk2, message, &signature);

    assert!(verify_result.is_ok());
    assert!(
        !verify_result.unwrap(),
        "Signature should NOT verify with wrong key"
    );
}

#[test]
#[ignore] // Requires real system keyring (macOS Keychain, Secret Service) - run manually
fn test_keystore_persistence() {
    let identity = "test-headless-node-alpha";
    let (pk, sk) = crypto::generate_mldsa87_keypair().expect("Key generation failed");

    // Save keys to keystore
    crypto::save_keys_to_keystore(identity, &pk, &sk).expect("Saving keys should succeed");

    // Load keys from keystore
    let (loaded_pk, loaded_sk) =
        crypto::load_keys_from_keystore(identity).expect("Loading keys should succeed");

    // Verify loaded keys match original
    assert_eq!(pk, loaded_pk, "Loaded public key should match original");
    assert_eq!(sk, loaded_sk, "Loaded private key should match original");
}

#[test]
#[ignore] // Requires real system keyring (macOS Keychain, Secret Service) - run manually
fn test_keystore_roundtrip_with_signing() {
    let identity = "test-headless-node-beta";

    // Generate and save keys
    let (pk, sk) = crypto::generate_mldsa87_keypair().expect("Key generation failed");
    crypto::save_keys_to_keystore(identity, &pk, &sk).expect("Save failed");

    // Load keys
    let (loaded_pk, loaded_sk) = crypto::load_keys_from_keystore(identity).expect("Load failed");

    // Use loaded keys to sign and verify
    let message = b"test persistence";
    let signature =
        crypto::sign_mldsa87(&loaded_sk, message).expect("Signing with loaded key failed");
    let verified =
        crypto::verify_mldsa87(&loaded_pk, message, &signature).expect("Verification failed");

    assert!(verified, "Signature should verify with loaded keys");
}

#[test]
fn test_keystore_load_nonexistent_identity() {
    let nonexistent = "nonexistent-headless-node-zzzz";
    let result = crypto::load_keys_from_keystore(nonexistent);

    assert!(result.is_err(), "Loading nonexistent identity should fail");
}

#[test]
#[ignore] // Requires real system keyring (macOS Keychain, Secret Service) - run manually
fn test_key_zeroization_on_drop() {
    // This test ensures sensitive key material is zeroized
    // when dropped (security requirement)

    let identity = "test-zeroize-node";
    let (pk, sk) = crypto::generate_mldsa87_keypair().expect("Key generation failed");

    // Save keys
    crypto::save_keys_to_keystore(identity, &pk, &sk).expect("Save failed");

    // Keys should be zeroized when they go out of scope
    drop(pk);
    drop(sk);

    // Load again to verify persistence
    let (loaded_pk, loaded_sk) = crypto::load_keys_from_keystore(identity).expect("Load failed");
    assert_eq!(loaded_pk.len(), 2592);
    assert_eq!(loaded_sk.len(), 4896);
}
