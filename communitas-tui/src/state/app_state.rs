#![allow(dead_code)]

use super::{AnimationManager, EntityData, Navigation, NetworkState};
use crate::components::{
    AccessibilityManager, CommandPalette, ContextMenu, DoubleClickDetector, DragState,
    ErrorRecovery, HoverState, Orientation, PerformanceMonitor, PluginManager, ResizableSplit,
    ScrollState, ThemeManager, TreeNode, TreeView,
};

/// Global application state
#[derive(Debug)]
pub struct AppState {
    /// Current navigation state (views, focus)
    pub navigation: Navigation,

    /// Network connection state
    pub network: NetworkState,

    /// Current identity (four-word address)
    pub identity: Option<String>,

    /// Display name for the user
    pub display_name: Option<String>,

    /// Entity data (channels, projects, etc.)
    pub entities: EntityData,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Status message to display
    pub status_message: Option<String>,

    /// Input buffer for text entry
    pub input_buffer: String,

    /// Whether input mode is active
    pub input_active: bool,

    // === Mouse Interaction State ===
    /// Hover state for tooltips and visual feedback
    pub hover_state: HoverState,

    /// Drag and drop state
    pub drag_state: DragState,

    /// Double-click detection
    pub double_click_detector: DoubleClickDetector,

    /// Scroll state for scrollable content
    pub scroll_state: ScrollState,

    /// Context menu (right-click menu)
    pub context_menu: ContextMenu,

    // === Advanced Components (Phase 2) ===
    /// Performance monitoring overlay (F12 to toggle)
    pub performance_monitor: PerformanceMonitor,

    /// Theme management and customization (Ctrl+T to toggle preview)
    pub theme_manager: ThemeManager,

    /// Accessibility features (Ctrl+S for screen reader, Ctrl+H for high contrast)
    pub accessibility_manager: AccessibilityManager,

    /// Error recovery and user-friendly error messages
    pub error_recovery: ErrorRecovery,

    // === New Advanced Components (Phases 3-7) ===
    /// Resizable panel splits for dashboard layout
    pub resizable_split: ResizableSplit,

    /// Tree view for hierarchical data (organizations, projects, files)
    pub tree_view: TreeView<String>,

    /// Command palette for global command search (Ctrl+K)
    pub command_palette: CommandPalette,

    /// Animation manager for smooth transitions
    pub animation_manager: AnimationManager,

    /// Plugin system for extensibility
    pub plugin_manager: PluginManager,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            navigation: Navigation::new(),
            network: NetworkState::new(),
            identity: None,
            display_name: None,
            entities: EntityData::new(),
            should_quit: false,
            status_message: None,
            input_buffer: String::new(),
            input_active: false,

            // Mouse interaction state
            hover_state: HoverState::new(),
            drag_state: DragState::new(),
            double_click_detector: DoubleClickDetector::new(500, 2), // 500ms, 2px tolerance
            scroll_state: ScrollState::new(100, 20),                 // Will be updated dynamically
            context_menu: ContextMenu::new(),

            // Advanced components (Phase 2)
            performance_monitor: PerformanceMonitor::new(),
            theme_manager: ThemeManager::new(),
            accessibility_manager: AccessibilityManager::new(),
            error_recovery: ErrorRecovery::new(),

            // New advanced components (Phases 3-7)
            resizable_split: ResizableSplit::new()
                .with_orientation(Orientation::Vertical)
                .with_position(30) // 30% left panel
                .with_min_size(20) // Min 20%
                .with_max_size(50), // Max 50%

            tree_view: TreeView::new(TreeNode::new("root", "Entities", String::new())),

            command_palette: create_initial_command_palette(),

            animation_manager: AnimationManager::new(),

