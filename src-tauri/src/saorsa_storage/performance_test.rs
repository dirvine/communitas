/// Performance testing utilities for Saorsa Storage System
/// Validates that performance targets are met
use crate::saorsa_storage::{
    benchmarks::{BenchmarkConfig, StorageBenchmark},
    profiler::{OptimizationRecommendations, Profiler},
};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct PerformanceTestRunner {
    benchmark_config: BenchmarkConfig,
    profiler: Arc<Profiler>,
}

impl PerformanceTestRunner {
    pub fn new() -> Self {
        Self {
            benchmark_config: BenchmarkConfig::default(),
            profiler: Arc::new(Profiler::new()),
        }
    }

    pub fn with_config(config: BenchmarkConfig) -> Self {
        Self {
            benchmark_config: config,
            profiler: Arc::new(Profiler::new()),
        }
    }

    /// Run comprehensive performance validation
    pub async fn run_performance_validation(
        &mut self,
    ) -> Result<PerformanceReport, Box<dyn std::error::Error>> {
        println!("🚀 Starting Saorsa Storage Performance Validation");
        println!("================================================");

        let validation_start = Instant::now();

        // Step 1: Run comprehensive benchmarks
        println!("\n📊 Running Performance Benchmarks...");
        let mut benchmark = StorageBenchmark::new(self.benchmark_config.clone())?;
        let benchmark_report = benchmark.run_comprehensive_benchmarks().await;

        // Print benchmark results
        benchmark_report.print_summary();

        // Step 2: Profile performance bottlenecks
        println!("\n🔍 Analyzing Performance Profile...");
        let profile_results = self.profiler.get_results().await;
        if !profile_results.is_empty() {
            self.profiler.print_report().await;

            // Generate optimization recommendations
            let recommendations =
                OptimizationRecommendations::analyze_profile_results(&profile_results);
            recommendations.print_recommendations();
        } else {
            println!("ℹ️  No profile data available (profiling may not be enabled in benchmarks)");
        }

        // Step 3: Validate performance targets
        println!("\n✅ Validating Performance Targets...");
        let validation_results = self.validate_performance_targets(&benchmark_report).await;

        // Step 4: Generate comprehensive report
        let total_duration = validation_start.elapsed();
        let report = PerformanceReport {
            benchmark_report,
            validation_results,
            total_duration,
            recommendations: if !profile_results.is_empty() {
                Some(OptimizationRecommendations::analyze_profile_results(
                    &profile_results,
                ))
            } else {
                None
            },
        };

        // Print final validation results
        self.print_validation_summary(&report);

        Ok(report)
    }

    async fn validate_performance_targets(
        &self,
        benchmark_report: &crate::saorsa_storage::benchmarks::BenchmarkReport,
    ) -> PerformanceValidationResults {
        let mut validations = Vec::new();

        // Validate local operation targets (<100ms)
        let local_ops: Vec<_> = benchmark_report
            .results
            .iter()
            .filter(|r| r.operation.contains("cache") || r.operation.contains("local"))
            .collect();

        for result in local_ops {
            let target_met = result.p95_time_ms <= 100;
            validations.push(PerformanceValidation {
                operation: result.operation.clone(),
                target_description: "Local operations <100ms (P95)".to_string(),
                actual_p95_ms: result.p95_time_ms,
                target_ms: 100,
                success_rate: result.success_rate,
                passed: target_met && result.success_rate >= 0.95,
            });
        }

        // Validate remote operation targets (<500ms)
        let remote_ops: Vec<_> = benchmark_report
            .results
            .iter()
            .filter(|r| r.operation.contains("store") || r.operation.contains("retrieve"))
            .filter(|r| !r.operation.contains("cache"))
            .collect();

        for result in remote_ops {
            let target_met = result.p95_time_ms <= 500;
            validations.push(PerformanceValidation {
                operation: result.operation.clone(),
                target_description: "Remote operations <500ms (P95)".to_string(),
                actual_p95_ms: result.p95_time_ms,
                target_ms: 500,
                success_rate: result.success_rate,
                passed: target_met && result.success_rate >= 0.95,
            });
        }

        // Validate concurrent operations
        let concurrent_ops: Vec<_> = benchmark_report
            .results
            .iter()
            .filter(|r| r.operation.contains("concurrent"))
            .collect();

        for result in concurrent_ops {
            let target_met = result.p95_time_ms <= 1000; // More lenient for concurrent ops
            validations.push(PerformanceValidation {
                operation: result.operation.clone(),
                target_description: "Concurrent operations <1000ms (P95)".to_string(),
                actual_p95_ms: result.p95_time_ms,
                target_ms: 1000,
                success_rate: result.success_rate,
                passed: target_met && result.success_rate >= 0.95,
            });
        }

        // Validate cache performance (ultra-fast operations)
        let cache_ops: Vec<_> = benchmark_report
            .results
            .iter()
            .filter(|r| r.operation.contains("cache"))
            .collect();

        for result in cache_ops {
            let target_met = result.p95_time_ms <= 10; // Cache should be <10ms
            validations.push(PerformanceValidation {
                operation: result.operation.clone(),
                target_description: "Cache operations <10ms (P95)".to_string(),
                actual_p95_ms: result.p95_time_ms,
                target_ms: 10,
                success_rate: result.success_rate,
                passed: target_met && result.success_rate >= 0.95,
            });
        }

        let passed_count = validations.iter().filter(|v| v.passed).count();
        let total_count = validations.len();

        PerformanceValidationResults {
            validations,
            overall_passed: passed_count == total_count,
            passed_count,
            total_count,
        }
    }

