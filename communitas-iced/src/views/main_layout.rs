// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Main application layout using PaneGrid.
//!
//! Design: "Warm Digital Commons" - A rich forest-themed aesthetic with
//! deep greens, warm earth tones, and jade accents.

use crate::app::{AppState, Document, PaneType};
use crate::message::{
    AuthMessage, CallMessage, ChatMessageEvent, ContactMessage, CreateEntityContext, FileInfo,
    KanbanMessage, Message, ModalMessage, ModalType, NavigationMessage, NetworkMessage,
    SidebarMessage, StorageMessage,
};
use crate::state::{
    ActiveView, CallStatus, CardPriority, ChatMessage, ContactStatus, DetailTab, Entity,
    EntityType, KanbanCard, KanbanColumn, MemberRole, SidebarSection,
};
use crate::theme::{self, Palette};
use crate::update::UpdateStatus;
use crate::views::update_banner::view_update_banner;
use iced::widget::pane_grid::{self, PaneGrid};
use iced::widget::{
    Column, Row, Space, button, column, container, row, rule, scrollable, text, text_input,
};
use iced::{Alignment, Border, Color, Element, Length, Padding, Theme};

/// Modal form state passed from the app to the view.
#[derive(Debug, Clone, Default)]
pub struct ModalFormState {
    /// Entity name for create entity modal.
    pub entity_name: String,
    /// Entity description for create entity modal.
    pub entity_description: String,
    /// Contact name for add contact modal.
    pub contact_name: String,
    /// Contact four-words for add contact modal.
    pub contact_four_words: String,
    /// Card title for create card modal.
    pub card_title: String,
    /// Card description for create card modal.
    pub card_description: String,
    /// Card column for create card modal.
    pub card_column: String,
    /// Card priority for create/edit card modal.
    pub card_priority: CardPriority,
    /// Card assignee for create/edit card modal.
    pub card_assignee: String,
    /// Card ID when editing (None for create).
    pub editing_card_id: Option<String>,
}

