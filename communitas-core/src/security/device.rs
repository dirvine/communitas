//! Device Fingerprinting for Security Tracking
//!
//! Provides stable device identification for:
//! - Detecting when a vault is accessed from a new device
//! - Tracking known devices per identity
//! - Emitting audit events on device changes
//!
//! The fingerprint is a Blake3 hash of machine-specific attributes that
//! remain stable across reboots but differ between machines.

use anyhow::{Context, Result};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn};

/// A device fingerprint that uniquely identifies a machine
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeviceFingerprint {
    /// The fingerprint hash (Blake3, hex-encoded)
    pub fingerprint: String,

    /// Human-readable device name
    pub device_name: String,

    /// Operating system name
    pub os_name: String,

    /// Operating system version
    pub os_version: String,

    /// When this device was first seen
    pub first_seen: DateTime<Utc>,

    /// When this device was last seen
    pub last_seen: DateTime<Utc>,
}

impl DeviceFingerprint {
    /// Generate a fingerprint for the current device
    ///
    /// The fingerprint is based on:
    /// - Machine hostname
    /// - OS name and version
    /// - CPU brand (if available)
    /// - Total memory (rounded to nearest GB for stability)
    #[instrument]
    pub fn current() -> Result<Self> {
        let mut sys = System::new_all();
        sys.refresh_all();

        // Gather system attributes
        let hostname = System::host_name().unwrap_or_else(|| "unknown".to_string());
        let os_name = System::name().unwrap_or_else(|| "unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "unknown".to_string());
        let cpu_brand = sys
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Round total memory to nearest GB for stability across minor changes
        let total_memory_gb = sys.total_memory() / (1024 * 1024 * 1024);

        // Build fingerprint input
        let fingerprint_input = format!(
            "communitas:device:v1:{}:{}:{}:{}:{}GB",
            hostname, os_name, os_version, cpu_brand, total_memory_gb
        );

        // Hash with Blake3
        let mut hasher = Hasher::new();
        hasher.update(fingerprint_input.as_bytes());
        let hash = hasher.finalize();
        let fingerprint = hex::encode(hash.as_bytes());

        let now = Utc::now();

        debug!(
            "Generated device fingerprint for {} ({})",
            hostname,
            &fingerprint[..16]
        );

        Ok(Self {
            fingerprint,
            device_name: hostname,
            os_name,
            os_version,
            first_seen: now,
            last_seen: now,
        })
    }

    /// Get a shortened fingerprint for display (first 16 chars)
    pub fn short_id(&self) -> &str {
        &self.fingerprint[..16.min(self.fingerprint.len())]
    }

    /// Update the last_seen timestamp
    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
    }
}

/// Storage for known devices associated with an identity
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnownDevices {
    /// Set of known device fingerprints
    devices: Vec<DeviceFingerprint>,

    /// Maximum number of devices to track (prevent unbounded growth)
    #[serde(default = "default_max_devices")]
    max_devices: usize,
}

fn default_max_devices() -> usize {
    10
}

impl KnownDevices {
    /// Create a new empty known devices store
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            max_devices: default_max_devices(),
        }
    }

    /// Create with custom max devices limit
    pub fn with_max(max_devices: usize) -> Self {
        Self {
            devices: Vec::new(),
            max_devices,
        }
    }

    /// Check if a device is known
    pub fn is_known(&self, fingerprint: &str) -> bool {
        self.devices.iter().any(|d| d.fingerprint == fingerprint)
    }

    /// Get a known device by fingerprint
    pub fn get(&self, fingerprint: &str) -> Option<&DeviceFingerprint> {
        self.devices.iter().find(|d| d.fingerprint == fingerprint)
    }

    /// Get a mutable reference to a known device
    pub fn get_mut(&mut self, fingerprint: &str) -> Option<&mut DeviceFingerprint> {
        self.devices
            .iter_mut()
            .find(|d| d.fingerprint == fingerprint)
    }

    /// Add a new device or update existing
    ///
    /// Returns `true` if this is a new device, `false` if it was already known
    pub fn add_or_update(&mut self, device: DeviceFingerprint) -> bool {
        if let Some(existing) = self.get_mut(&device.fingerprint) {
            existing.touch();
            false
        } else {
            // Check if we need to remove oldest device
            if self.devices.len() >= self.max_devices {
                // Remove oldest by last_seen
                if let Some(oldest_idx) = self
                    .devices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, d)| d.last_seen)
                    .map(|(i, _)| i)
                {
                    let removed = self.devices.remove(oldest_idx);
                    info!(
                        "Removed oldest device {} to make room for new device",
                        removed.short_id()
                    );
                }
            }

            self.devices.push(device);
            true
        }
    }

    /// Remove a device by fingerprint
    pub fn remove(&mut self, fingerprint: &str) -> Option<DeviceFingerprint> {
        if let Some(idx) = self
            .devices
            .iter()
            .position(|d| d.fingerprint == fingerprint)
        {
            Some(self.devices.remove(idx))
        } else {
            None
        }
    }

    /// List all known devices
    pub fn list(&self) -> &[DeviceFingerprint] {
        &self.devices
    }

    /// Count of known devices
    pub fn count(&self) -> usize {
        self.devices.len()
    }
}

