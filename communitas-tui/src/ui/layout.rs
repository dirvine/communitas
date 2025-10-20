use crate::state::{AppState, View};
use crate::ui::{
    auth, command_palette, contacts, context_menu, dashboard, direct_messages, groups, help,
    issue_detail, network_status, organizations, projects, status_bar,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Render main layout with status bar at bottom
pub fn render_layout(f: &mut Frame, state: &mut AppState) {
    // Record frame for performance monitoring
    state.performance_monitor.record_frame();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    // Render main content based on current view
    render_main_content(f, chunks[0], state);

    // Render status bar
    status_bar::render(f, chunks[1], state);

    // Render context menu as overlay
    context_menu::render(f, &state.context_menu);

    // Render command palette as overlay (appears on top of everything)
    command_palette::render(f, &state.command_palette);

    // TODO: Add advanced component rendering
    // The advanced components (PerformanceMonitor, ThemeManager, AccessibilityManager, ErrorRecovery)
    // use tuirealm::Frame which is incompatible with ratatui::Frame.
    // Options for future integration:
    // 1. Convert entire app to use tuirealm framework
    // 2. Create ratatui-compatible rendering functions for these components
    // 3. Use the components' state for logic but render manually with ratatui widgets
    //
    // For now, the components are initialized in AppState and their keyboard shortcuts work,
    // but visual rendering is deferred to future work.
}

/// Render main content area based on current view
fn render_main_content(f: &mut Frame, area: Rect, state: &AppState) {
    match state.navigation.current_view() {
        View::Auth => auth::render(f, area, state),
        View::Dashboard => dashboard::render(f, area, state),
        View::Organizations => render_organizations(f, area, state),
        View::Channel { .. } => render_channel(f, area, state),
        View::Thread { .. } => render_thread(f, area, state),
        View::Projects => render_projects(f, area, state),
        View::ProjectIssues { .. } => render_project_issues(f, area, state),
        View::IssueDetail { .. } => render_issue_detail(f, area, state),
        View::Groups => render_groups(f, area, state),
        View::GroupMessages { .. } => render_group_messages(f, area, state),
        View::Contacts => render_contacts(f, area, state),
        View::DirectMessages { .. } => render_direct_messages(f, area, state),
        View::NetworkStatus => render_network_status(f, area, state),
        View::Help => render_help(f, area, state),
    }
}

// Stub implementations for each view
fn render_organizations(f: &mut Frame, area: Rect, state: &AppState) {
    organizations::render(f, area, state);
}

fn render_channel(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::Channel { channel_id } = state.navigation.current_view() {
        organizations::render_channel_view(f, area, state, channel_id);
    }
}

fn render_thread(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::Thread {
        channel_id,
        thread_id,
    } = state.navigation.current_view()
    {
        organizations::render_thread_view(f, area, state, channel_id, thread_id);
    }
}

fn render_projects(f: &mut Frame, area: Rect, state: &AppState) {
    projects::render(f, area, state);
}

fn render_project_issues(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::ProjectIssues { project_id } = state.navigation.current_view() {
        projects::render_kanban_board(f, area, state, project_id);
    }
}

fn render_issue_detail(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::IssueDetail { issue_id } = state.navigation.current_view() {
        issue_detail::render(f, area, state, issue_id);
    }
}

fn render_groups(f: &mut Frame, area: Rect, state: &AppState) {
    groups::render(f, area, state);
}

fn render_group_messages(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::GroupMessages { group_id } = state.navigation.current_view() {
        groups::render_messages(f, area, state, group_id);
    }
}

fn render_contacts(f: &mut Frame, area: Rect, state: &AppState) {
    contacts::render(f, area, state);
}

fn render_direct_messages(f: &mut Frame, area: Rect, state: &AppState) {
    if let View::DirectMessages { contact_id } = state.navigation.current_view() {
        direct_messages::render(f, area, state, contact_id);
    }
}

fn render_network_status(f: &mut Frame, area: Rect, state: &AppState) {
    network_status::render(f, area, state);
}

fn render_help(f: &mut Frame, area: Rect, state: &AppState) {
    help::render(f, area, state);
}
