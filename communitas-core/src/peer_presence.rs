// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Peer Discovery Presence System (ADR-014)
//!
//! Provides network-wide peer discovery via signed presence records.
//! This is distinct from the MLS group-scoped presence in `presence_service.rs`.
//!
//! ## Components
//!
//! - `PresenceRecord`: Signed record of a peer's current location
//! - `CachedPresence`: Record with connectivity state tracking
//! - `ConnectivityState`: Tracks connection attempt outcomes
//! - `PresenceQuery`: Request to find a peer by pubkey
//! - `PresenceResponse`: Response containing presence record
//! - `PresenceCache`: Local cache of known peer locations with connectivity tracking
//!
//! ## Connectivity-Based Freshness Model
//!
//! Rather than using time-based staleness, records are evaluated by connectivity:
//!
//! | State | Meaning | Action |
//! |-------|---------|--------|
//! | Unknown | Never tried | Worth trying |
//! | Connected | Known good | Use it |
//! | FailedMaybeOffline | Failed but we might be offline | Keep, retry later |
//! | FailedWhileOnline | Failed while we're online | Peer moved, need fresh |
//!
//! This approach avoids removing valid records just because time passed, while
//! still ensuring we don't keep stale connection info for peers that have moved.
//!
//! ## Flow
//!
//! 1. Peer announces presence periodically (creates signed PresenceRecord)
//! 2. Other peers query for specific pubkeys via gossip
//! 3. Peers who are connected to target respond with cached records
//! 4. Querier verifies signature and connects to discovered address
//! 5. Connection success/failure updates the ConnectivityState

use saorsa_pqc::dsa_traits::{SerDes, Signer, Verifier};
use saorsa_pqc::ml_dsa_65::{PrivateKey, PublicKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// ML-DSA-65 public key size in bytes
pub const PUBLIC_KEY_SIZE: usize = 1952;

/// ML-DSA-65 private key size in bytes  
pub const PRIVATE_KEY_SIZE: usize = 4032;

/// ML-DSA-65 signature size in bytes
pub const SIGNATURE_SIZE: usize = 3309;

/// Errors that can occur in peer presence operations
#[derive(Debug, Error)]
pub enum PresenceError {
    #[error("Invalid public key size: expected {PUBLIC_KEY_SIZE}, got {0}")]
    InvalidPublicKeySize(usize),

    #[error("Invalid signature size: expected {SIGNATURE_SIZE}, got {0}")]
    InvalidSignatureSize(usize),

    #[error("Failed to parse public key: {0}")]
    PublicKeyParseFailed(String),

    #[error("Failed to parse private key: {0}")]
    PrivateKeyParseFailed(String),

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    #[error("System time error")]
    TimeError,
}

/// Result type for presence operations
pub type PresenceResult<T> = Result<T, PresenceError>;

/// Connectivity state for a cached presence record
///
/// Tracks the outcome of connection attempts to inform retry decisions.
/// This replaces time-based staleness with connection-outcome-based freshness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectivityState {
    /// Never attempted connection - worth trying
    #[default]
    Unknown,

    /// Successfully connected - known good
    Connected,

    /// Connection failed, but we might be offline ourselves
    /// Keep the record, retry when we know we're online
    FailedMaybeOffline,

    /// Connection failed while we were definitely online
    /// Peer has likely moved, need fresh presence info
    FailedWhileOnline,
}

impl ConnectivityState {
    /// Check if this state represents a viable record to try
    ///
    /// Records are viable if we haven't proven they're stale.
    pub fn is_viable(self) -> bool {
        matches!(
            self,
            ConnectivityState::Unknown
                | ConnectivityState::Connected
                | ConnectivityState::FailedMaybeOffline
        )
    }
}

/// Cached presence record with connectivity tracking
///
/// Wraps a `PresenceRecord` with connectivity state to track
/// connection attempt outcomes without modifying the signed record.
#[derive(Debug, Clone)]
pub struct CachedPresence {
    /// The signed presence record
    pub record: PresenceRecord,

    /// Current connectivity state based on connection attempts
    pub connectivity: ConnectivityState,
}

