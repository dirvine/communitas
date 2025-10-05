// Copyright (c) 2025 Saorsa Labs Limited
//
// This file is part of the Communitas P2P collaboration platform.
//
// Licensed under the GPL-3.0 license

//! Telemetry and Metrics
//!
//! Implements SPEC.md §6: Telemetry
//!
//! Metrics per topic:
//! - P50/P95 delivery latency
//! - Bytes per delivered message
//! - Mesh degree
//! - Score distribution
//!
//! Events:
//! - Join/leave
//! - Suspicion
//! - Reconvergence
//! - Anti-entropy stats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Telemetry data for a gossip topic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicMetrics {
    /// Topic identifier
    pub topic_id: String,

    /// Message delivery latency (milliseconds)
    pub latency_p50_ms: u64,
    pub latency_p95_ms: u64,

    /// Average bytes per delivered message
    pub avg_message_bytes: u64,

    /// Current mesh degree (number of peers)
    pub mesh_degree: usize,

    /// Peer score distribution
    pub score_distribution: ScoreDistribution,

    /// Total messages sent/received
    pub messages_sent: u64,
    pub messages_received: u64,

    /// Events
    pub join_events: u64,
    pub leave_events: u64,
    pub suspicion_events: u64,
    pub reconvergence_events: u64,

    /// Anti-entropy stats
    pub anti_entropy_rounds: u64,
    pub items_synced: u64,
}

/// Peer score distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDistribution {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub median: f64,
}

/// Telemetry collector
pub struct TelemetryCollector {
    topic_metrics: HashMap<String, TopicMetrics>,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new() -> Self {
        Self {
            topic_metrics: HashMap::new(),
        }
    }

    /// Record message delivery latency
    pub fn record_latency(&mut self, _topic_id: &str, latency: Duration) {
        // TODO: Implement P50/P95 calculation
        let _latency_ms = latency.as_millis() as u64;
    }

    /// Record message sent
    pub fn record_message_sent(&mut self, topic_id: &str, bytes: usize) {
        let metrics = self
            .topic_metrics
            .entry(topic_id.to_string())
            .or_insert_with(|| TopicMetrics::new(topic_id));

        metrics.messages_sent += 1;
        // Update avg_message_bytes
        let total = metrics.avg_message_bytes * (metrics.messages_sent - 1) + bytes as u64;
        metrics.avg_message_bytes = total / metrics.messages_sent;
    }

    /// Record message received
    pub fn record_message_received(&mut self, topic_id: &str, bytes: usize) {
        let metrics = self
            .topic_metrics
            .entry(topic_id.to_string())
            .or_insert_with(|| TopicMetrics::new(topic_id));

        metrics.messages_received += 1;
        // Update avg_message_bytes
        let total = metrics.avg_message_bytes * (metrics.messages_received - 1) + bytes as u64;
        metrics.avg_message_bytes = total / metrics.messages_received;
    }

    /// Record peer join event
    pub fn record_join(&mut self, topic_id: &str) {
        let metrics = self
            .topic_metrics
            .entry(topic_id.to_string())
            .or_insert_with(|| TopicMetrics::new(topic_id));
        metrics.join_events += 1;
        metrics.mesh_degree += 1;
    }

    /// Record peer leave event
    pub fn record_leave(&mut self, topic_id: &str) {
        let metrics = self
            .topic_metrics
            .entry(topic_id.to_string())
            .or_insert_with(|| TopicMetrics::new(topic_id));
        metrics.leave_events += 1;
        if metrics.mesh_degree > 0 {
            metrics.mesh_degree -= 1;
        }
    }

    /// Get metrics for a topic
    pub fn get_metrics(&self, topic_id: &str) -> Option<&TopicMetrics> {
        self.topic_metrics.get(topic_id)
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, TopicMetrics> {
        &self.topic_metrics
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TopicMetrics {
    fn new(topic_id: &str) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            latency_p50_ms: 0,
            latency_p95_ms: 0,
            avg_message_bytes: 0,
            mesh_degree: 0,
            score_distribution: ScoreDistribution {
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                median: 0.0,
            },
            messages_sent: 0,
            messages_received: 0,
            join_events: 0,
            leave_events: 0,
            suspicion_events: 0,
            reconvergence_events: 0,
            anti_entropy_rounds: 0,
            items_synced: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_collector() {
        let mut collector = TelemetryCollector::new();

        collector.record_message_sent("topic-1", 100);
        collector.record_message_sent("topic-1", 200);

        let metrics = collector.get_metrics("topic-1").expect("metrics");
        assert_eq!(metrics.messages_sent, 2);
        assert_eq!(metrics.avg_message_bytes, 150); // (100 + 200) / 2
    }
}
