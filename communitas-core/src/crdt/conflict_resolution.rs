// Copyright (c) 2025 Saorsa Labs Limited
//
// Licensed under the AGPL-3.0 license

//! Conflict Resolution Strategies
//!
//! Implements various conflict resolution strategies for different data types:
//! - Last-Write-Wins (LWW) for metadata fields
//! - CRDT Set merge for collections
//! - Counter merge for numeric aggregates
//! - State machine validation for state transitions

use super::operations::{Counter, LWWRegister, LamportTimestamp, ORSet};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Conflict resolution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution<T> {
    /// No conflict, use the local value
    NoConflict(T),
    /// Conflict resolved, use this value
    Resolved(T),
    /// Conflict requires manual resolution
    RequiresManual { local: T, remote: T, reason: String },
}

/// Conflict resolver trait
pub trait ConflictResolver<T> {
    fn resolve(&self, local: &T, remote: &T) -> ConflictResolution<T>;
}

/// LWW conflict resolver
pub struct LWWResolver;

impl<T: Clone> ConflictResolver<LWWRegister<T>> for LWWResolver {
    fn resolve(
        &self,
        local: &LWWRegister<T>,
        remote: &LWWRegister<T>,
    ) -> ConflictResolution<LWWRegister<T>> {
        if remote.timestamp.wins_over(&local.timestamp) {
            ConflictResolution::Resolved(remote.clone())
        } else {
            ConflictResolution::NoConflict(local.clone())
        }
    }
}

/// Counter conflict resolver (merge by taking max per peer)
pub struct CounterResolver;

impl ConflictResolver<Counter> for CounterResolver {
    fn resolve(&self, local: &Counter, remote: &Counter) -> ConflictResolution<Counter> {
        let mut merged = local.clone();
        merged.merge(remote);
        ConflictResolution::Resolved(merged)
    }
}

/// OR-Set conflict resolver
pub struct ORSetResolver;

impl ConflictResolver<ORSet> for ORSetResolver {
    fn resolve(&self, local: &ORSet, remote: &ORSet) -> ConflictResolution<ORSet> {
        let mut merged = local.clone();
        merged.merge(remote);
        ConflictResolution::Resolved(merged)
    }
}

/// State machine for valid state transitions
///
/// Used for entities with state (e.g., issues, calls) to ensure
/// only valid transitions occur during conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    pub current_state: String,
    pub timestamp: LamportTimestamp,
    valid_transitions: Vec<(String, Vec<String>)>, // (from_state, to_states)
}

impl StateMachine {
    pub fn new(initial_state: String, valid_transitions: Vec<(String, Vec<String>)>) -> Self {
        Self {
            current_state: initial_state,
            timestamp: LamportTimestamp {
                timestamp: 0,
                tie_breaker: 0,
            },
            valid_transitions,
        }
    }

    /// Check if a transition is valid
    pub fn is_valid_transition(&self, from: &str, to: &str) -> bool {
        for (from_state, to_states) in &self.valid_transitions {
            if from_state == from {
                return to_states.contains(&to.to_string());
            }
        }
        false
    }

    /// Attempt to transition to a new state
    pub fn transition(&mut self, new_state: String, timestamp: LamportTimestamp) -> Result<()> {
        if !self.is_valid_transition(&self.current_state, &new_state) {
            return Err(anyhow!(
                "Invalid state transition from {} to {}",
                self.current_state,
                new_state
            ));
        }

        if timestamp.wins_over(&self.timestamp) {
            self.current_state = new_state;
            self.timestamp = timestamp;
            Ok(())
        } else {
            Err(anyhow!("Timestamp not newer than current state"))
        }
    }

    /// Merge with another state machine (LWW with validation)
    pub fn merge(&mut self, other: &Self) -> Result<()> {
        if other.timestamp.wins_over(&self.timestamp) {
            // Validate the transition path
            if self.is_valid_transition(&self.current_state, &other.current_state)
                || self.current_state == other.current_state
            {
                self.current_state = other.current_state.clone();
                self.timestamp = other.timestamp;
                Ok(())
            } else {
                Err(anyhow!(
                    "Cannot merge: invalid transition from {} to {}",
                    self.current_state,
                    other.current_state
                ))
            }
        } else {
            Ok(()) // No update needed
        }
    }
}

/// Issue status state machine
pub fn issue_state_machine() -> StateMachine {
    StateMachine::new(
        "backlog".to_string(),
        vec![
            (
                "backlog".to_string(),
                vec!["todo".to_string(), "canceled".to_string()],
            ),
            (
                "todo".to_string(),
                vec!["in-progress".to_string(), "canceled".to_string()],
            ),
            (
                "in-progress".to_string(),
                vec![
                    "done".to_string(),
                    "todo".to_string(),
                    "canceled".to_string(),
                ],
            ),
            ("done".to_string(), vec!["in-progress".to_string()]), // Reopen
            ("canceled".to_string(), vec!["backlog".to_string()]), // Reactivate
        ],
    )
}

