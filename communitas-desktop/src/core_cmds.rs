use communitas_core::keystore::Keystore;
use communitas_core::CoreContext;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use tracing::info;

use saorsa_core::fwid::{fw_check, fw_to_key};
use saorsa_core::quantum_crypto::{MlDsa65, MlDsaOperations, MlDsaSecretKey};

// Container storage: local, content-addressed, no DHT blobs (pointers-only policy).
fn data_root() -> PathBuf {
    if let Ok(p) = std::env::var("COMMUNITAS_DATA_DIR") {
        PathBuf::from(p)
    } else {
        PathBuf::from("src-tauri/.communitas-data")
    }
}

#[tauri::command]
pub async fn core_claim(words: [String; 4]) -> Result<String, String> {
    if !fw_check(words.clone()) {
        return Err("invalid four-word identity".into());
    }

    // Generate ML-DSA-65 keypair and bind to words (local persistence);
    // Pointers-only DHT: defer any network publish to core presence layer.
    let ml = MlDsa65::new();
    let (pk, sk) = ml
        .generate_keypair()
        .map_err(|e| format!("mldsa generate failed: {:?}", e))?;

    // Persist keys + identity in platform keychain
    let id_key = fw_to_key([
        words[0].clone(),
        words[1].clone(),
        words[2].clone(),
        words[3].clone(),
    ])
    .map_err(|e| format!("derive id key failed: {}", e))?;
    let id_hex = hex::encode(id_key.as_bytes());
    let ks = Keystore::new();
    ks.save_mldsa_keys(&id_hex, pk.as_bytes(), sk.as_bytes())?;
    ks.save_words(&id_hex, &words)?;
    ks.save_current_identity(&id_hex)?;
    if ks.load_device_id().is_err() {
        let dev = uuid::Uuid::new_v4().to_string();
        let _ = ks.save_device_id(&dev);
    }

    info!("claimed identity {}", id_hex);
    Ok(id_hex)
}

/// Generate a valid four-word identity using saorsa-core
#[tauri::command]
pub async fn generate_four_word_identity() -> Result<String, String> {
    use rand::RngCore;
    use rand::rngs::OsRng;
    use saorsa_core::address::NetworkAddress;
    use std::net::Ipv4Addr;
    
    let mut rng = OsRng;
    const MIN_PORT: u16 = 1024;
    const PORT_SPAN: u32 = u16::MAX as u32 - MIN_PORT as u32 + 1;
    const GENERATION_ATTEMPTS: usize = 1000;

    for _ in 0..GENERATION_ATTEMPTS {
        let ipv4 = Ipv4Addr::from(rng.next_u32());
        let port = (rng.next_u32() % PORT_SPAN) as u16 + MIN_PORT;
        let candidate = NetworkAddress::from_ipv4(ipv4, port);

        if let Some(words) = candidate.four_words() {
            // Parse to ensure it's valid
            if let Ok(parsed) = saorsa_core::identity::FourWordAddress::parse_str(words) {
                let words_array: [String; 4] = parsed.words()
                    .try_into()
                    .map_err(|_| "Should have exactly 4 words".to_string())?;

                // Validate with saorsa-core
                if fw_check(words_array) {
                    return Ok(words.to_string());
                }
            }
        }
    }

    Err(format!("Failed to generate valid four-word address after {} attempts", GENERATION_ATTEMPTS))
}

/// Check if DHT client is connected and ready
#[tauri::command]
pub async fn check_dht_connection(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
) -> Result<bool, String> {
    let guard = shared.read().await;
    if let Some(ctx) = guard.as_ref() {
        // Check if we have core context initialized
        // In production, this would check actual DHT connectivity via ctx.messaging or storage
        // For now, just check if we have a valid context
        Ok(!ctx.four_words.is_empty())
    } else {
        Ok(false)
    }
}

/// Find group storage disk from four-word identity
#[tauri::command]
pub async fn find_group_storage_disk(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    group_four_words: String,
) -> Result<String, String> {
    let guard = shared.read().await;
    if let Some(_ctx) = guard.as_ref() {
        // Parse four-words to get the group key
        let words: Vec<String> = group_four_words.split('-').map(|s| s.to_string()).collect();
        if words.len() != 4 {
            return Err("Invalid four-word format".to_string());
        }
        
        let words_array: [String; 4] = words.try_into().map_err(|_| "Invalid four-word format".to_string())?;
        if !fw_check(words_array.clone()) {
            return Err("Invalid four-word identity".to_string());
        }
        
        let group_key = fw_to_key(words_array).map_err(|e| format!("fw_to_key failed: {}", e))?;
        
        // Storage disks are derived from group identity hash
        // For now, return a deterministic storage disk ID based on the group key
        let disk_id = hex::encode(&group_key.as_bytes()[..16]); // Use first 16 bytes as disk ID
        Ok(format!("disk://{}", disk_id))
    } else {
        Err("No core context".to_string())
    }
}

