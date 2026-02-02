// Copyright (c) 2025 Saorsa Labs Limited
//
// Distributed MCP Test Orchestrator
//
// Orchestrates distributed testing across multiple nodes using AI subagents

mod agent_spawner;
mod config;
mod mcp_client;
mod report;
mod sync_barrier;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::agent_spawner::AgentSpawner;
use crate::config::{NodeConfig, TestConfig, TestContext};
use crate::mcp_client::McpClient;
use crate::report::ReportGenerator;
use crate::sync_barrier::SyncBarrier;

/// Distributed MCP Test Orchestrator
///
/// Runs comprehensive tests across multiple MCP nodes using AI subagents
#[derive(Parser, Debug)]
#[command(name = "distributed-test-orchestrator")]
#[command(about = "Orchestrates distributed MCP tests using AI subagents")]
struct Args {
    /// Path to the test scenario YAML file
    #[arg(short, long)]
    config: PathBuf,

    /// Node configuration in format: name:host:port,name:host:port,...
    #[arg(short, long)]
    nodes: String,

    /// Output directory for reports
    #[arg(short, long, default_value = "tests/distributed/reports")]
    output: PathBuf,

    /// Anthropic API key (defaults to ANTHROPIC_API_KEY env var, optional for basic tests)
    #[arg(long, env = "ANTHROPIC_API_KEY", default_value = "")]
    anthropic_key: String,

    /// Model to use for AI subagents
    #[arg(long, default_value = "claude-3-haiku-20240307")]
    model: String,

    /// Override the anthropic-compatible API base URL (e.g. <https://api.kimi.com/coding/v1>)
    #[arg(long, env = "ANTHROPIC_BASE_URL")]
    api_base: Option<String>,

    /// Comma-separated scopes to request when auto-unlocking vaults
    #[arg(long, env = "UNLOCK_SCOPES", default_value = "full_access")]
    unlock_scopes: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run - parse and validate without executing
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    info!("Distributed MCP Test Orchestrator starting");

    // Parse node configuration
    let nodes = parse_nodes(&args.nodes)?;
    info!("Configured {} nodes", nodes.len());

    // Load test scenario
    let test_config = TestConfig::load(&args.config)
        .with_context(|| format!("Failed to load config from {:?}", args.config))?;
    info!(
        "Loaded scenario: {} ({} test cases)",
        test_config.name,
        test_config.test_cases.len()
    );

    let api_base = args
        .api_base
        .clone()
        .or_else(|| std::env::var("KIMI_API_BASE_URL").ok())
        .unwrap_or_else(|| "https://api.anthropic.com/v1".to_string());
    let unlock_scopes: Vec<String> = args
        .unlock_scopes
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if args.dry_run {
        info!("Dry run complete - scenario is valid");
        return Ok(());
    }

    // Verify all nodes are healthy
    verify_nodes_healthy(&nodes).await?;

    // Create orchestrator components
    let agent_spawner =
        AgentSpawner::new(&args.anthropic_key, &args.model, &api_base, unlock_scopes);
    let sync_barrier = SyncBarrier::new(nodes.len());
    let report_generator = ReportGenerator::new(&args.output);

    // Run the test scenario
    let results = run_scenario(&test_config, &nodes, &agent_spawner, &sync_barrier).await?;
    let _unlock_events = agent_spawner.unlock_events().await;

    // Generate reports
    report_generator.generate(&test_config, &results).await?;

    // Print summary
    print_summary(&results);

    // Exit with error code if any tests failed
    let failed = results.iter().any(|r| !r.passed);
    if failed {
        std::process::exit(1);
    }

    Ok(())
}

fn parse_nodes(nodes_str: &str) -> Result<Vec<NodeConfig>> {
    nodes_str
        .split(',')
        .map(|node| {
            let parts: Vec<&str> = node.trim().split(':').collect();
            if parts.len() != 3 {
                anyhow::bail!("Invalid node format: '{}'. Expected 'name:host:port'", node);
            }
            Ok(NodeConfig {
                name: parts[0].to_string(),
                host: parts[1].to_string(),
                port: parts[2]
                    .parse()
                    .with_context(|| format!("Invalid port: {}", parts[2]))?,
            })
        })
        .collect()
}

