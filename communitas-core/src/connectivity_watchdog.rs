// Copyright (c) 2025 Saorsa Labs Limited
//
// Connectivity watchdog for detecting internet collapse and network degradation
//
// Implements failure detection as specified in MESH_CAPABILITIES.md §3.2 Scenario A
// to enable graceful degradation to local-only mode when bootstrap/coordinators
// become unreachable.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Connectivity watchdog monitors bootstrap/coordinator reachability
///
/// When all bootstrap nodes fail for DETECTION_THRESHOLD, the system enters
/// local-only mode where:
/// - WAN dials are suspended
/// - Only LAN/loopback peers are contacted
/// - CRDT sync continues with reachable peers
///
/// The watchdog automatically exits local-only mode when bootstrap succeeds.
#[derive(Clone)]
pub struct ConnectivityWatchdog {
    /// Is the system currently in local-only mode?
    local_only_mode: Arc<AtomicBool>,

    /// Last successful bootstrap/coordinator contact
    last_success: Arc<RwLock<Option<Instant>>>,

    /// Watchdog configuration
    config: WatchdogConfig,
}

/// Configuration for connectivity watchdog
#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    /// How often to ping bootstrap nodes (default: 1 second)
    pub check_interval: Duration,

    /// How long all nodes must be unreachable before entering local-only (default: 10 seconds)
    pub detection_threshold: Duration,

    /// How long to wait in local-only mode before re-checking WAN (default: 30 seconds)
    pub recovery_check_interval: Duration,

    /// Enable watchdog monitoring
    pub enabled: bool,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(1),
            detection_threshold: Duration::from_secs(10),
            recovery_check_interval: Duration::from_secs(30),
            enabled: true,
        }
    }
}