/// Render the main application view.
#[must_use]
pub fn view_main<'a>(
    app_state: &'a AppState,
    panes: &'a pane_grid::State<PaneType>,
    active_modal: Option<&'a ModalType>,
    modal_form_state: &'a ModalFormState,
    update_status: &'a UpdateStatus,
) -> Element<'a, Message> {
    let pane_grid = PaneGrid::new(panes, |_pane, pane_type, _is_maximized| {
        let content: Element<'a, Message> = match pane_type {
            PaneType::Sidebar => view_sidebar(app_state),
            PaneType::Detail => view_detail(app_state),
            PaneType::Thread => view_thread_panel(app_state),
        };

        pane_grid::Content::new(content)
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .on_resize(10, Message::PaneResized);

    // Build main content with optional update banner
    let main_content = if let Some(update_banner) = view_update_banner(update_status) {
        column![
            view_profile_header(app_state),
            container(update_banner).padding([8, 16]),
            pane_grid,
        ]
    } else {
        column![view_profile_header(app_state), pane_grid,]
    };

    // Check for incoming calls first
    if let Some(ref incoming) = app_state.call_state.incoming_call {
        // Layer the incoming call overlay on top
        let overlay = view_incoming_call_overlay(incoming);
        return container(column![main_content, overlay])
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    // Check for active modal
    if let Some(modal_type) = active_modal {
        let modal_overlay = view_modal(modal_type, app_state, modal_form_state);
        return iced::widget::stack![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill),
            modal_overlay
        ]
        .into();
    }

    container(main_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render the profile header with the new design.
fn view_profile_header(app_state: &AppState) -> Element<'_, Message> {
    let identity_text = text(&app_state.four_words).size(11).color(Palette::STONE);

    let display_name = text(&app_state.display_name)
        .size(15)
        .color(Palette::TEXT_PRIMARY);

    let identity_column = column![display_name, identity_text].spacing(2);

    // Network toggle with new styling
    let network_status = if app_state.network_info.is_networking {
        format!("{} peers", app_state.network_info.peer_count())
    } else {
        "Offline".to_string()
    };

    let network_indicator_color = if app_state.network_info.is_networking {
        Palette::ONLINE
    } else {
        Palette::OFFLINE
    };

    let network_button = button(
        row![
            container(Space::new().width(8).height(8)).style(move |_t: &Theme| container::Style {
                background: Some(network_indicator_color.into()),
                border: Border::default().rounded(999),
                shadow: iced::Shadow {
                    color: Color::from_rgba(
                        network_indicator_color.r,
                        network_indicator_color.g,
                        network_indicator_color.b,
                        0.4
                    ),
                    offset: iced::Vector::new(0.0, 0.0),
                    blur_radius: 6.0,
                },
                ..Default::default()
            }),
            text(network_status).size(12).color(Palette::STONE),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Network(NetworkMessage::ToggleNetworking))
    .style(theme::ghost_button);

    // Logout button with subtle styling
    let logout_button = button(text("Logout").size(12).color(Palette::STONE))
        .on_press(Message::Auth(AuthMessage::Logout))
        .style(theme::ghost_button);

    let header_row = row![
        identity_column,
        Space::new().width(Length::Fill),
        network_button,
        logout_button,
    ]
    .spacing(16)
    .align_y(Alignment::Center)
    .padding(Padding::from([12, 20]));

    container(column![
        header_row,
        rule::horizontal(1).style(|_t: &Theme| rule::Style {
            color: theme::DIVIDER_LIGHT,
            radius: 0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
    ])
    .style(theme::header_bar_style)
    .into()
}

/// Render the sidebar with the forest theme.
fn view_sidebar(app_state: &AppState) -> Element<'_, Message> {
    let mut content: Vec<Element<'_, Message>> = Vec::new();

    // 1. My Organizations - entities the user owns
    content.push(view_sidebar_section(
        "My Organizations",
        SidebarSection::MyOrganizations,
        app_state
            .sidebar
            .is_section_expanded(SidebarSection::MyOrganizations),
    ));

    if app_state
        .sidebar
        .is_section_expanded(SidebarSection::MyOrganizations)
    {
        let my_orgs: Vec<&Entity> = app_state
            .entities
            .iter()
            .filter(|e| {
                e.entity_type == crate::state::EntityType::Organisation && e.role.is_owner()
            })
            .collect();

        for org in my_orgs {
            content.push(view_org_item(org, app_state));
        }
    }

    // 2. My Communities - entities the user is a member of (but not owner)
    content.push(Space::new().height(16).into());
    content.push(view_sidebar_section(
        "My Communities",
        SidebarSection::MyCommunities,
        app_state
            .sidebar
            .is_section_expanded(SidebarSection::MyCommunities),
    ));

    if app_state
        .sidebar
        .is_section_expanded(SidebarSection::MyCommunities)
    {
        let communities: Vec<&Entity> = app_state
            .entities
            .iter()
            .filter(|e| {
                e.entity_type == crate::state::EntityType::Organisation && !e.role.is_owner()
            })
            .collect();

        if communities.is_empty() {
            content.push(
                container(
                    text("Join a community to see it here")
                        .size(12)
                        .color(Palette::STONE),
                )
                .padding(Padding::from([8, 16]))
                .into(),
            );
        } else {
            for org in communities {
                content.push(view_org_item(org, app_state));
            }
        }
    }

    // 3. Personal - personal groups and spaces
    content.push(Space::new().height(16).into());
    content.push(view_sidebar_section(
        "Personal",
        SidebarSection::Personal,
        app_state
            .sidebar
            .is_section_expanded(SidebarSection::Personal),
    ));

    if app_state
        .sidebar
        .is_section_expanded(SidebarSection::Personal)
    {
        let personal: Vec<&Entity> = app_state
            .entities
            .iter()
            .filter(|e| e.is_personal)
            .collect();

        if personal.is_empty() {
            content.push(
                container(
                    text("Create a personal space")
                        .size(12)
                        .color(Palette::STONE),
                )
                .padding(Padding::from([8, 16]))
                .into(),
            );
        } else {
            for entity in personal {
                content.push(view_entity_item(entity, app_state, 1));
            }
        }
    }

    // 4. Direct Messages - contacts for 1:1 messaging
    content.push(Space::new().height(16).into());
    content.push(view_sidebar_section(
        "Direct Messages",
        SidebarSection::DirectMessages,
        app_state
            .sidebar
            .is_section_expanded(SidebarSection::DirectMessages),
    ));

    if app_state
        .sidebar
        .is_section_expanded(SidebarSection::DirectMessages)
    {
        // Contact search input
        content.push(
            container(
                text_input("Search contacts...", &app_state.sidebar.contact_search)
                    .on_input(|s| Message::Contact(ContactMessage::SearchChanged(s)))
                    .padding(Padding::from([6, 10]))
                    .size(12)
                    .style(theme::input_style_dark),
            )
            .padding(Padding::from([4, 8]))
            .into(),
        );

        // Filter contacts by search query
        let search_lower = app_state.sidebar.contact_search.to_lowercase();
        let filtered_contacts: Vec<_> = app_state
            .contacts
            .iter()
            .filter(|c| {
                if search_lower.is_empty() {
                    return true;
                }
                c.display_name.to_lowercase().contains(&search_lower)
                    || c.four_words
                        .as_ref()
                        .is_some_and(|fw| fw.to_lowercase().contains(&search_lower))
            })
            .collect();

        // Sort: favorites first, then by name
        let mut sorted_contacts = filtered_contacts;
        sorted_contacts.sort_by(|a, b| match (a.is_favorite, b.is_favorite) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a
                .display_name
                .to_lowercase()
                .cmp(&b.display_name.to_lowercase()),
        });

        if sorted_contacts.is_empty() {
            content.push(
                container(
                    text(if app_state.contacts.is_empty() {
                        "No contacts yet"
                    } else {
                        "No matching contacts"
                    })
                    .size(12)
                    .color(Palette::STONE),
                )
                .padding(Padding::from([8, 16]))
                .into(),
            );
        } else {
            for contact in sorted_contacts {
                content.push(view_contact_item(contact, app_state));
            }
        }
    }

    let sidebar_content = scrollable(
        column(content)
            .spacing(4)
            .padding(Padding::from([16, 16]))
            .width(Length::Fill),
    )
    .height(Length::Fill)
    .style(theme::scrollbar_style_dark);

    container(sidebar_content)
        .width(280)
        .height(Length::Fill)
        .style(theme::sidebar_container)
        .into()
}

/// Render a sidebar section header with the new design.
fn view_sidebar_section(
    title: &str,
    section: SidebarSection,
    is_expanded: bool,
) -> Element<'_, Message> {
    let arrow = if is_expanded { "▾" } else { "▸" };

    let toggle_btn = button(
        row![
            text(arrow).size(12).color(Palette::SAGE),
            text(title.to_uppercase()).size(10).color(Palette::SAGE),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .on_press(Message::Sidebar(SidebarMessage::ToggleSection(section)))
    .style(theme::ghost_button_dark)
    .width(Length::Fill);

    // Add button based on section type
    let add_btn: Element<'_, Message> = match section {
        SidebarSection::MyOrganizations => button(text("+").size(12).color(Palette::JADE))
            .on_press(Message::Sidebar(SidebarMessage::CreateEntity(
                CreateEntityContext {
                    parent_org_id: None,
                    entity_type: EntityType::Organisation,
                },
            )))
            .style(theme::ghost_button_dark)
            .padding(Padding::from([2, 6]))
            .into(),
        SidebarSection::Personal => {
            button(text("+").size(12).color(Palette::JADE))
                .on_press(Message::Sidebar(SidebarMessage::CreateEntity(
                    CreateEntityContext {
                        parent_org_id: None,
                        entity_type: EntityType::Group, // Personal spaces are groups
                    },
                )))
                .style(theme::ghost_button_dark)
                .padding(Padding::from([2, 6]))
                .into()
        }
        SidebarSection::DirectMessages => button(text("+").size(12).color(Palette::JADE))
            .on_press(Message::Contact(ContactMessage::AddContactPressed))
            .style(theme::ghost_button_dark)
            .padding(Padding::from([2, 6]))
            .into(),
        SidebarSection::MyCommunities => {
            // No add button for communities (user joins, doesn't create)
            Space::new().width(0).into()
        }
    };

    row![toggle_btn, add_btn].align_y(Alignment::Center).into()
}

/// Render an organization item with its children.
fn view_org_item<'a>(org: &'a Entity, app_state: &'a AppState) -> Element<'a, Message> {
    let is_expanded = app_state.sidebar.is_org_expanded(&org.id);
    let is_selected = app_state.sidebar.is_selected(&org.id);

    let arrow = if is_expanded { "▾" } else { "▸" };
    let color = Palette::ORGANISATION;

    let org_id = org.id.clone();
    let org_id_for_add = org.id.clone();

    let org_row = row![
        button(
            row![
                text(arrow).size(12).color(Palette::SAGE),
                container(Space::new().width(10).height(10)).style(move |_t: &Theme| {
                    container::Style {
                        background: Some(color.into()),
                        border: Border::default().rounded(3),
                        ..Default::default()
                    }
                }),
                text(&org.name).size(14).color(if is_selected {
                    Palette::JADE
                } else {
                    Palette::CREAM
                }),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .on_press(Message::Sidebar(SidebarMessage::ToggleOrg(org_id)))
        .style(move |t: &Theme, status| theme::sidebar_item_button(t, status, is_selected))
        .width(Length::Fill),
        // Add button for creating child entities
        button(text("+").size(14).color(Palette::JADE))
            .on_press(Message::Sidebar(SidebarMessage::CreateEntity(
                CreateEntityContext {
                    parent_org_id: Some(org_id_for_add),
                    entity_type: EntityType::Channel, // Default, user can change in modal
                },
            )))
            .style(theme::ghost_button_dark)
            .padding(Padding::from([2, 8])),
    ]
    .align_y(Alignment::Center);

    let mut items: Vec<Element<'_, Message>> = vec![org_row.into()];

    if is_expanded {
        // Add child entities (projects, channels, groups)
        let children: Vec<&Entity> = app_state
            .entities
            .iter()
            .filter(|e| e.parent_org_id.as_ref() == Some(&org.id))
            .collect();

        for child in children {
            items.push(view_entity_item(child, app_state, 1));
        }
    }

    column(items).spacing(4).into()
}

/// Render an entity item with proper indentation, role badge, and member count.
fn view_entity_item<'a>(
    entity: &'a Entity,
    app_state: &'a AppState,
    indent: u16,
) -> Element<'a, Message> {
    let is_selected = app_state.sidebar.is_selected(&entity.id);

    let (icon, color) = match entity.entity_type {
        crate::state::EntityType::Project => ("●", Palette::PROJECT),
        crate::state::EntityType::Channel => ("#", Palette::CHANNEL),
        crate::state::EntityType::Group => ("◉", Palette::GROUP),
        crate::state::EntityType::Organisation => ("◆", Palette::ORGANISATION),
    };

    let entity_clone = entity.clone();
    let role = entity.role;
    let member_count = entity.member_count;
    let is_read_only = entity.is_read_only();

    // Build row content dynamically
    let mut row_content: Vec<Element<'a, Message>> = vec![
        text(icon).size(12).color(color).into(),
        text(&entity.name)
            .size(13)
            .color(if is_selected {
                Palette::JADE
            } else {
                Palette::CREAM
            })
            .into(),
    ];

    // Add role badge if not a regular member
    if role != MemberRole::Member {
        row_content.push(view_role_badge(role));
    }

    // Add read-only indicator for guests
    if is_read_only {
        row_content.push(
            text("👁")
                .size(10)
                .color(Color::from_rgb(0.5, 0.5, 0.6))
                .into(),
        );
    }

    // Add spacer
    row_content.push(Space::new().width(Length::Fill).into());

    // Add member count
    if member_count > 0 {
        row_content.push(
            text(format!("{}", member_count))
                .size(10)
                .color(Color::from_rgb(0.5, 0.5, 0.5))
                .into(),
        );
    }

    let item = button(
        Row::from_vec(row_content)
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .on_press(Message::Sidebar(SidebarMessage::EntityClicked(
        entity_clone,
    )))
    .style(move |t: &Theme, status| theme::sidebar_item_button(t, status, is_selected))
    .width(Length::Fill);

    let left_padding = f32::from(indent) * 20.0;
    container(item)
        .padding(Padding {
            left: left_padding,
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
}

/// Render a role badge with emoji icon (👑 Owner, 🛡️ Admin, 👁️ Guest).
/// Member role is not displayed as a badge.
fn view_role_badge(role: MemberRole) -> Element<'static, Message> {
    let badge_color = role.color();
    let icon = role.icon();

    container(text(icon).size(11))
        .padding(Padding::from([1, 3]))
        .style(move |_t: &Theme| container::Style {
            background: Some(badge_color.scale_alpha(0.2).into()),
            border: Border::default().rounded(3),
            ..Default::default()
        })
        .into()
}

/// Render a contact item with status indicator, favorite star, and local badge.
fn view_contact_item<'a>(
    contact: &'a crate::state::Contact,
    app_state: &'a AppState,
) -> Element<'a, Message> {
    let status_color = match contact.status {
        ContactStatus::Online => Palette::ONLINE,
        ContactStatus::Away => Palette::AWAY,
        ContactStatus::Offline => Palette::OFFLINE,
    };

    let display = if contact.display_name.is_empty() {
        contact.short_identity()
    } else {
        contact.display_name.clone()
    };

    let is_selected = contact
        .four_words
        .as_ref()
        .is_some_and(|fw| app_state.sidebar.is_selected(fw));
    let contact_clone = contact.clone();
    let contact_for_detail = contact.clone();
    let contact_id_fav = contact.id.clone();
    let is_favorite = contact.is_favorite;
    let is_local = contact.is_local_only;

    // Build row elements
    let mut row_content: Vec<Element<'a, Message>> = vec![
        // Status indicator
        container(Space::new().width(8).height(8))
            .style(move |_t: &Theme| container::Style {
                background: Some(status_color.into()),
                border: Border::default().rounded(999),
                shadow: iced::Shadow {
                    color: Color::from_rgba(status_color.r, status_color.g, status_color.b, 0.4),
                    offset: iced::Vector::new(0.0, 0.0),
                    blur_radius: 4.0,
                },
                ..Default::default()
            })
            .into(),
        // Name
        text(display)
            .size(13)
            .color(if is_selected {
                Palette::JADE
            } else {
                Palette::CREAM
            })
            .into(),
    ];

    // Add local badge
    if is_local {
        row_content.push(
            container(text("Local").size(9).color(Palette::STONE))
                .padding(Padding::from([2, 4]))
                .style(|_t: &Theme| container::Style {
                    background: Some(Palette::STONE.scale_alpha(0.2).into()),
                    border: Border::default().rounded(3),
                    ..Default::default()
                })
                .into(),
        );
    }

    // Add spacer to push favorite star to the right
    row_content.push(Space::new().width(Length::Fill).into());

    // Favorite star button
    let star_color = if is_favorite {
        Palette::AMBER
    } else {
        Palette::STONE.scale_alpha(0.3)
    };
    row_content.push(
        button(
            text(if is_favorite { "★" } else { "☆" })
                .size(14)
                .color(star_color),
        )
        .on_press(Message::Contact(ContactMessage::ToggleFavorite(
            contact_id_fav,
        )))
        .style(theme::ghost_button_dark)
        .padding(Padding::from([0, 4]))
        .into(),
    );

    // Info button to show detail
    row_content.push(
        button(text("i").size(10).color(Palette::STONE))
            .on_press(Message::Contact(ContactMessage::ContactSelected(
                contact_for_detail,
            )))
            .style(theme::ghost_button_dark)
            .padding(Padding::from([2, 6]))
            .into(),
    );

    button(
        Row::from_vec(row_content)
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .on_press(Message::Sidebar(SidebarMessage::ContactClicked(
        contact_clone,
    )))
    .style(move |t: &Theme, status| theme::sidebar_item_button(t, status, is_selected))
    .width(Length::Fill)
    .into()
}

/// Render the detail pane.
fn view_detail(app_state: &AppState) -> Element<'_, Message> {
    match &app_state.active_view {
        ActiveView::Home => view_home(),
        ActiveView::Chat {
            entity_id,
            entity_name,
            ..
        } => view_chat(app_state, entity_id, entity_name),
        ActiveView::ContactChat {
            four_words,
            display_name,
        } => view_contact_chat(app_state, four_words, display_name.as_deref()),
        ActiveView::Drive { entity_id, .. } => view_drive(app_state, entity_id),
        ActiveView::Call { .. } => view_call(app_state),
        ActiveView::Project { project_id } => view_drive(app_state, project_id),
        ActiveView::NetworkPanel => view_network_panel(app_state),
    }
}

