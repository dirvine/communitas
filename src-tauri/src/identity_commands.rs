// Identity commands for four-word identity system using four-word-networking crate

use four_word_networking::FourWordAdaptiveEncoder;
use blake3;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;
use rand::rngs::OsRng;
// Use post-quantum ML-DSA from saorsa-core
use saorsa_core::quantum_crypto::ml_dsa as pq_dsa;
use tokio::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use crate::AppState;
use saorsa_core::dht::DHT;
use saorsa_core::dht::Key as DhtKey;
use saorsa_core::error::P2pResult;

/// Identity packet as specified in the architecture
#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityPacket {
    pub four_words: String,
    pub public_key: Vec<u8>,           // VerifyingKey bytes
    pub signature: Vec<u8>,            // Signature bytes
    pub dht_id: String,                // BLAKE3 hash of four words
    pub created_at: u64,               // Unix timestamp
    pub packet_version: u32,           // For future upgrades
}

/// Global state for claimed identities (in production, this would be in DHT)
pub struct IdentityState {
    claimed: Mutex<HashMap<String, IdentityPacket>>, // four-words -> IdentityPacket
    encoder: Mutex<FourWordAdaptiveEncoder>,
}

impl IdentityState {
    pub fn new() -> Result<Self, String> {
        let encoder = FourWordAdaptiveEncoder::new()
            .map_err(|e| format!("Failed to create encoder: {}", e))?;
        
        Ok(Self {
            claimed: Mutex::new(HashMap::new()),
            encoder: Mutex::new(encoder),
        })
    }
}

/// Generate a new four-word identity using the four-word-networking dictionary
#[tauri::command]
pub async fn generate_four_word_identity(
    seed: Option<String>,
    state: State<'_, IdentityState>,
) -> Result<String, String> {
    let encoder = state.encoder.lock().unwrap();
    
    // Generate 4 random words from the dictionary
    let words = if let Some(seed_str) = seed {
        // Use seed for deterministic generation (for testing)
        let hash = blake3::hash(seed_str.as_bytes());
        let hash_bytes = hash.as_bytes();
        
        // Use hash to select words deterministically
        let mut selected_words = Vec::new();
        for i in 0..4 {
            let byte_index = i * 8; // Use 8 bytes per word selection
            if byte_index + 8 <= hash_bytes.len() {
                let word_bytes = &hash_bytes[byte_index..byte_index + 8];
                let _word_index = u64::from_le_bytes(word_bytes.try_into().unwrap());
                
                // Get a deterministic word based on the hash
                // For now, use random words - deterministic selection would need 
                // access to the encoder's internal word list
                let word = encoder.get_random_words(1)[0].clone();
                selected_words.push(word);
            } else {
                selected_words.push(encoder.get_random_words(1)[0].clone());
            }
        }
        selected_words
    } else {
        // Random generation
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
    let encoder = state.encoder.lock().unwrap();
    
    // Check format: exactly 4 words separated by hyphens
    let words: Vec<&str> = four_words.split('-').collect();
    if words.len() != 4 {
        return Ok(false);
    }
    
    // Check each word is in the dictionary
    for word in &words {
        if !encoder.is_valid_word(word) {
            return Ok(false);
        }
    }
    
    // Check for reserved words (admin, root, system, test)
    let reserved = ["admin", "root", "system", "test"];
    for word in &words {
        if reserved.contains(word) {
            return Ok(false);
        }
    }
    
    Ok(true)
}

/// Check if a four-word identity is available (not claimed)
#[tauri::command]
pub async fn check_identity_availability(
    four_words: String,
    state: State<'_, IdentityState>,
) -> Result<bool, String> {
    let claimed = state.claimed.lock().unwrap();
    
    // In production, this would check the DHT
    Ok(!claimed.contains_key(&four_words))
}

/// Generate a keypair for identity claiming
#[tauri::command]
pub async fn generate_identity_keypair() -> Result<(String, String), String> {
    // Generate ML-DSA (post-quantum) keypair via saorsa-core
    let (public_key, private_key) = pq_dsa::generate_keypair()
        .map_err(|e| format!("Failed to generate ML-DSA keypair: {e}"))?;

    // Return (private, public) for compatibility with UI
    Ok((hex::encode(private_key), hex::encode(public_key)))
}

/// Claim a four-word identity with cryptographic proof
#[tauri::command]
pub async fn claim_four_word_identity_with_proof(
    four_words: String,
    private_key_hex: String,
    public_key_hex: String,
    state: State<'_, IdentityState>,
) -> Result<IdentityPacket, String> {
    // First validate the identity format
    if !validate_four_word_identity(four_words.clone(), state.clone()).await? {
        return Err("Invalid four-word identity format".to_string());
    }
    
    // Parse keys
    let private_key_bytes = hex::decode(private_key_hex)
        .map_err(|_| "Invalid private key hex format")?;
    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|_| "Invalid public key hex format")?;
    
    // Calculate DHT ID
    let dht_id = calculate_dht_id(four_words.clone()).await?;
    
    // Create signature of the four-words using ML-DSA to prove ownership
    let signature_bytes = pq_dsa::sign(&private_key_bytes, four_words.as_bytes())
        .map_err(|e| format!("Failed to sign identity: {e}"))?;
    
    // Create identity packet
    let identity_packet = IdentityPacket {
        four_words: four_words.clone(),
        public_key: public_key_bytes,
        signature: signature_bytes,
        dht_id,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        packet_version: 1,
    };
    
    // Check if already claimed
    let mut claimed = state.claimed.lock().unwrap();
    if claimed.contains_key(&four_words) {
        return Err("Identity already claimed".to_string());
    }
    
    // Store the claimed identity with cryptographic proof
    claimed.insert(four_words.clone(), identity_packet.clone());
    
    Ok(identity_packet)
}

