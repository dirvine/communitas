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

/// Handle down arrow - move selection down (max is context-dependent)
pub fn handle_down(state: &mut AppState) {
    // Get max items based on current view
    let max = match state.navigation.current_view() {
        View::Dashboard => 4, // 4 entity types
        View::Organizations => state.entities.channels.values().map(|v| v.len()).sum(),
        View::Projects => state.entities.projects.len(),
        View::Groups => state.entities.groups.len(),
        View::Contacts => state.entities.contacts.len(),
        View::Auth => 2, // Login or Signup
        _ => 0,
    };

    state.navigation.select_next(max);
}

/// Handle left arrow - move selection left (for horizontal navigation)
pub fn handle_left(state: &mut AppState) {
    match state.navigation.current_view() {
        View::Auth => {
            // Switch between login (0) and signup (1)
            if state.navigation.selected_index > 0 {
                state.navigation.selected_index -= 1;
            }
        }
        _ => {}
    }
}

/// Handle right arrow - move selection right (for horizontal navigation)
pub fn handle_right(state: &mut AppState) {
    match state.navigation.current_view() {
        View::Auth => {
            // Switch between login (0) and signup (1)
            if state.navigation.selected_index < 1 {
                state.navigation.selected_index += 1;
            }
        }
        _ => {}
    }
}

/// Handle enter key - select/open current item
pub async fn handle_enter(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    match state.navigation.current_view() {
        View::Auth => {
            // Activate input for login or signup
            state.activate_input();
        }
        View::Dashboard => {
            // Open selected entity type
            match state.navigation.selected_index {
                0 => handle_open_organizations(state, backend).await?,
                1 => handle_open_projects(state),
                2 => handle_open_groups(state),
                3 => handle_open_contacts(state),
                _ => {}
            }
        }
        View::Organizations => {
            // Open selected channel
            let mut all_channels = Vec::new();
            for channels in state.entities.channels.values() {
                all_channels.extend(channels.clone());
            }

            // Sort to match display order
            all_channels.sort_by(|a, b| {
                b.unread_count
                    .cmp(&a.unread_count)
                    .then_with(|| a.name.cmp(&b.name))
            });

            if let Some(channel) = all_channels.get(state.navigation.selected_index) {
                let channel_id = channel.id.clone();
                state.set_status("Loading messages...");

                // Load messages for the channel
                match backend.get_channel_messages(channel_id.clone()).await {
                    Ok(messages) => {
                        use crate::state::entities::MessageData;
                        let msg_data: Vec<MessageData> = messages
                            .iter()
                            .map(|m| {
                                let content_str = match &m.content {
                                    saorsa_core::chat::MessageContent::Text(text) => text.clone(),
                                    saorsa_core::chat::MessageContent::RichText {
                                        text, ..
                                    } => text.clone(),
                                    saorsa_core::chat::MessageContent::System(sys_msg) => {
                                        format!("[System: {:?}]", sys_msg)
                                    }
                                    saorsa_core::chat::MessageContent::Encrypted { .. } => {
                                        "[Encrypted Message]".to_string()
                                    }
                                };

                                MessageData {
                                    id: m.id.0.clone(),
                                    author_id: m.author.clone(),
                                    author_name: m.author.clone(), // TODO: Get display name from identity
                                    content: content_str,
                                    timestamp: m
                                        .created_at
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .map(|d| d.as_secs() as i64)
                                        .unwrap_or(0),
                                    thread_id: m.thread_id.as_ref().map(|tid| tid.0.clone()),
                                    thread_count: 0, // TODO: Get actual thread count
                                    reactions: Vec::new(), // TODO: Convert reactions
                                }
                            })
                            .collect();

                        state.entities.messages.insert(channel_id.clone(), msg_data);
                        state.set_status(format!("Loaded {} messages", messages.len()));
                    }
                    Err(e) => {
                        state.set_status(format!("Failed to load messages: {}", e));
                    }
                }

                state.navigation.navigate_to(View::Channel { channel_id });
                state.navigation.selected_index = 0; // Reset selection for messages
                state.activate_input(); // Activate input for messaging
            }
        }
        View::Channel { .. } => {
            // Activate input to send message
            state.activate_input();
        }
        View::Projects => {
            // Open selected project's Kanban board
            if let Some(project) = state.entities.projects.get(state.navigation.selected_index) {
                let project_id = project.id.clone();
                state
                    .navigation
                    .navigate_to(View::ProjectIssues { project_id });
                state.navigation.selected_index = 0; // Reset selection
            }
        }
        _ => {}
    }

    Ok(())
}

