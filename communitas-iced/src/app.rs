// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Main application struct implementing Iced's MVU architecture.
//!
//! This module contains the `CommunitasApp` struct which manages all application
//! state and handles the Model-View-Update cycle.

use crate::error::AppError;
use crate::message::{
    AuthMessage, CallMessage, ChatMessageEvent, ContactMessage, KanbanMessage, LoginSuccess,
    Message, ModalMessage, ModalType, NavigationMessage, NetworkMessage, NetworkStartedInfo,
    SidebarMessage, StorageMessage, UpdateMessage,
};
use crate::state::{
    ActiveView, AuthState, CallInfo, CallState, CallStatus, ChatMessage, Contact, DetailTab,
    Entity, MemberRole, NetworkInfo, SidebarState, ThreadState,
};
use crate::update::{self, UpdateCheckResult, UpdateConfig, UpdateInfo, UpdateStatus};
#[cfg(feature = "demo")]
use iced::keyboard;
use iced::widget::pane_grid;
use iced::{Task, Theme};
use std::collections::HashMap;
use std::time::Duration;

/// The pane types for the main layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    /// Sidebar pane (entity tree, contacts).
    Sidebar,
    /// Main detail pane (chat, kanban, etc.).
    Detail,
    /// Thread panel (when open).
    Thread,
}

/// Main application state.
pub struct CommunitasApp {
    /// Authentication state (pre-login).
    auth_state: AuthState,
    /// Main application state (post-login).
    app_state: Option<AppState>,
    /// Current theme.
    theme: Theme,
    /// Pane grid state for layout.
    panes: pane_grid::State<PaneType>,
    /// Main detail pane ID.
    detail_pane: pane_grid::Pane,
    /// Active modal (if any).
    active_modal: Option<ModalType>,
    /// Last error (for display).
    last_error: Option<AppError>,
    /// Modal form state.
    modal_form_state: crate::views::ModalFormState,
    /// Update status.
    update_status: UpdateStatus,
    /// Update configuration.
    update_config: UpdateConfig,
    /// Available update info (if any).
    available_update: Option<UpdateInfo>,
}

/// Application state after successful authentication.
pub struct AppState {
    /// User's four-word identity.
    pub four_words: String,
    /// User's display name.
    pub display_name: String,
    /// Current navigation view.
    pub active_view: ActiveView,
    /// Selected detail tab.
    pub detail_tab: DetailTab,
    /// Selected entity (if any).
    pub selected_entity: Option<Entity>,
    /// Sidebar UI state.
    pub sidebar: SidebarState,
    /// All entities (organizations, projects, channels, groups).
    pub entities: Vec<Entity>,
    /// All contacts.
    pub contacts: Vec<Contact>,
    /// Messages by entity ID.
    pub messages: HashMap<String, Vec<ChatMessage>>,
    /// Thread state (for Slack-style threading).
    pub thread_state: Option<ThreadState>,
    /// Kanban cards by entity ID.
    pub kanban_cards: HashMap<String, Vec<crate::state::KanbanCard>>,
    /// Call state.
    pub call_state: CallState,
    /// Network information.
    pub network_info: NetworkInfo,
    /// Message compose text.
    pub compose_text: String,
    /// Thread compose text.
    pub thread_compose_text: String,
    /// Files by entity ID.
    pub files: HashMap<String, Vec<crate::message::FileInfo>>,
    /// Documents by entity ID.
    pub documents: HashMap<String, Vec<Document>>,
    /// Editing message text (for edit message modal).
    pub editing_message_text: String,
}

/// A document in the system.
#[derive(Debug, Clone)]
pub struct Document {
    /// Document ID.
    pub id: String,
    /// Document title.
    pub title: String,
    /// Document content (markdown).
    pub content: String,
    /// Created timestamp.
    pub created_at: i64,
    /// Last modified timestamp.
    pub modified_at: i64,
}

impl Default for CommunitasApp {
    fn default() -> Self {
        let (app, _) = Self::new();
        app
    }
}

impl CommunitasApp {
    /// Create a new application instance with initial load task.
    ///
    /// Returns the app and a Task to load available vaults.
    #[allow(clippy::too_many_lines)]
    pub fn new() -> (Self, Task<Message>) {
        // Create pane grid with sidebar and detail
        let (mut panes, sidebar_pane) = pane_grid::State::new(PaneType::Sidebar);
        // Split returns Option, handle gracefully
        let detail_pane = panes
            .split(pane_grid::Axis::Vertical, sidebar_pane, PaneType::Detail)
            .map(|(pane, _)| pane)
            .unwrap_or(sidebar_pane); // Fallback to sidebar pane if split fails

        // Check for demo mode (auto-login with sample data)
        #[cfg(feature = "demo")]
        let app_state = Some(Self::create_demo_state());

        #[cfg(not(feature = "demo"))]
        let app_state = None;

        let app = Self {
            auth_state: AuthState::default(),
            app_state,
            theme: Theme::Light,
            panes,
            detail_pane,
            active_modal: None,
            last_error: None,
            modal_form_state: crate::views::ModalFormState::default(),
            update_status: UpdateStatus::default(),
            update_config: UpdateConfig::default(),
            available_update: None,
        };

        // Load available vaults on startup (skipped in demo mode since already logged in)
        #[cfg(feature = "demo")]
        let load_vaults = Task::none();

        #[cfg(not(feature = "demo"))]
        let load_vaults = {
            use crate::state::VaultInfo;
            Task::perform(
                async {
                    // In real app, load from storage
                    let path = dirs::data_local_dir()
                        .map(|p| {
                            p.join("communitas")
                                .join("vaults")
                                .join("default")
                                .to_string_lossy()
                                .into_owned()
                        })
                        .unwrap_or_else(|| String::from("./vaults/default"));

                    vec![VaultInfo {
                        four_words: String::new(),
                        display_name: "Default".to_string(),
                        path,
                        biometric_available: false,
                    }]
                },
                |vaults| Message::Auth(AuthMessage::VaultsLoaded(vaults)),
            )
        };

        // Check for updates 2 seconds after startup (silent background check)
        let update_check_task = Task::perform(
            async {
                // Delay before checking for updates
                tokio::time::sleep(Duration::from_secs(2)).await;
            },
            |()| Message::Update(UpdateMessage::CheckForUpdates),
        );

        // Batch startup tasks
        let startup_tasks = Task::batch([load_vaults, update_check_task]);

        (app, startup_tasks)
    }

