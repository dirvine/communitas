//! WebAuthn Protocol Implementation for Local-First Authentication
//!
//! This module provides WebAuthn (Web Authentication) protocol support for
//! passkey-based authentication. Unlike server-based WebAuthn, this implementation
//! is designed for local-first applications where credentials are stored locally.
//!
//! Architecture:
//! - Uses `communitas.local` as the Relying Party ID
//! - Credentials are stored in platform keyring (see passkey.rs)
//! - User verification is always required (biometric or PIN)
//! - No attestation verification (local-first, no server trust)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use webauthn_rs::prelude::*;

/// WebAuthn configuration for Communitas
pub struct WebAuthnConfig {
    /// Relying Party ID (domain-like identifier)
    pub rp_id: String,
    /// Relying Party name for display
    pub rp_name: String,
    /// Relying Party origin
    pub rp_origin: Url,
}

impl WebAuthnConfig {
    /// Create default configuration
    ///
    /// Uses `communitas.local` as the relying party ID.
    pub fn new() -> Result<Self> {
        Ok(Self {
            rp_id: "communitas.local".to_string(),
            rp_name: "Communitas".to_string(),
            // For local-first, use a localhost origin
            rp_origin: Url::parse("https://communitas.local")
                .context("Failed to parse RP origin URL")?,
        })
    }
}

// Note: WebAuthnConfig does not implement Default because URL parsing can
// theoretically fail. Use WebAuthnConfig::new() instead.

/// Result of starting a passkey registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    /// Base64-encoded challenge
    pub challenge: String,
    /// User ID for this registration
    pub user_id: String,
    /// User display name
    pub user_display_name: String,
    /// Relying Party ID
    pub rp_id: String,
    /// Serialized registration state for verification
    #[serde(skip)]
    pub state: Option<PasskeyRegistration>,
}

/// Result of a successful passkey registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResult {
    /// Credential ID (base64-encoded)
    pub credential_id: String,
    /// Raw credential ID bytes
    pub credential_id_bytes: Vec<u8>,
    /// Public key (COSE format, base64-encoded)
    pub public_key: String,
    /// Signature counter (for clone detection)
    pub counter: u32,
}

/// Result of starting a passkey authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    /// Base64-encoded challenge
    pub challenge: String,
    /// Allowed credential IDs (base64-encoded)
    pub allowed_credentials: Vec<String>,
    /// Relying Party ID
    pub rp_id: String,
    /// Serialized authentication state for verification
    #[serde(skip)]
    pub state: Option<PasskeyAuthentication>,
}

/// Result of a successful passkey authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResult {
    /// Credential ID that was used
    pub credential_id: String,
    /// Updated signature counter
    pub counter: u32,
    /// Whether user verification was performed
    pub user_verified: bool,
}

/// WebAuthn handler for passkey operations
pub struct WebAuthnHandler {
    webauthn: Webauthn,
    config: WebAuthnConfig,
}