            plugin_manager: PluginManager::new(),
        }
    }

    /// Set identity and display name
    pub fn set_identity(&mut self, identity: String, display_name: String) {
        self.identity = Some(identity);
        self.display_name = Some(display_name);
    }

    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        use crate::components::Animation;
        use std::time::Duration;

        let msg = message.into();

        // Add notification pulse animation (scale from 90% to 110%)
        self.animation_manager.add(
            "notification_pulse",
            Animation::pulse(90, 110, Duration::from_millis(500)),
        );

        // Add error shake animation for error/failure messages
        if msg.starts_with("Error:") || msg.starts_with("Failed:") {
            self.animation_manager.add(
                "error_shake",
                Animation::shake(5, Duration::from_millis(300)), // 5 pixel amplitude
            );
        }

        self.status_message = Some(msg);
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Activate input mode
    pub fn activate_input(&mut self) {
        self.input_active = true;
        self.input_buffer.clear();
    }

    /// Deactivate input mode
    pub fn deactivate_input(&mut self) {
        self.input_active = false;
        self.input_buffer.clear();
    }

    /// Get input buffer and clear it
    pub fn take_input(&mut self) -> String {
        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        input
    }

    /// Append character to input buffer
    pub fn push_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    /// Delete last character from input buffer
    pub fn pop_input_char(&mut self) {
        self.input_buffer.pop();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// COMMAND PALETTE INITIALIZATION
// ============================================================================

/// Create CommandPalette with initial commands
fn create_initial_command_palette() -> CommandPalette {
    use crate::components::Command;

    let commands = vec![
        // Navigation Commands
        Command::new(
            "nav.orgs",
            "Go to Organizations",
            "Navigate to organizations view",
            "Navigation",
        )
        .with_shortcut("Ctrl+1"),
        Command::new(
            "nav.projects",
            "Go to Projects",
            "Navigate to projects view",
            "Navigation",
        )
        .with_shortcut("Ctrl+2"),
        Command::new(
            "nav.groups",
            "Go to Groups",
            "Navigate to groups view",
            "Navigation",
        )
        .with_shortcut("Ctrl+3"),
        Command::new(
            "nav.contacts",
            "Go to Contacts",
            "Navigate to contacts view",
            "Navigation",
        )
        .with_shortcut("Ctrl+4"),
        Command::new(
            "nav.dashboard",
            "Go to Dashboard",
            "Navigate to dashboard view",
            "Navigation",
        )
        .with_shortcut("Ctrl+0"),
        // Action Commands
        Command::new(
            "create.org",
            "Create Organization",
            "Create a new organization",
            "Actions",
        )
        .with_shortcut("Ctrl+Shift+O"),
        Command::new(
            "create.project",
            "Create Project",
            "Create a new project",
            "Actions",
        )
        .with_shortcut("Ctrl+Shift+P"),
        Command::new(
            "create.group",
            "Create Group",
            "Create a new group",
            "Actions",
        )
        .with_shortcut("Ctrl+Shift+G"),
        Command::new(
            "create.contact",
            "Add Contact",
            "Add a new contact",
            "Actions",
        )
        .with_shortcut("Ctrl+Shift+C"),
        // Settings Commands
        Command::new(
            "toggle.theme",
            "Toggle Theme",
            "Switch between light and dark theme",
            "Settings",
        ),
        Command::new(
            "toggle.perf",
            "Toggle Performance Monitor",
            "Show/hide performance stats",
            "Settings",
        ),
        Command::new(
            "settings.open",
            "Open Settings",
            "Open application settings",
            "Settings",
        ),
        // Network Commands
        Command::new(
            "net.connect",
            "Connect to Network",
            "Connect to P2P network",
            "Network",
        ),
        Command::new(
            "net.disconnect",
            "Disconnect from Network",
            "Disconnect from P2P network",
            "Network",
        ),
        Command::new(
            "net.status",
            "Show Network Status",
            "Display network connection status",
            "Network",
        ),
        // View Commands
        Command::new(
            "view.focus_sidebar",
            "Focus Sidebar",
            "Move focus to sidebar panel",
            "View",
        )
        .with_shortcut("Ctrl+B"),
        Command::new(
            "view.focus_main",
            "Focus Main Panel",
            "Move focus to main panel",
            "View",
        )
        .with_shortcut("Ctrl+M"),
        Command::new(
            "view.split_reset",
            "Reset Panel Split",
            "Reset panel split to default",
            "View",
        )
        .with_shortcut("Ctrl+\\"),
    ];

    CommandPalette::with_commands(commands)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let state = AppState::new();
        assert!(!state.should_quit);
        assert!(state.identity.is_none());
        assert!(state.status_message.is_none());
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert!(!state.should_quit);
    }

    // === New Components Integration Tests ===

    #[test]
    fn test_resizable_split_initialized() {
        let state = AppState::new();
        // Should be initialized with 30% position
        assert_eq!(state.resizable_split.position(), 30);
    }

    #[test]
    fn test_tree_view_initialized() {
        let state = AppState::new();
        // TreeView should start with root node
        assert_eq!(state.tree_view.root().label, "Entities");
    }

    #[test]
    fn test_command_palette_initialized() {
        let state = AppState::new();
        // Command palette should start hidden
        assert!(!state.command_palette.is_visible());
        // Should have commands available (created in create_initial_command_palette)
        assert!(!state.command_palette.commands().is_empty());
    }

    #[test]
    fn test_animation_manager_initialized() {
        let state = AppState::new();
        // Should start with no active animations
        assert_eq!(state.animation_manager.count(), 0);
    }

    #[test]
    fn test_plugin_manager_initialized() {
        let state = AppState::new();
        // Should start with no plugins
        assert_eq!(state.plugin_manager.count(), 0);
    }

    #[test]
    fn test_resizable_split_position() {
        let state = AppState::new();
        // Verify initial position
        assert_eq!(state.resizable_split.position(), 30);
    }

    #[test]
    fn test_command_palette_can_be_toggled() {
        let mut state = AppState::new();
        assert!(!state.command_palette.is_visible());

        state.command_palette.toggle();
        assert!(state.command_palette.is_visible());

        state.command_palette.toggle();
        assert!(!state.command_palette.is_visible());
    }

    #[test]
    fn test_animation_manager_can_add_animations() {
        let mut state = AppState::new();
        use crate::components::Animation;
        use std::time::Duration;

        state
            .animation_manager
            .add("test", Animation::fade_in(Duration::from_millis(100)));
        assert_eq!(state.animation_manager.count(), 1);
        assert!(state.animation_manager.has("test"));
    }

    #[test]
    fn test_plugin_manager_can_register_plugins() {
        let state = AppState::new();
        // This will be tested properly when we have actual plugins
        // For now, just verify it's accessible
        assert_eq!(state.plugin_manager.count(), 0);
    }

    // === Existing Functionality Still Works ===

    #[test]
    fn test_set_identity() {
        let mut state = AppState::new();
        state.set_identity("ocean-forest-moon-star".to_string(), "Alice".to_string());
        assert_eq!(state.identity, Some("ocean-forest-moon-star".to_string()));
        assert_eq!(state.display_name, Some("Alice".to_string()));
    }

    #[test]
    fn test_set_status() {
        let mut state = AppState::new();
        state.set_status("Test message");
        assert_eq!(state.status_message, Some("Test message".to_string()));
    }

    #[test]
    fn test_clear_status() {
        let mut state = AppState::new();
        state.set_status("Test");
        state.clear_status();
        assert!(state.status_message.is_none());
    }

    #[test]
    fn test_input_activation() {
        let mut state = AppState::new();
        assert!(!state.input_active);

        state.activate_input();
        assert!(state.input_active);
        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn test_input_deactivation() {
        let mut state = AppState::new();
        state.activate_input();
        state.push_input_char('a');
        state.deactivate_input();
        assert!(!state.input_active);
        assert_eq!(state.input_buffer, "");
    }

    #[test]
    fn test_take_input() {
        let mut state = AppState::new();
        state.activate_input();
        state.push_input_char('h');
        state.push_input_char('i');
        let input = state.take_input();
        assert_eq!(input, "hi");
        assert_eq!(state.input_buffer, "");
    }
}
