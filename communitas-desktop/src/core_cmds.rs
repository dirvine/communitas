use communitas_core::keystore::Keystore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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
        && let Ok(port) = port_str.parse::<u16>() {
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
    let mut endpoint_fw4: Option<String> = None;
    if let Some((ref ip, port)) = ipv4
        && let Ok(v4) = ip.parse::<std::net::Ipv4Addr>() {
            let enc = four_word_networking::FourWordEncoder::new()
                .encode_ipv4(v4, port)
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