impl CachedPresence {
    /// Create a new cached presence with Unknown connectivity
    pub fn new(record: PresenceRecord) -> Self {
        Self {
            record,
            connectivity: ConnectivityState::Unknown,
        }
    }

    /// Check if this cached presence is viable for connection attempts
    pub fn is_viable(&self) -> bool {
        self.connectivity.is_viable()
    }
}

/// Signed presence record for peer discovery
///
/// Contains the information needed to locate and verify a peer's identity.
/// The signature proves the record was created by the pubkey owner.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceRecord {
    /// ML-DSA-65 public key (permanent identity, 1952 bytes)
    pub pubkey: Vec<u8>,

    /// Current IP:port as connection words (ephemeral)
    /// Encoded using FourWordAdaptiveEncoder
    pub connection_words: String,

    /// Unix timestamp (seconds since epoch) when created
    pub timestamp: u64,

    /// ML-DSA-65 signature over pubkey||connection_words||timestamp (3309 bytes)
    pub signature: Vec<u8>,
}

impl PresenceRecord {
    /// Create and sign a new presence record
    ///
    /// # Arguments
    /// * `public_key` - The ML-DSA-65 public key
    /// * `private_key` - The ML-DSA-65 private key for signing
    /// * `connection_words` - Current connection address as four words
    ///
    /// # Returns
    /// Signed presence record or error
    pub fn new(
        public_key: &PublicKey,
        private_key: &PrivateKey,
        connection_words: String,
    ) -> PresenceResult<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| PresenceError::TimeError)?
            .as_secs();

        let pubkey_bytes = public_key.clone().into_bytes().to_vec();

        let mut record = Self {
            pubkey: pubkey_bytes,
            connection_words,
            timestamp,
            signature: Vec::new(),
        };

        record.sign(private_key)?;
        Ok(record)
    }

    /// Create an unsigned presence record (for deserialization)
    ///
    /// Note: This should only be used when deserializing a record
    /// that will be verified separately.
    pub fn new_unsigned(pubkey: Vec<u8>, connection_words: String, timestamp: u64) -> Self {
        Self {
            pubkey,
            connection_words,
            timestamp,
            signature: Vec::new(),
        }
    }

    /// Get the canonical bytes for signing/verification
    fn to_sign_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.pubkey.len() + self.connection_words.len() + 8);
        bytes.extend_from_slice(&self.pubkey);
        bytes.extend_from_slice(self.connection_words.as_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes
    }

    /// Sign the presence record with the given private key
    fn sign(&mut self, private_key: &PrivateKey) -> PresenceResult<()> {
        let message = self.to_sign_bytes();
        let signature = private_key
            .try_sign(&message, &[])
            .map_err(|e| PresenceError::SigningFailed(e.to_string()))?;

        self.signature = signature.to_vec();
        Ok(())
    }

    /// Verify the signature is valid for this record
    ///
    /// # Returns
    /// `Ok(true)` if signature is valid, `Ok(false)` or `Err` otherwise
    pub fn verify(&self) -> PresenceResult<bool> {
        // Validate pubkey size
        if self.pubkey.len() != PUBLIC_KEY_SIZE {
            return Err(PresenceError::InvalidPublicKeySize(self.pubkey.len()));
        }

        // Validate signature size
        if self.signature.len() != SIGNATURE_SIZE {
            return Err(PresenceError::InvalidSignatureSize(self.signature.len()));
        }

        // Parse public key
        let pk_array: [u8; PUBLIC_KEY_SIZE] = self
            .pubkey
            .as_slice()
            .try_into()
            .map_err(|_| PresenceError::InvalidPublicKeySize(self.pubkey.len()))?;

        let public_key = PublicKey::try_from_bytes(pk_array)
            .map_err(|e| PresenceError::PublicKeyParseFailed(e.to_string()))?;

        // Parse signature
        let sig_array: [u8; SIGNATURE_SIZE] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| PresenceError::InvalidSignatureSize(self.signature.len()))?;

        // Verify
        let message = self.to_sign_bytes();
        let is_valid = public_key.verify(&message, &sig_array, &[]);

        Ok(is_valid)
    }

    /// Check if this record is fresher (more recent) than another record
    ///
    /// Records for the same pubkey should prefer the fresher one.
    /// Used for deduplication when multiple records exist for the same peer.
    pub fn is_fresher_than(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp
    }
}

