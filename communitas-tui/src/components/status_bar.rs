//! StatusBar component for displaying application status
//!
//! Following TDD: Tests written first, implementation follows.
//!
//! The StatusBar displays:
//! - Current status message
//! - Network connection status (with indicator)
//! - User identity (if authenticated)
//! - Error messages (in red)

use crate::messages::{Msg, NetworkStatus};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    Component, MockComponent, State,
};

/// StatusBar component properties
pub struct StatusBar {
    /// Component properties
    props: Props,
    /// Current status message
    status_message: String,
    /// Network connection status
    network_status: NetworkStatus,
    /// Current user identity (four-word address)
    identity: Option<String>,
    /// Error message if any
    error_message: Option<String>,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self {
            props: Props::default(),
            status_message: "Ready".to_string(),
            network_status: NetworkStatus::Disconnected,
            identity: None,
            error_message: None,
        }
    }
}

impl StatusBar {
    /// Create a new StatusBar with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the status message
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
        self.error_message = None; // Clear error when setting normal status
    }

    /// Set an error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
    }

    /// Clear the error message
    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    /// Set the network status
    pub fn set_network_status(&mut self, status: NetworkStatus) {
        self.network_status = status;
    }

    /// Set the user identity
    pub fn set_identity(&mut self, identity: Option<String>) {
        self.identity = identity;
    }

    /// Get network status indicator (colored dot + text)
    fn network_indicator(&self) -> (char, Color, &str) {
        match &self.network_status {
            NetworkStatus::Connected => ('●', Color::Green, "Connected"),
            NetworkStatus::Connecting => ('◐', Color::Yellow, "Connecting"),
            NetworkStatus::Disconnected => ('○', Color::Gray, "Offline"),
            NetworkStatus::Error(_) => ('✖', Color::Red, "Error"),
        }
    }
}

impl MockComponent for StatusBar {
    fn view(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        // Build the status bar content
        let mut spans = Vec::new();

        // Left section: Status or Error
        if let Some(ref error) = self.error_message {
            spans.push(Span::styled(
                format!("⚠ {} ", error),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("{} ", self.status_message),
                Style::default().fg(Color::White),
            ));
        }

        // Add separator
        spans.push(Span::raw("│ "));

        // Middle section: Network status
        let (indicator, color, text) = self.network_indicator();
        spans.push(Span::styled(
            format!("{} {} ", indicator, text),
            Style::default().fg(color),
        ));

        // Right section: Identity (if authenticated)
        if let Some(ref identity) = self.identity {
            spans.push(Span::raw("│ "));
            spans.push(Span::styled(
                format!("👤 {}", identity),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::ITALIC),
            ));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );

