// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! CRDT Operations Primitives
//!
//! Implements the fundamental CRDT operation types:
//! - Last-Write-Wins (LWW) for scalar values
//! - Counter for concurrent increments
//! - Set operations (add/remove) for collections
//! - Tombstone handling for soft deletes

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yrs::{Map, MapRef, ReadTxn, TransactionMut};

/// Last-Write-Wins (LWW) timestamp for conflict resolution
///
/// When two peers concurrently update the same field, the update with
/// the higher timestamp wins. If timestamps are equal, use a tie-breaker
/// (e.g., peer ID) for deterministic resolution.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct LamportTimestamp {
    /// Logical timestamp (milliseconds since epoch or logical counter)
    pub timestamp: i64,
    /// Tie-breaker: peer ID hash or counter
    pub tie_breaker: u64,
}

impl LamportTimestamp {
    /// Create a new timestamp with current system time
    pub fn now(peer_id: &str) -> Self {
        use std::time::SystemTime;
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);

        // Use deterministic hash of peer ID as tie-breaker to ensure convergence across replicas
        let hash = blake3::hash(peer_id.as_bytes());
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(&hash.as_bytes()[..8]);
        let tie_breaker = u64::from_le_bytes(bytes);

        Self {
            timestamp,
            tie_breaker,
        }
    }

    /// Check if this timestamp wins over another
    pub fn wins_over(&self, other: &Self) -> bool {
        self > other
    }
}

/// Last-Write-Wins Register
///
/// Stores a value with its associated timestamp. Updates replace the value
/// only if the new timestamp is greater.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LWWRegister<T> {
    pub value: T,
    pub timestamp: LamportTimestamp,
}

impl<T> LWWRegister<T> {
    pub fn new(value: T, timestamp: LamportTimestamp) -> Self {
        Self { value, timestamp }
    }

    /// Update the register if the new timestamp is greater
    pub fn update(&mut self, new_value: T, new_timestamp: LamportTimestamp) -> bool {
        if new_timestamp.wins_over(&self.timestamp) {
            self.value = new_value;
            self.timestamp = new_timestamp;
            true
        } else {
            false
        }
    }

    /// Merge with another register, keeping the value with higher timestamp
    pub fn merge(&mut self, other: &Self)
    where
        T: Clone,
    {
        if other.timestamp.wins_over(&self.timestamp) {
            self.value = other.value.clone();
            self.timestamp = other.timestamp;
        }
    }
}

/// CRDT Counter (G-Counter - Grow-only Counter)
///
/// Each peer has its own counter. The total is the sum of all peer counters.
/// This ensures concurrent increments are never lost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counter {
    /// Map of peer ID to their counter value
    counts: HashMap<String, i64>,
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

impl Counter {
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Increment the counter for a specific peer
    pub fn increment(&mut self, peer_id: &str, amount: i64) {
        *self.counts.entry(peer_id.to_string()).or_insert(0) += amount;
    }

    /// Get the total count across all peers
    pub fn value(&self) -> i64 {
        self.counts.values().sum()
    }

    /// Merge with another counter (take max for each peer)
    pub fn merge(&mut self, other: &Self) {
        for (peer, count) in &other.counts {
            let entry = self.counts.entry(peer.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
    }

    /// Get the counter map (for serialization)
    pub fn counts(&self) -> &HashMap<String, i64> {
        &self.counts
    }
}

/// CRDT Set Operations (OR-Set - Observed-Remove Set)
///
/// Each element has a unique identifier (UUID). Elements can be added and removed.
/// An element exists if it was added and not removed with the same ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetOperation {
    /// Element identifier (e.g., user ID, channel ID)
    pub element_id: String,
    /// Operation type
    pub op_type: SetOpType,
    /// Unique operation ID (UUID) to distinguish add/remove operations
    pub operation_id: String,
    /// Timestamp for ordering
    pub timestamp: LamportTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SetOpType {
    Add,
    Remove,
}

/// OR-Set (Observed-Remove Set)
///
/// Tracks add and remove operations. An element is in the set if it has
/// been added with an operation ID that hasn't been removed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORSet {
    /// Map of element ID -> set of (add_operation_id, timestamp)
    adds: HashMap<String, Vec<(String, LamportTimestamp)>>,
    /// Map of element ID -> set of (remove_operation_id, timestamp)
    removes: HashMap<String, Vec<(String, LamportTimestamp)>>,
}

impl Default for ORSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ORSet {
    pub fn new() -> Self {
        Self {
            adds: HashMap::new(),
            removes: HashMap::new(),
        }
    }

