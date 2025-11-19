// Headless Node Cryptography Module
// Provides ML-DSA-87 post-quantum cryptography with secure keystore integration

use anyhow::{Context, Result};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use blake3::Hasher;
use fips204::traits::{SerDes, Signer, Verifier};
use keyring::Entry;
use rand::rngs::OsRng;
use saorsa_pqc::ml_dsa_87::{PrivateKey, PublicKey, try_keygen_with_rng};
use zeroize::Zeroize;

const KEYRING_SERVICE: &str = "communitas-headless";

/// Generate a new ML-DSA-87 keypair using cryptographically secure randomness
///
/// Returns (public_key_bytes, private_key_bytes)
/// - Public key: 2592 bytes
/// - Private key: 4627 bytes
pub fn generate_mldsa87_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    let mut rng = OsRng;

    let (public_key, private_key) = try_keygen_with_rng(&mut rng)
        .map_err(|e| anyhow::anyhow!("ML-DSA-87 key generation failed: {}", e))?;

    let pk_bytes = public_key.into_bytes().to_vec();
    let sk_bytes = private_key.into_bytes().to_vec();

    tracing::info!(
        "Generated ML-DSA-87 keypair: pk={} bytes, sk={} bytes",
        pk_bytes.len(),
        sk_bytes.len()
    );

    Ok((pk_bytes, sk_bytes))
}

/// Sign a message with ML-DSA-87 private key
///
/// Returns signature bytes (4595 bytes for ML-DSA-87)
pub fn sign_mldsa87(sk: &[u8], message: &[u8]) -> Result<Vec<u8>> {
    // Deserialize private key
    let sk_array: [u8; 4627] = sk.try_into().map_err(|_| {
        anyhow::anyhow!(
            "Invalid private key length: expected 4627 bytes, got {}",
            sk.len()
        )
    })?;

    let private_key = PrivateKey::try_from_bytes(sk_array)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ML-DSA-87 private key: {}", e))?;

    // Sign message
    let signature = private_key
        .try_sign(message, &[])
        .map_err(|e| anyhow::anyhow!("ML-DSA-87 signing failed: {}", e))?;

    Ok(signature.to_vec())
}

