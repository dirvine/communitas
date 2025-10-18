//! FormInput component for text input
//!
//! Following TDD: Tests written first, implementation follows.
//!
//! The FormInput provides:
//! - Single-line and multi-line text input
//! - Cursor movement and editing
//! - Placeholder text
//! - Input validation
//! - Password masking
//! - Max length constraints
//! - Focus states

use crate::messages::{ComponentId, Msg};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    Component, MockComponent, State, StateValue,
};

/// Input mode for FormInput
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Single-line input
    SingleLine,
    /// Multi-line input (textarea)
    MultiLine,
}

/// FormInput component for text input
pub struct FormInput {
    /// Component properties
    props: Props,
    /// Component ID for message generation
    component_id: ComponentId,
    /// Current input value
    value: String,
    /// Cursor position (character index)
    cursor: usize,
    /// Placeholder text shown when empty
    placeholder: Option<String>,
    /// Maximum length (None = unlimited)
    max_length: Option<usize>,
    /// Input mode (single/multi-line)
    mode: InputMode,
    /// Password masking enabled
    password: bool,
    /// Validation error message
    error: Option<String>,
    /// Component title/label
    title: String,
}

impl FormInput {
    /// Create a new FormInput with a component ID
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            props: Props::default(),
            component_id,
            value: String::new(),
            cursor: 0,
            placeholder: None,
            max_length: None,
            mode: InputMode::SingleLine,
            password: false,
            error: None,
            title: String::new(),
        }
    }

    /// Set the title/label
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Enable multi-line mode
    pub fn multiline(mut self) -> Self {
        self.mode = InputMode::MultiLine;
        self
    }

    /// Enable password masking
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Get current value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set value programmatically
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        // Clamp cursor to valid position
        if self.cursor > self.value.len() {
            self.cursor = self.value.len();
        }
    }

    /// Get cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Set error message
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    /// Clear error message
    pub fn clear_error(&mut self) {
        self.error = None;
    }

    /// Check if input is empty
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Insert character at cursor
    pub fn insert_char(&mut self, c: char) {
        // Check max length
        if let Some(max) = self.max_length {
            if self.value.len() >= max {
                return;
            }
        }

        // Don't insert newlines in single-line mode
        if self.mode == InputMode::SingleLine && c == '\n' {
            return;
        }

        self.value.insert(self.cursor, c);
        self.cursor += 1;
        self.clear_error();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.value.remove(self.cursor - 1);
            self.cursor -= 1;
            self.clear_error();
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_forward(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
            self.clear_error();
        }
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        if self.cursor < self.value.len() {
            self.cursor += 1;
        }
    }

    /// Move cursor to start
    pub fn move_cursor_start(&mut self) {
        self.cursor = 0;
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) {
        self.cursor = self.value.len();
    }

    /// Clear all input
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
        self.clear_error();
    }

    /// Get display value (masked if password)
    fn display_value(&self) -> String {
        if self.password && !self.value.is_empty() {
            "•".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }
}

