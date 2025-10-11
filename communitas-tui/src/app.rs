use crate::backend::Backend;
use crate::handlers;
use crate::state::{AppState, ConnectionStatus};
use crate::ui;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
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
        loop {
            // Draw UI
            self.terminal.draw(|f| ui::render(f, &self.state))?;

            // Handle events
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key).await? {
                        break; // Should quit
                    }
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
                handlers::handle_check_network(&mut self.state, &mut self.backend).await?;
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
