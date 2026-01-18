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