impl MockComponent for FormInput {
    fn view(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        // Store display value to extend its lifetime
        let display_value = self.display_value();

        let display_text = if self.value.is_empty() {
            // Show placeholder if empty
            if let Some(ref placeholder) = self.placeholder {
                Span::styled(
                    placeholder,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                )
            } else {
                Span::raw("")
            }
        } else {
            // Show value (masked if password)
            Span::styled(&display_value, Style::default().fg(Color::White))
        };

        let line = Line::from(display_text);

        // Determine border color based on error state
        let border_style = if self.error.is_some() {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let title = if let Some(ref error) = self.error {
            format!("{} - Error: {}", self.title, error)
        } else if self.title.is_empty() {
            "Input".to_string()
        } else {
            self.title.clone()
        };

        let paragraph = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
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
        State::One(StateValue::String(self.value.clone()))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Type(c) => {
                self.insert_char(c);
                CmdResult::Changed(self.state())
            }
            Cmd::Delete => {
                self.delete_char();
                CmdResult::Changed(self.state())
            }
            Cmd::Cancel => {
                self.delete_char_forward();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Left) => {
                self.move_cursor_left();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Right) => {
                self.move_cursor_right();
                CmdResult::Changed(self.state())
            }
            Cmd::GoTo(Position::Begin) => {
                self.move_cursor_start();
                CmdResult::Changed(self.state())
            }
            Cmd::GoTo(Position::End) => {
                self.move_cursor_end();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => {
                CmdResult::Submit(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for FormInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Char(c),
                modifiers: KeyModifiers::NONE,
            }) => {
                self.insert_char(c);
                Some(Msg::InputChanged {
                    component: self.component_id.clone(),
                    value: self.value.clone(),
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.delete_char();
                Some(Msg::InputChanged {
                    component: self.component_id.clone(),
                    value: self.value.clone(),
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.delete_char_forward();
                Some(Msg::InputChanged {
                    component: self.component_id.clone(),
                    value: self.value.clone(),
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Left,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_cursor_left();
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Right,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_cursor_right();
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_cursor_start();
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_cursor_end();
                None
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.mode == InputMode::MultiLine {
                    // Insert newline in multiline mode
                    self.insert_char('\n');
                    Some(Msg::InputChanged {
                        component: self.component_id.clone(),
                        value: self.value.clone(),
                    })
                } else {
                    // Submit in single-line mode
                    Some(Msg::FormSubmitted(self.component_id.clone()))
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_input_creation() {
        let input = FormInput::new(ComponentId::MessageComposer);

        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
        assert!(input.is_empty());
        assert!(input.placeholder.is_none());
        assert!(input.max_length.is_none());
        assert_eq!(input.mode, InputMode::SingleLine);
        assert!(!input.password);
        assert!(input.error.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let input = FormInput::new(ComponentId::MessageComposer)
            .title("Message")
            .placeholder("Type a message...")
            .max_length(200);

        assert_eq!(input.title, "Message");
        assert_eq!(input.placeholder.as_deref(), Some("Type a message..."));
        assert_eq!(input.max_length, Some(200));
    }

    #[test]
    fn test_multiline_builder() {
        let input = FormInput::new(ComponentId::MessageComposer).multiline();

        assert_eq!(input.mode, InputMode::MultiLine);
    }

    #[test]
    fn test_password_builder() {
        let input = FormInput::new(ComponentId::MessageComposer).password();

        assert!(input.password);
    }

    #[test]
    fn test_insert_char() {
        let mut input = FormInput::new(ComponentId::MessageComposer);

        input.insert_char('H');
        input.insert_char('e');
        input.insert_char('l');
        input.insert_char('l');
        input.insert_char('o');

        assert_eq!(input.value(), "Hello");
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_insert_char_respects_max_length() {
        let mut input = FormInput::new(ComponentId::MessageComposer).max_length(3);

        input.insert_char('a');
        input.insert_char('b');
        input.insert_char('c');
        input.insert_char('d'); // Should be ignored

        assert_eq!(input.value(), "abc");
        assert_eq!(input.cursor(), 3);
    }

    #[test]
    fn test_insert_newline_blocked_in_singleline() {
        let mut input = FormInput::new(ComponentId::MessageComposer);

        input.insert_char('H');
        input.insert_char('\n'); // Should be ignored
        input.insert_char('i');

        assert_eq!(input.value(), "Hi");
    }

    #[test]
    fn test_insert_newline_allowed_in_multiline() {
        let mut input = FormInput::new(ComponentId::MessageComposer).multiline();

        input.insert_char('H');
        input.insert_char('\n');
        input.insert_char('i');

        assert_eq!(input.value(), "H\ni");
    }

    #[test]
    fn test_delete_char() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hello");
        input.cursor = 5;

        input.delete_char();

        assert_eq!(input.value(), "Hell");
        assert_eq!(input.cursor(), 4);
    }

    #[test]
    fn test_delete_char_at_start() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hello");
        input.cursor = 0;

        input.delete_char();

        assert_eq!(input.value(), "Hello"); // No change
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_delete_char_forward() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hello");
        input.cursor = 0;

        input.delete_char_forward();

        assert_eq!(input.value(), "ello");
        assert_eq!(input.cursor(), 0);
    }

    #[test]
    fn test_delete_char_forward_at_end() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hello");
        input.cursor = 5;

        input.delete_char_forward();

        assert_eq!(input.value(), "Hello"); // No change
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hello");
        input.cursor = 2;

        input.move_cursor_left();
        assert_eq!(input.cursor(), 1);

        input.move_cursor_right();
        assert_eq!(input.cursor(), 2);

        input.move_cursor_start();
        assert_eq!(input.cursor(), 0);

        input.move_cursor_end();
        assert_eq!(input.cursor(), 5);
    }

    #[test]
    fn test_cursor_boundaries() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hi");
        input.cursor = 0;

        input.move_cursor_left(); // Should not go below 0
        assert_eq!(input.cursor(), 0);

        input.cursor = 2;
        input.move_cursor_right(); // Should not exceed length
        assert_eq!(input.cursor(), 2);
    }

    #[test]
    fn test_set_value() {
        let mut input = FormInput::new(ComponentId::MessageComposer);

        input.set_value("Test");

        assert_eq!(input.value(), "Test");
        assert_eq!(input.cursor(), 0); // Cursor resets to valid position
    }

    #[test]
    fn test_set_value_clamps_cursor() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.cursor = 10;

        input.set_value("Hi");

        assert_eq!(input.cursor(), 2); // Clamped to length
    }

    #[test]
    fn test_clear() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("Hello");
        input.cursor = 3;

        input.clear();

        assert_eq!(input.value(), "");
        assert_eq!(input.cursor(), 0);
        assert!(input.is_empty());
    }

    #[test]
    fn test_error_management() {
        let mut input = FormInput::new(ComponentId::MessageComposer);

        input.set_error("Invalid input");
        assert_eq!(input.error, Some("Invalid input".to_string()));

        input.clear_error();
        assert!(input.error.is_none());
    }

    #[test]
    fn test_error_cleared_on_input() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_error("Error");

        input.insert_char('a');

        assert!(input.error.is_none());
    }

    #[test]
    fn test_password_display() {
        let mut input = FormInput::new(ComponentId::MessageComposer).password();
        input.set_value("secret123");

        let display = input.display_value();

        assert_eq!(display, "•••••••••");
        assert_eq!(input.value(), "secret123"); // Actual value unchanged
    }

    #[test]
    fn test_state_returns_value() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("test");

        let state = input.state();

        if let State::One(StateValue::String(val)) = state {
            assert_eq!(val, "test");
        } else {
            panic!("Expected String state");
        }
    }

    #[test]
    fn test_perform_type() {
        let mut input = FormInput::new(ComponentId::MessageComposer);

        let result = input.perform(Cmd::Type('a'));

        assert_eq!(input.value(), "a");
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_delete() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("ab");
        input.cursor = 2;

        let result = input.perform(Cmd::Delete);

        assert_eq!(input.value(), "a");
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_movements() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("test");
        input.cursor = 2;

        input.perform(Cmd::Move(Direction::Left));
        assert_eq!(input.cursor(), 1);

        input.perform(Cmd::Move(Direction::Right));
        assert_eq!(input.cursor(), 2);

        input.perform(Cmd::GoTo(Position::Begin));
        assert_eq!(input.cursor(), 0);

        input.perform(Cmd::GoTo(Position::End));
        assert_eq!(input.cursor(), 4);
    }

    #[test]
    fn test_perform_submit() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("test");

        let result = input.perform(Cmd::Submit);

        assert!(matches!(result, CmdResult::Submit(_)));
    }

    #[test]
    fn test_keyboard_char_event() {
        let mut input = FormInput::new(ComponentId::MessageComposer);

        let result = input.on(Event::Keyboard(KeyEvent {
            code: Key::Char('a'),
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(input.value(), "a");
        assert!(result.is_some());
        if let Some(Msg::InputChanged { component, value }) = result {
            assert_eq!(component, ComponentId::MessageComposer);
            assert_eq!(value, "a");
        }
    }

    #[test]
    fn test_keyboard_backspace_event() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("ab");
        input.cursor = 2;

        let result = input.on(Event::Keyboard(KeyEvent {
            code: Key::Backspace,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(input.value(), "a");
        assert!(result.is_some());
    }

    #[test]
    fn test_keyboard_delete_event() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("ab");
        input.cursor = 0;

        let result = input.on(Event::Keyboard(KeyEvent {
            code: Key::Delete,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(input.value(), "b");
        assert!(result.is_some());
    }

    #[test]
    fn test_keyboard_enter_singleline_submits() {
        let input = FormInput::new(ComponentId::MessageComposer);
        let mut input_single = input;

        let result = input_single.on(Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }));

        assert!(result.is_some());
        if let Some(Msg::FormSubmitted(component)) = result {
            assert_eq!(component, ComponentId::MessageComposer);
        } else {
            panic!("Expected FormSubmitted");
        }
    }

    #[test]
    fn test_keyboard_enter_multiline_inserts_newline() {
        let mut input = FormInput::new(ComponentId::MessageComposer).multiline();
        input.set_value("line1");
        input.cursor = 5;

        let result = input.on(Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(input.value(), "line1\n");
        assert!(result.is_some());
        if let Some(Msg::InputChanged { .. }) = result {
            // Correct
        } else {
            panic!("Expected InputChanged");
        }
    }

    #[test]
    fn test_keyboard_navigation() {
        let mut input = FormInput::new(ComponentId::MessageComposer);
        input.set_value("test");
        input.cursor = 2;

        input.on(Event::Keyboard(KeyEvent {
            code: Key::Left,
            modifiers: KeyModifiers::NONE,
        }));
        assert_eq!(input.cursor(), 1);

        input.on(Event::Keyboard(KeyEvent {
            code: Key::Right,
            modifiers: KeyModifiers::NONE,
        }));
        assert_eq!(input.cursor(), 2);

        input.on(Event::Keyboard(KeyEvent {
            code: Key::Home,
            modifiers: KeyModifiers::NONE,
        }));
        assert_eq!(input.cursor(), 0);

        input.on(Event::Keyboard(KeyEvent {
            code: Key::End,
            modifiers: KeyModifiers::NONE,
        }));
        assert_eq!(input.cursor(), 4);
    }
}