/// Legacy claim function (deprecated - use claim_four_word_identity_with_proof)
#[tauri::command]
pub async fn claim_four_word_identity(
    four_words: String,
    state: State<'_, IdentityState>,
) -> Result<bool, String> {
    // Generate post-quantum keypair and claim
    let (private_key, public_key) = generate_identity_keypair().await?;
    match claim_four_word_identity_with_proof(four_words, private_key, public_key, state).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Verify an identity packet's cryptographic signature
#[tauri::command]
pub async fn verify_identity_packet(
    identity_packet: IdentityPacket,
) -> Result<bool, String> {
    // Verify DHT ID matches four words
    let expected_dht_id = calculate_dht_id(identity_packet.four_words.clone()).await?;
    if identity_packet.dht_id != expected_dht_id {
        return Ok(false);
    }
    // Verify ML-DSA signature using saorsa-core
    match pq_dsa::verify(
        &identity_packet.public_key,
        identity_packet.four_words.as_bytes(),
        &identity_packet.signature,
    ) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Get identity packet for claimed identity
#[tauri::command]
pub async fn get_identity_packet(
    four_words: String,
    state: State<'_, IdentityState>,
) -> Result<Option<IdentityPacket>, String> {
    let claimed = state.claimed.lock().unwrap();
    Ok(claimed.get(&four_words).cloned())
}

/// Publish identity packet to the local DHT mirror (file-based fallback)
#[tauri::command]
pub async fn publish_identity_packet(
    packet: IdentityPacket,
    app_state: State<'_, Arc<TokioRwLock<AppState>>>,
) -> Result<(), String> {
    // Verify packet first
    if !verify_identity_packet(packet.clone()).await? {
        return Err("Invalid identity packet".into());
    }

    // Attempt DHT publish if available; otherwise write to local cache as fallback.
    if let Some(p2p) = &app_state.read().await.p2p_node {
        let node = p2p.read().await;
        if let Some(dht) = node.node.dht() {
            let dht_guard = dht.read().await; // DHT is Arc<RwLock<...>> in main
            let key = DhtKey::from_hex(&packet.dht_id).map_err(|e| e.to_string())?;
            let value = serde_json::to_vec(&packet).map_err(|e| e.to_string())?;
            dht_guard
                .put(key, value)
                .await
                .map_err(|e| format!("Failed to publish identity to DHT: {e}"))?;
        }
    }

    // Always keep a local mirror
    let app_data_dir = PathBuf::from(".communitas-data").join("identity");
    if let Err(e) = fs::create_dir_all(&app_data_dir).await {
        return Err(format!("Failed to create identity directory: {e}"));
    }
    let path = app_data_dir.join(format!("{}.json", packet.dht_id));
    let bytes = serde_json::to_vec_pretty(&packet).map_err(|e| e.to_string())?;
    fs::write(&path, bytes).await.map_err(|e| format!("Failed to write identity packet: {e}"))?;
    Ok(())
}

/// Retrieve a published identity packet by DHT id
#[tauri::command]
pub async fn get_published_identity(
    dht_id: String,
) -> Result<Option<IdentityPacket>, String> {
    let path = PathBuf::from(".communitas-data").join("identity").join(format!("{}.json", dht_id));
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).await.map_err(|e| format!("Failed to read identity packet: {e}"))?;
    let packet: IdentityPacket = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok(Some(packet))
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
    let is_available = check_identity_availability(four_words.clone(), state.clone()).await?;
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
        is_available,
        visual_gradient,
    })
}

