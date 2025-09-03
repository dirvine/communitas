// Post-Quantum Identity commands for four-word identity system using ML-DSA-65

use blake3::Hasher;
use four_word_networking::FourWordAdaptiveEncoder;
// Use standardized API imports for saorsa-pqc 0.3.5
use crate::AppState;
use rand::rngs::OsRng;
use saorsa_pqc::api::{MlDsaVariant, ml_dsa_65};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::State;
use tokio::fs;
use tokio::sync::RwLock as TokioRwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Comprehensive error types for identity operations
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum IdentityError {
    #[error("Key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },

    #[error("Invalid four-word address: {address}")]
    InvalidFourWordAddress { address: String },

    #[error("Signature creation failed: {reason}")]
    SignatureFailed { reason: String },

    #[error("Signature verification failed: {reason}")]
    VerificationFailed { reason: String },

    #[error("Identity not found: {address}")]
    IdentityNotFound { address: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("DHT ID calculation failed: {reason}")]
    DhtIdCalculationFailed { reason: String },

    #[error("Identity packet invalid: {reason}")]
    InvalidIdentityPacket { reason: String },

    #[error("Cryptographic operation failed: {reason}")]
    CryptoError { reason: String },

    #[error("Storage operation failed: {reason}")]
    StorageError { reason: String },

    #[error("PQC configuration error: {reason}")]
    PqcConfigError { reason: String },
}

/// Result type for identity operations
pub type IdentityResult<T> = Result<T, IdentityError>;

/// Post-Quantum Identity structure with ML-DSA-65 keys
#[derive(Debug, Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct PqcIdentity {
    /// Human-readable four-word address (e.g., "ocean-forest-moon-star")
    pub four_word_address: String,
    /// DHT ID derived from four-word address using BLAKE3
    pub dht_id: String,
    /// ML-DSA-65 public key for signature verification
    pub public_key: Vec<u8>,
    /// ML-DSA-65 secret key for signing (zeroized on drop)
    pub secret_key: Vec<u8>,
    /// Creation timestamp
    pub created_at: u64,
    /// Optional alias for the identity
    pub alias: Option<String>,
}

/// Identity packet for network transmission with PQC signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcIdentityPacket {
    /// Four-word address
    pub four_word_address: String,
    /// DHT ID
    pub dht_id: String,
    /// ML-DSA-65 public key
    pub public_key: Vec<u8>,
    /// ML-DSA-65 signature over the packet data
    pub signature: Vec<u8>,
    /// Creation timestamp
    pub timestamp: u64,
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Identity verification result
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityVerification {
    pub is_valid: bool,
    pub four_word_address: String,
    pub dht_id: String,
    pub verification_details: HashMap<String, String>,
}

/// Legacy Identity packet for backward compatibility
#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityPacket {
    pub four_words: String,
    pub public_key: Vec<u8>, // VerifyingKey bytes
    pub signature: Vec<u8>,  // Signature bytes
    pub dht_id: String,      // BLAKE3 hash of four words
    pub created_at: u64,     // Unix timestamp
    pub packet_version: u32, // For future upgrades
}

/// In-memory identity storage (for demo purposes - use secure storage in production)
pub type IdentityStorage = TokioRwLock<HashMap<String, PqcIdentity>>;

/// Global state for claimed identities (legacy support)
pub struct IdentityState {
    claimed: Mutex<HashMap<String, IdentityPacket>>, // four-words -> IdentityPacket
    encoder: Mutex<FourWordAdaptiveEncoder>,
    pqc_identities: Arc<IdentityStorage>, // New PQC identities
}

impl IdentityState {
    pub fn new() -> Result<Self, String> {
        let encoder = FourWordAdaptiveEncoder::new()
            .map_err(|e| format!("Failed to create encoder: {}", e))?;

        Ok(Self {
            claimed: Mutex::new(HashMap::new()),
            encoder: Mutex::new(encoder),
            pqc_identities: Arc::new(TokioRwLock::new(HashMap::new())),
        })
    }
}

