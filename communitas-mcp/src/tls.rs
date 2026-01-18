// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! TLS configuration for MCP HTTPS server
//!
//! Implements RFC 7250 Raw Public Keys with ML-DSA-65 (post-quantum) signatures.
//! This allows AI agents to connect securely without traditional X.509 certificates.
//!
//! ## Security Model
//!
//! - Server presents ML-DSA-65 public key directly (not wrapped in X.509)
//! - Client verifies server's public key is trusted (or allows any in demo mode)
//! - Optional mutual TLS where client also presents its public key
//! - TLS 1.3 only - no downgrade attacks possible

use std::sync::Arc;

use rustls::{
    CertificateError, DigitallySignedStruct, DistinguishedName, Error as TlsError, ServerConfig,
    SignatureScheme,
    pki_types::{CertificateDer, UnixTime},
    server::{ResolvesServerCert, danger::ClientCertVerifier},
    sign::{CertifiedKey, SigningKey},
};
use saorsa_pqc::{MlDsa65, MlDsaOperations, MlDsaPublicKey, MlDsaSecretKey, MlDsaSignature};
use thiserror::Error;
use tracing::{debug, info, warn};

/// ML-DSA-65 OID: 2.16.840.1.101.3.4.3.18 (NIST CSOR)
const ML_DSA_65_OID: [u8; 9] = [0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x03, 0x12];

/// ML-DSA-65 public key size (1952 bytes per FIPS 204)
pub const ML_DSA_65_PUBLIC_KEY_SIZE: usize = 1952;

/// ML-DSA-65 signature size (3309 bytes per FIPS 204)
pub const ML_DSA_65_SIGNATURE_SIZE: usize = 3309;

/// ML-DSA-65 signature scheme (IANA 0x0905)
const ML_DSA_65_SCHEME: SignatureScheme = SignatureScheme::ML_DSA_65;

/// TLS configuration errors
#[derive(Debug, Error)]
pub enum TlsConfigError {
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    #[allow(dead_code)]
    InvalidSignature(String),

    #[error("TLS error: {0}")]
    TlsError(#[from] TlsError),

    #[error("Key generation failed: {0}")]
    KeyGeneration(String),

    #[error("SPKI encoding failed: {0}")]
    #[allow(dead_code)]
    SpkiEncoding(String),
}

/// Server TLS configuration with ML-DSA-65 raw public keys
pub struct ServerTlsConfig {
    config: ServerConfig,
}

impl ServerTlsConfig {
    /// Get the rustls ServerConfig
    pub fn into_inner(self) -> ServerConfig {
        self.config
    }

    /// Get a reference to the rustls ServerConfig
    #[allow(dead_code)]
    pub fn as_ref(&self) -> &ServerConfig {
        &self.config
    }
}

/// Builder for server TLS configuration
pub struct ServerTlsConfigBuilder {
    secret_key: Option<MlDsaSecretKey>,
    public_key: Option<MlDsaPublicKey>,
    allow_any_client: bool,
    trusted_client_keys: Vec<MlDsaPublicKey>,
    require_client_auth: bool,
}

impl Default for ServerTlsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerTlsConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            secret_key: None,
            public_key: None,
            allow_any_client: false,
            trusted_client_keys: Vec::new(),
            require_client_auth: true,
        }
    }

    /// Set the server's ML-DSA-65 key pair
    pub fn with_keypair(mut self, secret_key: MlDsaSecretKey, public_key: MlDsaPublicKey) -> Self {
        self.secret_key = Some(secret_key);
        self.public_key = Some(public_key);
        self
    }

    /// Allow any valid ML-DSA-65 client key (demo mode)
    /// WARNING: Only use for development/testing!
    pub fn allow_any_client(mut self) -> Self {
        self.allow_any_client = true;
        self
    }

    /// Add a trusted client public key
    #[allow(dead_code)]
    pub fn add_trusted_client(mut self, public_key: MlDsaPublicKey) -> Self {
        self.trusted_client_keys.push(public_key);
        self
    }

    /// Disable client authentication (server-only TLS)
    pub fn no_client_auth(mut self) -> Self {
        self.require_client_auth = false;
        self
    }

    /// Build the server TLS configuration
    pub fn build(self) -> Result<ServerTlsConfig, TlsConfigError> {
        let secret_key = self
            .secret_key
            .ok_or_else(|| TlsConfigError::KeyGeneration("Server secret key required".into()))?;
        let public_key = self
            .public_key
            .ok_or_else(|| TlsConfigError::KeyGeneration("Server public key required".into()))?;

        // Create the server cert resolver
        let resolver = RawPublicKeyResolver::new(secret_key, public_key)?;

        // Get the crypto provider
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());

        // Build config with or without client auth
        let config = if self.require_client_auth {
            let client_verifier = if self.allow_any_client {
                RawPublicKeyClientVerifier::allow_any()
            } else {
                RawPublicKeyClientVerifier::new(self.trusted_client_keys)
            };

            ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()?
                .with_client_cert_verifier(Arc::new(client_verifier))
                .with_cert_resolver(Arc::new(resolver))
        } else {
            ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()?
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(resolver))
        };

        Ok(ServerTlsConfig { config })
    }
}

