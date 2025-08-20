/**
 * PQC Bridge - Frontend Integration for Post-Quantum Cryptography
 * Exposes ML-KEM-768 and ML-DSA-65 operations through Tauri commands
 */

use saorsa_pqc::api::{ml_kem_768, ml_dsa_65};
use saorsa_pqc::api::symmetric::{generate_key, generate_nonce};
use saorsa_pqc::api::ChaCha20Poly1305;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// PQC key pair for frontend operations
#[derive(Debug, Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct PqcKeyPair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub algorithm: String, // "ML-KEM-768" or "ML-DSA-65"
    pub created_at: u64,
}

/// PQC signature result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcSignature {
    pub signature: Vec<u8>,
    pub algorithm: String,
    pub context: String,
}

/// PQC encryption result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcEncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ml_kem_ciphertext: Vec<u8>,
    pub algorithm: String,
}

/// PQC verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcVerificationResult {
    pub is_valid: bool,
    pub algorithm: String,
    pub details: HashMap<String, String>,
}

/// PQC key encapsulation result
#[derive(Debug, Clone, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct PqcEncapsulationResult {
    pub ciphertext: Vec<u8>,
    pub shared_secret: Vec<u8>,
    pub algorithm: String,
}

/// In-memory storage for PQC operations
pub type PqcBridgeStorage = RwLock<HashMap<String, PqcKeyPair>>;

/// Generate ML-DSA-65 key pair
#[tauri::command]
pub async fn generate_ml_dsa_keypair() -> Result<PqcKeyPair, String> {
    // Generate ML-DSA-65 key pair using the convenience function
    let dsa = ml_dsa_65();
    let (public_key, secret_key) = dsa.generate_keypair()
        .map_err(|e| format!("ML-DSA-65 key generation failed: {:?}", e))?;
    
    // Keys are already in bytes format
    let public_key_bytes = public_key.to_vec();
    let secret_key_bytes = secret_key.to_vec();
    
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Timestamp generation failed: {}", e))?
        .as_secs();
    
    Ok(PqcKeyPair {
        public_key: public_key_bytes,
        secret_key: secret_key_bytes,
        algorithm: "ML-DSA-65".to_string(),
        created_at,
    })
}

/// Generate ML-KEM-768 key pair
#[tauri::command]
pub async fn generate_ml_kem_keypair() -> Result<PqcKeyPair, String> {
    // Generate ML-KEM-768 key pair using the convenience function
    let kem = ml_kem_768();
    let (public_key, secret_key) = kem.generate_keypair()
        .map_err(|e| format!("ML-KEM-768 key generation failed: {:?}", e))?;
    
    // Keys are already in bytes format  
    let public_key_bytes = public_key.to_vec();
    let secret_key_bytes = secret_key.to_vec();
    
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Timestamp generation failed: {}", e))?
        .as_secs();
    
    Ok(PqcKeyPair {
        public_key: public_key_bytes,
        secret_key: secret_key_bytes,
        algorithm: "ML-KEM-768".to_string(),
        created_at,
    })
}

/// Sign data with ML-DSA-65
#[tauri::command]
pub async fn ml_dsa_sign(
    data: Vec<u8>,
    secret_key_bytes: Vec<u8>,
    context: String,
) -> Result<PqcSignature, String> {
    // Sign the data using ML-DSA-65
    let dsa = ml_dsa_65();
    let signature = dsa.sign(&secret_key_bytes, &data)
        .map_err(|e| format!("ML-DSA-65 signing failed: {:?}", e))?;
    
    // Signature is already in bytes format
    let signature_bytes = signature.to_vec();
    
    Ok(PqcSignature {
        signature: signature_bytes,
        algorithm: "ML-DSA-65".to_string(),
        context,
    })
}

/// Verify ML-DSA-65 signature
#[tauri::command]
pub async fn ml_dsa_verify(
    data: Vec<u8>,
    signature_bytes: Vec<u8>,
    public_key_bytes: Vec<u8>,
    context: String,
) -> Result<PqcVerificationResult, String> {
    let mut details = HashMap::new();
    
    // Verify signature using ML-DSA-65
    let dsa = ml_dsa_65();
    let is_valid = match dsa.verify(&public_key_bytes, &data, &signature_bytes) {
        Ok(valid) => valid,
        Err(e) => {
            details.insert("error".to_string(), format!("Verification failed: {:?}", e));
            false
        }
    };
    
    details.insert("context".to_string(), context);
    details.insert("data_size".to_string(), data.len().to_string());
    details.insert("signature_size".to_string(), signature_bytes.len().to_string());
    
    Ok(PqcVerificationResult {
        is_valid,
        algorithm: "ML-DSA-65".to_string(),
        details,
    })
}

