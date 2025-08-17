/// Performance profiler for Saorsa Storage System
/// Identifies bottlenecks and optimization opportunities

use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ProfilePoint {
    pub name: String,
    pub start_time: Instant,
    pub duration: Option<Duration>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ProfileResult {
    pub name: String,
    pub total_duration: Duration,
    pub call_count: u64,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub metadata: HashMap<String, String>,
}

pub struct Profiler {
    points: Arc<RwLock<HashMap<String, Vec<ProfilePoint>>>>,
    enabled: bool,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            points: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            points: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
        }
    }

    pub async fn start_profile(&self, name: &str) -> ProfileGuard {
        if !self.enabled {
            return ProfileGuard::disabled();
        }

        let point = ProfilePoint {
            name: name.to_string(),
            start_time: Instant::now(),
            duration: None,
            metadata: HashMap::new(),
        };

        ProfileGuard {
            profiler: self.points.clone(),
            point: Some(point),
            enabled: self.enabled,
        }
    }

    pub async fn add_metadata(&self, name: &str, key: &str, value: &str) {
        if !self.enabled {
            return;
        }

        let mut points = self.points.write().await;
        if let Some(profile_points) = points.get_mut(name) {
            if let Some(last_point) = profile_points.last_mut() {
                last_point.metadata.insert(key.to_string(), value.to_string());
            }
        }
    }

    pub async fn get_results(&self) -> Vec<ProfileResult> {
        let points = self.points.read().await;
        let mut results = Vec::new();

        for (name, profile_points) in points.iter() {
            let completed_points: Vec<_> = profile_points
                .iter()
                .filter(|p| p.duration.is_some())
                .collect();

            if completed_points.is_empty() {
                continue;
            }

            let durations: Vec<Duration> = completed_points
                .iter()
                .map(|p| p.duration.unwrap())
                .collect();

            let total_duration = durations.iter().sum::<Duration>();
            let call_count = durations.len() as u64;
            let avg_duration = total_duration / call_count as u32;
            let min_duration = *durations.iter().min().unwrap();
            let max_duration = *durations.iter().max().unwrap();

            // Aggregate metadata
            let mut metadata = HashMap::new();
            for point in &completed_points {
                for (key, value) in &point.metadata {
                    metadata.insert(key.clone(), value.clone());
                }
            }

            results.push(ProfileResult {
                name: name.clone(),
                total_duration,
                call_count,
                avg_duration,
                min_duration,
                max_duration,
                metadata,
            });
        }

        // Sort by average duration (slowest first)
        results.sort_by(|a, b| b.avg_duration.cmp(&a.avg_duration));
        results
    }

    pub async fn reset(&self) {
        let mut points = self.points.write().await;
        points.clear();
    }

    pub async fn print_report(&self) {
        let results = self.get_results().await;
        
        println!("\n=== Performance Profile Report ===");
        if results.is_empty() {
            println!("No profile data collected");
            return;
        }

        println!("Top operations by average duration:");
        println!();

        for (i, result) in results.iter().enumerate().take(10) {
            println!("{}. {} ({} calls)", i + 1, result.name, result.call_count);
            println!("   Avg: {:>8.2}ms | Min: {:>8.2}ms | Max: {:>8.2}ms | Total: {:>8.2}ms",
                result.avg_duration.as_secs_f64() * 1000.0,
                result.min_duration.as_secs_f64() * 1000.0,
                result.max_duration.as_secs_f64() * 1000.0,
                result.total_duration.as_secs_f64() * 1000.0,
            );

            // Show metadata if available
            if !result.metadata.is_empty() {
                print!("   Metadata: ");
                for (key, value) in result.metadata.iter().take(3) {
                    print!("{}={} ", key, value);
                }
                println!();
            }
            println!();
        }

        // Performance analysis
        self.analyze_performance(&results).await;
    }

    async fn analyze_performance(&self, results: &[ProfileResult]) {
        println!("=== Performance Analysis ===");

        // Identify slow operations
        let slow_threshold = Duration::from_millis(100);
        let slow_ops: Vec<_> = results.iter()
            .filter(|r| r.avg_duration > slow_threshold)
            .collect();

        if !slow_ops.is_empty() {
            println!("⚠️  Slow operations (>100ms average):");
            for op in slow_ops {
                println!("   {} - {:.2}ms average", op.name, op.avg_duration.as_secs_f64() * 1000.0);
            }
            println!();
        }

        // Identify high-variance operations
        let high_variance_ops: Vec<_> = results.iter()
            .filter(|r| {
                let variance_ratio = r.max_duration.as_secs_f64() / r.min_duration.as_secs_f64();
                variance_ratio > 10.0 && r.call_count > 5
            })
            .collect();

        if !high_variance_ops.is_empty() {
            println!("⚠️  High variance operations (>10x difference):");
            for op in high_variance_ops {
                let ratio = op.max_duration.as_secs_f64() / op.min_duration.as_secs_f64();
                println!("   {} - {:.1}x variance", op.name, ratio);
            }
            println!();
        }

        // Total time analysis
        let total_time = results.iter()
            .map(|r| r.total_duration)
            .sum::<Duration>();

        println!("Total profiled time: {:.2}s", total_time.as_secs_f64());
        
        if let Some(top_op) = results.first() {
            let percentage = (top_op.total_duration.as_secs_f64() / total_time.as_secs_f64()) * 100.0;
            println!("Top operation '{}' accounts for {:.1}% of total time", top_op.name, percentage);
        }
    }
}

