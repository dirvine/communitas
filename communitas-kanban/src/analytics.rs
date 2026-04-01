// SPDX-License-Identifier: MIT OR Apache-2.0

//! Analytics and metrics for Kanban boards.
//!
//! This module provides data structures and calculations for tracking board
//! performance, including velocity, cycle time, burndown charts, and WIP metrics.
//!
//! # Example
//!
//! ```ignore
//! use communitas_kanban::analytics::{BoardAnalytics, VelocityMetric, TimeRange};
//!
//! // Calculate velocity for the last 4 weeks
//! let velocity = service.calculate_velocity(&board_id, TimeRange::Weeks(4)).await?;
//! println!("Cards completed per week: {:?}", velocity.data_points);
//!
//! // Get board analytics summary
//! let analytics = service.get_board_analytics(&board_id).await?;
//! println!("Cards in Done: {}", analytics.column_card_counts.get("done").unwrap_or(&0));
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Summary analytics for a Kanban board.
///
/// Provides a snapshot of the current board state with counts and metrics.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BoardAnalytics {
    /// Board ID this analytics relates to.
    pub board_id: String,

    /// Timestamp when this analytics was calculated.
    pub calculated_at: i64,

    /// Number of cards per column (column_id -> count).
    pub column_card_counts: HashMap<String, usize>,

    /// Total active cards (not deleted, not archived).
    pub total_active_cards: usize,

    /// Cards completed this week (based on completed_at).
    pub completed_this_week: usize,

    /// Cards completed this month.
    pub completed_this_month: usize,

    /// Average cycle time in seconds (time from creation to completion).
    pub avg_cycle_time_seconds: Option<i64>,

    /// Average time in each column (column_id -> avg seconds).
    ///
    /// Note: Requires column transition tracking which may not be available
    /// for all cards. Returns None for columns with insufficient data.
    pub avg_time_per_column: HashMap<String, i64>,

    /// Work In Progress count (cards in active columns).
    pub wip_count: usize,

    /// Cards that are overdue (past due_date but not completed).
    pub overdue_count: usize,

    /// Cards due in the next 7 days.
    pub due_soon_count: usize,
}

/// Velocity metric showing cards completed over time.
///
/// Velocity is a measure of throughput - how many cards are completed
/// in a given time period.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VelocityMetric {
    /// Board ID this velocity relates to.
    pub board_id: String,

    /// Time range used for calculation.
    pub time_range: TimeRange,

    /// Data points: (period_start_timestamp, cards_completed).
    ///
    /// For weekly velocity, each entry represents one week's worth of
    /// completed cards.
    pub data_points: Vec<VelocityDataPoint>,

    /// Average cards completed per period.
    pub average: f64,

    /// Standard deviation of completion rate.
    pub std_deviation: f64,
}

/// A single data point in velocity tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VelocityDataPoint {
    /// Start of the period (Unix timestamp).
    pub period_start: i64,

    /// End of the period (Unix timestamp).
    pub period_end: i64,

    /// Number of cards completed in this period.
    pub cards_completed: usize,
}

/// Burndown chart data for tracking remaining work.
///
/// Shows the ideal vs actual work remaining over a sprint or project.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BurndownChart {
    /// Board ID this burndown relates to.
    pub board_id: String,

    /// Start of the burndown period (Unix timestamp).
    pub period_start: i64,

    /// End of the burndown period (Unix timestamp).
    pub period_end: i64,

    /// Total cards at the start of the period.
    pub initial_card_count: usize,

    /// Ideal burndown line: (timestamp, remaining_count).
    pub ideal_line: Vec<(i64, f64)>,

    /// Actual burndown line: (timestamp, remaining_count).
    pub actual_line: Vec<BurndownDataPoint>,

    /// Current scope (total cards including added since start).
    pub current_scope: usize,

    /// Cards added since period start.
    pub scope_change: i32,
}

/// A single data point in burndown tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BurndownDataPoint {
    /// Timestamp of measurement.
    pub timestamp: i64,

    /// Cards remaining (not completed).
    pub remaining: usize,

    /// Cards completed so far.
    pub completed: usize,

    /// Cards added since start.
    pub added: usize,
}

