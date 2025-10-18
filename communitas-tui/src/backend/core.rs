use anyhow::Result;
use communitas_core::types::DeviceType;
use communitas_core::{
    AuthService, CoreContext, SessionInfo,
    encrypted_storage::{EncryptedStorageManager, RecentIdentity, StorageConfig},
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use super::events::{BackendEvent, EventFilter, EventManager};
use super::offline_queue::{OfflineQueue, QueuedOperation, QueuedOperationEntry, SyncProgress, SyncResult};
use tokio::sync::{mpsc, RwLock};

/// Backend wrapper around CoreContext and AuthService
pub struct Backend {
    /// Authentication service
    auth_service: AuthService,
    /// Core context (None if not authenticated)
    ctx: Option<CoreContext>,
    /// Data directory
    data_dir: PathBuf,
    /// Offline mode (network unavailable)
    offline: bool,
    /// Event subscription manager
    event_manager: EventManager,
    /// Offline operation queue (None until authenticated)
    offline_queue: Option<OfflineQueue>,
    /// Offline mode flag (user-controlled offline mode)
    offline_mode: bool,
    /// Network error simulation (for testing)
    network_error_simulation: bool,
    /// Sync progress subscribers
    sync_progress_subs: Arc<RwLock<HashMap<u64, mpsc::Sender<SyncProgress>>>>,
    /// Next sync progress subscription ID
    next_sync_sub_id: AtomicU64,
}

impl Backend {
    /// Create new backend with AuthService
    pub async fn new(data_dir: PathBuf, offline: bool) -> Result<Self> {
        // Create storage configuration
        let config = StorageConfig {
            vault_dir: data_dir.join("vaults"),
            use_keyring: true,
            ..Default::default()
        };

        // Initialize encrypted storage manager
        let storage_manager = EncryptedStorageManager::new(config).await?;

        // Create auth service
        let auth_service = AuthService::new(storage_manager);

        Ok(Self {
            auth_service,
            ctx: None,
            data_dir,
            offline,
            event_manager: EventManager::new(),
            offline_queue: None,
            offline_mode: false,
            network_error_simulation: false,
            sync_progress_subs: Arc::new(RwLock::new(HashMap::new())),
            next_sync_sub_id: AtomicU64::new(1),
        })
    }

    /// Create new backend with custom configuration
    pub async fn new_with_config(
        data_dir: PathBuf,
        pbkdf2_iterations: u32,
        use_keyring: bool,
        offline: bool,
    ) -> Result<Self> {
        // Create storage configuration with custom values
        let config = StorageConfig {
            vault_dir: data_dir.join("vaults"),
            pbkdf2_iterations,
            use_keyring,
            ..Default::default()
        };

        // Initialize encrypted storage manager
        let storage_manager = EncryptedStorageManager::new(config).await?;

        // Create auth service
        let auth_service = AuthService::new(storage_manager);

        Ok(Self {
            auth_service,
            ctx: None,
            data_dir,
            offline,
            event_manager: EventManager::new(),
            offline_queue: None,
            offline_mode: false,
            network_error_simulation: false,
            sync_progress_subs: Arc::new(RwLock::new(HashMap::new())),
            next_sync_sub_id: AtomicU64::new(1),
        })
    }

    // ========================================================================
    // Authentication Methods (delegated to AuthService)
    // ========================================================================

    /// Create a new vault and login
    pub async fn create_vault(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<SessionInfo> {
        tracing::info!("Creating vault for: {}", four_words);

        // Create vault via auth service
        let _vault_id = self
            .auth_service
            .create_vault(four_words, password, display_name)
            .await?;

        // Login to the new vault
        let session_info = self
            .auth_service
            .login(four_words, password, Some("TUI"))
            .await?;

        tracing::info!("Vault created and logged in: {}", four_words);
        Ok(session_info)
    }

    /// Create a new vault and login with timeout
    pub async fn create_vault_with_timeout(
        &mut self,
        four_words: &str,
        password: &str,
        display_name: &str,
    ) -> Result<SessionInfo> {
        use tokio::time::{Duration, timeout};

        let result = timeout(
            Duration::from_secs(60), // 60 second timeout
            self.create_vault(four_words, password, display_name),
        )
        .await;

        match result {
            Ok(Ok(session_info)) => Ok(session_info),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow::anyhow!("Vault creation timed out after 60 seconds")),
        }
    }

    /// Login with four-word identity and password
    pub async fn login(&mut self, four_words: &str, password: &str) -> Result<SessionInfo> {
        tracing::info!("Logging in: {}", four_words);

        let session_info = self
            .auth_service
            .login(four_words, password, Some("TUI"))
            .await?;

        tracing::info!("Login successful: {}", four_words);
        Ok(session_info)
    }

    /// Logout current session
    pub async fn logout(&mut self) -> Result<()> {
        tracing::info!("Logging out");

        self.auth_service.logout().await?;
        self.ctx = None;

        tracing::info!("Logout successful");
        Ok(())
    }

    /// Get current session info
    pub fn get_current_session(&self) -> Option<SessionInfo> {
        self.auth_service.get_current_session()
    }

    /// Check if user is logged in
    pub fn is_logged_in(&self) -> bool {
        self.auth_service.is_logged_in()
    }

    /// Get list of recent identities
    pub async fn get_recent_identities(&self) -> Result<Vec<RecentIdentity>> {
        self.auth_service.get_recent_identities().await
    }

    /// Try auto-login with last used identity
    pub async fn try_auto_login(&mut self) -> Result<Option<SessionInfo>> {
        self.auth_service.try_auto_login().await
    }

    /// Switch to another identity
    pub async fn switch_identity(&mut self, four_words: &str) -> Result<SessionInfo> {
        tracing::info!("Switching to identity: {}", four_words);

        let session_info = self.auth_service.switch_identity(four_words).await?;

        // Clear CoreContext when switching
        self.ctx = None;

        tracing::info!("Switched to identity: {}", four_words);
        Ok(session_info)
    }

    /// Check if vault exists
    pub async fn vault_exists(&self, four_words: &str) -> Result<bool> {
        self.auth_service.vault_exists(four_words).await
    }

    /// Check if identity has passkey registered
    pub async fn has_passkey(&self, four_words: &str) -> Result<bool> {
        self.auth_service.passkey_has_passkey(four_words).await
    }

    // ========================================================================
    // CoreContext Integration
    // ========================================================================

    /// Initialize CoreContext with current session
    pub async fn initialize_core_context(&mut self) -> Result<()> {
        let session = self
            .auth_service
            .get_current_session()
            .ok_or_else(|| anyhow::anyhow!("No active session"))?;

        tracing::info!("Initializing CoreContext for: {}", session.four_words);

        // Create storage directory for this user
        let storage_dir = self.data_dir.join("core_data").join(&session.four_words);

        let ctx = CoreContext::initialize(
            session.four_words.clone(),
            session.display_name.clone(),
            "TUI".to_string(),
            DeviceType::Desktop,
            storage_dir,
        )
        .await
        .map_err(|e| anyhow::anyhow!("CoreContext initialization failed: {}", e))?;

        self.ctx = Some(ctx);

        // Initialize offline queue for this user
        let queue_dir = self.data_dir.join("offline_queue").join(&session.four_words);
        let offline_queue = OfflineQueue::new(queue_dir).await?;
        self.offline_queue = Some(offline_queue);

        tracing::info!("CoreContext and offline queue initialized successfully");
        Ok(())
    }

    /// Check if CoreContext is initialized
    pub fn is_core_initialized(&self) -> bool {
        self.ctx.is_some()
    }

    /// Get reference to CoreContext (returns error if not initialized)
    pub fn context(&self) -> Result<&CoreContext> {
        self.ctx
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("CoreContext not initialized"))
    }

    /// Get mutable reference to CoreContext (returns error if not initialized)
    pub fn context_mut(&mut self) -> Result<&mut CoreContext> {
        self.ctx
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("CoreContext not initialized"))
    }

    /// Get current identity four-words
    pub fn identity(&self) -> Option<String> {
        self.get_current_session().map(|s| s.four_words)
    }

    // ========================================================================
    // Network / DHT Methods
    // ========================================================================

    /// Check DHT connection status
    pub async fn check_dht_connection(&self) -> Result<bool> {
        if self.offline {
            return Ok(false);
        }

        // Return true if context is initialized
        Ok(self.ctx.is_some())
    }

    /// Get bootstrap nodes (placeholder - gossip overlay handles this)
    pub async fn get_bootstrap_nodes(&self) -> Result<Vec<String>> {
        // Gossip overlay handles peer discovery automatically
        Ok(vec![])
    }

    /// Add bootstrap node (placeholder - gossip overlay handles this)
    pub async fn add_bootstrap_node(&self, _node: String) -> Result<()> {
        // Gossip overlay handles peer connections
        Ok(())
    }

    // ========================================================================
    // Utility Methods
    // ========================================================================

    /// Generate a new four-word identity
    pub fn generate_four_words() -> String {
        use rand::RngCore;
        use rand::rngs::OsRng;
        // Removed: saorsa-core imports - replaced with communitas-core
        // use saorsa_core::address::NetworkAddress;
        // use saorsa_core::fwid::fw_check;
        use std::net::Ipv4Addr;

        let mut rng = OsRng;
        const MIN_PORT: u16 = 1024;
        const PORT_SPAN: u32 = u16::MAX as u32 - MIN_PORT as u32 + 1;
        const GENERATION_ATTEMPTS: usize = 1000;

        for _ in 0..GENERATION_ATTEMPTS {
            let ipv4 = Ipv4Addr::from(rng.next_u32());
            let port = (rng.next_u32() % PORT_SPAN) as u16 + MIN_PORT;
            let addr = std::net::SocketAddr::from((ipv4, port));

            if let Ok(words) = communitas_core::identity::conn_words(&addr) {
                // Validate the words are correct
                if communitas_core::identity::validate_id_words(&words) {
                    return words;
                }
            }
        }

        // Fallback: return a simple valid format (this should rarely happen)
        "ocean-forest-moon-star".to_string()
    }

    /// Validate four-word format
    pub fn validate_four_words(four_words: &str) -> bool {
        // Basic validation: exactly 4 words separated by hyphens
        let parts: Vec<&str> = four_words.split('-').collect();
        parts.len() == 4 && parts.iter().all(|p| !p.is_empty())
    }

    // ========================================================================
    // Event Subscription Methods
    // ========================================================================

    /// Subscribe to all entity events
    ///
    /// Returns subscription ID that can be used to unsubscribe later.
    pub async fn subscribe_entity_events(
        &mut self,
        sender: mpsc::Sender<BackendEvent>,
    ) -> Result<u64> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot subscribe to events"
            ));
        }

        let id = self
            .event_manager
            .subscribe(sender, EventFilter::all())
            .await;
        Ok(id)
    }

    /// Subscribe to message events
    ///
    /// Returns subscription ID that can be used to unsubscribe later.
    pub async fn subscribe_message_events(
        &mut self,
        sender: mpsc::Sender<BackendEvent>,
    ) -> Result<u64> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot subscribe to events"
            ));
        }

        // Use all filter for now - message events will be filtered by type automatically
        let id = self
            .event_manager
            .subscribe(sender, EventFilter::all())
            .await;
        Ok(id)
    }

    /// Subscribe to entity events with filters
    ///
    /// Returns subscription ID that can be used to unsubscribe later.
    pub async fn subscribe_entity_events_filtered(
        &mut self,
        sender: mpsc::Sender<BackendEvent>,
        entity_type: Option<communitas_core::crdt::EntityType>,
        entity_id: Option<String>,
    ) -> Result<u64> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!(
                "CoreContext not initialized - cannot subscribe to events"
            ));
        }

        let filter = EventFilter {
            entity_type,
            entity_id,
        };

        let id = self.event_manager.subscribe(sender, filter).await;
        Ok(id)
    }

    /// Unsubscribe from events
    ///
    /// Returns true if subscription was found and removed, false otherwise.
    pub async fn unsubscribe(&mut self, subscription_id: u64) -> Result<()> {
        let removed = self.event_manager.unsubscribe(subscription_id).await;
        if removed {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Subscription ID {} not found",
                subscription_id
            ))
        }
    }

    /// Enable event queue for offline support
    ///
    /// Events will be queued when no subscribers are active and delivered
    /// when new subscribers connect.
    pub async fn enable_event_queue(&mut self, max_size: usize) -> Result<()> {
        self.event_manager.enable_queue(max_size).await;
        Ok(())
    }

    /// Publish an event to all subscribers (internal use)
    pub(crate) async fn publish_event(&self, event: BackendEvent) {
        self.event_manager.publish(event).await;
    }

    // ========================================================================
    // Offline Queue Methods
    // ========================================================================

    /// Set offline mode (for testing and manual control)
    pub async fn set_offline_mode(&mut self, offline: bool) -> Result<()> {
        self.offline_mode = offline;
        Ok(())
    }

    /// Check if in offline mode
    pub fn is_offline_mode(&self) -> bool {
        self.offline_mode || self.network_error_simulation
    }

    /// Simulate network error (for testing)
    pub async fn simulate_network_error(&mut self, simulate: bool) -> Result<()> {
        self.network_error_simulation = simulate;
        Ok(())
    }

    /// Set queue size limit
    pub async fn set_queue_size_limit(&mut self, max_size: usize) -> Result<()> {
        if let Some(queue) = &mut self.offline_queue {
            queue.set_max_size(max_size);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Offline queue not initialized - authenticate first"))
        }
    }

    /// Queue entity creation operation
    pub async fn queue_create_entity(
        &mut self,
        name: String,
        entity_type: communitas_core::crdt::EntityType,
        members: Vec<String>,
    ) -> Result<String> {
        if !self.is_logged_in() {
            return Err(anyhow::anyhow!("Must be authenticated to queue operations"));
        }

        let queue = self.offline_queue.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;

        let operation = QueuedOperation::CreateEntity {
            name,
            entity_type,
            members,
        };

        queue.enqueue(operation, 0).await
    }

    /// Queue message send operation
    pub async fn queue_send_message(
        &mut self,
        entity_id: String,
        entity_type: communitas_core::crdt::EntityType,
        text: String,
    ) -> Result<String> {
        if !self.is_logged_in() {
            return Err(anyhow::anyhow!("Must be authenticated to queue operations"));
        }

        let queue = self.offline_queue.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;

        let operation = QueuedOperation::SendMessage {
            entity_id,
            entity_type,
            text,
        };

        queue.enqueue(operation, 0).await
    }

    /// Queue message send operation with priority
    pub async fn queue_send_message_with_priority(
        &mut self,
        entity_id: String,
        entity_type: communitas_core::crdt::EntityType,
        text: String,
        priority: u8,
    ) -> Result<String> {
        if !self.is_logged_in() {
            return Err(anyhow::anyhow!("Must be authenticated to queue operations"));
        }

        let queue = self.offline_queue.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;

        let operation = QueuedOperation::SendMessage {
            entity_id,
            entity_type,
            text,
        };

        queue.enqueue(operation, priority).await
    }

    /// Queue member addition operation
    pub async fn queue_add_member(
        &mut self,
        entity_id: String,
        entity_type: communitas_core::crdt::EntityType,
        member_id: String,
    ) -> Result<String> {
        if !self.is_logged_in() {
            return Err(anyhow::anyhow!("Must be authenticated to queue operations"));
        }

        let queue = self.offline_queue.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;

        let operation = QueuedOperation::AddMember {
            entity_id,
            entity_type,
            member_id,
        };

        queue.enqueue(operation, 0).await
    }

    /// Get all queued operations
    pub async fn get_queued_operations(&self) -> Result<Vec<QueuedOperationEntry>> {
        let queue = self.offline_queue.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;

        Ok(queue.get_all())
    }

    /// Sync queued operations
    ///
    /// Attempts to execute all queued operations when back online.
    /// Returns results for each operation (Success, Failed, or Skipped).
    /// Successfully executed operations are removed from queue.
    /// Failed operations remain for retry.
    pub async fn sync_queued_operations(&mut self) -> Result<Vec<SyncResult>> {
        if !self.is_core_initialized() {
            return Err(anyhow::anyhow!("CoreContext not initialized"));
        }

        // Get all operations first (releases borrow)
        let operations = {
            let queue = self.offline_queue.as_ref()
                .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;
            queue.get_all()
        };

        if operations.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let total = operations.len();

        // Track successful operation IDs for removal
        let mut successful_ops = Vec::new();

        for (idx, entry) in operations.into_iter().enumerate() {
            // Send progress update
            self.publish_sync_progress(SyncProgress {
                total,
                completed: idx,
                current_operation_id: Some(entry.id.clone()),
            }).await;

            // Check for duplicates by comparing with previously queued ops
            let is_duplicate = {
                let queue = self.offline_queue.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;
                queue.is_duplicate(&entry.operation) && idx > 0
            };

            if is_duplicate {
                results.push(SyncResult::Skipped {
                    operation_id: entry.id.clone(),
                    reason: "Duplicate operation detected".to_string(),
                });
                successful_ops.push(entry.id.clone());
                continue;
            }

            // Simulate network error if testing
            if self.network_error_simulation {
                results.push(SyncResult::Failed {
                    operation_id: entry.id.clone(),
                    error: "Network error (simulated)".to_string(),
                });
                continue;
            }

            // Execute operation
            let result = self.execute_queued_operation(entry.operation.clone()).await;

            match result {
                Ok(_) => {
                    results.push(SyncResult::Success {
                        operation_id: entry.id.clone(),
                    });
                    successful_ops.push(entry.id);
                }
                Err(e) => {
                    results.push(SyncResult::Failed {
                        operation_id: entry.id.clone(),
                        error: e.to_string(),
                    });
                    // Keep failed operations in queue for retry
                }
            }
        }

        // Remove successful operations from queue
        if !successful_ops.is_empty() {
            let queue = self.offline_queue.as_mut()
                .ok_or_else(|| anyhow::anyhow!("Offline queue not initialized"))?;
            for op_id in successful_ops {
                queue.remove(&op_id).await?;
            }
        }

        // Final progress update
        self.publish_sync_progress(SyncProgress {
            total,
            completed: total,
            current_operation_id: None,
        }).await;

        Ok(results)
    }

    /// Execute a queued operation
    async fn execute_queued_operation(&mut self, operation: QueuedOperation) -> Result<()> {
        match operation {
            QueuedOperation::CreateEntity { name, entity_type, members } => {
                self.create_entity(name, entity_type, members).await?;
                Ok(())
            }
            QueuedOperation::SendMessage { entity_id, entity_type, text } => {
                self.send_message(entity_id, entity_type, text).await?;
                Ok(())
            }
            QueuedOperation::AddMember { entity_id, entity_type, member_id } => {
                self.add_entity_member(entity_type, &entity_id, member_id).await?;
                Ok(())
            }
            QueuedOperation::RemoveMember { entity_id, entity_type, member_id } => {
                self.remove_entity_member(entity_type, &entity_id, member_id).await?;
                Ok(())
            }
        }
    }

    /// Subscribe to sync progress updates
    pub async fn subscribe_sync_progress(
        &mut self,
        sender: mpsc::Sender<SyncProgress>,
    ) -> Result<()> {
        let id = self.next_sync_sub_id.fetch_add(1, Ordering::SeqCst);
        let mut subs = self.sync_progress_subs.write().await;
        subs.insert(id, sender);
        Ok(())
    }

    /// Publish sync progress to all subscribers
    async fn publish_sync_progress(&self, progress: SyncProgress) {
        let subs = self.sync_progress_subs.read().await;
        for sender in subs.values() {
            let _ = sender.send(progress.clone()).await;
        }
    }
}
