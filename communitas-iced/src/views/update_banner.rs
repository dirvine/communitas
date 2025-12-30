// Copyright (c) 2025 Saorsa Labs Limited
//
// Dual-licensed under the AGPL-3.0-or-later and a commercial license.
// You may use this file under the terms of the GNU Affero General Public License v3.0 or later.
// For commercial licensing, contact: saorsalabs@gmail.com

//! Update banner component for showing available updates.
//!
//! Displays a non-intrusive banner when a new version is available,
//! with options to update now, dismiss, or skip this version.

use crate::message::{Message, UpdateMessage};
use crate::theme::Palette;
use crate::update::{UpdateInfo, UpdateStatus};
use iced::widget::{Space, button, container, row, text};
use iced::{Alignment, Border, Element, Length, Theme};

/// Render the update banner if an update is available.
///
/// Returns `None` if no banner should be shown, or `Some(Element)` with the banner.
#[must_use]
pub fn view_update_banner(update_status: &UpdateStatus) -> Option<Element<'_, Message>> {
    match update_status {
        UpdateStatus::Available(info) => Some(view_available_banner(info)),
        UpdateStatus::Downloading { progress } => Some(view_downloading_banner(*progress)),
        UpdateStatus::Completed { new_version } => Some(view_completed_banner(new_version)),
        UpdateStatus::Failed(error) => Some(view_failed_banner(error)),
        _ => None,
    }
}

/// Render the "update available" banner.
fn view_available_banner(info: &UpdateInfo) -> Element<'_, Message> {
    let version_text = text(format!(
        "Update available: v{} -> v{}",
        info.current_version, info.new_version
    ))
    .size(13)
    .color(Palette::TEXT_PRIMARY);

    let update_button = button(text("Update Now").size(12).color(Palette::DEEP_FOREST))
        .padding([6, 12])
        .style(|_t: &Theme, status: button::Status| {
            let (bg, border) = match status {
                button::Status::Active => (Palette::JADE, Palette::JADE),
                button::Status::Hovered => (Palette::EMERALD, Palette::EMERALD),
                button::Status::Pressed => (Palette::EMERALD, Palette::EMERALD),
                button::Status::Disabled => (Palette::STONE, Palette::STONE),
            };
            button::Style {
                background: Some(bg.into()),
                text_color: Palette::DEEP_FOREST,
                border: Border::default().rounded(6).color(border),
                ..Default::default()
            }
        })
        .on_press(Message::Update(UpdateMessage::DownloadUpdate));

    let later_button = button(text("Later").size(12).color(Palette::STONE))
        .padding([6, 12])
        .style(ghost_button_style)
        .on_press(Message::Update(UpdateMessage::DismissUpdate));

    let skip_version = info.new_version.clone();
    let skip_button = button(text("Skip Version").size(12).color(Palette::STONE))
        .padding([6, 12])
        .style(ghost_button_style)
        .on_press(Message::Update(UpdateMessage::SkipVersion(skip_version)));

    let buttons = row![update_button, later_button, skip_button,]
        .spacing(8)
        .align_y(Alignment::Center);

    let content = row![version_text, Space::new().width(Length::Fill), buttons,]
        .spacing(16)
        .padding(12)
        .align_y(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::JADE.scale_alpha(0.15).into()),
            border: Border::default()
                .color(Palette::JADE.scale_alpha(0.3))
                .width(1.0)
                .rounded(8),
            ..Default::default()
        })
        .into()
}

/// Render the "downloading" banner.
fn view_downloading_banner(progress: u8) -> Element<'static, Message> {
    let progress_text = if progress > 0 {
        format!("Downloading update... {}%", progress)
    } else {
        "Downloading update...".to_string()
    };

    let content = row![text(progress_text).size(13).color(Palette::TEXT_PRIMARY),]
        .padding(12)
        .align_y(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::AMBER.scale_alpha(0.15).into()),
            border: Border::default()
                .color(Palette::AMBER.scale_alpha(0.3))
                .width(1.0)
                .rounded(8),
            ..Default::default()
        })
        .into()
}

/// Render the "update completed" banner.
fn view_completed_banner(new_version: &str) -> Element<'_, Message> {
    let message_text = text(format!(
        "Update to v{} complete! Restart to apply.",
        new_version
    ))
    .size(13)
    .color(Palette::TEXT_PRIMARY);

    let dismiss_button = button(text("Dismiss").size(12).color(Palette::STONE))
        .padding([6, 12])
        .style(ghost_button_style)
        .on_press(Message::Update(UpdateMessage::DismissUpdate));

    let content = row![
        message_text,
        Space::new().width(Length::Fill),
        dismiss_button,
    ]
    .spacing(16)
    .padding(12)
    .align_y(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::ONLINE.scale_alpha(0.15).into()),
            border: Border::default()
                .color(Palette::ONLINE.scale_alpha(0.3))
                .width(1.0)
                .rounded(8),
            ..Default::default()
        })
        .into()
}

/// Render the "update failed" banner.
fn view_failed_banner(error: &str) -> Element<'_, Message> {
    let error_text = text(format!("Update failed: {}", error))
        .size(13)
        .color(Palette::ERROR);

    let retry_button = button(text("Retry").size(12).color(Palette::STONE))
        .padding([6, 12])
        .style(ghost_button_style)
        .on_press(Message::Update(UpdateMessage::CheckForUpdates));

    let dismiss_button = button(text("Dismiss").size(12).color(Palette::STONE))
        .padding([6, 12])
        .style(ghost_button_style)
        .on_press(Message::Update(UpdateMessage::DismissUpdate));

    let content = row![
        error_text,
        Space::new().width(Length::Fill),
        retry_button,
        dismiss_button,
    ]
    .spacing(8)
    .padding(12)
    .align_y(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .style(|_t: &Theme| container::Style {
            background: Some(Palette::ERROR.scale_alpha(0.15).into()),
            border: Border::default()
                .color(Palette::ERROR.scale_alpha(0.3))
                .width(1.0)
                .rounded(8),
            ..Default::default()
        })
        .into()
}

/// Ghost button style for secondary actions.
fn ghost_button_style(_theme: &Theme, status: button::Status) -> button::Style {
    let bg = match status {
        button::Status::Active => iced::Color::TRANSPARENT,
        button::Status::Hovered => Palette::HOVER,
        button::Status::Pressed => Palette::HOVER,
        button::Status::Disabled => iced::Color::TRANSPARENT,
    };
    button::Style {
        background: Some(bg.into()),
        text_color: Palette::STONE,
        border: Border::default().rounded(6),
        ..Default::default()
    }
}
