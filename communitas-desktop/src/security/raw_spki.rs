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

    #[allow(dead_code)]
    pub fn set_pinned_key(&mut self, key: Vec<u8>, algorithm: String) {
        // For now, only support 32-byte keys (Ed25519)
        // TODO: Update to Vec<u8> to support variable-length PQC keys
        if key.len() == 32 {
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&key);
            self.pinned_key = Some(arr);
            self.fingerprint = Some(calculate_fingerprint(&arr));
        }
        // Store algorithm for future use
        let _ = algorithm; // Will use when we support variable-length keys
    }
}

// Placeholder types and functions for PQC SPKI parser (to be implemented)
#[allow(dead_code)]
pub struct ParsedSpki {
    pub algorithm_name: String,
    pub key_bytes: Vec<u8>,
}

#[allow(dead_code)]
pub fn parse_spki_any_algorithm(_spki: &[u8]) -> Result<ParsedSpki, String> {
    Err("PQC SPKI parser not yet implemented - see REMAINING_PRODUCTION_BLOCKERS.md".into())
}

#[allow(dead_code)]
pub fn calculate_key_fingerprint(key: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(key);
    let hash = hasher.finalize();
    hex::encode(&hash.as_bytes()[..16])
}

#[allow(dead_code)]
pub fn compare_keys(key1: &[u8], alg1: &str, key2: &[u8], alg2: &str) -> bool {
    alg1 == alg2 && key1 == key2
}

#[allow(dead_code)]
pub fn parse_raw_or_spki(_data: &[u8]) -> Result<ParsedSpki, String> {
    Err("Not yet implemented".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test vectors for different algorithm types
    fn generate_test_ed25519_spki() -> Vec<u8> {
        // Real Ed25519 SPKI structure (44 bytes total)
        let mut spki = vec![
            0x30, 0x2a, // SEQUENCE, 42 bytes
            0x30, 0x05, // SEQUENCE (algorithm), 5 bytes
            0x06, 0x03, 0x2b, 0x65, 0x70, // OID 1.3.101.112 (Ed25519)
            0x03, 0x21, 0x00, // BIT STRING, 33 bytes
        ];
        spki.extend_from_slice(&[0x42; 32]);
        spki
    }

    #[test]
    fn test_current_ed25519_parser_works() {
        let spki = generate_test_ed25519_spki();
        let result = extract_key_from_spki(&spki);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [0x42; 32]);
    }

    #[test]
    fn test_fingerprint_calculation() {
        let key = [0x42; 32];
        let fp = calculate_fingerprint(&key);
        assert!(!fp.is_empty());
        assert!(hex::decode(&fp).is_ok());
    }

    #[test]
    #[ignore] // Will pass after PQC implementation
    fn test_parse_pqc_spki_placeholder() {
        // Placeholder test - will be implemented with real PQC SPKI parser
        let mock_spki = vec![0x30; 100];
        let result = parse_spki_any_algorithm(&mock_spki);
        // Currently returns error (not implemented)
        assert!(result.is_err());
    }
}
