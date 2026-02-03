//! Identity key storage within a vault.
//!
//! Stores public/secret ML-DSA keys on disk with restrictive permissions.

use anyhow::{Context, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

const IDENTITY_KEYS_FILE: &str = "identity.keys.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityKeyMaterial {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IdentityKeyFile {
    version: u32,
    display_name: String,
    public_key_b64: String,
    secret_key_b64: String,
    created_at: u64,
}

pub fn vault_dir_from_root(storage_root: &Path) -> PathBuf {
    storage_root.join("vaults")
}

pub fn identity_keys_path(vault_dir: &Path, four_words: &str) -> PathBuf {
    vault_dir.join(four_words).join(IDENTITY_KEYS_FILE)
}

pub async fn identity_keys_exist(vault_dir: &Path, four_words: &str) -> bool {
    identity_keys_path(vault_dir, four_words).exists()
}

pub async fn load_identity_keys(vault_dir: &Path, four_words: &str) -> Result<IdentityKeyMaterial> {
    let path = identity_keys_path(vault_dir, four_words);
    let raw = fs::read(&path)
        .await
        .with_context(|| format!("Failed to read identity keys at {}", path.display()))?;
    let record: IdentityKeyFile = serde_json::from_slice(&raw)
        .with_context(|| format!("Failed to parse identity keys at {}", path.display()))?;

    let public_key = base64::engine::general_purpose::STANDARD
        .decode(&record.public_key_b64)
        .context("Failed to decode public key")?;
    let secret_key = base64::engine::general_purpose::STANDARD
        .decode(&record.secret_key_b64)
        .context("Failed to decode secret key")?;

    Ok(IdentityKeyMaterial {
        public_key,
        secret_key,
    })
}

pub async fn ensure_identity_keys(
    vault_dir: &Path,
    four_words: &str,
    display_name: &str,
    public_key: &[u8],
    secret_key: &[u8],
) -> Result<()> {
    let path = identity_keys_path(vault_dir, four_words);
    if path.exists() {
        let existing = load_identity_keys(vault_dir, four_words).await?;
        if existing.public_key == public_key && existing.secret_key == secret_key {
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "Identity keys already exist for {four_words} and do not match"
        ));
    }

    store_identity_keys(vault_dir, four_words, display_name, public_key, secret_key).await
}

pub async fn store_identity_keys(
    vault_dir: &Path,
    four_words: &str,
    display_name: &str,
    public_key: &[u8],
    secret_key: &[u8],
) -> Result<()> {
    let path = identity_keys_path(vault_dir, four_words);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await.with_context(|| {
            format!(
                "Failed to create identity key directory {}",
                parent.display()
            )
        })?;
    }

    let record = IdentityKeyFile {
        version: 1,
        display_name: display_name.to_string(),
        public_key_b64: base64::engine::general_purpose::STANDARD.encode(public_key),
        secret_key_b64: base64::engine::general_purpose::STANDARD.encode(secret_key),
        created_at: current_timestamp(),
    };

    let data = serde_json::to_vec(&record)?;
    fs::write(&path, data)
        .await
        .with_context(|| format!("Failed to write identity keys at {}", path.display()))?;
    secure_file(&path).await?;

    Ok(())
}

async fn secure_file(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path).await?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions).await?;
    }

    Ok(())
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
