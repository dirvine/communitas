use crate::services::{
    channel_service::{AppliedDiffResult, Channel, ChannelService, Message, Thread},
    issue_service::{Issue, IssueComment, IssuePriority, IssueService, IssueStatus, Project},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

// State container for services
pub struct OrgState {
    pub channel_service: Arc<ChannelService>,
    pub issue_service: Arc<IssueService>,
}

// === Channel Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChannelRequest {
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
}

#[tauri::command]
pub async fn create_channel(
    request: CreateChannelRequest,
    state: State<'_, OrgState>,
) -> Result<Channel, String> {
    state
        .channel_service
        .create_channel(
            &request.org_id,
            &request.name,
            request.description,
            &request.created_by,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_channel(
    channel_id: String,
    state: State<'_, OrgState>,
) -> Result<Option<Channel>, String> {
    state
        .channel_service
        .get_channel(&channel_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_channels(
    org_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<Channel>, String> {
    state
        .channel_service
        .list_channels(&org_id)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub channel_id: String,
    pub author_id: String,
    pub content: String,
    pub thread_id: Option<String>,
}

#[tauri::command]
pub async fn send_message(
    request: SendMessageRequest,
    state: State<'_, OrgState>,
) -> Result<Message, String> {
    state
        .channel_service
        .send_message(
            &request.channel_id,
            &request.author_id,
            &request.content,
            request.thread_id,
        )
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EditMessageRequest {
    pub message_id: String,
    pub new_content: String,
}

#[tauri::command]
pub async fn edit_message(
    request: EditMessageRequest,
    state: State<'_, OrgState>,
) -> Result<Message, String> {
    state
        .channel_service
        .edit_message(&request.message_id, &request.new_content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_message(message_id: String, state: State<'_, OrgState>) -> Result<(), String> {
    state
        .channel_service
        .delete_message(&message_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_messages(
    channel_id: String,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, OrgState>,
) -> Result<Vec<Message>, String> {
    state
        .channel_service
        .get_messages(&channel_id, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_thread(
    parent_message_id: String,
    state: State<'_, OrgState>,
) -> Result<Thread, String> {
    state
        .channel_service
        .create_thread(&parent_message_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_thread_replies(
    thread_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<Message>, String> {
    state
        .channel_service
        .get_thread_replies(&thread_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_channel_member(
    channel_id: String,
    user_id: String,
    role: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .channel_service
        .add_member(&channel_id, &user_id, &role)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_channel_member(
    channel_id: String,
    user_id: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .channel_service
        .remove_member(&channel_id, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_channel_members(
    channel_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<(String, String)>, String> {
    state
        .channel_service
        .get_members(&channel_id)
        .await
        .map_err(|e| e.to_string())
}

// === Project Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub org_id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_by: String,
}

#[tauri::command]
pub async fn create_project(
    request: CreateProjectRequest,
    state: State<'_, OrgState>,
) -> Result<Project, String> {
    state
        .issue_service
        .create_project(
            &request.org_id,
            &request.name,
            request.description,
            request.icon,
            request.color,
            &request.created_by,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project(
    project_id: String,
    state: State<'_, OrgState>,
) -> Result<Option<Project>, String> {
    state
        .issue_service
        .get_project(&project_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_projects(
    org_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<Project>, String> {
    state
        .issue_service
        .list_projects(&org_id)
        .await
        .map_err(|e| e.to_string())
}

// === Issue Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String, // "urgent", "high", "medium", "low"
    pub reporter_id: String,
}

#[tauri::command]
pub async fn create_issue(
    request: CreateIssueRequest,
    state: State<'_, OrgState>,
) -> Result<Issue, String> {
    let priority = IssuePriority::from_str(&request.priority).map_err(|e| e.to_string())?;

    state
        .issue_service
        .create_issue(
            &request.project_id,
            &request.title,
            request.description,
            priority,
            &request.reporter_id,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_issue(
    issue_id: String,
    state: State<'_, OrgState>,
) -> Result<Option<Issue>, String> {
    state
        .issue_service
        .get_issue(&issue_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_issues(
    project_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<Issue>, String> {
    state
        .issue_service
        .list_issues(&project_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_issues_by_status(
    project_id: String,
    status: String, // "backlog", "todo", "in-progress", "done", "canceled"
    state: State<'_, OrgState>,
) -> Result<Vec<Issue>, String> {
    let issue_status = IssueStatus::from_str(&status).map_err(|e| e.to_string())?;

    state
        .issue_service
        .list_issues_by_status(&project_id, issue_status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_issue_status(
    issue_id: String,
    new_status: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    let status = IssueStatus::from_str(&new_status).map_err(|e| e.to_string())?;

    state
        .issue_service
        .update_status(&issue_id, status)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn assign_issue(
    issue_id: String,
    assignee_id: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .issue_service
        .assign_issue(&issue_id, &assignee_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_issue_priority(
    issue_id: String,
    priority: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    let issue_priority = IssuePriority::from_str(&priority).map_err(|e| e.to_string())?;

    state
        .issue_service
        .update_priority(&issue_id, issue_priority)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_issue_comment(
    issue_id: String,
    author_id: String,
    content: String,
    state: State<'_, OrgState>,
) -> Result<IssueComment, String> {
    state
        .issue_service
        .add_comment(&issue_id, &author_id, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_issue_comments(
    issue_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<IssueComment>, String> {
    state
        .issue_service
        .get_comments(&issue_id)
        .await
        .map_err(|e| e.to_string())
}

// === Sync Commands ===

#[tauri::command]
pub async fn get_channel_sync_update(
    channel_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .channel_service
        .get_channel_update(&channel_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_channel_sync_update(
    channel_id: String,
    update: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .channel_service
        .apply_channel_update(&channel_id, &update)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_issue_sync_update(
    issue_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .issue_service
        .get_issue_update(&issue_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_issue_sync_update(
    issue_id: String,
    update: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .issue_service
        .apply_issue_update(&issue_id, &update)
        .await
        .map_err(|e| e.to_string())
}

// === Phase 3: Efficient Channel Sync Commands (State Vector Protocol) ===

#[tauri::command]
pub async fn get_channel_state_vector(
    channel_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .channel_service
        .get_channel_state_vector(&channel_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_channel_diff(
    channel_id: String,
    remote_state_vector: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .channel_service
        .get_channel_diff(&channel_id, &remote_state_vector)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_channel_diff(
    channel_id: String,
    diff: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<AppliedDiffResult, String> {
    state
        .channel_service
        .apply_channel_diff(&channel_id, &diff)
        .await
        .map_err(|e| e.to_string())
}