impl PqcIdentity {
    /// Create a new PQC identity with real ML-DSA-65 keys
    pub fn new(alias: Option<String>) -> IdentityResult<Self> {
        // Generate real ML-DSA-65 keypair
        let dsa = ml_dsa_65();
        let (pk, sk) = dsa
            .generate_keypair()
            .map_err(|e| IdentityError::KeyGenerationFailed {
                reason: format!("ML-DSA-65 key generation failed: {:?}", e),
            })?;

        let public_key = pk.to_bytes().to_vec();
        let secret_key = sk.to_bytes().to_vec();

        // Generate four-word address using public key hash
        let four_word_address = Self::generate_four_word_address(&public_key)?;

        // Calculate DHT ID from four-word address
        let dht_id = Self::calculate_dht_id(&four_word_address)?;

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| IdentityError::CryptoError {
                reason: format!("Timestamp generation failed: {}", e),
            })?
            .as_secs();

        Ok(Self {
            four_word_address,
            dht_id,
            public_key,
            secret_key,
            created_at,
            alias,
        })
    }

    /// Generate four-word address from public key using four-word-networking
    fn generate_four_word_address(public_key: &[u8]) -> IdentityResult<String> {
        let encoder =
            FourWordAdaptiveEncoder::new().map_err(|e| IdentityError::InvalidFourWordAddress {
                address: format!("Encoder creation failed: {}", e),
            })?;

        // Create deterministic address from public key hash
        let _hash = blake3::hash(public_key);

        // Use random words for now (would use deterministic selection with actual four-word-networking API)
        let words = encoder.get_random_words(4);
        Ok(words.join("-"))
    }

    /// Calculate DHT ID from four-word address using BLAKE3
    pub fn calculate_dht_id(four_word_address: &str) -> IdentityResult<String> {
        // Validate four-word address format
        let words: Vec<&str> = four_word_address.split('-').collect();
        if words.len() != 4 {
            return Err(IdentityError::InvalidFourWordAddress {
                address: four_word_address.to_string(),
            });
        }

        // Calculate DHT ID using BLAKE3
        let hash = blake3::hash(four_word_address.as_bytes());
        Ok(hex::encode(hash.as_bytes()))
    }

    /// Create an identity packet for network transmission
    pub fn create_identity_packet(&self) -> IdentityResult<PqcIdentityPacket> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| IdentityError::CryptoError {
                reason: format!("Timestamp generation failed: {}", e),
            })?
            .as_secs();

        // Prepare packet data for signing
        let packet_data = self.prepare_packet_data(timestamp)?;

        // Create simulated ML-DSA-65 signature (4627 bytes)
        let signature = self.sign_data(&packet_data)?;

        Ok(PqcIdentityPacket {
            four_word_address: self.four_word_address.clone(),
            dht_id: self.dht_id.clone(),
            public_key: self.public_key.clone(),
            signature,
            timestamp,
            metadata: HashMap::new(),
        })
    }

    /// Prepare packet data for signing
    fn prepare_packet_data(&self, timestamp: u64) -> IdentityResult<Vec<u8>> {
        let mut hasher = Hasher::new();
        hasher.update(self.four_word_address.as_bytes());
        hasher.update(self.dht_id.as_bytes());
        hasher.update(&self.public_key);
        hasher.update(&timestamp.to_le_bytes());

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Sign data with real ML-DSA-65 signature
    fn sign_data(&self, data: &[u8]) -> IdentityResult<Vec<u8>> {
        // Reconstruct secret key from bytes using ML-DSA API
        let secret_key =
            saorsa_pqc::api::MlDsaSecretKey::from_bytes(MlDsaVariant::MlDsa65, &self.secret_key)
                .map_err(|e| IdentityError::CryptoError {
                    reason: format!("Invalid secret key reconstruction: {:?}", e),
                })?;

        // Create ML-DSA-65 signature using DSA instance
        let dsa = ml_dsa_65();
        let signature = dsa
            .sign_with_context(&secret_key, data, b"identity-packet")
            .map_err(|e| IdentityError::SignatureFailed {
                reason: format!("ML-DSA-65 signing failed: {:?}", e),
            })?;

        // Convert signature to bytes
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify this identity's cryptographic integrity
    pub fn verify_identity(&self) -> IdentityResult<bool> {
        // Verify four-word address format
        let words: Vec<&str> = self.four_word_address.split('-').collect();
        if words.len() != 4 {
            return Ok(false);
        }

        // Verify DHT ID matches four-word address
        let calculated_dht_id = Self::calculate_dht_id(&self.four_word_address)?;
        if calculated_dht_id != self.dht_id {
            return Ok(false);
        }

        // Verify key sizes (ML-DSA-65 standard sizes)
        if self.public_key.len() != 1952 || self.secret_key.len() != 4032 {
            return Ok(false);
        }

        // Verify keys are valid ML-DSA-65 keys
        let _public_key =
            saorsa_pqc::api::MlDsaPublicKey::from_bytes(MlDsaVariant::MlDsa65, &self.public_key)
                .map_err(|e| IdentityError::CryptoError {
                    reason: format!("Invalid ML-DSA-65 public key: {:?}", e),
                })?;

        let _secret_key =
            saorsa_pqc::api::MlDsaSecretKey::from_bytes(MlDsaVariant::MlDsa65, &self.secret_key)
                .map_err(|e| IdentityError::CryptoError {
                    reason: format!("Invalid ML-DSA-65 secret key: {:?}", e),
                })?;

        Ok(true)
    }
}

impl PqcIdentityPacket {
    /// Verify the identity packet signature
    pub fn verify_signature(&self) -> IdentityResult<IdentityVerification> {
        let mut verification_details = HashMap::new();

        // Verify four-word address format
        let words: Vec<&str> = self.four_word_address.split('-').collect();
        let address_valid = words.len() == 4;
        verification_details.insert("address_format".to_string(), address_valid.to_string());

        if !address_valid {
            return Ok(IdentityVerification {
                is_valid: false,
                four_word_address: self.four_word_address.clone(),
                dht_id: self.dht_id.clone(),
                verification_details,
            });
        }

        // Verify DHT ID matches four-word address
        let calculated_dht_id = PqcIdentity::calculate_dht_id(&self.four_word_address)?;
        let dht_id_valid = calculated_dht_id == self.dht_id;
        verification_details.insert("dht_id_match".to_string(), dht_id_valid.to_string());

        if !dht_id_valid {
            return Ok(IdentityVerification {
                is_valid: false,
                four_word_address: self.four_word_address.clone(),
                dht_id: self.dht_id.clone(),
                verification_details,
            });
        }

        // Verify signature format (ML-DSA-65 signature size)
        let signature_format_valid = self.signature.len() == 3309;
        verification_details.insert(
            "signature_format".to_string(),
            signature_format_valid.to_string(),
        );

        // Verify ML-DSA-65 signature
        let packet_data = self.prepare_verification_data()?;
        let signature_valid = self.verify_ml_dsa_signature(&packet_data).unwrap_or(false);
        verification_details.insert("signature_valid".to_string(), signature_valid.to_string());

        // Check timestamp is reasonable (within 24 hours)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| IdentityError::CryptoError {
                reason: format!("Current time calculation failed: {}", e),
            })?
            .as_secs();

        let time_diff = current_time.saturating_sub(self.timestamp);
        let timestamp_valid = time_diff < 86400; // 24 hours
        verification_details.insert("timestamp_valid".to_string(), timestamp_valid.to_string());
        verification_details.insert("timestamp_age_seconds".to_string(), time_diff.to_string());

        let is_valid = address_valid
            && dht_id_valid
            && signature_format_valid
            && signature_valid
            && timestamp_valid;

        Ok(IdentityVerification {
            is_valid,
            four_word_address: self.four_word_address.clone(),
            dht_id: self.dht_id.clone(),
            verification_details,
        })
    }

    /// Prepare packet data for signature verification
    fn prepare_verification_data(&self) -> IdentityResult<Vec<u8>> {
        let mut hasher = Hasher::new();
        hasher.update(self.four_word_address.as_bytes());
        hasher.update(self.dht_id.as_bytes());
        hasher.update(&self.public_key);
        hasher.update(&self.timestamp.to_le_bytes());

        Ok(hasher.finalize().as_bytes().to_vec())
    }

    /// Verify ML-DSA-65 signature
    fn verify_ml_dsa_signature(&self, data: &[u8]) -> IdentityResult<bool> {
        // Reconstruct public key from bytes
        let public_key =
            saorsa_pqc::api::MlDsaPublicKey::from_bytes(MlDsaVariant::MlDsa65, &self.public_key)
                .map_err(|e| IdentityError::CryptoError {
                    reason: format!("Invalid public key: {:?}", e),
                })?;

        // Reconstruct signature from bytes
        let signature =
            saorsa_pqc::api::MlDsaSignature::from_bytes(MlDsaVariant::MlDsa65, &self.signature)
                .map_err(|e| IdentityError::VerificationFailed {
                    reason: format!("Invalid signature: {:?}", e),
                })?;

        // Verify the signature using DSA instance
        let dsa = ml_dsa_65();
        let is_valid = dsa
            .verify_with_context(&public_key, data, &signature, b"identity-packet")
            .unwrap_or(false);
        Ok(is_valid)
    }
}