/// Render the home view with the new design.
fn view_home<'a>() -> Element<'a, Message> {
    let welcome = text("Welcome to Communitas")
        .size(28)
        .color(Palette::TEXT_PRIMARY);

    let subtitle = text("Select an entity or contact from the sidebar to get started.")
        .size(14)
        .color(Palette::STONE);

    // Add a subtle decorative element
    let icon = text("◈").size(48).color(Palette::JADE.scale_alpha(0.3));

    container(
        column![
            icon,
            Space::new().height(16),
            welcome,
            Space::new().height(8),
            subtitle,
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(theme::detail_container)
    .into()
}

/// Render the chat view with improved design.
fn view_chat(
    app_state: &AppState,
    entity_id: &str,
    entity_name: &str,
) -> Element<'static, Message> {
    let name = entity_name.to_string();
    let eid = entity_id.to_string();

    // Find the entity to check permissions
    let entity = app_state.entities.iter().find(|e| e.id == entity_id);
    let is_read_only = entity.is_some_and(|e| e.is_read_only());
    let role = entity.map(|e| e.role).unwrap_or(MemberRole::Member);

    // Build header content
    let mut header_content: Vec<Element<'static, Message>> =
        vec![text(name).size(18).color(Palette::TEXT_PRIMARY).into()];

    // Add role badge if not a regular member
    if role != MemberRole::Member {
        header_content.push(view_role_badge(role));
    }

    header_content.push(Space::new().width(Length::Fill).into());
    header_content.push(view_tab_bar(app_state));

    let header = container(
        Row::from_vec(header_content)
            .spacing(8)
            .align_y(Alignment::Center)
            .padding(Padding::from([16, 20])),
    );

    // Tab content (pass read-only status to kanban)
    let tab_content = match app_state.detail_tab {
        DetailTab::Chat => view_messages(app_state, &eid),
        DetailTab::Board => view_kanban_board(app_state, &eid, is_read_only),
        DetailTab::Documents => view_documents(app_state, &eid),
        DetailTab::Drive => view_drive(app_state, &eid),
        DetailTab::Details => view_details_placeholder(),
    };

    let mut content_items: Vec<Element<'static, Message>> = Vec::new();

    // Add read-only banner if applicable
    if is_read_only {
        content_items.push(view_read_only_banner());
    }

    content_items.push(header.into());
    content_items.push(
        rule::horizontal(1)
            .style(|_t: &Theme| rule::Style {
                color: theme::DIVIDER_LIGHT,
                radius: 0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            })
            .into(),
    );
    content_items.push(tab_content);

    container(column(content_items))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::detail_container)
        .into()
}

/// Render a read-only banner for guests.
fn view_read_only_banner() -> Element<'static, Message> {
    container(
        row![
            text("👁").size(14),
            text("View Only").size(12).color(Color::WHITE),
            Space::new().width(8),
            text("You have read-only access to this entity.")
                .size(11)
                .color(Color::from_rgb(0.9, 0.9, 0.9)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(Padding::from([8, 16]))
    .style(|_t: &Theme| container::Style {
        background: Some(Color::from_rgb(0.6, 0.5, 0.3).into()),
        ..Default::default()
    })
    .into()
}

/// Render the tab bar with improved design.
fn view_tab_bar(app_state: &AppState) -> Element<'static, Message> {
    let tabs = vec![
        (DetailTab::Chat, "Chat"),
        (DetailTab::Board, "Board"),
        (DetailTab::Documents, "Docs"),
        (DetailTab::Drive, "Drive"),
    ];

    let current_tab = app_state.detail_tab;
    let tab_buttons: Vec<Element<'static, Message>> = tabs
        .into_iter()
        .map(|(tab, label)| {
            let is_selected = current_tab == tab;
            button(text(label).size(12))
                .on_press(Message::Navigate(NavigationMessage::SelectTab(tab)))
                .style(move |t: &Theme, status| theme::tab_button(t, status, is_selected))
                .padding(Padding::from([6, 12]))
                .into()
        })
        .collect();

    row(tab_buttons).spacing(4).into()
}

/// Render the messages list with improved bubbles.
fn view_messages(app_state: &AppState, entity_id: &str) -> Element<'static, Message> {
    let messages = app_state.messages.get(entity_id);

    let message_list: Element<'static, Message> = if let Some(msgs) = messages {
        let items: Vec<Element<'static, Message>> = msgs
            .iter()
            .map(|msg| view_message_bubble(msg, app_state))
            .collect();

        scrollable(column(items).spacing(12).padding(Padding::from([16, 20])))
            .height(Length::Fill)
            .style(theme::scrollbar_style)
            .into()
    } else {
        container(
            column![
                text("◈").size(32).color(Palette::STONE.scale_alpha(0.3)),
                Space::new().height(8),
                text("No messages yet").size(14).color(Palette::STONE),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    };

    // Composer
    let composer = view_composer(app_state);

    column![
        message_list,
        rule::horizontal(1).style(|_t: &Theme| rule::Style {
            color: theme::DIVIDER_LIGHT,
            radius: 0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        composer,
    ]
    .height(Length::Fill)
    .into()
}

/// Render a message bubble with improved design.
fn view_message_bubble(
    msg: &crate::state::ChatMessage,
    app_state: &AppState,
) -> Element<'static, Message> {
    let is_own = msg.author == app_state.four_words;
    let is_deleted = msg.is_deleted;
    let sender = msg.short_author();
    let time_str = msg.formatted_time();
    let text_content = msg.text.clone();
    let message_id = msg.id.clone();
    let msg_clone = msg.clone();

    // Header with optional edited indicator
    let header: Element<'static, Message> = {
        let mut header_items: Vec<Element<'static, Message>> = vec![
            text(sender)
                .size(12)
                .color(if is_own {
                    Palette::JADE
                } else {
                    Palette::ORGANISATION
                })
                .into(),
            Space::new().width(8).into(),
            text(time_str).size(10).color(Palette::STONE).into(),
        ];

        if msg.is_edited && !is_deleted {
            header_items.push(Space::new().width(6).into());
            header_items.push(text("(edited)").size(10).color(Palette::STONE).into());
        }

        Row::from_vec(header_items).into()
    };

    // Message content (greyed out if deleted)
    let content: Element<'static, Message> = if is_deleted {
        text(text_content)
            .size(14)
            .color(Palette::STONE)
            .style(|_t: &Theme| text::Style {
                ..Default::default()
            })
            .into()
    } else {
        text(text_content)
            .size(14)
            .color(Palette::TEXT_PRIMARY)
            .into()
    };

    // Reactions display
    let reactions_row: Element<'static, Message> = if msg.reactions.is_empty() || is_deleted {
        Space::new().height(0).into()
    } else {
        let reaction_elements: Vec<Element<'static, Message>> = msg
            .sorted_reactions()
            .into_iter()
            .map(|(emoji, count)| {
                let msg_id = message_id.clone();
                let emoji_display = emoji.clone();
                let emoji_for_msg = emoji.clone();
                let has_reacted = msg.has_reacted(&emoji, &app_state.four_words);
                let bg_color = if has_reacted {
                    Palette::JADE.scale_alpha(0.2)
                } else {
                    Palette::STONE.scale_alpha(0.1)
                };

                button(
                    row![
                        text(emoji_display).size(12),
                        text(format!(" {count}")).size(11).color(Palette::STONE)
                    ]
                    .spacing(0)
                    .align_y(Alignment::Center),
                )
                .on_press(if has_reacted {
                    Message::Chat(ChatMessageEvent::RemoveReaction {
                        message_id: msg_id,
                        emoji: emoji_for_msg,
                    })
                } else {
                    Message::Chat(ChatMessageEvent::AddReaction {
                        message_id: msg_id,
                        emoji: emoji_for_msg,
                    })
                })
                .padding(Padding::from([2, 6]))
                .style(move |_t: &Theme, _status| button::Style {
                    background: Some(bg_color.into()),
                    border: Border::default().rounded(12),
                    text_color: Palette::TEXT_PRIMARY,
                    ..Default::default()
                })
                .into()
            })
            .collect();

        Row::from_vec(reaction_elements)
            .spacing(4)
            .padding(Padding::from([4, 0]))
            .into()
    };

    // Quick reaction buttons (emoji picker simplified)
    let quick_reactions: Element<'static, Message> = if is_deleted {
        Space::new().height(0).into()
    } else {
        let emojis = ["👍", "❤️", "😂", "🎉"];
        let buttons: Vec<Element<'static, Message>> = emojis
            .iter()
            .map(|emoji| {
                let msg_id = message_id.clone();
                let emoji_str = (*emoji).to_string();
                button(text(*emoji).size(12))
                    .on_press(Message::Chat(ChatMessageEvent::AddReaction {
                        message_id: msg_id,
                        emoji: emoji_str,
                    }))
                    .padding(Padding::from([2, 4]))
                    .style(theme::ghost_button)
                    .into()
            })
            .collect();

        Row::from_vec(buttons).spacing(2).into()
    };

    // Action buttons for own messages
    let actions: Element<'static, Message> = if is_own && !is_deleted {
        row![
            button(text("✏️").size(10))
                .on_press(Message::Chat(ChatMessageEvent::StartEdit(
                    msg_clone.clone()
                )))
                .padding(Padding::from([2, 4]))
                .style(theme::ghost_button),
            button(text("🗑️").size(10))
                .on_press(Message::Chat(ChatMessageEvent::DeleteMessagePressed(
                    message_id.clone()
                )))
                .padding(Padding::from([2, 4]))
                .style(theme::ghost_button),
        ]
        .spacing(2)
        .into()
    } else {
        Space::new().width(0).into()
    };

    // Combine header with actions
    let header_row: Element<'static, Message> =
        row![header, Space::new().width(Length::Fill), actions]
            .align_y(Alignment::Center)
            .into();

    let bubble_content: Element<'static, Message> = column![
        header_row,
        Space::new().height(4),
        content,
        reactions_row,
        quick_reactions,
    ]
    .spacing(0)
    .into();

    let bubble = container(bubble_content)
        .padding(Padding::from([12, 16]))
        .max_width(480)
        .style(move |_t: &Theme| {
            if is_deleted {
                container::Style {
                    background: Some(Palette::STONE.scale_alpha(0.1).into()),
                    border: Border::default().rounded(16),
                    ..Default::default()
                }
            } else if is_own {
                container::Style {
                    background: Some(Color::from_rgb(0.878, 0.941, 0.906).into()),
                    border: Border::default().rounded(16),
                    ..Default::default()
                }
            } else {
                container::Style {
                    background: Some(Color::from_rgb(0.945, 0.940, 0.930).into()),
                    border: Border::default().rounded(16),
                    ..Default::default()
                }
            }
        });

    let alignment = if is_own {
        Alignment::End
    } else {
        Alignment::Start
    };

    container(bubble)
        .width(Length::Fill)
        .align_x(alignment)
        .into()
}

/// Render the message composer with improved design.
fn view_composer(app_state: &AppState) -> Element<'static, Message> {
    let compose_text = app_state.compose_text.clone();
    let is_empty = compose_text.trim().is_empty();

    let input = text_input("Type a message...", &compose_text)
        .on_input(|s| Message::Chat(ChatMessageEvent::ComposeChanged(s)))
        .on_submit(Message::Chat(ChatMessageEvent::SendMessage))
        .width(Length::Fill)
        .padding(Padding::from([12, 16]))
        .style(theme::input_style);

    let send_button = button(text("Send").size(14))
        .on_press_maybe(if is_empty {
            None
        } else {
            Some(Message::Chat(ChatMessageEvent::SendMessage))
        })
        .padding(Padding::from([10, 20]))
        .style(theme::primary_button);

    container(
        row![input, send_button]
            .spacing(12)
            .align_y(Alignment::Center)
            .padding(Padding::from([16, 20])),
    )
    .width(Length::Fill)
    .style(|_t: &Theme| container::Style {
        background: Some(Palette::WARM_WHITE.into()),
        ..Default::default()
    })
    .into()
}

