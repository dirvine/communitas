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
pub use backend::{FocusChange, FocusManager};
pub use components::{
    Avatar, AvatarShape, AvatarSize, AvatarState, Column, ColumnWidth, FormInput, InputMode,
    ListItem, Message, MessageList, Modal, ModalSize, ModalType, SelectList, SplitLayout,
    StatusBar,
};
pub use messages::{ComponentId, Msg, UserEvent};
pub use model::Model;
