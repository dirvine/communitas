//! MessageList component for displaying chat messages
//!
//! Following TDD: Tests written first, implementation follows.
//!
//! The MessageList displays:
//! - Scrollable list of messages
//! - Message sender and content
//! - Timestamps
//! - Selection/focus state
//! - Unread indicators
//! - Thread indicators

use crate::messages::{ComponentId, Msg};
use tuirealm::{
    Component, Frame, MockComponent, State, StateValue,
    command::{Cmd, CmdResult, Direction},
    event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    ratatui::{
        layout::Rect,
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, ListState},
    },
};

/// Represents a single message in the list
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Unique message ID
    pub id: String,
    /// Sender's four-word identity
    pub sender: String,
    /// Message content
    pub content: String,
    /// Unix timestamp
    pub timestamp: i64,
    /// Whether message is unread
    pub unread: bool,
    /// Whether message has a thread
    pub has_thread: bool,
    /// Thread reply count
    pub thread_count: usize,
}

impl Message {
    /// Create a new message
    pub fn new(
        id: impl Into<String>,
        sender: impl Into<String>,
        content: impl Into<String>,
        timestamp: i64,
    ) -> Self {
        Self {
            id: id.into(),
            sender: sender.into(),
            content: content.into(),
            timestamp,
            unread: false,
            has_thread: false,
            thread_count: 0,
        }
    }

    /// Mark message as unread
    pub fn mark_unread(mut self) -> Self {
        self.unread = true;
        self
    }

    /// Add thread information
    pub fn with_thread(mut self, count: usize) -> Self {
        self.has_thread = count > 0;
        self.thread_count = count;
        self
    }
}

/// MessageList component for displaying and navigating messages
pub struct MessageList {
    /// Component properties
    props: Props,
    /// List of messages
    messages: Vec<Message>,
    /// List state for navigation
    list_state: ListState,
    /// Currently selected index
    selected: usize,
}

impl Default for MessageList {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            props: Props::default(),
            messages: Vec::new(),
            list_state,
            selected: 0,
        }
    }
}

impl MessageList {
    /// Create a new empty MessageList
    pub fn new() -> Self {
        Self::default()
    }