/// Presence query message sent via gossip
///
/// Used to ask the network if anyone knows the current location of a peer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceQuery {
    /// Public key of the peer we're looking for
    pub target_pubkey: Vec<u8>,

    /// Address where responses should be sent
    pub reply_to: SocketAddr,
}

impl PresenceQuery {
    /// Create a new presence query
    pub fn new(target_pubkey: Vec<u8>, reply_to: SocketAddr) -> Self {
        Self {
            target_pubkey,
            reply_to,
        }
    }
}

/// Presence response message
///
/// Sent in reply to a PresenceQuery, containing the known presence record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceResponse {
    /// The signed presence record for the queried peer
    pub record: PresenceRecord,
}

impl PresenceResponse {
    /// Create a new presence response
    pub fn new(record: PresenceRecord) -> Self {
        Self { record }
    }

    /// Verify the contained record's signature
    pub fn verify(&self) -> PresenceResult<bool> {
        self.record.verify()
    }
}

/// Cache for storing received presence records with connectivity tracking
///
/// Maintains the most recent presence record for each known peer,
/// along with connectivity state based on connection attempt outcomes.
/// Records are indexed by public key bytes for efficient lookup.
#[derive(Debug, Default)]
pub struct PresenceCache {
    /// Map from pubkey bytes to cached presence with connectivity state
    records: HashMap<Vec<u8>, CachedPresence>,
}

impl PresenceCache {
    /// Create a new empty presence cache
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    /// Insert a presence record into the cache
    ///
    /// Only inserts if the record is fresher than any existing record
    /// for the same pubkey, or if no record exists. New records start
    /// with Unknown connectivity state.
    ///
    /// # Arguments
    /// * `record` - The presence record to insert
    ///
    /// # Returns
    /// `true` if the record was inserted (fresher or new), `false` if rejected
    pub fn insert(&mut self, record: PresenceRecord) -> bool {
        match self.records.get(&record.pubkey) {
            Some(existing) if !record.is_fresher_than(&existing.record) => false,
            _ => {
                self.records
                    .insert(record.pubkey.clone(), CachedPresence::new(record));
                true
            }
        }
    }

    /// Get a cached presence by pubkey
    ///
    /// # Arguments
    /// * `pubkey` - The public key bytes to look up
    ///
    /// # Returns
    /// Reference to the cached presence if found, None otherwise
    pub fn get(&self, pubkey: &[u8]) -> Option<&CachedPresence> {
        self.records.get(pubkey)
    }

    /// Get a viable cached presence by pubkey
    ///
    /// Returns the cached presence only if its connectivity state
    /// indicates it's worth trying (not FailedWhileOnline).
    ///
    /// # Arguments
    /// * `pubkey` - The public key bytes to look up
    ///
    /// # Returns
    /// Reference to the cached presence if found and viable, None otherwise
    pub fn get_viable(&self, pubkey: &[u8]) -> Option<&CachedPresence> {
        self.records.get(pubkey).filter(|cp| cp.is_viable())
    }

    /// Mark a peer as successfully connected
    ///
    /// Updates the connectivity state to Connected, indicating the
    /// presence info is known good.
    ///
    /// # Arguments
    /// * `pubkey` - The public key bytes of the connected peer
    pub fn mark_connected(&mut self, pubkey: &[u8]) {
        if let Some(cached) = self.records.get_mut(pubkey) {
            cached.connectivity = ConnectivityState::Connected;
        }
    }

    /// Mark a peer connection as failed
    ///
    /// Updates the connectivity state based on whether we believe
    /// we're currently online.
    ///
    /// # Arguments
    /// * `pubkey` - The public key bytes of the peer
    /// * `we_are_online` - Whether we believe we're online (have other connections)
    pub fn mark_failed(&mut self, pubkey: &[u8], we_are_online: bool) {
        if let Some(cached) = self.records.get_mut(pubkey) {
            cached.connectivity = if we_are_online {
                ConnectivityState::FailedWhileOnline
            } else {
                ConnectivityState::FailedMaybeOffline
            };
        }
    }

