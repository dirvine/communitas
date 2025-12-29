// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Message types for the Iced MVU architecture.
//!
//! All user interactions and async results flow through these messages.

use crate::error::AppError;
use crate::state::{
    ActiveView, CardPriority, ChatMessage, Contact, DetailTab, Entity, KanbanCard, PeerInfo,
    SidebarSection,
};
use iced::widget::pane_grid;

/// Root application message.
#[derive(Debug, Clone)]
pub enum Message {
    /// Authentication messages.
    Auth(AuthMessage),
    /// Navigation messages.
    Navigate(NavigationMessage),
    /// Sidebar messages.
    Sidebar(SidebarMessage),
    /// Chat/messaging messages.
    Chat(ChatMessageEvent),
    /// Contact messages.
    Contact(ContactMessage),
    /// Kanban messages.
    Kanban(KanbanMessage),
    /// Call/WebRTC messages.
    Call(CallMessage),
    /// Network messages.
    Network(NetworkMessage),
    /// Storage/drive messages.
    Storage(StorageMessage),
    /// Modal messages.
    Modal(ModalMessage),
    /// Theme changed.
    ThemeChanged(iced::Theme),
    /// Periodic tick (for updates/animations).
    Tick(std::time::Instant),
    /// Pane grid resized.
    PaneResized(pane_grid::ResizeEvent),
    /// Pane grid dragged.
    PaneDragged(pane_grid::DragEvent),
    /// Core event from backend.
    CoreEvent(CoreEvent),
    /// Error occurred.
    Error(AppError),
    /// No-op (for composability).
    Noop,
    /// Test action for automated testing (Cmd+T sends message, Cmd+1/2/3 switches views).
    #[cfg(feature = "demo")]
    TestAction(TestAction),
}

/// Test actions for automated testing (only in demo mode).
#[cfg(feature = "demo")]
#[derive(Debug, Clone)]
pub enum TestAction {
    /// Send a test message.
    SendTestMessage,
    /// Switch to home view.
    SwitchToHome,
    /// Switch to contact chat.
    SwitchToContactChat,
    /// Switch to channel chat.
    SwitchToChannelChat,
    /// Create a test organization.
    CreateOrganization,
    /// Create a test project.
    CreateProject,
    /// Create a test group.
    CreateGroup,
    /// Open network panel.
    OpenNetworkPanel,
    /// Toggle sidebar section (cycles through sections).
    ToggleSidebarSection,
}

/// Authentication messages.
#[derive(Debug, Clone)]
pub enum AuthMessage {
    /// Vault list loaded.
    VaultsLoaded(Vec<crate::state::VaultInfo>),
    /// Vault selected.
    VaultSelected(String),
    /// Password input changed.
    PasswordChanged(String),
    /// Login button pressed.
    LoginPressed,
    /// Login result.
    LoginResult(Result<LoginSuccess, String>),
    /// Switch to create identity mode.
    CreateIdentityPressed,
    /// Display name changed (for new identity).
    DisplayNameChanged(String),
    /// New password changed.
    NewPasswordChanged(String),
    /// Confirm password changed.
    ConfirmPasswordChanged(String),
    /// Create identity submitted.
    CreateIdentitySubmit,
    /// Identity created.
    IdentityCreated(Result<LoginSuccess, String>),
    /// Biometric auth pressed.
    BiometricPressed,
    /// Logout pressed.
    Logout,
    /// Cancel create identity.
    CancelCreate,
}

/// Successful login data.
#[derive(Debug, Clone)]
pub struct LoginSuccess {
    /// Four-word identity.
    pub four_words: String,
    /// Display name.
    pub display_name: String,
}

/// Navigation messages.
#[derive(Debug, Clone)]
pub enum NavigationMessage {
    /// Navigate to a view.
    GoTo(ActiveView),
    /// Go back in history.
    Back,
    /// Select a detail tab.
    SelectTab(DetailTab),
    /// Select an entity.
    SelectEntity(Option<Entity>),
}

/// Sidebar messages.
#[derive(Debug, Clone)]
pub enum SidebarMessage {
    /// Toggle a section's expansion.
    ToggleSection(SidebarSection),
    /// Toggle an organization's expansion.
    ToggleOrg(String),
    /// Entity clicked.
    EntityClicked(Entity),
    /// Contact clicked.
    ContactClicked(Contact),
    /// Create entity button pressed.
    CreateEntity(CreateEntityContext),
}

/// Context for creating a new entity.
#[derive(Debug, Clone)]
pub struct CreateEntityContext {
    /// Parent organization ID (if any).
    pub parent_org_id: Option<String>,
    /// Entity type to create.
    pub entity_type: crate::state::EntityType,
}

