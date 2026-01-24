// Copyright (c) 2025 Saorsa Labs Limited
//
// AI Agent Spawner
//
// Spawns and manages Claude Haiku subagents for distributed testing

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::config::{NodeConfig, TestContext, TestStep};
use crate::mcp_client::McpClient;

/// Manages AI subagent execution
#[derive(Clone)]
pub struct AgentSpawner {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AgentSpawner {
    /// Create a new agent spawner
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Run test steps for an actor on a specific node
    pub async fn run_steps(
        &self,
        actor: &str,
        node: &NodeConfig,
        steps: &[TestStep],
        context: std::sync::Arc<tokio::sync::Mutex<TestContext>>,
    ) -> Result<()> {
        let mcp_client = McpClient::new(&node.host, node.port)?;

        info!(
            "Running {} steps for {} on {}",
            steps.len(),
            actor,
            node.name
        );

        for (i, step) in steps.iter().enumerate() {
            debug!(
                "  Step {}: {} with params {:?}",
                i + 1,
                step.tool,
                step.params
            );

            // Substitute variables in params (lock briefly to read)
            let params: HashMap<String, serde_json::Value> = {
                let ctx = context.lock().await;
                step.params
                    .iter()
                    .map(|(k, v)| (k.clone(), ctx.substitute_json(v)))
                    .collect()
            };

            // Execute the MCP tool call
            let result = mcp_client.call_tool(&step.tool, &params).await?;

            // Verify expectations
            self.verify_expectations(step, &result)?;

            // Store variables if specified
            if let Some(store) = &step.store {
                let mut ctx = context.lock().await;
                for (var_name, json_path) in store {
                    // MCP responses have content in content[0].text as JSON string
                    if let Some(value) = extract_from_mcp_response(&result, json_path) {
                        ctx.set(var_name, value.clone());
                        debug!("    Stored {} = {:?}", var_name, value);
                    }
                }
            }

            debug!("  Step {} completed successfully", i + 1);
        }

        info!("All steps completed for {} on {}", actor, node.name);
        Ok(())
    }

    /// Verify that a result matches expectations
    fn verify_expectations(&self, step: &TestStep, result: &serde_json::Value) -> Result<()> {
        let expect = &step.expect;

        // Check status if specified
        // MCP uses isError: true/false, so derive status from that
        if let Some(expected_status) = &expect.status {
            let actual_status = if let Some(is_error) = result.get("isError") {
                if is_error.as_bool().unwrap_or(false) {
                    "error"
                } else {
                    "success"
                }
            } else {
                result
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            };

            if actual_status != expected_status {
                anyhow::bail!(
                    "Expected status '{}', got '{}'",
                    expected_status,
                    actual_status
                );
            }
        }

        // Check error message if expecting an error
        if let Some(error_substring) = &expect.error_contains {
            // MCP error messages are in content[0].text when isError is true
            let error_msg = result
                .get("content")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|item| item.get("text"))
                .and_then(|t| t.as_str())
                .or_else(|| result.get("error").and_then(|v| v.as_str()))
                .unwrap_or("");

            if !error_msg.contains(error_substring) {
                anyhow::bail!(
                    "Expected error containing '{}', got '{}'",
                    error_substring,
                    error_msg
                );
            }
        }

        // Check that result contains expected values
        if let Some(contains) = &expect.contains {
            let result_array = result.get("result").or(Some(result));

            for expected in contains {
                if !json_contains(result_array, expected) {
                    anyhow::bail!(
                        "Expected result to contain {:?}, got {:?}",
                        expected,
                        result
                    );
                }
            }
        }

        // Check specific field values
        for (field, expected_value) in &expect.fields {
            let actual_value = extract_json_path(result, field);

            if actual_value != Some(expected_value) {
                anyhow::bail!(
                    "Expected field '{}' to be {:?}, got {:?}",
                    field,
                    expected_value,
                    actual_value
                );
            }
        }

        Ok(())
    }