    /// Reset all failed states to Unknown
    ///
    /// Called when network conditions change (e.g., we come back online)
    /// to allow retrying previously failed connections.
    pub fn reset_failed_states(&mut self) {
        for cached in self.records.values_mut() {
            if matches!(
                cached.connectivity,
                ConnectivityState::FailedMaybeOffline | ConnectivityState::FailedWhileOnline
            ) {
                cached.connectivity = ConnectivityState::Unknown;
            }
        }
    }

    /// Check if a record exists for the given pubkey
    pub fn contains(&self, pubkey: &[u8]) -> bool {
        self.records.contains_key(pubkey)
    }

    /// Remove a specific record by pubkey
    pub fn remove(&mut self, pubkey: &[u8]) -> Option<CachedPresence> {
        self.records.remove(pubkey)
    }

    /// Get the number of records in the cache
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Get all records sorted by freshness (most recent first)
    pub fn get_all_sorted_by_freshness(&self) -> Vec<&CachedPresence> {
        let mut records: Vec<_> = self.records.values().collect();
        records.sort_by(|a, b| b.record.timestamp.cmp(&a.record.timestamp));
        records
    }

    /// Get all viable records sorted by freshness (most recent first)
    pub fn get_viable_sorted_by_freshness(&self) -> Vec<&CachedPresence> {
        let mut records: Vec<_> = self.records.values().filter(|cp| cp.is_viable()).collect();
        records.sort_by(|a, b| b.record.timestamp.cmp(&a.record.timestamp));
        records
    }

    /// Get all pubkeys in the cache
    pub fn pubkeys(&self) -> Vec<&Vec<u8>> {
        self.records.keys().collect()
    }

    /// Clear all records from the cache
    pub fn clear(&mut self) {
        self.records.clear();
    }

