use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Location of persistent UI data on disk.
#[derive(Debug, Clone)]
pub struct UiStorage {
    root: PathBuf,
}

impl UiStorage {
    /// Locate the Communitas application data directory (prefers OS-specific data dir, falls back to home).
    pub fn discover() -> Result<Self, StorageError> {
        let base = dirs::data_dir()
            .or_else(dirs::home_dir)
            .ok_or(StorageError::MissingBaseDir)?;
        let root = base.join("Communitas");
        Self::from_path(root)
    }

    /// Create storage using an explicit path.
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, StorageError> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        Ok(Self { root: path })
    }

    /// Root directory path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Root as owned [`PathBuf`].
    pub fn root_path(&self) -> PathBuf {
        self.root.clone()
    }

    /// Root directory as UTF-8 string (required by Communitas core APIs).
    pub fn root_string(&self) -> Result<String, StorageError> {
        self.root
            .to_str()
            .map(|s| s.to_string())
            .ok_or(StorageError::InvalidUtf8Path)
    }

    /// Disk location for navigation state persistence.
    pub fn navigation_state_file(&self) -> PathBuf {
        self.root.join("ui_navigation_state.json")
    }

    /// Disk location for unread message counts persistence.
    pub fn unread_counts_file(&self) -> PathBuf {
        self.root.join("unread_counts.json")
    }
}

/// Shared storage error type.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("unable to determine base data directory")]
    MissingBaseDir,
    #[error("storage path is not valid UTF-8")]
    InvalidUtf8Path,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Utility for persisting small JSON blobs (e.g., navigation state).
#[derive(Default, Serialize, Deserialize)]
pub struct JsonFile<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> JsonFile<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub fn load(path: &Path) -> Result<Option<T>, StorageError> {
        match fs::read_to_string(path) {
            Ok(contents) => {
                let value = serde_json::from_str(&contents)?;
                Ok(Some(value))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(StorageError::Io(err)),
        }
    }

    pub fn save(path: &Path, value: &T) -> Result<(), StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(value)?;
        fs::write(path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn storage_from_path_creates_directory() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("test_storage");
        assert!(!storage_path.exists());

        let storage = UiStorage::from_path(&storage_path).unwrap();
        assert!(storage_path.exists());
        assert_eq!(storage.root(), storage_path);
    }

    #[test]
    fn storage_root_string_returns_utf8() {
        let temp = TempDir::new().unwrap();
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let root_str = storage.root_string().unwrap();
        assert!(!root_str.is_empty());
        assert!(root_str.contains(temp.path().file_name().unwrap().to_str().unwrap()));
    }

    #[test]
    fn storage_navigation_state_file_returns_correct_path() {
        let temp = TempDir::new().unwrap();
        let storage = UiStorage::from_path(temp.path()).unwrap();
        let nav_file = storage.navigation_state_file();
        assert!(nav_file.ends_with("ui_navigation_state.json"));
        assert_eq!(nav_file.parent().unwrap(), temp.path());
    }

    #[test]
    fn json_file_save_load_roundtrip() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.json");

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestData {
            name: String,
            count: u32,
        }

        let original = TestData {
            name: "test".to_string(),
            count: 42,
        };

        JsonFile::save(&file_path, &original).unwrap();
        let loaded: Option<TestData> = JsonFile::load(&file_path).unwrap();

        assert_eq!(loaded, Some(original));
    }

    #[test]
    fn json_file_load_nonexistent_returns_none() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("does_not_exist.json");

        let loaded: Option<String> = JsonFile::load(&file_path).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn json_file_save_creates_parent_dirs() {
        let temp = TempDir::new().unwrap();
        let nested_path = temp.path().join("a").join("b").join("c").join("test.json");

        JsonFile::save(&nested_path, &"test data".to_string()).unwrap();
        assert!(nested_path.exists());
    }
}