/// Render the thread panel with improved design.
fn view_thread_panel(app_state: &AppState) -> Element<'_, Message> {
    if let Some(ref thread) = app_state.thread_state {
        let header = row![
            text("Thread").size(16).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Chat(ChatMessageEvent::CloseThread))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([16, 20]));

        // Parent message
        let parent = view_message_bubble(&thread.parent_message, app_state);

        // Replies
        let replies: Vec<Element<'_, Message>> = thread
            .replies
            .iter()
            .map(|msg| view_message_bubble(msg, app_state))
            .collect();

        let reply_list = scrollable(column(replies).spacing(12).padding(Padding::from([16, 20])))
            .height(Length::Fill)
            .style(theme::scrollbar_style);

        // Thread composer
        let input = text_input("Reply...", &app_state.thread_compose_text)
            .on_input(|s| Message::Chat(ChatMessageEvent::ThreadComposeChanged(s)))
            .on_submit(Message::Chat(ChatMessageEvent::SendThreadReply))
            .width(Length::Fill)
            .padding(Padding::from([10, 14]))
            .style(theme::input_style);

        let send_btn = button(text("Reply").size(13))
            .on_press_maybe(if app_state.thread_compose_text.trim().is_empty() {
                None
            } else {
                Some(Message::Chat(ChatMessageEvent::SendThreadReply))
            })
            .padding(Padding::from([8, 16]))
            .style(theme::primary_button);

        let composer = container(
            row![input, send_btn]
                .spacing(8)
                .align_y(Alignment::Center)
                .padding(Padding::from([12, 16])),
        );

        container(column![
            header,
            rule::horizontal(1).style(|_t: &Theme| rule::Style {
                color: theme::DIVIDER_LIGHT,
                radius: 0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            }),
            parent,
            rule::horizontal(1).style(|_t: &Theme| rule::Style {
                color: theme::DIVIDER_LIGHT,
                radius: 0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            }),
            reply_list,
            rule::horizontal(1).style(|_t: &Theme| rule::Style {
                color: theme::DIVIDER_LIGHT,
                radius: 0.into(),
                fill_mode: rule::FillMode::Full,
                snap: true,
            }),
            composer,
        ])
        .width(320)
        .height(Length::Fill)
        .style(theme::thread_panel_style)
        .into()
    } else {
        container(Space::new().width(0)).into()
    }
}

/// Render the contact chat view.
fn view_contact_chat(
    app_state: &AppState,
    four_words: &str,
    display_name: Option<&str>,
) -> Element<'static, Message> {
    let name = display_name.unwrap_or(four_words).to_string();
    let fw = four_words.to_string();
    let entity_id = four_words.to_string();

    let header = container(
        row![
            text(name).size(18).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(row![text("📞").size(14), text("Call").size(14),].spacing(6))
                .on_press(Message::Call(CallMessage::Initiate {
                    four_words: fw,
                    has_video: false,
                }))
                .padding(Padding::from([8, 16]))
                .style(theme::secondary_button),
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([16, 20])),
    );

    let messages = view_messages(app_state, &entity_id);

    container(column![
        header,
        rule::horizontal(1).style(|_t: &Theme| rule::Style {
            color: theme::DIVIDER_LIGHT,
            radius: 0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }),
        messages,
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::detail_container)
    .into()
}

/// Render the call view with improved design including participants and device selection.
fn view_call(app_state: &AppState) -> Element<'static, Message> {
    if let Some(ref call) = app_state.call_state.active_call {
        let peer_name = call
            .peer_display_name
            .clone()
            .unwrap_or_else(|| call.peer_four_words.clone());

        let status_text = call.status.display().to_string();
        let duration = if call.status == CallStatus::Connected {
            call.formatted_duration()
        } else {
            String::new()
        };

        // Main video area
        let video_placeholder = container(
            column![
                text("◉").size(64).color(Palette::JADE.scale_alpha(0.5)),
                Space::new().height(16),
                text(peer_name.clone()).size(28).color(Palette::CREAM),
            ]
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::CALL_BG.into()),
            ..Default::default()
        });

        // Participants panel
        let participants_panel = view_call_participants(app_state, &peer_name);

        // Controls bar with device selection
        let controls = view_call_controls(call);
        let device_selection = view_device_selection(app_state);

        let controls_area = container(
            column![
                row![device_selection,]
                    .spacing(16)
                    .align_y(Alignment::Center),
                Space::new().height(16),
                row![
                    text(status_text).size(14).color(Palette::SAGE),
                    Space::new().width(16),
                    text(duration).size(14).color(Palette::CREAM),
                ]
                .align_y(Alignment::Center),
                Space::new().height(12),
                controls,
            ]
            .align_x(Alignment::Center)
            .spacing(4)
            .padding(16),
        )
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::CALL_CONTROLS_BG.into()),
            ..Default::default()
        })
        .width(Length::Fill);

        // Main layout: video area with participants sidebar
        let main_content = row![
            container(video_placeholder)
                .width(Length::FillPortion(3))
                .height(Length::Fill),
            container(participants_panel)
                .width(Length::FillPortion(1))
                .height(Length::Fill)
                .style(|_t: &Theme| container::Style {
                    background: Some(Palette::DEEP_FOREST.scale_alpha(0.9).into()),
                    border: Border::default().color(Palette::BORDER).width(1).rounded(0),
                    ..Default::default()
                }),
        ];

        container(column![
            container(main_content).height(Length::FillPortion(4)),
            controls_area,
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    } else {
        view_home()
    }
}

/// Render the participants panel for a call.
fn view_call_participants(app_state: &AppState, peer_name: &str) -> Element<'static, Message> {
    let participants = &app_state.call_state.participants;

    let header = row![
        text("Participants").size(14).color(Palette::TEXT_PRIMARY),
        Space::new().width(Length::Fill),
        text(format!("{}", participants.len() + 1)) // +1 for self
            .size(12)
            .color(Palette::STONE),
    ]
    .align_y(Alignment::Center)
    .padding(Padding::from([12, 16]));

    // Self (you)
    let self_row = row![
        container(text("◉").size(16).color(Palette::JADE))
            .width(32)
            .height(32)
            .center_x(32)
            .center_y(32)
            .style(|_t: &Theme| container::Style {
                background: Some(Palette::JADE.scale_alpha(0.2).into()),
                border: Border::default().rounded(16),
                ..Default::default()
            }),
        Space::new().width(8),
        column![
            text("You").size(13).color(Palette::TEXT_PRIMARY),
            text("(Host)").size(11).color(Palette::STONE),
        ]
        .spacing(2),
        Space::new().width(Length::Fill),
        row![
            text(
                if app_state
                    .call_state
                    .active_call
                    .as_ref()
                    .is_some_and(|c| c.is_audio_enabled)
                {
                    "🔊"
                } else {
                    "🔇"
                }
            )
            .size(12),
            Space::new().width(4),
            text(
                if app_state
                    .call_state
                    .active_call
                    .as_ref()
                    .is_some_and(|c| c.is_video_enabled)
                {
                    "📹"
                } else {
                    "📷"
                }
            )
            .size(12),
        ]
        .spacing(4),
    ]
    .align_y(Alignment::Center)
    .padding(Padding::from([8, 16]));

    // Remote peer (if not in participants list)
    let peer_name_owned = peer_name.to_string();
    let peer_row = row![
        container(text("◉").size(16).color(Palette::SAGE))
            .width(32)
            .height(32)
            .center_x(32)
            .center_y(32)
            .style(|_t: &Theme| container::Style {
                background: Some(Palette::SAGE.scale_alpha(0.2).into()),
                border: Border::default().rounded(16),
                ..Default::default()
            }),
        Space::new().width(8),
        column![text(peer_name_owned).size(13).color(Palette::TEXT_PRIMARY),].spacing(2),
        Space::new().width(Length::Fill),
        row![
            text("🔊").size(12),
            Space::new().width(4),
            text("📹").size(12),
        ]
        .spacing(4),
    ]
    .align_y(Alignment::Center)
    .padding(Padding::from([8, 16]));

    // Additional participants
    let participant_rows: Vec<Element<'static, Message>> = participants
        .iter()
        .map(|p| {
            let name = p
                .display_name
                .clone()
                .unwrap_or_else(|| p.four_words.clone());
            let audio_icon = if p.audio_enabled { "🔊" } else { "🔇" };
            let video_icon = if p.video_enabled { "📹" } else { "📷" };

            row![
                container(text("◉").size(16).color(Palette::SAGE))
                    .width(32)
                    .height(32)
                    .center_x(32)
                    .center_y(32)
                    .style(|_t: &Theme| container::Style {
                        background: Some(Palette::SAGE.scale_alpha(0.2).into()),
                        border: Border::default().rounded(16),
                        ..Default::default()
                    }),
                Space::new().width(8),
                column![text(name).size(13).color(Palette::TEXT_PRIMARY),].spacing(2),
                Space::new().width(Length::Fill),
                row![
                    text(audio_icon).size(12),
                    Space::new().width(4),
                    text(video_icon).size(12),
                ]
                .spacing(4),
            ]
            .align_y(Alignment::Center)
            .padding(Padding::from([8, 16]))
            .into()
        })
        .collect();

    let mut content_items: Vec<Element<'static, Message>> =
        vec![header.into(), self_row.into(), peer_row.into()];
    content_items.extend(participant_rows);

    scrollable(Column::from_vec(content_items).spacing(0))
        .height(Length::Fill)
        .into()
}

