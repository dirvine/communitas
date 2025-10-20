//! Plugin system for extensible TUI functionality
//!
//! Provides a trait-based plugin interface for extending the application
//! with custom features, event handlers, and UI components.

use std::collections::HashMap;
use std::fmt;
use tuirealm::Frame;
use tuirealm::event::Event;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Plugin error types
#[derive(Debug, Clone, PartialEq)]
pub enum PluginError {
    /// Plugin failed to load
    LoadFailed(String),
    /// Plugin failed to unload
    UnloadFailed(String),
    /// Plugin is already loaded
    AlreadyLoaded(String),
    /// Plugin not found
    NotFound(String),
    /// Event handling failed
    EventHandlingFailed(String),
    /// Render failed
    RenderFailed(String),
    /// Generic error
    Other(String),
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::LoadFailed(msg) => write!(f, "Plugin load failed: {}", msg),
            PluginError::UnloadFailed(msg) => write!(f, "Plugin unload failed: {}", msg),
            PluginError::AlreadyLoaded(name) => write!(f, "Plugin '{}' is already loaded", name),
            PluginError::NotFound(name) => write!(f, "Plugin '{}' not found", name),
            PluginError::EventHandlingFailed(msg) => write!(f, "Event handling failed: {}", msg),
            PluginError::RenderFailed(msg) => write!(f, "Render failed: {}", msg),
            PluginError::Other(msg) => write!(f, "Plugin error: {}", msg),
        }
    }
}

impl std::error::Error for PluginError {}

/// Plugin event types
#[derive(Debug, Clone, PartialEq)]
pub enum PluginEvent {
    /// Message received
    MessageReceived { content: String, sender: String },
    /// Channel changed
    ChannelChanged { channel_id: String },
    /// User joined
    UserJoined { user_id: String },
    /// User left
    UserLeft { user_id: String },
    /// Custom event
    Custom { event_type: String, data: String },
}

/// Plugin command
#[derive(Debug, Clone, PartialEq)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
}

impl PluginCommand {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        usage: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: usage.into(),
        }
    }
}

/// Plugin metadata
#[derive(Debug, Clone, PartialEq)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
}

impl PluginMetadata {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            author: author.into(),
            description: description.into(),
        }
    }
}

/// Plugin trait interface
///
/// Plugins implement this trait to extend the application with custom functionality.
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Called when plugin is loaded
    fn on_load(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> PluginResult<()> {
        Ok(())
    }

    /// Handle plugin events
    fn on_event(&mut self, event: &PluginEvent) -> PluginResult<bool> {
        let _ = event; // Unused parameter
        Ok(false) // Not handled by default
    }

    /// Handle UI events
    fn on_ui_event(&mut self, event: &Event<tuirealm::NoUserEvent>) -> PluginResult<bool> {
        let _ = event; // Unused parameter
        Ok(false) // Not handled by default
    }

    /// Handle custom commands
    fn on_command(&mut self, command: &str, args: &[String]) -> PluginResult<Option<String>> {
        let _ = (command, args); // Unused parameters
        Ok(None) // Not handled by default
    }

    /// Get plugin commands
    fn commands(&self) -> Vec<PluginCommand> {
        Vec::new() // No commands by default
    }

    /// Plugin-specific rendering (optional)
    fn render(
        &mut self,
        frame: &mut Frame,
        area: tuirealm::ratatui::layout::Rect,
    ) -> PluginResult<()> {
        let _ = (frame, area); // Unused parameters
        Ok(())
    }

    /// Check if plugin is enabled
    fn is_enabled(&self) -> bool {
        true // Enabled by default
    }

    /// Enable or disable the plugin
    fn set_enabled(&mut self, enabled: bool) -> PluginResult<()> {
        let _ = enabled; // Unused parameter
        Ok(())
    }
}

/// Plugin state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
    Error,
}

/// Registered plugin entry
struct PluginEntry {
    plugin: Box<dyn Plugin>,
    state: PluginState,
}