    /// Create demo state with sample data for testing.
    #[allow(dead_code)]
    fn create_demo_state() -> AppState {
        use crate::state::{
            BootstrapNode, CardPriority, ContactStatus, EntityType, KanbanCard, PeerInfo,
        };

        let now = chrono::Utc::now().timestamp();

        // Sample contacts
        let contacts = vec![
            Contact {
                id: "contact-1".to_string(),
                display_name: "Alice Johnson".to_string(),
                four_words: Some("ocean-forest-moon-star".to_string()),
                status: ContactStatus::Online,
                is_local_only: false,
                is_favorite: true,
                last_seen: Some(now),
            },
            Contact {
                id: "contact-2".to_string(),
                display_name: "Bob Smith".to_string(),
                four_words: Some("river-mountain-sun-cloud".to_string()),
                status: ContactStatus::Away,
                is_local_only: false,
                is_favorite: false,
                last_seen: Some(now - 300),
            },
            Contact {
                id: "contact-3".to_string(),
                display_name: "Carol Davis".to_string(),
                four_words: Some("wind-earth-fire-water".to_string()),
                status: ContactStatus::Offline,
                is_local_only: false,
                is_favorite: false,
                last_seen: Some(now - 86400),
            },
            Contact {
                id: "contact-4".to_string(),
                display_name: "David Lee".to_string(),
                four_words: Some("alpha-beta-gamma-delta".to_string()),
                status: ContactStatus::Online,
                is_local_only: true,
                is_favorite: true,
                last_seen: Some(now),
            },
        ];

        // Sample entities (organizations, projects, channels, groups)
        let mut entities = vec![
            Entity {
                id: "org-1".to_string(),
                name: "Saorsa Labs".to_string(),
                four_words: Some("saorsa-labs-main-org".to_string()),
                entity_type: EntityType::Organisation,
                description: Some("Main organization for Saorsa Labs projects".to_string()),
                parent_org_id: None,
                role: MemberRole::Owner, // User owns this org
                member_count: 12,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 30,
            },
            Entity {
                id: "org-2".to_string(),
                name: "Open Source Collective".to_string(),
                four_words: Some("open-source-dev-org".to_string()),
                entity_type: EntityType::Organisation,
                description: Some("Open source development community".to_string()),
                parent_org_id: None,
                role: MemberRole::Member, // User is just a member here
                member_count: 45,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 60,
            },
        ];

        // Add projects, channels, groups under organizations
        entities.extend(vec![
            Entity {
                id: "proj-1".to_string(),
                name: "Communitas App".to_string(),
                four_words: Some("communitas-main-project".to_string()),
                entity_type: EntityType::Project,
                description: Some("Main Communitas application development".to_string()),
                parent_org_id: Some("org-1".to_string()),
                role: MemberRole::Owner,
                member_count: 8,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 20,
            },
            Entity {
                id: "channel-1".to_string(),
                name: "general".to_string(),
                four_words: Some("general-chat-channel".to_string()),
                entity_type: EntityType::Channel,
                description: Some("General discussion channel".to_string()),
                parent_org_id: Some("org-1".to_string()),
                role: MemberRole::Owner,
                member_count: 12,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 30,
            },
            Entity {
                id: "channel-2".to_string(),
                name: "development".to_string(),
                four_words: Some("dev-discuss-channel".to_string()),
                entity_type: EntityType::Channel,
                description: Some("Development discussions and code reviews".to_string()),
                parent_org_id: Some("org-1".to_string()),
                role: MemberRole::Owner,
                member_count: 8,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 25,
            },
            Entity {
                id: "group-1".to_string(),
                name: "Core Team".to_string(),
                four_words: Some("core-team-group".to_string()),
                entity_type: EntityType::Group,
                description: Some("Core development team".to_string()),
                parent_org_id: Some("org-1".to_string()),
                role: MemberRole::Owner,
                member_count: 4,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 28,
            },
            Entity {
                id: "proj-2".to_string(),
                name: "P2P Network".to_string(),
                four_words: Some("p2p-network-project".to_string()),
                entity_type: EntityType::Project,
                description: Some("Peer-to-peer networking layer".to_string()),
                parent_org_id: Some("org-2".to_string()),
                role: MemberRole::Member,
                member_count: 15,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 45,
            },
            Entity {
                id: "channel-3".to_string(),
                name: "announcements".to_string(),
                four_words: Some("announce-channel-oss".to_string()),
                entity_type: EntityType::Channel,
                description: Some("Important announcements".to_string()),
                parent_org_id: Some("org-2".to_string()),
                role: MemberRole::Member,
                member_count: 45,
                is_local_only: false,
                is_personal: false,
                created_at: now - 86400 * 55,
            },
        ]);

        // Add a personal entity for the Personal section
        entities.push(Entity {
            id: "personal-notes".to_string(),
            name: "My Notes".to_string(),
            four_words: None,
            entity_type: EntityType::Group,
            description: Some("Personal notes and reminders".to_string()),
            parent_org_id: None,
            role: MemberRole::Owner,
            member_count: 1,
            is_local_only: true,
            is_personal: true,
            created_at: now - 86400 * 10,
        });

        // Sample messages for entities
        let mut messages: HashMap<String, Vec<ChatMessage>> = HashMap::new();

        // Messages for general channel
        messages.insert(
            "channel-1".to_string(),
            vec![
                ChatMessage {
                    id: "msg-1".to_string(),
                    entity_id: "channel-1".to_string(),
                    author: "ocean-forest-moon-star".to_string(),
                    author_display_name: Some("Alice Johnson".to_string()),
                    text: "Hey everyone! Welcome to the new Communitas app!".to_string(),
                    reply_to_id: None,
                    timestamp: chrono::Utc::now().timestamp() - 3600,
                    is_edited: false,
                    reactions: HashMap::new(),
                    is_deleted: false,
                },
                ChatMessage {
                    id: "msg-2".to_string(),
                    entity_id: "channel-1".to_string(),
                    author: "river-mountain-sun-cloud".to_string(),
                    author_display_name: Some("Bob Smith".to_string()),
                    text: "This looks great! Love the decentralized approach.".to_string(),
                    reply_to_id: None,
                    timestamp: chrono::Utc::now().timestamp() - 3000,
                    is_edited: false,
                    reactions: HashMap::new(),
                    is_deleted: false,
                },
                ChatMessage {
                    id: "msg-3".to_string(),
                    entity_id: "channel-1".to_string(),
                    author: "demo-user-four-words".to_string(),
                    author_display_name: Some("Demo User".to_string()),
                    text: "Excited to be part of this!".to_string(),
                    reply_to_id: None,
                    timestamp: chrono::Utc::now().timestamp() - 1800,
                    is_edited: false,
                    reactions: HashMap::new(),
                    is_deleted: false,
                },
            ],
        );

        // Messages for development channel
        messages.insert(
            "channel-2".to_string(),
            vec![
                ChatMessage {
                    id: "msg-4".to_string(),
                    entity_id: "channel-2".to_string(),
                    author: "wind-earth-fire-water".to_string(),
                    author_display_name: Some("Carol Davis".to_string()),
                    text: "Just pushed a fix for the WebRTC connection issue.".to_string(),
                    reply_to_id: None,
                    timestamp: chrono::Utc::now().timestamp() - 7200,
                    is_edited: false,
                    reactions: HashMap::new(),
                    is_deleted: false,
                },
                ChatMessage {
                    id: "msg-5".to_string(),
                    entity_id: "channel-2".to_string(),
                    author: "alpha-beta-gamma-delta".to_string(),
                    author_display_name: Some("David Lee".to_string()),
                    text: "Thanks Carol! I'll test it now.".to_string(),
                    reply_to_id: None,
                    timestamp: chrono::Utc::now().timestamp() - 6000,
                    is_edited: false,
                    reactions: HashMap::new(),
                    is_deleted: false,
                },
            ],
        );

        // Sample kanban cards
        let mut kanban_cards: HashMap<String, Vec<KanbanCard>> = HashMap::new();
        kanban_cards.insert(
            "proj-1".to_string(),
            vec![
                KanbanCard {
                    id: "card-1".to_string(),
                    project_id: "proj-1".to_string(),
                    title: "Implement WebRTC calls".to_string(),
                    description: Some("Add video and audio calling support".to_string()),
                    column: "in_progress".to_string(),
                    priority: CardPriority::High,
                    assignee: Some("ocean-forest-moon-star".to_string()),
                    position: 0,
                    comment_count: 3,
                    created_at: now - 86400 * 5,
                    is_archived: false,
                },
                KanbanCard {
                    id: "card-2".to_string(),
                    project_id: "proj-1".to_string(),
                    title: "Add file sharing".to_string(),
                    description: Some("Enable drag-and-drop file uploads".to_string()),
                    column: "todo".to_string(),
                    priority: CardPriority::Normal,
                    assignee: None,
                    position: 0,
                    comment_count: 0,
                    created_at: now - 86400 * 3,
                    is_archived: false,
                },
                KanbanCard {
                    id: "card-3".to_string(),
                    project_id: "proj-1".to_string(),
                    title: "Fix login flow".to_string(),
                    description: None,
                    column: "done".to_string(),
                    priority: CardPriority::Normal,
                    assignee: Some("river-mountain-sun-cloud".to_string()),
                    position: 0,
                    comment_count: 5,
                    created_at: now - 86400 * 10,
                    is_archived: false,
                },
                KanbanCard {
                    id: "card-4".to_string(),
                    project_id: "proj-1".to_string(),
                    title: "Add dark mode".to_string(),
                    description: Some("Theme toggle in settings".to_string()),
                    column: "backlog".to_string(),
                    priority: CardPriority::Low,
                    assignee: None,
                    position: 0,
                    comment_count: 1,
                    created_at: now - 86400 * 2,
                    is_archived: false,
                },
            ],
        );

        // Network info with some demo peers
        let mut network_info = NetworkInfo::new();
        network_info.is_networking = true;
        network_info.listen_address = Some("0.0.0.0:50000".to_string());
        network_info.external_address = Some("203.0.113.42:50000".to_string());
        network_info.peers = vec![
            PeerInfo {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: Some("Alice Johnson".to_string()),
                endpoint: "192.168.1.100:50001".to_string(),
                last_seen: now,
                is_bootstrap: false,
            },
            PeerInfo {
                four_words: "river-mountain-sun-cloud".to_string(),
                display_name: Some("Bob Smith".to_string()),
                endpoint: "192.168.1.101:50002".to_string(),
                last_seen: now - 60,
                is_bootstrap: false,
            },
        ];
        network_info.bootstrap_nodes = vec![
            BootstrapNode {
                name: "Bootstrap EU".to_string(),
                address: "eu.communitas.network:50000".to_string(),
                is_connected: true,
            },
            BootstrapNode {
                name: "Bootstrap US".to_string(),
                address: "us.communitas.network:50000".to_string(),
                is_connected: false,
            },
        ];

        // Create sidebar state with expanded orgs
        let mut sidebar = SidebarState::new();
        sidebar.toggle_org("org-1".to_string());
        sidebar.toggle_org("org-2".to_string());

        // Sample files for entities
        let mut files: HashMap<String, Vec<crate::message::FileInfo>> = HashMap::new();
        files.insert(
            "org-1".to_string(),
            vec![
                crate::message::FileInfo {
                    id: "file-1".to_string(),
                    name: "README.md".to_string(),
                    size: 2048,
                    is_folder: false,
                    created_at: now - 86400 * 10,
                },
                crate::message::FileInfo {
                    id: "file-2".to_string(),
                    name: "team-photo.jpg".to_string(),
                    size: 1_500_000,
                    is_folder: false,
                    created_at: now - 86400 * 5,
                },
                crate::message::FileInfo {
                    id: "folder-1".to_string(),
                    name: "docs".to_string(),
                    size: 0,
                    is_folder: true,
                    created_at: now - 86400 * 15,
                },
                crate::message::FileInfo {
                    id: "file-3".to_string(),
                    name: "roadmap.pdf".to_string(),
                    size: 450_000,
                    is_folder: false,
                    created_at: now - 86400 * 3,
                },
            ],
        );

        // Sample documents for entities
        let mut documents: HashMap<String, Vec<Document>> = HashMap::new();
        documents.insert(
            "org-1".to_string(),
            vec![
                Document {
                    id: "doc-1".to_string(),
                    title: "Getting Started Guide".to_string(),
                    content: "# Getting Started\n\nWelcome to Saorsa Labs!".to_string(),
                    created_at: now - 86400 * 20,
                    modified_at: now - 86400 * 2,
                },
                Document {
                    id: "doc-2".to_string(),
                    title: "Architecture Overview".to_string(),
                    content: "# Architecture\n\n## Core Components\n\n- P2P Network\n- CRDT Storage\n- Encryption Layer".to_string(),
                    created_at: now - 86400 * 15,
                    modified_at: now - 86400,
                },
            ],
        );
        documents.insert(
            "proj-1".to_string(),
            vec![Document {
                id: "doc-3".to_string(),
                title: "Project Roadmap".to_string(),
                content: "# Communitas Roadmap\n\n## Q1 2025\n- [ ] WebRTC Integration\n- [ ] File Sharing".to_string(),
                created_at: now - 86400 * 10,
                modified_at: now - 86400,
            }],
        );

        AppState {
            four_words: "demo-user-four-words".to_string(),
            display_name: "Demo User".to_string(),
            // Start with Alice Johnson's chat for testing
            active_view: ActiveView::ContactChat {
                four_words: "ocean-forest-moon-star".to_string(),
                display_name: Some("Alice Johnson".to_string()),
            },
            detail_tab: DetailTab::Chat,
            selected_entity: None,
            sidebar,
            entities,
            contacts,
            messages,
            thread_state: None,
            kanban_cards,
            call_state: CallState::default(),
            network_info,
            compose_text: String::new(),
            thread_compose_text: String::new(),
            files,
            documents,
            editing_message_text: String::new(),
        }
    }