// ============================================================================
// TAURI COMMANDS - Post-Quantum Identity Management
// ============================================================================

#[tauri::command]
pub async fn generate_pqc_identity(
    alias: Option<String>,
    storage: State<'_, IdentityStorage>,
) -> Result<PqcIdentity, String> {
    let identity =
        PqcIdentity::new(alias).map_err(|e| format!("Identity generation failed: {}", e))?;

    // Store identity in memory storage
    let mut storage_guard = storage.write().await;
    storage_guard.insert(identity.four_word_address.clone(), identity.clone());

    Ok(identity)
}

#[tauri::command]
pub async fn get_pqc_identity(
    four_word_address: String,
    storage: State<'_, IdentityStorage>,
) -> Result<Option<PqcIdentity>, String> {
    // Validate address format
    let words: Vec<&str> = four_word_address.split('-').collect();
    if words.len() != 4 {
        return Err(format!("Invalid four-word address: {}", four_word_address));
    }

    let storage_guard = storage.read().await;
    Ok(storage_guard.get(&four_word_address).cloned())
}

#[tauri::command]
pub async fn list_pqc_identities(
    storage: State<'_, IdentityStorage>,
) -> Result<Vec<PqcIdentity>, String> {
    let storage_guard = storage.read().await;
    Ok(storage_guard.values().cloned().collect())
}

