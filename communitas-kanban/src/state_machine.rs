//! Card state machine for workflow transitions.
//!
//! This module defines the [`CardState`] enum and valid state transitions.
//! The state machine ensures cards follow a proper workflow:
//!
//! ```text
//! Open <──────> Closed
//!   │              │
//!   v              v
//! Postponed ───> Archived <─── (from Open)
//!   │
//!   v
//!  Open
//! ```

use serde::{Deserialize, Serialize};

/// The state of a card in the Kanban workflow.
///
/// Cards progress through states representing their lifecycle:
/// - `Open`: Active work item, ready for work
/// - `Closed`: Completed work item
/// - `Postponed`: Work item deferred to later
/// - `Archived`: Removed from active view but preserved
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum CardState {
    /// Active work item, ready for work
    #[default]
    Open,
    /// Completed work item
    Closed,
    /// Work item deferred to later
    Postponed,
    /// Removed from active view but preserved
    Archived,
}

impl CardState {
    /// Check if transition to the target state is valid.
    ///
    /// # Valid Transitions
    ///
    /// | From | To |
    /// |------|------|
    /// | Open | Closed, Postponed, Archived |
    /// | Closed | Open, Archived |
    /// | Postponed | Open, Archived |
    /// | Archived | Open (restore) |
    ///
    /// # Example
    ///
    /// ```
    /// use communitas_kanban::CardState;
    ///
    /// assert!(CardState::Open.can_transition_to(CardState::Closed));
    /// assert!(CardState::Closed.can_transition_to(CardState::Archived));
    /// assert!(CardState::Archived.can_transition_to(CardState::Open)); // restore
    /// assert!(!CardState::Closed.can_transition_to(CardState::Postponed));
    /// ```
    #[must_use]
    pub fn can_transition_to(&self, target: CardState) -> bool {
        matches!(
            (self, target),
            // From Open: can close, postpone, or archive
            (CardState::Open, CardState::Closed)
                | (CardState::Open, CardState::Postponed)
                | (CardState::Open, CardState::Archived)
                // From Closed: can reopen or archive
                | (CardState::Closed, CardState::Open)
                | (CardState::Closed, CardState::Archived)
                // From Postponed: can reopen or archive
                | (CardState::Postponed, CardState::Open)
                | (CardState::Postponed, CardState::Archived)
                // From Archived: can only restore to Open
                | (CardState::Archived, CardState::Open)
        )
    }

    /// Get all valid target states from this state.
    ///
    /// # Example
    ///
    /// ```
    /// use communitas_kanban::CardState;
    ///
    /// let targets = CardState::Open.valid_transitions();
    /// assert!(targets.contains(&CardState::Closed));
    /// assert!(targets.contains(&CardState::Postponed));
    /// assert!(targets.contains(&CardState::Archived));
    /// ```
    #[must_use]
    pub fn valid_transitions(&self) -> Vec<CardState> {
        match self {
            CardState::Open => vec![CardState::Closed, CardState::Postponed, CardState::Archived],
            CardState::Closed => vec![CardState::Open, CardState::Archived],
            CardState::Postponed => vec![CardState::Open, CardState::Archived],
            CardState::Archived => vec![CardState::Open],
        }
    }

    /// Check if the card is in a terminal state (archived).
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, CardState::Archived)
    }

    /// Check if the card is considered "done" (closed or archived).
    #[must_use]
    pub fn is_done(&self) -> bool {
        matches!(self, CardState::Closed | CardState::Archived)
    }

    /// Check if the card is in an active state (open or postponed).
    #[must_use]
    pub fn is_active(&self) -> bool {
        matches!(self, CardState::Open | CardState::Postponed)
    }

    /// Get a human-readable label for the state.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            CardState::Open => "Open",
            CardState::Closed => "Closed",
            CardState::Postponed => "Postponed",
            CardState::Archived => "Archived",
        }
    }

    /// Get an action verb for transitioning to this state.
    #[must_use]
    pub fn action_verb(&self) -> &'static str {
        match self {
            CardState::Open => "Reopen",
            CardState::Closed => "Close",
            CardState::Postponed => "Postpone",
            CardState::Archived => "Archive",
        }
    }
}

impl std::fmt::Display for CardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_transitions() {
        assert!(CardState::Open.can_transition_to(CardState::Closed));
        assert!(CardState::Open.can_transition_to(CardState::Postponed));
        assert!(CardState::Open.can_transition_to(CardState::Archived));
        assert!(!CardState::Open.can_transition_to(CardState::Open));
    }

    #[test]
    fn test_closed_transitions() {
        assert!(CardState::Closed.can_transition_to(CardState::Open));
        assert!(CardState::Closed.can_transition_to(CardState::Archived));
        assert!(!CardState::Closed.can_transition_to(CardState::Postponed));
        assert!(!CardState::Closed.can_transition_to(CardState::Closed));
    }

    #[test]
    fn test_postponed_transitions() {
        assert!(CardState::Postponed.can_transition_to(CardState::Open));
        assert!(CardState::Postponed.can_transition_to(CardState::Archived));
        assert!(!CardState::Postponed.can_transition_to(CardState::Closed));
        assert!(!CardState::Postponed.can_transition_to(CardState::Postponed));
    }

    #[test]
    fn test_archived_transitions() {
        assert!(CardState::Archived.can_transition_to(CardState::Open));
        assert!(!CardState::Archived.can_transition_to(CardState::Closed));
        assert!(!CardState::Archived.can_transition_to(CardState::Postponed));
        assert!(!CardState::Archived.can_transition_to(CardState::Archived));
    }

    #[test]
    fn test_valid_transitions_list() {
        let open_transitions = CardState::Open.valid_transitions();
        assert_eq!(open_transitions.len(), 3);
        assert!(open_transitions.contains(&CardState::Closed));
        assert!(open_transitions.contains(&CardState::Postponed));
        assert!(open_transitions.contains(&CardState::Archived));

        let archived_transitions = CardState::Archived.valid_transitions();
        assert_eq!(archived_transitions.len(), 1);
        assert!(archived_transitions.contains(&CardState::Open));
    }

    #[test]
    fn test_state_predicates() {
        assert!(!CardState::Open.is_done());
        assert!(CardState::Closed.is_done());
        assert!(CardState::Archived.is_done());
        assert!(!CardState::Postponed.is_done());

        assert!(CardState::Open.is_active());
        assert!(CardState::Postponed.is_active());
        assert!(!CardState::Closed.is_active());
        assert!(!CardState::Archived.is_active());

        assert!(!CardState::Open.is_terminal());
        assert!(CardState::Archived.is_terminal());
    }

    #[test]
    fn test_default_state() {
        assert_eq!(CardState::default(), CardState::Open);
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", CardState::Open), "Open");
        assert_eq!(format!("{}", CardState::Closed), "Closed");
        assert_eq!(format!("{}", CardState::Postponed), "Postponed");
        assert_eq!(format!("{}", CardState::Archived), "Archived");
    }
}