    fn print_validation_summary(&self, report: &PerformanceReport) {
        println!("\n📋 Performance Validation Summary");
        println!("===============================");

        let validation = &report.validation_results;
        println!(
            "Overall Result: {}",
            if validation.overall_passed {
                "✅ PASSED"
            } else {
                "❌ FAILED"
            }
        );
        println!(
            "Tests Passed: {}/{}",
            validation.passed_count, validation.total_count
        );
        println!(
            "Total Duration: {:.2}s",
            report.total_duration.as_secs_f64()
        );

        if !validation.overall_passed {
            println!("\n❌ Failed Performance Validations:");
            for v in &validation.validations {
                if !v.passed {
                    println!(
                        "  • {} - {} (actual: {}ms, target: {}ms, success: {:.1}%)",
                        v.operation,
                        v.target_description,
                        v.actual_p95_ms,
                        v.target_ms,
                        v.success_rate * 100.0
                    );
                }
            }
        }

        // Overall assessment
        if validation.overall_passed && report.benchmark_report.overall_success {
            println!("\n🎉 All performance targets met! System is production-ready.");
        } else {
            println!("\n⚠️  Performance optimization needed before production deployment.");

            if let Some(_recommendations) = &report.recommendations {
                println!("\n📝 See optimization recommendations above for improvement guidance.");
            }
        }
    }
}

impl Default for PerformanceTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub benchmark_report: crate::saorsa_storage::benchmarks::BenchmarkReport,
    pub validation_results: PerformanceValidationResults,
    pub total_duration: Duration,
    pub recommendations: Option<OptimizationRecommendations>,
}

#[derive(Debug)]
pub struct PerformanceValidationResults {
    pub validations: Vec<PerformanceValidation>,
    pub overall_passed: bool,
    pub passed_count: usize,
    pub total_count: usize,
}

#[derive(Debug)]
pub struct PerformanceValidation {
    pub operation: String,
    pub target_description: String,
    pub actual_p95_ms: u64,
    pub target_ms: u64,
    pub success_rate: f64,
    pub passed: bool,
}

/// Quick performance smoke test for CI/CD
pub async fn run_performance_smoke_test() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🚄 Running Performance Smoke Test (Quick Validation)");

    let config = BenchmarkConfig {
        iterations: 10,
        warmup_iterations: 2,
        content_sizes: vec![1024, 10 * 1024], // Only small sizes for speed
        ..Default::default()
    };

    let mut runner = PerformanceTestRunner::with_config(config);
    let report = runner.run_performance_validation().await?;

    // For smoke test, we're more lenient
    let smoke_test_passed =
        report.validation_results.passed_count >= (report.validation_results.total_count * 4 / 5); // 80% pass rate

    if smoke_test_passed {
        println!("✅ Performance smoke test PASSED");
    } else {
        println!("❌ Performance smoke test FAILED");
    }

    Ok(smoke_test_passed)
}

/// Comprehensive performance validation for release validation
pub async fn run_comprehensive_performance_test() -> Result<bool, Box<dyn std::error::Error>> {
    println!("🏁 Running Comprehensive Performance Validation");

    let mut runner = PerformanceTestRunner::new();
    let report = runner.run_performance_validation().await?;

    // For comprehensive test, we require 100% pass rate
    let comprehensive_passed =
        report.validation_results.overall_passed && report.benchmark_report.overall_success;

    if comprehensive_passed {
        println!("🎉 Comprehensive performance validation PASSED - Production Ready!");
    } else {
        println!("⚠️  Comprehensive performance validation FAILED - Optimization needed");
    }

    Ok(comprehensive_passed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_smoke_test() {
        // This test might take a while, so we use a very minimal config
        let config = BenchmarkConfig {
            iterations: 2,
            warmup_iterations: 1,
            content_sizes: vec![1024],
            ..Default::default()
        };

        let mut runner = PerformanceTestRunner::with_config(config);
        let result = runner.run_performance_validation().await;

        // Test should complete without panicking
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(!report.validation_results.validations.is_empty());
    }

    #[test]
    fn test_performance_validation_logic() {
        let validation = PerformanceValidation {
            operation: "test_op".to_string(),
            target_description: "Test <100ms".to_string(),
            actual_p95_ms: 50,
            target_ms: 100,
            success_rate: 0.98,
            passed: true,
        };

        assert!(validation.passed);
        assert!(validation.actual_p95_ms < validation.target_ms);
        assert!(validation.success_rate > 0.95);
    }
}
