// Copyright (c) 2025 Saorsa Labs Limited
//
// AI Agent Spawner
//
// Spawns and manages Claude Haiku subagents for distributed testing

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::config::{ExpectedResult, NodeConfig, TestContext, TestStep};
use crate::mcp_client::McpClient;

/// Manages AI subagent execution
#[derive(Clone)]
pub struct AgentSpawner {
    api_key: String,
    model: String,
    api_base: String,
    client: reqwest::Client,
    unlock_scopes: Vec<String>,
    unlock_cache: Arc<Mutex<HashMap<String, UnlockLeaseCache>>>,
    unlock_refresh_margin: Duration,
    unlock_events: Arc<Mutex<Vec<UnlockEvent>>>,
}

/// Unlock lease telemetry collected during orchestrated runs
#[derive(Debug, Clone, Serialize)]
pub struct UnlockEvent {
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub node: String,
    pub event: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

impl AgentSpawner {
    /// Create a new agent spawner
    pub fn new(api_key: &str, model: &str, api_base: &str, unlock_scopes: Vec<String>) -> Self {
        let normalized_base = api_base.trim_end_matches('/').to_string();
        let scopes = if unlock_scopes.is_empty() {
            vec!["full_access".to_string()]
        } else {
            unlock_scopes
        };
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            api_base: if normalized_base.is_empty() {
                "https://api.anthropic.com/v1".to_string()
            } else {
                normalized_base
            },
            client: reqwest::Client::new(),
            unlock_scopes: scopes,
            unlock_cache: Arc::new(Mutex::new(HashMap::new())),
            unlock_refresh_margin: Duration::from_secs(60),
            unlock_events: Arc::new(Mutex::new(Vec::new())),
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

            if step.tool == "unlock_actor" {
                info!(
                    "  Step {}: unlock_actor to refresh lease for {} on {}",
                    i + 1,
                    actor,
                    node.name
                );
                self.ensure_unlocked(actor, node, &mcp_client).await?;
                continue;
            }

            if step.tool == "ensure_network_started" {
                info!(
                    "  Step {}: ensure_network_started for {} on {}",
                    i + 1,
                    actor,
                    node.name
                );
                self.ensure_network_started(actor, node, &mcp_client)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to start networking for actor {} on {}",
                            actor, node.name
                        )
                    })?;
                continue;
            }

            if step.tool == "wait" {
                let duration_ms = step
                    .params
                    .get("ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1000);
                info!(
                    "  Step {}: wait {}ms for {} on {}",
                    i + 1,
                    duration_ms,
                    actor,
                    node.name
                );
                tokio::time::sleep(std::time::Duration::from_millis(duration_ms)).await;
                continue;
            }

            if Self::requires_unlock(&step.tool) {
                self.ensure_unlocked(actor, node, &mcp_client).await?;
            }

            // Substitute variables in params (lock briefly to read)
            let params: HashMap<String, serde_json::Value> = {
                let ctx = context.lock().await;
                step.params
                    .iter()
                    .map(|(k, v)| (k.clone(), ctx.substitute_json(v)))
                    .collect()
            };

            // Execute the MCP tool call
            let result = match mcp_client.call_tool(&step.tool, &params).await {
                Ok(result) => result,
                Err(err) => {
                    if err.to_string().contains("unlock required") {
                        self.log_unlock_event(
                            actor,
                            &node.name,
                            "unlock_retry",
                            format!("lease expired while calling {}", step.tool),
                            None,
                        )
                        .await;
                        self.ensure_unlocked(actor, node, &mcp_client).await?;
                        mcp_client.call_tool(&step.tool, &params).await?
                    } else {
                        return Err(err);
                    }
                }
            };

            // Verify expectations (with variable substitution)
            let substituted_expect = {
                let ctx = context.lock().await;
                &step.expect
            };
            self.verify_expectations(&substituted_expect, &result)?;

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

            if Self::tool_resets_unlock(&step.tool) {
                self.invalidate_unlock(actor, &node.name).await;
            }
        }

        info!("All steps completed for {} on {}", actor, node.name);
        Ok(())
    }

    fn requires_unlock(tool: &str) -> bool {
        // Align with server-side UNLOCK_EXEMPT_TOOLS in communitas-mcp/src/auth.rs
        !matches!(
            tool,
            "health_check"
                | "create_vault"
                | "import_vault"
                | "list_vaults"
                | "authenticate"
                | "authenticate_token"
                | "get_session"
                | "logout"
                | "create_delegate_token"
                | "create_unlock_grant"
                | "get_unlock_status"
                | "core_status"
        )
    }

    fn tool_resets_unlock(tool: &str) -> bool {
        matches!(
            tool,
            "logout" | "authenticate" | "authenticate_token" | "create_vault" | "import_vault"
        )
    }

    async fn invalidate_unlock(&self, actor: &str, node: &str) {
        let mut cache = self.unlock_cache.lock().await;
        if cache.remove(actor).is_some() {
            info!("Cleared unlock lease cache for {}", actor);
            self.log_unlock_event(actor, node, "cache_invalidated", "lease reset", None)
                .await;
        }
    }

    async fn ensure_unlocked(
        &self,
        actor: &str,
        node: &NodeConfig,
        client: &McpClient,
    ) -> Result<()> {
        {
            let cache = self.unlock_cache.lock().await;
            if let Some(lease) = cache.get(actor) {
                let remaining = lease
                    .expires_at
                    .saturating_duration_since(Instant::now())
                    .as_secs();
                if Instant::now() + self.unlock_refresh_margin < lease.expires_at {
                    self.log_unlock_event(
                        actor,
                        &node.name,
                        "cache_hit",
                        format!("existing lease valid for {}s", remaining),
                        Some(remaining),
                    )
                    .await;
                    return Ok(());
                }
            }
        }
        self.log_unlock_event(
            actor,
            &node.name,
            "unlock_refresh",
            "requesting new unlock lease",
            None,
        )
        .await;

        let lease = match self.request_unlock(actor, &node.name, client).await {
            Ok(lease) => lease,
            Err(err) => {
                self.log_unlock_event(
                    actor,
                    &node.name,
                    "unlock_error",
                    format!("create_unlock_grant failed: {err}"),
                    None,
                )
                .await;
                return Err(err);
            }
        };

        if let Err(err) = self.verify_unlock_status(actor, &node.name, client).await {
            self.log_unlock_event(
                actor,
                &node.name,
                "unlock_error",
                format!("get_unlock_status failed: {err}"),
                None,
            )
            .await;
            return Err(err);
        }

        let ttl_secs = lease
            .expires_at
            .saturating_duration_since(Instant::now())
            .as_secs();
        if ttl_secs < 120 {
            warn!(
                "Unlock lease for {} expires soon ({}s remaining); refreshing early",
                actor, ttl_secs
            );
        } else {
            info!(
                "Unlock lease refreshed for {} ({}s remaining)",
                actor, ttl_secs
            );
        }

        self.log_unlock_event(
            actor,
            &node.name,
            "unlock_granted",
            format!("lease expires in {} seconds", ttl_secs),
            Some(ttl_secs),
        )
        .await;

        let mut cache = self.unlock_cache.lock().await;
        cache.insert(actor.to_string(), lease);
        Ok(())
    }

    async fn ensure_network_started(
        &self,
        actor: &str,
        node: &NodeConfig,
        client: &McpClient,
    ) -> Result<()> {
        let status = client.call_tool("network_status", &HashMap::new()).await;
        if let Ok(result) = status {
            if let Some(active) =
                extract_from_mcp_response(&result, "active").and_then(|v| v.as_bool())
            {
                if active {
                    self.log_unlock_event(
                        actor,
                        &node.name,
                        "network_already_active",
                        "network already running",
                        None,
                    )
                    .await;
                    return Ok(());
                }
            }
        }

        // network_start requires an unlock lease, so ensure we have one
        self.ensure_unlocked(actor, node, client).await?;

        let mut params = HashMap::new();
        params.insert("preferred_port".to_string(), serde_json::json!(0));
        let result = client
            .call_tool("network_start", &params)
            .await
            .context("network_start RPC failed")?;

        if let Some(parsed) =
            extract_from_mcp_response(&result, "success").and_then(|v| v.as_bool())
        {
            if parsed {
                self.log_unlock_event(
                    actor,
                    &node.name,
                    "network_started",
                    "network_start succeeded",
                    None,
                )
                .await;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                return Ok(());
            }
        }

        let err_msg = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown error");

        Err(anyhow!("network_start failed: {}", err_msg))
    }

    async fn request_unlock(
        &self,
        actor: &str,
        node: &str,
        client: &McpClient,
    ) -> Result<UnlockLeaseCache> {
        let mut params = HashMap::new();
        params.insert(
            "request_hash".to_string(),
            serde_json::json!(format!("unlock-{actor}-{}", Uuid::new_v4())),
        );
        params.insert(
            "scopes".to_string(),
            serde_json::json!(self.unlock_scopes.clone()),
        );
        params.insert("max_total_seconds".to_string(), serde_json::json!(0));

        let response = client
            .call_tool("create_unlock_grant", &params)
            .await
            .with_context(|| format!("Failed to unlock actor {}", actor))?;
        let payload =
            Self::parse_payload(&response).context("create_unlock_grant missing payload")?;

        let expires_at_unix = payload
            .get("expires_at")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow!("Missing expires_at in unlock response"))?;

        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let ttl_secs = expires_at_unix.saturating_sub(now_unix);

        self.log_unlock_event(
            actor,
            node,
            "create_unlock_grant",
            format!("lease ttl {}s", ttl_secs),
            Some(ttl_secs),
        )
        .await;

        Ok(UnlockLeaseCache {
            lease_id: payload
                .get("lease_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            expires_at: Instant::now() + Duration::from_secs(ttl_secs.max(1)),
        })
    }

    async fn verify_unlock_status(
        &self,
        actor: &str,
        node: &str,
        client: &McpClient,
    ) -> Result<()> {
        let params = HashMap::new();
        let response = client
            .call_tool("get_unlock_status", &params)
            .await
            .with_context(|| format!("get_unlock_status failed for {actor}"))?;
        let payload =
            Self::parse_payload(&response).context("get_unlock_status missing payload")?;

        let status = payload
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let expires_at = payload
            .get("expires_at")
            .and_then(|v| v.as_u64())
            .map(|unix| {
                unix.saturating_sub(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                )
            });

        self.log_unlock_event(
            actor,
            node,
            "get_unlock_status",
            format!("status={}", status),
            expires_at,
        )
        .await;

        if status != "unlocked" {
            let reason = payload
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("no reason provided");
            anyhow::bail!("Unlock status is '{}': {}", status, reason);
        }

        Ok(())
    }

    pub async fn unlock_events(&self) -> Vec<UnlockEvent> {
        self.unlock_events.lock().await.clone()
    }

    async fn log_unlock_event(
        &self,
        actor: &str,
        node: &str,
        event: &str,
        detail: impl Into<String>,
        ttl_seconds: Option<u64>,
    ) {
        let mut events = self.unlock_events.lock().await;
        events.push(UnlockEvent {
            timestamp: Utc::now(),
            actor: actor.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            detail: detail.into(),
            ttl_seconds,
        });
    }

    fn parse_payload(result: &serde_json::Value) -> Option<serde_json::Value> {
        let text = result
            .get("content")?
            .as_array()?
            .first()?
            .get("text")?
            .as_str()?;
        serde_json::from_str(text).ok()
    }

    /// Verify that a result matches expectations
    fn verify_expectations(
        &self,
        expect: &ExpectedResult,
        result: &serde_json::Value,
    ) -> Result<()> {
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
            let haystack = parse_mcp_content(result).unwrap_or_else(|| result.clone());

            for expected in contains {
                if !json_contains(Some(&haystack), expected) {
                    anyhow::bail!(
                        "Expected result to contain {:?}, got {:?}",
                        expected,
                        haystack
                    );
                }
            }
        }

        // Check specific field values
        for (field, expected_value) in &expect.fields {
            let actual_value = extract_from_mcp_response(result, field)
                .or_else(|| extract_json_path(result, field).cloned());

            if actual_value.as_ref() != Some(expected_value) {
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
            .post(format!("{}/messages", self.api_base))
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

#[derive(Debug, Clone)]
struct UnlockLeaseCache {
    #[allow(dead_code)]
    lease_id: String,
    expires_at: Instant,
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
    let parsed = parse_mcp_content(result)?;

    if path.is_empty() {
        return Some(parsed);
    }

    let mut current = &parsed;
    for part in path.split('.') {
        current = match part.parse::<usize>() {
            Ok(index) => current.get(index)?,
            Err(_) => current.get(part)?,
        };
    }

    Some(current.clone())
}

fn parse_mcp_content(result: &serde_json::Value) -> Option<serde_json::Value> {
    let text = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())?;

    serde_json::from_str(text).ok()
}

/// Check if a JSON value contains another (for array/object containment checks)
fn json_contains(haystack: Option<&serde_json::Value>, needle: &serde_json::Value) -> bool {
    let Some(haystack) = haystack else {
        return false;
    };

    match (haystack, needle) {
        (serde_json::Value::Array(arr), serde_json::Value::Array(expected)) => {
            expected.iter().all(|expected_item| {
                arr.iter()
                    .any(|item| json_contains(Some(item), expected_item))
            })
        }
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

    #[test]
    fn test_json_contains_nested_entities() {
        let haystack = serde_json::json!({
            "entities": [
                {
                    "id": "123",
                    "name": "Acme Corp",
                    "entity_type": "Organisation",
                    "description": null
                },
                {
                    "id": "456",
                    "name": "Engineering Team",
                    "entity_type": "Group",
                    "description": null
                }
            ]
        });

        let needle = serde_json::json!({
            "entities": [
                {"name": "Acme Corp"},
                {"name": "Engineering Team"}
            ]
        });

        assert!(json_contains(Some(&haystack), &needle));
    }

    #[test]
    fn test_json_contains_large_entity_list() {
        let haystack = serde_json::json!({
            "entities": [
                {"description": null, "entity_type": "Channel", "id": "1", "name": "general"},
                {"description": null, "entity_type": "Project", "id": "2", "name": "Secret Project"},
                {"description": null, "entity_type": "Organisation", "id": "3", "name": "Acme Corp"},
                {"description": null, "entity_type": "Organisation", "id": "4", "name": "Acme Corp"},
                {"description": null, "entity_type": "Group", "id": "5", "name": "Engineering Team"},
                {"description": null, "entity_type": "Project", "id": "6", "name": "Q1 Sprint"}
            ]
        });

        let contains = [
            serde_json::json!({"entities": [{"name": "Acme Corp"}]}),
            serde_json::json!({"entities": [{"name": "Engineering Team"}]}),
            serde_json::json!({"entities": [{"name": "general"}]}),
            serde_json::json!({"entities": [{"name": "Q1 Sprint"}]}),
        ];

        for expected in contains {
            assert!(
                json_contains(Some(&haystack), &expected),
                "Expected {:?} to be found",
                expected
            );
        }
    }
}
