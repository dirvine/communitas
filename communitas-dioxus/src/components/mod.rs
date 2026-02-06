//! Dioxus UI components for Communitas.

pub mod announcer;
pub mod app_shell;
pub mod auth_v2;
pub mod call;
pub mod canvas;
mod composer;
pub mod drive;
pub mod entity_view;
pub mod error_boundary;
pub mod kanban;
pub mod layout;
mod message_list;
pub mod messaging_v2;
pub mod offline;
mod presence_badge;
mod search_bar;
pub mod settings;
pub mod sidebar;
pub mod skeleton;
mod thread_list;
pub mod virtual_list;

// Re-export call components (will be used when call UI is integrated into routes)
#[allow(unused_imports)]
pub use call::{
    CallButton, CallControls, CallLobby, CallStatusBar, CallView, DeviceSelector,
    IncomingCallBanner, InlineCallControls, MediaErrorBanner, MediaErrorIndicator, MiniCallView,
    MissedCallBadge, MissedCallsPanel, ParticipantGrid, ParticipantTile, QualityDetailsPanel,
    QualityDot, QualityIndicator, ReactiveMissedCallBadge, RecordingControls, RecordingDot,
    RecordingIndicator,
};
// Re-export canvas components (will be used when canvas UI is integrated into routes)
#[allow(unused_imports)]
pub use canvas::{
    CanvasToolbar, CanvasView, CollaboratorList, HistoryIndicator, HistoryScrubber, LayerPanel,
    OfflineIndicator, RemoteCursors, SyncStatusBadge,
};
pub use composer::MessageComposer;
pub use drive::DriveBrowser;
// Re-export main kanban components for external use
// Note: BoardListPage and BoardView are used in main.rs routes
pub use message_list::MessageList;
// Re-export presence components (InCall variants will be used when presence UI is enhanced)
#[allow(unused_imports)]
pub use presence_badge::{
    InCallBadge, InCallDot, PresenceBadge, PresenceDot, PresenceOrCallDot, PresenceWithCallBadge,
    PresentingBadge, PresentingDot,
};
// Re-export search components (will be integrated into messaging UI)
#[allow(unused_imports)]
pub use search_bar::{SearchBar, SearchResultSelection};
pub use thread_list::ThreadListSidebar;
// Re-export TypingIndicator for external use (can be used in other message views)
#[allow(unused_imports)]
pub use thread_list::TypingIndicator;
// Re-export skeleton components for loading states (will be used when refactoring existing skeletons)
#[allow(unused_imports)]
pub use skeleton::{
    SkeletonCard, SkeletonCircle, SkeletonGrid, SkeletonLine, SkeletonList, SkeletonTable,
    SkeletonText,
};
// Re-export error boundary components for consistent error handling
#[allow(unused_imports)]
pub use error_boundary::{
    ErrorBanner, ErrorCard, ErrorPage, NetworkError, WarningBanner, user_friendly_error,
};
// Re-export announcer components for screen reader accessibility
#[allow(unused_imports)]
pub use announcer::{
    AnnouncementMode, Announcer, AnnouncerContext, announce_action, announce_assertive,
    announce_count, announce_error, announce_loaded, announce_loading, announce_navigation,
    announce_polite, announce_success, use_announcer,
};
// Re-export layout components for responsive layouts
#[allow(unused_imports)]
pub use layout::{
    AspectRatio, Breakpoint, Center, Container, Direction, Divider, Grid, Row, Spacer, Stack,
};
// Re-export global offline state components (distinct from canvas-specific OfflineIndicator)
#[allow(unused_imports)]
pub use offline::{
    ConflictBanner, ConflictBannerVariant, ConnectionBadge, ConnectionState, OfflineBanner,
    SyncState, SyncStatusIndicator, Toast, ToastContainer, ToastNotification, ToastVariant,
    use_connection_state,
};
// Re-export settings components for application preferences
// Note: These will be used when settings UI is integrated
#[allow(unused_imports)]
pub use settings::UpdateAvailableModal;
#[allow(unused_imports)]
pub use settings::UpdateBadgeStatus;
#[allow(unused_imports)]
pub use settings::UpdateCard;
#[allow(unused_imports)]
pub use settings::UpdateProgressBar;
#[allow(unused_imports)]
pub use settings::UpdateStatusBadge;
// Re-export enhanced auth components (v2 - Digital Forest Sanctuary theme)
#[allow(unused_imports)]
pub use auth_v2::{
    AuthBackground, AuthLayoutV2, ErrorBanner as ErrorBannerV2, FormField, FormSelect,
    FormTextarea, Logo, PasswordStrength, PrimaryButton, SecondaryButton, TextLink,
};

// Re-export app shell components for main layout
#[allow(unused_imports)]
pub use app_shell::{
    AppShell, ContactNavItem, EntityNavItem, ExpandableEntityNavItem, ProfileHeader,
    QuickActionButton, SidebarSearch, SidebarSection,
};

// Re-export entity view components
#[allow(unused_imports)]
pub use entity_view::{
    EmptyState, EntityDetailView, EntityHeader, EntitySkeleton, EntityTab, EntityTabBar,
    HeaderAction,
};

// Re-export enhanced messaging components (v2)
#[allow(unused_imports)]
pub use messaging_v2::{
    ChatView, DateSeparator, MessageBubble, MessageComposerV2, MessageDisplay,
    MessageListContainer, NewMessageIndicator, ReactionChip, ReactionDisplay, TypingIndicatorV2,
};

// Re-export sidebar components for main app layout
#[allow(unused_imports)]
pub use sidebar::{ContactListSection, EntityListSection, filter_contacts, filter_entities};

// Re-export virtual list component for efficient large list rendering
#[allow(unused_imports)]
pub use virtual_list::VirtualList;