/// Call state machine
pub fn call_state_machine() -> StateMachine {
    StateMachine::new(
        "idle".to_string(),
        vec![
            ("idle".to_string(), vec!["initiating".to_string()]),
            (
                "initiating".to_string(),
                vec!["ringing".to_string(), "failed".to_string()],
            ),
            (
                "ringing".to_string(),
                vec!["connecting".to_string(), "ended".to_string()],
            ),
            (
                "connecting".to_string(),
                vec!["connected".to_string(), "failed".to_string()],
            ),
            ("connected".to_string(), vec!["disconnecting".to_string()]),
            ("disconnecting".to_string(), vec!["ended".to_string()]),
            ("ended".to_string(), vec![]),  // Terminal state
            ("failed".to_string(), vec![]), // Terminal state
        ],
    )
}

/// Conflict notification for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictNotification {
    pub entity_type: String,
    pub entity_id: String,
    pub field_name: String,
    pub local_value: String,
    pub remote_value: String,
    pub local_timestamp: LamportTimestamp,
    pub remote_timestamp: LamportTimestamp,
    pub resolution: ResolutionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    /// Automatically resolved using LWW
    AutomaticLWW,
    /// Automatically resolved using CRDT merge
    AutomaticMerge,
    /// Requires manual user resolution
    ManualRequired,
}

/// Conflict detector that identifies conflicts during sync
pub struct ConflictDetector {
    notifications: Vec<ConflictNotification>,
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    /// Record a conflict for later notification
    pub fn record_conflict(&mut self, notification: ConflictNotification) {
        self.notifications.push(notification);
    }

    /// Get all pending conflict notifications
    pub fn pending_conflicts(&self) -> &[ConflictNotification] {
        &self.notifications
    }

    /// Clear conflict notifications
    pub fn clear(&mut self) {
        self.notifications.clear();
    }

    /// Check if there are unresolved conflicts
    pub fn has_unresolved(&self) -> bool {
        self.notifications
            .iter()
            .any(|n| matches!(n.resolution, ResolutionStrategy::ManualRequired))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lww_resolver() {
        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };
        let t2 = LamportTimestamp {
            timestamp: 200,
            tie_breaker: 1,
        };

        let local = LWWRegister::new("value1".to_string(), t1);
        let remote = LWWRegister::new("value2".to_string(), t2);

        let resolver = LWWResolver;
        let result = resolver.resolve(&local, &remote);

        match result {
            ConflictResolution::Resolved(r) => {
                assert_eq!(r.value, "value2");
                assert_eq!(r.timestamp, t2);
            }
            _ => panic!("Expected Resolved"),
        }
    }

    #[test]
    fn test_counter_resolver() {
        let mut counter1 = Counter::new();
        counter1.increment("peer1", 5);
        counter1.increment("peer2", 3);

        let mut counter2 = Counter::new();
        counter2.increment("peer1", 7);
        counter2.increment("peer3", 2);

        let resolver = CounterResolver;
        let result = resolver.resolve(&counter1, &counter2);

        match result {
            ConflictResolution::Resolved(c) => {
                assert_eq!(c.value(), 12); // max(5,7) + 3 + 2 = 7 + 3 + 2 = 12
            }
            _ => panic!("Expected Resolved"),
        }
    }

    #[test]
    fn test_issue_state_machine() {
        let mut sm = issue_state_machine();
        assert_eq!(sm.current_state, "backlog");

        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };
        assert!(sm.transition("todo".to_string(), t1).is_ok());
        assert_eq!(sm.current_state, "todo");

        let t2 = LamportTimestamp {
            timestamp: 200,
            tie_breaker: 1,
        };
        assert!(sm.transition("in-progress".to_string(), t2).is_ok());

        // Invalid transition
        let t3 = LamportTimestamp {
            timestamp: 300,
            tie_breaker: 1,
        };
        assert!(sm.transition("backlog".to_string(), t3).is_err());
    }

    #[test]
    fn test_call_state_machine() {
        let mut sm = call_state_machine();
        assert_eq!(sm.current_state, "idle");

        let t1 = LamportTimestamp {
            timestamp: 100,
            tie_breaker: 1,
        };
        assert!(sm.transition("initiating".to_string(), t1).is_ok());

        let t2 = LamportTimestamp {
            timestamp: 200,
            tie_breaker: 1,
        };
        assert!(sm.transition("ringing".to_string(), t2).is_ok());

        let t3 = LamportTimestamp {
            timestamp: 300,
            tie_breaker: 1,
        };
        assert!(sm.transition("connecting".to_string(), t3).is_ok());

        let t4 = LamportTimestamp {
            timestamp: 400,
            tie_breaker: 1,
        };
        assert!(sm.transition("connected".to_string(), t4).is_ok());
    }

    #[test]
    fn test_conflict_detector() {
        let mut detector = ConflictDetector::new();
        assert!(!detector.has_unresolved());

        let notification = ConflictNotification {
            entity_type: "channel".to_string(),
            entity_id: "channel-123".to_string(),
            field_name: "name".to_string(),
            local_value: "General".to_string(),
            remote_value: "Main".to_string(),
            local_timestamp: LamportTimestamp {
                timestamp: 100,
                tie_breaker: 1,
            },
            remote_timestamp: LamportTimestamp {
                timestamp: 200,
                tie_breaker: 1,
            },
            resolution: ResolutionStrategy::AutomaticLWW,
        };

        detector.record_conflict(notification);
        assert_eq!(detector.pending_conflicts().len(), 1);
        assert!(!detector.has_unresolved());

        detector.clear();
        assert_eq!(detector.pending_conflicts().len(), 0);
    }
}