/// Chat/messaging events.
#[derive(Debug, Clone)]
pub enum ChatMessageEvent {
    /// Compose text changed.
    ComposeChanged(String),
    /// Send message pressed.
    SendMessage,
    /// Message sent result.
    MessageSent(Result<ChatMessage, String>),
    /// Open thread panel.
    OpenThread(ChatMessage),
    /// Close thread panel.
    CloseThread,
    /// Thread compose text changed.
    ThreadComposeChanged(String),
    /// Send thread reply.
    SendThreadReply,
    /// Messages loaded.
    MessagesLoaded(String, Vec<ChatMessage>),
    /// Thread replies loaded.
    ThreadRepliesLoaded(String, Vec<ChatMessage>),
    /// New message received (from subscription).
    MessageReceived(ChatMessage),
    /// Start editing a message.
    StartEdit(ChatMessage),
    /// Cancel message editing.
    CancelEdit,
    /// Edit text changed.
    EditTextChanged(String),
    /// Submit edited message.
    SubmitEdit,
    /// Message edited result.
    MessageEdited(Result<ChatMessage, String>),
    /// Delete message pressed.
    DeleteMessagePressed(String),
    /// Confirm delete message.
    ConfirmDeleteMessage(String),
    /// Message deleted result.
    MessageDeleted(Result<String, String>),
    /// Add reaction to message.
    AddReaction {
        /// The message ID.
        message_id: String,
        /// The emoji reaction.
        emoji: String,
    },
    /// Remove reaction from message.
    RemoveReaction {
        /// The message ID.
        message_id: String,
        /// The emoji reaction.
        emoji: String,
    },
    /// Reaction updated result.
    ReactionUpdated(Result<ChatMessage, String>),
}

/// Contact messages.
#[derive(Debug, Clone)]
pub enum ContactMessage {
    /// Contacts loaded.
    ContactsLoaded(Vec<Contact>),
    /// Add contact pressed.
    AddContactPressed,
    /// Contact name input changed.
    NameChanged(String),
    /// Four-word input changed.
    FourWordsChanged(String),
    /// Local-only toggle changed.
    LocalOnlyChanged(bool),
    /// Submit add contact.
    SubmitAddContact,
    /// Contact added result.
    ContactAdded(Result<Contact, String>),
    /// Presence updated.
    PresenceUpdated(String, crate::state::ContactStatus),
    /// Search query changed.
    SearchChanged(String),
    /// Toggle favorite status.
    ToggleFavorite(String),
    /// Remove contact pressed.
    RemoveContactPressed(String),
    /// Confirm remove contact.
    ConfirmRemoveContact(String),
    /// Contact removed result.
    ContactRemoved(Result<String, String>),
    /// Link to network pressed (for local-only contacts).
    LinkToNetworkPressed(String),
    /// Contact selected for detail view.
    ContactSelected(Contact),
    /// Close contact detail view.
    CloseContactDetail,
}

/// Kanban messages.
#[derive(Debug, Clone)]
pub enum KanbanMessage {
    /// Cards loaded.
    CardsLoaded(String, Vec<KanbanCard>),
    /// Create card pressed.
    CreateCardPressed(String),
    /// Card title input changed.
    CardTitleChanged(String),
    /// Card description changed.
    CardDescriptionChanged(String),
    /// Card priority changed.
    CardPriorityChanged(CardPriority),
    /// Card assignee changed.
    CardAssigneeChanged(String),
    /// Submit create card.
    SubmitCreateCard,
    /// Card created.
    CardCreated(Result<KanbanCard, String>),
    /// Card clicked.
    CardClicked(KanbanCard),
    /// Close card modal.
    CloseCardModal,
    /// Enter edit mode for card.
    EditCard(KanbanCard),
    /// Submit card edit.
    SubmitEditCard,
    /// Card updated result.
    CardUpdated(Result<KanbanCard, String>),
    /// Delete card button pressed.
    DeleteCardPressed(String),
    /// Confirm delete card.
    ConfirmDeleteCard(String),
    /// Card deleted result.
    CardDeleted(Result<String, String>),
    /// Card drag started.
    CardDragStarted(String),
    /// Card dropped.
    CardDropped {
        /// The card ID.
        card_id: String,
        /// The target column.
        column: String,
        /// The position in the column.
        position: u32,
    },
    /// Card moved result.
    CardMoved(Result<(), String>),
}