/// Plugin manager
///
/// Manages plugin lifecycle, event dispatching, and command routing.
pub struct PluginManager {
    plugins: HashMap<String, PluginEntry>,
    event_log: Vec<PluginEvent>,
    max_log_size: usize,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            event_log: Vec::new(),
            max_log_size: 100,
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> PluginResult<()> {
        let metadata = plugin.metadata();
        let name = metadata.name.clone();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name));
        }

        self.plugins.insert(
            name.clone(),
            PluginEntry {
                plugin,
                state: PluginState::Unloaded,
            },
        );

        Ok(())
    }

    /// Load a plugin by name
    pub fn load(&mut self, name: &str) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        entry.state = PluginState::Loading;

        match entry.plugin.on_load() {
            Ok(()) => {
                entry.state = PluginState::Loaded;
                Ok(())
            }
            Err(e) => {
                entry.state = PluginState::Error;
                Err(e)
            }
        }
    }

    /// Unload a plugin by name
    pub fn unload(&mut self, name: &str) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        entry.state = PluginState::Unloading;

        match entry.plugin.on_unload() {
            Ok(()) => {
                entry.state = PluginState::Unloaded;
                Ok(())
            }
            Err(e) => {
                entry.state = PluginState::Error;
                Err(e)
            }
        }
    }

    /// Remove a plugin entirely
    pub fn unregister(&mut self, name: &str) -> PluginResult<()> {
        // First unload if loaded
        if let Some(entry) = self.plugins.get(name)
            && entry.state == PluginState::Loaded
        {
            self.unload(name)?;
        }

        self.plugins
            .remove(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        Ok(())
    }

    /// Broadcast event to all loaded plugins
    pub fn broadcast_event(&mut self, event: &PluginEvent) -> PluginResult<Vec<String>> {
        // Add to event log
        self.event_log.push(event.clone());
        if self.event_log.len() > self.max_log_size {
            self.event_log.remove(0);
        }

        let mut handlers = Vec::new();

        for (name, entry) in &mut self.plugins {
            if entry.state == PluginState::Loaded && entry.plugin.is_enabled() {
                match entry.plugin.on_event(event) {
                    Ok(true) => handlers.push(name.clone()),
                    Ok(false) => {} // Not handled
                    Err(e) => {
                        return Err(PluginError::EventHandlingFailed(format!("{}: {}", name, e)));
                    }
                }
            }
        }

        Ok(handlers)
    }

    /// Execute a command through plugins
    pub fn execute_command(
        &mut self,
        command: &str,
        args: &[String],
    ) -> PluginResult<Option<String>> {
        for (name, entry) in &mut self.plugins {
            if entry.state == PluginState::Loaded && entry.plugin.is_enabled() {
                match entry.plugin.on_command(command, args) {
                    Ok(Some(result)) => return Ok(Some(result)),
                    Ok(None) => continue, // Try next plugin
                    Err(e) => return Err(PluginError::Other(format!("{}: {}", name, e))),
                }
            }
        }

        Ok(None) // No plugin handled the command
    }

    /// Get all plugin metadata
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins
            .values()
            .map(|entry| entry.plugin.metadata())
            .collect()
    }

    /// Get plugin state
    pub fn get_state(&self, name: &str) -> Option<PluginState> {
        self.plugins.get(name).map(|entry| entry.state)
    }

    /// Get all loaded plugins
    pub fn loaded_plugins(&self) -> Vec<String> {
        self.plugins
            .iter()
            .filter_map(|(name, entry)| {
                if entry.state == PluginState::Loaded {
                    Some(name.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all commands from all plugins
    pub fn all_commands(&self) -> Vec<(String, PluginCommand)> {
        let mut commands = Vec::new();

        for (plugin_name, entry) in &self.plugins {
            if entry.state == PluginState::Loaded {
                for cmd in entry.plugin.commands() {
                    commands.push((plugin_name.clone(), cmd));
                }
            }
        }

        commands
    }

    /// Get event log
    pub fn event_log(&self) -> &[PluginEvent] {
        &self.event_log
    }

    /// Clear event log
    pub fn clear_event_log(&mut self) {
        self.event_log.clear();
    }

    /// Get number of registered plugins
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Enable a plugin
    pub fn enable(&mut self, name: &str) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        entry.plugin.set_enabled(true)
    }

    /// Disable a plugin
    pub fn disable(&mut self, name: &str) -> PluginResult<()> {
        let entry = self
            .plugins
            .get_mut(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        entry.plugin.set_enabled(false)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for PluginManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginManager")
            .field("plugin_count", &self.plugins.len())
            .field(
                "plugins",
                &self
                    .plugins
                    .iter()
                    .map(|(name, entry)| (name.clone(), entry.state))
                    .collect::<Vec<_>>(),
            )
            .field("event_log_size", &self.event_log.len())
            .field("max_log_size", &self.max_log_size)
            .finish()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test plugin implementation
    struct TestPlugin {
        metadata: PluginMetadata,
        load_count: usize,
        unload_count: usize,
        event_count: usize,
        enabled: bool,
        should_fail_load: bool,
        should_fail_event: bool,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                metadata: PluginMetadata::new(name, "1.0.0", "Test Author", "A test plugin"),
                load_count: 0,
                unload_count: 0,
                event_count: 0,
                enabled: true,
                should_fail_load: false,
                should_fail_event: false,
            }
        }

        fn with_fail_load(mut self) -> Self {
            self.should_fail_load = true;
            self
        }

        fn with_fail_event(mut self) -> Self {
            self.should_fail_event = true;
            self
        }
    }

    impl Plugin for TestPlugin {
        fn metadata(&self) -> PluginMetadata {
            self.metadata.clone()
        }

        fn on_load(&mut self) -> PluginResult<()> {
            if self.should_fail_load {
                return Err(PluginError::LoadFailed("Test failure".to_string()));
            }
            self.load_count += 1;
            Ok(())
        }

        fn on_unload(&mut self) -> PluginResult<()> {
            self.unload_count += 1;
            Ok(())
        }

        fn on_event(&mut self, _event: &PluginEvent) -> PluginResult<bool> {
            if self.should_fail_event {
                return Err(PluginError::EventHandlingFailed("Test failure".to_string()));
            }
            self.event_count += 1;
            Ok(true) // Handled
        }

        fn on_command(&mut self, command: &str, args: &[String]) -> PluginResult<Option<String>> {
            if command == "test" {
                Ok(Some(format!("Executed with {} args", args.len())))
            } else {
                Ok(None)
            }
        }

        fn commands(&self) -> Vec<PluginCommand> {
            vec![PluginCommand::new("test", "Test command", "test [args...]")]
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }

        fn set_enabled(&mut self, enabled: bool) -> PluginResult<()> {
            self.enabled = enabled;
            Ok(())
        }
    }

    // ========================================================================
    // PluginMetadata Tests
    // ========================================================================

    #[test]
    fn test_plugin_metadata_creation() {
        let metadata = PluginMetadata::new("test", "1.0.0", "author", "description");
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "author");
        assert_eq!(metadata.description, "description");
    }

    #[test]
    fn test_plugin_metadata_equality() {
        let meta1 = PluginMetadata::new("test", "1.0.0", "author", "desc");
        let meta2 = PluginMetadata::new("test", "1.0.0", "author", "desc");
        let meta3 = PluginMetadata::new("other", "1.0.0", "author", "desc");
        assert_eq!(meta1, meta2);
        assert_ne!(meta1, meta3);
    }

    // ========================================================================
    // PluginCommand Tests
    // ========================================================================

    #[test]
    fn test_plugin_command_creation() {
        let cmd = PluginCommand::new("test", "Test command", "test [args]");
        assert_eq!(cmd.name, "test");
        assert_eq!(cmd.description, "Test command");
        assert_eq!(cmd.usage, "test [args]");
    }

    // ========================================================================
    // PluginEvent Tests
    // ========================================================================

    #[test]
    fn test_plugin_event_variants() {
        let msg = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };
        let channel = PluginEvent::ChannelChanged {
            channel_id: "general".to_string(),
        };
        let custom = PluginEvent::Custom {
            event_type: "test".to_string(),
            data: "data".to_string(),
        };

        assert!(matches!(msg, PluginEvent::MessageReceived { .. }));
        assert!(matches!(channel, PluginEvent::ChannelChanged { .. }));
        assert!(matches!(custom, PluginEvent::Custom { .. }));
    }

    #[test]
    fn test_plugin_event_equality() {
        let event1 = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };
        let event2 = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };
        let event3 = PluginEvent::MessageReceived {
            content: "bye".to_string(),
            sender: "alice".to_string(),
        };

        assert_eq!(event1, event2);
        assert_ne!(event1, event3);
    }

    // ========================================================================
    // PluginError Tests
    // ========================================================================

    #[test]
    fn test_plugin_error_display() {
        let err = PluginError::LoadFailed("test".to_string());
        assert_eq!(err.to_string(), "Plugin load failed: test");

        let err = PluginError::NotFound("plugin".to_string());
        assert_eq!(err.to_string(), "Plugin 'plugin' not found");
    }

    #[test]
    fn test_plugin_error_equality() {
        let err1 = PluginError::LoadFailed("test".to_string());
        let err2 = PluginError::LoadFailed("test".to_string());
        let err3 = PluginError::LoadFailed("other".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    // ========================================================================
    // PluginManager Tests
    // ========================================================================

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_plugin_manager_default() {
        let manager = PluginManager::default();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_register_plugin() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));
        manager.register(plugin).unwrap();
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_register_duplicate_plugin_fails() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        let result = manager.register(Box::new(TestPlugin::new("test")));
        assert!(matches!(result, Err(PluginError::AlreadyLoaded(_))));
    }

    #[test]
    fn test_load_plugin() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();
        assert_eq!(manager.get_state("test"), Some(PluginState::Loaded));
    }

    #[test]
    fn test_load_nonexistent_plugin_fails() {
        let mut manager = PluginManager::new();
        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_load_calls_on_load() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();

        // Verify on_load was called by checking loaded state
        assert_eq!(manager.get_state("test"), Some(PluginState::Loaded));
    }

    #[test]
    fn test_load_failure_sets_error_state() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test").with_fail_load()))
            .unwrap();
        let result = manager.load("test");
        assert!(result.is_err());
        assert_eq!(manager.get_state("test"), Some(PluginState::Error));
    }

    #[test]
    fn test_unload_plugin() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();
        manager.unload("test").unwrap();
        assert_eq!(manager.get_state("test"), Some(PluginState::Unloaded));
    }

    #[test]
    fn test_unload_nonexistent_plugin_fails() {
        let mut manager = PluginManager::new();
        let result = manager.unload("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_unregister_plugin() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        assert_eq!(manager.count(), 1);
        manager.unregister("test").unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_unregister_unloads_first() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();
        manager.unregister("test").unwrap();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_unregister_nonexistent_plugin_fails() {
        let mut manager = PluginManager::new();
        let result = manager.unregister("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    // ========================================================================
    // Event Broadcasting Tests
    // ========================================================================

    #[test]
    fn test_broadcast_event_to_loaded_plugins() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();
        manager.load("test1").unwrap();
        manager.load("test2").unwrap();

        let event = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };

        let handlers = manager.broadcast_event(&event).unwrap();
        assert_eq!(handlers.len(), 2);
    }

    #[test]
    fn test_broadcast_event_skips_unloaded_plugins() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();
        manager.load("test1").unwrap();
        // test2 not loaded

        let event = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };

        let handlers = manager.broadcast_event(&event).unwrap();
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0], "test1");
    }

    #[test]
    fn test_broadcast_event_adds_to_log() {
        let mut manager = PluginManager::new();
        let event = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };

        manager.broadcast_event(&event).unwrap();
        assert_eq!(manager.event_log().len(), 1);
    }

    #[test]
    fn test_event_log_max_size() {
        let mut manager = PluginManager::new();

        // Broadcast 150 events (more than max_log_size of 100)
        for i in 0..150 {
            let event = PluginEvent::Custom {
                event_type: "test".to_string(),
                data: format!("event {}", i),
            };
            manager.broadcast_event(&event).unwrap();
        }

        assert_eq!(manager.event_log().len(), 100);
    }

    #[test]
    fn test_clear_event_log() {
        let mut manager = PluginManager::new();
        let event = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };

        manager.broadcast_event(&event).unwrap();
        assert_eq!(manager.event_log().len(), 1);

        manager.clear_event_log();
        assert_eq!(manager.event_log().len(), 0);
    }

    #[test]
    fn test_broadcast_event_failure_propagates() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test").with_fail_event()))
            .unwrap();
        manager.load("test").unwrap();

        let event = PluginEvent::MessageReceived {
            content: "hello".to_string(),
            sender: "alice".to_string(),
        };

        let result = manager.broadcast_event(&event);
        assert!(matches!(result, Err(PluginError::EventHandlingFailed(_))));
    }

    // ========================================================================
    // Command Execution Tests
    // ========================================================================

    #[test]
    fn test_execute_command_handled() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();

        let result = manager.execute_command("test", &[]).unwrap();
        assert_eq!(result, Some("Executed with 0 args".to_string()));
    }

    #[test]
    fn test_execute_command_not_handled() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();

        let result = manager.execute_command("unknown", &[]).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_execute_command_with_args() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();

        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let result = manager.execute_command("test", &args).unwrap();
        assert_eq!(result, Some("Executed with 2 args".to_string()));
    }

    #[test]
    fn test_all_commands() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();
        manager.load("test1").unwrap();
        manager.load("test2").unwrap();

        let commands = manager.all_commands();
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn test_all_commands_only_from_loaded() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();
        manager.load("test1").unwrap();
        // test2 not loaded

        let commands = manager.all_commands();
        assert_eq!(commands.len(), 1);
    }

    // ========================================================================
    // Plugin State Tests
    // ========================================================================

    #[test]
    fn test_get_state_loaded() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();
        assert_eq!(manager.get_state("test"), Some(PluginState::Loaded));
    }

    #[test]
    fn test_get_state_unloaded() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        assert_eq!(manager.get_state("test"), Some(PluginState::Unloaded));
    }

    #[test]
    fn test_get_state_nonexistent() {
        let manager = PluginManager::new();
        assert_eq!(manager.get_state("nonexistent"), None);
    }

    #[test]
    fn test_loaded_plugins_list() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test3")))
            .unwrap();
        manager.load("test1").unwrap();
        manager.load("test3").unwrap();

        let loaded = manager.loaded_plugins();
        assert_eq!(loaded.len(), 2);
        assert!(loaded.contains(&"test1".to_string()));
        assert!(loaded.contains(&"test3".to_string()));
    }

    // ========================================================================
    // Plugin Enable/Disable Tests
    // ========================================================================

    #[test]
    fn test_enable_plugin() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();
        manager.disable("test").unwrap();
        manager.enable("test").unwrap();

        // Verify by checking if it handles events
        let event = PluginEvent::Custom {
            event_type: "test".to_string(),
            data: "data".to_string(),
        };
        let handlers = manager.broadcast_event(&event).unwrap();
        assert_eq!(handlers.len(), 1);
    }

    #[test]
    fn test_disable_plugin() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test"))).unwrap();
        manager.load("test").unwrap();
        manager.disable("test").unwrap();

        // Verify by checking if it handles events
        let event = PluginEvent::Custom {
            event_type: "test".to_string(),
            data: "data".to_string(),
        };
        let handlers = manager.broadcast_event(&event).unwrap();
        assert_eq!(handlers.len(), 0); // Disabled, shouldn't handle
    }

    #[test]
    fn test_enable_nonexistent_plugin_fails() {
        let mut manager = PluginManager::new();
        let result = manager.enable("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    #[test]
    fn test_disable_nonexistent_plugin_fails() {
        let mut manager = PluginManager::new();
        let result = manager.disable("nonexistent");
        assert!(matches!(result, Err(PluginError::NotFound(_))));
    }

    // ========================================================================
    // Plugin List Tests
    // ========================================================================

    #[test]
    fn test_list_plugins() {
        let mut manager = PluginManager::new();
        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();

        let list = manager.list_plugins();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|m| m.name == "test1"));
        assert!(list.iter().any(|m| m.name == "test2"));
    }

    #[test]
    fn test_count() {
        let mut manager = PluginManager::new();
        assert_eq!(manager.count(), 0);

        manager
            .register(Box::new(TestPlugin::new("test1")))
            .unwrap();
        assert_eq!(manager.count(), 1);

        manager
            .register(Box::new(TestPlugin::new("test2")))
            .unwrap();
        assert_eq!(manager.count(), 2);

        manager.unregister("test1").unwrap();
        assert_eq!(manager.count(), 1);
    }
}
