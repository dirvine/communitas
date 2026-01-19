//! Shared UI-facing models for Communitas front-ends.

pub mod call;
pub mod canvas;
pub mod drive;
pub mod kanban;
pub mod messaging;

pub use call::{
    CallInfo, CallSettings, CallSnapshot, CallState, DeviceType, MediaDevice, MediaError,
    MediaErrorKind, Participant,
};
pub use canvas::{
    CanvasElement, CanvasInfo, CanvasSnapshot, CanvasState, ElementType, HistoryActionType,
    HistoryEntry, Layer, OfflineOperation, OfflineStatus, OperationType, Point, RemoteCursor,
    ToolType, Transform,
};
pub use drive::{
    DirectoryEntry, DiskInfo, DiskType, DownloadProgress, DownloadState, FileMetadata, FilePreview,
    QuotaInfo, UploadProgress, UploadState,
};
pub use kanban::{
    ActivityEntry, AttachmentView, BoardSettings, BoardSummary, BoardView, CardDetail, CardState,
    CardView, ChecklistProgress, ColumnView, CommentView, StepView, TagView,
};
pub use messaging::{ContactWithPresence, Message, MessageReaction, PresenceStatus, ThreadSummary};

/// Small helper used by the Dioxus prototype to display generated identity words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleWords {
    words: String,
}

impl SampleWords {
    /// Create a new sample words value, accepting anything convertible into `String`.
    pub fn new(words: impl Into<String>) -> Self {
        Self {
            words: words.into(),
        }
    }

    /// Returns the raw words payload for display.
    pub fn as_str(&self) -> &str {
        self.words.as_str()
    }
}

/// Snapshot of identity metadata shown in nav bars and headers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnifiedIdentity {
    pub display_name: String,
    pub four_words: String,
}

/// Logical classification for organization-style entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrganizationCategory {
    Organization,
    Community,
}

/// Entity type enumeration used by UI layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifiedEntityType {
    Organization,
    Project,
    Group,
    Channel,
    Person,
}

/// View-model for entities shown across navigation, home, and detail screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedEntity {
    pub id: String,
    pub entity_type: UnifiedEntityType,
    pub name: String,
    pub description: String,
    pub member_count: u64,
    pub parent_id: Option<String>,
    pub category: Option<OrganizationCategory>,
}

impl UnifiedEntity {
    pub fn is_personal_group(&self) -> bool {
        matches!(self.entity_type, UnifiedEntityType::Group) && self.parent_id.is_none()
    }
}

/// Simplified contact view rendered in navigation and contact lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnifiedContact {
    pub id: String,
    pub display_name: String,
    pub status: String,
}