    /// Count records by connectivity state
    pub fn count_by_state(&self) -> (usize, usize, usize, usize) {
        let mut unknown = 0;
        let mut connected = 0;
        let mut failed_maybe_offline = 0;
        let mut failed_while_online = 0;

        for cached in self.records.values() {
            match cached.connectivity {
                ConnectivityState::Unknown => unknown += 1,
                ConnectivityState::Connected => connected += 1,
                ConnectivityState::FailedMaybeOffline => failed_maybe_offline += 1,
                ConnectivityState::FailedWhileOnline => failed_while_online += 1,
            }
        }

        (
            unknown,
            connected,
            failed_maybe_offline,
            failed_while_online,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_gossip_identity::MlDsaKeyPair;

    fn create_test_keypair() -> (PublicKey, PrivateKey) {
        let keypair = MlDsaKeyPair::generate().expect("keypair generation");

        let pk_bytes = keypair.public_key();
        let pk_array: [u8; PUBLIC_KEY_SIZE] = pk_bytes.try_into().expect("pubkey size");
        let public_key = PublicKey::try_from_bytes(pk_array).expect("parse pubkey");

        let sk_typed = keypair.get_secret_key_typed().expect("get secret key");
        let sk_bytes = sk_typed.as_bytes();
        let sk_array: [u8; PRIVATE_KEY_SIZE] = sk_bytes.try_into().expect("privkey size");
        let private_key = PrivateKey::try_from_bytes(sk_array).expect("parse privkey");

        (public_key, private_key)
    }

    #[test]
    fn test_presence_record_create_and_verify() {
        let (pk, sk) = create_test_keypair();
        let connection_words = "alpha bravo charlie delta".to_string();

        let record =
            PresenceRecord::new(&pk, &sk, connection_words.clone()).expect("create record");

        assert_eq!(record.connection_words, connection_words);
        assert_eq!(record.pubkey.len(), PUBLIC_KEY_SIZE);
        assert_eq!(record.signature.len(), SIGNATURE_SIZE);
        assert!(record.timestamp > 0);

        // Verify signature
        let is_valid = record.verify().expect("verify");
        assert!(is_valid, "Signature should be valid");
    }

    #[test]
    fn test_presence_record_tampered_signature_fails() {
        let (pk, sk) = create_test_keypair();
        let mut record =
            PresenceRecord::new(&pk, &sk, "test words".to_string()).expect("create record");

        // Tamper with the connection words
        record.connection_words = "tampered words".to_string();

        let is_valid = record.verify().expect("verify should complete");
        assert!(!is_valid, "Tampered record should fail verification");
    }

    #[test]
    fn test_presence_record_freshness() {
        let (pk, _sk) = create_test_keypair();

        let older = PresenceRecord {
            pubkey: pk.clone().into_bytes().to_vec(),
            connection_words: "old".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        let newer = PresenceRecord {
            pubkey: pk.into_bytes().to_vec(),
            connection_words: "new".to_string(),
            timestamp: 2000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        assert!(newer.is_fresher_than(&older));
        assert!(!older.is_fresher_than(&newer));
        assert!(!older.is_fresher_than(&older)); // Same timestamp
    }

    #[test]
    fn test_connectivity_state_viability() {
        assert!(ConnectivityState::Unknown.is_viable());
        assert!(ConnectivityState::Connected.is_viable());
        assert!(ConnectivityState::FailedMaybeOffline.is_viable());
        assert!(!ConnectivityState::FailedWhileOnline.is_viable());
    }

    #[test]
    fn test_cached_presence_viability() {
        let record = PresenceRecord {
            pubkey: vec![1u8; PUBLIC_KEY_SIZE],
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        let mut cached = CachedPresence::new(record);
        assert!(cached.is_viable()); // Unknown is viable

        cached.connectivity = ConnectivityState::Connected;
        assert!(cached.is_viable());

        cached.connectivity = ConnectivityState::FailedMaybeOffline;
        assert!(cached.is_viable());

        cached.connectivity = ConnectivityState::FailedWhileOnline;
        assert!(!cached.is_viable());
    }

    #[test]
    fn test_presence_cache_insert_and_get() {
        let mut cache = PresenceCache::new();

        let pubkey = vec![1u8; PUBLIC_KEY_SIZE];
        let record = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        assert!(cache.insert(record.clone()));
        assert!(cache.contains(&pubkey));
        assert_eq!(cache.get(&pubkey).map(|cp| cp.record.timestamp), Some(1000));
    }

    #[test]
    fn test_presence_cache_fresher_replaces_older() {
        let mut cache = PresenceCache::new();

        let pubkey = vec![1u8; PUBLIC_KEY_SIZE];

        let older = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "old".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        let newer = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "new".to_string(),
            timestamp: 2000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        assert!(cache.insert(older));
        assert!(cache.insert(newer));

        let cached = cache.get(&pubkey).expect("should exist");
        assert_eq!(cached.record.connection_words, "new");
        assert_eq!(cached.record.timestamp, 2000);
    }

    #[test]
    fn test_presence_cache_older_rejected() {
        let mut cache = PresenceCache::new();

        let pubkey = vec![1u8; PUBLIC_KEY_SIZE];

        let newer = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "new".to_string(),
            timestamp: 2000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        let older = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "old".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        assert!(cache.insert(newer));
        assert!(!cache.insert(older)); // Should be rejected

        let cached = cache.get(&pubkey).expect("should exist");
        assert_eq!(cached.record.connection_words, "new");
    }

    #[test]
    fn test_presence_cache_connectivity_state_transitions() {
        let mut cache = PresenceCache::new();

        let pubkey = vec![1u8; PUBLIC_KEY_SIZE];
        let record = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        cache.insert(record);

        // Initial state is Unknown
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::Unknown)
        );

        // Mark as connected
        cache.mark_connected(&pubkey);
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::Connected)
        );

        // Mark as failed while online
        cache.mark_failed(&pubkey, true);
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::FailedWhileOnline)
        );

