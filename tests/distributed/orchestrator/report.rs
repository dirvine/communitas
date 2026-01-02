// Copyright (c) 2025 Saorsa Labs Limited
//
// Report Generator
//
// Generates HTML and JSON reports from test results

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::config::TestConfig;
use crate::TestResult;

/// Generates test reports in multiple formats
pub struct ReportGenerator {
    output_dir: PathBuf,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new(output_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
        }
    }

    /// Generate all report formats
    pub async fn generate(&self, config: &TestConfig, results: &[TestResult]) -> Result<()> {
        // Ensure output directory exists
        std::fs::create_dir_all(&self.output_dir)
            .with_context(|| format!("Failed to create output directory: {:?}", self.output_dir))?;

        // Generate JSON report
        self.generate_json(config, results).await?;

        // Generate HTML report
        self.generate_html(config, results).await?;

        Ok(())
    }

    /// Generate JSON report
    async fn generate_json(&self, config: &TestConfig, results: &[TestResult]) -> Result<()> {
        let report = JsonReport {
            timestamp: Utc::now(),
            scenario_name: config.name.clone(),
            tools_covered: config.tools_covered.clone(),
            total_tests: results.len(),
            passed: results.iter().filter(|r| r.passed).count(),
            failed: results.iter().filter(|r| !r.passed).count(),
            results: results
                .iter()
                .map(|r| JsonTestResult {
                    test_id: r.test_id.clone(),
                    test_name: r.test_name.clone(),
                    passed: r.passed,
                    error_message: r.error_message.clone(),
                    duration_ms: r.duration_ms,
                })
                .collect(),
        };

        let json_path = self.output_dir.join("report.json");
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&json_path, json)
            .with_context(|| format!("Failed to write JSON report to {:?}", json_path))?;

        tracing::info!("JSON report written to {:?}", json_path);
        Ok(())
    }

    /// Generate HTML report
    async fn generate_html(&self, config: &TestConfig, results: &[TestResult]) -> Result<()> {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        let pass_rate = if results.is_empty() {
            0.0
        } else {
            (passed as f64 / results.len() as f64) * 100.0
        };

        let status_class = if failed == 0 { "success" } else { "failure" };

        let mut test_rows = String::new();
        for result in results {
            let status_icon = if result.passed { "✅" } else { "❌" };
            let row_class = if result.passed { "pass" } else { "fail" };
            let error_cell = result
                .error_message
                .as_ref()
                .map(|e| format!("<td class=\"error\">{}</td>", html_escape(e)))
                .unwrap_or_else(|| "<td>-</td>".to_string());

            test_rows.push_str(&format!(
                r#"        <tr class="{}">
          <td>{}</td>
          <td>{}</td>
          <td>{}</td>
          {}
          <td>{}ms</td>
        </tr>
"#,
                row_class,
                status_icon,
                html_escape(&result.test_id),
                html_escape(&result.test_name),
                error_cell,
                result.duration_ms
            ));
        }

        let tools_list = config
            .tools_covered
            .iter()
            .map(|t| format!("<li>{}</li>", html_escape(t)))
            .collect::<Vec<_>>()
            .join("\n          ");

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Distributed MCP Test Report</title>
  <style>
    :root {{
      --success-color: #22c55e;
      --failure-color: #ef4444;
      --bg-color: #1a1a2e;
      --card-bg: #16213e;
      --text-color: #e5e7eb;
      --border-color: #374151;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--bg-color);
      color: var(--text-color);
      margin: 0;
      padding: 2rem;
    }}
    .container {{
      max-width: 1200px;
      margin: 0 auto;
    }}
    h1 {{
      color: #f1f5f9;
      margin-bottom: 0.5rem;
    }}
    .timestamp {{
      color: #9ca3af;
      font-size: 0.875rem;
      margin-bottom: 2rem;
    }}
    .summary {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
    }}
    .card {{
      background: var(--card-bg);
      border-radius: 0.5rem;
      padding: 1.5rem;
      border: 1px solid var(--border-color);
    }}
    .card h3 {{
      margin: 0 0 0.5rem 0;
      font-size: 0.875rem;
      color: #9ca3af;
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }}
    .card .value {{
      font-size: 2rem;
      font-weight: bold;
    }}
    .card.success .value {{ color: var(--success-color); }}
    .card.failure .value {{ color: var(--failure-color); }}
    .tools {{
      background: var(--card-bg);
      border-radius: 0.5rem;
      padding: 1.5rem;
      border: 1px solid var(--border-color);
      margin-bottom: 2rem;
    }}
    .tools h2 {{
      margin-top: 0;
    }}
    .tools ul {{
      columns: 3;
      list-style: none;
      padding: 0;
      margin: 0;
    }}
    .tools li {{
      padding: 0.25rem 0;
      font-family: monospace;
      font-size: 0.875rem;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      background: var(--card-bg);
      border-radius: 0.5rem;
      overflow: hidden;
    }}
    th, td {{
      padding: 1rem;
      text-align: left;
      border-bottom: 1px solid var(--border-color);
    }}
    th {{
      background: #0f172a;
      font-weight: 600;
      text-transform: uppercase;
      font-size: 0.75rem;
      letter-spacing: 0.05em;
    }}
    tr.pass {{ background: rgba(34, 197, 94, 0.1); }}
    tr.fail {{ background: rgba(239, 68, 68, 0.1); }}
    .error {{
      color: var(--failure-color);
      font-size: 0.875rem;
    }}
  </style>
