// SPDX-License-Identifier: MIT OR Apache-2.0

// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com
//
// See the LICENSE-AGPL-3.0 and LICENSE-COMMERCIAL.md files for details.

//! Peer Discovery Presence System (ADR-014)
//!
//! Provides presence record types for peer discovery.
//! With x0x integration, PQC signing is handled by the x0x daemon.
//! This module retains the data types for caching and querying.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Errors that can occur in peer presence operations
#[derive(Debug, Error)]
pub enum PresenceError {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ConnectivityState {
    /// Never attempted connection - worth trying
    #[default]
    Unknown,

    /// Successfully connected - known good
    Connected,

    /// Connection failed, but we might be offline ourselves
    FailedMaybeOffline,

    /// Connection failed while we were definitely online
    FailedWhileOnline,
}

impl ConnectivityState {
    /// Check if this state represents a viable record to try
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
#[derive(Debug, Clone)]
pub struct CachedPresence {
    /// The presence record
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

/// Presence record for peer discovery
///
/// With x0x integration, signing is handled by the daemon.
/// This struct stores the relevant presence data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceRecord {
    /// Public key or agent identifier bytes
    pub pubkey: Vec<u8>,

    /// Current connection info (address string or agent ID)
    pub connection_words: String,

    /// Unix timestamp (seconds since epoch) when created
    pub timestamp: u64,

    /// Signature bytes (may be empty when x0x handles signing)
    pub signature: Vec<u8>,
}

impl PresenceRecord {
    /// Create an unsigned presence record
    pub fn new_unsigned(pubkey: Vec<u8>, connection_words: String, timestamp: u64) -> Self {
        Self {
            pubkey,
            connection_words,
            timestamp,
            signature: Vec::new(),
        }
    }

    /// Create a presence record with the current timestamp
    pub fn new_now(pubkey: Vec<u8>, connection_words: String) -> PresenceResult<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| PresenceError::TimeError)?
            .as_secs();

        Ok(Self {
            pubkey,
            connection_words,
            timestamp,
            signature: Vec::new(),
        })
    }

    /// Check if this record is fresher (more recent) than another record
    pub fn is_fresher_than(&self, other: &Self) -> bool {
        self.timestamp > other.timestamp
    }
}

/// Presence query message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceQuery {
    /// Public key of the peer we're looking for
    pub target_pubkey: Vec<u8>,

    /// Address where responses should be sent
    pub reply_to: std::net::SocketAddr,
}

impl PresenceQuery {
    /// Create a new presence query
    pub fn new(target_pubkey: Vec<u8>, reply_to: std::net::SocketAddr) -> Self {
        Self {
            target_pubkey,
            reply_to,
        }
    }
}

/// Presence response message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresenceResponse {
    /// The presence record for the queried peer
    pub record: PresenceRecord,
}

impl PresenceResponse {
    /// Create a new presence response
    pub fn new(record: PresenceRecord) -> Self {
        Self { record }
    }
}

