use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use blake3::Hasher;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Default)]
pub struct RawSpkiState {
    pinned_key: Option<[u8; 32]>, // Ed25519 raw public key bytes
    fingerprint: Option<String>,  // SHA-256 fingerprint for logging
}

fn try_parse_hex(s: &str) -> Option<Vec<u8>> {
    hex::decode(s).ok()
}

fn try_parse_b64(s: &str) -> Option<Vec<u8>> {
    B64.decode(s.as_bytes()).ok()
}

fn extract_key_from_spki(spki: &[u8]) -> Result<[u8; 32], String> {
    if spki.len() == 44 {
        let mut out = [0u8; 32];
        out.copy_from_slice(&spki[12..44]);
        return Ok(out);
    }
    Err("unsupported SPKI format (expected Ed25519 44-byte SPKI)".into())
}

fn parse_spki_or_key_bytes(input: &str) -> Result<[u8; 32], String> {
    // Allow prefixes like spki:hex:..., spki:b64:..., key:hex:..., key:b64:...
    let lower = input.trim();
    let parts: Vec<&str> = lower.splitn(2, ':').collect();
    let (kind, rest) = if parts.len() == 2 && (parts[0] == "spki" || parts[0] == "key") {
        (parts[0], parts[1])
    } else {
        ("", lower)
    };

    // Try hex first, then base64
    let bytes = try_parse_hex(rest)
        .or_else(|| try_parse_b64(rest))
        .ok_or_else(|| "value is not valid hex or base64".to_string())?;

    match (kind, bytes.len()) {
        ("spki", 44) | ("", 44) => extract_key_from_spki(&bytes),
        ("key", 32) | ("", 32) => {
            let mut out = [0u8; 32];
            out.copy_from_slice(&bytes);
            Ok(out)
        }
        _ => Err(format!(
            "unexpected byte length {} (want 32 key or 44 SPKI)",
            bytes.len()
        )),
    }
}

/// Calculate BLAKE3 fingerprint of SPKI key for logging
fn calculate_fingerprint(key: &[u8; 32]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(key);
    let hash = hasher.finalize();
    // Take first 16 bytes for compact fingerprint
    hex::encode(&hash.as_bytes()[..16])
}

#[tauri::command]
pub async fn sync_set_quic_pinned_spki(
    state: tauri::State<'_, Arc<RwLock<RawSpkiState>>>,
    value: String,
) -> Result<bool, String> {
    // Release build guard: reject allow-any bypass
    #[cfg(not(debug_assertions))]
    {
        if value.to_lowercase() == "allow-any" || value.to_lowercase() == "any" {
            return Err("SPKI pinning bypass not allowed in release builds".into());
        }
    }

    // Debug build warning
    #[cfg(debug_assertions)]
    {
        if value.to_lowercase() == "allow-any" || value.to_lowercase() == "any" {
            warn!("⚠️  SECURITY: SPKI pinning disabled in development mode");
            let mut w = state.write().await;
            w.pinned_key = None;
            w.fingerprint = None;
            return Ok(true);
        }
    }

    let key = parse_spki_or_key_bytes(&value)?;
    let fingerprint = calculate_fingerprint(&key);

    info!("SPKI pin set: fingerprint={}", fingerprint);

    let mut w = state.write().await;
    w.pinned_key = Some(key);
    w.fingerprint = Some(fingerprint);
    Ok(true)
}

#[tauri::command]
pub async fn sync_clear_quic_pinned_spki(
    state: tauri::State<'_, Arc<RwLock<RawSpkiState>>>,
) -> Result<bool, String> {
    info!("SPKI pin cleared");
    let mut w = state.write().await;
    w.pinned_key = None;
    w.fingerprint = None;
    Ok(true)
}

pub fn _parse_env_pinned_spki() -> Option<[u8; 32]> {
    if let Ok(val) = std::env::var("COMMUNITAS_QUIC_PINNED_SPKI") {
        parse_spki_or_key_bytes(&val).ok()
    } else {
        None
    }
}

impl RawSpkiState {
    pub fn _get(&self) -> Option<[u8; 32]> {
        self.pinned_key
    }
}
