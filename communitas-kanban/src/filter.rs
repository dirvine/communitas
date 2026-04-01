// SPDX-License-Identifier: MIT OR Apache-2.0

//! Card filtering system for the Kanban.
//!
//! This module provides [`CardFilter`] for querying cards based on
//! various criteria like state, assignees, tags, due dates, and text search.

use serde::{Deserialize, Serialize};

use crate::state_machine::CardState;
use crate::types::Card;

/// Filter criteria for querying cards.
///
/// All filter fields are optional and combine with AND logic.
/// An empty filter matches all cards.
///
/// # Example
///
/// ```
/// use communitas_kanban::{CardFilter, CardState};
///
/// // Filter for open cards assigned to a specific user
/// let filter = CardFilter::new()
///     .with_states(vec![CardState::Open])
///     .with_assignees(vec!["ocean-forest-moon-star".to_string()]);
///
/// // Filter for overdue cards
/// let overdue_filter = CardFilter::new()
///     .with_overdue(true);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardFilter {
    /// Filter by card states (matches any in list)
    pub states: Option<Vec<CardState>>,
    /// Filter by assignee IDs (matches any in list)
    pub assignee_ids: Option<Vec<String>>,
    /// Filter by tag IDs (matches any in list)
    pub tag_ids: Option<Vec<String>>,
    /// Filter by draft status
    pub is_draft: Option<bool>,
    /// Filter by golden/starred status
    pub is_golden: Option<bool>,
    /// Filter by having a due date
    pub has_due_date: Option<bool>,
    /// Filter for cards due before this timestamp
    pub due_before: Option<i64>,
    /// Filter for cards due after this timestamp
    pub due_after: Option<i64>,
    /// Filter for overdue cards (due_date < now)
    pub overdue: Option<bool>,
    /// Text search in title and description
    pub search_text: Option<String>,
    /// Filter by column IDs (matches any in list)
    pub column_ids: Option<Vec<String>>,
    /// Exclude deleted cards (default: true)
    pub exclude_deleted: bool,
}