    /// Execute an AI agent to run complex test scenarios
    /// This uses the Anthropic API to spawn a Haiku subagent
    #[allow(dead_code)]
    async fn spawn_ai_agent(
        &self,
        actor: &str,
        node: &NodeConfig,
        system_prompt: &str,
        task: &str,
    ) -> Result<AgentResponse> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: vec![Message {
                role: "user".to_string(),
                content: task.to_string(),
            }],
            system: Some(system_prompt.to_string()),
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, body);
        }

        let api_response: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic response")?;

        // Extract the response text
        let text = api_response
            .content
            .iter()
            .filter_map(|c| {
                if c.content_type == "text" {
                    Some(c.text.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(AgentResponse {
            actor: actor.to_string(),
            node: node.name.clone(),
            response: text,
            usage: api_response.usage,
        })
    }

    /// Generate a system prompt for the test agent
    #[allow(dead_code)]
    fn generate_system_prompt(&self, actor: &str, node: &NodeConfig, phase: &str) -> String {
        format!(
            r#"# Test Agent: {actor}

You are {actor}, a test user in the Communitas distributed testing framework.

## Your Identity
- Connected to: http://{}:{}

## Current Phase
{phase}

## Instructions
1. Execute each MCP tool call as instructed
2. Report results clearly
3. If a step fails, report the error and stop
4. Be precise in verifying expected results

## Output Format
For each step, output:
STEP: <step number>
TOOL: <tool name>
RESULT: <success|error>
DETAILS: <relevant details>
"#,
            node.host, node.port
        )
    }
}

/// Response from an AI agent execution
#[derive(Debug)]
#[allow(dead_code)]
pub struct AgentResponse {
    pub actor: String,
    pub node: String,
    pub response: String,
    pub usage: Usage,
}

/// Anthropic API request structure
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// Anthropic API response structure
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Extract a value from JSON using a simple path (e.g., "result.id")
fn extract_json_path<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    let mut current = value;

    for part in path.split('.') {
        current = match part.parse::<usize>() {
            Ok(index) => current.get(index)?,
            Err(_) => current.get(part)?,
        };
    }

    Some(current)
}

/// Extract a value from an MCP response format
/// MCP responses have the actual data inside `content[0].text` as a JSON string
fn extract_from_mcp_response(result: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    // First try to get the content text and parse it as JSON
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())?;

    // Parse the text content as JSON
    let parsed: serde_json::Value = serde_json::from_str(text).ok()?;

    // Extract the value at the given path
    let mut current = &parsed;
    for part in path.split('.') {
        current = match part.parse::<usize>() {
            Ok(index) => current.get(index)?,
            Err(_) => current.get(part)?,
        };
    }

    Some(current.clone())
}

/// Check if a JSON value contains another (for array/object containment checks)
fn json_contains(haystack: Option<&serde_json::Value>, needle: &serde_json::Value) -> bool {
    let Some(haystack) = haystack else {
        return false;
    };

    match (haystack, needle) {
        (serde_json::Value::Array(arr), needle) => {
            arr.iter().any(|item| json_contains(Some(item), needle))
        }
        (serde_json::Value::Object(obj1), serde_json::Value::Object(obj2)) => {
            // Check if all fields in needle exist and match in haystack
            obj2.iter()
                .all(|(k, v)| obj1.get(k).is_some_and(|hv| json_contains(Some(hv), v)))
        }
        (a, b) => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_path() {
        let json = serde_json::json!({
            "result": {
                "id": "entity-123",
                "name": "Test"
            },
            "status": "success"
        });

        assert_eq!(
            extract_json_path(&json, "status"),
            Some(&serde_json::json!("success"))
        );
        assert_eq!(
            extract_json_path(&json, "result.id"),
            Some(&serde_json::json!("entity-123"))
        );
        assert_eq!(extract_json_path(&json, "nonexistent"), None);
    }

    #[test]
    fn test_json_contains() {
        let arr = serde_json::json!([
            {"name": "Alice", "role": "admin"},
            {"name": "Bob", "role": "member"}
        ]);

        assert!(json_contains(
            Some(&arr),
            &serde_json::json!({"name": "Alice"})
        ));
        assert!(json_contains(
            Some(&arr),
            &serde_json::json!({"role": "member"})
        ));
        assert!(!json_contains(
            Some(&arr),
            &serde_json::json!({"name": "Charlie"})
        ));
    }
}