pub struct ProfileGuard {
    profiler: Arc<RwLock<HashMap<String, Vec<ProfilePoint>>>>,
    point: Option<ProfilePoint>,
    enabled: bool,
}

impl ProfileGuard {
    fn disabled() -> Self {
        Self {
            profiler: Arc::new(RwLock::new(HashMap::new())),
            point: None,
            enabled: false,
        }
    }

    pub async fn add_metadata(&mut self, key: &str, value: &str) {
        if !self.enabled || self.point.is_none() {
            return;
        }

        if let Some(ref mut point) = self.point {
            point.metadata.insert(key.to_string(), value.to_string());
        }
    }

    pub async fn add_size_metadata(&mut self, size: usize) {
        self.add_metadata("size", &size.to_string()).await;
    }

    pub async fn add_count_metadata(&mut self, count: usize) {
        self.add_metadata("count", &count.to_string()).await;
    }
}

impl Drop for ProfileGuard {
    fn drop(&mut self) {
        if !self.enabled || self.point.is_none() {
            return;
        }

        if let Some(mut point) = self.point.take() {
            point.duration = Some(point.start_time.elapsed());
            
            let profiler = self.profiler.clone();
            let name = point.name.clone();
            
            // Spawn a task to avoid blocking in Drop
            tokio::spawn(async move {
                let mut points = profiler.write().await;
                points.entry(name).or_insert_with(Vec::new).push(point);
            });
        }
    }
}

/// Macro for easy profiling
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $block:block) => {{
        let _guard = $profiler.start_profile($name).await;
        $block
    }};
    
    ($profiler:expr, $name:expr, $block:block, $($meta_key:expr => $meta_value:expr),+) => {{
        let mut _guard = $profiler.start_profile($name).await;
        $(
            _guard.add_metadata($meta_key, &$meta_value.to_string()).await;
        )+
        $block
    }};
}

/// Performance optimization recommendations
#[derive(Debug)]
pub struct OptimizationRecommendations {
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone)]
pub struct Recommendation {
    pub category: RecommendationCategory,
    pub priority: Priority,
    pub title: String,
    pub description: String,
    pub estimated_impact: ImpactLevel,
}

#[derive(Debug, Clone)]
pub enum RecommendationCategory {
    Caching,
    Networking,
    Encryption,
    Storage,
    Concurrency,
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum ImpactLevel {
    Minor,      // <10% improvement
    Moderate,   // 10-30% improvement
    Major,      // 30-50% improvement
    Significant, // >50% improvement
}

impl OptimizationRecommendations {
    pub fn analyze_profile_results(results: &[ProfileResult]) -> Self {
        let mut recommendations = Vec::new();

        // Analyze each operation
        for result in results {
            // Check for slow operations
            if result.avg_duration > Duration::from_millis(100) {
                if result.name.contains("encryption") || result.name.contains("decrypt") {
                    recommendations.push(Recommendation {
                        category: RecommendationCategory::Encryption,
                        priority: Priority::High,
                        title: format!("Optimize {} encryption", result.name),
                        description: format!(
                            "Encryption operation takes {:.2}ms on average. Consider hardware acceleration or optimized algorithms.",
                            result.avg_duration.as_secs_f64() * 1000.0
                        ),
                        estimated_impact: ImpactLevel::Major,
                    });
                }

                if result.name.contains("dht") || result.name.contains("network") {
                    recommendations.push(Recommendation {
                        category: RecommendationCategory::Networking,
                        priority: Priority::Medium,
                        title: format!("Optimize {} networking", result.name),
                        description: format!(
                            "Network operation takes {:.2}ms on average. Consider connection pooling or caching.",
                            result.avg_duration.as_secs_f64() * 1000.0
                        ),
                        estimated_impact: ImpactLevel::Moderate,
                    });
                }

                if result.name.contains("compress") || result.name.contains("decompress") {
                    recommendations.push(Recommendation {
                        category: RecommendationCategory::Storage,
                        priority: Priority::Medium,
                        title: format!("Optimize {} compression", result.name),
                        description: format!(
                            "Compression operation takes {:.2}ms on average. Consider faster algorithms or parallel compression.",
                            result.avg_duration.as_secs_f64() * 1000.0
                        ),
                        estimated_impact: ImpactLevel::Moderate,
                    });
                }
            }

            // Check for high variance
            let variance_ratio = result.max_duration.as_secs_f64() / result.min_duration.as_secs_f64();
            if variance_ratio > 10.0 && result.call_count > 5 {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Caching,
                    priority: Priority::High,
                    title: format!("Reduce variance in {}", result.name),
                    description: format!(
                        "Operation has high variance ({:.1}x). Consider adding caching or connection pooling.",
                        variance_ratio
                    ),
                    estimated_impact: ImpactLevel::Major,
                });
            }

