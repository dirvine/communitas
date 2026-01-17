# Core Library API Reference

Rust library API reference for the Communitas core (`communitas-core` crate).

## Overview

The `communitas-core` crate provides the core functionality for identity management, storage, messaging, and P2P networking. This document covers the public Rust API for building custom applications on top of Communitas.

**Primary Crates**:
- `communitas-core` - Core identity, storage, and messaging
- `saorsa-pqc` - Post-quantum cryptography primitives
- `ant-quic` - QUIC transport layer
- `four-word-networking` - Connection word encoding/decoding (IP:port)

**Terminology**: Identity is the public key (pubkey_hex). Four-word networking is used only for
connection words. Some APIs still use legacy field names like `four_words` to carry the identity
value during migration.

**Cargo.toml**:
```toml
[dependencies]
communitas-core = "0.1.54"
```

---

## communitas-core API

### CommunitasApp

Primary entry point for core functionality (command/query API).

```rust
use communitas_core::app::CommunitasApp;
use communitas_core::command::{Command, Query};

let app = CommunitasApp::new(
    "ocean-forest-moon-star".to_string(),
    "Alice".to_string(),
    "MacBook Pro".to_string(),
    "/path/to/storage".to_string(),
).await?;

// Create an entity
app.execute(Command::CreateEntity {
    name: "Engineering".to_string(),
    entity_type: communitas_core::EntityType::Group,
    description: None,
    initial_members: vec![],
}).await?;

// Query profile
let profile = app.query(Query::GetProfile).await?;
```

---

### AuthService

Authentication and session management.

```rust
use communitas_core::{AuthService, SessionInfo};
use communitas_core::encrypted_storage::{EncryptedStorageManager, StorageConfig};

pub struct AuthService {
    // Internal fields
}

impl AuthService {
    /// Create new auth service
    pub fn new(storage_manager: EncryptedStorageManager) -> Self;

    /// Create a new vault
    pub async fn create_vault(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<String>; // Returns vault ID

    /// Login with credentials
    pub async fn login(
        &mut self,
        four_words: &str,
        password: &str,
        device_name: Option<&str>,
    ) -> Result<SessionInfo>;

    /// Logout current session
    pub async fn logout(&mut self) -> Result<()>;

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool;

    /// Get current session
    pub fn get_current_session(&self) -> Option<SessionInfo>;

    /// List all vaults
    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>>;

    /// Delete a vault
    pub async fn delete_vault(
        &mut self,
        four_words: &str,
        password: &str,
    ) -> Result<()>;

    /// Register passkey
    pub async fn passkey_register(
        &mut self,
        four_words: &str,
        device_name: &str,
    ) -> Result<PasskeyInfo>;

    /// Authenticate with passkey
    pub async fn passkey_authenticate(
        &mut self,
        four_words: &str,
    ) -> Result<SessionInfo>;
}
```

**Example**:
```rust
let config = StorageConfig::default();
let storage_manager = EncryptedStorageManager::new(config).await?;
let mut auth_service = AuthService::new(storage_manager);

// Create vault
let vault_id = auth_service.create_vault(
    "ocean-forest-moon-star",
    "secure-password",
    "Alice",
).await?;

// Login
let session = auth_service.login(
    "ocean-forest-moon-star",
    "secure-password",
    Some("MacBook Pro"),
).await?;

println!("Logged in as: {}", session.display_name);
```

---

### Encrypted Storage

Vault management and encrypted storage.

```rust
use communitas_core::encrypted_storage::{
    EncryptedStorageManager,
    StorageConfig,
    Vault,
};

pub struct EncryptedStorageManager {
    // Internal fields
}

impl EncryptedStorageManager {
    /// Create new storage manager
    pub async fn new(config: StorageConfig) -> Result<Self>;

    /// Create a vault
    pub async fn create_vault(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<String>;

    /// Open a vault
    pub async fn open_vault(
        &mut self,
        four_words: &str,
        password: &str,
    ) -> Result<Vault>;

    /// List available vaults
    pub async fn list_vaults(&self) -> Result<Vec<VaultInfo>>;

    /// Delete a vault
    pub async fn delete_vault(
        &mut self,
        four_words: &str,
        password: &str,
    ) -> Result<()>;

    /// Store password in keyring
    pub async fn store_password_in_keyring(
        &self,
        four_words: &str,
        password: &str,
    ) -> Result<()>;

    /// Retrieve password from keyring
    pub async fn get_password_from_keyring(
        &self,
        four_words: &str,
    ) -> Result<String>;
}

pub struct Vault {
    // Internal fields
}

impl Vault {
    /// Read data from vault
    pub async fn read(&self, key: &str) -> Result<Vec<u8>>;

    /// Write data to vault
    pub async fn write(&mut self, key: &str, data: &[u8]) -> Result<()>;

    /// Delete data from vault
    pub async fn delete(&mut self, key: &str) -> Result<()>;

    /// List all keys
    pub async fn list_keys(&self) -> Result<Vec<String>>;
}
```