/// Handle 'o' key - open organizations view
pub async fn handle_open_organizations(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    if !backend.is_logged_in() {
        state.set_status("Please initialize identity first (press 'i')");
        return Ok(());
    }

    state.set_status("Loading channels...");
    state.navigation.navigate_to(View::Organizations);
    state.navigation.selected_index = 0; // Reset selection

    // Load channels in background
    match backend.get_channels().await {
        Ok(channels) => {
            state.set_status(format!("Loaded {} channels", channels.len()));

            // Convert backend channels to ChannelData and store
            use crate::state::entities::ChannelData;
            let channel_data: Vec<ChannelData> = channels
                .iter()
                .map(|c| ChannelData {
                    id: c.id.0.clone(),
                    name: c.name.clone(),
                    description: Some(c.description.clone()),
                    member_count: c.members.len(),
                    unread_count: 0, // TODO: Calculate actual unread count
                })
                .collect();

            // Store all channels under default org (for now)
            state
                .entities
                .channels
                .insert("default".to_string(), channel_data);
        }
        Err(e) => {
            state.set_status(format!("Failed to load channels: {}", e));
        }
    }

    Ok(())
}

/// Handle 'p' key - open projects view
pub fn handle_open_projects(state: &mut AppState) {
    state.set_status("Projects view");
    state.navigation.navigate_to(View::Projects);
    state.navigation.selected_index = 0; // Reset selection
}

/// Handle 'g' key - open groups view
pub fn handle_open_groups(state: &mut AppState) {
    state.set_status("Groups view not yet implemented");
    state.navigation.navigate_to(View::Groups);
}

/// Handle 'c' key - open contacts view
pub fn handle_open_contacts(state: &mut AppState) {
    state.set_status("Contacts view not yet implemented");
    state.navigation.navigate_to(View::Contacts);
}

/// Handle 'n' key - check network status
pub async fn handle_check_network(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    state.set_status("Checking network...");

    match backend.check_dht_connection().await {
        Ok(connected) => {
            if connected {
                state.set_status("Network: Connected");
            } else {
                state.set_status("Network: Disconnected");
            }
        }
        Err(e) => {
            state.set_status(format!("Network check failed: {}", e));
        }
    }

    Ok(())
}

/// Handle 'i' key - initialize identity (prompt)
pub fn handle_initialize_identity(state: &mut AppState) {
    state.set_status("Identity initialization requires restart with --identity flag");
}

/// Handle '?' or F1 - show help
pub fn handle_show_help(state: &mut AppState) {
    state.navigation.navigate_to(View::Help);
}

/// Handle login submission
pub async fn handle_login(
    state: &mut AppState,
    backend: &mut Backend,
    four_words: String,
) -> Result<()> {
    state.set_status("🔄 Logging in...");

    // Validate four-word format
    let words: Vec<&str> = four_words.split('-').collect();
    if words.len() != 4 {
        state.set_status("❌ Invalid format. Use: word-word-word-word");
        return Ok(());
    }

    // Login with existing identity (for now, use a placeholder password)
    // TODO: Implement proper password input in TUI
    let password = "default-password"; // This should be collected from user input

    state.set_status("🔐 Verifying identity...");

    match backend.login(&four_words, password).await {
        Ok(session_info) => {
            state.set_status(format!(
                "✅ Logged in as {} ({})",
                session_info.four_words, session_info.display_name
            ));

            // Initialize CoreContext for P2P features
            state.set_status("🌐 Connecting to network...");
            if let Err(e) = backend.initialize_core_context().await {
                tracing::warn!("Failed to initialize CoreContext: {}", e);
                state.set_status(format!("⚠️ Logged in (local mode): {}", e));
            } else {
                state.set_status("🚀 Connected successfully");
            }

            state.navigation.go_to_dashboard();
        }
        Err(e) => {
            state.set_status(format!("❌ Login failed: {}", e));
        }
    }

    Ok(())
}