            // Check for frequent operations
            if result.call_count > 100 && result.avg_duration > Duration::from_millis(10) {
                recommendations.push(Recommendation {
                    category: RecommendationCategory::Caching,
                    priority: Priority::Medium,
                    title: format!("Cache results of {}", result.name),
                    description: format!(
                        "Operation called {} times with {:.2}ms average. High-impact caching opportunity.",
                        result.call_count,
                        result.avg_duration.as_secs_f64() * 1000.0
                    ),
                    estimated_impact: ImpactLevel::Major,
                });
            }
        }

        // General recommendations based on patterns
        if results.iter().any(|r| r.name.contains("mutex") || r.name.contains("lock")) {
            recommendations.push(Recommendation {
                category: RecommendationCategory::Concurrency,
                priority: Priority::Medium,
                title: "Review locking strategy".to_string(),
                description: "Lock contention detected. Consider using lock-free data structures or reducing critical sections.".to_string(),
                estimated_impact: ImpactLevel::Moderate,
            });
        }

        // Sort by priority
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));

        Self { recommendations }
    }

    pub fn print_recommendations(&self) {
        println!("\n=== Performance Optimization Recommendations ===");
        
        if self.recommendations.is_empty() {
            println!("✅ No optimization recommendations - performance looks good!");
            return;
        }

        for (i, rec) in self.recommendations.iter().enumerate() {
            let priority_icon = match rec.priority {
                Priority::Critical => "🔴",
                Priority::High => "🟠",
                Priority::Medium => "🟡",
                Priority::Low => "🟢",
            };

            let impact_desc = match rec.estimated_impact {
                ImpactLevel::Significant => "High impact",
                ImpactLevel::Major => "Major impact",
                ImpactLevel::Moderate => "Moderate impact",
                ImpactLevel::Minor => "Minor impact",
            };

            println!("{}. {} {} - {}", i + 1, priority_icon, rec.title, impact_desc);
            println!("   {}", rec.description);
            println!("   Category: {:?} | Priority: {:?}", rec.category, rec.priority);
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_profiler_basic() {
        let profiler = Profiler::new();
        
        {
            let _guard = profiler.start_profile("test_operation").await;
            sleep(Duration::from_millis(10)).await;
        }

        let results = profiler.get_results().await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_operation");
        assert_eq!(results[0].call_count, 1);
        assert!(results[0].avg_duration >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_profiler_with_metadata() {
        let profiler = Profiler::new();
        
        {
            let mut guard = profiler.start_profile("test_with_metadata").await;
            guard.add_metadata("test_key", "test_value").await;
            sleep(Duration::from_millis(1)).await;
        }

        let results = profiler.get_results().await;
        assert_eq!(results.len(), 1);
        assert!(results[0].metadata.contains_key("test_key"));
    }

    #[tokio::test]
    async fn test_multiple_calls() {
        let profiler = Profiler::new();
        
        for _ in 0..5 {
            let _guard = profiler.start_profile("repeated_operation").await;
            sleep(Duration::from_millis(1)).await;
        }

        let results = profiler.get_results().await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].call_count, 5);
    }

    #[test]
    fn test_optimization_recommendations() {
        let results = vec![
            ProfileResult {
                name: "slow_encryption".to_string(),
                total_duration: Duration::from_millis(1000),
                call_count: 10,
                avg_duration: Duration::from_millis(100),
                min_duration: Duration::from_millis(50),
                max_duration: Duration::from_millis(150),
                metadata: HashMap::new(),
            }
        ];

        let recommendations = OptimizationRecommendations::analyze_profile_results(&results);
        assert!(!recommendations.recommendations.is_empty());
        
        let crypto_rec = recommendations.recommendations.iter()
            .find(|r| matches!(r.category, RecommendationCategory::Encryption));
        assert!(crypto_rec.is_some());
    }
}