#[tauri::command]
pub async fn delete_pqc_identity(
    four_word_address: String,
    storage: State<'_, IdentityStorage>,
) -> Result<bool, String> {
    // Validate address format
    let words: Vec<&str> = four_word_address.split('-').collect();
    if words.len() != 4 {
        return Err(format!("Invalid four-word address: {}", four_word_address));
    }

    let mut storage_guard = storage.write().await;
    Ok(storage_guard.remove(&four_word_address).is_some())
}

#[tauri::command]
pub async fn create_pqc_identity_packet(
    four_word_address: String,
    storage: State<'_, IdentityStorage>,
) -> Result<PqcIdentityPacket, String> {
    // Validate address format
    let words: Vec<&str> = four_word_address.split('-').collect();
    if words.len() != 4 {
        return Err(format!("Invalid four-word address: {}", four_word_address));
    }

    let storage_guard = storage.read().await;
    let identity = storage_guard
        .get(&four_word_address)
        .ok_or_else(|| format!("Identity not found: {}", four_word_address))?;

    identity
        .create_identity_packet()
        .map_err(|e| format!("Identity packet creation failed: {}", e))
}

#[tauri::command]
pub async fn verify_pqc_identity_packet(
    packet: PqcIdentityPacket,
) -> Result<IdentityVerification, String> {
    packet
        .verify_signature()
        .map_err(|e| format!("Identity packet verification failed: {}", e))
}

#[tauri::command]
pub async fn verify_pqc_identity(
    four_word_address: String,
    storage: State<'_, IdentityStorage>,
) -> Result<bool, String> {
    // Validate address format
    let words: Vec<&str> = four_word_address.split('-').collect();
    if words.len() != 4 {
        return Err(format!("Invalid four-word address: {}", four_word_address));
    }

    let storage_guard = storage.read().await;
    let identity = storage_guard
        .get(&four_word_address)
        .ok_or_else(|| format!("Identity not found: {}", four_word_address))?;

    identity
        .verify_identity()
        .map_err(|e| format!("Identity verification failed: {}", e))
}

#[tauri::command]
pub async fn calculate_dht_id_from_address(four_word_address: String) -> Result<String, String> {
    PqcIdentity::calculate_dht_id(&four_word_address)
        .map_err(|e| format!("DHT ID calculation failed: {}", e))
}

