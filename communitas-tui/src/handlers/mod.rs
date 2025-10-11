use crate::backend::Backend;
use crate::state::{AppState, View};
use anyhow::Result;

/// Handle back/escape key - pop view stack
pub fn handle_back(state: &mut AppState) {
    if state.navigation.pop_view().is_none() {
        // Already at root, do nothing
    }
}

/// Handle tab key - cycle focus between panels
pub fn handle_tab(state: &mut AppState) {
    state.navigation.cycle_focus();
}

/// Handle up arrow - move selection up
pub fn handle_up(state: &mut AppState) {
    state.navigation.select_previous();
}

/// Handle down arrow - move selection down
pub fn handle_down(state: &mut AppState) {
    let max = match state.navigation.current_view() {
        View::Dashboard => 4,
        View::Organizations => 0, // TODO: implement with entities
        View::Projects => 0,
        View::Groups => 0,
        View::Contacts => 0,
        View::Auth => 2,
        _ => 0,
    };
    state.navigation.select_next(max);
}

/// Handle left arrow
pub fn handle_left(state: &mut AppState) {
    match state.navigation.current_view() {
        View::Auth => {
            if state.navigation.selected_index > 0 {
                state.navigation.selected_index -= 1;
            }
        }
        _ => {}
    }
}

/// Handle right arrow  
pub fn handle_right(state: &mut AppState) {
    match state.navigation.current_view() {
        View::Auth => {
            if state.navigation.selected_index == 0 {
                state.navigation.selected_index = 1;
            }
        }
        _ => {}
    }
}

// ============================================================================
// ASYNC HANDLERS - STUBBED FOR NOW (HTTP API will be the main interface)
// ============================================================================

/// Handle enter key - TODO: Implement with new types
pub async fn handle_enter(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    state.set_status("TUI handlers temporarily disabled - use HTTP control API");
    Ok(())
}

/// Handle open organizations - TODO: Implement with new types
pub async fn handle_open_organizations(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    state.set_status("TUI handlers temporarily disabled - use HTTP control API");
    Ok(())
}

/// Handle network check - TODO: Implement with new types
pub async fn handle_check_network(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    let connected = backend.check_dht_connection().await.unwrap_or(false);
    state.set_status(&format!("Network: {}", if connected { "Connected" } else { "Offline" }));
    Ok(())
}

/// Handle login - TODO: Implement with new types
pub async fn handle_login(
    _state: &mut AppState,
    _backend: &mut Backend,
    _identity: String,
    _password: String,
) -> Result<()> {
    Ok(())
}

/// Handle signup - TODO: Implement with new types
pub async fn handle_signup(
    _state: &mut AppState,
    _backend: &mut Backend,
    _display_name: String,
    _password: String,
) -> Result<()> {
    Ok(())
}

/// Handle input submit - TODO: Implement with new types
pub async fn handle_input_submit(
    state: &mut AppState,
    _backend: &mut Backend,
    _input: String,
) -> Result<()> {
    state.set_status("TUI handlers temporarily disabled - use HTTP control API");
    Ok(())
}

/// Handle create thread - TODO: Implement with new types
pub async fn handle_create_thread(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    state.set_status("TUI handlers temporarily disabled - use HTTP control API");
    Ok(())
}

/// Handle add reaction - TODO: Implement with new types
pub async fn handle_add_reaction(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    state.set_status("TUI handlers temporarily disabled - use HTTP control API");
    Ok(())
}

// ============================================================================
// NAVIGATION HANDLERS - Simple state updates
// ============================================================================

/// Handle open projects view
pub fn handle_open_projects(state: &mut AppState) {
    state.navigation.push_view(View::Projects);
}

/// Handle open groups view
pub fn handle_open_groups(state: &mut AppState) {
    state.navigation.push_view(View::Groups);
}

/// Handle open contacts view
pub fn handle_open_contacts(state: &mut AppState) {
    state.navigation.push_view(View::Contacts);
}

/// Handle initialize identity view
pub fn handle_initialize_identity(state: &mut AppState) {
    state.set_status("Use HTTP control API to initialize identity");
}

/// Handle show help
pub fn handle_show_help(state: &mut AppState) {
    state.set_status("TUI help - use HTTP control API at http://localhost:3040 for automation");
}