/// Generate a new ML-DSA-65 keypair for the server
pub fn generate_keypair() -> Result<(MlDsaPublicKey, MlDsaSecretKey), TlsConfigError> {
    let ml_dsa = MlDsa65::new();
    ml_dsa
        .generate_keypair()
        .map_err(|e| TlsConfigError::KeyGeneration(format!("{e:?}")))
}

/// Create SubjectPublicKeyInfo DER encoding for ML-DSA-65 public key
pub fn create_spki(public_key: &MlDsaPublicKey) -> Result<Vec<u8>, TlsConfigError> {
    let key_bytes = public_key.as_bytes();
    let key_len = key_bytes.len();

    if key_len != ML_DSA_65_PUBLIC_KEY_SIZE {
        return Err(TlsConfigError::InvalidPublicKey(format!(
            "Expected {ML_DSA_65_PUBLIC_KEY_SIZE} bytes, got {key_len}"
        )));
    }

    // Algorithm identifier: SEQUENCE { OID }
    let oid_with_tag_len = 2 + ML_DSA_65_OID.len(); // 11 bytes
    let algorithm_seq_content_len = oid_with_tag_len;

    // BIT STRING: tag (0x03) + length + 0x00 (unused bits) + key
    let bit_string_content_len = 1 + key_len;
    let bit_string_len_encoding = length_encoding_size(bit_string_content_len);
    let bit_string_total = 1 + bit_string_len_encoding + bit_string_content_len;

    // Algorithm SEQUENCE
    let algo_seq_len_encoding = length_encoding_size(algorithm_seq_content_len);
    let algo_seq_total = 1 + algo_seq_len_encoding + algorithm_seq_content_len;

    // Outer SEQUENCE content
    let outer_content_len = algo_seq_total + bit_string_total;

    let mut spki = Vec::with_capacity(4 + outer_content_len);

    // Outer SEQUENCE
    spki.push(0x30);
    encode_length(&mut spki, outer_content_len);

    // Algorithm identifier SEQUENCE
    spki.push(0x30);
    encode_length(&mut spki, algorithm_seq_content_len);

    // OID
    spki.push(0x06);
    spki.push(ML_DSA_65_OID.len() as u8);
    spki.extend_from_slice(&ML_DSA_65_OID);

    // Subject public key BIT STRING
    spki.push(0x03);
    encode_length(&mut spki, bit_string_content_len);
    spki.push(0x00); // No unused bits
    spki.extend_from_slice(key_bytes);

    Ok(spki)
}