/// Manager for device fingerprinting and tracking
pub struct DeviceManager {
    /// Current device fingerprint
    current_device: DeviceFingerprint,

    /// Path to store known devices (encrypted by vault)
    storage_path: Option<PathBuf>,

    /// Cached known devices
    known_devices: RwLock<KnownDevices>,
}

impl DeviceManager {
    /// Create a new device manager
    ///
    /// # Arguments
    /// * `storage_path` - Optional path for persisting known devices
    #[instrument(skip(storage_path))]
    pub async fn new(storage_path: Option<PathBuf>) -> Result<Self> {
        let current_device = DeviceFingerprint::current()?;

        // Load known devices from storage if path provided
        let known_devices = if let Some(ref path) = storage_path {
            match Self::load_known_devices(path).await {
                Ok(devices) => devices,
                Err(e) => {
                    warn!(
                        "Failed to load known devices from {:?}: {}. Starting fresh.",
                        path, e
                    );
                    KnownDevices::new()
                }
            }
        } else {
            KnownDevices::new()
        };

        info!(
            "Device manager initialized: {} ({})",
            current_device.device_name,
            current_device.short_id()
        );

        Ok(Self {
            current_device,
            storage_path,
            known_devices: RwLock::new(known_devices),
        })
    }

    /// Get the current device fingerprint
    pub fn current_fingerprint(&self) -> &DeviceFingerprint {
        &self.current_device
    }

    /// Get the current device fingerprint string (for audit logs)
    pub fn fingerprint_string(&self) -> &str {
        &self.current_device.fingerprint
    }

    /// Check if current device is new (not in known devices)
    #[instrument(skip(self))]
    pub async fn is_new_device(&self) -> bool {
        let known = self.known_devices.read().await;
        !known.is_known(&self.current_device.fingerprint)
    }

    /// Register the current device as known
    ///
    /// Returns `true` if this was a new device
    #[instrument(skip(self))]
    pub async fn register_current_device(&self) -> Result<bool> {
        let mut known = self.known_devices.write().await;
        let is_new = known.add_or_update(self.current_device.clone());

        if is_new {
            info!(
                "Registered new device: {} ({})",
                self.current_device.device_name,
                self.current_device.short_id()
            );
        }

        // Persist to storage
        if let Some(ref path) = self.storage_path {
            Self::save_known_devices(path, &known).await?;
        }

        Ok(is_new)
    }

    /// Check and register device, returning whether it's new
    ///
    /// This is the primary method for auth flows to detect new devices
    #[instrument(skip(self))]
    pub async fn check_and_register(&self) -> Result<DeviceCheckResult> {
        let is_new = self.is_new_device().await;

        if is_new {
            self.register_current_device().await?;
        } else {
            // Update last_seen for existing device
            let mut known = self.known_devices.write().await;
            if let Some(device) = known.get_mut(&self.current_device.fingerprint) {
                device.touch();
            }

            // Persist updated timestamp
            if let Some(ref path) = self.storage_path {
                Self::save_known_devices(path, &known).await?;
            }
        }

        Ok(DeviceCheckResult {
            is_new,
            fingerprint: self.current_device.fingerprint.clone(),
            device_name: self.current_device.device_name.clone(),
        })
    }

    /// Remove a device from known devices
    #[instrument(skip(self))]
    pub async fn remove_device(&self, fingerprint: &str) -> Result<Option<DeviceFingerprint>> {
        let mut known = self.known_devices.write().await;
        let removed = known.remove(fingerprint);

        if let Some(ref path) = self.storage_path {
            Self::save_known_devices(path, &known).await?;
        }

        Ok(removed)
    }

    /// List all known devices
    pub async fn list_known_devices(&self) -> Vec<DeviceFingerprint> {
        let known = self.known_devices.read().await;
        known.list().to_vec()
    }