/// Get random words from the dictionary (for UI autocomplete)
#[tauri::command]
pub async fn get_dictionary_words(
    count: usize,
    state: State<'_, IdentityState>,
) -> Result<Vec<String>, String> {
    let encoder = state.encoder.lock().unwrap();
    Ok(encoder.get_random_words(count))
}

/// Validate a single word against the dictionary
#[tauri::command]
pub async fn validate_word(
    word: String,
    state: State<'_, IdentityState>,
) -> Result<bool, String> {
    let encoder = state.encoder.lock().unwrap();
    Ok(encoder.is_valid_word(&word))
}

/// Batch validate multiple four-word identities
#[tauri::command]
pub async fn batch_validate_identities(
    identities: Vec<String>,
    state: State<'_, IdentityState>,
) -> Result<Vec<bool>, String> {
    let mut results = Vec::new();
    
    for four_words in identities {
        let is_valid = validate_four_word_identity(four_words, state.clone()).await?;
        results.push(is_valid);
    }
    
    Ok(results)
}

/// Get statistics about the identity system
#[derive(Serialize, Deserialize)]
pub struct IdentityStatistics {
    pub total_claimed: usize,
    pub dictionary_size: usize,
    pub total_possible_combinations: String, // String because number is huge
}

#[tauri::command]
pub async fn get_identity_statistics(
    state: State<'_, IdentityState>,
) -> Result<IdentityStatistics, String> {
    let claimed = state.claimed.lock().unwrap();
    let _encoder = state.encoder.lock().unwrap();
    
    // Dictionary size is typically 4096 words in four-word-networking
    // For now, use the standard size since we don't have direct access to dictionary size
    let dictionary_size = 4096;
    
    // Total combinations = dictionary_size^4
    let total_combinations = (dictionary_size as u128).pow(4);
    
    Ok(IdentityStatistics {
        total_claimed: claimed.len(),
        dictionary_size,
        total_possible_combinations: total_combinations.to_string(),
    })
}