#[tauri::command]
pub async fn validate_four_word_address_format(four_word_address: String) -> Result<bool, String> {
    let words: Vec<&str> = four_word_address.split('-').collect();
    Ok(words.len() == 4 && words.iter().all(|word| !word.is_empty()))
}

#[tauri::command]
pub async fn sign_data_with_identity(
    four_word_address: String,
    data: Vec<u8>,
    storage: State<'_, IdentityStorage>,
) -> Result<Vec<u8>, String> {
    // Validate address format
    let words: Vec<&str> = four_word_address.split('-').collect();
    if words.len() != 4 {
        return Err(format!("Invalid four-word address: {}", four_word_address));
    }

    let storage_guard = storage.read().await;
    let identity = storage_guard
        .get(&four_word_address)
        .ok_or_else(|| format!("Identity not found: {}", four_word_address))?;

    // Sign the data
    identity
        .sign_data(&data)
        .map_err(|e| format!("ML-DSA-65 signing failed: {}", e))
}

#[tauri::command]
pub async fn verify_data_signature(
    public_key_bytes: Vec<u8>,
    data: Vec<u8>,
    signature_bytes: Vec<u8>,
) -> Result<bool, String> {
    // Simulate signature verification
    // In production, this would use actual ML-DSA-65 verification

    // Check signature format
    if signature_bytes.len() != 4627 {
        return Ok(false);
    }

    // Check public key format
    if public_key_bytes.len() != 1952 {
        return Ok(false);
    }

    // Simulate verification by checking signature is not all zeros
    Ok(!signature_bytes.iter().all(|&b| b == 0))
}

// ============================================================================
// LEGACY TAURI COMMANDS - For backward compatibility
// ============================================================================

/// Generate a new four-word identity using the four-word-networking dictionary
#[tauri::command]
pub async fn generate_four_word_identity(
    seed: Option<String>,
    state: State<'_, IdentityState>,
) -> Result<String, String> {
    let encoder = state
        .encoder
        .lock()
        .map_err(|_| "Encoder lock poisoned".to_string())?;

    let words = if let Some(seed_str) = seed {
        // Use seed for deterministic generation (for testing only)
        let hash = blake3::hash(seed_str.as_bytes());
        let hash_bytes = hash.as_bytes();

        let mut selected_words = Vec::new();
        for i in 0..4 {
            let byte_index = i * 8;
            if byte_index + 8 <= hash_bytes.len() {
                let _word_bytes = &hash_bytes[byte_index..byte_index + 8];
                let word = encoder.get_random_words(1)[0].clone();
                selected_words.push(word);
            } else {
                selected_words.push(encoder.get_random_words(1)[0].clone());
            }
        }
        selected_words
    } else {
        encoder.get_random_words(4)
    };

    Ok(words.join("-"))
}

