use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;
use tauri::command;
use tracing::warn;

const STORAGE_DIR: &str = "storage";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified_at: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StorageStats {
    pub total_size: u64,
    pub used_size: u64,
    pub file_count: u64,
    pub folder_count: u64,
    pub encrypted_count: u64,
    pub shared_count: u64,
}

fn data_root() -> PathBuf {
    if let Ok(p) = std::env::var("COMMUNITAS_DATA_DIR") {
        PathBuf::from(p)
    } else {
        PathBuf::from("src-tauri/.communitas-data")
    }
}

fn storage_root(entity_id: &str) -> Result<PathBuf, String> {
    validate_entity_id(entity_id)?;
    Ok(data_root().join(STORAGE_DIR).join(entity_id))
}

fn validate_entity_id(entity_id: &str) -> Result<(), String> {
    if entity_id.is_empty() {
        return Err("entityId cannot be empty".to_string());
    }
    if !entity_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return Err("entityId contains invalid characters".to_string());
    }
    Ok(())
}

fn normalize_relative_path(path: &str) -> Result<PathBuf, String> {
    if path.trim().is_empty() || path == "/" {
        return Ok(PathBuf::new());
    }

    let trimmed = path.trim_start_matches('/').trim_end_matches('/');

    let mut normalized = PathBuf::new();
    for component in Path::new(trimmed).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            _ => {
                return Err("path contains invalid segments".to_string());
            }
        }
    }

    Ok(normalized)
}

fn ensure_directory(path: &Path) -> Result<(), String> {
    if let Err(err) = fs::create_dir_all(path) {
        return Err(format!("failed to create directory {:?}: {}", path, err));
    }
    Ok(())
}

fn to_storage_entry(root: &Path, full_path: PathBuf) -> Result<StorageEntry, String> {
    let metadata = fs::metadata(&full_path)
        .map_err(|err| format!("metadata failed for {:?}: {}", full_path, err))?;

    let name = full_path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| "invalid UTF-8 in file name".to_string())?
        .to_string();

    let relative = full_path
        .strip_prefix(root)
        .map_err(|err| format!("failed to build relative path: {}", err))?
        .to_string_lossy()
        .replace('\\', "/");

    let modified_at = metadata.modified().ok().and_then(|time| {
        let datetime: Option<chrono::DateTime<chrono::Utc>> = system_time_to_utc(time);
        datetime.map(|dt| dt.to_rfc3339())
    });

    let size = if metadata.is_file() {
        Some(metadata.len())
    } else {
        None
    };

    Ok(StorageEntry {
        name,
        path: format!("/{}", relative.trim_start_matches('/')),
        is_directory: metadata.is_dir(),
        size,
        modified_at,
        content_type: if metadata.is_file() {
            mime_guess::from_path(&full_path)
                .first_raw()
                .map(|m| m.to_string())
        } else {
            None
        },
    })
}

fn system_time_to_utc(time: SystemTime) -> Option<chrono::DateTime<chrono::Utc>> {
    let datetime: chrono::DateTime<chrono::Utc> = time.into();
    Some(datetime)
}