/// Call/WebRTC messages.
#[derive(Debug, Clone)]
pub enum CallMessage {
    /// Initiate a call.
    Initiate {
        /// The four-word address of the peer.
        four_words: String,
        /// Whether to include video.
        has_video: bool,
    },
    /// Call initiated result.
    CallInitiated(Result<String, String>),
    /// Accept incoming call.
    Accept,
    /// Reject incoming call.
    Reject,
    /// End active call.
    End,
    /// Toggle mute.
    ToggleMute,
    /// Toggle video.
    ToggleVideo,
    /// Toggle screen share.
    ToggleScreenShare,
    /// Call status changed.
    StatusChanged(crate::state::CallStatus),
    /// Call ended.
    CallEnded(Option<String>),
    /// Incoming call received.
    IncomingCall(crate::state::CallInfo),
    /// Refresh available media devices.
    RefreshDevices,
    /// Devices list updated.
    DevicesUpdated(crate::state::MediaDevices),
    /// Select audio input device.
    SelectAudioInput(String),
    /// Select audio output device.
    SelectAudioOutput(String),
    /// Select video device.
    SelectVideoDevice(String),
    /// Participant joined the call.
    ParticipantJoined(crate::state::CallParticipant),
    /// Participant left the call.
    ParticipantLeft(String),
    /// Participant media state changed.
    ParticipantMediaChanged {
        /// The participant's four-word identity.
        four_words: String,
        /// Whether video is enabled.
        video_enabled: bool,
        /// Whether audio is enabled.
        audio_enabled: bool,
    },
}

/// Network messages.
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Toggle networking on/off.
    ToggleNetworking,
    /// Network started.
    NetworkStarted(Result<NetworkStartedInfo, String>),
    /// Network stopped.
    NetworkStopped,
    /// Peer connected.
    PeerConnected(PeerInfo),
    /// Peer disconnected.
    PeerDisconnected(String),
    /// Connect to specific peer.
    ConnectToPeer(String),
    /// Refresh peer list.
    RefreshPeers,
    /// Peers updated.
    PeersUpdated(Vec<PeerInfo>),
}

/// Info returned when network starts.
#[derive(Debug, Clone)]
pub struct NetworkStartedInfo {
    /// Listen address.
    pub listen_address: String,
    /// External address (if known).
    pub external_address: Option<String>,
}

/// Storage/drive messages.
#[derive(Debug, Clone)]
pub enum StorageMessage {
    /// Load files for entity.
    LoadFiles(String),
    /// Files loaded.
    FilesLoaded(String, Vec<FileInfo>),
    /// Upload file.
    UploadFile(String, Vec<u8>),
    /// File uploaded.
    FileUploaded(Result<FileInfo, String>),
    /// Delete file.
    DeleteFile(String),
    /// File deleted.
    FileDeleted(Result<(), String>),
    /// Create folder.
    CreateFolder(String),
}

/// File information.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// File ID.
    pub id: String,
    /// File name.
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// Whether this is a folder.
    pub is_folder: bool,
    /// Created timestamp.
    pub created_at: i64,
}

/// Modal messages.
#[derive(Debug, Clone)]
pub enum ModalMessage {
    /// Show a modal.
    Show(ModalType),
    /// Close the current modal.
    Close,
    /// Entity name input changed.
    EntityNameChanged(String),
    /// Entity description input changed.
    EntityDescriptionChanged(String),
    /// Submit create entity form.
    SubmitCreateEntity,
    /// Contact name changed (for add contact modal).
    ContactNameChanged(String),
    /// Contact four-words changed.
    ContactFourWordsChanged(String),
    /// Submit add contact form.
    SubmitAddContact,
}

/// Modal types.
#[derive(Debug, Clone)]
pub enum ModalType {
    /// Create entity modal.
    CreateEntity(CreateEntityContext),
    /// Add contact modal.
    AddContact,
    /// Create kanban card modal (column name).
    CreateCard(String),
    /// Card detail modal.
    CardDetail(KanbanCard),
    /// Edit card modal.
    EditCard(KanbanCard),
    /// Delete card confirmation modal.
    DeleteCardConfirm(KanbanCard),
    /// Contact detail modal.
    ContactDetail(Contact),
    /// Remove contact confirmation modal.
    RemoveContactConfirm(Contact),
    /// Settings modal.
    Settings,
    /// Linking modal (link local entity to network).
    Linking(String),
    /// Edit message modal.
    EditMessage(ChatMessage),
    /// Delete message confirmation modal.
    DeleteMessageConfirm(ChatMessage),
}

/// Core events from the backend.
#[derive(Debug, Clone)]
pub enum CoreEvent {
    /// Message received from network.
    MessageReceived(ChatMessage),
    /// Peer connected.
    PeerConnected(PeerInfo),
    /// Peer disconnected.
    PeerDisconnected(String),
    /// Entity updated.
    EntityUpdated(Entity),
    /// CRDT sync completed.
    SyncCompleted(String, usize),
}