impl CardFilter {
    /// Create a new empty filter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            exclude_deleted: true,
            ..Default::default()
        }
    }

    /// Set state filter (builder pattern).
    #[must_use]
    pub fn with_states(mut self, states: Vec<CardState>) -> Self {
        self.states = Some(states);
        self
    }

    /// Set assignee filter (builder pattern).
    #[must_use]
    pub fn with_assignees(mut self, assignee_ids: Vec<String>) -> Self {
        self.assignee_ids = Some(assignee_ids);
        self
    }

    /// Set tag filter (builder pattern).
    #[must_use]
    pub fn with_tags(mut self, tag_ids: Vec<String>) -> Self {
        self.tag_ids = Some(tag_ids);
        self
    }

    /// Set draft filter (builder pattern).
    #[must_use]
    pub fn with_draft(mut self, is_draft: bool) -> Self {
        self.is_draft = Some(is_draft);
        self
    }

    /// Set golden/starred filter (builder pattern).
    #[must_use]
    pub fn with_golden(mut self, is_golden: bool) -> Self {
        self.is_golden = Some(is_golden);
        self
    }

    /// Set has_due_date filter (builder pattern).
    #[must_use]
    pub fn with_has_due_date(mut self, has_due_date: bool) -> Self {
        self.has_due_date = Some(has_due_date);
        self
    }

    /// Set due_before filter (builder pattern).
    #[must_use]
    pub fn with_due_before(mut self, timestamp: i64) -> Self {
        self.due_before = Some(timestamp);
        self
    }

    /// Set due_after filter (builder pattern).
    #[must_use]
    pub fn with_due_after(mut self, timestamp: i64) -> Self {
        self.due_after = Some(timestamp);
        self
    }

    /// Set overdue filter (builder pattern).
    #[must_use]
    pub fn with_overdue(mut self, overdue: bool) -> Self {
        self.overdue = Some(overdue);
        self
    }

    /// Set text search filter (builder pattern).
    #[must_use]
    pub fn with_search(mut self, text: String) -> Self {
        self.search_text = Some(text);
        self
    }

    /// Set column filter (builder pattern).
    #[must_use]
    pub fn with_columns(mut self, column_ids: Vec<String>) -> Self {
        self.column_ids = Some(column_ids);
        self
    }

    /// Set whether to include deleted cards (builder pattern).
    #[must_use]
    pub fn include_deleted(mut self) -> Self {
        self.exclude_deleted = false;
        self
    }

    /// Check if a card matches this filter.
    ///
    /// # Arguments
    ///
    /// * `card` - The card to check
    /// * `current_time` - Current Unix timestamp (for overdue checking)
    #[must_use]
    pub fn matches(&self, card: &Card, current_time: i64) -> bool {
        // Check deleted status
        if self.exclude_deleted && card.deleted {
            return false;
        }

        // Check states
        if let Some(ref states) = self.states
            && !states.contains(&card.state)
        {
            return false;
        }

        // Check assignees (any match)
        if let Some(ref assignee_ids) = self.assignee_ids {
            let has_match = card.assignee_ids.iter().any(|a| assignee_ids.contains(a));
            if !has_match {
                return false;
            }
        }

        // Check tags (any match)
        if let Some(ref tag_ids) = self.tag_ids {
            let has_match = card.tag_ids.iter().any(|t| tag_ids.contains(t));
            if !has_match {
                return false;
            }
        }

        // Check draft status
        if let Some(is_draft) = self.is_draft
            && card.is_draft != is_draft
        {
            return false;
        }

        // Check golden status
        if let Some(is_golden) = self.is_golden
            && card.is_golden != is_golden
        {
            return false;
        }

        // Check has_due_date
        if let Some(has_due_date) = self.has_due_date
            && card.due_date.is_some() != has_due_date
        {
            return false;
        }

        // Check due_before
        if let Some(due_before) = self.due_before {
            match card.due_date {
                Some(due) if due < due_before => {}
                _ => return false,
            }
        }

        // Check due_after
        if let Some(due_after) = self.due_after {
            match card.due_date {
                Some(due) if due > due_after => {}
                _ => return false,
            }
        }

        // Check overdue
        if let Some(overdue) = self.overdue {
            let is_overdue = card
                .due_date
                .map(|due| due < current_time && card.state.is_active())
                .unwrap_or(false);
            if is_overdue != overdue {
                return false;
            }
        }

        // Check columns
        if let Some(ref column_ids) = self.column_ids
            && !column_ids.contains(&card.column_id)
        {
            return false;
        }

        // Check text search (case-insensitive in title and description)
        if let Some(ref search_text) = self.search_text {
            let search_lower = search_text.to_lowercase();
            let title_match = card.title.to_lowercase().contains(&search_lower);
            let desc_match = card.description.to_lowercase().contains(&search_lower);
            if !title_match && !desc_match {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_card() -> Card {
        Card {
            id: "card-1".to_string(),
            board_id: "board-1".to_string(),
            column_id: "col-1".to_string(),
            title: "Test Card".to_string(),
            description: "A test card description".to_string(),
            position: 0,
            state: CardState::Open,
            priority: None,
            is_draft: false,
            is_golden: false,
            created_by: "user-1".to_string(),
            created_at: 1000,
            updated_at: 1000,
            due_date: Some(2000),
            completed_at: None,
            assignee_ids: vec!["user-1".to_string(), "user-2".to_string()],
            tag_ids: vec!["tag-1".to_string()],
            deleted: false,
            deleted_at: None,
            deleted_by: None,
            linked_thread_id: None,
        }
    }

    #[test]
    fn test_empty_filter_matches_all() {
        let filter = CardFilter::new();
        let card = make_test_card();
        assert!(filter.matches(&card, 1500));
    }

    #[test]
    fn test_state_filter() {
        let card = make_test_card();

        let filter = CardFilter::new().with_states(vec![CardState::Open]);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_states(vec![CardState::Closed]);
        assert!(!filter.matches(&card, 1500));

        let filter = CardFilter::new().with_states(vec![CardState::Open, CardState::Closed]);
        assert!(filter.matches(&card, 1500));
    }

    #[test]
    fn test_assignee_filter() {
        let card = make_test_card();

        let filter = CardFilter::new().with_assignees(vec!["user-1".to_string()]);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_assignees(vec!["user-3".to_string()]);
        assert!(!filter.matches(&card, 1500));

        let filter =
            CardFilter::new().with_assignees(vec!["user-2".to_string(), "user-3".to_string()]);
        assert!(filter.matches(&card, 1500));
    }

    #[test]
    fn test_tag_filter() {
        let card = make_test_card();

        let filter = CardFilter::new().with_tags(vec!["tag-1".to_string()]);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_tags(vec!["tag-2".to_string()]);
        assert!(!filter.matches(&card, 1500));
    }

    #[test]
    fn test_due_date_filters() {
        let card = make_test_card(); // due_date = 2000

        let filter = CardFilter::new().with_has_due_date(true);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_has_due_date(false);
        assert!(!filter.matches(&card, 1500));

        let filter = CardFilter::new().with_due_before(2500);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_due_before(1500);
        assert!(!filter.matches(&card, 1500));

        let filter = CardFilter::new().with_due_after(1500);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_due_after(2500);
        assert!(!filter.matches(&card, 1500));
    }

    #[test]
    fn test_overdue_filter() {
        let card = make_test_card(); // due_date = 2000, state = Open

        // Not overdue yet (current_time < due_date)
        let filter = CardFilter::new().with_overdue(true);
        assert!(!filter.matches(&card, 1500));

        // Overdue (current_time > due_date)
        let filter = CardFilter::new().with_overdue(true);
        assert!(filter.matches(&card, 2500));

        // Filter for non-overdue
        let filter = CardFilter::new().with_overdue(false);
        assert!(filter.matches(&card, 1500));
        assert!(!filter.matches(&card, 2500));
    }

    #[test]
    fn test_text_search() {
        let card = make_test_card();

        let filter = CardFilter::new().with_search("test".to_string());
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_search("TEST".to_string()); // case insensitive
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_search("description".to_string());
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_search("nonexistent".to_string());
        assert!(!filter.matches(&card, 1500));
    }

    #[test]
    fn test_deleted_filter() {
        let mut card = make_test_card();
        card.deleted = true;

        // Default excludes deleted
        let filter = CardFilter::new();
        assert!(!filter.matches(&card, 1500));

        // Include deleted
        let filter = CardFilter::new().include_deleted();
        assert!(filter.matches(&card, 1500));
    }

    #[test]
    fn test_column_filter() {
        let card = make_test_card();

        let filter = CardFilter::new().with_columns(vec!["col-1".to_string()]);
        assert!(filter.matches(&card, 1500));

        let filter = CardFilter::new().with_columns(vec!["col-2".to_string()]);
        assert!(!filter.matches(&card, 1500));
    }

    #[test]
    fn test_combined_filters() {
        let card = make_test_card();

        // Multiple filters combined with AND
        let filter = CardFilter::new()
            .with_states(vec![CardState::Open])
            .with_assignees(vec!["user-1".to_string()])
            .with_tags(vec!["tag-1".to_string()]);
        assert!(filter.matches(&card, 1500));

        // One filter fails
        let filter = CardFilter::new()
            .with_states(vec![CardState::Closed]) // fails
            .with_assignees(vec!["user-1".to_string()]);
        assert!(!filter.matches(&card, 1500));
    }
}
