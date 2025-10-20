mod auth;
mod command_palette;
mod contacts;
mod context_menu;
mod dashboard;
mod direct_messages;
mod groups;
mod help;
mod issue_detail;
mod layout;
mod network_status;
pub mod organizations;
mod projects;
mod status_bar;

use crate::state::AppState;
use ratatui::Frame;

/// Main render function - dispatches to appropriate view
pub fn render(f: &mut Frame, state: &mut AppState) {
    layout::render_layout(f, state);
}
