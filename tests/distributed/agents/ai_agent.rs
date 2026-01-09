use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct AiAgent {
    name: String,
    role: AgentRole,
    client: Client,
    api_key: String,
    model: String,
    system_prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentRole {
    Alice,
    Bob,
    Charlie,
    Judge,
}

impl AgentRole {
    pub fn system_prompt(&self) -> &'static str {
        match self {
            AgentRole::Alice => r#"You are Alice, a project manager testing a collaboration platform.
Your personality: organized, detail-oriented, proactive about creating structure.
Your job: Create organizations, projects, and channels. Invite team members. 
Plan work using Kanban boards. Send messages and share documents.
Always respond with specific MCP tool calls in JSON format.
Track your work and report any issues you find."#,

            AgentRole::Bob => r#"You are Bob, a developer testing a collaboration platform.
Your personality: pragmatic, focused on getting work done, good at following up.
Your job: Accept invites, respond to messages, work on assigned tasks.
Update Kanban cards, add comments, complete checklists.
Collaborate on shared documents. Report progress.
Always respond with specific MCP tool calls in JSON format."#,

            AgentRole::Charlie => r#"You are Charlie, a team member testing a collaboration platform.
Your personality: curious, asks questions, sometimes works offline.
Your job: Join conversations, add reactions, create threads.
Work on tasks, share files, and test sync after going offline.
Always respond with specific MCP tool calls in JSON format."#,

            AgentRole::Judge => r#"You are the Judge, validating that the collaboration platform works correctly.
Your job: Verify all operations completed successfully.
Check that messages, files, and entities are properly synced across nodes.
Validate data consistency and report any discrepancies.
Provide a detailed assessment of what passed and what failed.
Be thorough and precise in your validation."#,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AgentRole::Alice => "alice",
            AgentRole::Bob => "bob",
            AgentRole::Charlie => "charlie",
            AgentRole::Judge => "judge",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentAction {
    pub tool: String,
    pub arguments: Value,
    pub reasoning: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentResponse {
    pub actions: Vec<AgentAction>,
    pub thoughts: String,
    pub complete: bool,
}

impl AiAgent {
    pub fn new(role: AgentRole, api_key: &str) -> Self {
        Self {
            name: role.name().to_string(),
            role,
            client: Client::new(),
            api_key: api_key.to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            system_prompt: role.system_prompt().to_string(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> AgentRole {
        self.role
    }

    pub async fn decide_actions(
        &self,
        context: &str,
        available_tools: &[String],
        history: &[AgentAction],
    ) -> Result<AgentResponse> {
        let tools_list = available_tools.join(", ");
        let history_json = serde_json::to_string_pretty(history).unwrap_or_default();

        let user_message = format!(
            r#"## Current Context
{}

## Available MCP Tools
{}

## Your Previous Actions
{}

## Your Task
Based on the context, decide what actions to take next.
Respond with a JSON object containing:
- "actions": array of {{"tool": "tool_name", "arguments": {{}}, "reasoning": "why"}}
- "thoughts": your reasoning about the current situation
- "complete": true if you've finished your current objective, false otherwise

Important:
- Use only the available tools listed above
- Each action should have proper arguments for the tool
- Be specific with entity IDs, message content, etc.
- If waiting for others, set complete=true"#,
            context, tools_list, history_json
        );

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": self.model,
                "max_tokens": 4096,
                "system": self.system_prompt,
                "messages": [
                    {"role": "user", "content": user_message}
                ]
            }))
            .send()
            .await
            .context("Failed to call Anthropic API")?;

        let status = response.status();
        let body: Value = response.json().await.context("Failed to parse API response")?;

        if !status.is_success() {
            anyhow::bail!("API error {}: {:?}", status, body);
        }

        let content = body
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("{}");

        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_str = &content[json_start..json_end];

        let agent_response: AgentResponse =
            serde_json::from_str(json_str).unwrap_or(AgentResponse {
                actions: vec![],
                thoughts: content.to_string(),
                complete: true,
            });

        Ok(agent_response)
    }

    pub async fn validate_results(
        &self,
        scenario: &str,
        results: &[ValidationItem],
    ) -> Result<ValidationReport> {
        if self.role != AgentRole::Judge {
            anyhow::bail!("Only Judge agent can validate results");
        }

        let results_json = serde_json::to_string_pretty(results).unwrap_or_default();

        let user_message = format!(
            r#"## Scenario
{}

## Results to Validate
{}

## Your Task
Analyze these results and provide a comprehensive validation report.
Check for:
1. All operations completed successfully
2. Data consistency across nodes
3. Proper sync behavior
4. Any error conditions or failures

Respond with JSON:
{{
    "passed": boolean,
    "score": 0-100,
    "summary": "brief summary",
    "issues": ["list of issues found"],
    "recommendations": ["suggestions for improvement"]
}}"#,
            scenario, results_json
        );

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&json!({
                "model": self.model,
                "max_tokens": 4096,
                "system": self.system_prompt,
                "messages": [
                    {"role": "user", "content": user_message}
                ]
            }))
            .send()
            .await
            .context("Failed to call Anthropic API for validation")?;

        let body: Value = response.json().await?;
        let content = body
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("{}");

        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_str = &content[json_start..json_end];

        let report: ValidationReport = serde_json::from_str(json_str).unwrap_or(ValidationReport {
            passed: false,
            score: 0,
            summary: "Failed to parse validation response".to_string(),
            issues: vec![content.to_string()],
            recommendations: vec![],
        });

        Ok(report)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationItem {
    pub operation: String,
    pub node: String,
    pub success: bool,
    pub details: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub passed: bool,
    pub score: u32,
    pub summary: String,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

impl ValidationReport {
    pub fn print(&self) {
        println!("\n=== Validation Report ===");
        println!("Passed: {}", if self.passed { "YES" } else { "NO" });
        println!("Score: {}/100", self.score);
        println!("Summary: {}", self.summary);
        if !self.issues.is_empty() {
            println!("\nIssues:");
            for issue in &self.issues {
                println!("  - {}", issue);
            }
        }
        if !self.recommendations.is_empty() {
            println!("\nRecommendations:");
            for rec in &self.recommendations {
                println!("  - {}", rec);
            }
        }
        println!("=========================\n");
    }
}