/// Extract ML-DSA-65 public key from SubjectPublicKeyInfo
pub fn extract_key_from_spki(spki: &[u8]) -> Result<MlDsaPublicKey, TlsConfigError> {
    let mut pos = 0;

    // Parse outer SEQUENCE
    if spki.get(pos) != Some(&0x30) {
        return Err(TlsConfigError::InvalidPublicKey(
            "Missing outer SEQUENCE".into(),
        ));
    }
    pos += 1;

    let (outer_len, len_bytes) =
        parse_length(&spki[pos..]).map_err(|e| TlsConfigError::InvalidPublicKey(e.to_string()))?;
    pos += len_bytes;

    if spki.len() < pos + outer_len {
        return Err(TlsConfigError::InvalidPublicKey("Truncated SPKI".into()));
    }

    // Parse algorithm identifier SEQUENCE
    if spki.get(pos) != Some(&0x30) {
        return Err(TlsConfigError::InvalidPublicKey(
            "Missing algorithm SEQUENCE".into(),
        ));
    }
    pos += 1;

    let (algo_len, len_bytes) =
        parse_length(&spki[pos..]).map_err(|e| TlsConfigError::InvalidPublicKey(e.to_string()))?;
    pos += len_bytes;
    let algo_end = pos + algo_len;

    // Parse OID
    if spki.get(pos) != Some(&0x06) {
        return Err(TlsConfigError::InvalidPublicKey("Missing OID".into()));
    }
    pos += 1;

    let (oid_len, len_bytes) =
        parse_length(&spki[pos..]).map_err(|e| TlsConfigError::InvalidPublicKey(e.to_string()))?;
    pos += len_bytes;

    if oid_len != ML_DSA_65_OID.len() {
        return Err(TlsConfigError::InvalidPublicKey(
            "Invalid OID length".into(),
        ));
    }

    if spki.get(pos..pos + oid_len) != Some(&ML_DSA_65_OID[..]) {
        return Err(TlsConfigError::InvalidPublicKey(
            "Not an ML-DSA-65 key".into(),
        ));
    }
    pos = algo_end;

    // Parse BIT STRING
    if spki.get(pos) != Some(&0x03) {
        return Err(TlsConfigError::InvalidPublicKey(
            "Missing BIT STRING".into(),
        ));
    }
    pos += 1;

    let (bit_string_len, len_bytes) =
        parse_length(&spki[pos..]).map_err(|e| TlsConfigError::InvalidPublicKey(e.to_string()))?;
    pos += len_bytes;

    // First byte is unused bits (must be 0)
    if spki.get(pos) != Some(&0x00) {
        return Err(TlsConfigError::InvalidPublicKey(
            "Invalid unused bits".into(),
        ));
    }
    pos += 1;

    let key_len = bit_string_len - 1;
    if key_len != ML_DSA_65_PUBLIC_KEY_SIZE {
        return Err(TlsConfigError::InvalidPublicKey(format!(
            "Invalid key size: expected {ML_DSA_65_PUBLIC_KEY_SIZE}, got {key_len}"
        )));
    }

    let key_bytes = spki
        .get(pos..pos + key_len)
        .ok_or_else(|| TlsConfigError::InvalidPublicKey("Truncated key".into()))?;

    MlDsaPublicKey::from_bytes(key_bytes)
        .map_err(|e| TlsConfigError::InvalidPublicKey(format!("{e:?}")))
}

// =============================================================================
// Internal Types
// =============================================================================

/// Raw Public Key resolver for server
#[derive(Debug)]
struct RawPublicKeyResolver {
    certified_key: Arc<CertifiedKey>,
}

impl RawPublicKeyResolver {
    fn new(secret_key: MlDsaSecretKey, public_key: MlDsaPublicKey) -> Result<Self, TlsConfigError> {
        let spki = create_spki(&public_key)?;
        let rustls_signing_key = MlDsaRustlsSigningKey::new(secret_key);

        let certified_key = Arc::new(CertifiedKey {
            cert: vec![CertificateDer::from(spki)],
            key: Arc::new(rustls_signing_key),
            ocsp: None,
        });

        Ok(Self { certified_key })
    }
}

impl ResolvesServerCert for RawPublicKeyResolver {
    fn resolve(&self, _client_hello: rustls::server::ClientHello) -> Option<Arc<CertifiedKey>> {
        debug!("Resolving server certificate with ML-DSA-65 Raw Public Key");
        Some(self.certified_key.clone())
    }
}

/// Raw Public Key client verifier for mutual TLS
#[derive(Debug)]
struct RawPublicKeyClientVerifier {
    trusted_keys: Vec<MlDsaPublicKey>,
    allow_any_key: bool,
}

impl RawPublicKeyClientVerifier {
    fn new(trusted_keys: Vec<MlDsaPublicKey>) -> Self {
        Self {
            trusted_keys,
            allow_any_key: false,
        }
    }

