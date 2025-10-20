//! Communitas TUI Library
//!
//! This library provides the core functionality for the Communitas Terminal UI.
//! It exposes modules for testing and reusability.

pub mod backend;
pub mod components;
pub mod messages;
pub mod model;
pub mod state;
pub mod ui;

// Re-export commonly used types
pub use components::{
    ComponentArea, ContextMenu, DoubleClickDetector, DragState, EnhancedMouseEvent, HoverState,
    MenuAction, MenuContext, ScrollState, classify_mouse_event,
};
pub use messages::{ComponentId, Msg, UserEvent};
pub use model::Model;