    /// Load known devices from file
    async fn load_known_devices(path: &Path) -> Result<KnownDevices> {
        if !path.exists() {
            return Ok(KnownDevices::new());
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read known devices from {:?}", path))?;

        serde_json::from_str(&content).with_context(|| "Failed to parse known devices")
    }

    /// Save known devices to file
    async fn save_known_devices(path: &Path, devices: &KnownDevices) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("Failed to create directory {:?}", parent))?;
        }

        let content = serde_json::to_string_pretty(devices)
            .with_context(|| "Failed to serialize known devices")?;

        tokio::fs::write(path, content)
            .await
            .with_context(|| format!("Failed to write known devices to {:?}", path))?;

        debug!("Saved {} known devices to {:?}", devices.count(), path);
        Ok(())
    }
}

/// Result of checking device status
#[derive(Debug, Clone)]
pub struct DeviceCheckResult {
    /// Whether this is a new/unknown device
    pub is_new: bool,

    /// The device fingerprint
    pub fingerprint: String,

    /// Human-readable device name
    pub device_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_fingerprint_generation() {
        let fp = DeviceFingerprint::current().unwrap();
        assert!(!fp.fingerprint.is_empty());
        assert_eq!(fp.fingerprint.len(), 64); // Blake3 hex output
        assert!(!fp.device_name.is_empty());
    }

    #[test]
    fn test_fingerprint_stability() {
        let fp1 = DeviceFingerprint::current().unwrap();
        let fp2 = DeviceFingerprint::current().unwrap();
        assert_eq!(fp1.fingerprint, fp2.fingerprint);
    }

    #[test]
    fn test_short_id() {
        let fp = DeviceFingerprint::current().unwrap();
        assert_eq!(fp.short_id().len(), 16);
    }

    #[test]
    fn test_known_devices_add() {
        let mut known = KnownDevices::new();
        let fp = DeviceFingerprint::current().unwrap();

        // First add should return true (new)
        assert!(known.add_or_update(fp.clone()));

        // Second add should return false (existing)
        assert!(!known.add_or_update(fp.clone()));

        assert_eq!(known.count(), 1);
    }

    #[test]
    fn test_known_devices_max_limit() {
        let mut known = KnownDevices::with_max(2);

        // Create fake fingerprints
        let mut fp1 = DeviceFingerprint::current().unwrap();
        fp1.fingerprint = "aaa".repeat(21) + "a"; // 64 chars
        fp1.first_seen = Utc::now() - chrono::Duration::hours(3);
        fp1.last_seen = Utc::now() - chrono::Duration::hours(3);

        let mut fp2 = DeviceFingerprint::current().unwrap();
        fp2.fingerprint = "bbb".repeat(21) + "b";
        fp2.first_seen = Utc::now() - chrono::Duration::hours(2);
        fp2.last_seen = Utc::now() - chrono::Duration::hours(2);

        let mut fp3 = DeviceFingerprint::current().unwrap();
        fp3.fingerprint = "ccc".repeat(21) + "c";
        fp3.first_seen = Utc::now() - chrono::Duration::hours(1);
        fp3.last_seen = Utc::now() - chrono::Duration::hours(1);

        known.add_or_update(fp1.clone());
        known.add_or_update(fp2.clone());
        assert_eq!(known.count(), 2);

        // Adding third should remove oldest (fp1)
        known.add_or_update(fp3.clone());
        assert_eq!(known.count(), 2);
        assert!(!known.is_known(&fp1.fingerprint));
        assert!(known.is_known(&fp2.fingerprint));
        assert!(known.is_known(&fp3.fingerprint));
    }

    #[tokio::test]
    async fn test_device_manager_new_device() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("known_devices.json");

        let manager = DeviceManager::new(Some(storage_path.clone()))
            .await
            .unwrap();

        // First check should detect new device
        let result = manager.check_and_register().await.unwrap();
        assert!(result.is_new);

        // Second check should not be new
        let result2 = manager.check_and_register().await.unwrap();
        assert!(!result2.is_new);
    }

    #[tokio::test]
    async fn test_device_manager_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("known_devices.json");

        // First manager instance
        {
            let manager = DeviceManager::new(Some(storage_path.clone()))
                .await
                .unwrap();
            manager.check_and_register().await.unwrap();
        }

        // Second manager instance should load persisted data
        {
            let manager = DeviceManager::new(Some(storage_path.clone()))
                .await
                .unwrap();
            let is_new = manager.is_new_device().await;
            assert!(!is_new, "Device should be recognized from persisted data");
        }
    }

    #[tokio::test]
    async fn test_device_manager_remove() {
        let manager = DeviceManager::new(None).await.unwrap();
        manager.register_current_device().await.unwrap();

        let fp = manager.fingerprint_string().to_string();
        let removed = manager.remove_device(&fp).await.unwrap();
        assert!(removed.is_some());

        assert!(manager.is_new_device().await);
    }
}