</head>
<body>
  <div class="container">
    <h1>{}</h1>
    <div class="timestamp">Generated: {}</div>

    <div class="summary">
      <div class="card {}">
        <h3>Status</h3>
        <div class="value">{}</div>
      </div>
      <div class="card">
        <h3>Total Tests</h3>
        <div class="value">{}</div>
      </div>
      <div class="card success">
        <h3>Passed</h3>
        <div class="value">{}</div>
      </div>
      <div class="card failure">
        <h3>Failed</h3>
        <div class="value">{}</div>
      </div>
      <div class="card">
        <h3>Pass Rate</h3>
        <div class="value">{:.1}%</div>
      </div>
    </div>

    <div class="tools">
      <h2>Tools Covered</h2>
      <ul>
        {}
      </ul>
    </div>

    <h2>Test Results</h2>
    <table>
      <thead>
        <tr>
          <th>Status</th>
          <th>Test ID</th>
          <th>Name</th>
          <th>Error</th>
          <th>Duration</th>
        </tr>
      </thead>
      <tbody>
{}
      </tbody>
    </table>
  </div>
</body>
</html>"#,
            html_escape(&config.name),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            status_class,
            if failed == 0 { "PASS" } else { "FAIL" },
            results.len(),
            passed,
            failed,
            pass_rate,
            tools_list,
            test_rows
        );

        let html_path = self.output_dir.join("report.html");
        std::fs::write(&html_path, html)
            .with_context(|| format!("Failed to write HTML report to {:?}", html_path))?;

        tracing::info!("HTML report written to {:?}", html_path);
        Ok(())
    }
}

/// JSON report structure
#[derive(Debug, Serialize)]
struct JsonReport {
    timestamp: DateTime<Utc>,
    scenario_name: String,
    tools_covered: Vec<String>,
    total_tests: usize,
    passed: usize,
    failed: usize,
    results: Vec<JsonTestResult>,
}

#[derive(Debug, Serialize)]
struct JsonTestResult {
    test_id: String,
    test_name: String,
    passed: bool,
    error_message: Option<String>,
    duration_ms: u64,
}

/// Escape HTML special characters
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    #[tokio::test]
    async fn test_report_generation() {
        let temp_dir = std::env::temp_dir().join("test_report");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let generator = ReportGenerator::new(&temp_dir);

        let config = TestConfig {
            name: "Test Scenario".to_string(),
            description: "Test description".to_string(),
            tools_covered: vec!["health_check".to_string(), "create_vault".to_string()],
            test_cases: vec![],
        };

        let results = vec![
            TestResult {
                test_id: "TEST-001".to_string(),
                test_name: "Health check".to_string(),
                passed: true,
                error_message: None,
                duration_ms: 50,
            },
            TestResult {
                test_id: "TEST-002".to_string(),
                test_name: "Create vault".to_string(),
                passed: false,
                error_message: Some("Connection refused".to_string()),
                duration_ms: 100,
            },
        ];

        generator.generate(&config, &results).await.unwrap();

        assert!(temp_dir.join("report.json").exists());
        assert!(temp_dir.join("report.html").exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