impl ConnectivityWatchdog {
    /// Create a new connectivity watchdog
    pub fn new(config: WatchdogConfig) -> Self {
        Self {
            local_only_mode: Arc::new(AtomicBool::new(false)),
            last_success: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Check if system is in local-only mode
    pub fn is_local_only_mode(&self) -> bool {
        self.local_only_mode.load(Ordering::Acquire)
    }

    /// Record successful bootstrap/coordinator contact
    ///
    /// This resets the failure detection timer and exits local-only mode
    pub async fn record_success(&self) {
        let mut last_success = self.last_success.write().await;
        *last_success = Some(Instant::now());

        let was_local_only = self.local_only_mode.swap(false, Ordering::AcqRel);
        if was_local_only {
            info!("🌐 Connectivity restored - exiting local-only mode");
        }
    }

    /// Record failed bootstrap/coordinator contact
    ///
    /// If enough time has passed without success, enter local-only mode
    pub async fn record_failure(&self) {
        let last_success = self.last_success.read().await;

        if let Some(last_ok) = *last_success {
            let elapsed = Instant::now().duration_since(last_ok);

            if elapsed > self.config.detection_threshold {
                let was_online = !self.local_only_mode.swap(true, Ordering::AcqRel);
                if was_online {
                    warn!(
                        "⚠️  All bootstrap nodes unreachable for {:?} - entering local-only mode",
                        elapsed
                    );
                    warn!("    WAN connections suspended, operating with local peers only");
                }
            } else {
                debug!(
                    "Bootstrap nodes unreachable for {:?} (threshold: {:?})",
                    elapsed, self.config.detection_threshold
                );
            }
        } else {
            // First failure, start timer
            drop(last_success);
            let mut last_success = self.last_success.write().await;
            if last_success.is_none() {
                *last_success = Some(Instant::now());
            }
        }
    }

    /// Start background monitoring task
    ///
    /// This spawns a tokio task that periodically checks connectivity.
    /// The caller must provide a health check function that returns true
    /// if bootstrap/coordinator is reachable.
    pub fn start_monitoring<F, Fut>(self, health_check: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn() -> Fut + Send + 'static,
        Fut: std::future::Future<Output = bool> + Send + 'static,
    {
        tokio::spawn(async move {
            if !self.config.enabled {
                info!("Connectivity watchdog disabled");
                return;
            }

            info!(
                "Starting connectivity watchdog (threshold: {:?})",
                self.config.detection_threshold
            );

            loop {
                let interval = if self.is_local_only_mode() {
                    // In local-only mode, check less frequently
                    self.config.recovery_check_interval
                } else {
                    // Normal mode, check frequently
                    self.config.check_interval
                };

                sleep(interval).await;

                // Run health check
                match tokio::time::timeout(Duration::from_secs(5), health_check()).await {
                    Ok(true) => {
                        self.record_success().await;
                    }
                    Ok(false) => {
                        self.record_failure().await;
                    }
                    Err(_) => {
                        // Timeout
                        debug!("Health check timed out after 5s");
                        self.record_failure().await;
                    }
                }
            }
        })
    }

    /// Get time since last successful contact (for diagnostics)
    pub async fn time_since_last_success(&self) -> Option<Duration> {
        let last_success = self.last_success.read().await;
        last_success.map(|instant| Instant::now().duration_since(instant))
    }

    /// Force enter local-only mode (for testing)
    pub fn force_local_only(&self) {
        self.local_only_mode.store(true, Ordering::Release);
    }

    /// Force exit local-only mode (for testing)
    pub fn force_online(&self) {
        self.local_only_mode.store(false, Ordering::Release);
    }
}

impl Default for ConnectivityWatchdog {
    fn default() -> Self {
        Self::new(WatchdogConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_watchdog_enters_local_only_after_threshold() {
        let config = WatchdogConfig {
            detection_threshold: Duration::from_millis(100),
            ..Default::default()
        };

        let watchdog = ConnectivityWatchdog::new(config);

        // Initially online
        assert!(!watchdog.is_local_only_mode());

        // First failure starts timer
        watchdog.record_failure().await;
        assert!(!watchdog.is_local_only_mode());

        // Wait past threshold
        sleep(Duration::from_millis(150)).await;

        // Next failure triggers local-only
        watchdog.record_failure().await;
        assert!(watchdog.is_local_only_mode());
    }

    #[tokio::test]
    async fn test_watchdog_exits_local_only_on_success() {
        let watchdog = ConnectivityWatchdog::default();

        // Force into local-only mode
        watchdog.force_local_only();
        assert!(watchdog.is_local_only_mode());

        // Success exits local-only
        watchdog.record_success().await;
        assert!(!watchdog.is_local_only_mode());
    }

    #[tokio::test]
    async fn test_monitoring_task() {
        let config = WatchdogConfig {
            check_interval: Duration::from_millis(50),
            detection_threshold: Duration::from_millis(100),
            ..Default::default()
        };

        let watchdog = ConnectivityWatchdog::new(config);
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        // Health check that fails
        let health_check = move || {
            let count = call_count_clone.clone();
            async move {
                let mut c = count.lock().await;
                *c += 1;
                false // Always fail
            }
        };

        let handle = watchdog.clone().start_monitoring(health_check);

        // Wait for several checks
        sleep(Duration::from_millis(250)).await;

        // Verify health check was called multiple times
        let count = *call_count.lock().await;
        assert!(
            count >= 3,
            "Health check should be called at least 3 times, got {}",
            count
        );

        // Should be in local-only mode now
        assert!(watchdog.is_local_only_mode());

        handle.abort();
    }

    #[tokio::test]
    async fn test_time_since_last_success() {
        let watchdog = ConnectivityWatchdog::default();

        // No success yet
        assert!(watchdog.time_since_last_success().await.is_none());

        // Record success
        watchdog.record_success().await;
        sleep(Duration::from_millis(50)).await;

        let elapsed = watchdog.time_since_last_success().await.unwrap();
        assert!(elapsed >= Duration::from_millis(50));
        assert!(elapsed < Duration::from_millis(200));
    }
}
