//! Call UI components for real-time voice/video communication.

mod call_button;
mod call_controls;
mod call_lobby;
mod call_notification;
mod call_view;
mod device_selector;
mod media_error_banner;
mod participant_tile;
mod quality_indicator;
mod recording_indicator;
mod screen_share_picker;

pub use call_button::CallButton;
pub use call_controls::{CallControls, InlineCallControls};
pub use call_lobby::CallLobby;
pub use call_notification::{
    IncomingCallBanner, MissedCallBadge, MissedCallsPanel, ReactiveMissedCallBadge,
};
pub use call_view::{CallStatusBar, CallView, MiniCallView};
pub use device_selector::DeviceSelector;
pub use media_error_banner::{MediaErrorBanner, MediaErrorIndicator};
pub use participant_tile::{ParticipantGrid, ParticipantTile};
pub use quality_indicator::{QualityDetailsPanel, QualityDot, QualityIndicator};
pub use recording_indicator::{RecordingControls, RecordingDot, RecordingIndicator};
// Public API - used externally from the call module
#[allow(unused_imports)]
pub use screen_share_picker::{ScreenShareButton, ScreenShareIndicator, ScreenSharePicker};
