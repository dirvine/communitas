//! Canvas UI components for collaborative drawing and visual surfaces.

mod canvas_view;
mod history_scrubber;
mod layer_panel;
mod offline_indicator;
mod remote_cursors;
mod toolbar;

pub use canvas_view::CanvasView;
pub use history_scrubber::{HistoryIndicator, HistoryScrubber};
pub use layer_panel::LayerPanel;
pub use offline_indicator::{OfflineIndicator, SyncStatusBadge};
pub use remote_cursors::{CollaboratorList, RemoteCursors};
pub use toolbar::CanvasToolbar;
