// Simple encrypted keystore wrapper using platform keychain via `keyring`.
// Stores ML-DSA keys and current device/identity metadata.

use base64::Engine;
use keyring::Entry;
use zeroize::Zeroize;

// Primary service name for current releases.
const SERVICE: &str = "communitas";
// Legacy service name from earlier desktop builds for keyring migration.
const LEGACY_SERVICE: &str = "communitas-tauri";

fn entry(service: &str, user: &str) -> Result<Entry, String> {
    Entry::new(service, user).map_err(|e| format!("keyring entry error: {}", e))
}

fn entry_primary(user: &str) -> Result<Entry, String> {
    entry(SERVICE, user)
}

fn load_password_with_legacy(user: &str) -> Result<String, String> {
    match entry_primary(user)?.get_password() {
        Ok(value) => Ok(value),
        Err(primary_err) => {
            let legacy_entry = entry(LEGACY_SERVICE, user)?;
            match legacy_entry.get_password() {
                Ok(value) => {
                    // Best-effort migrate to new service.
                    let _ = entry_primary(user)
                        .and_then(|entry| entry.set_password(&value).map_err(|e| e.to_string()));
                    Ok(value)
                }
                Err(legacy_err) => Err(format!(
                    "load keyring entry failed: {} (legacy: {})",
                    primary_err, legacy_err
                )),
            }
        }
    }
}

pub struct Keystore;

impl Default for Keystore {
    fn default() -> Self {
        Self::new()
    }
}

impl Keystore {
    pub fn new() -> Self {
        Self
    }

    pub fn save_current_identity(&self, id_hex: &str) -> Result<(), String> {
        entry_primary("current_id")?
            .set_password(id_hex)
            .map_err(|e| e.to_string())
    }

    pub fn load_current_identity(&self) -> Result<String, String> {
        load_password_with_legacy("current_id")
            .map_err(|e| format!("load current identity failed: {}", e))
    }

    pub fn save_words(&self, id_hex: &str, words: &[String; 4]) -> Result<(), String> {
        let val = words.join("-");
        entry_primary(&format!("words:{}", id_hex))?
            .set_password(&val)
            .map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn load_words(&self, id_hex: &str) -> Result<[String; 4], String> {
        let joined = load_password_with_legacy(&format!("words:{}", id_hex))
            .map_err(|e| format!("load words failed: {}", e))?;
        let parts: Vec<String> = joined.split('-').map(|s| s.to_string()).collect();
        if parts.len() != 4 {
            return Err("stored words invalid".into());
        }
        Ok([
            parts[0].clone(),
            parts[1].clone(),
            parts[2].clone(),
            parts[3].clone(),
        ])
    }

    pub fn save_mldsa_keys(&self, id_hex: &str, pk: &[u8], sk: &[u8]) -> Result<(), String> {
        let pk_b64 = base64::engine::general_purpose::STANDARD.encode(pk);
        let sk_b64 = base64::engine::general_purpose::STANDARD.encode(sk);
        entry_primary(&format!("mldsa_pk:{}", id_hex))?
            .set_password(&pk_b64)
            .map_err(|e| e.to_string())?;
        entry_primary(&format!("mldsa_sk:{}", id_hex))?
            .set_password(&sk_b64)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_mldsa_keys(&self, id_hex: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        let mut pk_b64 = load_password_with_legacy(&format!("mldsa_pk:{}", id_hex))
            .map_err(|e| format!("load pk failed: {}", e))?;
        let mut sk_b64 = load_password_with_legacy(&format!("mldsa_sk:{}", id_hex))
            .map_err(|e| format!("load sk failed: {}", e))?;

        let pk = base64::engine::general_purpose::STANDARD
            .decode(&pk_b64)
            .map_err(|e| {
                pk_b64.zeroize();
                sk_b64.zeroize();
                format!("pk decode: {}", e)
            })?;
        let sk = base64::engine::general_purpose::STANDARD
            .decode(&sk_b64)
            .map_err(|e| {
                pk_b64.zeroize();
                sk_b64.zeroize();
                format!("sk decode: {}", e)
            })?;

        // Zeroize base64 strings after use
        pk_b64.zeroize();
        sk_b64.zeroize();

        Ok((pk, sk))
    }

    pub fn save_device_id(&self, device_id: &str) -> Result<(), String> {
        entry_primary("device_id")?
            .set_password(device_id)
            .map_err(|e| e.to_string())
    }

    pub fn load_device_id(&self) -> Result<String, String> {
        load_password_with_legacy("device_id").map_err(|e| format!("load device_id failed: {}", e))
    }
}