**Example**:
```rust
let config = StorageConfig {
    vault_dir: PathBuf::from("/path/to/vaults"),
    use_keyring: true,
    ..Default::default()
};

let mut storage_manager = EncryptedStorageManager::new(config).await?;

// Create and open vault
let vault_id = storage_manager.create_vault(
    "ocean-forest-moon-star",
    "password",
    "Alice",
).await?;

let mut vault = storage_manager.open_vault(
    "ocean-forest-moon-star",
    "password",
).await?;

// Store data
vault.write("api-key", b"secret-api-key").await?;

// Read data
let data = vault.read("api-key").await?;
```

---

### Types

Core type definitions.

```rust
use communitas_core::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub four_words: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    pub four_words: String,
    pub display_name: String,
    pub created_at: u64,
    pub last_accessed: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Server,
    Embedded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub members: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: u64,
    pub thread_id: Option<String>,
}
```

---

## saorsa-pqc API

Post-quantum cryptography primitives.

### ML-DSA (Digital Signatures)

```rust
use saorsa_pqc::mldsa::{MlDsa65, KeyPair, Signature};

// Generate keypair
let keypair = KeyPair::generate()?;
let public_key = keypair.public_key();
let secret_key = keypair.secret_key();

// Sign message
let message = b"Hello, world!";
let signature = keypair.sign(message)?;

// Verify signature
let valid = public_key.verify(message, &signature)?;
assert!(valid);
```

### ML-KEM (Key Encapsulation)

```rust
use saorsa_pqc::mlkem::{MlKem768, KeyPair};

// Recipient generates keypair
let recipient_keypair = KeyPair::generate()?;
let recipient_public = recipient_keypair.public_key();

// Sender encapsulates shared secret
let (ciphertext, shared_secret_sender) = recipient_public.encapsulate()?;

// Recipient decapsulates
let shared_secret_recipient = recipient_keypair.decapsulate(&ciphertext)?;

// Shared secrets match
assert_eq!(shared_secret_sender, shared_secret_recipient);
```

### ChaCha20-Poly1305

```rust
use saorsa_pqc::symmetric::{encrypt, decrypt};

let key = [0u8; 32]; // 256-bit key
let plaintext = b"Secret message";

// Encrypt
let ciphertext = encrypt(&key, plaintext)?;

// Decrypt
let decrypted = decrypt(&key, &ciphertext)?;
assert_eq!(decrypted, plaintext);
```

---

## four-word-networking API

Connection word encoding/decoding (IP:port).

```rust
use four_word_networking::{
    encode_socket_addr,
    decode_socket_addr,
    validate_words,
    suggest_corrections,
};

// Encode IP address
let addr = "192.168.1.100:8080".parse()?;
let four_words = encode_socket_addr(&addr);
// → "ocean-forest-moon-star"

// Decode to IP address
let decoded = decode_socket_addr("ocean-forest-moon-star")?;
assert_eq!(decoded, addr);

// Validate connection words
assert!(validate_words("ocean-forest-moon-star"));
assert!(!validate_words("invalid-words-not-real"));

// Get suggestions for typos
let suggestions = suggest_corrections("occean-forest-moon-star");
// → ["ocean-forest-moon-star"]
```

---

## saorsa-gossip-transport QUIC API

Communitas uses `saorsa_gossip_transport::AntQuicTransport` (built on ant-quic).
For advanced QUIC primitives, use the re-exported module `saorsa_gossip_transport::quic`.

```rust
use bytes::Bytes;
use saorsa_gossip_transport::{AntQuicTransport, AntQuicTransportConfig, GossipStreamType};
use std::net::SocketAddr;

let bind_addr: SocketAddr = "0.0.0.0:0".parse()?;
let transport = AntQuicTransport::with_config(
    AntQuicTransportConfig::new(bind_addr, vec![]),
    None,
)
.await?;

// Send data over the pubsub stream
transport
    .send_to_peer(peer_id, GossipStreamType::PubSub, Bytes::from("Hello"))
    .await?;
```

---

## Error Handling

All operations return `Result<T, Error>` where `Error` implements:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Cryptography error: {0}")]
    CryptoError(String),

    #[error("Invalid input: {0}")]
    ValidationError(String),
}
```

**Example**:
```rust
match auth_service.login(four_words, password, None).await {
    Ok(session) => println!("Logged in: {}", session.display_name),
    Err(Error::AuthError(msg)) => eprintln!("Auth failed: {}", msg),
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_vault() {
        let config = StorageConfig::default();
        let mut storage = EncryptedStorageManager::new(config).await.unwrap();

        let vault_id = storage.create_vault(
            "test-vault",
            "password",
            "Test User",
        ).await.unwrap();

        assert!(!vault_id.is_empty());
    }

    #[tokio::test]
    async fn test_login() {
        let mut auth = setup_auth_service().await;

        auth.create_vault("test", "pass", "User").await.unwrap();
        let session = auth.login("test", "pass", None).await.unwrap();

        assert_eq!(session.four_words, "test");
    }
}
```

---

## See Also

- [Flutter FFI API](README.md) - flutter_rust_bridge surface
- [MCP Server](../../communitas-mcp/README.md) - AI agent interface
- [Security Architecture](../architecture/security.md) - Cryptography details
- [crates.io documentation](https://docs.rs/communitas-core)

---

**Core API**: Build custom Rust applications on Communitas. 🦀🔒
