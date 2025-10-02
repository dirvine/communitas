//! Key Management with PBKDF2 and Platform Keyring Integration
//!
//! Implements secure key derivation using PBKDF2 with 100,000 iterations
//! as specified in DESIGN.md, with optional platform keyring support.

use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroizing;

/// Key manager for deriving and managing encryption keys
pub struct KeyManager {
    iterations: u32,
    use_keyring: bool,
    #[cfg(feature = "keyring")]
    keyring_service: String,
}

impl KeyManager {
    /// Create a new key manager with specified PBKDF2 iterations
    pub async fn new(iterations: u32, use_keyring: bool) -> Result<Self> {
        Ok(Self {
            iterations,
            use_keyring,
            #[cfg(feature = "keyring")]
            keyring_service: "com.p2pfoundation.communitas".to_string(),
        })
    }

    /// Derive an encryption key from password using PBKDF2
    pub async fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let mut key = Zeroizing::new(vec![0u8; 32]); // ChaCha20-Poly1305 uses 256-bit keys

        // PBKDF2 with SHA-256 and configured iterations (100,000 as per DESIGN.md)
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            self.iterations,
            &mut key,
        );

        Ok(key)
    }

    /// Hash password for lookup (different from key derivation)
    pub async fn hash_password(&self, password: &str) -> Result<Vec<u8>> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        hasher.update(b"communitas:password:v1:");
        hasher.update(password.as_bytes());

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Store key in platform keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    #[cfg(feature = "keyring")]
    pub async fn store_in_keyring(&self, four_words: &str, key: &[u8]) -> Result<()> {
        if !self.use_keyring {
            return Ok(());
        }

        use keyring::Entry;

        let entry = Entry::new(&self.keyring_service, four_words)
            .context("Failed to create keyring entry")?;

        // Convert key to base64 for storage
        let key_b64 = base64::encode(key);

        entry.set_password(&key_b64)
            .context("Failed to store key in keyring")?;

        Ok(())
    }

    /// Retrieve key from platform keyring
    #[cfg(feature = "keyring")]
    pub async fn get_from_keyring(&self, four_words: &str) -> Result<Zeroizing<Vec<u8>>> {
        if !self.use_keyring {
            return Err(anyhow::anyhow!("Keyring not enabled"));
        }

        use keyring::Entry;

        let entry = Entry::new(&self.keyring_service, four_words)
            .context("Failed to access keyring entry")?;

        let key_b64 = entry.get_password()
            .context("Failed to retrieve key from keyring")?;

        let key = Zeroizing::new(
            base64::decode(&key_b64)
                .context("Failed to decode key from keyring")?
        );

        Ok(key)
    }

    /// Delete key from platform keyring
    #[cfg(feature = "keyring")]
    pub async fn delete_from_keyring(&self, four_words: &str) -> Result<()> {
        if !self.use_keyring {
            return Ok(());
        }

        use keyring::Entry;

        let entry = Entry::new(&self.keyring_service, four_words)
            .context("Failed to access keyring entry")?;

        entry.delete_password()
            .context("Failed to delete key from keyring")?;

        Ok(())
    }

    // Stub implementations when keyring feature is disabled
    #[cfg(not(feature = "keyring"))]
    pub async fn store_in_keyring(&self, _four_words: &str, _key: &[u8]) -> Result<()> {
        Ok(())
    }

    #[cfg(not(feature = "keyring"))]
    pub async fn get_from_keyring(&self, _four_words: &str) -> Result<Zeroizing<Vec<u8>>> {
        Err(anyhow::anyhow!("Keyring feature not enabled"))
    }

    #[cfg(not(feature = "keyring"))]
    pub async fn delete_from_keyring(&self, _four_words: &str) -> Result<()> {
        Ok(())
    }

    /// Derive a subkey for specific purposes (e.g., file encryption vs metadata)
    pub async fn derive_subkey(&self, master_key: &[u8], context: &str) -> Result<Vec<u8>> {
        use blake3::derive_key;

        let context_string = format!("communitas:subkey:{}:v1", context);
        let subkey = derive_key(&context_string, master_key);

        Ok(subkey.to_vec())
    }

    /// Generate a random nonce for ChaCha20-Poly1305
    pub fn generate_nonce() -> Nonce {
        ChaCha20Poly1305::generate_nonce(&mut rand::thread_rng())
    }

    /// Encrypt data using ChaCha20-Poly1305
    pub fn encrypt(&self, key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher_key = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        let nonce = Self::generate_nonce();

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt data using ChaCha20-Poly1305
    pub fn decrypt(&self, key: &[u8], ciphertext_with_nonce: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        if ciphertext_with_nonce.len() < 12 {
            return Err(anyhow::anyhow!("Invalid ciphertext: too short"));
        }

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = ciphertext_with_nonce.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher_key = Key::from_slice(key);
        let cipher = ChaCha20Poly1305::new(cipher_key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(Zeroizing::new(plaintext))
    }

    /// Generate a secure random salt
    pub fn generate_salt() -> Vec<u8> {
        use rand::Rng;
        let mut salt = vec![0u8; 32];
        rand::thread_rng().fill(&mut salt[..]);
        salt
    }
}

/// Passkey support for WebAuthn/FIDO2 authentication
#[derive(Debug, Clone)]
pub struct PasskeyManager {
    relying_party: String,
}

impl PasskeyManager {
    pub fn new() -> Self {
        Self {
            relying_party: "communitas.life".to_string(),
        }
    }

    /// Register a new passkey for a four-word identity
    pub async fn register_passkey(&self, four_words: &str) -> Result<PasskeyCredential> {
        // In a real implementation, this would interact with WebAuthn API
        // For now, return a placeholder
        Ok(PasskeyCredential {
            credential_id: blake3::hash(four_words.as_bytes()).as_bytes().to_vec(),
            public_key: vec![0u8; 65], // Placeholder for P-256 public key
        })
    }

    /// Authenticate using passkey
    pub async fn authenticate_passkey(&self, credential: &PasskeyCredential) -> Result<bool> {
        // In a real implementation, this would verify the WebAuthn assertion
        // For now, return success
        Ok(true)
    }
}

#[derive(Debug, Clone)]
pub struct PasskeyCredential {
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_derivation() {
        let manager = KeyManager::new(1000, false).await.unwrap(); // Use fewer iterations for tests
        let salt = KeyManager::generate_salt();

        let key1 = manager.derive_key("test_password", &salt).await.unwrap();
        let key2 = manager.derive_key("test_password", &salt).await.unwrap();

        // Same password and salt should produce same key
        assert_eq!(&*key1, &*key2);

        // Different password should produce different key
        let key3 = manager.derive_key("different_password", &salt).await.unwrap();
        assert_ne!(&*key1, &*key3);

        // Different salt should produce different key
        let salt2 = KeyManager::generate_salt();
        let key4 = manager.derive_key("test_password", &salt2).await.unwrap();
        assert_ne!(&*key1, &*key4);
    }

    #[tokio::test]
    async fn test_encryption_decryption() {
        let manager = KeyManager::new(1000, false).await.unwrap();
        let key = vec![0u8; 32]; // Test key

        let plaintext = b"Hello, encrypted world!";

        // Encrypt
        let ciphertext = manager.encrypt(&key, plaintext).unwrap();
        assert_ne!(&ciphertext[12..], plaintext); // Should be encrypted

        // Decrypt
        let decrypted = manager.decrypt(&key, &ciphertext).unwrap();
        assert_eq!(&*decrypted, plaintext);

        // Wrong key should fail
        let wrong_key = vec![1u8; 32];
        assert!(manager.decrypt(&wrong_key, &ciphertext).is_err());
    }

    #[tokio::test]
    async fn test_password_hashing() {
        let manager = KeyManager::new(1000, false).await.unwrap();

        let hash1 = manager.hash_password("test_password").await.unwrap();
        let hash2 = manager.hash_password("test_password").await.unwrap();

        // Same password should produce same hash
        assert_eq!(hash1, hash2);

        // Different password should produce different hash
        let hash3 = manager.hash_password("different_password").await.unwrap();
        assert_ne!(hash1, hash3);
    }

    #[tokio::test]
    async fn test_subkey_derivation() {
        let manager = KeyManager::new(1000, false).await.unwrap();
        let master_key = vec![0u8; 32];

        let subkey1 = manager.derive_subkey(&master_key, "files").await.unwrap();
        let subkey2 = manager.derive_subkey(&master_key, "metadata").await.unwrap();

        // Different contexts should produce different subkeys
        assert_ne!(subkey1, subkey2);

        // Same context should produce same subkey
        let subkey3 = manager.derive_subkey(&master_key, "files").await.unwrap();
        assert_eq!(subkey1, subkey3);
    }
}