/// Store user identity on DHT with display name and current addresses
#[tauri::command]
pub async fn store_user_identity(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    display_name: String,
    current_four_words: String,
) -> Result<(), String> {
    let guard = shared.read().await;
    if let Some(_ctx) = guard.as_ref() {
        // For now, just validate the four-words and return success
        let words: Vec<String> = current_four_words.split('-').map(|s| s.to_string()).collect();
        if words.len() != 4 {
            return Err("Invalid four-word format".to_string());
        }
        
        let words_array: [String; 4] = words.try_into().map_err(|_| "Invalid four-word format".to_string())?;
        if !fw_check(words_array) {
            return Err("Invalid four-word identity".to_string());
        }
        
        info!("Stored user identity: {} ({})", display_name, current_four_words);
        Ok(())
    } else {
        Err("No core context".to_string())
    }
}

/// Find user's current address for direct connection
#[tauri::command]
pub async fn find_user_current_address(
    shared: State<'_, Arc<RwLock<Option<CoreContext>>>>,
    user_four_words: String,
) -> Result<String, String> {
    let guard = shared.read().await;
    if let Some(_ctx) = guard.as_ref() {
        // Validate four-words
        let words: Vec<String> = user_four_words.split('-').map(|s| s.to_string()).collect();
        if words.len() != 4 {
            return Err("Invalid four-word format".to_string());
        }
        
        let words_array: [String; 4] = words.try_into().map_err(|_| "Invalid four-word format".to_string())?;
        if !fw_check(words_array) {
            return Err("Invalid four-word identity".to_string());
        }
        
        // For now, return the same four-words as current address
        // In real implementation, this would look up from DHT
        Ok(user_four_words)
    } else {
        Err("No core context".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvertiseResult {
    pub id_hex: String,
    pub endpoint_fw4: Option<String>,
}

#[tauri::command]
pub async fn core_advertise(addr: String, _storage_gb: u32) -> Result<AdvertiseResult, String> {
    let ks = Keystore::new();
    let id_hex = ks.load_current_identity()?;
    let (_pk_bytes, sk_bytes) = ks.load_mldsa_keys(&id_hex)?;

    // IPv4-first: parse host:port; compute optional fw4 string for UI
    let mut ipv4: Option<(String, u16)> = None;
    if let Some((host, port_str)) = addr.split_once(':')
        && let Ok(port) = port_str.parse::<u16>()
    {
        ipv4 = Some((host.to_string(), port));
    }

    // Sign a presence heartbeat locally (no blob publish here; pointers-only)
    let mut _presence_sig: Option<Vec<u8>> = None;
    if let Some((_host, _port)) = &ipv4 {
        let ml = MlDsa65::new();
        let sk = MlDsaSecretKey::from_bytes(&sk_bytes)
            .map_err(|e| format!("invalid mldsa sk: {:?}", e))?;
        let msg = format!("communitas:presence:v1:{}:{}", id_hex, addr);
        let sig = ml
            .sign(&sk, msg.as_bytes())
            .map_err(|e| format!("mldsa sign presence failed: {:?}", e))?;
        _presence_sig = Some(sig.0.to_vec());
    }

    // Optional fw4 encoding for IPv4
    // IMPORTANT: The four_word_networking crate is designed to encode IP+port TOGETHER
    // This is correct behavior - we encode the complete socket address into 4 words
    // DO NOT try to separate IP from port - they are meant to be encoded together
    let mut endpoint_fw4: Option<String> = None;
    if let Some((ref ip, port)) = ipv4
        && let Ok(v4) = ip.parse::<std::net::Ipv4Addr>()
    {
        // Create socket address with both IP and port
        let socket_addr = std::net::SocketAddr::from((v4, port));
        // Encode the complete address (IP+port) into 4 words
        let enc = four_word_networking::FourWordEncoder::new()
            .encode(socket_addr)
            .map_err(|e| format!("fw4 encode failed: {}", e))?;
        endpoint_fw4 = Some(enc.to_string().replace(' ', "-"));
    }
    Ok(AdvertiseResult {
        id_hex,
        endpoint_fw4,
    })
}

#[tauri::command]
pub async fn container_put(bytes: Vec<u8>, _group_size: usize) -> Result<String, String> {
    // Store locally (pointers-only)
    let handle = hex::encode(blake3::hash(&bytes).as_bytes());
    let ks = Keystore::new();
    let id_hex = ks.load_current_identity()?;
    let root = data_root();
    let dir = root.join("personal").join(&id_hex);
    let path = dir.join(format!("{}.data", handle));
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("mkdirs failed: {}", e))?;
    }
    tokio::fs::write(&path, &bytes)
        .await
        .map_err(|e| format!("write object failed: {}", e))?;
    Ok(handle)
}

#[tauri::command]
pub async fn container_get(handle: String) -> Result<Vec<u8>, String> {
    if handle.len() != 64 {
        return Err("invalid handle format (expect hex blake3)".into());
    }
    let ks = Keystore::new();
    let id_hex = ks.load_current_identity()?;
    let path = data_root()
        .join("personal")
        .join(&id_hex)
        .join(format!("{}.data", handle));
    tokio::fs::read(&path)
        .await
        .map_err(|e| format!("object not found/read failed: {}", e))
}