/// Cycle time distribution for understanding work flow.
///
/// Shows how long cards typically take to complete.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CycleTimeDistribution {
    /// Board ID.
    pub board_id: String,

    /// Minimum cycle time in seconds.
    pub min_seconds: i64,

    /// Maximum cycle time in seconds.
    pub max_seconds: i64,

    /// Median cycle time in seconds.
    pub median_seconds: i64,

    /// Average cycle time in seconds.
    pub avg_seconds: i64,

    /// 85th percentile cycle time in seconds.
    pub p85_seconds: i64,

    /// Histogram buckets: (bucket_label, count).
    ///
    /// Example buckets: "< 1 day", "1-3 days", "3-7 days", "1-2 weeks", "> 2 weeks"
    pub histogram: Vec<CycleTimeBucket>,
}

/// A bucket in the cycle time histogram.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CycleTimeBucket {
    /// Human-readable label (e.g., "1-3 days").
    pub label: String,

    /// Minimum seconds for this bucket (inclusive).
    pub min_seconds: i64,

    /// Maximum seconds for this bucket (exclusive, except for last bucket).
    pub max_seconds: i64,

    /// Number of cards in this bucket.
    pub count: usize,
}

/// Time range for analytics queries.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeRange {
    /// Last N days.
    Days(u32),
    /// Last N weeks.
    Weeks(u32),
    /// Last N months.
    Months(u32),
    /// Custom range (start, end) as Unix timestamps.
    Custom(i64, i64),
}

impl TimeRange {
    /// Create a custom time range with validation.
    ///
    /// Returns `None` if start > end (invalid time range).
    ///
    /// # Example
    ///
    /// ```
    /// use communitas_kanban::TimeRange;
    ///
    /// let valid = TimeRange::custom(100, 200);
    /// assert!(valid.is_some());
    ///
    /// let invalid = TimeRange::custom(200, 100); // start > end
    /// assert!(invalid.is_none());
    /// ```
    #[must_use]
    pub fn custom(start: i64, end: i64) -> Option<Self> {
        if start > end {
            None
        } else {
            Some(TimeRange::Custom(start, end))
        }
    }

    /// Check if this time range is valid.
    ///
    /// For `Custom` ranges, validates that start <= end.
    /// Other variants are always valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        match self {
            TimeRange::Custom(start, end) => start <= end,
            _ => true,
        }
    }

    /// Convert to (start_timestamp, end_timestamp) given current time.
    ///
    /// For `Custom` ranges where start > end (invalid), this swaps the values
    /// to ensure the returned tuple always has start <= end.
    #[must_use]
    pub fn to_timestamps(self, now: i64) -> (i64, i64) {
        const SECS_PER_DAY: i64 = 86400;
        const SECS_PER_WEEK: i64 = SECS_PER_DAY * 7;

        match self {
            TimeRange::Days(n) => {
                let start = now - (i64::from(n) * SECS_PER_DAY);
                (start, now)
            }
            TimeRange::Weeks(n) => {
                let start = now - (i64::from(n) * SECS_PER_WEEK);
                (start, now)
            }
            TimeRange::Months(n) => {
                // Approximate: 30 days per month
                let start = now - (i64::from(n) * 30 * SECS_PER_DAY);
                (start, now)
            }
            TimeRange::Custom(start, end) => {
                // Ensure start <= end (swap if invalid)
                if start > end {
                    (end, start)
                } else {
                    (start, end)
                }
            }
        }
    }

    /// Get the period duration in seconds for each data point.
    #[must_use]
    pub fn period_duration_seconds(self) -> i64 {
        const SECS_PER_DAY: i64 = 86400;
        const SECS_PER_WEEK: i64 = SECS_PER_DAY * 7;

        match self {
            TimeRange::Days(_) => SECS_PER_DAY,
            TimeRange::Weeks(_) | TimeRange::Custom(_, _) => SECS_PER_WEEK,
            TimeRange::Months(_) => 30 * SECS_PER_DAY,
        }
    }
}

impl Default for TimeRange {
    fn default() -> Self {
        TimeRange::Weeks(4)
    }
}

impl BoardAnalytics {
    /// Create a new analytics instance for a board.
    #[must_use]
    pub fn new(board_id: String, calculated_at: i64) -> Self {
        Self {
            board_id,
            calculated_at,
            ..Default::default()
        }
    }
}

impl VelocityMetric {
    /// Create a new velocity metric from data points.
    pub fn from_data_points(
        board_id: String,
        time_range: TimeRange,
        data_points: Vec<VelocityDataPoint>,
    ) -> Self {
        let values: Vec<f64> = data_points
            .iter()
            .map(|p| p.cards_completed as f64)
            .collect();

        let average = if values.is_empty() {
            0.0
        } else {
            values.iter().sum::<f64>() / values.len() as f64
        };

        let std_deviation = if values.len() < 2 {
            0.0
        } else {
            let variance = values.iter().map(|v| (v - average).powi(2)).sum::<f64>()
                / (values.len() - 1) as f64;
            variance.sqrt()
        };

        Self {
            board_id,
            time_range,
            data_points,
            average,
            std_deviation,
        }
    }
}