/// Render device selection dropdowns.
fn view_device_selection(app_state: &AppState) -> Element<'static, Message> {
    let devices = &app_state.call_state.devices;

    // Audio input dropdown
    let audio_input_label = devices
        .audio_inputs
        .iter()
        .find(|d| Some(&d.id) == devices.selected_audio_input.as_ref())
        .map(|d| d.name.clone())
        .unwrap_or_else(|| "Select Microphone".to_string());

    let _audio_input_options: Vec<Element<'static, Message>> = devices
        .audio_inputs
        .iter()
        .map(|d| {
            let device_id = d.id.clone();
            let device_name = d.name.clone();
            let is_selected = Some(&d.id) == devices.selected_audio_input.as_ref();
            button(
                row![
                    text(if is_selected { "✓ " } else { "  " })
                        .size(12)
                        .color(Palette::JADE),
                    text(device_name).size(12).color(Palette::TEXT_PRIMARY),
                ]
                .spacing(4),
            )
            .on_press(Message::Call(CallMessage::SelectAudioInput(device_id)))
            .padding(Padding::from([6, 12]))
            .width(Length::Fill)
            .style(theme::ghost_button)
            .into()
        })
        .collect();

    let audio_input_menu = column![
        text("🎤").size(14).color(Palette::CREAM),
        text(truncate_text(&audio_input_label, 15))
            .size(11)
            .color(Palette::STONE),
    ]
    .spacing(2)
    .align_x(Alignment::Center);

    // Video dropdown
    let video_label = devices
        .video_devices
        .iter()
        .find(|d| Some(&d.id) == devices.selected_video.as_ref())
        .map(|d| d.name.clone())
        .unwrap_or_else(|| "Select Camera".to_string());

    let video_menu = column![
        text("📹").size(14).color(Palette::CREAM),
        text(truncate_text(&video_label, 15))
            .size(11)
            .color(Palette::STONE),
    ]
    .spacing(2)
    .align_x(Alignment::Center);

    // Audio output dropdown
    let audio_output_label = devices
        .audio_outputs
        .iter()
        .find(|d| Some(&d.id) == devices.selected_audio_output.as_ref())
        .map(|d| d.name.clone())
        .unwrap_or_else(|| "Select Speaker".to_string());

    let audio_output_menu = column![
        text("🔊").size(14).color(Palette::CREAM),
        text(truncate_text(&audio_output_label, 15))
            .size(11)
            .color(Palette::STONE),
    ]
    .spacing(2)
    .align_x(Alignment::Center);

    row![
        container(audio_input_menu).padding(Padding::from([4, 12])),
        container(video_menu).padding(Padding::from([4, 12])),
        container(audio_output_menu).padding(Padding::from([4, 12])),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

/// Truncate text to a maximum length with ellipsis.
fn truncate_text(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Render call controls with improved button styling.
fn view_call_controls(call: &crate::state::CallInfo) -> Element<'static, Message> {
    let mute_label = if call.is_audio_enabled {
        "🔊"
    } else {
        "🔇"
    };
    let video_label = if call.is_video_enabled {
        "📹"
    } else {
        "📷"
    };
    let screen_label = if call.is_screen_sharing {
        "⏹"
    } else {
        "🖥"
    };

    row![
        button(text(mute_label).size(20))
            .on_press(Message::Call(CallMessage::ToggleMute))
            .padding(Padding::from([12, 16]))
            .style(theme::ghost_button_dark),
        button(text(video_label).size(20))
            .on_press(Message::Call(CallMessage::ToggleVideo))
            .padding(Padding::from([12, 16]))
            .style(theme::ghost_button_dark),
        button(text(screen_label).size(20))
            .on_press(Message::Call(CallMessage::ToggleScreenShare))
            .padding(Padding::from([12, 16]))
            .style(theme::ghost_button_dark),
        button(text("End Call").size(14))
            .on_press(Message::Call(CallMessage::End))
            .padding(Padding::from([12, 20]))
            .style(theme::danger_button),
    ]
    .spacing(16)
    .into()
}

/// Render the incoming call overlay with improved design.
fn view_incoming_call_overlay(call: &crate::state::CallInfo) -> Element<'static, Message> {
    let peer_name = call
        .peer_display_name
        .clone()
        .unwrap_or_else(|| call.peer_four_words.clone());

    let content = column![
        text("◉").size(48).color(Palette::JADE),
        Space::new().height(16),
        text("Incoming Call").size(24).color(Palette::TEXT_PRIMARY),
        Space::new().height(8),
        text(peer_name).size(16).color(Palette::STONE),
        Space::new().height(32),
        row![
            button(text("Accept").size(14))
                .on_press(Message::Call(CallMessage::Accept))
                .padding(Padding::from([12, 24]))
                .style(theme::primary_button),
            button(text("Decline").size(14))
                .on_press(Message::Call(CallMessage::Reject))
                .padding(Padding::from([12, 24]))
                .style(theme::danger_button),
        ]
        .spacing(16),
    ]
    .align_x(Alignment::Center);

    container(container(content).padding(40).style(theme::modal_content))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::modal_overlay)
        .into()
}

/// Render the network panel with improved design.
fn view_network_panel(app_state: &AppState) -> Element<'static, Message> {
    let header = text("Network Status").size(24).color(Palette::TEXT_PRIMARY);

    let status = if app_state.network_info.is_networking {
        "Connected"
    } else {
        "Disconnected"
    };

    let status_color = if app_state.network_info.is_networking {
        Palette::ONLINE
    } else {
        Palette::OFFLINE
    };

    let status_row = row![
        text("Status").size(14).color(Palette::STONE),
        Space::new().width(8),
        container(
            row![
                container(Space::new().width(8).height(8)).style(move |_t: &Theme| {
                    container::Style {
                        background: Some(status_color.into()),
                        border: Border::default().rounded(999),
                        ..Default::default()
                    }
                }),
                text(status).size(14).color(status_color),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let listen_addr = app_state
        .network_info
        .listen_address
        .clone()
        .unwrap_or_else(|| "—".to_string());
    let listen_row = row![
        text("Listen Address").size(14).color(Palette::STONE),
        Space::new().width(8),
        text(listen_addr).size(14).color(Palette::TEXT_PRIMARY),
    ]
    .spacing(8);

    let peer_count = app_state.network_info.peer_count();
    let peers_row = row![
        text("Connected Peers").size(14).color(Palette::STONE),
        Space::new().width(8),
        text(peer_count.to_string()).size(14).color(Palette::JADE),
    ]
    .spacing(8);

    // Peer list
    let peer_items: Vec<Element<'static, Message>> = app_state
        .network_info
        .peers
        .iter()
        .map(|peer| {
            let label = peer.display_label();
            let endpoint = peer.endpoint.clone();
            container(
                row![
                    container(Space::new().width(8).height(8)).style(|_t: &Theme| {
                        container::Style {
                            background: Some(Palette::ONLINE.into()),
                            border: Border::default().rounded(999),
                            ..Default::default()
                        }
                    }),
                    text(label).size(14).color(Palette::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    text(endpoint).size(12).color(Palette::STONE),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .padding(8),
            )
            .style(|_t: &Theme| container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()),
                border: Border::default().rounded(8),
                ..Default::default()
            })
            .into()
        })
        .collect();

    let peer_list: Element<'static, Message> = if peer_items.is_empty() {
        container(text("No peers connected").size(14).color(Palette::STONE))
            .padding(16)
            .into()
    } else {
        scrollable(column(peer_items).spacing(8))
            .height(200)
            .style(theme::scrollbar_style)
            .into()
    };

    // Bootstrap nodes
    let bootstrap_header = text("Bootstrap Nodes")
        .size(16)
        .color(Palette::TEXT_PRIMARY);

    let bootstrap_items: Vec<Element<'static, Message>> = app_state
        .network_info
        .bootstrap_nodes
        .iter()
        .map(|node| {
            let status_color = if node.is_connected {
                Palette::ONLINE
            } else {
                Palette::OFFLINE
            };
            let name = node.name.clone();
            let address = node.address.clone();

            container(
                row![
                    container(Space::new().width(8).height(8)).style(move |_t: &Theme| {
                        container::Style {
                            background: Some(status_color.into()),
                            border: Border::default().rounded(999),
                            ..Default::default()
                        }
                    }),
                    text(name).size(14).color(Palette::TEXT_PRIMARY),
                    Space::new().width(Length::Fill),
                    text(address).size(12).color(Palette::STONE),
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .padding(8),
            )
            .style(|_t: &Theme| container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()),
                border: Border::default().rounded(8),
                ..Default::default()
            })
            .into()
        })
        .collect();

    let bootstrap_list = column(bootstrap_items).spacing(8);

    container(
        scrollable(
            column![
                header,
                Space::new().height(32),
                // Status card
                container(
                    column![status_row, listen_row, peers_row,]
                        .spacing(12)
                        .padding(20),
                )
                .style(theme::card_style)
                .width(Length::Fill),
                Space::new().height(24),
                text("Connected Peers")
                    .size(16)
                    .color(Palette::TEXT_PRIMARY),
                Space::new().height(8),
                peer_list,
                Space::new().height(24),
                bootstrap_header,
                Space::new().height(8),
                bootstrap_list,
            ]
            .spacing(8)
            .padding(Padding::from([24, 32])),
        )
        .style(theme::scrollbar_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::detail_container)
    .into()
}

/// Render the kanban board view.
fn view_kanban_board(
    app_state: &AppState,
    entity_id: &str,
    is_read_only: bool,
) -> Element<'static, Message> {
    let columns = KanbanColumn::defaults();
    let cards = app_state.kanban_cards.get(entity_id);
    let entity_id_owned = entity_id.to_string();

    // Create column views
    let column_views: Vec<Element<'static, Message>> = columns
        .into_iter()
        .map(|col| {
            let col_id = col.id.clone();
            let col_name = col.name;
            let entity_id_for_col = entity_id_owned.clone();

            // Get cards for this column
            let column_cards: Vec<KanbanCard> = cards
                .map(|c| {
                    c.iter()
                        .filter(|card| card.column == col_id)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();

            let card_views: Vec<Element<'static, Message>> = column_cards
                .into_iter()
                .map(|card| view_kanban_card_owned(card, is_read_only))
                .collect();

            // Column header with add button (disabled for read-only)
            let add_btn = if is_read_only {
                button(text("+").size(14).color(Color::from_rgb(0.5, 0.5, 0.5)))
                    .style(theme::ghost_button)
                    .padding(Padding::from([2, 8]))
            } else {
                button(text("+").size(14).color(Palette::JADE))
                    .on_press(Message::Kanban(KanbanMessage::CreateCardPressed(
                        entity_id_for_col.clone(),
                    )))
                    .style(theme::ghost_button)
                    .padding(Padding::from([2, 8]))
            };

            let header = row![
                text(col_name).size(14).color(Palette::TEXT_PRIMARY),
                Space::new().width(Length::Fill),
                add_btn,
            ]
            .align_y(Alignment::Center)
            .padding(Padding::from([12, 16]));

            let card_list = scrollable(
                column(card_views)
                    .spacing(8)
                    .padding(Padding::from([8, 12])),
            )
            .height(Length::Fill)
            .style(theme::scrollbar_style);

            container(column![header, card_list])
                .width(280)
                .height(Length::Fill)
                .style(|_t: &Theme| container::Style {
                    background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()),
                    border: Border::default().rounded(12),
                    ..Default::default()
                })
                .into()
        })
        .collect();

    container(
        scrollable(
            row(column_views)
                .spacing(16)
                .padding(Padding::from([16, 20])),
        )
        .direction(scrollable::Direction::Horizontal(
            scrollable::Scrollbar::default(),
        ))
        .style(theme::scrollbar_style),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(theme::detail_container)
    .into()
}

/// Render a kanban card (owned version).
fn view_kanban_card_owned(card: KanbanCard, _is_read_only: bool) -> Element<'static, Message> {
    let card_clone = card.clone();
    let priority_color = card.priority.color();
    let title = card.title;
    let desc = card.description;
    let assignee = card.assignee;
    let comment_count = card.comment_count;

    let mut content: Vec<Element<'static, Message>> = vec![
        row![
            container(Space::new().width(4).height(16)).style(move |_t: &Theme| container::Style {
                background: Some(priority_color.into()),
                border: Border::default().rounded(2),
                ..Default::default()
            }),
            text(title).size(14).color(Palette::TEXT_PRIMARY),
        ]
        .spacing(8)
        .into(),
    ];

    if let Some(description) = desc
        && !description.is_empty()
    {
        content.push(text(description).size(12).color(Palette::STONE).into());
    }

    let mut footer_items: Vec<Element<'static, Message>> = Vec::new();
    if let Some(assignee_fw) = assignee {
        let short_assignee: String = assignee_fw.split('-').take(2).collect::<Vec<_>>().join("-");
        footer_items.push(
            text(format!("@{short_assignee}"))
                .size(11)
                .color(Palette::JADE)
                .into(),
        );
    }
    if comment_count > 0 {
        footer_items.push(Space::new().width(Length::Fill).into());
        footer_items.push(
            text(format!("💬 {comment_count}"))
                .size(11)
                .color(Palette::STONE)
                .into(),
        );
    }

    if !footer_items.is_empty() {
        content.push(row(footer_items).align_y(Alignment::Center).into());
    }

    button(
        container(column(content).spacing(6).width(Length::Fill))
            .padding(Padding::from([12, 14]))
            .style(|_t: &Theme| container::Style {
                background: Some(Palette::WARM_WHITE.into()),
                border: Border::default().rounded(10),
                shadow: iced::Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.08),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
                ..Default::default()
            }),
    )
    .on_press(Message::Kanban(KanbanMessage::CardClicked(card_clone)))
    .style(|_t: &Theme, _status| button::Style {
        background: None,
        text_color: Palette::TEXT_PRIMARY,
        border: Border::default(),
        shadow: iced::Shadow::default(),
        snap: false,
    })
    .width(Length::Fill)
    .into()
}