/// Cache for storing received presence records with connectivity tracking
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
    /// for the same pubkey, or if no record exists.
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
    pub fn get(&self, pubkey: &[u8]) -> Option<&CachedPresence> {
        self.records.get(pubkey)
    }

    /// Get a viable cached presence by pubkey
    pub fn get_viable(&self, pubkey: &[u8]) -> Option<&CachedPresence> {
        self.records.get(pubkey).filter(|cp| cp.is_viable())
    }

    /// Mark a peer as successfully connected
    pub fn mark_connected(&mut self, pubkey: &[u8]) {
        if let Some(cached) = self.records.get_mut(pubkey) {
            cached.connectivity = ConnectivityState::Connected;
        }
    }

    /// Mark a peer connection as failed
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
        records.sort_by_key(|record| std::cmp::Reverse(record.record.timestamp));
        records
    }

    /// Get all viable records sorted by freshness (most recent first)
    pub fn get_viable_sorted_by_freshness(&self) -> Vec<&CachedPresence> {
        let mut records: Vec<_> = self.records.values().filter(|cp| cp.is_viable()).collect();
        records.sort_by_key(|record| std::cmp::Reverse(record.record.timestamp));
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

    #[test]
    fn test_presence_record_freshness() {
        let older = PresenceRecord {
            pubkey: vec![1u8; 32],
            connection_words: "old".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        let newer = PresenceRecord {
            pubkey: vec![1u8; 32],
            connection_words: "new".to_string(),
            timestamp: 2000,
            signature: Vec::new(),
        };

        assert!(newer.is_fresher_than(&older));
        assert!(!older.is_fresher_than(&newer));
        assert!(!older.is_fresher_than(&older));
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
            pubkey: vec![1u8; 32],
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        let mut cached = CachedPresence::new(record);
        assert!(cached.is_viable());

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

        let pubkey = vec![1u8; 32];
        let record = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        assert!(cache.insert(record.clone()));
        assert!(cache.contains(&pubkey));
        assert_eq!(cache.get(&pubkey).map(|cp| cp.record.timestamp), Some(1000));
    }

    #[test]
    fn test_presence_cache_fresher_replaces_older() {
        let mut cache = PresenceCache::new();
        let pubkey = vec![1u8; 32];

        let older = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "old".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        let newer = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "new".to_string(),
            timestamp: 2000,
            signature: Vec::new(),
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
        let pubkey = vec![1u8; 32];

        let newer = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "new".to_string(),
            timestamp: 2000,
            signature: Vec::new(),
        };

        let older = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "old".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        assert!(cache.insert(newer));
        assert!(!cache.insert(older));

        let cached = cache.get(&pubkey).expect("should exist");
        assert_eq!(cached.record.connection_words, "new");
    }

    #[test]
    fn test_presence_cache_connectivity_state_transitions() {
        let mut cache = PresenceCache::new();
        let pubkey = vec![1u8; 32];
        let record = PresenceRecord {
            pubkey: pubkey.clone(),
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        cache.insert(record);

        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::Unknown)
        );

        cache.mark_connected(&pubkey);
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::Connected)
        );

        cache.mark_failed(&pubkey, true);
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::FailedWhileOnline)
        );

        cache.mark_failed(&pubkey, false);
        assert_eq!(
            cache.get(&pubkey).map(|cp| cp.connectivity),
            Some(ConnectivityState::FailedMaybeOffline)
        );
    }

    #[test]
    fn test_presence_cache_reset_failed_states() {
        let mut cache = PresenceCache::new();

        for i in 0..4 {
            let record = PresenceRecord {
                pubkey: vec![i; 32],
                connection_words: format!("peer-{}", i),
                timestamp: (i as u64) * 1000,
                signature: Vec::new(),
            };
            cache.insert(record);
        }

        let pk0 = vec![0u8; 32];
        let pk1 = vec![1u8; 32];
        let pk2 = vec![2u8; 32];

        cache.mark_connected(&pk0);
        cache.mark_failed(&pk1, false);
        cache.mark_failed(&pk2, true);

        let (unknown, connected, maybe_offline, while_online) = cache.count_by_state();
        assert_eq!(unknown, 1);
        assert_eq!(connected, 1);
        assert_eq!(maybe_offline, 1);
        assert_eq!(while_online, 1);

        cache.reset_failed_states();

        let (unknown, connected, maybe_offline, while_online) = cache.count_by_state();
        assert_eq!(unknown, 3);
        assert_eq!(connected, 1);
        assert_eq!(maybe_offline, 0);
        assert_eq!(while_online, 0);
    }

    #[test]
    fn test_presence_query_and_response() {
        let pubkey = vec![1u8; 32];
        let reply_to: std::net::SocketAddr = "127.0.0.1:9000".parse().unwrap();

        let query = PresenceQuery::new(pubkey.clone(), reply_to);
        assert_eq!(query.target_pubkey, pubkey);
        assert_eq!(query.reply_to, reply_to);

        let record = PresenceRecord {
            pubkey,
            connection_words: "test".to_string(),
            timestamp: 1000,
            signature: Vec::new(),
        };

        let response = PresenceResponse::new(record.clone());
        assert_eq!(response.record, record);
    }
}