    /// Set messages
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
        if !self.messages.is_empty() && self.selected >= self.messages.len() {
            self.selected = self.messages.len() - 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Get currently selected message
    pub fn selected_message(&self) -> Option<&Message> {
        self.messages.get(self.selected)
    }

    /// Get selected index
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if !self.messages.is_empty() && self.selected < self.messages.len() - 1 {
            self.selected += 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Move to top
    pub fn move_to_top(&mut self) {
        if !self.messages.is_empty() {
            self.selected = 0;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Move to bottom
    pub fn move_to_bottom(&mut self) {
        if !self.messages.is_empty() {
            self.selected = self.messages.len() - 1;
            self.list_state.select(Some(self.selected));
        }
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Format a message as a ListItem
    fn format_message<'a>(&self, message: &'a Message, is_selected: bool) -> ListItem<'a> {
        let mut spans = Vec::new();

        // Unread indicator
        if message.unread {
            spans.push(Span::styled(
                "● ",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::raw("  "));
        }

        // Sender
        let sender_style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        spans.push(Span::styled(format!("{}: ", message.sender), sender_style));

        // Content
        let content_style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(&message.content, content_style));

        // Thread indicator
        if message.has_thread {
            spans.push(Span::styled(
                format!(" 💬 {}", message.thread_count),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::DIM),
            ));
        }

        ListItem::new(Line::from(spans))
    }
}

impl MockComponent for MessageList {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let selected = self.selected;
        let items: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(idx, msg)| self.format_message(msg, idx == selected))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Messages")
                    .border_style(Style::default().fg(Color::DarkGray)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        if let Some(msg) = self.selected_message() {
            State::One(StateValue::String(msg.id.clone()))
        } else {
            State::None
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Up) => {
                self.move_up();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Down) => {
                self.move_down();
                CmdResult::Changed(self.state())
            }
            Cmd::GoTo(tuirealm::command::Position::Begin) => {
                self.move_to_top();
                CmdResult::Changed(self.state())
            }
            Cmd::GoTo(tuirealm::command::Position::End) => {
                self.move_to_bottom();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => {
                // Return the selected message ID
                CmdResult::Submit(self.state())
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for MessageList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_up();
                Some(Msg::SelectionChanged {
                    component: ComponentId::MessageList,
                    index: self.selected,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_down();
                Some(Msg::SelectionChanged {
                    component: ComponentId::MessageList,
                    index: self.selected,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_to_top();
                Some(Msg::SelectionChanged {
                    component: ComponentId::MessageList,
                    index: self.selected,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_to_bottom();
                Some(Msg::SelectionChanged {
                    component: ComponentId::MessageList,
                    index: self.selected,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => self.selected_message().map(|msg| Msg::MessageSelected {
                message_id: msg.id.clone(),
            }),
            Event::Keyboard(KeyEvent {
                code: Key::Char('t'),
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(msg) = self.selected_message() {
                    if msg.has_thread {
                        Some(Msg::ThreadOpened {
                            message_id: msg.id.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
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
    fn test_message_creation() {
        let msg = Message::new("msg1", "alice", "Hello", 1234567890);

        assert_eq!(msg.id, "msg1");
        assert_eq!(msg.sender, "alice");
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.timestamp, 1234567890);
        assert!(!msg.unread, "Message should be read by default");
        assert!(!msg.has_thread, "Message should have no thread by default");
        assert_eq!(msg.thread_count, 0);
    }

    #[test]
    fn test_message_mark_unread() {
        let msg = Message::new("msg1", "alice", "Hello", 1234567890).mark_unread();

        assert!(msg.unread, "Message should be marked as unread");
    }

    #[test]
    fn test_message_with_thread() {
        let msg = Message::new("msg1", "alice", "Hello", 1234567890).with_thread(5);

        assert!(msg.has_thread, "Message should have thread");
        assert_eq!(msg.thread_count, 5);
    }

    #[test]
    fn test_message_list_creation() {
        let list = MessageList::new();

        assert!(list.is_empty(), "List should be empty on creation");
        assert_eq!(list.message_count(), 0);
        assert_eq!(list.selected_index(), 0);
        assert!(list.selected_message().is_none());
    }

    #[test]
    fn test_set_messages() {
        let mut list = MessageList::new();
        let messages = vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi there", 2000),
            Message::new("msg3", "charlie", "Hey", 3000),
        ];

        list.set_messages(messages.clone());

        assert_eq!(list.message_count(), 3);
        assert!(!list.is_empty());
        assert_eq!(list.selected_index(), 0);
        assert_eq!(list.selected_message().unwrap().id, "msg1");
    }

    #[test]
    fn test_navigation_down() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
            Message::new("msg3", "charlie", "Hey", 3000),
        ]);

        assert_eq!(list.selected_index(), 0);

        list.move_down();
        assert_eq!(list.selected_index(), 1);
        assert_eq!(list.selected_message().unwrap().id, "msg2");

        list.move_down();
        assert_eq!(list.selected_index(), 2);
        assert_eq!(list.selected_message().unwrap().id, "msg3");

        // Should not go beyond last item
        list.move_down();
        assert_eq!(list.selected_index(), 2);
    }

    #[test]
    fn test_navigation_up() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
            Message::new("msg3", "charlie", "Hey", 3000),
        ]);

        // Move to bottom first
        list.move_to_bottom();
        assert_eq!(list.selected_index(), 2);

        list.move_up();
        assert_eq!(list.selected_index(), 1);
        assert_eq!(list.selected_message().unwrap().id, "msg2");

        list.move_up();
        assert_eq!(list.selected_index(), 0);
        assert_eq!(list.selected_message().unwrap().id, "msg1");

        // Should not go below 0
        list.move_up();
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_move_to_top() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
            Message::new("msg3", "charlie", "Hey", 3000),
        ]);

        list.move_to_bottom();
        assert_eq!(list.selected_index(), 2);

        list.move_to_top();
        assert_eq!(list.selected_index(), 0);
        assert_eq!(list.selected_message().unwrap().id, "msg1");
    }

    #[test]
    fn test_move_to_bottom() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
            Message::new("msg3", "charlie", "Hey", 3000),
        ]);

        assert_eq!(list.selected_index(), 0);

        list.move_to_bottom();
        assert_eq!(list.selected_index(), 2);
        assert_eq!(list.selected_message().unwrap().id, "msg3");
    }

    #[test]
    fn test_navigation_on_empty_list() {
        let mut list = MessageList::new();

        list.move_down();
        assert_eq!(list.selected_index(), 0);

        list.move_up();
        assert_eq!(list.selected_index(), 0);

        list.move_to_top();
        assert_eq!(list.selected_index(), 0);

        list.move_to_bottom();
        assert_eq!(list.selected_index(), 0);
    }

    #[test]
    fn test_perform_move_up() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        list.move_down(); // Move to index 1
        let result = list.perform(Cmd::Move(Direction::Up));

        assert_eq!(list.selected_index(), 0);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_move_down() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let result = list.perform(Cmd::Move(Direction::Down));

        assert_eq!(list.selected_index(), 1);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_goto_begin() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        list.move_to_bottom();
        let result = list.perform(Cmd::GoTo(tuirealm::command::Position::Begin));

        assert_eq!(list.selected_index(), 0);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_goto_end() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let result = list.perform(Cmd::GoTo(tuirealm::command::Position::End));

        assert_eq!(list.selected_index(), 1);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_submit() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let result = list.perform(Cmd::Submit);

        assert!(matches!(result, CmdResult::Submit(_)));
        if let CmdResult::Submit(State::One(StateValue::String(id))) = result {
            assert_eq!(id, "msg1");
        } else {
            panic!("Expected Submit with message ID");
        }
    }

    #[test]
    fn test_state_returns_selected_message_id() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let state = list.state();
        assert!(matches!(state, State::One(StateValue::String(_))));

        if let State::One(StateValue::String(id)) = state {
            assert_eq!(id, "msg1");
        }

        list.move_down();
        let state = list.state();
        if let State::One(StateValue::String(id)) = state {
            assert_eq!(id, "msg2");
        }
    }

    #[test]
    fn test_state_none_when_empty() {
        let list = MessageList::new();
        let state = list.state();
        assert_eq!(state, State::None);
    }

    #[test]
    fn test_keyboard_up_event() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        list.move_down(); // Start at index 1
        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Up,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(list.selected_index(), 0);
        assert!(result.is_some());
        if let Some(Msg::SelectionChanged { component, index }) = result {
            assert_eq!(component, ComponentId::MessageList);
            assert_eq!(index, 0);
        } else {
            panic!("Expected SelectionChanged message");
        }
    }

    #[test]
    fn test_keyboard_down_event() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Down,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(list.selected_index(), 1);
        assert!(result.is_some());
    }

    #[test]
    fn test_keyboard_enter_event() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }));

        assert!(result.is_some());
        if let Some(Msg::MessageSelected { message_id }) = result {
            assert_eq!(message_id, "msg1");
        } else {
            panic!("Expected MessageSelected message");
        }
    }

    #[test]
    fn test_keyboard_thread_event() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000).with_thread(3),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Char('t'),
            modifiers: KeyModifiers::NONE,
        }));

        assert!(result.is_some());
        if let Some(Msg::ThreadOpened { message_id }) = result {
            assert_eq!(message_id, "msg1");
        } else {
            panic!("Expected ThreadOpened message");
        }
    }

    #[test]
    fn test_keyboard_thread_event_no_thread() {
        let mut list = MessageList::new();
        list.set_messages(vec![Message::new("msg1", "alice", "Hello", 1000)]);

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Char('t'),
            modifiers: KeyModifiers::NONE,
        }));

        assert!(
            result.is_none(),
            "Should not open thread if message has no thread"
        );
    }

    #[test]
    fn test_unread_messages() {
        let mut list = MessageList::new();
        list.set_messages(vec![
            Message::new("msg1", "alice", "Hello", 1000).mark_unread(),
            Message::new("msg2", "bob", "Hi", 2000),
        ]);

        let msg = list.selected_message().unwrap();
        assert!(msg.unread, "First message should be unread");

        list.move_down();
        let msg = list.selected_message().unwrap();
        assert!(!msg.unread, "Second message should be read");
    }
}
