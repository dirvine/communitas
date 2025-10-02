use super::{EntityData, Navigation, NetworkState};

/// Global application state
#[derive(Debug)]
pub struct AppState {
    /// Current navigation state (views, focus)
    pub navigation: Navigation,

    /// Network connection state
    pub network: NetworkState,

    /// Current identity (four-word address)
    pub identity: Option<String>,

    /// Display name for the user
    pub display_name: Option<String>,

    /// Entity data (channels, projects, etc.)
    pub entities: EntityData,

    /// Whether the app should quit
    pub should_quit: bool,

    /// Status message to display
    pub status_message: Option<String>,

    /// Input buffer for text entry
    pub input_buffer: String,

    /// Whether input mode is active
    pub input_active: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            navigation: Navigation::new(),
            network: NetworkState::new(),
            identity: None,
            display_name: None,
            entities: EntityData::new(),
            should_quit: false,
            status_message: None,
            input_buffer: String::new(),
            input_active: false,
        }
    }

    /// Set identity and display name
    pub fn set_identity(&mut self, identity: String, display_name: String) {
        self.identity = Some(identity);
        self.display_name = Some(display_name);
    }

    /// Set status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Activate input mode
    pub fn activate_input(&mut self) {
        self.input_active = true;
        self.input_buffer.clear();
    }

    /// Deactivate input mode
    pub fn deactivate_input(&mut self) {
        self.input_active = false;
        self.input_buffer.clear();
    }

    /// Get input buffer and clear it
    pub fn take_input(&mut self) -> String {
        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        input
    }

    /// Append character to input buffer
    pub fn push_input_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    /// Delete last character from input buffer
    pub fn pop_input_char(&mut self) {
        self.input_buffer.pop();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