    fn allow_any() -> Self {
        Self {
            trusted_keys: Vec::new(),
            allow_any_key: true,
        }
    }
}

impl ClientCertVerifier for RawPublicKeyClientVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<rustls::server::danger::ClientCertVerified, TlsError> {
        debug!("Verifying client certificate with ML-DSA-65 Raw Public Key verifier");

        let public_key = extract_key_from_spki(end_entity.as_ref())
            .map_err(|_| TlsError::InvalidCertificate(CertificateError::BadEncoding))?;

        if self.allow_any_key {
            info!("Accepting any ML-DSA-65 client key (demo mode)");
            return Ok(rustls::server::danger::ClientCertVerified::assertion());
        }

        for trusted in &self.trusted_keys {
            if public_key.as_bytes() == trusted.as_bytes() {
                info!("Client public key is trusted");
                return Ok(rustls::server::danger::ClientCertVerified::assertion());
            }
        }

        warn!("Unknown client public key");
        Err(TlsError::InvalidCertificate(
            CertificateError::UnknownIssuer,
        ))
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        // TLS 1.2 not supported for Raw Public Keys
        Err(TlsError::UnsupportedNameType)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, TlsError> {
        debug!("Verifying TLS 1.3 ML-DSA-65 client signature");

        let public_key = extract_key_from_spki(cert.as_ref())
            .map_err(|_| TlsError::InvalidCertificate(CertificateError::BadEncoding))?;

        // Verify signature
        let signature_bytes = dss.signature();
        if signature_bytes.len() != ML_DSA_65_SIGNATURE_SIZE {
            return Err(TlsError::General(format!(
                "Invalid signature size: expected {}, got {}",
                ML_DSA_65_SIGNATURE_SIZE,
                signature_bytes.len()
            )));
        }

        let signature = MlDsaSignature::from_bytes(signature_bytes)
            .map_err(|e| TlsError::General(format!("Invalid signature: {e:?}")))?;

        let ml_dsa = MlDsa65::new();
        let valid = ml_dsa
            .verify(&public_key, message, &signature)
            .map_err(|e| TlsError::General(format!("Signature verification failed: {e:?}")))?;

        if !valid {
            return Err(TlsError::General(
                "Signature verification failed".to_string(),
            ));
        }

        debug!("TLS 1.3 ML-DSA-65 client signature verification successful");
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![ML_DSA_65_SCHEME]
    }

    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self) -> bool {
        true
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

/// ML-DSA-65 signing key wrapper for rustls
struct MlDsaRustlsSigningKey {
    secret_key: MlDsaSecretKey,
}

impl std::fmt::Debug for MlDsaRustlsSigningKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MlDsaRustlsSigningKey")
            .field("secret_key", &"[REDACTED]")
            .finish()
    }
}

impl MlDsaRustlsSigningKey {
    fn new(secret_key: MlDsaSecretKey) -> Self {
        Self { secret_key }
    }
}

impl SigningKey for MlDsaRustlsSigningKey {
    fn choose_scheme(&self, offered: &[SignatureScheme]) -> Option<Box<dyn rustls::sign::Signer>> {
        if offered.contains(&ML_DSA_65_SCHEME) {
            // Clone the secret key bytes for the signer
            let key_bytes = self.secret_key.as_bytes().to_vec();
            Some(Box::new(MlDsaSigner { key_bytes }))
        } else {
            warn!("ML-DSA-65 not in offered signature schemes: {:?}", offered);
            None
        }
    }

    fn algorithm(&self) -> rustls::SignatureAlgorithm {
        // Use Unknown since ML-DSA-65 isn't in rustls's enum yet
        rustls::SignatureAlgorithm::Unknown(0x09)
    }
}

/// ML-DSA-65 signer for rustls
#[derive(Debug)]
struct MlDsaSigner {
    key_bytes: Vec<u8>,
}

impl rustls::sign::Signer for MlDsaSigner {
    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, TlsError> {
        let secret_key = MlDsaSecretKey::from_bytes(&self.key_bytes)
            .map_err(|e| TlsError::General(format!("Invalid secret key: {e:?}")))?;

        let ml_dsa = MlDsa65::new();
        let signature = ml_dsa
            .sign(&secret_key, message)
            .map_err(|e| TlsError::General(format!("ML-DSA-65 sign failed: {e:?}")))?;
        Ok(signature.as_bytes().to_vec())
    }

