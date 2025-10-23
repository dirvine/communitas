//! Key Management with PBKDF2 and Platform Keyring Integration
//!
//! Implements secure key derivation using PBKDF2 with 100,000 iterations
//! as specified in DESIGN.md, with optional platform keyring support.

use anyhow::Result;
use base64::Engine;
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit},
};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use zeroize::Zeroizing;

/// Key manager for deriving and managing encryption keys
pub struct KeyManager {
    iterations: u32,
    use_keyring: bool,
    keyring_service: String,
}

impl KeyManager {
    /// Create a new key manager with specified PBKDF2 iterations
    pub async fn new(iterations: u32, use_keyring: bool) -> Result<Self> {
        Ok(Self {
            iterations,
            use_keyring,
            keyring_service: "com.saorsalabs.communitas".to_string(),
        })
    }

    /// Derive an encryption key from password using PBKDF2
    pub async fn derive_key(&self, password: &str, salt: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
        let password = password.to_string();
        let salt = salt.to_vec();
        let iterations = self.iterations;

        tokio::task::spawn_blocking(move || {
            let mut key = Zeroizing::new(vec![0u8; 32]); // ChaCha20-Poly1305 uses 256-bit keys

            // PBKDF2 with SHA-256 and configured iterations (100,000 as per DESIGN.md)
            pbkdf2_hmac::<Sha256>(password.as_bytes(), &salt, iterations, &mut key);

            Ok(key) as Result<Zeroizing<Vec<u8>>>
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to derive key: {}", e))?
    }

    /// Hash password for lookup (different from key derivation)
    pub async fn hash_password(&self, password: &str) -> Result<Vec<u8>> {
        let password = password.to_string();

        tokio::task::spawn_blocking(move || {
            use blake3::Hasher;

            let mut hasher = Hasher::new();
            hasher.update(b"communitas:password:v1:");
            hasher.update(password.as_bytes());

            Ok(hasher.finalize().as_bytes().to_vec())
        })
        .await
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
    }

    /// Store key in platform keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    pub async fn store_in_keyring(&self, four_words: &str, key: &[u8]) -> Result<()> {
        if !self.use_keyring {
            return Ok(());
        }

        use keyring::Entry;

        let entry = Entry::new(&self.keyring_service, four_words)
            .map_err(|e| anyhow::anyhow!("Failed to create keyring entry: {}", e))?;

        // Convert key to base64 for storage
        let key_b64 = base64::engine::general_purpose::STANDARD.encode(key);

        entry
            .set_password(&key_b64)
            .map_err(|e| anyhow::anyhow!("Failed to store key in keyring: {}", e))?;

        Ok(())
    }

    /// Retrieve key from platform keyring
    pub async fn get_from_keyring(&self, four_words: &str) -> Result<Zeroizing<Vec<u8>>> {
        if !self.use_keyring {
            return Err(anyhow::anyhow!("Keyring not enabled"));
        }

        use keyring::Entry;

        let entry = Entry::new(&self.keyring_service, four_words)
            .map_err(|e| anyhow::anyhow!("Failed to access keyring entry: {}", e))?;

        let key_b64 = entry
            .get_password()
            .map_err(|e| anyhow::anyhow!("Failed to retrieve key from keyring: {}", e))?;

        let key = Zeroizing::new(
            base64::engine::general_purpose::STANDARD
                .decode(&key_b64)
                .map_err(|e| anyhow::anyhow!("Failed to decode key from keyring: {}", e))?,
        );

        Ok(key)
    }

    /// Delete key from platform keyring
    pub async fn delete_from_keyring(&self, four_words: &str) -> Result<()> {
        if !self.use_keyring {
            return Ok(());
        }

        use keyring::Entry;

        let entry = Entry::new(&self.keyring_service, four_words)
            .map_err(|e| anyhow::anyhow!("Failed to access keyring entry: {}", e))?;

        entry
            .delete_credential()
            .map_err(|e| anyhow::anyhow!("Failed to delete key from keyring: {}", e))?;

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
        use rand::SeedableRng;
        ChaCha20Poly1305::generate_nonce(&mut rand::rngs::StdRng::from_entropy())
    }

    /// Encrypt data using ChaCha20-Poly1305
    pub fn encrypt(&self, key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher_key = Key::from(
            *<&[u8; 32]>::try_from(key).map_err(|_| anyhow::anyhow!("Invalid key length"))?,
        );
        let cipher = ChaCha20Poly1305::new(&cipher_key);
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
        let nonce = Nonce::from(
            *<&[u8; 12]>::try_from(nonce_bytes)
                .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?,
        );

        let cipher_key = Key::from(
            *<&[u8; 32]>::try_from(key).map_err(|_| anyhow::anyhow!("Invalid key length"))?,
        );
        let cipher = ChaCha20Poly1305::new(&cipher_key);

        let plaintext = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(Zeroizing::new(plaintext))
    }

    /// Generate a secure random salt
    pub fn generate_salt() -> Vec<u8> {
        use rand::{Rng, SeedableRng};
        let mut salt = vec![0u8; 32];
        rand::rngs::StdRng::from_entropy().fill(&mut salt[..]);
        salt
    }
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
        let key3 = manager
            .derive_key("different_password", &salt)
            .await
            .unwrap();
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
        let subkey2 = manager
            .derive_subkey(&master_key, "metadata")
            .await
            .unwrap();

        // Different contexts should produce different subkeys
        assert_ne!(subkey1, subkey2);

        // Same context should produce same subkey
        let subkey3 = manager.derive_subkey(&master_key, "files").await.unwrap();
        assert_eq!(subkey1, subkey3);
    }
}
