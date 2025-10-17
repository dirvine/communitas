use crate::services::{
    channel_service::{AppliedDiffResult, Channel, ChannelService, Message, Thread},
    group_service::{Group, GroupService},
    issue_service::{Issue, IssueComment, IssuePriority, IssueService, IssueStatus, Project},
    member_service::MemberService,
    organization_service::OrganizationService,
    virtual_disk_service::VirtualDiskService,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

// State container for services
pub struct OrgState {
    pub channel_service: Arc<ChannelService>,
    pub issue_service: Arc<IssueService>,
    pub group_service: Arc<GroupService>,
    pub organization_service: Arc<OrganizationService>,
    pub member_service: Arc<MemberService>,
    pub virtual_disk_service: Arc<VirtualDiskService>,
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

// === Group Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub org_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
    pub group_type: String, // "organization" or "personal"
}

#[tauri::command]
pub async fn create_group(
    request: CreateGroupRequest,
    state: State<'_, OrgState>,
) -> Result<Group, String> {
    state
        .group_service
        .create_group(
            request.org_id.as_deref(),
            &request.name,
            request.description,
            &request.created_by,
            &request.group_type,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_group(
    group_id: String,
    state: State<'_, OrgState>,
) -> Result<Option<Group>, String> {
    state
        .group_service
        .get_group(&group_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_org_groups(
    org_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<Group>, String> {
    state
        .group_service
        .list_org_groups(&org_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_personal_groups(
    user_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<Group>, String> {
    state
        .group_service
        .list_personal_groups(&user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_group_member(
    group_id: String,
    user_id: String,
    role: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .group_service
        .add_member(&group_id, &user_id, &role)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_group_member(
    group_id: String,
    user_id: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .group_service
        .remove_member(&group_id, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_group_members(
    group_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<(String, String)>, String> {
    state
        .group_service
        .get_members(&group_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_group(
    group_id: String,
    name: Option<String>,
    description: Option<String>,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .group_service
        .update_group(&group_id, name.as_deref(), description)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_group(group_id: String, state: State<'_, OrgState>) -> Result<(), String> {
    state
        .group_service
        .delete_group(&group_id)
        .await
        .map_err(|e| e.to_string())
}

// === Group Sync Commands ===

#[tauri::command]
pub async fn get_group_sync_update(
    group_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .group_service
        .get_group_update(&group_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_group_sync_update(
    group_id: String,
    update: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .group_service
        .apply_group_update(&group_id, &update)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_group_state_vector(
    group_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .group_service
        .get_group_state_vector(&group_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_group_diff(
    group_id: String,
    remote_state_vector: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    state
        .group_service
        .get_group_diff(&group_id, &remote_state_vector)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn apply_group_diff(
    group_id: String,
    diff: Vec<u8>,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .group_service
        .apply_group_diff(&group_id, &diff)
        .await
        .map_err(|e| e.to_string())
}
// === Organization Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
    pub created_by: String,
}

#[tauri::command]
pub async fn create_organization(
    request: CreateOrganizationRequest,
    state: State<'_, OrgState>,
) -> Result<serde_json::Value, String> {
    let org = state
        .organization_service
        .create_organization(&request.name, request.description, &request.created_by)
        .await
        .map_err(|e| e.to_string())?;
    
    serde_json::to_value(org).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_organization(
    org_id: String,
    state: State<'_, OrgState>,
) -> Result<Option<serde_json::Value>, String> {
    let org = state
        .organization_service
        .get_organization(&org_id)
        .await
        .map_err(|e| e.to_string())?;
    
    match org {
        Some(o) => Ok(Some(serde_json::to_value(o).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_organization_by_four_words(
    four_words: String,
    state: State<'_, OrgState>,
) -> Result<Option<serde_json::Value>, String> {
    let org = state
        .organization_service
        .get_organization_by_four_words(&four_words)
        .await
        .map_err(|e| e.to_string())?;
    
    match org {
        Some(o) => Ok(Some(serde_json::to_value(o).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganizationRequest {
    pub org_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn update_organization(
    request: UpdateOrganizationRequest,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .organization_service
        .update_organization(&request.org_id, request.name.as_deref(), request.description)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_organization(
    org_id: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .organization_service
        .delete_organization(&org_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_organization_website_root(
    org_id: String,
    website_root: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .organization_service
        .set_website_root(&org_id, &website_root)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_organization_member(
    org_id: String,
    user_id: String,
    role: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .organization_service
        .add_member(&org_id, &user_id, &role)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_organization_member(
    org_id: String,
    user_id: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .organization_service
        .remove_member(&org_id, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_organization_member_role(
    org_id: String,
    user_id: String,
    role: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .organization_service
        .update_member_role(&org_id, &user_id, &role)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_organization_members(
    org_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<serde_json::Value>, String> {
    let members = state
        .organization_service
        .get_members(&org_id)
        .await
        .map_err(|e| e.to_string())?;
    
    members
        .into_iter()
        .map(|m| serde_json::to_value(m).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub async fn is_organization_member(
    org_id: String,
    user_id: String,
    state: State<'_, OrgState>,
) -> Result<bool, String> {
    state
        .organization_service
        .is_member(&org_id, &user_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_user_organizations(
    user_id: String,
    state: State<'_, OrgState>,
) -> Result<Vec<serde_json::Value>, String> {
    let orgs = state
        .organization_service
        .list_user_organizations(&user_id)
        .await
        .map_err(|e| e.to_string())?;
    
    orgs
        .into_iter()
        .map(|o| serde_json::to_value(o).map_err(|e| e.to_string()))
        .collect()
}

// === Member Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMemberRequest {
    pub display_name: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[tauri::command]
pub async fn create_member(
    request: CreateMemberRequest,
    state: State<'_, OrgState>,
) -> Result<serde_json::Value, String> {
    let member = state
        .member_service
        .create_member(
            &request.display_name,
            request.email,
        )
        .await
        .map_err(|e| e.to_string())?;
    
    serde_json::to_value(member).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_member(
    member_id: String,
    state: State<'_, OrgState>,
) -> Result<Option<serde_json::Value>, String> {
    let member = state
        .member_service
        .get_member(&member_id)
        .await
        .map_err(|e| e.to_string())?;
    
    match member {
        Some(m) => Ok(Some(serde_json::to_value(m).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_member_by_four_words(
    four_words: String,
    state: State<'_, OrgState>,
) -> Result<Option<serde_json::Value>, String> {
    let member = state
        .member_service
        .get_member_by_four_words(&four_words)
        .await
        .map_err(|e| e.to_string())?;
    
    match member {
        Some(m) => Ok(Some(serde_json::to_value(m).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateMemberRequest {
    pub member_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
}

#[tauri::command]
pub async fn update_member(
    request: UpdateMemberRequest,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .member_service
        .update_member(
            &request.member_id,
            request.display_name.as_deref(),
            request.email,
            request.avatar_url,
            request.bio,
        )
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_member(
    member_id: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .member_service
        .delete_member(&member_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_all_members(
    state: State<'_, OrgState>,
) -> Result<Vec<serde_json::Value>, String> {
    let members = state
        .member_service
        .list_all_members()
        .await
        .map_err(|e| e.to_string())?;
    
    members
        .into_iter()
        .map(|m| serde_json::to_value(m).map_err(|e| e.to_string()))
        .collect()
}

#[tauri::command]
pub async fn set_member_website_root(
    member_id: String,
    website_root: String,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    state
        .member_service
        .set_website_root(&member_id, &website_root)
        .await
        .map_err(|e| e.to_string())
}

// === Virtual Disk Commands ===

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteFileRequest {
    pub entity_id: String,
    pub disk_type: String, // "PrivateShared" or "PublicWeb"
    pub path: String,
    pub content: Vec<u8>,
}

#[tauri::command]
pub async fn write_virtual_disk_file(
    request: WriteFileRequest,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    // Determine content type from file extension
    let content_type = mime_guess::from_path(&request.path)
        .first_or_octet_stream()
        .to_string();

    // Enable CRDT for text files
    let enable_crdt = content_type.starts_with("text/");

    state
        .virtual_disk_service
        .write_file(&request.entity_id, &request.path, &request.content, &content_type, enable_crdt)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadFileRequest {
    pub entity_id: String,
    pub disk_type: String,
    pub path: String,
}

#[tauri::command]
pub async fn read_virtual_disk_file(
    request: ReadFileRequest,
    state: State<'_, OrgState>,
) -> Result<Vec<u8>, String> {
    
    
    state
        .virtual_disk_service
        .read_file(&request.entity_id, &request.path)
        .await
        .map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteFileRequest {
    pub entity_id: String,
    pub disk_type: String,
    pub path: String,
}

#[tauri::command]
pub async fn delete_virtual_disk_file(
    request: DeleteFileRequest,
    state: State<'_, OrgState>,
) -> Result<(), String> {
    
    
    state
        .virtual_disk_service
        .delete_file(&request.entity_id, &request.path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn file_exists_virtual_disk(
    entity_id: String,
    path: String,
    state: State<'_, OrgState>,
) -> Result<bool, String> {
    state
        .virtual_disk_service
        .file_exists(&entity_id, &path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_virtual_disk_directory(
    entity_id: String,
    path: String,
    state: State<'_, OrgState>,
) -> Result<Vec<serde_json::Value>, String> {
    

    let entries = state
        .virtual_disk_service
        .list_directory(&entity_id, &path)
        .await
        .map_err(|e| e.to_string())?;

    // Convert DiskEntry to JSON for frontend
    let json_entries: Vec<serde_json::Value> = entries
        .into_iter()
        .map(|entry| serde_json::json!({
            "name": entry.name,
            "path": entry.path,
            "isDirectory": entry.is_directory,
            "size": entry.size,
            "updatedAt": entry.updated_at,
        }))
        .collect();

    Ok(json_entries)
}
