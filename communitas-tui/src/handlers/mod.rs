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
    use crate::state::navigation::FocusedPanel;

    // Check if TreeView (Sidebar) is focused in Dashboard
    if matches!(state.navigation.focused_panel, FocusedPanel::Sidebar)
        && matches!(state.navigation.current_view(), View::Dashboard)
    {
        state.tree_view.navigate_up();
    } else {
        state.navigation.select_previous();
    }
}

/// Handle down arrow - move selection down
pub fn handle_down(state: &mut AppState) {
    use crate::state::navigation::FocusedPanel;

    // Check if TreeView (Sidebar) is focused in Dashboard
    if matches!(state.navigation.focused_panel, FocusedPanel::Sidebar)
        && matches!(state.navigation.current_view(), View::Dashboard)
    {
        state.tree_view.navigate_down();
    } else {
        let max = match state.navigation.current_view() {
            View::Dashboard => 4,
            View::Organizations => state.entities.channels.len().saturating_sub(1),
            View::Projects => state.entities.projects.len().saturating_sub(1),
            View::Groups => state.entities.groups.len().saturating_sub(1),
            View::Contacts => state.entities.contacts.len().saturating_sub(1),
            View::Auth => 2,
            _ => 0,
        };
        state.navigation.select_next(max);
    }
}

/// Handle left arrow
pub fn handle_left(state: &mut AppState) {
    use crate::state::navigation::FocusedPanel;

    // Check if TreeView (Sidebar) is focused in Dashboard
    if matches!(state.navigation.focused_panel, FocusedPanel::Sidebar)
        && matches!(state.navigation.current_view(), View::Dashboard)
    {
        // Collapse current node if expanded
        if let Some(selected_id) = state.tree_view.selected() {
            let selected_id = selected_id.to_string();
            if state.tree_view.is_expanded(&selected_id) {
                state.tree_view.collapse_node(&selected_id);
            }
        }
    } else if state.navigation.current_view() == &View::Auth && state.navigation.selected_index > 0
    {
        state.navigation.selected_index -= 1;
    }
}

/// Handle right arrow
pub fn handle_right(state: &mut AppState) {
    use crate::state::navigation::FocusedPanel;

    // Check if TreeView (Sidebar) is focused in Dashboard
    if matches!(state.navigation.focused_panel, FocusedPanel::Sidebar)
        && matches!(state.navigation.current_view(), View::Dashboard)
    {
        // Expand current node if not expanded
        if let Some(selected_id) = state.tree_view.selected() {
            let selected_id = selected_id.to_string();
            if !state.tree_view.is_expanded(&selected_id) {
                state.tree_view.expand_node(&selected_id);
            }
        }
    } else if state.navigation.current_view() == &View::Auth && state.navigation.selected_index == 0
    {
        state.navigation.selected_index = 1;
    }
}

/// Handle space key - toggle TreeView expansion when in Sidebar
pub fn handle_space(state: &mut AppState) {
    use crate::state::navigation::FocusedPanel;

    // Check if TreeView (Sidebar) is focused in Dashboard
    if matches!(state.navigation.focused_panel, FocusedPanel::Sidebar)
        && matches!(state.navigation.current_view(), View::Dashboard)
    {
        // Toggle expansion of current node
        if let Some(selected_id) = state.tree_view.selected() {
            let selected_id = selected_id.to_string();
            state.tree_view.toggle_expanded(&selected_id);
        }
    }
    // If not in TreeView, space does nothing (could be extended for other views)
}

// ============================================================================
// ASYNC HANDLERS - STUBBED FOR NOW (HTTP API will be the main interface)
// ============================================================================