impl CycleTimeDistribution {
    /// Create standard time buckets for the histogram.
    #[must_use]
    pub fn standard_buckets() -> Vec<CycleTimeBucket> {
        const DAY: i64 = 86400;

        vec![
            CycleTimeBucket {
                label: "< 1 day".to_string(),
                min_seconds: 0,
                max_seconds: DAY,
                count: 0,
            },
            CycleTimeBucket {
                label: "1-3 days".to_string(),
                min_seconds: DAY,
                max_seconds: 3 * DAY,
                count: 0,
            },
            CycleTimeBucket {
                label: "3-7 days".to_string(),
                min_seconds: 3 * DAY,
                max_seconds: 7 * DAY,
                count: 0,
            },
            CycleTimeBucket {
                label: "1-2 weeks".to_string(),
                min_seconds: 7 * DAY,
                max_seconds: 14 * DAY,
                count: 0,
            },
            CycleTimeBucket {
                label: "> 2 weeks".to_string(),
                min_seconds: 14 * DAY,
                max_seconds: i64::MAX,
                count: 0,
            },
        ]
    }

    /// Calculate distribution from a list of cycle times (in seconds).
    #[must_use]
    pub fn from_cycle_times(board_id: String, mut cycle_times: Vec<i64>) -> Self {
        if cycle_times.is_empty() {
            return Self {
                board_id,
                ..Default::default()
            };
        }

        cycle_times.sort_unstable();
        let len = cycle_times.len();

        let min_seconds = cycle_times[0];
        let max_seconds = cycle_times[len - 1];
        let median_seconds = cycle_times[len / 2];
        let avg_seconds = cycle_times.iter().sum::<i64>() / len as i64;
        let p85_index = (len as f64 * 0.85) as usize;
        let p85_seconds = cycle_times.get(p85_index).copied().unwrap_or(max_seconds);

        // Build histogram
        let mut histogram = Self::standard_buckets();
        for &time in &cycle_times {
            for bucket in &mut histogram {
                if time >= bucket.min_seconds && time < bucket.max_seconds {
                    bucket.count += 1;
                    break;
                }
            }
        }

        Self {
            board_id,
            min_seconds,
            max_seconds,
            median_seconds,
            avg_seconds,
            p85_seconds,
            histogram,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_to_timestamps() {
        let now = 1_700_000_000i64; // Some fixed timestamp

        // Days
        let (start, end) = TimeRange::Days(7).to_timestamps(now);
        assert_eq!(end, now);
        assert_eq!(start, now - 7 * 86400);

        // Weeks
        let (start, end) = TimeRange::Weeks(4).to_timestamps(now);
        assert_eq!(end, now);
        assert_eq!(start, now - 4 * 7 * 86400);

        // Custom
        let (start, end) = TimeRange::Custom(100, 200).to_timestamps(now);
        assert_eq!(start, 100);
        assert_eq!(end, 200);
    }

    #[test]
    fn test_velocity_metric_calculation() {
        let data_points = vec![
            VelocityDataPoint {
                period_start: 0,
                period_end: 100,
                cards_completed: 5,
            },
            VelocityDataPoint {
                period_start: 100,
                period_end: 200,
                cards_completed: 7,
            },
            VelocityDataPoint {
                period_start: 200,
                period_end: 300,
                cards_completed: 6,
            },
            VelocityDataPoint {
                period_start: 300,
                period_end: 400,
                cards_completed: 8,
            },
        ];

        let velocity = VelocityMetric::from_data_points(
            "board-1".to_string(),
            TimeRange::Weeks(4),
            data_points,
        );

        assert_eq!(velocity.data_points.len(), 4);
        assert!((velocity.average - 6.5).abs() < 0.001);
        assert!(velocity.std_deviation > 0.0);
    }

    #[test]
    fn test_velocity_empty() {
        let velocity =
            VelocityMetric::from_data_points("board-1".to_string(), TimeRange::Weeks(4), vec![]);

        assert_eq!(velocity.average, 0.0);
        assert_eq!(velocity.std_deviation, 0.0);
    }

    #[test]
    fn test_cycle_time_distribution() {
        // Mix of cycle times
        let cycle_times = vec![
            3600,    // 1 hour (< 1 day)
            86400,   // 1 day (1-3 days)
            172800,  // 2 days (1-3 days)
            432000,  // 5 days (3-7 days)
            604800,  // 7 days (1-2 weeks)
            1209600, // 14 days (> 2 weeks)
            2592000, // 30 days (> 2 weeks)
        ];

        let dist = CycleTimeDistribution::from_cycle_times("board-1".to_string(), cycle_times);

        assert_eq!(dist.min_seconds, 3600);
        assert_eq!(dist.max_seconds, 2592000);

        // Check histogram buckets
        let bucket_counts: Vec<usize> = dist.histogram.iter().map(|b| b.count).collect();
        assert_eq!(bucket_counts, vec![1, 2, 1, 1, 2]);
    }

    #[test]
    fn test_cycle_time_empty() {
        let dist = CycleTimeDistribution::from_cycle_times("board-1".to_string(), vec![]);

        assert_eq!(dist.min_seconds, 0);
        assert_eq!(dist.max_seconds, 0);
        assert!(dist.histogram.is_empty() || dist.histogram.iter().all(|b| b.count == 0));
    }

    #[test]
    fn test_board_analytics_new() {
        let analytics = BoardAnalytics::new("board-1".to_string(), 1_700_000_000);

        assert_eq!(analytics.board_id, "board-1");
        assert_eq!(analytics.calculated_at, 1_700_000_000);
        assert_eq!(analytics.total_active_cards, 0);
        assert!(analytics.column_card_counts.is_empty());
    }

    #[test]
    fn test_time_range_period_duration() {
        assert_eq!(TimeRange::Days(7).period_duration_seconds(), 86400);
        assert_eq!(TimeRange::Weeks(4).period_duration_seconds(), 7 * 86400);
        assert_eq!(TimeRange::Months(1).period_duration_seconds(), 30 * 86400);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_time_range_custom_validation() {
        // Valid custom range
        let valid = TimeRange::custom(100, 200);
        assert!(valid.is_some());
        assert!(valid.unwrap().is_valid());

        // Equal start and end is valid
        let equal = TimeRange::custom(100, 100);
        assert!(equal.is_some());
        assert!(equal.unwrap().is_valid());

        // Invalid: start > end
        let invalid = TimeRange::custom(200, 100);
        assert!(invalid.is_none());

        // Direct construction can create invalid range
        let direct_invalid = TimeRange::Custom(200, 100);
        assert!(!direct_invalid.is_valid());

        // Other variants are always valid
        assert!(TimeRange::Days(7).is_valid());
        assert!(TimeRange::Weeks(4).is_valid());
        assert!(TimeRange::Months(3).is_valid());
    }

    #[test]
    fn test_time_range_custom_to_timestamps_swaps_invalid() {
        let now = 1_700_000_000i64;

        // Valid custom range preserves order
        let (start, end) = TimeRange::Custom(100, 200).to_timestamps(now);
        assert_eq!(start, 100);
        assert_eq!(end, 200);

        // Invalid custom range (start > end) gets swapped
        let (start, end) = TimeRange::Custom(200, 100).to_timestamps(now);
        assert_eq!(start, 100); // swapped
        assert_eq!(end, 200); // swapped
    }

    #[test]
    fn test_velocity_empty_single_data_point() {
        // Single data point should have zero std deviation
        let single = VelocityMetric::from_data_points(
            "board-1".to_string(),
            TimeRange::Weeks(1),
            vec![VelocityDataPoint {
                period_start: 0,
                period_end: 100,
                cards_completed: 5,
            }],
        );
        assert_eq!(single.average, 5.0);
        assert_eq!(single.std_deviation, 0.0);
    }

    #[test]
    fn test_cycle_time_single_value() {
        // Single cycle time
        let dist = CycleTimeDistribution::from_cycle_times("board-1".to_string(), vec![86400]); // 1 day

        assert_eq!(dist.min_seconds, 86400);
        assert_eq!(dist.max_seconds, 86400);
        assert_eq!(dist.median_seconds, 86400);
        assert_eq!(dist.avg_seconds, 86400);
    }

    #[test]
    fn test_board_analytics_defaults() {
        let analytics = BoardAnalytics::default();
        assert!(analytics.board_id.is_empty());
        assert_eq!(analytics.calculated_at, 0);
        assert_eq!(analytics.total_active_cards, 0);
        assert_eq!(analytics.completed_this_week, 0);
        assert!(analytics.avg_cycle_time_seconds.is_none());
    }
}
