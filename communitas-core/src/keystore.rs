// Simple encrypted keystore wrapper using platform keychain via `keyring`.
// Stores ML-DSA keys and current device/identity metadata.

use base64::Engine;
use keyring::Entry;

const SERVICE: &str = "communitas-tauri";

fn entry(user: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, user).map_err(|e| format!("keyring entry error: {}", e))
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
        entry("current_id")?
            .set_password(id_hex)
            .map_err(|e| e.to_string())
    }

    pub fn load_current_identity(&self) -> Result<String, String> {
        entry("current_id")?
            .get_password()
            .map_err(|e| format!("load current identity failed: {}", e))
    }

    pub fn save_words(&self, id_hex: &str, words: &[String; 4]) -> Result<(), String> {
        let val = words.join("-");
        entry(&format!("words:{}", id_hex))?
            .set_password(&val)
            .map_err(|e| e.to_string())
    }

    #[allow(dead_code)]
    pub fn load_words(&self, id_hex: &str) -> Result<[String; 4], String> {
        let joined = entry(&format!("words:{}", id_hex))?
            .get_password()
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
        entry(&format!("mldsa_pk:{}", id_hex))?
            .set_password(&pk_b64)
            .map_err(|e| e.to_string())?;
        entry(&format!("mldsa_sk:{}", id_hex))?
            .set_password(&sk_b64)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn load_mldsa_keys(&self, id_hex: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
        let pk_b64 = entry(&format!("mldsa_pk:{}", id_hex))?
            .get_password()
            .map_err(|e| format!("load pk failed: {}", e))?;
        let sk_b64 = entry(&format!("mldsa_sk:{}", id_hex))?
            .get_password()
            .map_err(|e| format!("load sk failed: {}", e))?;
        let pk = base64::engine::general_purpose::STANDARD
            .decode(pk_b64)
            .map_err(|e| format!("pk decode: {}", e))?;
        let sk = base64::engine::general_purpose::STANDARD
            .decode(sk_b64)
            .map_err(|e| format!("sk decode: {}", e))?;
        Ok((pk, sk))
    }

    pub fn save_device_id(&self, device_id: &str) -> Result<(), String> {
        entry("device_id")?
            .set_password(device_id)
            .map_err(|e| e.to_string())
    }

    pub fn load_device_id(&self) -> Result<String, String> {
        entry("device_id")?
            .get_password()
            .map_err(|e| format!("load device_id failed: {}", e))
    }
}