/// Encapsulate shared secret with ML-KEM-768
#[tauri::command]
pub async fn ml_kem_encapsulate(
    public_key_bytes: Vec<u8>,
) -> Result<PqcEncapsulationResult, String> {
    // Perform encapsulation using ML-KEM-768
    let kem = ml_kem_768();
    let (shared_secret, ciphertext) = kem.encapsulate(&public_key_bytes)
        .map_err(|e| format!("ML-KEM-768 encapsulation failed: {:?}", e))?;
    
    // Convert to bytes
    let ciphertext_bytes = ciphertext.to_vec();
    
    Ok(PqcEncapsulationResult {
        ciphertext: ciphertext_bytes,
        shared_secret: shared_secret.to_vec(),
        algorithm: "ML-KEM-768".to_string(),
    })
}

/// Decapsulate shared secret with ML-KEM-768
#[tauri::command]
pub async fn ml_kem_decapsulate(
    ciphertext_bytes: Vec<u8>,
    secret_key_bytes: Vec<u8>,
) -> Result<Vec<u8>, String> {
    // Perform decapsulation using ML-KEM-768
    let kem = ml_kem_768();
    let shared_secret = kem.decapsulate(&secret_key_bytes, &ciphertext_bytes)
        .map_err(|e| format!("ML-KEM-768 decapsulation failed: {:?}", e))?;
    
    Ok(shared_secret.to_vec())
}

/// Encrypt data using ChaCha20Poly1305 with ML-KEM-768 key encapsulation
#[tauri::command]
pub async fn pqc_encrypt(
    data: Vec<u8>,
    public_key_bytes: Vec<u8>,
) -> Result<PqcEncryptionResult, String> {
    // Encapsulate shared secret
    let encap_result = ml_kem_encapsulate(public_key_bytes).await?;
    
    // Use shared secret as ChaCha20Poly1305 key (first 32 bytes)
    let key = &encap_result.shared_secret[..32];
    let cipher = ChaCha20Poly1305::new(key.into());
    
    // Generate nonce using the provided function
    let nonce = generate_nonce();
    
    // Encrypt data using the clearer API
    let ciphertext = cipher.encrypt(&nonce, &data)
        .map_err(|e| format!("ChaCha20Poly1305 encryption failed: {}", e))?;
    
    Ok(PqcEncryptionResult {
        ciphertext,
        nonce: nonce.to_vec(),
        ml_kem_ciphertext: encap_result.ciphertext,
        algorithm: "ML-KEM-768+ChaCha20Poly1305".to_string(),
    })
}

/// Decrypt data using ChaCha20Poly1305 with ML-KEM-768 key decapsulation
#[tauri::command]
pub async fn pqc_decrypt(
    ciphertext: Vec<u8>,
    nonce: Vec<u8>,
    ml_kem_ciphertext: Vec<u8>,
    secret_key_bytes: Vec<u8>,
) -> Result<Vec<u8>, String> {
    // Decapsulate shared secret
    let shared_secret = ml_kem_decapsulate(ml_kem_ciphertext, secret_key_bytes).await?;
    
    // Use shared secret as ChaCha20Poly1305 key (first 32 bytes)
    let key = &shared_secret[..32];
    let cipher = ChaCha20Poly1305::new(key.into());
    
    // Verify nonce length
    if nonce.len() != 24 {
        return Err("Invalid nonce length".to_string());
    }
    
    // Decrypt data using the clearer API
    let plaintext = cipher.decrypt(&nonce, &ciphertext)
        .map_err(|e| format!("ChaCha20Poly1305 decryption failed: {}", e))?;
    
    Ok(plaintext)
}

/// Get PQC algorithm information
#[tauri::command]
pub async fn get_pqc_info() -> Result<HashMap<String, String>, String> {
    let mut info = HashMap::new();
    
    // Standard sizes for ML-KEM-768
    info.insert("ml_kem_768_public_key_size".to_string(), "1184".to_string());
    info.insert("ml_kem_768_secret_key_size".to_string(), "2400".to_string());
    info.insert("ml_kem_768_ciphertext_size".to_string(), "1088".to_string());
    
    // Standard sizes for ML-DSA-65
    info.insert("ml_dsa_65_public_key_size".to_string(), "1952".to_string());
    info.insert("ml_dsa_65_secret_key_size".to_string(), "4032".to_string());
    info.insert("ml_dsa_65_signature_size".to_string(), "3309".to_string());
    
    info.insert("chacha20poly1305_key_size".to_string(), "32".to_string());
    info.insert("chacha20poly1305_nonce_size".to_string(), "24".to_string());
    
    info.insert("algorithms".to_string(), "ML-KEM-768,ML-DSA-65,ChaCha20Poly1305".to_string());
    info.insert("version".to_string(), "0.3.0".to_string());
    
    Ok(info)
}