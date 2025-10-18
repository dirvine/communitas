//! Interactive UI components for Communitas TUI
//!
//! All components follow the tui-realm Component pattern with:
//! - MockComponent trait for rendering and state management
//! - Component trait for event handling and message generation

pub mod avatar;
pub mod form_input;
pub mod message_list;
pub mod modal;
pub mod select_list;
pub mod split_layout;
pub mod status_bar;

pub use avatar::{Avatar, AvatarShape, AvatarSize, AvatarState};
pub use form_input::{FormInput, InputMode};
pub use message_list::{Message, MessageList};
pub use modal::{Modal, ModalSize, ModalType};
pub use select_list::{ListItem, SelectList};
pub use split_layout::{Column, ColumnWidth, SplitLayout};
pub use status_bar::StatusBar;