    fn scheme(&self) -> SignatureScheme {
        ML_DSA_65_SCHEME
    }
}

// =============================================================================
// ASN.1 Helpers
// =============================================================================

fn length_encoding_size(len: usize) -> usize {
    if len < 128 {
        1
    } else if len < 256 {
        2
    } else {
        3
    }
}

fn encode_length(output: &mut Vec<u8>, len: usize) {
    if len < 128 {
        output.push(len as u8);
    } else if len < 256 {
        output.push(0x81);
        output.push(len as u8);
    } else {
        output.push(0x82);
        output.push((len >> 8) as u8);
        output.push((len & 0xFF) as u8);
    }
}

fn parse_length(data: &[u8]) -> Result<(usize, usize), &'static str> {
    if data.is_empty() {
        return Err("Empty length data");
    }

    let first = data[0];
    if first < 128 {
        Ok((first as usize, 1))
    } else if first == 0x81 {
        if data.len() < 2 {
            return Err("Truncated length");
        }
        Ok((data[1] as usize, 2))
    } else if first == 0x82 {
        if data.len() < 3 {
            return Err("Truncated length");
        }
        let len = ((data[1] as usize) << 8) | (data[2] as usize);
        Ok((len, 3))
    } else {
        Err("Unsupported length encoding")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spki_round_trip() {
        let (public_key, _secret_key) = generate_keypair().unwrap();

        let spki = create_spki(&public_key).unwrap();
        let recovered = extract_key_from_spki(&spki).unwrap();

        assert_eq!(public_key.as_bytes(), recovered.as_bytes());
    }

    #[test]
    fn test_length_encoding() {
        // Short form
        let mut buf = Vec::new();
        encode_length(&mut buf, 50);
        assert_eq!(buf, vec![50]);

        // Long form (1 byte)
        buf.clear();
        encode_length(&mut buf, 200);
        assert_eq!(buf, vec![0x81, 200]);

        // Long form (2 bytes)
        buf.clear();
        encode_length(&mut buf, 1000);
        assert_eq!(buf, vec![0x82, 0x03, 0xE8]);
    }

    #[test]
    fn test_parse_length() {
        let (len, consumed) = parse_length(&[50]).unwrap();
        assert_eq!(len, 50);
        assert_eq!(consumed, 1);

        let (len, consumed) = parse_length(&[0x81, 200]).unwrap();
        assert_eq!(len, 200);
        assert_eq!(consumed, 2);

        let (len, consumed) = parse_length(&[0x82, 0x03, 0xE8]).unwrap();
        assert_eq!(len, 1000);
        assert_eq!(consumed, 3);
    }

    #[test]
    fn test_invalid_spki_rejected() {
        let (public_key, _secret_key) = generate_keypair().unwrap();
        let mut spki = create_spki(&public_key).unwrap();
        spki.pop();
        let err = extract_key_from_spki(&spki).unwrap_err();
        assert!(matches!(err, TlsConfigError::InvalidPublicKey(_)));
    }

    #[test]
    fn test_client_verifier_accepts_trusted_key() {
        let (public_key, _secret_key) = generate_keypair().unwrap();
        let spki = create_spki(&public_key).unwrap();
        let cert = CertificateDer::from(spki);
        let verifier = RawPublicKeyClientVerifier::new(vec![public_key]);
        let result = verifier.verify_client_cert(&cert, &[], UnixTime::now());
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_verifier_rejects_unknown_key() {
        let (public_key, _secret_key) = generate_keypair().unwrap();
        let (other_public_key, _other_secret_key) = generate_keypair().unwrap();
        let spki = create_spki(&public_key).unwrap();
        let cert = CertificateDer::from(spki);
        let verifier = RawPublicKeyClientVerifier::new(vec![other_public_key]);
        let result = verifier.verify_client_cert(&cert, &[], UnixTime::now());
        assert!(matches!(
            result,
            Err(TlsError::InvalidCertificate(
                CertificateError::UnknownIssuer
            ))
        ));
    }
}
