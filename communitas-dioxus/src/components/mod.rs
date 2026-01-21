//! Dioxus UI components for Communitas.

pub mod call;
pub mod canvas;
mod composer;
pub mod drive;
mod identity_switcher;
pub mod kanban;
mod message_list;
mod presence_badge;
mod thread_list;

// Re-export call components (will be used when call UI is integrated into routes)
#[allow(unused_imports)]
pub use call::{CallButton, CallLobby, DeviceSelector, MediaErrorBanner, ParticipantTile};
// Re-export canvas components (will be used when canvas UI is integrated into routes)
#[allow(unused_imports)]
pub use canvas::{
    CanvasToolbar, CanvasView, CollaboratorList, HistoryIndicator, HistoryScrubber, LayerPanel,
    OfflineIndicator, RemoteCursors, SyncStatusBadge,
};
pub use composer::MessageComposer;
pub use drive::DriveBrowser;
pub use identity_switcher::IdentitySwitcher;
// Re-export main kanban components for external use
// Note: BoardListPage and BoardView are used in main.rs routes
pub use message_list::MessageList;
pub use presence_badge::{PresenceBadge, PresenceDot};
pub use thread_list::ThreadListSidebar;
