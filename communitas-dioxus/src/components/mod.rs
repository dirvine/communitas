//! Dioxus UI components for Communitas.

pub mod announcer;
#[allow(dead_code)]
pub mod app_shell;
pub mod app_sidebar;
#[allow(dead_code)]
pub mod auth_v2;
#[allow(dead_code)]
pub mod canvas;
pub mod channel_chat;
pub mod channel_sidebar;
#[allow(dead_code)]
mod composer;
#[allow(dead_code)]
pub mod confirm_dialog;
pub mod create_channel_modal;
pub mod create_space_modal;
#[allow(dead_code)]
pub mod daemon_status;
pub mod dashboard;
pub mod detail_panel;
pub mod dm_view;
#[allow(dead_code)]
pub mod drive;
#[allow(dead_code)]
pub mod entity_view;
#[allow(dead_code)]
pub mod error_boundary;
pub mod feed_view;
pub mod files_view;
#[allow(dead_code)]
pub mod filter_chips;
#[allow(dead_code, unused_imports)]
pub mod kanban;
#[allow(dead_code)]
pub mod layout;
pub mod local_x0x_profile_view;
#[allow(dead_code)]
mod message_list;
#[allow(dead_code)]
pub mod messaging_v2;
pub mod network_view;
#[allow(dead_code)]
pub mod offline;
#[allow(dead_code)]
pub mod pagination;
pub mod people_view;
#[allow(dead_code)]
mod presence_badge;
#[allow(dead_code)]
mod search_bar;
#[allow(dead_code)]
pub mod settings;
pub mod settings_view;
#[allow(dead_code)]
pub mod sidebar;
#[allow(dead_code)]
pub mod skeleton;
pub mod space_view;
pub mod status_bar;
pub mod swarm_view;
#[allow(dead_code)]
mod thread_list;
pub mod thread_panel;
#[allow(dead_code)]
pub mod virtual_list;
pub mod web_view;
pub mod wiki_view;

// Re-export canvas components (will be used when canvas UI is integrated into routes)
#[allow(unused_imports)]
pub use canvas::{
    CanvasToolbar, CanvasView, CollaboratorList, HistoryIndicator, HistoryScrubber, LayerPanel,
    OfflineIndicator, RemoteCursors, SyncStatusBadge,
};
#[allow(unused_imports)]
pub use composer::MessageComposer;
#[allow(unused_imports)]
pub use drive::DriveBrowser;
// Re-export main kanban components for external use
#[allow(unused_imports)]
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
#[allow(unused_imports)]
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

// Re-export pagination components for list navigation
#[allow(unused_imports)]
pub use pagination::{LoadMore, Pagination};

// Re-export filter chips for quick filtering
#[allow(unused_imports)]
pub use filter_chips::{FilterChips, FilterOption};

// Re-export confirm dialog for user confirmations
#[allow(unused_imports)]
pub use confirm_dialog::ConfirmDialog;

// Re-export daemon status bar for x0xd connectivity
#[allow(unused_imports)]
pub use daemon_status::DaemonStatusBar;

// Re-export channel and thread components for space-based messaging
pub use channel_chat::ChannelChatView;
pub use channel_sidebar::ChannelSidebar;
pub use create_channel_modal::CreateChannelModal;
pub use create_space_modal::{CreateSpaceModal, SpaceModalTab};
pub use detail_panel::{DetailContent, DetailPanel};
pub use thread_panel::ThreadPanel;

// Re-export new Deep Space components
pub use app_sidebar::{AppSidebar, ContactEntry, GroupEntry};
pub use dashboard::Dashboard;
pub use dm_view::DmView;
#[allow(unused_imports)]
pub use feed_view::FeedView;
#[allow(unused_imports)]
pub use files_view::FilesView;
pub use local_x0x_profile_view::LocalX0xProfileView;
pub use network_view::NetworkView;
pub use people_view::PeopleView;
pub use settings_view::SettingsView;
pub use space_view::SpaceView;
pub use status_bar::StatusBar;
#[allow(unused_imports)]
pub use swarm_view::SwarmView;
#[allow(unused_imports)]
pub use web_view::WebView;
#[allow(unused_imports)]
pub use wiki_view::WikiView;
