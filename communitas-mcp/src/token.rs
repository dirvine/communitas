// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

use crate::auth::{DelegateToken, Scope};
use anyhow::{Result, anyhow};
use base64::Engine;
use rand::Rng;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const SERVER_SECRET_FILE: &str = "mcp_server_secret";
const SECRET_SIZE: usize = 32;

pub struct TokenManager {
    server_secret: [u8; SECRET_SIZE],
}

impl TokenManager {
    pub async fn new(vault_dir: PathBuf) -> Result<Self> {
        let secret_path = vault_dir.join(SERVER_SECRET_FILE);
        let server_secret = Self::load_or_create_secret(&secret_path).await?;
        Ok(Self { server_secret })
    }

    async fn load_or_create_secret(path: &PathBuf) -> Result<[u8; SECRET_SIZE]> {
        if path.exists() {
            let data = fs::read(path).await?;
            if data.len() != SECRET_SIZE {
                return Err(anyhow!("Invalid server secret file size"));
            }
            let mut secret = [0u8; SECRET_SIZE];
            secret.copy_from_slice(&data);
            Ok(secret)
        } else {
            let mut secret = [0u8; SECRET_SIZE];
            rand::thread_rng().fill(&mut secret);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await?;
            }

            let mut file = fs::File::create(path).await?;
            file.write_all(&secret).await?;
            Ok(secret)
        }
    }

    pub fn create_token(
        &self,
        issuer: &str,
        delegate_name: &str,
        scopes: Vec<Scope>,
        expires_in_hours: u64,
    ) -> Result<String> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("System time before UNIX epoch: {}", e))?
            .as_secs();

        let nonce: [u8; 16] = rand::thread_rng().r#gen();
        let nonce_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(nonce);

        let token = DelegateToken {
            issuer: issuer.to_string(),
            delegate_name: delegate_name.to_string(),
            scopes,
            issued_at: now,
            expires_at: now + (expires_in_hours * 3600),
            nonce: nonce_b64,
        };

        let payload = serde_json::to_vec(&token)?;
        let signature = self.sign(&payload);

        let payload_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&payload);
        let sig_b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(signature);

        Ok(format!("{}.{}", payload_b64, sig_b64))
    }

    pub fn verify_token(&self, token_str: &str) -> Result<DelegateToken> {
        let parts: Vec<&str> = token_str.split('.').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid token format"));
        }

        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[0])
            .map_err(|_| anyhow!("Invalid token encoding"))?;

        let provided_sig = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| anyhow!("Invalid signature encoding"))?;

        let expected_sig = self.sign(&payload);

        if !constant_time_eq(&provided_sig, &expected_sig) {
            return Err(anyhow!("Invalid token signature"));
        }

        let token: DelegateToken =
            serde_json::from_slice(&payload).map_err(|_| anyhow!("Invalid token payload"))?;

        if token.is_expired() {
            return Err(anyhow!("Token has expired"));
        }

        Ok(token)
    }

    fn sign(&self, data: &[u8]) -> Vec<u8> {
        let hash = blake3::keyed_hash(&self.server_secret, data);
        hash.as_bytes().to_vec()
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_token_creation_and_verification() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = TokenManager::new(temp_dir.path().to_path_buf())
            .await
            .expect("Failed to create token manager");

        let token = manager
            .create_token(
                "ocean-forest-moon-star",
                "test-agent",
                vec![Scope::ReadMessages, Scope::SendMessages],
                24,
            )
            .expect("Failed to create token");

        let verified = manager
            .verify_token(&token)
            .expect("Failed to verify token");
        assert_eq!(verified.issuer, "ocean-forest-moon-star");
        assert_eq!(verified.delegate_name, "test-agent");
        assert!(verified.has_scope(&Scope::ReadMessages));
        assert!(verified.has_scope(&Scope::SendMessages));
        assert!(!verified.has_scope(&Scope::WriteFiles));
    }

    #[tokio::test]
    async fn test_invalid_signature() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = TokenManager::new(temp_dir.path().to_path_buf())
            .await
            .expect("Failed to create token manager");

        let token = manager
            .create_token("test-issuer", "test-agent", vec![Scope::Full], 24)
            .expect("Failed to create token");

        let parts: Vec<&str> = token.split('.').collect();
        let tampered = format!("{}.{}", parts[0], "invalid_signature");

        assert!(manager.verify_token(&tampered).is_err());
    }

    #[tokio::test]
    async fn test_expired_token() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let manager = TokenManager::new(temp_dir.path().to_path_buf())
            .await
            .expect("Failed to create token manager");

        let token = manager
            .create_token("test-issuer", "test-agent", vec![Scope::Full], 0)
            .expect("Failed to create token");

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        assert!(manager.verify_token(&token).is_err());
    }
}