/// Render the documents view for an entity.
fn view_documents(app_state: &AppState, entity_id: &str) -> Element<'static, Message> {
    let documents = app_state.documents.get(entity_id).cloned();

    let header = container(
        row![
            text("Documents").size(18).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("+ New Document").size(13))
                .style(theme::primary_button)
                .padding(Padding::from([8, 16]))
                .on_press(Message::Noop), // TODO: Wire to CreateDocument modal
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([12, 16])),
    )
    .style(|_t: &Theme| container::Style {
        background: Some(iced::Background::Color(Palette::CREAM.scale_alpha(0.3))),
        ..Default::default()
    });

    let content: Element<'static, Message> = if let Some(docs) = documents {
        if docs.is_empty() {
            view_empty_documents()
        } else {
            view_document_list(docs)
        }
    } else {
        view_empty_documents()
    };

    container(column![header, content,])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::detail_container)
        .into()
}

/// Render an empty documents state.
fn view_empty_documents() -> Element<'static, Message> {
    container(
        column![
            text("📄").size(48).color(Palette::STONE.scale_alpha(0.5)),
            Space::new().height(16),
            text("No Documents Yet")
                .size(18)
                .color(Palette::TEXT_PRIMARY),
            Space::new().height(8),
            text("Create a document to get started")
                .size(14)
                .color(Palette::STONE),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Render a list of documents.
fn view_document_list(documents: Vec<Document>) -> Element<'static, Message> {
    let items: Vec<Element<'static, Message>> =
        documents.into_iter().map(view_document_row).collect();

    scrollable(column(items).spacing(1).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render a single document row.
fn view_document_row(doc: Document) -> Element<'static, Message> {
    let modified = format_timestamp(doc.modified_at);

    container(
        row![
            // Document icon
            container(text("📄").size(20))
                .width(36)
                .center_x(36)
                .center_y(Length::Shrink),
            // Document info
            column![
                text(doc.title).size(14).color(Palette::TEXT_PRIMARY),
                text(format!("Modified {}", modified))
                    .size(11)
                    .color(Palette::STONE),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            // Actions
            button(text("Open").size(12))
                .style(theme::primary_button)
                .padding(Padding::from([6, 12]))
                .on_press(Message::Noop), // TODO: Wire to OpenDocument
        ]
        .align_y(Alignment::Center)
        .spacing(12)
        .padding(Padding::from([12, 16])),
    )
    .style(|_t: &Theme| container::Style {
        background: Some(iced::Background::Color(Palette::WARM_WHITE)),
        border: Border {
            color: theme::DIVIDER_LIGHT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Render the drive/file browser view for an entity.
fn view_drive(app_state: &AppState, entity_id: &str) -> Element<'static, Message> {
    let files = app_state.files.get(entity_id).cloned();

    let header = container(
        row![
            text("Virtual Drive").size(18).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("+ New Folder").size(13))
                .style(theme::secondary_button)
                .padding(Padding::from([8, 16]))
                .on_press(Message::Storage(StorageMessage::CreateFolder(
                    entity_id.to_string()
                ))),
            Space::new().width(8),
            button(text("Upload").size(13))
                .style(theme::primary_button)
                .padding(Padding::from([8, 16]))
                .on_press(Message::Noop), // TODO: Wire to file picker
        ]
        .align_y(Alignment::Center)
        .padding(Padding::from([12, 16])),
    )
    .style(|_t: &Theme| container::Style {
        background: Some(iced::Background::Color(Palette::CREAM.scale_alpha(0.3))),
        ..Default::default()
    });

    let content: Element<'static, Message> = if let Some(file_list) = files {
        if file_list.is_empty() {
            view_empty_drive()
        } else {
            view_file_list(file_list)
        }
    } else {
        view_empty_drive()
    };

    container(column![header, content,])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::detail_container)
        .into()
}

/// Render an empty drive state.
fn view_empty_drive() -> Element<'static, Message> {
    container(
        column![
            text("📁").size(48).color(Palette::STONE.scale_alpha(0.5)),
            Space::new().height(16),
            text("No Files Yet").size(18).color(Palette::TEXT_PRIMARY),
            Space::new().height(8),
            text("Upload files or create folders to get started")
                .size(14)
                .color(Palette::STONE),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

/// Render a list of files.
fn view_file_list(files: Vec<FileInfo>) -> Element<'static, Message> {
    // Sort folders first, then files by name
    let mut sorted = files;
    sorted.sort_by(|a, b| match (a.is_folder, b.is_folder) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    let items: Vec<Element<'static, Message>> = sorted.into_iter().map(view_file_row).collect();

    scrollable(column(items).spacing(1).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Render a single file or folder row.
fn view_file_row(file: FileInfo) -> Element<'static, Message> {
    let icon = if file.is_folder {
        "📁"
    } else {
        get_file_icon(&file.name)
    };
    let size_str = if file.is_folder {
        String::new()
    } else {
        format_file_size(file.size)
    };
    let date_str = format_timestamp(file.created_at);

    let delete_btn = button(text("🗑").size(14))
        .style(theme::icon_button)
        .padding(Padding::from([4, 8]))
        .on_press(Message::Storage(StorageMessage::DeleteFile(file.id)));

    // Build the detail text based on whether it's a folder
    let detail_text = if size_str.is_empty() {
        date_str
    } else {
        format!("{} • {}", size_str, date_str)
    };

    container(
        row![
            // File/folder icon
            container(text(icon).size(20))
                .width(36)
                .center_x(36)
                .center_y(Length::Shrink),
            // File info
            column![
                text(file.name).size(14).color(Palette::TEXT_PRIMARY),
                text(detail_text).size(11).color(Palette::STONE),
            ]
            .spacing(2),
            Space::new().width(Length::Fill),
            // Actions
            delete_btn,
        ]
        .align_y(Alignment::Center)
        .spacing(12)
        .padding(Padding::from([10, 16])),
    )
    .style(|_t: &Theme| container::Style {
        background: Some(iced::Background::Color(Palette::WARM_WHITE)),
        border: Border {
            color: theme::DIVIDER_LIGHT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

/// Get an appropriate icon for a file based on its extension.
fn get_file_icon(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => "📕",
        "doc" | "docx" => "📘",
        "xls" | "xlsx" => "📗",
        "ppt" | "pptx" => "📙",
        "jpg" | "jpeg" | "png" | "gif" | "webp" => "🖼",
        "mp3" | "wav" | "ogg" | "flac" => "🎵",
        "mp4" | "mov" | "avi" | "mkv" => "🎬",
        "zip" | "tar" | "gz" | "7z" => "📦",
        "md" | "txt" => "📝",
        "rs" | "js" | "ts" | "py" | "swift" => "💻",
        _ => "📄",
    }
}

/// Format a file size in bytes to a human-readable string.
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format a Unix timestamp to a relative time string.
fn format_timestamp(timestamp: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{}m ago", mins)
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{}h ago", hours)
    } else if diff < 604800 {
        let days = diff / 86400;
        format!("{}d ago", days)
    } else {
        // Format as date
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

/// Placeholder for details view with improved design.
fn view_details_placeholder<'a>() -> Element<'a, Message> {
    container(
        column![
            text("⚙").size(48).color(Palette::STONE.scale_alpha(0.5)),
            Space::new().height(16),
            text("Entity Details").size(24).color(Palette::TEXT_PRIMARY),
            Space::new().height(8),
            text("Details and settings coming soon...")
                .size(14)
                .color(Palette::STONE),
        ]
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(theme::detail_container)
    .into()
}

/// Render a modal overlay.
fn view_modal<'a>(
    modal_type: &'a ModalType,
    app_state: &'a AppState,
    modal_form_state: &'a ModalFormState,
) -> Element<'a, Message> {
    let content: Element<'a, Message> = match modal_type {
        ModalType::CreateEntity(context) => view_create_entity_modal(context, modal_form_state),
        ModalType::AddContact => view_add_contact_modal(modal_form_state),
        ModalType::CreateCard(column) => view_create_card_modal(column, modal_form_state),
        ModalType::CardDetail(card) => view_card_detail_modal(card, app_state),
        ModalType::EditCard(card) => view_edit_card_modal(card, modal_form_state),
        ModalType::DeleteCardConfirm(card) => view_delete_card_confirm_modal(card),
        ModalType::ContactDetail(contact) => view_contact_detail_modal(contact),
        ModalType::RemoveContactConfirm(contact) => view_remove_contact_confirm_modal(contact),
        ModalType::Settings => view_settings_modal(),
        ModalType::Linking(entity_id) => view_linking_modal(entity_id),
        ModalType::EditMessage(message) => view_edit_message_modal(message, app_state),
        ModalType::DeleteMessageConfirm(message) => view_delete_message_confirm_modal(message),
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(theme::modal_overlay)
        .into()
}

/// Render the create entity modal.
fn view_create_entity_modal<'a>(
    context: &CreateEntityContext,
    modal_form_state: &'a ModalFormState,
) -> Element<'a, Message> {
    let title = match context.entity_type {
        EntityType::Organisation => "Create Organisation",
        EntityType::Project => "Create Project",
        EntityType::Channel => "Create Channel",
        EntityType::Group => "Create Group",
    };

    let parent_text = if let Some(ref parent_id) = context.parent_org_id {
        format!("Parent: {parent_id}")
    } else {
        "Top-level organisation".to_string()
    };

    let has_parent = context.parent_org_id.is_some();
    let parent_org_id = context.parent_org_id.clone();

    // Entity type buttons for switching type
    let type_section: Element<'a, Message> = if has_parent {
        let parent_id = parent_org_id.clone();
        column![
            text("Type").size(12).color(Palette::STONE),
            row![
                view_entity_type_button_static(
                    EntityType::Channel,
                    context.entity_type,
                    parent_id.clone()
                ),
                view_entity_type_button_static(
                    EntityType::Project,
                    context.entity_type,
                    parent_id.clone()
                ),
                view_entity_type_button_static(EntityType::Group, context.entity_type, parent_id),
            ]
            .spacing(8)
        ]
        .spacing(8)
        .into()
    } else {
        Space::new().height(0).into()
    };

    // Check if form is valid for submission
    let can_submit = !modal_form_state.entity_name.trim().is_empty();

    let content = column![
        // Header
        row![
            text(title).size(20).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(8),
        text(parent_text).size(12).color(Palette::STONE),
        Space::new().height(16),
        type_section,
        Space::new().height(16),
        // Name input
        text("Name").size(12).color(Palette::STONE),
        text_input("Enter name...", &modal_form_state.entity_name)
            .on_input(|s| Message::Modal(ModalMessage::EntityNameChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(16),
        // Description input
        text("Description (optional)")
            .size(12)
            .color(Palette::STONE),
        text_input("Enter description...", &modal_form_state.entity_description)
            .on_input(|s| Message::Modal(ModalMessage::EntityDescriptionChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Create").size(14))
                .on_press_maybe(if can_submit {
                    Some(Message::Modal(ModalMessage::SubmitCreateEntity))
                } else {
                    None
                })
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render an entity type selection button (static version).
fn view_entity_type_button_static(
    entity_type: EntityType,
    current_type: EntityType,
    parent_org_id: Option<String>,
) -> Element<'static, Message> {
    let is_selected = current_type == entity_type;
    let label = entity_type.display_name();

    let context_clone = CreateEntityContext {
        parent_org_id,
        entity_type,
    };

    button(text(label).size(12))
        .on_press(Message::Sidebar(SidebarMessage::CreateEntity(
            context_clone,
        )))
        .style(move |t: &Theme, status| theme::tab_button(t, status, is_selected))
        .padding(Padding::from([6, 12]))
        .into()
}

/// Render the add contact modal.
fn view_add_contact_modal<'a>(modal_form_state: &'a ModalFormState) -> Element<'a, Message> {
    // Validate four-word format (basic check: 4 words separated by hyphens)
    let four_words_valid = {
        let trimmed = modal_form_state.contact_four_words.trim();
        !trimmed.is_empty() && trimmed.split('-').count() == 4
    };
    let can_submit = four_words_valid;

    let content = column![
        // Header
        row![
            text("Add Contact").size(20).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Four-word input
        text("Four-Word Address").size(12).color(Palette::STONE),
        text_input(
            "e.g., ocean-forest-moon-star",
            &modal_form_state.contact_four_words
        )
        .on_input(|s| Message::Modal(ModalMessage::ContactFourWordsChanged(s)))
        .padding(Padding::from([12, 16]))
        .style(theme::input_style),
        Space::new().height(16),
        // Display name input
        text("Display Name (optional)")
            .size(12)
            .color(Palette::STONE),
        text_input("Enter display name...", &modal_form_state.contact_name)
            .on_input(|s| Message::Modal(ModalMessage::ContactNameChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Add Contact").size(14))
                .on_press_maybe(if can_submit {
                    Some(Message::Modal(ModalMessage::SubmitAddContact))
                } else {
                    None
                })
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render the create card modal.
fn view_create_card_modal<'a>(
    column: &str,
    modal_form_state: &'a ModalFormState,
) -> Element<'a, Message> {
    let can_submit = !modal_form_state.card_title.trim().is_empty();

    let content = column![
        // Header
        row![
            text(format!("New Card in {column}"))
                .size(20)
                .color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Title input
        text("Title").size(12).color(Palette::STONE),
        text_input("Enter card title...", &modal_form_state.card_title)
            .on_input(|s| Message::Kanban(KanbanMessage::CardTitleChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(16),
        // Description input
        text("Description (optional)")
            .size(12)
            .color(Palette::STONE),
        text_input("Enter description...", &modal_form_state.card_description)
            .on_input(|s| Message::Kanban(KanbanMessage::CardDescriptionChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Create Card").size(14))
                .on_press_maybe(if can_submit {
                    Some(Message::Kanban(KanbanMessage::SubmitCreateCard))
                } else {
                    None
                })
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render the card detail modal.
fn view_card_detail_modal(card: &KanbanCard, _app_state: &AppState) -> Element<'static, Message> {
    let title = card.title.clone();
    let description = card.description.clone().unwrap_or_default();
    let column_name = card.column.clone();
    let assignee = card
        .assignee
        .clone()
        .unwrap_or_else(|| "Unassigned".to_string());
    let priority_color = card.priority.color();
    let priority_name = card.priority.display_name();
    let card_id = card.id.clone();
    let card_for_edit = card.clone();
    let card_id_for_delete = card.id.clone();

    let desc_display: Element<'static, Message> = if description.is_empty() {
        text("No description")
            .size(14)
            .color(Palette::STONE.scale_alpha(0.5))
            .into()
    } else {
        text(description)
            .size(14)
            .color(Palette::TEXT_PRIMARY)
            .into()
    };

    let content = column![
        // Header with close button
        row![
            row![
                container(Space::new().width(4).height(20)).style(move |_t: &Theme| {
                    container::Style {
                        background: Some(priority_color.into()),
                        border: Border::default().rounded(2),
                        ..Default::default()
                    }
                }),
                text(title).size(20).color(Palette::TEXT_PRIMARY),
            ]
            .spacing(12),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Column and Priority badges
        row![
            text("Column:").size(12).color(Palette::STONE),
            container(text(column_name.clone()).size(12).color(Palette::JADE))
                .padding(Padding::from([4, 8]))
                .style(|_t: &Theme| container::Style {
                    background: Some(Palette::JADE.scale_alpha(0.1).into()),
                    border: Border::default().rounded(4),
                    ..Default::default()
                }),
            Space::new().width(16),
            text("Priority:").size(12).color(Palette::STONE),
            container(text(priority_name).size(12).color(Palette::TEXT_PRIMARY))
                .padding(Padding::from([4, 8]))
                .style(move |_t: &Theme| container::Style {
                    background: Some(priority_color.scale_alpha(0.15).into()),
                    border: Border::default().rounded(4),
                    ..Default::default()
                }),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        Space::new().height(12),
        // Assignee
        row![
            text("Assignee:").size(12).color(Palette::STONE),
            text(assignee).size(12).color(Palette::TEXT_PRIMARY),
        ]
        .spacing(8),
        Space::new().height(16),
        // Description
        text("Description").size(12).color(Palette::STONE),
        container(desc_display)
            .padding(12)
            .width(Length::Fill)
            .style(|_t: &Theme| container::Style {
                background: Some(Color::from_rgba(0.0, 0.0, 0.0, 0.03).into()),
                border: Border::default().rounded(8),
                ..Default::default()
            }),
        Space::new().height(24),
        // Move buttons
        text("Move to:").size(12).color(Palette::STONE),
        row![
            view_move_column_button_static(
                "backlog",
                "Backlog",
                column_name.clone(),
                card_id.clone()
            ),
            view_move_column_button_static("todo", "To Do", column_name.clone(), card_id.clone()),
            view_move_column_button_static(
                "in_progress",
                "In Progress",
                column_name.clone(),
                card_id.clone()
            ),
            view_move_column_button_static("done", "Done", column_name, card_id),
        ]
        .spacing(8),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Edit").size(14))
                .on_press(Message::Kanban(KanbanMessage::EditCard(card_for_edit)))
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
            button(text("Delete").size(14))
                .on_press(Message::Kanban(KanbanMessage::DeleteCardPressed(
                    card_id_for_delete
                )))
                .style(theme::danger_button)
                .padding(Padding::from([10, 20])),
            Space::new().width(Length::Fill),
            button(text("Close").size(14))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(500);

    container(content).style(theme::modal_content).into()
}

/// Render the edit card modal.
fn view_edit_card_modal<'a>(
    card: &KanbanCard,
    modal_form_state: &'a ModalFormState,
) -> Element<'a, Message> {
    let can_submit = !modal_form_state.card_title.trim().is_empty();
    let card_title = card.title.clone();

    // Priority selector buttons
    let priority_buttons: Element<'a, Message> = row(CardPriority::all()
        .into_iter()
        .map(|p| {
            let is_selected = modal_form_state.card_priority == p;
            let color = p.color();
            button(text(p.display_name()).size(12))
                .on_press(Message::Kanban(KanbanMessage::CardPriorityChanged(p)))
                .style(move |t: &Theme, status| {
                    if is_selected {
                        let mut style = theme::primary_button(t, status);
                        style.background = Some(color.into());
                        style
                    } else {
                        theme::secondary_button(t, status)
                    }
                })
                .padding(Padding::from([6, 12]))
                .into()
        })
        .collect::<Vec<Element<'a, Message>>>())
    .spacing(8)
    .into();

    let content = column![
        // Header
        row![
            text(format!("Edit: {card_title}"))
                .size(20)
                .color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Title input
        text("Title").size(12).color(Palette::STONE),
        text_input("Enter card title...", &modal_form_state.card_title)
            .on_input(|s| Message::Kanban(KanbanMessage::CardTitleChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(16),
        // Description input
        text("Description").size(12).color(Palette::STONE),
        text_input("Enter description...", &modal_form_state.card_description)
            .on_input(|s| Message::Kanban(KanbanMessage::CardDescriptionChanged(s)))
            .padding(Padding::from([12, 16]))
            .style(theme::input_style),
        Space::new().height(16),
        // Priority selector
        text("Priority").size(12).color(Palette::STONE),
        priority_buttons,
        Space::new().height(16),
        // Assignee input
        text("Assignee (four-word identity)")
            .size(12)
            .color(Palette::STONE),
        text_input(
            "e.g., ocean-forest-moon-star",
            &modal_form_state.card_assignee
        )
        .on_input(|s| Message::Kanban(KanbanMessage::CardAssigneeChanged(s)))
        .padding(Padding::from([12, 16]))
        .style(theme::input_style),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Save Changes").size(14))
                .on_press_maybe(if can_submit {
                    Some(Message::Kanban(KanbanMessage::SubmitEditCard))
                } else {
                    None
                })
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(450);

    container(content).style(theme::modal_content).into()
}

/// Render the delete card confirmation modal.
fn view_delete_card_confirm_modal(card: &KanbanCard) -> Element<'static, Message> {
    let card_title = card.title.clone();
    let card_id = card.id.clone();

    let content = column![
        // Header
        row![
            text("Delete Card").size(20).color(Palette::ERROR),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Warning message
        text("Are you sure you want to delete this card?")
            .size(14)
            .color(Palette::TEXT_PRIMARY),
        Space::new().height(8),
        container(text(card_title).size(16).color(Palette::TEXT_PRIMARY))
            .padding(12)
            .width(Length::Fill)
            .style(|_t: &Theme| container::Style {
                background: Some(Palette::ERROR.scale_alpha(0.1).into()),
                border: Border::default()
                    .rounded(8)
                    .color(Palette::ERROR.scale_alpha(0.3)),
                ..Default::default()
            }),
        Space::new().height(8),
        text("This action cannot be undone.")
            .size(12)
            .color(Palette::STONE),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Kanban(KanbanMessage::CloseCardModal))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Delete Card").size(14))
                .on_press(Message::Kanban(KanbanMessage::ConfirmDeleteCard(card_id)))
                .style(theme::danger_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render a move-to-column button (static version).
fn view_move_column_button_static(
    column_id: &str,
    label: &str,
    current_column: String,
    card_id: String,
) -> Element<'static, Message> {
    let is_current = column_id == current_column;
    let target_column = column_id.to_string();
    let label_owned = label.to_string();

    button(text(label_owned).size(11))
        .on_press_maybe(if is_current {
            None
        } else {
            Some(Message::Kanban(KanbanMessage::CardDropped {
                card_id,
                column: target_column,
                position: 0,
            }))
        })
        .style(move |t: &Theme, status| theme::tab_button(t, status, is_current))
        .padding(Padding::from([4, 8]))
        .into()
}

/// Render the settings modal.
fn view_settings_modal() -> Element<'static, Message> {
    let content = column![
        row![
            text("Settings").size(20).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        text("Settings coming soon...")
            .size(14)
            .color(Palette::STONE),
        Space::new().height(24),
        button(text("Close").size(14))
            .on_press(Message::Modal(ModalMessage::Close))
            .style(theme::secondary_button)
            .padding(Padding::from([10, 20])),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render the linking modal.
fn view_linking_modal(entity_id: &str) -> Element<'static, Message> {
    let eid = entity_id.to_string();
    let content = column![
        row![
            text("Link to Network")
                .size(20)
                .color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        text(format!("Linking entity: {eid}"))
            .size(14)
            .color(Palette::STONE),
        Space::new().height(8),
        text("This will create a network identity for this entity.")
            .size(12)
            .color(Palette::STONE),
        Space::new().height(24),
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Link").size(14))
                .on_press(Message::Modal(ModalMessage::Close))
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render the contact detail modal.
fn view_contact_detail_modal(contact: &crate::state::Contact) -> Element<'static, Message> {
    let display_name = contact.display_name.clone();
    let four_words = contact
        .four_words
        .clone()
        .unwrap_or_else(|| "Local contact".to_string());
    let is_local = contact.is_local_only;
    let is_favorite = contact.is_favorite;
    let contact_id = contact.id.clone();
    let contact_id_fav = contact.id.clone();
    let contact_id_remove = contact.id.clone();
    let contact_id_link = contact.id.clone();

    let status_text = match contact.status {
        ContactStatus::Online => "Online",
        ContactStatus::Away => "Away",
        ContactStatus::Offline => "Offline",
    };
    let status_color = contact.status.color();

    let last_seen_text = contact
        .last_seen
        .map(|ts| {
            let now = chrono::Utc::now().timestamp();
            let diff = now - ts;
            if diff < 60 {
                "Just now".to_string()
            } else if diff < 3600 {
                format!("{} minutes ago", diff / 60)
            } else if diff < 86400 {
                format!("{} hours ago", diff / 3600)
            } else {
                format!("{} days ago", diff / 86400)
            }
        })
        .unwrap_or_else(|| "Unknown".to_string());

    let mut actions: Vec<Element<'static, Message>> = vec![];

    // Favorite toggle
    actions.push(
        button(
            row![
                text(if is_favorite { "★" } else { "☆" })
                    .size(14)
                    .color(if is_favorite {
                        Palette::AMBER
                    } else {
                        Palette::STONE
                    }),
                text(if is_favorite {
                    "Unfavorite"
                } else {
                    "Favorite"
                })
                .size(14),
            ]
            .spacing(6),
        )
        .on_press(Message::Contact(ContactMessage::ToggleFavorite(
            contact_id_fav,
        )))
        .style(theme::secondary_button)
        .padding(Padding::from([8, 16]))
        .into(),
    );

    // Link to network (only for local contacts)
    if is_local {
        actions.push(
            button(text("Link to Network").size(14))
                .on_press(Message::Contact(ContactMessage::LinkToNetworkPressed(
                    contact_id_link,
                )))
                .style(theme::primary_button)
                .padding(Padding::from([8, 16]))
                .into(),
        );
    }

    // Remove contact
    actions.push(
        button(text("Remove").size(14))
            .on_press(Message::Contact(ContactMessage::RemoveContactPressed(
                contact_id_remove,
            )))
            .style(theme::danger_button)
            .padding(Padding::from([8, 16]))
            .into(),
    );

    let content = column![
        // Header
        row![
            text("Contact Details")
                .size(20)
                .color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Contact(ContactMessage::CloseContactDetail))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(20),
        // Avatar placeholder and name
        row![
            container(text("👤").size(40))
                .width(60)
                .height(60)
                .center_x(60)
                .center_y(60)
                .style(|_t: &Theme| container::Style {
                    background: Some(Palette::FERN.into()),
                    border: Border::default().rounded(30),
                    ..Default::default()
                }),
            column![
                text(display_name).size(18).color(Palette::TEXT_PRIMARY),
                text(four_words).size(12).color(Palette::STONE),
            ]
            .spacing(4),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
        Space::new().height(20),
        // Status
        row![
            text("Status:").size(12).color(Palette::STONE),
            container(Space::new().width(8).height(8)).style(move |_t: &Theme| container::Style {
                background: Some(status_color.into()),
                border: Border::default().rounded(999),
                ..Default::default()
            }),
            text(status_text).size(12).color(Palette::TEXT_PRIMARY),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        Space::new().height(8),
        // Last seen
        row![
            text("Last seen:").size(12).color(Palette::STONE),
            text(last_seen_text).size(12).color(Palette::TEXT_PRIMARY),
        ]
        .spacing(8),
        Space::new().height(8),
        // Type badge
        row![
            text("Type:").size(12).color(Palette::STONE),
            container(
                text(if is_local { "Local" } else { "Network" })
                    .size(11)
                    .color(if is_local {
                        Palette::STONE
                    } else {
                        Palette::JADE
                    })
            )
            .padding(Padding::from([2, 6]))
            .style(move |_t: &Theme| container::Style {
                background: Some(
                    if is_local {
                        Palette::STONE.scale_alpha(0.2)
                    } else {
                        Palette::JADE.scale_alpha(0.2)
                    }
                    .into()
                ),
                border: Border::default().rounded(4),
                ..Default::default()
            }),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        Space::new().height(24),
        // Actions
        Row::from_vec(actions).spacing(12),
        Space::new().height(16),
        // Close button
        button(text("Close").size(14))
            .on_press(Message::Contact(ContactMessage::CloseContactDetail))
            .style(theme::secondary_button)
            .padding(Padding::from([10, 20])),
    ]
    .spacing(4)
    .padding(24)
    .width(420);

    // Suppress unused variable warning
    let _ = contact_id;

    container(content).style(theme::modal_content).into()
}

/// Render the remove contact confirmation modal.
fn view_remove_contact_confirm_modal(contact: &crate::state::Contact) -> Element<'static, Message> {
    let display_name = contact.display_name.clone();
    let contact_id = contact.id.clone();

    let content = column![
        // Header
        row![
            text("Remove Contact").size(20).color(Palette::ERROR),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Contact(ContactMessage::CloseContactDetail))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Warning message
        text("Are you sure you want to remove this contact?")
            .size(14)
            .color(Palette::TEXT_PRIMARY),
        Space::new().height(8),
        container(text(display_name).size(16).color(Palette::TEXT_PRIMARY))
            .padding(12)
            .width(Length::Fill)
            .style(|_t: &Theme| container::Style {
                background: Some(Palette::ERROR.scale_alpha(0.1).into()),
                border: Border::default()
                    .rounded(8)
                    .color(Palette::ERROR.scale_alpha(0.3)),
                ..Default::default()
            }),
        Space::new().height(8),
        text("This will remove them from your contacts list.")
            .size(12)
            .color(Palette::STONE),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Contact(ContactMessage::CloseContactDetail))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Remove Contact").size(14))
                .on_press(Message::Contact(ContactMessage::ConfirmRemoveContact(
                    contact_id
                )))
                .style(theme::danger_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}

/// Render the edit message modal.
fn view_edit_message_modal<'a>(
    message: &ChatMessage,
    app_state: &'a AppState,
) -> Element<'a, Message> {
    let content = column![
        // Header
        row![
            text("Edit Message").size(20).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Chat(ChatMessageEvent::CancelEdit))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Original message info
        text(format!("Editing message from {}", message.formatted_time()))
            .size(12)
            .color(Palette::STONE),
        Space::new().height(8),
        // Text input
        text_input("Edit your message...", &app_state.editing_message_text)
            .on_input(|text| Message::Chat(ChatMessageEvent::EditTextChanged(text)))
            .on_submit(Message::Chat(ChatMessageEvent::SubmitEdit))
            .padding(12)
            .style(theme::input_style),
        Space::new().height(16),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Chat(ChatMessageEvent::CancelEdit))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Save").size(14))
                .on_press(Message::Chat(ChatMessageEvent::SubmitEdit))
                .style(theme::primary_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(500);

    container(content).style(theme::modal_content).into()
}

/// Render the delete message confirmation modal.
fn view_delete_message_confirm_modal(message: &ChatMessage) -> Element<'static, Message> {
    let message_id = message.id.clone();
    let preview = if message.text.len() > 50 {
        format!("{}...", &message.text[..50])
    } else {
        message.text.clone()
    };

    let content = column![
        // Header with warning icon
        row![
            text("⚠️").size(24),
            Space::new().width(8),
            text("Delete Message").size(20).color(Palette::TEXT_PRIMARY),
            Space::new().width(Length::Fill),
            button(text("×").size(20).color(Palette::STONE))
                .on_press(Message::Chat(ChatMessageEvent::CancelEdit))
                .style(theme::ghost_button),
        ]
        .align_y(Alignment::Center),
        Space::new().height(16),
        // Warning text
        container(
            column![
                text("Message preview:").size(12).color(Palette::STONE),
                Space::new().height(4),
                text(preview).size(14).color(Palette::TEXT_MUTED),
            ]
            .padding(12)
        )
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::ERROR.scale_alpha(0.1).into()),
            border: Border::default()
                .rounded(8)
                .color(Palette::ERROR.scale_alpha(0.3)),
            ..Default::default()
        }),
        Space::new().height(8),
        text("This message will be permanently deleted.")
            .size(12)
            .color(Palette::STONE),
        Space::new().height(24),
        // Action buttons
        row![
            button(text("Cancel").size(14))
                .on_press(Message::Chat(ChatMessageEvent::CancelEdit))
                .style(theme::secondary_button)
                .padding(Padding::from([10, 20])),
            button(text("Delete Message").size(14))
                .on_press(Message::Chat(ChatMessageEvent::ConfirmDeleteMessage(
                    message_id
                )))
                .style(theme::danger_button)
                .padding(Padding::from([10, 20])),
        ]
        .spacing(12),
    ]
    .spacing(4)
    .padding(24)
    .width(400);

    container(content).style(theme::modal_content).into()
}