impl WebAuthnHandler {
    /// Create a new WebAuthn handler with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(WebAuthnConfig::new()?)
    }

    /// Create a new WebAuthn handler with custom configuration
    pub fn with_config(config: WebAuthnConfig) -> Result<Self> {
        let builder = WebauthnBuilder::new(&config.rp_id, &config.rp_origin)
            .context("Failed to create WebAuthn builder")?
            .rp_name(&config.rp_name);

        let webauthn = builder.build().context("Failed to build WebAuthn")?;

        Ok(Self { webauthn, config })
    }

    /// Start passkey registration for a user
    ///
    /// Returns a challenge that should be passed to the authenticator (browser/platform).
    /// The returned state must be kept for verification.
    pub fn start_registration(
        &self,
        user_id: &str,
        user_display_name: &str,
        existing_credentials: &[CredentialID],
    ) -> Result<(CreationChallengeResponse, PasskeyRegistration)> {
        // Create user unique ID from the four-word identity using blake3 hash
        let hash = blake3::hash(user_id.as_bytes());
        let hash_bytes = hash.as_bytes();
        // Take first 16 bytes for UUID
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash_bytes[..16]);
        let user_unique_id = uuid::Uuid::from_bytes(uuid_bytes);

        let (ccr, reg_state) = self
            .webauthn
            .start_passkey_registration(
                user_unique_id,
                user_id,
                user_display_name,
                Some(existing_credentials.to_vec()),
            )
            .context("Failed to start passkey registration")?;

        Ok((ccr, reg_state))
    }

    /// Complete passkey registration with authenticator response
    ///
    /// Verifies the registration response and returns the credential.
    pub fn finish_registration(
        &self,
        response: &RegisterPublicKeyCredential,
        state: &PasskeyRegistration,
    ) -> Result<Passkey> {
        let passkey = self
            .webauthn
            .finish_passkey_registration(response, state)
            .context("Failed to finish passkey registration")?;

        Ok(passkey)
    }

    /// Start passkey authentication
    ///
    /// Returns a challenge that should be passed to the authenticator.
    pub fn start_authentication(
        &self,
        credentials: &[Passkey],
    ) -> Result<(RequestChallengeResponse, PasskeyAuthentication)> {
        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(credentials)
            .context("Failed to start passkey authentication")?;

        Ok((rcr, auth_state))
    }

    /// Complete passkey authentication with authenticator response
    ///
    /// Verifies the authentication response and returns the updated credential.
    pub fn finish_authentication(
        &self,
        response: &PublicKeyCredential,
        state: &PasskeyAuthentication,
    ) -> Result<AuthenticationResult> {
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(response, state)
            .context("Failed to finish passkey authentication")?;

        Ok(AuthenticationResult {
            credential_id: base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                auth_result.cred_id().as_ref(),
            ),
            counter: auth_result.counter(),
            user_verified: auth_result.user_verified(),
        })
    }

    /// Get the Relying Party ID
    pub fn rp_id(&self) -> &str {
        &self.config.rp_id
    }

    /// Convert a Passkey to our WebAuthnCredential format for storage
    pub fn passkey_to_credential(passkey: &Passkey) -> super::WebAuthnCredential {
        super::WebAuthnCredential {
            id: base64::Engine::encode(
                &base64::engine::general_purpose::URL_SAFE_NO_PAD,
                passkey.cred_id().as_ref(),
            ),
            raw_id: passkey.cred_id().to_vec(),
            credential_type: "public-key".to_string(),
            // Serialize the passkey for storage
            attestation_object: serde_json::to_vec(passkey).unwrap_or_default(),
            client_data_json: Vec::new(), // Not needed for local storage
        }
    }

    /// Convert our WebAuthnCredential back to a Passkey for authentication
    pub fn credential_to_passkey(credential: &super::WebAuthnCredential) -> Result<Passkey> {
        serde_json::from_slice(&credential.attestation_object)
            .context("Failed to deserialize passkey from credential")
    }
}

// Note: WebAuthnHandler does not implement Default because creation can fail.
// Use WebAuthnHandler::new() instead.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webauthn_handler_creation() {
        let handler = WebAuthnHandler::new();
        assert!(handler.is_ok());
        let handler = handler.unwrap();
        assert_eq!(handler.rp_id(), "communitas.local");
    }

    #[test]
    fn test_webauthn_config_new() {
        let config = WebAuthnConfig::new().unwrap();
        assert_eq!(config.rp_id, "communitas.local");
        assert_eq!(config.rp_name, "Communitas");
    }

    #[test]
    fn test_start_registration() {
        let handler = WebAuthnHandler::new().unwrap();
        let result = handler.start_registration("ocean-forest-moon-star", "Test User", &[]);
        assert!(result.is_ok());
        let (ccr, _state) = result.unwrap();
        // Check that the challenge was generated
        assert!(!ccr.public_key.challenge.as_ref().is_empty());
    }
}
