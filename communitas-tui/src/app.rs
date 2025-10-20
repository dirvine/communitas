use crate::backend::Backend;
use crate::components::{ComponentArea, EnhancedMouseEvent, MenuContext, classify_mouse_event};
use crate::handlers;
use crate::state::{AppState, ConnectionStatus};
use crate::ui;
use anyhow::Result;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// Main TUI application
pub struct App {
    /// Application state
    state: AppState,
    /// Backend integration
    backend: Backend,
    /// Terminal interface
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl App {
    /// Create new application
    pub async fn new(data_dir: PathBuf, offline: bool) -> Result<Self> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend_term = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend_term)?;

        let state = AppState::new();
        let backend = Backend::new(data_dir, offline).await?;

        Ok(Self {
            state,
            backend,
            terminal,
        })
    }

    /// Create new application with custom configuration
    pub async fn new_with_config(
        data_dir: PathBuf,
        pbkdf2_iterations: u32,
        use_keyring: bool,
        offline: bool,
    ) -> Result<Self> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend_term = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend_term)?;

        let state = AppState::new();
        let backend =
            Backend::new_with_config(data_dir, pbkdf2_iterations, use_keyring, offline).await?;

        Ok(Self {
            state,
            backend,
            terminal,
        })
    }

    /// Start with authentication screen (no identity provided)
    pub fn start_with_auth(&mut self) {
        use crate::state::View;
        self.state.navigation.view_stack.clear();
        self.state.navigation.view_stack.push(View::Auth);
        self.state
            .set_status("Welcome! Please login or signup to continue");
    }

    /// Initialize identity and CoreContext
    pub async fn initialize_identity(
        &mut self,
        four_words: String,
        _display_name: String,
        _device_name: String,
    ) -> Result<()> {
        self.state.set_status("Initializing identity...");

        self.state.network.set_status(ConnectionStatus::Connecting);

        // TODO: Implement proper password input
        let password = "default-password";

        // Try to login with existing vault
        match self.backend.login(&four_words, password).await {
            Ok(session_info) => {
                self.state.set_identity(
                    session_info.four_words.clone(),
                    session_info.display_name.clone(),
                );

                // Initialize CoreContext for P2P features
                if let Err(e) = self.backend.initialize_core_context().await {
                    tracing::warn!("Failed to initialize CoreContext: {}", e);
                    self.state
                        .set_status(format!("Logged in (local mode): {}", e));
                    self.state
                        .network
                        .set_status(ConnectionStatus::Disconnected);
                } else {
                    self.state.network.set_status(ConnectionStatus::Connected);
                    self.state.set_status("Identity initialized successfully");
                }
                Ok(())
            }
            Err(e) => {
                self.state
                    .network
                    .set_status(ConnectionStatus::Error(e.to_string()));
                self.state
                    .set_status(format!("Failed to initialize identity: {}", e));
                Err(e)
            }
        }
    }

    /// Main event loop
    pub async fn run(&mut self) -> Result<()> {
        use std::time::Duration;

        loop {
            // Update animations (assuming ~60fps = 16ms per frame)
            self.state.animation_manager.update_all();

            // Draw UI
            self.terminal.draw(|f| ui::render(f, &mut self.state))?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        if self.handle_key_event(key).await? {
                            break; // Should quit
                        }
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse).await?;
                    }
                    Event::Resize(_, _) => {
                        // Terminal was resized, re-draw on next iteration
                    }
                    _ => {}
                }
            }

            if self.state.should_quit {
                break;
            }
        }

        self.cleanup()?;
        Ok(())
    }

    /// Handle keyboard input
    async fn handle_key_event(&mut self, key: event::KeyEvent) -> Result<bool> {
        // Handle Ctrl+C for quit
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.state.should_quit = true;
            return Ok(true);
        }

        // Handle advanced component keyboard shortcuts (these take priority)
        // F12: Toggle performance monitor
        if key.code == KeyCode::F(12) {
            self.state.performance_monitor.toggle_visibility();
            self.state.set_status(format!(
                "Performance monitor {}",
                if self.state.performance_monitor.is_visible() {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
            return Ok(false);
        }

        // Ctrl+T: Toggle theme preview
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
            self.state.theme_manager.toggle_theme_preview();
            self.state.set_status(format!(
                "Theme preview {}",
                if self.state.theme_manager.is_preview_visible() {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
            return Ok(false);
        }

        // Ctrl+S: Toggle screen reader
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
            use crate::components::AccessibilitySetting;
            self.state
                .accessibility_manager
                .toggle_setting(AccessibilitySetting::ScreenReader);
            self.state.set_status(format!(
                "Screen reader {}",
                if self.state.accessibility_manager.settings().screen_reader {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
            return Ok(false);
        }

        // Ctrl+H: Toggle high contrast
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('h') {
            use crate::components::AccessibilitySetting;
            self.state
                .accessibility_manager
                .toggle_setting(AccessibilitySetting::HighContrast);
            self.state.set_status(format!(
                "High contrast {}",
                if self.state.accessibility_manager.settings().high_contrast {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
            return Ok(false);
        }

        // Ctrl+\: Reset split panel to default position (Dashboard only)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('\\') {
            use crate::state::View;
            if let View::Dashboard = self.state.navigation.current_view() {
                self.state.resizable_split.reset_position();
                self.state.set_status(format!(
                    "Split panel reset to {}%",
                    self.state.resizable_split.position()
                ));
                return Ok(false);
            }
        }

        // Ctrl+K: Toggle command palette (global)
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('k') {
            use crate::components::Animation;
            use std::time::Duration;

            if self.state.command_palette.is_visible() {
                // Hiding - add fade-out animation
                self.state.command_palette.hide();
                self.state.animation_manager.add(
                    "command_palette_fade",
                    Animation::fade_out(Duration::from_millis(200)),
                );
            } else {
                // Showing - add fade-in animation
                self.state.command_palette.show();
                self.state.animation_manager.add(
                    "command_palette_fade",
                    Animation::fade_in(Duration::from_millis(200)),
                );
            }
            return Ok(false);
        }

        // Alt+K: Toggle keyboard help
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('k') {
            use crate::components::AccessibilitySetting;
            self.state
                .accessibility_manager
                .toggle_setting(AccessibilitySetting::KeyboardHelp);
            self.state.set_status(format!(
                "Keyboard help {}",
                if self.state.accessibility_manager.is_keyboard_help_visible() {
                    "enabled"
                } else {
                    "disabled"
                }
            ));
            return Ok(false);
        }

        // If input mode is active, handle text input
        if self.state.input_active {
            match key.code {
                KeyCode::Char(c) => {
                    self.state.push_input_char(c);
                }
                KeyCode::Backspace => {
                    self.state.pop_input_char();
                }
                KeyCode::Enter => {
                    let input = self.state.take_input();
                    self.state.deactivate_input();
                    handlers::handle_input_submit(&mut self.state, &mut self.backend, input)
                        .await?;
                }
                KeyCode::Esc => {
                    self.state.deactivate_input();
                }
                _ => {}
            }
            return Ok(false);
        }

        // If command palette is active, handle palette-specific keys
        if self.state.command_palette.is_visible() {
            use crate::components::Animation;
            use std::time::Duration;

            match key.code {
                KeyCode::Enter => {
                    // Execute selected command
                    if let Some(cmd) = self.state.command_palette.selected_command() {
                        let cmd_id = cmd.id.clone();
                        self.state.command_palette.hide();
                        // Add fade-out animation
                        self.state.animation_manager.add(
                            "command_palette_fade",
                            Animation::fade_out(Duration::from_millis(200)),
                        );
                        self.execute_command(&cmd_id).await?;
                    }
                }
                KeyCode::Up => {
                    self.state.command_palette.select_previous();
                }
                KeyCode::Down => {
                    self.state.command_palette.select_next();
                }
                KeyCode::Esc => {
                    // First Esc: clear query, Second Esc: close palette
                    if self.state.command_palette.query().is_empty() {
                        self.state.command_palette.hide();
                        // Add fade-out animation
                        self.state.animation_manager.add(
                            "command_palette_fade",
                            Animation::fade_out(Duration::from_millis(200)),
                        );
                    } else {
                        self.state.command_palette.clear_query();
                    }
                }
                KeyCode::Backspace => {
                    // Remove last char from query
                    let query = self.state.command_palette.query();
                    if !query.is_empty() {
                        let new_query = query[..query.len() - 1].to_string();
                        self.state.command_palette.set_query(&new_query);
                    }
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Add char to query (unless it's a control char)
                    let mut query = self.state.command_palette.query().to_string();
                    query.push(c);
                    self.state.command_palette.set_query(&query);
                }
                _ => {}
            }
            return Ok(false);
        }

        // Handle navigation keys
        match key.code {
            KeyCode::Char('q') => {
                self.state.should_quit = true;
                return Ok(true);
            }
            KeyCode::Esc => {
                handlers::handle_back(&mut self.state);
            }
            KeyCode::Tab => {
                handlers::handle_tab(&mut self.state);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                handlers::handle_up(&mut self.state);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                handlers::handle_down(&mut self.state);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                handlers::handle_left(&mut self.state);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                handlers::handle_right(&mut self.state);
            }
            KeyCode::Enter => {
                handlers::handle_enter(&mut self.state, &mut self.backend).await?;
            }
            KeyCode::Char(' ') => {
                handlers::handle_space(&mut self.state);
            }
            KeyCode::Char('o') => {
                handlers::handle_open_organizations(&mut self.state, &mut self.backend).await?;
            }
            KeyCode::Char('p') => {
                handlers::handle_open_projects(&mut self.state);
            }
            KeyCode::Char('g') => {
                handlers::handle_open_groups(&mut self.state);
            }
            KeyCode::Char('c') => {
                handlers::handle_open_contacts(&mut self.state);
            }
            KeyCode::Char('n') => {
                // Context-sensitive: create new entity in list views, otherwise check network
                use crate::state::View;
                match self.state.navigation.current_view() {
                    View::Organizations | View::Projects | View::Groups | View::Contacts => {
                        handlers::handle_create_entity(&mut self.state, &mut self.backend).await?;
                    }
                    _ => {
                        handlers::handle_check_network(&mut self.state, &mut self.backend).await?;
                    }
                }
            }
            KeyCode::Char('i') => {
                handlers::handle_initialize_identity(&mut self.state);
            }
            KeyCode::Char('?') | KeyCode::F(1) => {
                handlers::handle_show_help(&mut self.state);
            }
            KeyCode::Char('t') => {
                handlers::handle_create_thread(&mut self.state, &mut self.backend).await?;
            }
            KeyCode::Char('r') => {
                handlers::handle_add_reaction(&mut self.state, &mut self.backend).await?;
            }
            _ => {}
        }

        Ok(false)
    }

    /// Handle mouse input with enhanced capabilities (hover, drag, scroll, context menus)
    async fn handle_mouse_event(&mut self, mouse: event::MouseEvent) -> Result<()> {
        use crate::state::View;

        let col = mouse.column;
        let row = mouse.row;

        // Get terminal size for component area calculations
        let size = self.terminal.size()?;

        // Define component areas based on terminal size
        // Sidebar: left 20 columns
        let sidebar_area = ComponentArea::new(0, 0, 20, size.height);

        // Message list: center area (organizations/projects/etc views)
        let message_list_area = ComponentArea::new(
            20,
            5,
            size.width.saturating_sub(20),
            size.height.saturating_sub(9),
        );

        // Input area: bottom area
        let input_area = ComponentArea::new(
            20,
            size.height.saturating_sub(4),
            size.width.saturating_sub(20),
            4,
        );

        // Classify the mouse event using our enhanced system
        let enhanced_event = classify_mouse_event(mouse, &self.state.drag_state);

        // Handle context menu if visible (takes priority)
        if self.state.context_menu.visible {
            if let Some(action) = self.state.context_menu.handle_mouse(mouse) {
                // Execute menu action
                match action {
                    crate::components::MenuAction::Reply => {
                        // Handle reply action
                        tracing::debug!("Reply action triggered");
                    }
                    crate::components::MenuAction::Edit => {
                        tracing::debug!("Edit action triggered");
                    }
                    crate::components::MenuAction::Delete => {
                        tracing::debug!("Delete action triggered");
                    }
                    crate::components::MenuAction::Copy => {
                        tracing::debug!("Copy action triggered");
                    }
                    _ => {
                        tracing::debug!("Menu action triggered: {:?}", action);
                    }
                }
            }
            return Ok(());
        }

        // Handle dashboard split divider interactions (only in dashboard view)
        if let View::Dashboard = self.state.navigation.current_view() {
            // Calculate divider position based on terminal size and split percentage
            // Dashboard layout: margin(2) + tabs(3) + content + instructions(8)
            let content_y = 2 + 3; // margin + tabs
            let content_height = size.height.saturating_sub(2 + 3 + 8); // Total - margin - tabs - instructions
            let content_area = ratatui::layout::Rect::new(
                2,
                content_y,
                size.width.saturating_sub(4),
                content_height,
            );

            let split_position = self.state.resizable_split.position();
            let divider_x = content_area.x + ((content_area.width * split_position) / 100);

            // Check if mouse is over divider (tolerance of 1 column on either side)
            let is_over_divider = col >= divider_x.saturating_sub(1)
                && col <= divider_x.saturating_add(1)
                && row >= content_area.y
                && row < content_area.y + content_area.height;

            if is_over_divider {
                // Mouse is over divider - handle divider-specific events
                if let Some(event) = &enhanced_event {
                    match event {
                        EnhancedMouseEvent::LeftClick { x, y } => {
                            // Start drag on click
                            self.state.resizable_split.start_drag(*x, *y);
                            tracing::debug!("Started divider drag at x={}", x);
                        }
                        EnhancedMouseEvent::DragStart { x, y } => {
                            // Start drag
                            self.state.resizable_split.start_drag(*x, *y);
                            tracing::debug!("Divider drag started at x={}", x);
                        }
                        EnhancedMouseEvent::Dragging { x, y, .. } => {
                            // Update drag position
                            // Convert screen x to percentage of content area
                            let relative_x = x.saturating_sub(content_area.x);
                            let percentage = ((relative_x * 100) / content_area.width).min(100);

                            self.state.resizable_split.update_drag(percentage, *y);
                            tracing::debug!("Divider dragged to {}%", percentage);
                        }
                        EnhancedMouseEvent::MouseUp { .. } => {
                            // End drag
                            self.state.resizable_split.end_drag();
                            tracing::debug!(
                                "Divider drag ended at {}%",
                                self.state.resizable_split.position()
                            );
                        }
                        EnhancedMouseEvent::Hover { x, y } => {
                            // Update hover state
                            self.state.resizable_split.update_hover(*x, *y);
                        }
                        _ => {}
                    }
                    return Ok(()); // Divider consumed the event
                } else {
                    // Update hover even without enhanced event
                    self.state.resizable_split.update_hover(col, row);
                }
            }
        }

        // Process enhanced mouse events
        if let Some(event) = enhanced_event {
            match event {
                EnhancedMouseEvent::Hover { x, y } => {
                    // Update hover state for tooltip support
                    if message_list_area.contains(x, y) {
                        if !self.state.hover_state.hovered {
                            self.state.hover_state.on_mouse_enter(x, y);
                        }
                        self.state.hover_state.update_duration();
                    } else if self.state.hover_state.hovered {
                        self.state.hover_state.on_mouse_exit();
                    }
                }

                EnhancedMouseEvent::RightClick { x, y } => {
                    // Show context menu based on what was clicked
                    if message_list_area.contains(x, y) {
                        // Right-clicked in message/list area
                        let context = MenuContext::Message {
                            is_own: true, // TODO: Determine from actual message
                            can_edit: true,
                        };
                        self.state.context_menu.show_at(x, y, context);
                        self.state
                            .context_menu
                            .adjust_position_for_screen(size.width, size.height);
                    } else if sidebar_area.contains(x, y) {
                        // Right-clicked in sidebar (channel list)
                        let context = MenuContext::Channel {
                            is_admin: false, // TODO: Determine from actual channel
                            is_muted: false,
                        };
                        self.state.context_menu.show_at(x, y, context);
                        self.state
                            .context_menu
                            .adjust_position_for_screen(size.width, size.height);
                    }
                }

                EnhancedMouseEvent::LeftClick { x, y } => {
                    // Handle double-click detection
                    let is_double_click = self.state.double_click_detector.register_click(x, y);

                    if is_double_click {
                        tracing::debug!("Double-click detected at ({}, {})", x, y);
                        // TODO: Handle double-click actions (e.g., open message details)
                    }

                    // Get current view for click handling
                    let current_view = self.state.navigation.current_view();

                    match current_view {
                        View::Auth => {
                            // Auth screen has two options side by side
                            let half_width = size.width / 2;

                            // Login is on left half, Signup is on right half
                            if row >= 3 && row <= size.height - 3 {
                                if col < half_width {
                                    self.state.navigation.selected_index = 0;
                                    handlers::handle_enter(&mut self.state, &mut self.backend)
                                        .await?;
                                } else {
                                    self.state.navigation.selected_index = 1;
                                    handlers::handle_enter(&mut self.state, &mut self.backend)
                                        .await?;
                                }
                            }
                        }
                        View::Dashboard
                        | View::Organizations
                        | View::Projects
                        | View::Groups
                        | View::Contacts => {
                            // For list views, calculate which item was clicked
                            if message_list_area.contains(x, y) {
                                let list_row = (y.saturating_sub(message_list_area.y)) as usize;

                                let max_index = match current_view {
                                    View::Organizations => self
                                        .state
                                        .entities
                                        .channels
                                        .values()
                                        .map(|channels| channels.len())
                                        .sum::<usize>(),
                                    View::Projects => self.state.entities.projects.len(),
                                    View::Groups => self.state.entities.groups.len(),
                                    View::Contacts => self.state.entities.contacts.len(),
                                    _ => 0,
                                };

                                if list_row < max_index {
                                    self.state.navigation.selected_index = list_row;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                EnhancedMouseEvent::DragStart { x, y } => {
                    self.state.drag_state.start_drag(x, y);
                    tracing::debug!("Drag started at ({}, {})", x, y);
                }

                EnhancedMouseEvent::Dragging {
                    x,
                    y,
                    start_x: _,
                    start_y: _,
                } => {
                    self.state.drag_state.update_drag(x, y);
                }

                EnhancedMouseEvent::MouseUp { x, y, button } => {
                    if button == MouseButton::Left && self.state.drag_state.is_dragging() {
                        self.state.drag_state.end_drag();
                        if let Some((dx, dy)) = self.state.drag_state.get_drag_delta() {
                            tracing::debug!("Drag ended with delta: ({}, {})", dx, dy);
                            // TODO: Handle drag-and-drop actions
                        }
                        self.state.drag_state.reset();
                    }
                }

                EnhancedMouseEvent::ScrollUp { x, y } => {
                    if message_list_area.contains(x, y) || input_area.contains(x, y) {
                        // Scroll up in message list
                        self.state.scroll_state.scroll_by(-3);
                        tracing::debug!(
                            "Scrolled up to offset: {}",
                            self.state.scroll_state.scroll_offset
                        );
                    }
                }

                EnhancedMouseEvent::ScrollDown { x, y } => {
                    if message_list_area.contains(x, y) || input_area.contains(x, y) {
                        // Scroll down in message list
                        self.state.scroll_state.scroll_by(3);
                        tracing::debug!(
                            "Scrolled down to offset: {}",
                            self.state.scroll_state.scroll_offset
                        );
                    }
                }

                _ => {
                    // Other enhanced events not yet implemented
                }
            }
        }

        Ok(())
    }

    /// Execute command from command palette
    async fn execute_command(&mut self, cmd_id: &str) -> Result<()> {
        use crate::components::Animation;
        use crate::state::{FocusedPanel, View};
        use std::time::Duration;

        match cmd_id {
            // Navigation Commands
            "nav.orgs" => {
                let old_view = self.state.navigation.current_view().clone();
                self.state.navigation.push_view(View::Organizations);
                if old_view != View::Organizations {
                    use crate::components::{Animation, Axis};
                    self.state.animation_manager.add(
                        "panel_slide",
                        Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(300)),
                    );
                }
                self.state.set_status("Navigated to Organizations");
            }
            "nav.projects" => {
                let old_view = self.state.navigation.current_view().clone();
                self.state.navigation.push_view(View::Projects);
                if old_view != View::Projects {
                    use crate::components::{Animation, Axis};
                    self.state.animation_manager.add(
                        "panel_slide",
                        Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(300)),
                    );
                }
                self.state.set_status("Navigated to Projects");
            }
            "nav.groups" => {
                let old_view = self.state.navigation.current_view().clone();
                self.state.navigation.push_view(View::Groups);
                if old_view != View::Groups {
                    use crate::components::{Animation, Axis};
                    self.state.animation_manager.add(
                        "panel_slide",
                        Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(300)),
                    );
                }
                self.state.set_status("Navigated to Groups");
            }
            "nav.contacts" => {
                let old_view = self.state.navigation.current_view().clone();
                self.state.navigation.push_view(View::Contacts);
                if old_view != View::Contacts {
                    use crate::components::{Animation, Axis};
                    self.state.animation_manager.add(
                        "panel_slide",
                        Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(300)),
                    );
                }
                self.state.set_status("Navigated to Contacts");
            }
            "nav.dashboard" => {
                let old_view = self.state.navigation.current_view().clone();
                self.state.navigation.push_view(View::Dashboard);
                if old_view != View::Dashboard {
                    use crate::components::{Animation, Axis};
                    self.state.animation_manager.add(
                        "panel_slide",
                        Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(300)),
                    );
                }
                self.state.set_status("Navigated to Dashboard");
            }

            // Action Commands (placeholders for now)
            "create.org" => {
                self.state
                    .set_status("Create Organization (not yet implemented)");
            }
            "create.project" => {
                self.state
                    .set_status("Create Project (not yet implemented)");
            }
            "create.group" => {
                self.state.set_status("Create Group (not yet implemented)");
            }
            "create.contact" => {
                self.state.set_status("Add Contact (not yet implemented)");
            }

            // Settings Commands
            "toggle.theme" => {
                self.state.theme_manager.toggle_theme_preview();
                self.state.set_status("Theme preview toggled".to_string());
            }
            "toggle.perf" => {
                let visible = self.state.performance_monitor.is_visible();
                self.state.performance_monitor.set_visible(!visible);
                self.state.set_status(format!(
                    "Performance monitor {}",
                    if self.state.performance_monitor.is_visible() {
                        "shown"
                    } else {
                        "hidden"
                    }
                ));
            }
            "settings.open" => {
                self.state.set_status("Settings (not yet implemented)");
            }

            // Network Commands (placeholders)
            "net.connect" => {
                self.state
                    .set_status("Connect to Network (not yet implemented)");
            }
            "net.disconnect" => {
                self.state
                    .set_status("Disconnect from Network (not yet implemented)");
            }
            "net.status" => {
                let old_view = self.state.navigation.current_view().clone();
                self.state.navigation.push_view(View::NetworkStatus);
                if old_view != View::NetworkStatus {
                    use crate::components::{Animation, Axis};
                    self.state.animation_manager.add(
                        "panel_slide",
                        Animation::slide(0, 100, Axis::Horizontal, Duration::from_millis(300)),
                    );
                }
                self.state.set_status("Showing network status");
            }

            // View Commands
            "view.focus_sidebar" => {
                self.state.navigation.focused_panel = FocusedPanel::Sidebar;
                self.state.set_status("Focused sidebar");
            }
            "view.focus_main" => {
                self.state.navigation.focused_panel = FocusedPanel::Main;
                self.state.set_status("Focused main panel");
            }
            "view.split_reset" => {
                if let View::Dashboard = self.state.navigation.current_view() {
                    self.state.resizable_split.reset_position();
                    self.state.set_status(format!(
                        "Split panel reset to {}%",
                        self.state.resizable_split.position()
                    ));
                }
            }

            _ => {
                self.state
                    .set_status(format!("Unknown command: {}", cmd_id));
            }
        }

        Ok(())
    }

    /// Cleanup terminal on exit
    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