    /// Get the application title.
    #[must_use]
    pub fn title(&self) -> String {
        if let Some(ref app) = self.app_state {
            format!("Communitas - {}", app.display_name)
        } else {
            "Communitas".to_string()
        }
    }

    /// Handle incoming messages and update state.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Auth(auth_msg) => self.handle_auth(auth_msg),
            Message::Navigate(nav_msg) => self.handle_navigation(nav_msg),
            Message::Sidebar(sidebar_msg) => self.handle_sidebar(sidebar_msg),
            Message::Chat(chat_msg) => self.handle_chat(chat_msg),
            Message::Contact(contact_msg) => self.handle_contact(contact_msg),
            Message::Kanban(kanban_msg) => self.handle_kanban(kanban_msg),
            Message::Call(call_msg) => self.handle_call(call_msg),
            Message::Network(network_msg) => self.handle_network(network_msg),
            Message::Storage(storage_msg) => self.handle_storage(storage_msg),
            Message::Modal(modal_msg) => self.handle_modal(modal_msg),
            Message::Update(update_msg) => self.handle_update(update_msg),
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                Task::none()
            }
            Message::Tick(_instant) => {
                // Periodic updates (e.g., call duration)
                Task::none()
            }
            Message::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
                Task::none()
            }
            Message::PaneDragged(_event) => {
                // Handle pane drag if needed
                Task::none()
            }
            Message::CoreEvent(event) => self.handle_core_event(event),
            Message::Error(error) => {
                self.last_error = Some(error);
                Task::none()
            }
            Message::Noop => Task::none(),
            #[cfg(feature = "demo")]
            Message::TestAction(action) => self.handle_test_action(action),
        }
    }

    /// Handle test actions (only in demo mode).
    #[cfg(feature = "demo")]
    fn handle_test_action(&mut self, action: crate::message::TestAction) -> Task<Message> {
        use crate::message::TestAction;

        if let Some(ref mut app) = self.app_state {
            match action {
                TestAction::SendTestMessage => {
                    tracing::info!("TestAction: SendTestMessage triggered");
                    // Create and add a test message
                    let test_msg = ChatMessage {
                        id: format!("test-{}", chrono::Utc::now().timestamp_millis()),
                        entity_id: match &app.active_view {
                            ActiveView::ContactChat { four_words, .. } => four_words.clone(),
                            ActiveView::Chat { entity_id, .. } => entity_id.clone(),
                            _ => "test-entity".to_string(),
                        },
                        author: app.four_words.clone(),
                        author_display_name: Some(app.display_name.clone()),
                        text: format!(
                            "Test message sent at {}",
                            chrono::Utc::now().format("%H:%M:%S")
                        ),
                        reply_to_id: None,
                        timestamp: chrono::Utc::now().timestamp(),
                        is_edited: false,
                        reactions: std::collections::HashMap::new(),
                        is_deleted: false,
                    };

                    // Add to messages for current entity
                    let entity_id = test_msg.entity_id.clone();
                    if let Some(msgs) = app.messages.get_mut(&entity_id) {
                        msgs.push(test_msg);
                    } else {
                        app.messages.insert(entity_id, vec![test_msg]);
                    }
                }
                TestAction::SwitchToHome => {
                    tracing::info!("TestAction: SwitchToHome");
                    app.active_view = ActiveView::Home;
                }
                TestAction::SwitchToContactChat => {
                    tracing::info!("TestAction: SwitchToContactChat");
                    app.active_view = ActiveView::ContactChat {
                        four_words: "ocean-forest-moon-star".to_string(),
                        display_name: Some("Alice Johnson".to_string()),
                    };
                }
                TestAction::SwitchToChannelChat => {
                    tracing::info!("TestAction: SwitchToChannelChat");
                    if let Some(entity) = app
                        .entities
                        .iter()
                        .find(|e| e.entity_type == crate::state::EntityType::Channel)
                    {
                        app.active_view = ActiveView::Chat {
                            entity_type: "Channel".to_string(),
                            entity_id: entity.id.clone(),
                            entity_name: entity.name.clone(),
                        };
                    }
                }
                TestAction::CreateOrganization => {
                    tracing::info!("TestAction: CreateOrganization");
                    use crate::state::{EntityType, MemberRole};
                    let org_count = app
                        .entities
                        .iter()
                        .filter(|e| e.entity_type == EntityType::Organisation)
                        .count()
                        + 1;
                    let org = Entity {
                        id: format!("org-test-{}", chrono::Utc::now().timestamp_millis()),
                        four_words: None,
                        entity_type: EntityType::Organisation,
                        name: format!("Test Org {}", org_count),
                        description: Some(
                            "A test organization created via keyboard shortcut".to_string(),
                        ),
                        parent_org_id: None,
                        role: MemberRole::Owner,
                        member_count: 1,
                        is_local_only: true,
                        is_personal: false,
                        created_at: chrono::Utc::now().timestamp(),
                    };
                    tracing::info!("Created organization: {}", org.name);
                    app.entities.push(org);
                }
                TestAction::CreateProject => {
                    tracing::info!("TestAction: CreateProject");
                    use crate::state::{EntityType, MemberRole};
                    // Find parent org for the project
                    let parent_org_id = app
                        .entities
                        .iter()
                        .find(|e| e.entity_type == EntityType::Organisation)
                        .map(|e| e.id.clone());
                    let proj_count = app
                        .entities
                        .iter()
                        .filter(|e| e.entity_type == EntityType::Project)
                        .count()
                        + 1;
                    let proj = Entity {
                        id: format!("proj-test-{}", chrono::Utc::now().timestamp_millis()),
                        four_words: None,
                        entity_type: EntityType::Project,
                        name: format!("Test Project {}", proj_count),
                        description: Some(
                            "A test project created via keyboard shortcut".to_string(),
                        ),
                        parent_org_id,
                        role: MemberRole::Owner,
                        member_count: 1,
                        is_local_only: true,
                        is_personal: false,
                        created_at: chrono::Utc::now().timestamp(),
                    };
                    tracing::info!("Created project: {}", proj.name);
                    app.entities.push(proj);
                }
                TestAction::CreateGroup => {
                    tracing::info!("TestAction: CreateGroup");
                    use crate::state::{EntityType, MemberRole};
                    let parent_org_id = app
                        .entities
                        .iter()
                        .find(|e| e.entity_type == EntityType::Organisation)
                        .map(|e| e.id.clone());
                    let grp_count = app
                        .entities
                        .iter()
                        .filter(|e| e.entity_type == EntityType::Group)
                        .count()
                        + 1;
                    let group = Entity {
                        id: format!("grp-test-{}", chrono::Utc::now().timestamp_millis()),
                        four_words: None,
                        entity_type: EntityType::Group,
                        name: format!("Test Group {}", grp_count),
                        description: Some("A test group created via keyboard shortcut".to_string()),
                        parent_org_id,
                        role: MemberRole::Owner,
                        member_count: 1,
                        is_local_only: true,
                        is_personal: false,
                        created_at: chrono::Utc::now().timestamp(),
                    };
                    tracing::info!("Created group: {}", group.name);
                    app.entities.push(group);
                }
                TestAction::OpenNetworkPanel => {
                    tracing::info!("TestAction: OpenNetworkPanel");
                    app.active_view = ActiveView::NetworkPanel;
                }
                TestAction::ToggleSidebarSection => {
                    tracing::info!("TestAction: ToggleSidebarSection");
                    // Cycle through sidebar sections
                    use crate::state::SidebarSection;
                    let sections = [
                        SidebarSection::MyOrganizations,
                        SidebarSection::MyCommunities,
                        SidebarSection::Personal,
                        SidebarSection::DirectMessages,
                    ];
                    for section in &sections {
                        let is_expanded = app.sidebar.is_section_expanded(*section);
                        if is_expanded {
                            app.sidebar.toggle_section(*section);
                            break;
                        }
                    }
                }
            }
        }
        Task::none()
    }

    /// Handle authentication messages.
    fn handle_auth(&mut self, msg: AuthMessage) -> Task<Message> {
        match msg {
            AuthMessage::VaultsLoaded(vaults) => {
                self.auth_state.vaults = vaults;
                if let Some(first) = self.auth_state.vaults.first() {
                    self.auth_state.selected_vault = Some(first.display_name.clone());
                }
                Task::none()
            }
            AuthMessage::VaultSelected(name) => {
                self.auth_state.selected_vault = Some(name);
                Task::none()
            }
            AuthMessage::PasswordChanged(password) => {
                self.auth_state.password = password;
                Task::none()
            }
            AuthMessage::LoginPressed => {
                self.auth_state.is_loading = true;
                self.auth_state.error = None;
                // In a real app, this would call the core login service
                // For now, simulate with a task
                Task::perform(
                    async {
                        // Simulated login delay
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        Ok(LoginSuccess {
                            four_words: "ocean-forest-moon-star".to_string(),
                            display_name: "Test User".to_string(),
                        })
                    },
                    |result: Result<LoginSuccess, String>| {
                        Message::Auth(AuthMessage::LoginResult(result))
                    },
                )
            }
            AuthMessage::LoginResult(result) => {
                self.auth_state.is_loading = false;
                match result {
                    Ok(success) => {
                        self.app_state = Some(AppState {
                            four_words: success.four_words,
                            display_name: success.display_name,
                            active_view: ActiveView::Home,
                            detail_tab: DetailTab::Chat,
                            selected_entity: None,
                            sidebar: SidebarState::new(),
                            entities: Vec::new(),
                            contacts: Vec::new(),
                            messages: HashMap::new(),
                            thread_state: None,
                            kanban_cards: HashMap::new(),
                            call_state: CallState::default(),
                            network_info: NetworkInfo::new(),
                            compose_text: String::new(),
                            thread_compose_text: String::new(),
                            files: HashMap::new(),
                            documents: HashMap::new(),
                            editing_message_text: String::new(),
                        });
                        self.auth_state.password.clear();
                    }
                    Err(error) => {
                        self.auth_state.error = Some(error);
                    }
                }
                Task::none()
            }
            AuthMessage::CreateIdentityPressed => {
                self.auth_state.creating_identity = true;
                Task::none()
            }
            AuthMessage::DisplayNameChanged(name) => {
                self.auth_state.new_display_name = name;
                Task::none()
            }
            AuthMessage::NewPasswordChanged(password) => {
                self.auth_state.new_password = password;
                Task::none()
            }
            AuthMessage::ConfirmPasswordChanged(password) => {
                self.auth_state.new_password_confirm = password;
                Task::none()
            }
            AuthMessage::CreateIdentitySubmit => {
                self.auth_state.is_loading = true;
                // Validate passwords match
                if !self.auth_state.passwords_match() {
                    self.auth_state.is_loading = false;
                    self.auth_state.error = Some("Passwords do not match".to_string());
                    return Task::none();
                }
                // In real app, call core identity creation
                let display_name = self.auth_state.new_display_name.clone();
                Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        Ok(LoginSuccess {
                            four_words: "alpha-beta-gamma-delta".to_string(),
                            display_name,
                        })
                    },
                    |result| Message::Auth(AuthMessage::IdentityCreated(result)),
                )
            }
            AuthMessage::IdentityCreated(result) => {
                self.auth_state.is_loading = false;
                self.auth_state.creating_identity = false;
                match result {
                    Ok(success) => {
                        self.app_state = Some(AppState {
                            four_words: success.four_words,
                            display_name: success.display_name,
                            active_view: ActiveView::Home,
                            detail_tab: DetailTab::Chat,
                            selected_entity: None,
                            sidebar: SidebarState::new(),
                            entities: Vec::new(),
                            contacts: Vec::new(),
                            messages: HashMap::new(),
                            thread_state: None,
                            kanban_cards: HashMap::new(),
                            call_state: CallState::default(),
                            network_info: NetworkInfo::new(),
                            compose_text: String::new(),
                            thread_compose_text: String::new(),
                            files: HashMap::new(),
                            documents: HashMap::new(),
                            editing_message_text: String::new(),
                        });
                        self.auth_state = AuthState::default();
                    }
                    Err(error) => {
                        self.auth_state.error = Some(error);
                    }
                }
                Task::none()
            }
            AuthMessage::BiometricPressed => {
                // Platform-specific biometric auth
                Task::none()
            }
            AuthMessage::Logout => {
                self.app_state = None;
                self.auth_state = AuthState::default();
                Task::none()
            }
            AuthMessage::CancelCreate => {
                self.auth_state.creating_identity = false;
                self.auth_state.new_display_name.clear();
                self.auth_state.new_password.clear();
                self.auth_state.new_password_confirm.clear();
                Task::none()
            }
        }
    }

    /// Handle navigation messages.
    fn handle_navigation(&mut self, msg: NavigationMessage) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                NavigationMessage::GoTo(view) => {
                    app.active_view = view;
                }
                NavigationMessage::Back => {
                    app.active_view = ActiveView::Home;
                }
                NavigationMessage::SelectTab(tab) => {
                    app.detail_tab = tab;
                }
                NavigationMessage::SelectEntity(entity) => {
                    app.selected_entity = entity;
                }
            }
        }
        Task::none()
    }

    /// Handle sidebar messages.
    fn handle_sidebar(&mut self, msg: SidebarMessage) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                SidebarMessage::ToggleSection(section) => {
                    app.sidebar.toggle_section(section);
                }
                SidebarMessage::ToggleOrg(org_id) => {
                    app.sidebar.toggle_org(org_id);
                }
                SidebarMessage::EntityClicked(entity) => {
                    let entity_id = entity.id.clone();
                    let entity_name = entity.name.clone();
                    let entity_type = entity.entity_type;
                    app.sidebar.select_entity(Some(entity_id.clone()));
                    app.selected_entity = Some(entity);
                    app.active_view = ActiveView::Chat {
                        entity_type: format!("{entity_type:?}"),
                        entity_id,
                        entity_name,
                    };
                }
                SidebarMessage::ContactClicked(contact) => {
                    tracing::info!("ContactClicked: {}", contact.display_name);
                    // Use the contact's display_name and four_words (which is Option<String>)
                    let four_words = contact
                        .four_words
                        .clone()
                        .unwrap_or_else(|| contact.id.clone());
                    let display_name = Some(contact.display_name.clone());
                    tracing::info!("Setting active_view to ContactChat for: {}", four_words);
                    app.active_view = ActiveView::ContactChat {
                        four_words,
                        display_name,
                    };
                }
                SidebarMessage::CreateEntity(context) => {
                    self.active_modal = Some(ModalType::CreateEntity(context));
                }
            }
        }
        Task::none()
    }

    /// Handle chat messages.
    fn handle_chat(&mut self, msg: ChatMessageEvent) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                ChatMessageEvent::ComposeChanged(text) => {
                    app.compose_text = text;
                }
                ChatMessageEvent::SendMessage => {
                    let text = std::mem::take(&mut app.compose_text);
                    if text.trim().is_empty() {
                        return Task::none();
                    }
                    // Get entity_id before moving into async
                    let entity_id = match &app.active_view {
                        ActiveView::Chat { entity_id, .. } => entity_id.clone(),
                        _ => return Task::none(),
                    };
                    // In real app, send via core message service
                    let four_words = app.four_words.clone();
                    let display_name = app.display_name.clone();
                    return Task::perform(
                        async move {
                            Ok(ChatMessage {
                                id: uuid::Uuid::new_v4().to_string(),
                                entity_id,
                                author: four_words,
                                author_display_name: Some(display_name),
                                text,
                                reply_to_id: None,
                                timestamp: chrono::Utc::now().timestamp(),
                                is_edited: false,
                                reactions: HashMap::new(),
                                is_deleted: false,
                            })
                        },
                        |result| Message::Chat(ChatMessageEvent::MessageSent(result)),
                    );
                }
                ChatMessageEvent::MessageSent(result) => {
                    if let Ok(message) = result {
                        let entity_id = message.entity_id.clone();
                        app.messages.entry(entity_id).or_default().push(message);
                    }
                    return Task::none();
                }
                ChatMessageEvent::OpenThread(message) => {
                    let entity_id = message.entity_id.clone();
                    app.thread_state = Some(ThreadState::new(message, entity_id));
                    // Load thread replies
                }
                ChatMessageEvent::CloseThread => {
                    app.thread_state = None;
                }
                ChatMessageEvent::ThreadComposeChanged(text) => {
                    app.thread_compose_text = text;
                }
                ChatMessageEvent::SendThreadReply => {
                    let text = std::mem::take(&mut app.thread_compose_text);
                    if text.trim().is_empty() {
                        return Task::none();
                    }
                    // Create reply message and add to thread
                    if let Some(ref mut thread) = app.thread_state {
                        let reply = ChatMessage {
                            id: uuid::Uuid::new_v4().to_string(),
                            entity_id: thread.entity_id.clone(),
                            author: app.four_words.clone(),
                            author_display_name: Some(app.display_name.clone()),
                            text,
                            reply_to_id: Some(thread.parent_message.id.clone()),
                            timestamp: chrono::Utc::now().timestamp(),
                            is_edited: false,
                            reactions: HashMap::new(),
                            is_deleted: false,
                        };
                        thread.replies.push(reply);
                    }
                }
                ChatMessageEvent::MessagesLoaded(entity_id, messages) => {
                    app.messages.insert(entity_id, messages);
                }
                ChatMessageEvent::ThreadRepliesLoaded(thread_id, replies) => {
                    if let Some(ref mut thread) = app.thread_state
                        && thread.parent_message.id == thread_id
                    {
                        thread.replies = replies;
                    }
                }
                ChatMessageEvent::MessageReceived(message) => {
                    // Route to appropriate entity
                    let entity_id = message.entity_id.clone();
                    app.messages.entry(entity_id).or_default().push(message);
                }
                ChatMessageEvent::StartEdit(message) => {
                    // Store the message text for editing
                    app.editing_message_text = message.text.clone();
                    self.active_modal = Some(ModalType::EditMessage(message));
                }
                ChatMessageEvent::CancelEdit => {
                    app.editing_message_text.clear();
                    self.active_modal = None;
                }
                ChatMessageEvent::EditTextChanged(text) => {
                    app.editing_message_text = text;
                }
                ChatMessageEvent::SubmitEdit => {
                    // Get the message being edited from modal
                    if let Some(ModalType::EditMessage(ref message)) = self.active_modal {
                        let message_id = message.id.clone();
                        let entity_id = message.entity_id.clone();
                        let new_text = std::mem::take(&mut app.editing_message_text);

                        // Find and update the message
                        if let Some(messages) = app.messages.get_mut(&entity_id)
                            && let Some(msg) = messages.iter_mut().find(|m| m.id == message_id)
                        {
                            msg.text = new_text;
                            msg.is_edited = true;
                        }
                        self.active_modal = None;
                    }
                }
                ChatMessageEvent::MessageEdited(result) => {
                    if let Ok(updated_message) = result {
                        let entity_id = updated_message.entity_id.clone();
                        if let Some(messages) = app.messages.get_mut(&entity_id)
                            && let Some(msg) =
                                messages.iter_mut().find(|m| m.id == updated_message.id)
                        {
                            *msg = updated_message;
                        }
                    }
                    self.active_modal = None;
                }
                ChatMessageEvent::DeleteMessagePressed(message_id) => {
                    // Find the message and show confirmation modal
                    if let ActiveView::Chat { entity_id, .. } = &app.active_view
                        && let Some(messages) = app.messages.get(entity_id)
                        && let Some(message) = messages.iter().find(|m| m.id == message_id)
                    {
                        self.active_modal = Some(ModalType::DeleteMessageConfirm(message.clone()));
                    }
                }
                ChatMessageEvent::ConfirmDeleteMessage(message_id) => {
                    // Soft delete the message
                    if let ActiveView::Chat { entity_id, .. } = &app.active_view
                        && let Some(messages) = app.messages.get_mut(entity_id)
                        && let Some(msg) = messages.iter_mut().find(|m| m.id == message_id)
                    {
                        msg.is_deleted = true;
                        msg.text = "[Message deleted]".to_string();
                    }
                    self.active_modal = None;
                }
                ChatMessageEvent::MessageDeleted(result) => {
                    if let Ok(message_id) = result {
                        // Already handled via ConfirmDeleteMessage
                        let _ = message_id;
                    }
                }
                ChatMessageEvent::AddReaction { message_id, emoji } => {
                    // Add user's reaction to the message
                    if let ActiveView::Chat { entity_id, .. } = &app.active_view
                        && let Some(messages) = app.messages.get_mut(entity_id)
                        && let Some(msg) = messages.iter_mut().find(|m| m.id == message_id)
                    {
                        let users = msg.reactions.entry(emoji).or_default();
                        if !users.contains(&app.four_words) {
                            users.push(app.four_words.clone());
                        }
                    }
                }
                ChatMessageEvent::RemoveReaction { message_id, emoji } => {
                    // Remove user's reaction from the message
                    if let ActiveView::Chat { entity_id, .. } = &app.active_view
                        && let Some(messages) = app.messages.get_mut(entity_id)
                        && let Some(msg) = messages.iter_mut().find(|m| m.id == message_id)
                        && let Some(users) = msg.reactions.get_mut(&emoji)
                    {
                        users.retain(|u| u != &app.four_words);
                        if users.is_empty() {
                            msg.reactions.remove(&emoji);
                        }
                    }
                }
                ChatMessageEvent::ReactionUpdated(result) => {
                    if let Ok(updated_message) = result {
                        let entity_id = updated_message.entity_id.clone();
                        if let Some(messages) = app.messages.get_mut(&entity_id)
                            && let Some(msg) =
                                messages.iter_mut().find(|m| m.id == updated_message.id)
                        {
                            msg.reactions = updated_message.reactions;
                        }
                    }
                }
            }
        }
        Task::none()
    }

    /// Handle contact messages.
    fn handle_contact(&mut self, msg: ContactMessage) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                ContactMessage::ContactsLoaded(contacts) => {
                    app.contacts = contacts;
                }
                ContactMessage::AddContactPressed => {
                    self.active_modal = Some(ModalType::AddContact);
                }
                ContactMessage::NameChanged(_)
                | ContactMessage::FourWordsChanged(_)
                | ContactMessage::LocalOnlyChanged(_) => {
                    // Update modal state
                }
                ContactMessage::SubmitAddContact => {
                    // Add contact via core
                }
                ContactMessage::ContactAdded(result) => {
                    if let Ok(contact) = result {
                        app.contacts.push(contact);
                        self.active_modal = None;
                    }
                }
                ContactMessage::PresenceUpdated(four_words, status) => {
                    // four_words is a String, contact.four_words is Option<String>
                    if let Some(contact) = app
                        .contacts
                        .iter_mut()
                        .find(|c| c.four_words.as_ref() == Some(&four_words))
                    {
                        contact.status = status;
                    }
                }
                ContactMessage::SearchChanged(query) => {
                    app.sidebar.contact_search = query;
                }
                ContactMessage::ToggleFavorite(contact_id) => {
                    if let Some(contact) = app.contacts.iter_mut().find(|c| c.id == contact_id) {
                        contact.is_favorite = !contact.is_favorite;
                    }
                }
                ContactMessage::RemoveContactPressed(contact_id) => {
                    if let Some(contact) = app.contacts.iter().find(|c| c.id == contact_id) {
                        self.active_modal = Some(ModalType::RemoveContactConfirm(contact.clone()));
                    }
                }
                ContactMessage::ConfirmRemoveContact(contact_id) => {
                    app.contacts.retain(|c| c.id != contact_id);
                    self.active_modal = None;
                }
                ContactMessage::ContactRemoved(result) => {
                    if let Ok(contact_id) = result {
                        app.contacts.retain(|c| c.id != contact_id);
                    }
                }
                ContactMessage::LinkToNetworkPressed(contact_id) => {
                    self.active_modal = Some(ModalType::Linking(contact_id));
                }
                ContactMessage::ContactSelected(contact) => {
                    self.active_modal = Some(ModalType::ContactDetail(contact));
                }
                ContactMessage::CloseContactDetail => {
                    self.active_modal = None;
                }
            }
        }
        Task::none()
    }

    /// Handle kanban messages.
    fn handle_kanban(&mut self, msg: KanbanMessage) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                KanbanMessage::CardsLoaded(entity_id, cards) => {
                    app.kanban_cards.insert(entity_id, cards);
                }
                KanbanMessage::CreateCardPressed(column) => {
                    // Show create card modal and store the column
                    self.modal_form_state.card_column = column.clone();
                    self.modal_form_state.card_title.clear();
                    self.modal_form_state.card_description.clear();
                    self.active_modal = Some(ModalType::CreateCard(column));
                }
                KanbanMessage::CardTitleChanged(title) => {
                    self.modal_form_state.card_title = title;
                }
                KanbanMessage::CardDescriptionChanged(description) => {
                    self.modal_form_state.card_description = description;
                }
                KanbanMessage::CardPriorityChanged(priority) => {
                    self.modal_form_state.card_priority = priority;
                }
                KanbanMessage::CardAssigneeChanged(assignee) => {
                    self.modal_form_state.card_assignee = assignee;
                }
                KanbanMessage::SubmitCreateCard => {
                    // Create card and add to state
                    let title = std::mem::take(&mut self.modal_form_state.card_title);
                    let description = std::mem::take(&mut self.modal_form_state.card_description);
                    let column = std::mem::take(&mut self.modal_form_state.card_column);

                    if !title.trim().is_empty()
                        && let ActiveView::Chat { entity_id, .. } = &app.active_view
                    {
                        let cards = app.kanban_cards.entry(entity_id.clone()).or_default();
                        let position = cards
                            .iter()
                            .filter(|c| c.column == column)
                            .count()
                            .try_into()
                            .unwrap_or(0);

                        let card = crate::state::KanbanCard {
                            id: uuid::Uuid::new_v4().to_string(),
                            project_id: entity_id.clone(),
                            title,
                            description: if description.trim().is_empty() {
                                None
                            } else {
                                Some(description)
                            },
                            column,
                            position,
                            assignee: None,
                            priority: crate::state::CardPriority::Normal,
                            comment_count: 0,
                            created_at: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs() as i64)
                                .unwrap_or(0),
                            is_archived: false,
                        };
                        cards.push(card);
                    }
                    self.active_modal = None;
                }
                KanbanMessage::CardCreated(result) => {
                    if let Ok(card) = result
                        && let ActiveView::Chat { entity_id, .. } = &app.active_view
                    {
                        app.kanban_cards
                            .entry(entity_id.clone())
                            .or_default()
                            .push(card);
                    }
                }
                KanbanMessage::CardClicked(card) => {
                    self.active_modal = Some(ModalType::CardDetail(card));
                }
                KanbanMessage::CloseCardModal => {
                    self.active_modal = None;
                    self.modal_form_state.editing_card_id = None;
                }
                KanbanMessage::EditCard(card) => {
                    // Populate form state with card data
                    self.modal_form_state.card_title = card.title.clone();
                    self.modal_form_state.card_description =
                        card.description.clone().unwrap_or_default();
                    self.modal_form_state.card_priority = card.priority;
                    self.modal_form_state.card_assignee = card.assignee.clone().unwrap_or_default();
                    self.modal_form_state.card_column = card.column.clone();
                    self.modal_form_state.editing_card_id = Some(card.id.clone());
                    self.active_modal = Some(ModalType::EditCard(card));
                }
                KanbanMessage::SubmitEditCard => {
                    // Update the card in state
                    if let Some(card_id) = self.modal_form_state.editing_card_id.take() {
                        let title = std::mem::take(&mut self.modal_form_state.card_title);
                        let description =
                            std::mem::take(&mut self.modal_form_state.card_description);
                        let priority = self.modal_form_state.card_priority;
                        let assignee = std::mem::take(&mut self.modal_form_state.card_assignee);

                        // Find and update the card
                        for cards in app.kanban_cards.values_mut() {
                            if let Some(card) = cards.iter_mut().find(|c| c.id == card_id) {
                                card.title = title;
                                card.description = if description.trim().is_empty() {
                                    None
                                } else {
                                    Some(description)
                                };
                                card.priority = priority;
                                card.assignee = if assignee.trim().is_empty() {
                                    None
                                } else {
                                    Some(assignee)
                                };
                                break;
                            }
                        }
                    }
                    self.active_modal = None;
                }
                KanbanMessage::CardUpdated(result) => {
                    if let Ok(updated_card) = result {
                        // Update card in state
                        for cards in app.kanban_cards.values_mut() {
                            if let Some(card) = cards.iter_mut().find(|c| c.id == updated_card.id) {
                                *card = updated_card;
                                break;
                            }
                        }
                    }
                }
                KanbanMessage::DeleteCardPressed(card_id) => {
                    // Find the card and show delete confirmation
                    for cards in app.kanban_cards.values() {
                        if let Some(card) = cards.iter().find(|c| c.id == card_id) {
                            self.active_modal = Some(ModalType::DeleteCardConfirm(card.clone()));
                            break;
                        }
                    }
                }
                KanbanMessage::ConfirmDeleteCard(card_id) => {
                    // Delete the card from state
                    for cards in app.kanban_cards.values_mut() {
                        cards.retain(|c| c.id != card_id);
                    }
                    self.active_modal = None;
                }
                KanbanMessage::CardDeleted(result) => {
                    if let Ok(card_id) = result {
                        // Remove card from state
                        for cards in app.kanban_cards.values_mut() {
                            cards.retain(|c| c.id != card_id);
                        }
                    }
                }
                KanbanMessage::CardDragStarted(_) => {
                    // Track drag state
                }
                KanbanMessage::CardDropped {
                    card_id,
                    column,
                    position,
                } => {
                    // Find and move the card to the new column
                    for cards in app.kanban_cards.values_mut() {
                        if let Some(card) = cards.iter_mut().find(|c| c.id == card_id) {
                            card.column = column.clone();
                            card.position = position;
                            break;
                        }
                    }
                    // Close the modal after moving
                    self.active_modal = None;
                }
                KanbanMessage::CardMoved(_result) => {
                    // Handle move result
                }
            }
        }
        Task::none()
    }

    /// Handle call messages.
    fn handle_call(&mut self, msg: CallMessage) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                CallMessage::Initiate {
                    four_words,
                    has_video,
                } => {
                    let call_id = uuid::Uuid::new_v4().to_string();
                    app.call_state.active_call = Some(CallInfo::new_outgoing(
                        call_id.clone(),
                        four_words.clone(),
                        has_video,
                    ));
                    app.active_view = ActiveView::Call {
                        peer_four_words: four_words,
                    };
                    // Initiate call via core WebRTC
                    return Task::perform(async move { Ok(call_id) }, |result| {
                        Message::Call(CallMessage::CallInitiated(result))
                    });
                }
                CallMessage::CallInitiated(result) => {
                    if let Err(error) = result {
                        app.call_state.active_call = None;
                        self.last_error = Some(AppError::Call(error));
                    }
                }
                CallMessage::Accept => {
                    if let Some(incoming) = app.call_state.incoming_call.take() {
                        let peer = incoming.peer_four_words.clone();
                        app.call_state.active_call = Some(incoming);
                        app.active_view = ActiveView::Call {
                            peer_four_words: peer,
                        };
                    }
                }
                CallMessage::Reject => {
                    app.call_state.incoming_call = None;
                }
                CallMessage::End => {
                    app.call_state.active_call = None;
                    app.active_view = ActiveView::Home;
                }
                CallMessage::ToggleMute => {
                    if let Some(ref mut call) = app.call_state.active_call {
                        call.is_audio_enabled = !call.is_audio_enabled;
                    }
                }
                CallMessage::ToggleVideo => {
                    if let Some(ref mut call) = app.call_state.active_call {
                        call.is_video_enabled = !call.is_video_enabled;
                    }
                }
                CallMessage::ToggleScreenShare => {
                    if let Some(ref mut call) = app.call_state.active_call {
                        call.is_screen_sharing = !call.is_screen_sharing;
                    }
                }
                CallMessage::StatusChanged(status) => {
                    if let Some(ref mut call) = app.call_state.active_call {
                        call.status = status;
                        if status == CallStatus::Connected {
                            call.start_time = Some(std::time::Instant::now());
                        }
                    }
                }
                CallMessage::CallEnded(_reason) => {
                    app.call_state.active_call = None;
                    if matches!(app.active_view, ActiveView::Call { .. }) {
                        app.active_view = ActiveView::Home;
                    }
                }
                CallMessage::IncomingCall(call_info) => {
                    app.call_state.incoming_call = Some(call_info);
                }
                CallMessage::RefreshDevices => {
                    // Refresh device list - mock for now
                    use crate::state::MediaDevice;
                    let devices = crate::state::MediaDevices {
                        audio_inputs: vec![
                            MediaDevice {
                                id: "default-mic".to_string(),
                                name: "Default Microphone".to_string(),
                                is_default: true,
                            },
                            MediaDevice {
                                id: "external-mic".to_string(),
                                name: "External USB Microphone".to_string(),
                                is_default: false,
                            },
                        ],
                        audio_outputs: vec![
                            MediaDevice {
                                id: "default-speaker".to_string(),
                                name: "Default Speakers".to_string(),
                                is_default: true,
                            },
                            MediaDevice {
                                id: "headphones".to_string(),
                                name: "Headphones".to_string(),
                                is_default: false,
                            },
                        ],
                        video_devices: vec![
                            MediaDevice {
                                id: "default-camera".to_string(),
                                name: "Built-in Camera".to_string(),
                                is_default: true,
                            },
                            MediaDevice {
                                id: "external-camera".to_string(),
                                name: "External Webcam".to_string(),
                                is_default: false,
                            },
                        ],
                        selected_audio_input: Some("default-mic".to_string()),
                        selected_audio_output: Some("default-speaker".to_string()),
                        selected_video: Some("default-camera".to_string()),
                    };
                    app.call_state.devices = devices;
                }
                CallMessage::DevicesUpdated(devices) => {
                    app.call_state.devices = devices;
                }
                CallMessage::SelectAudioInput(device_id) => {
                    app.call_state.devices.selected_audio_input = Some(device_id);
                }
                CallMessage::SelectAudioOutput(device_id) => {
                    app.call_state.devices.selected_audio_output = Some(device_id);
                }
                CallMessage::SelectVideoDevice(device_id) => {
                    app.call_state.devices.selected_video = Some(device_id);
                }
                CallMessage::ParticipantJoined(participant) => {
                    app.call_state.participants.push(participant);
                }
                CallMessage::ParticipantLeft(four_words) => {
                    app.call_state
                        .participants
                        .retain(|p| p.four_words != four_words);
                }
                CallMessage::ParticipantMediaChanged {
                    four_words,
                    video_enabled,
                    audio_enabled,
                } => {
                    if let Some(p) = app
                        .call_state
                        .participants
                        .iter_mut()
                        .find(|p| p.four_words == four_words)
                    {
                        p.video_enabled = video_enabled;
                        p.audio_enabled = audio_enabled;
                    }
                }
            }
        }
        Task::none()
    }

    /// Handle network messages.
    fn handle_network(&mut self, msg: NetworkMessage) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match msg {
                NetworkMessage::ToggleNetworking => {
                    if app.network_info.is_networking {
                        // Stop networking
                        app.network_info.is_networking = false;
                        app.network_info.peers.clear();
                        return Task::perform(async {}, |()| {
                            Message::Network(NetworkMessage::NetworkStopped)
                        });
                    } else {
                        // Start networking
                        return Task::perform(
                            async {
                                // In real app, start via core
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                Ok(NetworkStartedInfo {
                                    listen_address: "0.0.0.0:50000".to_string(),
                                    external_address: None,
                                })
                            },
                            |result| Message::Network(NetworkMessage::NetworkStarted(result)),
                        );
                    }
                }
                NetworkMessage::NetworkStarted(result) => match result {
                    Ok(info) => {
                        app.network_info.is_networking = true;
                        app.network_info.listen_address = Some(info.listen_address);
                        app.network_info.external_address = info.external_address;
                    }
                    Err(error) => {
                        app.network_info.last_error = Some(error);
                    }
                },
                NetworkMessage::NetworkStopped => {
                    app.network_info.is_networking = false;
                    app.network_info.listen_address = None;
                    app.network_info.external_address = None;
                }
                NetworkMessage::PeerConnected(peer) => {
                    app.network_info.peers.push(peer);
                }
                NetworkMessage::PeerDisconnected(four_words) => {
                    app.network_info
                        .peers
                        .retain(|p| p.four_words != four_words);
                }
                NetworkMessage::ConnectToPeer(_address) => {
                    // Connect via core
                }
                NetworkMessage::RefreshPeers => {
                    // Refresh peer list
                }
                NetworkMessage::PeersUpdated(peers) => {
                    app.network_info.peers = peers;
                }
            }
        }
        Task::none()
    }

    /// Handle storage messages.
    fn handle_storage(&mut self, msg: StorageMessage) -> Task<Message> {
        match msg {
            StorageMessage::LoadFiles(_entity_id) => {
                // Load files via core
            }
            StorageMessage::FilesLoaded(_entity_id, _files) => {
                // Store files
            }
            StorageMessage::UploadFile(_name, _data) => {
                // Upload via core
            }
            StorageMessage::FileUploaded(_result) => {
                // Handle result
            }
            StorageMessage::DeleteFile(_file_id) => {
                // Delete via core
            }
            StorageMessage::FileDeleted(_result) => {
                // Handle result
            }
            StorageMessage::CreateFolder(_name) => {
                // Create folder via core
            }
        }
        Task::none()
    }

    /// Handle modal messages.
    fn handle_modal(&mut self, msg: ModalMessage) -> Task<Message> {
        match msg {
            ModalMessage::Show(modal_type) => {
                // Clear form state when showing a new modal
                self.modal_form_state = crate::views::ModalFormState::default();
                self.active_modal = Some(modal_type);
            }
            ModalMessage::Close => {
                self.active_modal = None;
                // Clear form state on close
                self.modal_form_state = crate::views::ModalFormState::default();
            }
            ModalMessage::EntityNameChanged(name) => {
                self.modal_form_state.entity_name = name;
            }
            ModalMessage::EntityDescriptionChanged(description) => {
                self.modal_form_state.entity_description = description;
            }
            ModalMessage::SubmitCreateEntity => {
                // Create the entity and add to state
                if let Some(ref mut app) = self.app_state
                    && let Some(ModalType::CreateEntity(context)) = &self.active_modal
                {
                    let name = std::mem::take(&mut self.modal_form_state.entity_name);
                    let description = std::mem::take(&mut self.modal_form_state.entity_description);

                    if !name.trim().is_empty() {
                        let mut entity = Entity::new(
                            uuid::Uuid::new_v4().to_string(),
                            context.entity_type,
                            name,
                        );
                        if !description.trim().is_empty() {
                            entity.description = Some(description);
                        }
                        entity.parent_org_id = context.parent_org_id.clone();

                        app.entities.push(entity);
                    }
                }
                self.active_modal = None;
            }
            ModalMessage::ContactNameChanged(name) => {
                self.modal_form_state.contact_name = name;
            }
            ModalMessage::ContactFourWordsChanged(four_words) => {
                self.modal_form_state.contact_four_words = four_words;
            }
            ModalMessage::SubmitAddContact => {
                // Create the contact and add to state
                if let Some(ref mut app) = self.app_state {
                    let name = std::mem::take(&mut self.modal_form_state.contact_name);
                    let four_words = std::mem::take(&mut self.modal_form_state.contact_four_words);

                    if !name.trim().is_empty() || !four_words.trim().is_empty() {
                        let contact = crate::state::Contact {
                            id: uuid::Uuid::new_v4().to_string(),
                            display_name: name,
                            four_words: if four_words.trim().is_empty() {
                                None
                            } else {
                                Some(four_words)
                            },
                            status: crate::state::ContactStatus::Offline,
                            is_local_only: true,
                            is_favorite: false,
                            last_seen: None,
                        };
                        app.contacts.push(contact);
                    }
                }
                self.active_modal = None;
            }
        }
        Task::none()
    }

    /// Handle update messages for self-update functionality.
    fn handle_update(&mut self, msg: UpdateMessage) -> Task<Message> {
        use crate::message::UpdateMessage;

        match msg {
            UpdateMessage::CheckForUpdates => {
                self.update_status = UpdateStatus::Checking;
                let config = self.update_config.clone();
                Task::perform(
                    async move { update::check_for_update(&config).await },
                    |result| Message::Update(UpdateMessage::UpdateCheckResult(result)),
                )
            }
            UpdateMessage::UpdateCheckResult(result) => {
                match result {
                    UpdateCheckResult::UpdateAvailable(info) => {
                        // Check if user has skipped this version
                        if update::is_version_skipped(&info.new_version) {
                            self.update_status = UpdateStatus::Skipped(info.new_version);
                            tracing::info!("Update available but version was skipped by user");
                        } else {
                            tracing::info!(
                                "Update available: {} -> {}",
                                info.current_version,
                                info.new_version
                            );
                            self.update_status = UpdateStatus::Available(info.clone());
                            self.available_update = Some(info);
                        }
                    }
                    UpdateCheckResult::UpToDate => {
                        tracing::debug!("Application is up to date");
                        self.update_status = UpdateStatus::Idle;
                    }
                    UpdateCheckResult::Error(err) => {
                        tracing::warn!("Update check failed: {}", err);
                        self.update_status = UpdateStatus::Failed(err);
                    }
                }
                Task::none()
            }
            UpdateMessage::DownloadUpdate => {
                self.update_status = UpdateStatus::Downloading { progress: 0 };
                let config = self.update_config.clone();
                Task::perform(
                    async move { update::perform_update(&config).await },
                    |result| {
                        Message::Update(UpdateMessage::UpdateCompleted(
                            result.map(|r| r.new_version),
                        ))
                    },
                )
            }
            UpdateMessage::UpdateCompleted(result) => {
                match result {
                    Ok(new_version) => {
                        tracing::info!("Update completed successfully: {}", new_version);
                        self.update_status = UpdateStatus::Completed { new_version };
                    }
                    Err(err) => {
                        tracing::error!("Update failed: {}", err);
                        self.update_status = UpdateStatus::Failed(err);
                    }
                }
                Task::none()
            }
            UpdateMessage::DismissUpdate => {
                self.update_status = UpdateStatus::Dismissed;
                self.available_update = None;
                Task::none()
            }
            UpdateMessage::SkipVersion(version) => {
                if let Err(e) = update::skip_version(&version) {
                    tracing::warn!("Failed to skip version: {}", e);
                }
                self.update_status = UpdateStatus::Skipped(version);
                self.available_update = None;
                Task::none()
            }
            UpdateMessage::ShowUpdateBanner(info) => {
                self.update_status = UpdateStatus::Available(info.clone());
                self.available_update = Some(info);
                Task::none()
            }
        }
    }

    /// Handle core events from the backend.
    fn handle_core_event(&mut self, event: crate::message::CoreEvent) -> Task<Message> {
        if let Some(ref mut app) = self.app_state {
            match event {
                crate::message::CoreEvent::MessageReceived(message) => {
                    // Route to appropriate entity based on message's entity_id
                    let entity_id = message.entity_id.clone();
                    app.messages.entry(entity_id).or_default().push(message);
                }
                crate::message::CoreEvent::PeerConnected(peer) => {
                    app.network_info.peers.push(peer);
                }
                crate::message::CoreEvent::PeerDisconnected(four_words) => {
                    app.network_info
                        .peers
                        .retain(|p| p.four_words != four_words);
                }
                crate::message::CoreEvent::EntityUpdated(entity) => {
                    if let Some(idx) = app.entities.iter().position(|e| e.id == entity.id) {
                        app.entities[idx] = entity;
                    }
                }
                crate::message::CoreEvent::SyncCompleted(_entity_id, _count) => {
                    // Handle sync completion
                }
            }
        }
        Task::none()
    }

    /// Check if the user is authenticated.
    #[must_use]
    pub fn is_authenticated(&self) -> bool {
        self.app_state.is_some()
    }

    /// Get the current theme.
    #[must_use]
    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    /// Get the active modal.
    #[must_use]
    pub fn active_modal(&self) -> Option<&ModalType> {
        self.active_modal.as_ref()
    }

    /// Get the auth state.
    #[must_use]
    pub fn auth_state(&self) -> &AuthState {
        &self.auth_state
    }

    /// Get the app state.
    #[must_use]
    pub fn app_state(&self) -> Option<&AppState> {
        self.app_state.as_ref()
    }

    /// Get the pane grid state.
    #[must_use]
    pub fn panes(&self) -> &pane_grid::State<PaneType> {
        &self.panes
    }

    /// Get the detail pane.
    #[must_use]
    pub fn detail_pane(&self) -> pane_grid::Pane {
        self.detail_pane
    }

    /// Get the last error.
    #[must_use]
    pub fn last_error(&self) -> Option<&AppError> {
        self.last_error.as_ref()
    }

    /// Main view function for rendering the UI.
    #[must_use]
    pub fn view(&self) -> iced::Element<'_, Message> {
        use crate::views::{view_authentication, view_main};

        if self.is_authenticated() {
            if let Some(ref app_state) = self.app_state {
                view_main(
                    app_state,
                    &self.panes,
                    self.active_modal.as_ref(),
                    &self.modal_form_state,
                    &self.update_status,
                )
            } else {
                view_authentication(&self.auth_state)
            }
        } else {
            view_authentication(&self.auth_state)
        }
    }

    /// Subscriptions for real-time events.
    pub fn subscription(&self) -> iced::Subscription<Message> {
        let mut subscriptions = vec![];

        // Periodic tick for animations and updates (every second)
        subscriptions.push(iced::time::every(std::time::Duration::from_secs(1)).map(Message::Tick));

        // Call duration update (faster when in call)
        if let Some(ref app) = self.app_state
            && app.call_state.has_active_call()
        {
            subscriptions
                .push(iced::time::every(std::time::Duration::from_millis(100)).map(Message::Tick));
        }

        // Keyboard shortcuts for testing (only in demo mode)
        // Cmd+T: Send test message
        // Cmd+1: Switch to Home
        // Cmd+2: Switch to Contact Chat
        // Cmd+3: Switch to Channel Chat
        // Cmd+4: Create Organization
        // Cmd+5: Create Project
        // Cmd+6: Create Group
        // Cmd+7: Open Network Panel
        // Cmd+8: Toggle Sidebar Section
        #[cfg(feature = "demo")]
        {
            use crate::message::TestAction;
            subscriptions.push(keyboard::listen().filter_map(|event| {
                if let keyboard::Event::KeyPressed { key, modifiers, .. } = event
                    && modifiers.command()
                {
                    match key.as_ref() {
                        keyboard::Key::Character("t") => {
                            return Some(Message::TestAction(TestAction::SendTestMessage));
                        }
                        keyboard::Key::Character("1") => {
                            return Some(Message::TestAction(TestAction::SwitchToHome));
                        }
                        keyboard::Key::Character("2") => {
                            return Some(Message::TestAction(TestAction::SwitchToContactChat));
                        }
                        keyboard::Key::Character("3") => {
                            return Some(Message::TestAction(TestAction::SwitchToChannelChat));
                        }
                        keyboard::Key::Character("4") => {
                            return Some(Message::TestAction(TestAction::CreateOrganization));
                        }
                        keyboard::Key::Character("5") => {
                            return Some(Message::TestAction(TestAction::CreateProject));
                        }
                        keyboard::Key::Character("6") => {
                            return Some(Message::TestAction(TestAction::CreateGroup));
                        }
                        keyboard::Key::Character("7") => {
                            return Some(Message::TestAction(TestAction::OpenNetworkPanel));
                        }
                        keyboard::Key::Character("8") => {
                            return Some(Message::TestAction(TestAction::ToggleSidebarSection));
                        }
                        _ => {}
                    }
                }
                None
            }));
        }

        iced::Subscription::batch(subscriptions)
    }
}
