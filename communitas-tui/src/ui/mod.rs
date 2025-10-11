mod auth;
mod dashboard;
mod layout;
pub mod organizations;
mod projects;
mod status_bar;

use crate::state::AppState;
use ratatui::Frame;

/// Main render function - dispatches to appropriate view
pub fn render(f: &mut Frame, state: &AppState) {
    layout::render_layout(f, state);
}