/// Handle signup submission
pub async fn handle_signup(
    state: &mut AppState,
    backend: &mut Backend,
    display_name: String,
) -> Result<()> {
    state.set_status("🔄 Generating secure identity...");

    if display_name.is_empty() {
        state.set_status("❌ Display name cannot be empty");
        return Ok(());
    }

    // Generate new four-word identity
    let four_words = Backend::generate_four_words();
    state.set_status(&format!("🔑 Generated identity: {}", four_words));

    // Create vault for new identity (use placeholder password for now)
    // TODO: Implement proper password input in TUI
    let password = "default-password"; // This should be collected from user input

    state.set_status("🔒 Creating secure vault (this may take 10-30 seconds)...");

    match backend
        .create_vault_with_timeout(&four_words, password, &display_name)
        .await
    {
        Ok(session_info) => {
            state.set_status("✅ Vault created successfully");
            state.set_status(format!(
                "Welcome! Your identity: {} ({})",
                session_info.four_words, session_info.display_name
            ));

            // Initialize CoreContext for P2P features
            state.set_status("🌐 Initializing P2P features...");
            if let Err(e) = backend.initialize_core_context().await {
                tracing::warn!("Failed to initialize CoreContext: {}", e);
                state.set_status(format!("⚠️ Created (local mode): {}", e));
            } else {
                state.set_status("🚀 Ready for P2P collaboration");
            }

            state.navigation.go_to_dashboard();
        }
        Err(e) => {
            state.set_status(format!("❌ Signup failed: {}", e));
        }
    }

    Ok(())
}

/// Handle input submission (message send, thread reply, login, signup, etc.)
pub async fn handle_input_submit(
    state: &mut AppState,
    backend: &mut Backend,
    input: String,
) -> Result<()> {
    if input.is_empty() {
        return Ok(());
    }

    match state.navigation.current_view() {
        View::Auth => {
            // Handle login or signup based on selected option
            match state.navigation.selected_index {
                0 => handle_login(state, backend, input).await?,
                1 => handle_signup(state, backend, input).await?,
                _ => {}
            }
        }
        View::Channel { channel_id } => {
            // Send message to channel
            let channel_id = channel_id.clone();
            state.set_status("Sending message...");

            match backend.send_message_to_channel(channel_id, input).await {
                Ok(msg_id) => {
                    state.set_status(format!("Message sent: {}", msg_id));
                }
                Err(e) => {
                    state.set_status(format!("Failed to send message: {}", e));
                }
            }
        }
        View::Thread {
            channel_id,
            thread_id,
        } => {
            // Send reply to thread
            let channel_id = channel_id.clone();
            let thread_id = thread_id.clone();
            state.set_status("Sending reply...");

            match backend
                .send_thread_reply(channel_id, thread_id, input)
                .await
            {
                Ok(msg_id) => {
                    state.set_status(format!("Reply sent: {}", msg_id));
                }
                Err(e) => {
                    state.set_status(format!("Failed to send reply: {}", e));
                }
            }
        }
        _ => {}
    }

    Ok(())
}

/// Handle 't' key - create thread from selected message
pub async fn handle_create_thread(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    // Only works in channel view with a selected message
    if let View::Channel { channel_id } = state.navigation.current_view() {
        let channel_id = channel_id.clone();

        // Get messages for this channel
        if let Some(messages) = state.entities.messages.get(&channel_id) {
            if let Some(msg) = messages.get(state.navigation.selected_index) {
                let message_id = msg.id.clone();
                state.set_status("Creating thread...");

                match backend.create_thread(channel_id.clone(), message_id).await {
                    Ok(thread_id) => {
                        state.set_status("Thread created");
                        state.navigation.navigate_to(View::Thread {
                            channel_id,
                            thread_id,
                        });
                        state.activate_input();
                    }
                    Err(e) => {
                        state.set_status(format!("Failed to create thread: {}", e));
                    }
                }
            } else {
                state.set_status("No message selected");
            }
        }
    } else {
        state.set_status("Thread creation only works in channel view");
    }

    Ok(())
}

/// Handle 'r' key - add reaction to selected message
pub async fn handle_add_reaction(state: &mut AppState, backend: &mut Backend) -> Result<()> {
    // Only works in channel view with a selected message
    if let View::Channel { channel_id } = state.navigation.current_view() {
        let channel_id = channel_id.clone();

        // Get messages for this channel
        if let Some(messages) = state.entities.messages.get(&channel_id) {
            if let Some(msg) = messages.get(state.navigation.selected_index) {
                let message_id = msg.id.clone();

                // Common emoji reactions (like Slack/Discord)
                let emoji = "👍"; // Default thumb up for now
                // TODO: Show emoji picker UI

                state.set_status("Adding reaction...");

                match backend
                    .add_reaction(channel_id, message_id, emoji.to_string())
                    .await
                {
                    Ok(()) => {
                        state.set_status(format!("Added reaction: {}", emoji));
                    }
                    Err(e) => {
                        state.set_status(format!("Failed to add reaction: {}", e));
                    }
                }
            } else {
                state.set_status("No message selected");
            }
        }
    } else {
        state.set_status("Reactions only work in channel view");
    }

    Ok(())
}