    /// Add an element with a unique operation ID
    pub fn add(&mut self, element_id: String, operation_id: String, timestamp: LamportTimestamp) {
        self.adds
            .entry(element_id)
            .or_default()
            .push((operation_id, timestamp));
    }

    /// Remove an element (removes all add operations for this element)
    pub fn remove(
        &mut self,
        element_id: String,
        _operation_id: String,
        timestamp: LamportTimestamp,
    ) {
        // Get all current add operation IDs for this element
        if let Some(adds) = self.adds.get(&element_id) {
            let add_ids: Vec<String> = adds.iter().map(|(id, _)| id.clone()).collect();
            for add_id in add_ids {
                self.removes
                    .entry(element_id.clone())
                    .or_default()
                    .push((add_id, timestamp));
            }
        }
    }

    /// Check if an element is in the set
    /// (has an add operation not matched by a remove)
    pub fn contains(&self, element_id: &str) -> bool {
        if let Some(adds) = self.adds.get(element_id) {
            if let Some(removes) = self.removes.get(element_id) {
                // Element is in set if there's an add operation not in removes
                adds.iter()
                    .any(|(add_id, _)| !removes.iter().any(|(rem_id, _)| rem_id == add_id))
            } else {
                // No removes, so element is in set if it has adds
                !adds.is_empty()
            }
        } else {
            false
        }
    }

    /// Get all elements in the set
    pub fn elements(&self) -> Vec<String> {
        self.adds
            .keys()
            .filter(|k| self.contains(k))
            .cloned()
            .collect()
    }

    /// Merge with another OR-Set
    pub fn merge(&mut self, other: &Self) {
        // Merge adds
        for (element, ops) in &other.adds {
            self.adds
                .entry(element.clone())
                .or_default()
                .extend(ops.clone());
        }

        // Merge removes
        for (element, ops) in &other.removes {
            self.removes
                .entry(element.clone())
                .or_default()
                .extend(ops.clone());
        }
    }
}

/// Tombstone for soft-delete semantics
///
/// Used to mark entities as deleted without removing them from the CRDT.
/// The tombstone timestamp is treated with LWW semantics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tombstone {
    pub deleted_at: Option<LamportTimestamp>,
}

impl Default for Tombstone {
    fn default() -> Self {
        Self::new()
    }
}

impl Tombstone {
    pub fn new() -> Self {
        Self { deleted_at: None }
    }