        frame.render_widget(paragraph, area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        // Return composite state with all important values
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        // StatusBar doesn't respond to commands in this simple version
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for StatusBar {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // StatusBar is display-only, doesn't handle events
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_creation() {
        let status_bar = StatusBar::new();

        assert_eq!(status_bar.status_message, "Ready");
        assert!(status_bar.identity.is_none());
        assert!(status_bar.error_message.is_none());
        assert!(matches!(
            status_bar.network_status,
            NetworkStatus::Disconnected
        ));
    }

    #[test]
    fn test_set_status_message() {
        let mut status_bar = StatusBar::new();

        status_bar.set_status("Processing...");

        assert_eq!(status_bar.status_message, "Processing...");
        assert!(status_bar.error_message.is_none(), "Setting status should clear error");
    }

    #[test]
    fn test_set_error_message() {
        let mut status_bar = StatusBar::new();

        status_bar.set_error("Connection failed");

        assert_eq!(
            status_bar.error_message,
            Some("Connection failed".to_string())
        );
    }

    #[test]
    fn test_clear_error() {
        let mut status_bar = StatusBar::new();
        status_bar.set_error("Test error");

        status_bar.clear_error();

        assert!(status_bar.error_message.is_none());
    }

    #[test]
    fn test_error_clears_when_setting_status() {
        let mut status_bar = StatusBar::new();
        status_bar.set_error("Old error");

        status_bar.set_status("New status");

        assert!(status_bar.error_message.is_none());
        assert_eq!(status_bar.status_message, "New status");
    }

    #[test]
    fn test_set_network_status_connected() {
        let mut status_bar = StatusBar::new();

        status_bar.set_network_status(NetworkStatus::Connected);

        assert!(matches!(
            status_bar.network_status,
            NetworkStatus::Connected
        ));
    }

    #[test]
    fn test_set_network_status_connecting() {
        let mut status_bar = StatusBar::new();

        status_bar.set_network_status(NetworkStatus::Connecting);

        assert!(matches!(
            status_bar.network_status,
            NetworkStatus::Connecting
        ));
    }

    #[test]
    fn test_set_network_status_error() {
        let mut status_bar = StatusBar::new();

        status_bar.set_network_status(NetworkStatus::Error("Timeout".to_string()));

        match &status_bar.network_status {
            NetworkStatus::Error(msg) => assert_eq!(msg, "Timeout"),
            _ => panic!("Expected NetworkStatus::Error"),
        }
    }

    #[test]
    fn test_set_identity() {
        let mut status_bar = StatusBar::new();

        status_bar.set_identity(Some("ocean-forest-moon-star".to_string()));

        assert_eq!(
            status_bar.identity,
            Some("ocean-forest-moon-star".to_string())
        );
    }

    #[test]
    fn test_clear_identity() {
        let mut status_bar = StatusBar::new();
        status_bar.set_identity(Some("test-identity".to_string()));

        status_bar.set_identity(None);

        assert!(status_bar.identity.is_none());
    }

    #[test]
    fn test_network_indicator_connected() {
        let mut status_bar = StatusBar::new();
        status_bar.set_network_status(NetworkStatus::Connected);

        let (indicator, color, text) = status_bar.network_indicator();

        assert_eq!(indicator, '●');
        assert_eq!(color, Color::Green);
        assert_eq!(text, "Connected");
    }

    #[test]
    fn test_network_indicator_connecting() {
        let mut status_bar = StatusBar::new();
        status_bar.set_network_status(NetworkStatus::Connecting);

        let (indicator, color, text) = status_bar.network_indicator();

        assert_eq!(indicator, '◐');
        assert_eq!(color, Color::Yellow);
        assert_eq!(text, "Connecting");
    }

    #[test]
    fn test_network_indicator_disconnected() {
        let mut status_bar = StatusBar::new();
        status_bar.set_network_status(NetworkStatus::Disconnected);

        let (indicator, color, text) = status_bar.network_indicator();

        assert_eq!(indicator, '○');
        assert_eq!(color, Color::Gray);
        assert_eq!(text, "Offline");
    }

    #[test]
    fn test_network_indicator_error() {
        let mut status_bar = StatusBar::new();
        status_bar.set_network_status(NetworkStatus::Error("Test".to_string()));

        let (indicator, color, text) = status_bar.network_indicator();

        assert_eq!(indicator, '✖');
        assert_eq!(color, Color::Red);
        assert_eq!(text, "Error");
    }

    #[test]
    fn test_mock_component_state() {
        let status_bar = StatusBar::new();

        let state = status_bar.state();

        assert_eq!(state, State::None);
    }

    #[test]
    fn test_mock_component_perform() {
        let mut status_bar = StatusBar::new();

        let result = status_bar.perform(Cmd::Submit);

        assert_eq!(result, CmdResult::None);
    }

    #[test]
    fn test_component_on_event() {
        let mut status_bar = StatusBar::new();

        let result = status_bar.on(Event::Keyboard(KeyEvent {
            code: Key::Char('q'),
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(result, None, "StatusBar should not handle events");
    }

    #[test]
    fn test_multiple_status_updates() {
        let mut status_bar = StatusBar::new();

        status_bar.set_status("First");
        assert_eq!(status_bar.status_message, "First");

        status_bar.set_status("Second");
        assert_eq!(status_bar.status_message, "Second");

        status_bar.set_error("Error");
        assert_eq!(status_bar.error_message, Some("Error".to_string()));

        status_bar.set_status("Third");
        assert_eq!(status_bar.status_message, "Third");
        assert!(status_bar.error_message.is_none());
    }
}
