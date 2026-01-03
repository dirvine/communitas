// Copyright (c) 2025 Saorsa Labs Limited
//
// Synchronization Barrier
//
// Provides synchronization between test phases and actors

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{Barrier, Notify, RwLock};

/// Synchronization barrier for coordinating test phases
#[allow(dead_code)]
pub struct SyncBarrier {
    /// Number of participants expected at each barrier
    num_participants: usize,

    /// Current barrier for sync points
    barrier: Arc<RwLock<Arc<Barrier>>>,

    /// Counter for completed phases
    phase_counter: AtomicUsize,

    /// Notification for phase completion
    phase_complete: Arc<Notify>,
}

impl SyncBarrier {
    /// Create a new sync barrier with the specified number of participants
    pub fn new(num_participants: usize) -> Self {
        Self {
            num_participants,
            barrier: Arc::new(RwLock::new(Arc::new(Barrier::new(1)))),
            phase_counter: AtomicUsize::new(0),
            phase_complete: Arc::new(Notify::new()),
        }
    }

    /// Wait at the current barrier point
    ///
    /// All participants must reach this point before any can proceed
    pub async fn wait(&self) {
        let barrier = self.barrier.read().await.clone();
        barrier.wait().await;
    }

    /// Wait for all actors to complete, then signal phase completion
    #[allow(dead_code)]
    pub async fn wait_for_actors(&self, num_actors: usize) {
        // Create a new barrier for this specific wait
        let barrier = Arc::new(Barrier::new(num_actors));

        // Wait at the barrier
        barrier.wait().await;
    }

    /// Signal that current phase is complete and advance to next
    #[allow(dead_code)]
    pub async fn advance_phase(&self) {
        self.phase_counter.fetch_add(1, Ordering::SeqCst);
        self.phase_complete.notify_waiters();
    }

    /// Get the current phase number
    #[allow(dead_code)]
    pub fn current_phase(&self) -> usize {
        self.phase_counter.load(Ordering::SeqCst)
    }

    /// Reset the barrier for a new test run
    #[allow(dead_code)]
    pub async fn reset(&self) {
        let mut barrier = self.barrier.write().await;
        *barrier = Arc::new(Barrier::new(self.num_participants.max(1)));
        self.phase_counter.store(0, Ordering::SeqCst);
    }
}

/// A multi-actor synchronization point for parallel test execution
#[allow(dead_code)]
pub struct ActorSync {
    /// Barrier for actor synchronization
    barrier: Arc<Barrier>,

    /// Completion status for each actor
    completed: Arc<RwLock<Vec<bool>>>,
}

#[allow(dead_code)]
impl ActorSync {
    /// Create a new actor sync for the specified actors
    pub fn new(num_actors: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(num_actors)),
            completed: Arc::new(RwLock::new(vec![false; num_actors])),
        }
    }

    /// Wait for all actors to reach this point
    pub async fn sync(&self) {
        self.barrier.wait().await;
    }

    /// Mark an actor as completed
    pub async fn mark_completed(&self, actor_index: usize) {
        let mut completed = self.completed.write().await;
        if actor_index < completed.len() {
            completed[actor_index] = true;
        }
    }

    /// Check if all actors have completed
    pub async fn all_completed(&self) -> bool {
        let completed = self.completed.read().await;
        completed.iter().all(|&c| c)
    }

    /// Reset completion status
    pub async fn reset(&self) {
        let mut completed = self.completed.write().await;
        completed.iter_mut().for_each(|c| *c = false);
    }
}

/// Phase-level synchronization for ordered test execution
#[allow(dead_code)]
pub struct PhaseSync {
    /// Current phase being executed
    current_phase: AtomicUsize,

    /// Total number of phases
    total_phases: usize,

    /// Notification for phase transitions
    phase_notify: Arc<Notify>,
}

#[allow(dead_code)]
impl PhaseSync {
    /// Create a new phase sync with the specified number of phases
    pub fn new(total_phases: usize) -> Self {
        Self {
            current_phase: AtomicUsize::new(0),
            total_phases,
            phase_notify: Arc::new(Notify::new()),
        }
    }

    /// Get the current phase number
    pub fn current(&self) -> usize {
        self.current_phase.load(Ordering::SeqCst)
    }

    /// Check if all phases are complete
    pub fn is_complete(&self) -> bool {
        self.current() >= self.total_phases
    }

    /// Advance to the next phase
    pub fn advance(&self) -> bool {
        let current = self.current_phase.fetch_add(1, Ordering::SeqCst);
        self.phase_notify.notify_waiters();
        current + 1 < self.total_phases
    }

    /// Wait for a specific phase to be reached
    pub async fn wait_for_phase(&self, phase: usize) {
        while self.current() < phase {
            self.phase_notify.notified().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sync_barrier_basic() {
        let barrier = SyncBarrier::new(2);

        // Should be at phase 0
        assert_eq!(barrier.current_phase(), 0);

        // Advance phase
        barrier.advance_phase().await;
        assert_eq!(barrier.current_phase(), 1);
    }

    #[tokio::test]
    async fn test_actor_sync() {
        let sync = ActorSync::new(3);

        // Mark actors as completed
        sync.mark_completed(0).await;
        assert!(!sync.all_completed().await);

        sync.mark_completed(1).await;
        assert!(!sync.all_completed().await);

        sync.mark_completed(2).await;
        assert!(sync.all_completed().await);
    }

    #[tokio::test]
    async fn test_phase_sync() {
        let sync = PhaseSync::new(3);

        assert_eq!(sync.current(), 0);
        assert!(!sync.is_complete());

        sync.advance();
        assert_eq!(sync.current(), 1);

        sync.advance();
        assert_eq!(sync.current(), 2);

        sync.advance();
        assert!(sync.is_complete());
    }
}