    pub fn delete(&mut self, timestamp: LamportTimestamp) {
        if let Some(existing) = self.deleted_at {
            if timestamp.wins_over(&existing) {
                self.deleted_at = Some(timestamp);
            }
        } else {
            self.deleted_at = Some(timestamp);
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn merge(&mut self, other: &Self) {
        match (self.deleted_at, other.deleted_at) {
            (Some(a), Some(b)) => {
                self.deleted_at = Some(if b.wins_over(&a) { b } else { a });
            }
            (None, Some(b)) => {
                self.deleted_at = Some(b);
            }
            _ => {}
        }
    }
}

/// Helper functions for working with CRDT operations in Yrs documents
/// Set an LWW value in a Yrs map
pub fn set_lww_value(
    txn: &mut TransactionMut,
    map: &MapRef,
    key: &str,
    value: impl Into<String>,
    timestamp: LamportTimestamp,
) -> Result<()> {
    // Store value with timestamp as key suffix
    let value_key = format!("{}_value", key);
    let timestamp_key = format!("{}_timestamp", key);
    let tie_breaker_key = format!("{}_tie_breaker", key);

    map.insert(txn, value_key.as_str(), value.into());
    map.insert(txn, timestamp_key.as_str(), timestamp.timestamp);
    map.insert(txn, tie_breaker_key.as_str(), timestamp.tie_breaker as i64);

    Ok(())
}

/// Get an LWW value from a Yrs map
pub fn get_lww_value(
    txn: &impl ReadTxn,
    map: &MapRef,
    key: &str,
) -> Result<Option<(String, LamportTimestamp)>> {
    let value_key = format!("{}_value", key);
    let timestamp_key = format!("{}_timestamp", key);
    let tie_breaker_key = format!("{}_tie_breaker", key);

    if let Some(value) = map.get(txn, &value_key) {
        let timestamp = map
            .get(txn, &timestamp_key)
            .and_then(|v| i64::try_from(v).ok())
            .unwrap_or(0);
        let tie_breaker = map
            .get(txn, &tie_breaker_key)
            .and_then(|v| i64::try_from(v).ok())
            .unwrap_or(0) as u64;

        let value_str = String::try_from(value).unwrap_or_default();

        Ok(Some((
            value_str,
            LamportTimestamp {
                timestamp,
                tie_breaker,
            },
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamport_timestamp_ordering() {
        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };
        let t2 = LamportTimestamp {
            timestamp: 200,
            tie_breaker: 1,
        };
        let t3 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 2,
        };

        assert!(t2.wins_over(&t1));
        assert!(!t1.wins_over(&t2));
        assert!(t3.wins_over(&t1));
        assert!(!t1.wins_over(&t3));
    }

    #[test]
    fn test_lww_register() {
        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };
        let t2 = LamportTimestamp {
            timestamp: 200,
            tie_breaker: 1,
        };

        let mut register = LWWRegister::new("value1".to_string(), t1);
        assert_eq!(register.value, "value1");

        // Update with newer timestamp succeeds
        assert!(register.update("value2".to_string(), t2));
        assert_eq!(register.value, "value2");

        // Update with older timestamp fails
        assert!(!register.update("value3".to_string(), t1));
        assert_eq!(register.value, "value2");
    }

    #[test]
    fn test_counter() {
        let mut counter = Counter::new();

        counter.increment("peer1", 1);
        assert_eq!(counter.value(), 1);

        counter.increment("peer2", 2);
        assert_eq!(counter.value(), 3);

        counter.increment("peer1", 1);
        assert_eq!(counter.value(), 4);

        // Test merge
        let mut counter2 = Counter::new();
        counter2.increment("peer1", 5);
        counter2.increment("peer3", 3);

        counter.merge(&counter2);
        assert_eq!(counter.value(), 10); // max(2, 5) + 2 + 3 = 5 + 2 + 3 = 10
    }

    #[test]
    fn test_or_set() {
        let mut set = ORSet::new();
        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };

        // Add element
        set.add("user1".to_string(), "op1".to_string(), t1);
        assert!(set.contains("user1"));

        // Remove element
        set.remove("user1".to_string(), "op2".to_string(), t1);
        assert!(!set.contains("user1"));

        // Add again with different operation ID
        set.add("user1".to_string(), "op3".to_string(), t1);
        assert!(set.contains("user1"));
    }

    #[test]
    fn test_tombstone() {
        let mut tombstone = Tombstone::new();
        assert!(!tombstone.is_deleted());

        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };
        tombstone.delete(t1);
        assert!(tombstone.is_deleted());

        // Older delete doesn't override
        let t0 = LamportTimestamp {
            timestamp: 50,
            tie_breaker: 1,
        };
        tombstone.delete(t0);
        assert_eq!(tombstone.deleted_at, Some(t1));

        // Newer delete overrides
        let t2 = LamportTimestamp {
            timestamp: 200,
            tie_breaker: 1,
        };
        tombstone.delete(t2);
        assert_eq!(tombstone.deleted_at, Some(t2));
    }
}
