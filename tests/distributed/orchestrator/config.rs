// Copyright (c) 2025 Saorsa Labs Limited
//
// Configuration and YAML scenario parsing

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Test scenario configuration loaded from YAML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestConfig {
    /// Name of the test scenario
    pub name: String,

    /// Description of what this scenario tests
    #[serde(default)]
    pub description: String,

    /// List of MCP tools covered by this scenario
    #[serde(default)]
    pub tools_covered: Vec<String>,

    /// List of test cases to execute
    pub test_cases: Vec<TestCase>,
}

impl TestConfig {
    /// Load test configuration from a YAML file
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;

        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML from {:?}", path))
    }

    /// Load all scenario files from a directory
    #[allow(dead_code)]
    pub fn load_all(dir: &Path) -> Result<Vec<Self>> {
        let mut configs = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
                configs.push(Self::load(&path)?);
            }
        }

        // Sort by filename to ensure consistent ordering
        configs.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(configs)
    }
}

/// A single test case within a scenario
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCase {
    /// Unique identifier for this test case
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Which actors (nodes) participate in this test
    pub actors: Vec<String>,

    /// Whether actors run in parallel or sequentially
    #[serde(default)]
    pub parallel: bool,

    /// Milliseconds to wait after steps complete
    #[serde(default)]
    pub wait_ms: u64,

    /// Steps to execute
    pub steps: Vec<TestStep>,
}

/// A single step within a test case
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestStep {
    /// The MCP tool to call
    pub tool: String,

    /// Parameters for the tool call
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Expected result
    #[serde(default)]
    pub expect: ExpectedResult,

    /// Variable to store result in for later use
    #[serde(default)]
    pub store: Option<HashMap<String, String>>,

    /// Actor override (if different from test case actors)
    #[serde(default)]
    pub actor: Option<String>,
}

/// Expected result from a test step
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ExpectedResult {
    /// Expected status (success, error)
    #[serde(default)]
    pub status: Option<String>,

    /// Expected error message substring
    #[serde(default)]
    pub error_contains: Option<String>,

    /// Expected response fields
    #[serde(default)]
    pub contains: Option<Vec<serde_json::Value>>,

    /// Field-level assertions
    #[serde(default)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Node configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeConfig {
    /// Name of this node (e.g., "alice", "bob")
    pub name: String,

    /// Hostname or IP address
    pub host: String,

    /// Port number
    pub port: u16,
}

impl NodeConfig {
    /// Get the base URL for this node
    #[allow(dead_code)]
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Agent persona configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentPersona {
    /// Name of the agent (e.g., "Alice")
    pub name: String,

    /// Display name shown in UI
    pub display_name: String,

    /// Four-word identity
    pub identity: String,

    /// Node this agent connects to
    pub node: String,

    /// Additional persona traits
    #[serde(default)]
    pub traits: Vec<String>,
}

impl AgentPersona {
    /// Load persona from YAML file
    #[allow(dead_code)]
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;

        serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse persona from {:?}", path))
    }
}

/// Shared test context with variables
#[derive(Debug, Clone, Default)]
pub struct TestContext {
    /// Variables stored from previous steps
    pub variables: HashMap<String, serde_json::Value>,

    /// Current phase name
    #[allow(dead_code)]
    pub phase_name: String,
}

impl TestContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Store a variable
    pub fn set(&mut self, key: &str, value: serde_json::Value) {
        self.variables.insert(key.to_string(), value);
    }

    /// Get a variable
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }

    /// Substitute variables in a string (e.g., "${org_id}" -> actual value)
    pub fn substitute(&self, s: &str) -> String {
        let mut result = s.to_string();

        for (key, value) in &self.variables {
            let placeholder = format!("${{{}}}", key);
            if let Some(str_val) = value.as_str() {
                result = result.replace(&placeholder, str_val);
            } else {
                result = result.replace(&placeholder, &value.to_string());
            }
        }

        result
    }

    /// Substitute variables in a JSON value
    pub fn substitute_json(&self, value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::String(s) => serde_json::Value::String(self.substitute(s)),
            serde_json::Value::Object(obj) => {
                let mut new_obj = serde_json::Map::new();
                for (k, v) in obj {
                    new_obj.insert(k.clone(), self.substitute_json(v));
                }
                serde_json::Value::Object(new_obj)
            }
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| self.substitute_json(v)).collect())
            }
            _ => value.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_scenario() {
        let yaml = r#"
name: "Test Scenario"
description: "A test scenario"
tools_covered: [health_check, create_vault]
test_cases:
  - id: TEST-001
    name: "Basic health check"
    actors: [alice]
    steps:
      - tool: health_check
        expect:
          status: success
"#;

        let config: TestConfig = serde_yaml::from_str(yaml).expect("Failed to parse");
        assert_eq!(config.name, "Test Scenario");
        assert_eq!(config.test_cases.len(), 1);
        assert_eq!(config.test_cases[0].id, "TEST-001");
    }

    #[test]
    fn test_context_substitution() {
        let mut ctx = TestContext::new();
        ctx.set("org_id", serde_json::json!("org-123"));
        ctx.set("user_name", serde_json::json!("Alice"));

        assert_eq!(ctx.substitute("Entity: ${org_id}"), "Entity: org-123");
        assert_eq!(ctx.substitute("Hello ${user_name}!"), "Hello Alice!");
    }
}
