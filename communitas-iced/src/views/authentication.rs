// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Authentication view for login and identity creation.

use crate::message::{AuthMessage, Message};
use crate::state::AuthState;
use crate::theme::Palette;
use iced::widget::{Space, button, column, container, pick_list, text, text_input};
use iced::{Alignment, Border, Element, Length, Theme};

/// Render the authentication view.
#[must_use]
pub fn view_authentication(state: &AuthState) -> Element<'static, Message> {
    let content = if state.creating_identity {
        view_create_identity(state)
    } else {
        view_login(state)
    };

    container(
        container(content)
            .padding(40)
            .max_width(400)
            .style(|_t: &Theme| container::Style {
                background: Some(iced::Color::WHITE.into()),
                border: Border::default().rounded(12),
                shadow: iced::Shadow {
                    color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 20.0,
                },
                ..Default::default()
            }),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .style(|_t: &Theme| container::Style {
        background: Some(Palette::SIDEBAR_BG.into()),
        ..Default::default()
    })
    .into()
}

/// Render the login form.
fn view_login(state: &AuthState) -> Element<'static, Message> {
    let title = text("Welcome to Communitas")
        .size(28)
        .color(Palette::TEXT_PRIMARY);

    let subtitle = text("Sign in to continue")
        .size(14)
        .color(Palette::TEXT_MUTED);

    // Vault selection - use display names for the picker
    let vault_names: Vec<String> = state
        .vaults
        .iter()
        .map(|v| v.display_name.clone())
        .collect();
    let vault_picker = pick_list(vault_names, state.selected_vault.clone(), |name| {
        Message::Auth(AuthMessage::VaultSelected(name))
    })
    .placeholder("Select a vault...")
    .width(Length::Fill);

    let vault_section = column![
        text("Vault").size(12).color(Palette::TEXT_MUTED),
        vault_picker,
    ]
    .spacing(4);

    // Password input
    let password_input = text_input("Enter password...", &state.password)
        .on_input(|s| Message::Auth(AuthMessage::PasswordChanged(s)))
        .on_submit(Message::Auth(AuthMessage::LoginPressed))
        .secure(true)
        .width(Length::Fill);

    let password_section = column![
        text("Password").size(12).color(Palette::TEXT_MUTED),
        password_input,
    ]
    .spacing(4);

    // Error message
    let error_message: Element<'static, Message> = if let Some(ref error) = state.error {
        container(text(error.clone()).size(12).color(Palette::ERROR))
            .padding(8)
            .into()
    } else {
        Space::new().height(0).into()
    };

    // Login button
    let login_button = button(
        text(if state.is_loading {
            "Signing in..."
        } else {
            "Sign In"
        })
        .width(Length::Fill)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(12)
    .on_press_maybe(if state.is_loading || state.password.is_empty() {
        None
    } else {
        Some(Message::Auth(AuthMessage::LoginPressed))
    })
    .style(|_t: &Theme, status| {
        let base = button::Style {
            background: Some(Palette::ORGANISATION.into()),
            text_color: iced::Color::WHITE,
            border: Border::default().rounded(8),
            ..Default::default()
        };
        match status {
            button::Status::Active | button::Status::Pressed => base,
            button::Status::Hovered => button::Style {
                background: Some(iced::Color::from_rgb(0.15, 0.4, 0.8).into()),
                ..base
            },
            button::Status::Disabled => button::Style {
                background: Some(Palette::OFFLINE.into()),
                ..base
            },
        }
    });

    // Create identity link
    let create_link = button(
        text("Create new identity")
            .size(14)
            .color(Palette::ORGANISATION),
    )
    .on_press(Message::Auth(AuthMessage::CreateIdentityPressed))
    .style(|_t: &Theme, status| {
        let base = button::Style {
            background: None,
            text_color: Palette::ORGANISATION,
            border: Border::default().rounded(6),
            ..Default::default()
        };
        match status {
            button::Status::Active | button::Status::Pressed => base,
            button::Status::Hovered => button::Style {
                background: Some(Palette::HOVER.into()),
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: Palette::OFFLINE,
                ..base
            },
        }
    });

    column![
        title,
        subtitle,
        Space::new().height(24),
        vault_section,
        Space::new().height(16),
        password_section,
        error_message,
        Space::new().height(16),
        login_button,
        Space::new().height(16),
        container(create_link)
            .width(Length::Fill)
            .center_x(Length::Fill),
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .into()
}

/// Render the create identity form.
fn view_create_identity(state: &AuthState) -> Element<'static, Message> {
    let title = text("Create Identity")
        .size(28)
        .color(Palette::TEXT_PRIMARY);

    let subtitle = text("Set up your Communitas identity")
        .size(14)
        .color(Palette::TEXT_MUTED);

    // Display name input
    let name_input = text_input("Display name...", &state.new_display_name)
        .on_input(|s| Message::Auth(AuthMessage::DisplayNameChanged(s)))
        .width(Length::Fill);

    let name_section = column![
        text("Display Name").size(12).color(Palette::TEXT_MUTED),
        name_input,
    ]
    .spacing(4);

    // Password input
    let password_input = text_input("Password...", &state.new_password)
        .on_input(|s| Message::Auth(AuthMessage::NewPasswordChanged(s)))
        .secure(true)
        .width(Length::Fill);

    let password_section = column![
        text("Password").size(12).color(Palette::TEXT_MUTED),
        password_input,
    ]
    .spacing(4);

    // Confirm password input
    let confirm_input = text_input("Confirm password...", &state.new_password_confirm)
        .on_input(|s| Message::Auth(AuthMessage::ConfirmPasswordChanged(s)))
        .on_submit(Message::Auth(AuthMessage::CreateIdentitySubmit))
        .secure(true)
        .width(Length::Fill);

    let confirm_section = column![
        text("Confirm Password").size(12).color(Palette::TEXT_MUTED),
        confirm_input,
    ]
    .spacing(4);

    // Error message
    let error_message: Element<'static, Message> = if let Some(ref error) = state.error {
        container(text(error.clone()).size(12).color(Palette::ERROR))
            .padding(8)
            .into()
    } else {
        Space::new().height(0).into()
    };

    // Validation using the state helper method
    let is_valid = state.can_create_identity();

    // Create button
    let create_button = button(
        text(if state.is_loading {
            "Creating..."
        } else {
            "Create Identity"
        })
        .width(Length::Fill)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .padding(12)
    .on_press_maybe(if state.is_loading || !is_valid {
        None
    } else {
        Some(Message::Auth(AuthMessage::CreateIdentitySubmit))
    })
    .style(|_t: &Theme, status| {
        let base = button::Style {
            background: Some(Palette::ORGANISATION.into()),
            text_color: iced::Color::WHITE,
            border: Border::default().rounded(8),
            ..Default::default()
        };
        match status {
            button::Status::Active | button::Status::Pressed => base,
            button::Status::Hovered => button::Style {
                background: Some(iced::Color::from_rgb(0.15, 0.4, 0.8).into()),
                ..base
            },
            button::Status::Disabled => button::Style {
                background: Some(Palette::OFFLINE.into()),
                ..base
            },
        }
    });

    // Cancel button
    let cancel_button = button(text("Back to login").size(14).color(Palette::TEXT_MUTED))
        .on_press(Message::Auth(AuthMessage::CancelCreate))
        .style(|_t: &Theme, status| {
            let base = button::Style {
                background: None,
                text_color: Palette::TEXT_MUTED,
                border: Border::default().rounded(6),
                ..Default::default()
            };
            match status {
                button::Status::Active | button::Status::Pressed => base,
                button::Status::Hovered => button::Style {
                    background: Some(Palette::HOVER.into()),
                    ..base
                },
                button::Status::Disabled => base,
            }
        });

    column![
        title,
        subtitle,
        Space::new().height(24),
        name_section,
        Space::new().height(16),
        password_section,
        Space::new().height(16),
        confirm_section,
        error_message,
        Space::new().height(16),
        create_button,
        Space::new().height(16),
        container(cancel_button)
            .width(Length::Fill)
            .center_x(Length::Fill),
    ]
    .spacing(4)
    .align_x(Alignment::Center)
    .into()
}