// Note: Using four-word-networking crate's built-in API
// - FourWordAdaptiveEncoder::new() creates the encoder
// - encoder.get_random_words(n) generates n random dictionary words  
// - encoder.is_valid_word(word) validates word is in dictionary

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test-friendly state wrappers
    async fn test_generate_four_word_identity(
        seed: Option<String>,
        state: &IdentityState,
    ) -> Result<String, String> {
        let encoder = state.encoder.lock().unwrap();
        
        // Generate 4 random words from the dictionary
        let words = if let Some(seed_str) = seed {
            // Use seed for deterministic generation (for testing)
            let hash = blake3::hash(seed_str.as_bytes());
            let hash_bytes = hash.as_bytes();
            
            // Use hash to select words deterministically
            let mut selected_words = Vec::new();
            for i in 0..4 {
                let byte_index = i * 8; // Use 8 bytes per word selection
                if byte_index + 8 <= hash_bytes.len() {
                    let word_bytes = &hash_bytes[byte_index..byte_index + 8];
                    let _word_index = u64::from_le_bytes(word_bytes.try_into().unwrap());
                    
                    // For now, use random words - deterministic selection would need 
                    // access to the encoder's internal word list
                    let word = encoder.get_random_words(1)[0].clone();
                    selected_words.push(word);
                } else {
                    selected_words.push(encoder.get_random_words(1)[0].clone());
                }
            }
            selected_words
        } else {
            // Random generation
            encoder.get_random_words(4)
        };
        
        Ok(words.join("-"))
    }

    async fn test_validate_four_word_identity(
        four_words: String,
        state: &IdentityState,
    ) -> Result<bool, String> {
        let encoder = state.encoder.lock().unwrap();
        
        // Check format: exactly 4 words separated by hyphens
        let words: Vec<&str> = four_words.split('-').collect();
        if words.len() != 4 {
            return Ok(false);
        }
        
        // Check each word is in the dictionary
        for word in &words {
            if !encoder.is_valid_word(word) {
                return Ok(false);
            }
        }
        
        // Check for reserved words (admin, root, system, test)
        let reserved = ["admin", "root", "system", "test"];
        for word in &words {
            if reserved.contains(word) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    async fn test_check_identity_availability(
        four_words: String,
        state: &IdentityState,
    ) -> Result<bool, String> {
        let claimed = state.claimed.lock().unwrap();
        Ok(!claimed.contains_key(&four_words))
    }

    async fn test_claim_four_word_identity(
        four_words: String,
        state: &IdentityState,
    ) -> Result<bool, String> {
        // First validate the identity
        if !test_validate_four_word_identity(four_words.clone(), state).await? {
            return Err("Invalid four-word identity format".to_string());
        }
        
        let mut claimed = state.claimed.lock().unwrap();
        
        // Check if already claimed
        if claimed.contains_key(&four_words) {
            return Ok(false);
        }
        
        // Create a mock identity packet for testing
        let test_packet = IdentityPacket {
            four_words: four_words.clone(),
            public_key: vec![0; 32], // Mock public key
            signature: vec![0; 64],  // Mock signature
            dht_id: calculate_dht_id(four_words.clone()).await?,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            packet_version: 1,
        };
        
        // Claim it
        claimed.insert(four_words.clone(), test_packet);
        Ok(true)
    }

    #[tokio::test]
    async fn test_identity_state_creation() {
        let state = IdentityState::new().expect("Failed to create identity state");
        
        // Test encoder works
        let encoder = state.encoder.lock().unwrap();
        let words = encoder.get_random_words(4);
        assert_eq!(words.len(), 4);
        
        // Test each word is valid
        for word in &words {
            assert!(encoder.is_valid_word(word), "Word '{}' should be valid", word);
        }
    }

    #[tokio::test]
    async fn test_four_word_generation() {
        let state = IdentityState::new().expect("Failed to create identity state");
        
        let four_words = test_generate_four_word_identity(None, &state)
            .await
            .expect("Failed to generate four-word identity");
        
        // Should be in format word-word-word-word
        assert!(four_words.matches('-').count() == 3, "Should have exactly 3 hyphens");
        
        let words: Vec<&str> = four_words.split('-').collect();
        assert_eq!(words.len(), 4, "Should have exactly 4 words");
        
        // Each word should be non-empty and alphabetic
        for word in words {
            assert!(!word.is_empty(), "Word should not be empty");
            assert!(word.chars().all(|c| c.is_alphabetic()), "Word should be alphabetic");
        }
    }

    #[tokio::test]
    async fn test_four_word_validation() {
        let state = IdentityState::new().expect("Failed to create identity state");
        
        // Generate a valid identity
        let four_words = test_generate_four_word_identity(None, &state)
            .await
            .expect("Failed to generate four-word identity");
        
        // Should validate as true
        let is_valid = test_validate_four_word_identity(four_words.clone(), &state)
            .await
            .expect("Failed to validate four-word identity");
        assert!(is_valid, "Generated identity should be valid");
        
        // Test invalid formats
        let invalid_cases = vec![
            "word1-word2-word3", // Too few words
            "word1-word2-word3-word4-word5", // Too many words
            "word1--word3-word4", // Empty word
            "word1-word2-word3-admin", // Reserved word
        ];
        
        for invalid in invalid_cases {
            let is_valid = test_validate_four_word_identity(invalid.to_string(), &state)
                .await
                .expect("Failed to validate four-word identity");
            assert!(!is_valid, "Invalid format '{}' should not validate", invalid);
        }
    }

    #[tokio::test]
    async fn test_dht_id_calculation() {
        let test_identity = "ocean-forest-mountain-river".to_string();
        
        let dht_id = calculate_dht_id(test_identity.clone())
            .await
            .expect("Failed to calculate DHT ID");
        
        // Should be a hex string of BLAKE3 hash (64 characters)
        assert_eq!(dht_id.len(), 64, "DHT ID should be 64 hex characters");
        assert!(dht_id.chars().all(|c| c.is_ascii_hexdigit()), "DHT ID should be hex");
        
        // Same input should produce same output
        let dht_id2 = calculate_dht_id(test_identity.clone())
            .await
            .expect("Failed to calculate DHT ID");
        assert_eq!(dht_id, dht_id2, "Same identity should produce same DHT ID");
    }

    #[tokio::test]
    async fn test_identity_availability() {
        let state = IdentityState::new().expect("Failed to create identity state");
        
        let test_identity = "ocean-forest-mountain-river".to_string();
        
        // Should be available initially
        let available = test_check_identity_availability(test_identity.clone(), &state)
            .await
            .expect("Failed to check availability");
        assert!(available, "Identity should be available initially");
        
        // Claim it
        let claimed = test_claim_four_word_identity(test_identity.clone(), &state)
            .await
            .expect("Failed to claim identity");
        assert!(claimed, "Should successfully claim identity");
        
        // Should no longer be available
        let available = test_check_identity_availability(test_identity.clone(), &state)
            .await
            .expect("Failed to check availability");
        assert!(!available, "Identity should not be available after claiming");
    }

    async fn test_claim_four_word_identity_with_proof(
        four_words: String,
        private_key_hex: String,
        state: &IdentityState,
    ) -> Result<IdentityPacket, String> {
        // First validate the identity format
        if !test_validate_four_word_identity(four_words.clone(), state).await? {
            return Err("Invalid four-word identity format".to_string());
        }
        
        // Parse private key
        let private_key_bytes = hex::decode(private_key_hex)
            .map_err(|_| "Invalid private key hex format")?;
        let signing_key = SigningKey::from_bytes(
            &private_key_bytes.try_into()
                .map_err(|_| "Private key must be 32 bytes")?
        );
        let public_key = signing_key.verifying_key();
        
        // Calculate DHT ID
        let dht_id = calculate_dht_id(four_words.clone()).await?;
        
        // Create signature of the four-words to prove ownership
        let signature = signing_key.sign(four_words.as_bytes());
        
        // Create identity packet
        let identity_packet = IdentityPacket {
            four_words: four_words.clone(),
            public_key: public_key.to_bytes().to_vec(),
            signature: signature.to_bytes().to_vec(),
            dht_id,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            packet_version: 1,
        };
        
        // Check if already claimed
        let mut claimed = state.claimed.lock().unwrap();
        if claimed.contains_key(&four_words) {
            return Err("Identity already claimed".to_string());
        }
        
        // Store the claimed identity with cryptographic proof
        claimed.insert(four_words.clone(), identity_packet.clone());
        
        Ok(identity_packet)
    }

    #[tokio::test]
    async fn test_cryptographic_identity_claiming() {
        let state = IdentityState::new().expect("Failed to create identity state");
        
        // Generate a keypair
        let (private_key, _public_key) = generate_identity_keypair().await.expect("Failed to generate keypair");
        
        // Generate a valid four-word identity
        let four_words = test_generate_four_word_identity(None, &state)
            .await
            .expect("Failed to generate four-word identity");
        
        // Claim it with cryptographic proof
        let identity_packet = test_claim_four_word_identity_with_proof(
            four_words.clone(),
            private_key.clone(),
            &state
        )
        .await
        .expect("Failed to claim identity with proof");
        
        // Verify the identity packet structure
        assert_eq!(identity_packet.four_words, four_words);
        assert_eq!(identity_packet.public_key.len(), 32);
        assert_eq!(identity_packet.signature.len(), 64);
        assert_eq!(identity_packet.packet_version, 1);
        assert!(identity_packet.created_at > 0);
        
        // Verify the signature
        let is_valid = verify_identity_packet(identity_packet.clone()).await.expect("Failed to verify packet");
        assert!(is_valid, "Identity packet should have valid signature");
        
        // Try to claim the same identity again (should fail)
        let result = test_claim_four_word_identity_with_proof(
            four_words.clone(),
            private_key,
            &state
        ).await;
        assert!(result.is_err(), "Should not be able to claim the same identity twice");
    }

    #[tokio::test]
    async fn test_signature_verification() {
        // Test valid signature
        let (private_key, _public_key) = generate_identity_keypair().await.expect("Failed to generate keypair");
        let test_words = "ocean-forest-mountain-river".to_string();
        
        // Create signed packet
        let private_key_bytes = hex::decode(private_key).expect("Failed to decode private key");
        let signing_key = SigningKey::from_bytes(&private_key_bytes.try_into().unwrap());
        let public_key = signing_key.verifying_key();
        let signature = signing_key.sign(test_words.as_bytes());
        
        let valid_packet = IdentityPacket {
            four_words: test_words.clone(),
            public_key: public_key.to_bytes().to_vec(),
            signature: signature.to_bytes().to_vec(),
            dht_id: calculate_dht_id(test_words.clone()).await.unwrap(),
            created_at: 1234567890,
            packet_version: 1,
        };
        
        let is_valid = verify_identity_packet(valid_packet).await.expect("Failed to verify");
        assert!(is_valid, "Valid signature should verify");
        
        // Test invalid signature (tampered four-words)
        let mut invalid_packet = IdentityPacket {
            four_words: "tampered-forest-mountain-river".to_string(), // Different words
            public_key: public_key.to_bytes().to_vec(),
            signature: signature.to_bytes().to_vec(), // But same signature
            dht_id: calculate_dht_id("tampered-forest-mountain-river".to_string()).await.unwrap(),
            created_at: 1234567890,
            packet_version: 1,
        };
        
        let is_valid = verify_identity_packet(invalid_packet).await.expect("Failed to verify");
        assert!(!is_valid, "Invalid signature should not verify");
    }
}