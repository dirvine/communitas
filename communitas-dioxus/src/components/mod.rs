// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dioxus UI components for Communitas.
//!
//! This is a binary crate's component library. Many components and their re-exports
//! are used via full paths (e.g. `crate::components::foo::Bar`) rather than through
//! the re-exports declared here. The module-level allow attributes prevent false
//! positives from the compiler's dead-code and unused-import analysis.
#![allow(dead_code, unused_imports)]

pub mod announcer;
pub mod app_shell;
pub mod app_sidebar;
pub mod auth_v2;
pub mod board_view;
pub mod canvas;
pub mod channel_chat;
pub mod channel_sidebar;
mod composer;
pub mod confirm_dialog;
pub mod constitution_view;
pub mod create_channel_modal;
pub mod create_space_modal;
pub mod daemon_status;
pub mod dashboard;
pub mod detail_panel;
pub mod dm_view;
pub mod drive;
pub mod emoji_data;
pub mod emoji_picker;
pub mod entity_view;
pub mod error_boundary;
pub mod feed_view;
pub mod files_view;
pub mod filter_chips;
pub mod kanban;
pub mod layout;
pub mod local_x0x_profile_view;
pub mod markdown;
pub mod mention;
mod message_list;
pub mod messaging_v2;
pub mod network_view;
pub mod offline;
pub mod pagination;
pub mod people_view;
mod presence_badge;
mod search_bar;
pub mod settings;
pub mod settings_view;
pub mod sidebar;
pub mod skeleton;
pub mod space_view;
pub mod status_bar;
pub mod swarm_view;
mod thread_list;
pub mod thread_panel;
pub mod virtual_list;
pub mod web_view;
pub mod wiki_view;

// Toast notification system
pub mod toast_system;
pub use toast_system::{Toast, ToastContainer, ToastKind, ToastManager, use_toast};

// Re-export canvas components
pub use canvas::{
    CanvasToolbar, CanvasView, CollaboratorList, HistoryIndicator, HistoryScrubber, LayerPanel,
    OfflineIndicator, RemoteCursors, SyncStatusBadge,
};
pub use composer::MessageComposer;
pub use drive::DriveBrowser;
pub use message_list::MessageList;

// Re-export presence components
pub use presence_badge::{
    InCallBadge, InCallDot, PresenceBadge, PresenceDot, PresenceOrCallDot, PresenceWithCallBadge,
    PresentingBadge, PresentingDot,
};

// Re-export search components
pub use search_bar::{SearchBar, SearchResultSelection};
pub use thread_list::ThreadListSidebar;
pub use thread_list::TypingIndicator;

// Re-export skeleton components for loading states
pub use skeleton::{
    SkeletonCard, SkeletonCircle, SkeletonGrid, SkeletonLine, SkeletonList, SkeletonTable,
    SkeletonText,
};

// Re-export error boundary components for consistent error handling
pub use error_boundary::{
    ErrorBanner, ErrorCard, ErrorPage, NetworkError, WarningBanner, user_friendly_error,
};

// Re-export announcer components for screen reader accessibility
pub use announcer::{
    AnnouncementMode, Announcer, AnnouncerContext, announce_action, announce_assertive,
    announce_count, announce_error, announce_loaded, announce_loading, announce_navigation,
    announce_polite, announce_success, use_announcer,
};

// Re-export layout components for responsive layouts
pub use layout::{
    AspectRatio, Breakpoint, Center, Container, Direction, Divider, Grid, Row, Spacer, Stack,
};

// Re-export global offline state components (distinct from canvas-specific OfflineIndicator)
pub use offline::{
    ConflictBanner, ConflictBannerVariant, ConnectionBadge, ConnectionState, OfflineBanner,
    SyncState, SyncStatusIndicator, Toast as OfflineToast, ToastContainer as OfflineToastContainer,
    ToastNotification, ToastVariant, use_connection_state,
};

// Re-export settings components for application preferences
pub use settings::UpdateAvailableModal;
pub use settings::UpdateBadgeStatus;
pub use settings::UpdateCard;
pub use settings::UpdateProgressBar;
pub use settings::UpdateStatusBadge;

// Re-export enhanced auth components (v2 - Digital Forest Sanctuary theme)
pub use auth_v2::{
    AuthBackground, AuthLayoutV2, ErrorBanner as ErrorBannerV2, FormField, FormSelect,
    FormTextarea, Logo, PasswordStrength, PrimaryButton, SecondaryButton, TextLink,
};

// Re-export app shell components for main layout
pub use app_shell::{
    AppShell, ContactNavItem, EntityNavItem, ExpandableEntityNavItem, ProfileHeader,
    QuickActionButton, SidebarSearch, SidebarSection,
};

// Re-export entity view components
pub use entity_view::{
    EmptyState, EntityDetailView, EntityHeader, EntitySkeleton, EntityTab, EntityTabBar,
    HeaderAction,
};

// Re-export enhanced messaging components (v2)
pub use messaging_v2::{
    ChatView, DateSeparator, MessageBubble, MessageComposerV2, MessageDisplay,
    MessageListContainer, NewMessageIndicator, ReactionChip, ReactionDisplay, TypingIndicatorV2,
};

// Re-export markdown renderer for message display
pub use markdown::MarkdownContent;

// Re-export @mention autocomplete components
pub use mention::{
    MentionAutocomplete, MentionCandidate, filter_candidates as filter_mention_candidates,
};

// Re-export sidebar components for main app layout
pub use sidebar::{ContactListSection, EntityListSection, filter_contacts, filter_entities};

// Re-export virtual list component for efficient large list rendering
pub use virtual_list::VirtualList;

// Re-export pagination components for list navigation
pub use pagination::{LoadMore, Pagination};

// Re-export filter chips for quick filtering
pub use filter_chips::{FilterChips, FilterOption};

// Re-export confirm dialog for user confirmations
pub use confirm_dialog::ConfirmDialog;

// Re-export daemon status bar for x0xd connectivity
pub use daemon_status::DaemonStatusBar;

pub mod onboarding_gate;
pub use onboarding_gate::OnboardingGate;

// Re-export channel and thread components for space-based messaging
pub use channel_chat::ChannelChatView;
pub use channel_sidebar::ChannelSidebar;
pub use create_channel_modal::CreateChannelModal;
pub use create_space_modal::{CreateSpaceModal, SpaceModalTab};
pub use detail_panel::{DetailContent, DetailPanel};
pub use thread_panel::ThreadPanel;

// Re-export new Deep Space components
pub use app_sidebar::{AppSidebar, ContactEntry, GroupEntry};
pub use constitution_view::ConstitutionView;
pub use dashboard::Dashboard;
pub use dm_view::DmView;
pub use feed_view::FeedView;
pub use files_view::FilesView;
pub use local_x0x_profile_view::LocalX0xProfileView;
pub use network_view::NetworkView;
pub use people_view::PeopleView;
pub use settings_view::SettingsView;
pub use space_view::SpaceView;
pub use status_bar::StatusBar;
pub use swarm_view::SwarmView;
pub use web_view::WebView;
pub use wiki_view::WikiView;