/// Handle enter key
pub async fn handle_enter(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    use crate::state::navigation::FocusedPanel;

    match state.navigation.current_view() {
        View::Auth => {
            // Check if we're in input mode
            if matches!(state.navigation.focused_panel, FocusedPanel::Input) {
                // Submit the form
                let input = state.take_input();
                state.navigation.focused_panel = FocusedPanel::Main;

                match state.navigation.selected_index {
                    0 => {
                        // Login flow
                        if input.is_empty() {
                            state.set_status("Please enter a four-word identity");
                            return Ok(());
                        }

                        state.set_status(format!("Logging in with identity: {}", input));

                        // Attempt login with backend using default password
                        // NOTE: For production, implement proper password/passkey input
                        let default_password = "communitas-tui-password";

                        match backend.login(&input, default_password).await {
                            Ok(session_info) => {
                                // Set identity in state
                                state.set_identity(
                                    session_info.four_words.clone(),
                                    session_info.display_name.clone(),
                                );

                                // Initialize CoreContext for P2P features
                                if let Err(e) = backend.initialize_core_context().await {
                                    tracing::warn!("Failed to initialize CoreContext: {}", e);
                                    state.set_status(format!("Logged in (local mode): {}", e));
                                } else {
                                    state.set_status(format!(
                                        "Logged in: {} ({})",
                                        session_info.display_name, session_info.four_words
                                    ));
                                }

                                // Reset state for dashboard
                                state.deactivate_input();
                                state.navigation.focused_panel = FocusedPanel::Main;
                                state.navigation.selected_index = 0;

                                // Transition to dashboard
                                state.navigation.push_view(View::Dashboard);
                            }
                            Err(e) => {
                                state.set_status(format!("Login failed: {}", e));
                                tracing::error!("Login error: {}", e);
                            }
                        }
                    }
                    1 => {
                        // Signup flow
                        if input.is_empty() {
                            state.set_status("Please enter a display name");
                            return Ok(());
                        }

                        state.set_status(format!("Creating new identity for: {}", input));

                        // Generate a new four-word identity
                        let four_words = Backend::generate_four_words();
                        tracing::info!("Generated four-word identity: {}", four_words);

                        // Create vault with default password (in production, you'd ask for password)
                        let default_password = "communitas-tui-password";

                        match backend
                            .create_vault_with_timeout(&four_words, default_password, &input)
                            .await
                        {
                            Ok(session_info) => {
                                tracing::info!(
                                    "Vault created successfully: {}",
                                    session_info.four_words
                                );

                                // Initialize core context
                                if let Err(e) = backend.initialize_core_context().await {
                                    tracing::error!("Failed to initialize core context: {}", e);
                                    state.set_status(format!("Error initializing: {}", e));
                                    return Ok(());
                                }

                                state.set_status(format!("✓ Identity created: {}", four_words));

                                // Transition to dashboard
                                state.navigation.push_view(View::Dashboard);
                            }
                            Err(e) => {
                                tracing::error!("Failed to create vault: {}", e);
                                state.set_status(format!("Error creating identity: {}", e));
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                // Activate input mode
                state.activate_input();
                state.navigation.focused_panel = FocusedPanel::Input;

                match state.navigation.selected_index {
                    0 => state.set_status("Enter your four-word identity"),
                    1 => state.set_status("Enter your display name"),
                    _ => {}
                }
            }
        }
        _ => {
            state.set_status("TUI handlers temporarily disabled - use HTTP control API");
        }
    }

    Ok(())
}

/// Handle open organizations
pub async fn handle_open_organizations(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    state.navigation.push_view(View::Organizations);
    state.set_status("Organizations - Press 'n' to create new channel");
    Ok(())
}

/// Handle network check - TODO: Implement with new types
pub async fn handle_check_network(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    let connected = backend.check_dht_connection().await.unwrap_or(false);
    state.set_status(format!(
        "Network: {}",
        if connected { "Connected" } else { "Offline" }
    ));
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

/// Handle input submit
pub async fn handle_input_submit(
    state: &mut AppState,
    backend: &mut Backend,
    input: String,
) -> Result<()> {
    use crate::state::navigation::FocusedPanel;

    match state.navigation.current_view() {
        View::Auth => {
            // Handle auth form submission
            state.navigation.focused_panel = FocusedPanel::Main;

            match state.navigation.selected_index {
                0 => {
                    // Login flow
                    if input.is_empty() {
                        state.set_status("Please enter a four-word identity");
                        return Ok(());
                    }

                    state.set_status(format!("Logging in with identity: {}", input));

                    // Attempt login with backend using default password
                    let default_password = "communitas-tui-password";

                    match backend.login(&input, default_password).await {
                        Ok(session_info) => {
                            state.set_identity(
                                session_info.four_words.clone(),
                                session_info.display_name.clone(),
                            );

                            if let Err(e) = backend.initialize_core_context().await {
                                tracing::warn!("Failed to initialize CoreContext: {}", e);
                                state.set_status(format!("Logged in (local mode): {}", e));
                            } else {
                                state.set_status(format!(
                                    "Logged in: {} ({})",
                                    session_info.display_name, session_info.four_words
                                ));
                            }

                            // Reset state for dashboard
                            state.navigation.focused_panel = FocusedPanel::Main;
                            state.navigation.selected_index = 0;

                            state.navigation.push_view(View::Dashboard);
                        }
                        Err(e) => {
                            state.set_status(format!("Login failed: {}", e));
                            tracing::error!("Login error: {}", e);
                        }
                    }
                }
                1 => {
                    // Signup flow
                    if input.is_empty() {
                        state.set_status("Please enter a display name");
                        return Ok(());
                    }

                    state.set_status(format!("Creating new identity for: {}", input));

                    // Generate a new four-word identity
                    let four_words = Backend::generate_four_words();
                    tracing::info!("Generated four-word identity: {}", four_words);

                    // Create vault with default password
                    let default_password = "communitas-tui-password";

                    match backend
                        .create_vault_with_timeout(&four_words, default_password, &input)
                        .await
                    {
                        Ok(session_info) => {
                            tracing::info!(
                                "Vault created successfully: {}",
                                session_info.four_words
                            );

                            // Initialize core context
                            if let Err(e) = backend.initialize_core_context().await {
                                tracing::error!("Failed to initialize core context: {}", e);
                                state.set_status(format!("Error initializing: {}", e));
                                return Ok(());
                            }

                            state.set_status(format!("✓ Identity created: {}", four_words));

                            // Reset state for dashboard
                            state.deactivate_input();
                            state.navigation.focused_panel = FocusedPanel::Main;
                            state.navigation.selected_index = 0;

                            // Transition to dashboard
                            state.navigation.push_view(View::Dashboard);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create vault: {}", e);
                            state.set_status(format!("Error creating identity: {}", e));
                        }
                    }
                }
                _ => {}
            }
        }
        View::Organizations | View::Projects | View::Groups | View::Contacts => {
            // Creating a new entity in one of these views
            state.navigation.focused_panel = FocusedPanel::Main;
            handle_submit_new_entity(state, backend, input).await?;
        }
        _ => {
            state.set_status("Input not supported in this view");
        }
    }

    Ok(())
}

/// Handle creating a new entity based on current view
pub async fn handle_create_entity(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    use crate::state::navigation::FocusedPanel;

    // Determine which type of entity to create based on current view
    let entity_type_to_create = match state.navigation.current_view() {
        View::Organizations => Some(communitas_core::crdt::EntityType::Channel),
        View::Projects => Some(communitas_core::crdt::EntityType::Project),
        View::Groups => Some(communitas_core::crdt::EntityType::Group),
        View::Contacts => Some(communitas_core::crdt::EntityType::Person),
        _ => None,
    };

    if let Some(_entity_type) = entity_type_to_create {
        // Activate input mode to collect entity name
        state.activate_input();
        state.navigation.focused_panel = FocusedPanel::Input;
        state.set_status("Enter name for new entity");
    } else {
        state.set_status("Cannot create entity from this view");
    }

    Ok(())
}

/// Handle submitting a new entity creation
pub async fn handle_submit_new_entity(
    state: &mut AppState,
    backend: &mut Backend,
    name: String,
) -> Result<()> {
    // Determine which type of entity to create based on current view
    let entity_type = match state.navigation.current_view() {
        View::Organizations => communitas_core::crdt::EntityType::Channel,
        View::Projects => communitas_core::crdt::EntityType::Project,
        View::Groups => communitas_core::crdt::EntityType::Group,
        View::Contacts => communitas_core::crdt::EntityType::Person,
        _ => {
            state.set_status("Cannot create entity from this view");
            return Ok(());
        }
    };

    state.set_status(format!("Creating {}...", name));

    // Create the entity (validation now happens in backend)
    match backend
        .create_entity(name.clone(), entity_type, vec![])
        .await
    {
        Ok(entity) => {
            state.set_status(format!("✓ Created: {}", entity.name));
            tracing::info!("Created entity: {} ({})", entity.name, entity.id);

            // Refresh the view to show the new entity
            // (In a real implementation, this would trigger a data reload)
        }
        Err(e) => {
            // Backend validation errors will be user-friendly
            state.set_status(format!("Error: {}", e));
            tracing::error!("Failed to create entity: {}", e);
        }
    }

    Ok(())
}

/// Handle create thread (reply to selected message)
pub async fn handle_create_thread(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    use crate::state::navigation::FocusedPanel;

    // Activate input mode for thread reply
    // In a real implementation, we'd track which message is selected
    state.activate_input();
    state.navigation.focused_panel = FocusedPanel::Input;
    state.set_status("Enter your reply to create thread");

    Ok(())
}

/// Handle add reaction (add emoji reaction to selected message)
pub async fn handle_add_reaction(state: &mut AppState, _backend: &mut Backend) -> Result<()> {
    // Emoji picker would be shown here in a full implementation
    // For TUI, we could show a simple list of common emojis
    state.set_status("Reactions: Press 👍 (1), ❤️ (2), 😂 (3), 🎉 (4) for quick reactions");
    Ok(())
}

// ============================================================================
// NAVIGATION HANDLERS - Simple state updates
// ============================================================================

/// Handle open projects view
pub fn handle_open_projects(state: &mut AppState) {
    state.navigation.push_view(View::Projects);
    state.set_status("Projects - Press 'n' to create new project");
}

/// Handle open groups view
pub fn handle_open_groups(state: &mut AppState) {
    state.navigation.push_view(View::Groups);
    state.set_status("Groups - Press 'n' to create new group");
}

/// Handle open contacts view
pub fn handle_open_contacts(state: &mut AppState) {
    state.navigation.push_view(View::Contacts);
    state.set_status("Contacts - Press 'n' to add new contact");
}

/// Handle initialize identity view
pub fn handle_initialize_identity(state: &mut AppState) {
    state.set_status("Use HTTP control API to initialize identity");
}

/// Handle show help
pub fn handle_show_help(state: &mut AppState) {
    state.set_status("TUI help - use HTTP control API at http://localhost:3040 for automation");
}