/// Verify an ML-DSA-87 signature
///
/// Returns true if signature is valid, false otherwise
pub fn verify_mldsa87(pk: &[u8], message: &[u8], signature: &[u8]) -> Result<bool> {
    // Deserialize public key
    let pk_array: [u8; 2592] = pk.try_into().map_err(|_| {
        anyhow::anyhow!(
            "Invalid public key length: expected 2592 bytes, got {}",
            pk.len()
        )
    })?;

    let public_key = PublicKey::try_from_bytes(pk_array)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ML-DSA-87 public key: {}", e))?;

    // Deserialize signature
    let sig_array: [u8; 4595] = signature.try_into().map_err(|_| {
        anyhow::anyhow!(
            "Invalid signature length: expected 4595 bytes, got {}",
            signature.len()
        )
    })?;

    let sig = saorsa_pqc::ml_dsa_87::Signature::try_from_bytes(sig_array)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ML-DSA-87 signature: {}", e))?;

    // Verify signature
    match public_key.try_verify(message, &sig, &[]) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Save ML-DSA-87 keys to platform keychain
///
/// Keys are stored with identity as the username, base64-encoded
pub fn save_keys_to_keystore(identity: &str, pk: &[u8], sk: &[u8]) -> Result<()> {
    // Create keyring entry for public key
    let pk_entry = Entry::new(KEYRING_SERVICE, &format!("mldsa_pk:{}", identity))
        .context("Failed to create keyring entry for public key")?;

    // Create keyring entry for private key
    let sk_entry = Entry::new(KEYRING_SERVICE, &format!("mldsa_sk:{}", identity))
        .context("Failed to create keyring entry for private key")?;

    // Encode keys as base64
    let pk_b64 = BASE64.encode(pk);
    let sk_b64 = BASE64.encode(sk);

    // Save to keyring
    pk_entry
        .set_password(&pk_b64)
        .context("Failed to save public key to keyring")?;

    sk_entry
        .set_password(&sk_b64)
        .context("Failed to save private key to keyring")?;

    tracing::info!("Saved ML-DSA-87 keys to keyring for identity: {}", identity);

    Ok(())
}

/// Load ML-DSA-87 keys from platform keychain
///
/// Returns (public_key_bytes, private_key_bytes)
pub fn load_keys_from_keystore(identity: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    // Create keyring entries
    let pk_entry = Entry::new(KEYRING_SERVICE, &format!("mldsa_pk:{}", identity))
        .context("Failed to create keyring entry for public key")?;

    let sk_entry = Entry::new(KEYRING_SERVICE, &format!("mldsa_sk:{}", identity))
        .context("Failed to create keyring entry for private key")?;

    // Load from keyring
    let mut pk_b64 = pk_entry.get_password().context(format!(
        "Failed to load public key for identity: {}",
        identity
    ))?;

    let mut sk_b64 = sk_entry.get_password().context(format!(
        "Failed to load private key for identity: {}",
        identity
    ))?;

    // Decode from base64
    let pk = BASE64
        .decode(&pk_b64)
        .context("Failed to decode public key from base64")?;

    let sk = BASE64
        .decode(&sk_b64)
        .context("Failed to decode private key from base64")?;

    // Zeroize base64 strings after decoding
    pk_b64.zeroize();
    sk_b64.zeroize();

    // Validate key lengths
    if pk.len() != 2592 {
        anyhow::bail!("Invalid public key length: expected 2592, got {}", pk.len());
    }
    if sk.len() != 4627 {
        anyhow::bail!(
            "Invalid private key length: expected 4627, got {}",
            sk.len()
        );
    }

    tracing::info!(
        "Loaded ML-DSA-87 keys from keyring for identity: {}",
        identity
    );

    Ok((pk, sk))
}

/// Generate an identity hash for keystore lookups
///
/// Uses BLAKE3 hash of the identity string to create a hex identifier
pub fn identity_hash(identity: &str) -> String {
    let mut hasher = Hasher::new();
    hasher.update(identity.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash.as_bytes())
}

/// Delete keys from keystore for a given identity
///
/// Useful for key rotation or cleanup
pub fn delete_keys_from_keystore(identity: &str) -> Result<()> {
    let pk_entry = Entry::new(KEYRING_SERVICE, &format!("mldsa_pk:{}", identity))
        .context("Failed to create keyring entry for public key")?;

    let sk_entry = Entry::new(KEYRING_SERVICE, &format!("mldsa_sk:{}", identity))
        .context("Failed to create keyring entry for private key")?;

    // Attempt to delete (ignore errors if not found)
    let _ = pk_entry.delete_credential();
    let _ = sk_entry.delete_credential();

    tracing::info!("Deleted keys from keyring for identity: {}", identity);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let result = generate_mldsa87_keypair();
        assert!(result.is_ok());

        let (pk, sk) = result.unwrap();
        assert_eq!(pk.len(), 2592);
        assert_eq!(sk.len(), 4627);
    }

    #[test]
    fn test_sign_verify() {
        let (pk, sk) = generate_mldsa87_keypair().unwrap();
        let message = b"test message";

        let signature = sign_mldsa87(&sk, message).unwrap();
        assert_eq!(signature.len(), 4595);

        let verified = verify_mldsa87(&pk, message, &signature).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_verify_fails_wrong_message() {
        let (pk, sk) = generate_mldsa87_keypair().unwrap();

        let message1 = b"original";
        let message2 = b"tampered";

        let signature = sign_mldsa87(&sk, message1).unwrap();
        let verified = verify_mldsa87(&pk, message2, &signature).unwrap();

        assert!(!verified);
    }
}