        // Mark as failed while maybe offline
        cache.mark_failed(&pubkey, false);
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::FailedMaybeOffline)
        );
    }

    #[test]
    fn test_presence_cache_get_viable() {
        let mut cache = PresenceCache::new();

        let pubkey = vec![1u8; PUBLIC_KEY_SIZE];
        let record = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        cache.insert(record);

        // Unknown is viable
        assert!(cache.get_viable(&pubkey).is_some());

        // Connected is viable
        cache.mark_connected(&pubkey);
        assert!(cache.get_viable(&pubkey).is_some());

        // FailedMaybeOffline is viable
        cache.mark_failed(&pubkey, false);
        assert!(cache.get_viable(&pubkey).is_some());

        // FailedWhileOnline is NOT viable
        cache.mark_failed(&pubkey, true);
        assert!(cache.get_viable(&pubkey).is_none());
        // But get() still returns it
        assert!(cache.get(&pubkey).is_some());
    }

    #[test]
    fn test_presence_cache_reset_failed_states() {
        let mut cache = PresenceCache::new();

        // Add multiple records with different states
        for i in 0..4 {
            let record = PresenceRecord {
                pubkey: vec![i; PUBLIC_KEY_SIZE],
                connection_words: format!("peer-{}", i),
                timestamp: (i as u64) * 1000,
                signature: vec![0; SIGNATURE_SIZE],
            };
            cache.insert(record);
        }

        let pk0 = vec![0u8; PUBLIC_KEY_SIZE];
        let pk1 = vec![1u8; PUBLIC_KEY_SIZE];
        let pk2 = vec![2u8; PUBLIC_KEY_SIZE];
        let _pk3 = vec![3u8; PUBLIC_KEY_SIZE];

        // Set various states
        cache.mark_connected(&pk0);
        cache.mark_failed(&pk1, false); // FailedMaybeOffline
        cache.mark_failed(&pk2, true); // FailedWhileOnline
        // pk3 stays Unknown

        let (unknown, connected, maybe_offline, while_online) = cache.count_by_state();
        assert_eq!(unknown, 1);
        assert_eq!(connected, 1);
        assert_eq!(maybe_offline, 1);
        assert_eq!(while_online, 1);

        // Reset failed states
        cache.reset_failed_states();

        let (unknown, connected, maybe_offline, while_online) = cache.count_by_state();
        assert_eq!(unknown, 3); // pk1, pk2, pk3 all Unknown now
        assert_eq!(connected, 1); // pk0 stays Connected
        assert_eq!(maybe_offline, 0);
        assert_eq!(while_online, 0);
    }

    #[test]
    fn test_presence_query_and_response() {
        let pubkey = vec![1u8; PUBLIC_KEY_SIZE];
        let reply_to: SocketAddr = "127.0.0.1:9000".parse().unwrap();

        let query = PresenceQuery::new(pubkey.clone(), reply_to);
        assert_eq!(query.target_pubkey, pubkey);
        assert_eq!(query.reply_to, reply_to);

        let record = PresenceRecord {
            pubkey,
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: vec![0; SIGNATURE_SIZE],
        };

        let response = PresenceResponse::new(record.clone());
        assert_eq!(response.record, record);
    }

    #[test]
    fn test_presence_cache_sorted_by_freshness() {
        let mut cache = PresenceCache::new();

        for i in 0..5 {
            let record = PresenceRecord {
                pubkey: vec![i; PUBLIC_KEY_SIZE],
                connection_words: format!("peer-{}", i),
                timestamp: (i as u64) * 1000,
                signature: vec![0; SIGNATURE_SIZE],
            };
            cache.insert(record);
        }

        let sorted = cache.get_all_sorted_by_freshness();
        assert_eq!(sorted.len(), 5);

        // Most recent should be first
        assert_eq!(sorted[0].record.timestamp, 4000);
        assert_eq!(sorted[4].record.timestamp, 0);
    }

    #[test]
    fn test_presence_cache_get_viable_sorted_by_freshness() {
        let mut cache = PresenceCache::new();

        for i in 0..5 {
            let record = PresenceRecord {
                pubkey: vec![i; PUBLIC_KEY_SIZE],
                connection_words: format!("peer-{}", i),
                timestamp: (i as u64) * 1000,
                signature: vec![0; SIGNATURE_SIZE],
            };
            cache.insert(record);
        }

        // Mark peer-2 as FailedWhileOnline (not viable)
        let pk2 = vec![2u8; PUBLIC_KEY_SIZE];
        cache.mark_failed(&pk2, true);

        let viable = cache.get_viable_sorted_by_freshness();
        assert_eq!(viable.len(), 4); // One less than total

        // Should not contain peer-2
        assert!(
            viable
                .iter()
                .all(|cp| cp.record.connection_words != "peer-2")
        );
    }
}