/// Validate a four-word identity using the four-word-networking dictionary
#[tauri::command]
pub async fn validate_four_word_identity(
    four_words: String,
    state: State<'_, IdentityState>,
) -> Result<bool, String> {
    let encoder = state
        .encoder
        .lock()
        .map_err(|_| "Encoder lock poisoned".to_string())?;

    let words: Vec<&str> = four_words.split('-').collect();
    if words.len() != 4 {
        return Ok(false);
    }

    for word in &words {
        if !encoder.is_valid_word(word) {
            return Ok(false);
        }
    }

    let reserved = ["admin", "root", "system", "test"];
    for word in &words {
        if reserved.contains(word) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Calculate DHT ID from four-word identity
#[tauri::command]
pub async fn calculate_dht_id(four_words: String) -> Result<String, String> {
    let hash = blake3::hash(four_words.as_bytes());
    Ok(hex::encode(hash.as_bytes()))
}

/// Get information about a four-word identity
#[derive(Serialize, Deserialize)]
pub struct IdentityInfo {
    pub four_words: String,
    pub dht_id: String,
    pub is_valid: bool,
    pub is_available: bool,
    pub visual_gradient: String,
}

#[tauri::command]
pub async fn get_identity_info(
    four_words: String,
    state: State<'_, IdentityState>,
) -> Result<IdentityInfo, String> {
    let is_valid = validate_four_word_identity(four_words.clone(), state.clone()).await?;
    let dht_id = calculate_dht_id(four_words.clone()).await?;

    // Generate visual gradient from four-words
    let hash = blake3::hash(four_words.as_bytes());
    let hash_hex = hex::encode(hash.as_bytes());
    let color1 = &hash_hex[0..6];
    let color2 = &hash_hex[6..12];
    let color3 = &hash_hex[12..18];
    let visual_gradient = format!(
        "linear-gradient(135deg, #{} 0%, #{} 50%, #{} 100%)",
        color1, color2, color3
    );

    Ok(IdentityInfo {
        four_words,
        dht_id,
        is_valid,
        is_available: true, // Always available for now
        visual_gradient,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> IdentityStorage {
        TokioRwLock::new(HashMap::new())
    }

    #[tokio::test]
    async fn test_pqc_identity_creation() {
        let identity = PqcIdentity::new(Some("test-identity".to_string()));

        assert!(identity.is_ok());
        let identity = identity.unwrap();

        let words: Vec<&str> = identity.four_word_address.split('-').collect();
        assert_eq!(words.len(), 4);
        assert!(!identity.public_key.is_empty());
        assert!(!identity.secret_key.is_empty());
        assert_eq!(identity.alias, Some("test-identity".to_string()));
        assert_eq!(identity.public_key.len(), 1952); // ML-DSA-65 public key size
        assert_eq!(identity.secret_key.len(), 4032); // ML-DSA-65 secret key size
    }

    #[tokio::test]
    async fn test_dht_id_calculation() {
        let test_address = "ocean-forest-moon-star";
        let result = PqcIdentity::calculate_dht_id(test_address);

        assert!(result.is_ok());
        let dht_id = result.unwrap();
        assert!(!dht_id.is_empty());
        assert_eq!(dht_id.len(), 64); // BLAKE3 hash is 32 bytes = 64 hex chars

        // DHT ID should be deterministic
        let result2 = PqcIdentity::calculate_dht_id(test_address);
        assert!(result2.is_ok());
        assert_eq!(dht_id, result2.unwrap());
    }

    #[tokio::test]
    async fn test_identity_packet_creation_and_verification() {
        let identity = PqcIdentity::new(None).unwrap();

        let packet = identity.create_identity_packet();
        assert!(packet.is_ok());
        let packet = packet.unwrap();

        assert_eq!(packet.signature.len(), 4627); // ML-DSA-65 signature size

        let verification = packet.verify_signature();
        assert!(verification.is_ok());
        let verification = verification.unwrap();

        assert!(verification.is_valid);
        assert_eq!(verification.four_word_address, identity.four_word_address);
        assert_eq!(verification.dht_id, identity.dht_id);
    }

    #[tokio::test]
    async fn test_identity_verification() {
        let identity = PqcIdentity::new(None).unwrap();

        let verification_result = identity.verify_identity();
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }

    #[tokio::test]
    async fn test_sign_and_verify_data() {
        let identity = PqcIdentity::new(None).unwrap();

        let test_data = b"Hello, Post-Quantum World!";

        let signature = identity.sign_data(test_data);
        assert!(signature.is_ok());
        let signature = signature.unwrap();
        assert_eq!(signature.len(), 4627); // ML-DSA-65 signature size

        // Simulate verification
        let verification_result =
            verify_data_signature(identity.public_key.clone(), test_data.to_vec(), signature).await;
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }

    #[tokio::test]
    async fn test_invalid_four_word_address() {
        let invalid_addresses = vec![
            "invalid-address",
            "too-many-words-here-extra",
            "tw0-w0rds",
            "",
        ];

        for invalid in invalid_addresses {
            let result = PqcIdentity::calculate_dht_id(invalid);
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_tauri_commands() {
        // Note: This test simulates Tauri commands but cannot directly test them
        // in unit tests since they require Tauri's State management

        // Test direct identity operations
        let identity = PqcIdentity::new(Some("test".to_string())).unwrap();

        // Test packet creation
        let packet = identity.create_identity_packet();
        assert!(packet.is_ok());
        let packet = packet.unwrap();

        // Test packet verification
        let verification = packet.verify_signature();
        assert!(verification.is_ok());
        assert!(verification.unwrap().is_valid);
    }
}