fn read_directory(entity_root: &Path, dir_path: &Path) -> Result<Vec<StorageEntry>, String> {
    let mut entries = Vec::new();
    let read_dir = fs::read_dir(dir_path)
        .map_err(|err| format!("failed to read directory {:?}: {}", dir_path, err))?;

    for entry in read_dir {
        match entry {
            Ok(dir_entry) => {
                let path = dir_entry.path();
                match to_storage_entry(entity_root, path) {
                    Ok(item) => entries.push(item),
                    Err(err) => warn!("storage list entry error: {}", err),
                }
            }
            Err(err) => warn!("storage list failed to read entry: {}", err),
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

fn compute_stats(dir_path: &Path) -> Result<StorageStats, String> {
    fn visit(path: &Path, totals: &mut (u64, u64, u64)) -> Result<(), String> {
        let metadata =
            fs::metadata(path).map_err(|err| format!("metadata failed for {:?}: {}", path, err))?;
        if metadata.is_dir() {
            totals.2 += 1; // folder count
            for entry in fs::read_dir(path)
                .map_err(|err| format!("read_dir failed for {:?}: {}", path, err))?
            {
                let entry = entry.map_err(|err| format!("dir entry error: {}", err))?;
                visit(&entry.path(), totals)?;
            }
        } else {
            totals.0 += metadata.len();
            totals.1 += 1;
        }
        Ok(())
    }

    let mut totals = (0u64, 0u64, 0u64);
    visit(dir_path, &mut totals)?;
    let (total_size, file_count, folder_count) = totals;

    Ok(StorageStats {
        total_size,
        used_size: total_size,
        file_count,
        folder_count,
        encrypted_count: 0,
        shared_count: 0,
    })
}

fn write_file(entity_root: &Path, relative_path: PathBuf, data: Vec<u8>) -> Result<(), String> {
    let target = entity_root.join(&relative_path);
    if let Some(parent) = target.parent() {
        ensure_directory(parent)?;
    }
    fs::write(&target, data).map_err(|err| format!("failed to write file {:?}: {}", target, err))
}

fn remove_path(entity_root: &Path, relative_path: PathBuf) -> Result<(), String> {
    let target = entity_root.join(&relative_path);
    if !target.exists() {
        return Ok(());
    }
    let metadata = fs::metadata(&target)
        .map_err(|err| format!("metadata failed for {:?}: {}", target, err))?;
    if metadata.is_dir() {
        fs::remove_dir_all(&target)
            .map_err(|err| format!("failed to remove directory {:?}: {}", target, err))
    } else {
        fs::remove_file(&target)
            .map_err(|err| format!("failed to remove file {:?}: {}", target, err))
    }
}

fn rename_path(entity_root: &Path, old_path: PathBuf, new_path: PathBuf) -> Result<(), String> {
    let from = entity_root.join(&old_path);
    let to = entity_root.join(&new_path);
    if let Some(parent) = to.parent() {
        ensure_directory(parent)?;
    }
    fs::rename(&from, &to)
        .map_err(|err| format!("failed to rename {:?} to {:?}: {}", from, to, err))
}

fn read_file(entity_root: &Path, relative_path: PathBuf) -> Result<Vec<u8>, String> {
    let target = entity_root.join(&relative_path);
    let mut file = fs::File::open(&target)
        .map_err(|err| format!("failed to open file {:?}: {}", target, err))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| format!("failed to read file {:?}: {}", target, err))?;
    Ok(buffer)
}

#[command]
pub async fn core_storage_list(
    entity_id: String,
    path: String,
) -> Result<Vec<StorageEntry>, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    let relative = normalize_relative_path(&path)?;
    let dir_path = entity_root.join(&relative);
    ensure_directory(&dir_path)?;
    read_directory(&entity_root, &dir_path)
}

#[command]
pub async fn core_storage_read(entity_id: String, path: String) -> Result<Vec<u8>, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    let relative = normalize_relative_path(&path)?;
    read_file(&entity_root, relative)
}

#[command]
pub async fn core_storage_write(
    entity_id: String,
    path: String,
    content: serde_json::Value,
) -> Result<bool, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    let relative = normalize_relative_path(&path)?;
    let bytes = value_to_bytes(content)?;
    write_file(&entity_root, relative, bytes)?;
    Ok(true)
}

#[command]
pub async fn core_storage_mkdir(entity_id: String, path: String) -> Result<bool, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    let relative = normalize_relative_path(&path)?;
    let dir = entity_root.join(&relative);
    ensure_directory(&dir)?;
    Ok(true)
}

#[command]
pub async fn core_storage_delete(entity_id: String, path: String) -> Result<bool, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    let relative = normalize_relative_path(&path)?;
    remove_path(&entity_root, relative)?;
    Ok(true)
}

#[command]
pub async fn core_storage_rename(
    entity_id: String,
    old_path: String,
    new_path: String,
) -> Result<bool, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    let old_relative = normalize_relative_path(&old_path)?;
    let new_relative = normalize_relative_path(&new_path)?;
    rename_path(&entity_root, old_relative, new_relative)?;
    Ok(true)
}

#[command]
pub async fn core_storage_stats(entity_id: String) -> Result<StorageStats, String> {
    let entity_root = storage_root(&entity_id)?;
    ensure_directory(&entity_root)?;
    compute_stats(&entity_root)
}

fn value_to_bytes(value: serde_json::Value) -> Result<Vec<u8>, String> {
    match value {
        serde_json::Value::String(s) => Ok(s.into_bytes()),
        serde_json::Value::Array(items) => {
            let mut bytes = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    serde_json::Value::Number(num) => {
                        if let Some(b) = num.as_u64() {
                            if b > u8::MAX as u64 {
                                return Err("byte value out of range".to_string());
                            }
                            bytes.push(b as u8);
                        } else {
                            return Err("array element is not an unsigned integer".to_string());
                        }
                    }
                    _ => return Err("array must contain only numbers".to_string()),
                }
            }
            Ok(bytes)
        }
        serde_json::Value::Object(mut map) => {
            if let Some(serde_json::Value::Array(data)) = map.remove("data") {
                value_to_bytes(serde_json::Value::Array(data))
            } else {
                Err("unsupported content object".to_string())
            }
        }
        serde_json::Value::Null => Ok(Vec::new()),
        _ => Err("unsupported content type".to_string()),
    }
}
