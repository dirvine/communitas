// Copyright (c) 2025 Saorsa Labs Limited

// This file is part of the Saorsa P2P network.

// Licensed under the AGPL-3.0 license:
// <https://www.gnu.org/licenses/agpl-3.0.html>

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.


use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

const DATA_DIR: &str = ".communitas-data";
const METADATA_FILE: &str = "metadata.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub contacts: Vec<ContactEntry>,
    pub organizations: Vec<OrganizationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactEntry {
    pub id: String,
    pub display_name: String,
    pub four_word_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationEntry {
    pub id: String,
    pub name: String,
    pub groups: Vec<NamedEntry>,
    pub projects: Vec<NamedEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedEntry {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopePath {
    /// One of: contact/{contact_id}, org/{org_id}, org/{org_id}/group/{group_id}, org/{org_id}/project/{project_id}
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownFileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
}

fn app_data_dir() -> PathBuf {
    PathBuf::from(DATA_DIR)
}

fn metadata_path() -> PathBuf {
    app_data_dir().join(METADATA_FILE)
}

fn scope_dir(scope: &str) -> Result<PathBuf> {
    // Map scope to directory under DATA_DIR
    let dir = match scope.split('/').collect::<Vec<_>>().as_slice() {
        ["contact", contact_id] => app_data_dir().join("contacts").join(contact_id).join("markdown"),
        ["org", org_id] => app_data_dir().join("orgs").join(org_id).join("markdown"),
        ["org", org_id, "group", group_id] => app_data_dir().join("orgs").join(org_id).join("groups").join(group_id).join("markdown"),
        ["org", org_id, "project", project_id] => app_data_dir().join("orgs").join(org_id).join("projects").join(project_id).join("markdown"),
        _ => anyhow::bail!("Invalid scope: {}", scope),
    };
    Ok(dir)
}

async fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).await.context("Failed to create directory")
}

async fn read_metadata() -> Result<Metadata> {
    let path = metadata_path();
    if !path.exists() {
        return Ok(Metadata::default());
    }
    let bytes = fs::read(&path).await.context("Failed to read metadata.json")?;
    let meta: Metadata = serde_json::from_slice(&bytes).context("Failed to parse metadata.json")?;
    Ok(meta)
}

async fn write_metadata(meta: &Metadata) -> Result<()> {
    let path = metadata_path();
    let parent = path.parent().unwrap();
    fs::create_dir_all(parent).await.ok();
    let data = serde_json::to_vec_pretty(meta).context("Failed to serialize metadata")?;
    fs::write(&path, data).await.context("Failed to write metadata.json")
}

#[tauri::command]
pub async fn init_local_stores() -> Result<String, String> {
    // Ensure base directories
    let base = app_data_dir();
    ensure_dir(&base).await.map_err(|e| e.to_string())?;
    ensure_dir(&base.join("contacts")).await.map_err(|e| e.to_string())?;
    ensure_dir(&base.join("orgs")).await.map_err(|e| e.to_string())?;
    // Ensure metadata exists
    if !metadata_path().exists() {
        write_metadata(&Metadata::default()).await.map_err(|e| e.to_string())?;
    }
    Ok(base.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_metadata() -> Result<Metadata, String> {
    read_metadata().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_organization(name: String) -> Result<OrganizationEntry, String> {
    let mut meta = read_metadata().await.map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let org = OrganizationEntry { id: id.clone(), name, groups: vec![], projects: vec![] };
    // Ensure org directories
    let base = app_data_dir().join("orgs").join(&id);
    for sub in ["markdown", "groups", "projects"] {
        ensure_dir(&base.join(sub)).await.map_err(|e| e.to_string())?;
    }
    meta.organizations.push(org.clone());
    write_metadata(&meta).await.map_err(|e| e.to_string())?;
    Ok(org)
}

#[tauri::command]
pub async fn create_group_local(org_id: String, name: String) -> Result<NamedEntry, String> {
    let mut meta = read_metadata().await.map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let entry = NamedEntry { id: id.clone(), name };
    // Ensure directories
    let dir = app_data_dir().join("orgs").join(&org_id).join("groups").join(&id);
    ensure_dir(&dir.join("markdown")).await.map_err(|e| e.to_string())?;
    if let Some(org) = meta.organizations.iter_mut().find(|o| o.id == org_id) {
        org.groups.push(entry.clone());
        write_metadata(&meta).await.map_err(|e| e.to_string())?;
        Ok(entry)
    } else {
        Err("Organization not found".into())
    }
}

#[tauri::command]
pub async fn create_project(org_id: String, name: String) -> Result<NamedEntry, String> {
    let mut meta = read_metadata().await.map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let entry = NamedEntry { id: id.clone(), name };
    let dir = app_data_dir().join("orgs").join(&org_id).join("projects").join(&id);
    ensure_dir(&dir.join("markdown")).await.map_err(|e| e.to_string())?;
    if let Some(org) = meta.organizations.iter_mut().find(|o| o.id == org_id) {
        org.projects.push(entry.clone());
        write_metadata(&meta).await.map_err(|e| e.to_string())?;
        Ok(entry)
    } else {
        Err("Organization not found".into())
    }
}

#[tauri::command]
pub async fn add_contact_local(display_name: String, four_word_address: String) -> Result<ContactEntry, String> {
    let mut meta = read_metadata().await.map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let contact = ContactEntry { id: id.clone(), display_name, four_word_address };
    let dir = app_data_dir().join("contacts").join(&id);
    ensure_dir(&dir.join("markdown")).await.map_err(|e| e.to_string())?;
    meta.contacts.push(contact.clone());
    write_metadata(&meta).await.map_err(|e| e.to_string())?;
    Ok(contact)
}

#[tauri::command]
pub async fn list_markdown(scope: ScopePath) -> Result<Vec<MarkdownFileInfo>, String> {
    let dir = scope_dir(&scope.scope).map_err(|e| e.to_string())?;
    ensure_dir(&dir).await.map_err(|e| e.to_string())?;
    let mut entries = Vec::new();
    let mut rd = fs::read_dir(&dir).await.map_err(|e| e.to_string())?;
    while let Ok(Some(entry)) = rd.next_entry().await {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            let meta = entry.metadata().await.map_err(|e| e.to_string())?;
            entries.push(MarkdownFileInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.to_string_lossy().to_string(),
                size: meta.len(),
            });
        }
    }
    Ok(entries)
}

#[tauri::command]
pub async fn read_markdown_file(path: String) -> Result<String, String> {
    let content = fs::read_to_string(&path).await.map_err(|e| e.to_string())?;
    Ok(content)
}

#[tauri::command]
pub async fn write_markdown_file(path: String, content: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    if let Some(parent) = p.parent() { ensure_dir(parent).await.map_err(|e| e.to_string())?; }
    fs::write(&p, content.as_bytes()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_markdown(scope: ScopePath, name: String, initial_content: Option<String>) -> Result<String, String> {
    let dir = scope_dir(&scope.scope).map_err(|e| e.to_string())?;
    ensure_dir(&dir).await.map_err(|e| e.to_string())?;
    let safe_name = if name.ends_with(".md") { name } else { format!("{}.md", name) };
    let path = dir.join(safe_name);
    if path.exists() { return Err("File already exists".into()); }
    fs::write(&path, initial_content.unwrap_or_default()).await.map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