async fn verify_nodes_healthy(nodes: &[NodeConfig]) -> Result<()> {
    info!("Verifying node health...");

    for node in nodes {
        let client = McpClient::new(&node.host, node.port)?;
        match client.health_check().await {
            Ok(()) => info!("  {} ({}:{}): healthy", node.name, node.host, node.port),
            Err(e) => {
                anyhow::bail!(
                    "Node {} ({}:{}) is not healthy: {}",
                    node.name,
                    node.host,
                    node.port,
                    e
                );
            }
        }
    }

    info!("All {} nodes healthy", nodes.len());
    Ok(())
}

async fn run_scenario(
    config: &TestConfig,
    nodes: &[NodeConfig],
    agent_spawner: &AgentSpawner,
    sync_barrier: &SyncBarrier,
) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();
    // Shared context across all test cases for variable passing
    let context = std::sync::Arc::new(tokio::sync::Mutex::new(TestContext::new()));

    for test_case in &config.test_cases {
        info!("Running test case: {} - {}", test_case.id, test_case.name);

        let start = Instant::now();
        let result = run_test_case(
            test_case,
            nodes,
            agent_spawner,
            sync_barrier,
            context.clone(),
        )
        .await;
        let duration_ms = start.elapsed().as_millis() as u64;

        let passed = result.is_ok();
        let error_message = result.as_ref().err().map(|e| e.to_string());

        if passed {
            info!("  PASS: {}", test_case.id);
        } else {
            info!(
                "  FAIL: {} - {}",
                test_case.id,
                error_message.as_deref().unwrap_or("unknown error")
            );
        }

        results.push(TestResult {
            test_id: test_case.id.clone(),
            test_name: test_case.name.clone(),
            passed,
            error_message,
            duration_ms,
        });

        // Sync barrier between test cases
        sync_barrier.wait().await;
    }

    Ok(results)
}

async fn run_test_case(
    test_case: &config::TestCase,
    nodes: &[NodeConfig],
    agent_spawner: &AgentSpawner,
    sync_barrier: &SyncBarrier,
    context: std::sync::Arc<tokio::sync::Mutex<TestContext>>,
) -> Result<()> {
    // Find the node for each actor
    let actor_nodes: Vec<_> = test_case
        .actors
        .iter()
        .filter_map(|actor| {
            nodes
                .iter()
                .find(|n| n.name.to_lowercase() == actor.to_lowercase())
        })
        .collect();

    if actor_nodes.len() != test_case.actors.len() {
        anyhow::bail!(
            "Could not find nodes for all actors: {:?}",
            test_case.actors
        );
    }

    if test_case.parallel {
        // Run all actors in parallel
        let mut handles = Vec::new();

        for (actor, node) in test_case.actors.iter().zip(actor_nodes.iter()) {
            let steps = test_case.steps.clone();
            let actor = actor.clone();
            let node = (*node).clone();
            let spawner = agent_spawner.clone();
            let ctx = context.clone();

            let handle =
                tokio::spawn(async move { spawner.run_steps(&actor, &node, &steps, ctx).await });

            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await??;
        }
    } else {
        // Run sequentially
        for (actor, node) in test_case.actors.iter().zip(actor_nodes.iter()) {
            agent_spawner
                .run_steps(actor, node, &test_case.steps, context.clone())
                .await?;
        }
    }

    // Wait at sync barrier if specified
    if test_case.wait_ms > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(test_case.wait_ms)).await;
    }

    sync_barrier.wait().await;

    Ok(())
}

fn print_summary(results: &[TestResult]) {
    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    println!();
    println!("=====================================");
    println!("           TEST SUMMARY              ");
    println!("=====================================");
    println!("Total:  {}", total);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("=====================================");

    if failed > 0 {
        println!();
        println!("Failed tests:");
        for result in results.iter().filter(|r| !r.passed) {
            println!(
                "  - {} ({}): {}",
                result.test_id,
                result.test_name,
                result.error_message.as_deref().unwrap_or("unknown error")
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub test_name: String,
    pub passed: bool,
    pub error_message: Option<String>,
    pub duration_ms: u64,
}
