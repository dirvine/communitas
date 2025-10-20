//! Interactive UI components for Communitas TUI
//!
//! All components follow the tui-realm Component pattern with:
//! - MockComponent trait for rendering and state management
//! - Component trait for event handling and message generation

// Core UI components
pub mod avatar;
pub mod form_input;
pub mod message_list;
pub mod modal;
pub mod select_list;
pub mod split_layout;
pub mod status_bar;
pub mod tabs;

// Advanced components
pub mod accessibility;
pub mod animation;
pub mod calendar;
pub mod command_palette;
pub mod context_menu;
pub mod data_vis;
pub mod error_recovery;
pub mod mouse_handler;
pub mod performance;
pub mod plugin_system;
pub mod resizable_split;
pub mod theme;
pub mod tree_view;

// Core UI exports
pub use avatar::{Avatar, AvatarShape, AvatarSize, AvatarState};
pub use form_input::{FormInput, InputMode};
pub use message_list::{Message, MessageList};
pub use modal::{Modal, ModalSize, ModalType};
pub use select_list::{ListItem, SelectList};
pub use split_layout::{Column, ColumnWidth, SplitLayout};
pub use status_bar::StatusBar;
pub use tabs::{ModernTabs, TabConfig};

// Advanced component exports
pub use accessibility::{
    AccessibilityManager, AccessibilitySetting, AccessibilitySettings, Announcement,
    AnnouncementPriority, FocusIndicator, FocusTracking,
};
pub use animation::{
    Animation, AnimationState, AnimationType, AnimationValue, Axis, EasingFunction,
};
pub use calendar::{
    CalendarConfig, CalendarEvent, CalendarView, EventCategory, EventImportance, ModernCalendar,
};
pub use command_palette::{Command, CommandPalette};
pub use context_menu::{ContextMenu, MenuAction, MenuContext, MenuItem};
pub use data_vis::{
    ColorScheme, DataPoint, DataVisOptions, ModernChart, ModernSparkline, NetworkMetrics,
    SystemResources, TimeSeriesData,
};
pub use error_recovery::{ErrorCategory, ErrorEntry, ErrorRecovery, ErrorSeverity, ErrorStats};
pub use mouse_handler::{
    ComponentArea, DoubleClickDetector, DragState, EnhancedMouseEvent, HoverState, ScrollState,
    classify_mouse_event,
};
pub use performance::{PerformanceMetrics, PerformanceMonitor};
pub use plugin_system::{
    Plugin, PluginCommand, PluginError, PluginEvent, PluginManager, PluginMetadata, PluginResult,
    PluginState,
};
pub use resizable_split::{Orientation, ResizableSplit};
pub use theme::{ColorPalette, ComponentStyles, ThemeConfig, ThemeManager, ThemeMode};
pub use tree_view::{TreeNode, TreeView};
