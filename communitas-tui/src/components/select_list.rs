//! SelectList component for selectable item lists
//!
//! Following TDD: Tests written first, implementation follows.
//!
//! The SelectList provides:
//! - Generic list items with id and label
//! - Single and multi-select modes
//! - Keyboard navigation
//! - Visual selection indicators
//! - Empty state handling
//! - Filtering support

use crate::messages::{ComponentId, Msg};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem as RatatuiListItem, ListState},
};
use tuirealm::{
    command::{Cmd, CmdResult, Direction, Position},
    event::{Event, Key, KeyEvent, KeyModifiers, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    Component, MockComponent, State, StateValue,
};

/// A single item in the list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListItem {
    /// Unique identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether item is selected (for multi-select)
    pub selected: bool,
}

impl ListItem {
    /// Create a new list item
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            selected: false,
        }
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark as selected (for multi-select)
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// SelectList component for navigating and selecting items
pub struct SelectList {
    /// Component properties
    props: Props,
    /// Component ID for message generation
    component_id: ComponentId,
    /// List of items
    items: Vec<ListItem>,
    /// List state for navigation
    list_state: ListState,
    /// Currently focused index
    focused: usize,
    /// Multi-select mode enabled
    multi_select: bool,
    /// Component title
    title: String,
    /// Empty state message
    empty_message: String,
}

impl SelectList {
    /// Create a new SelectList with a component ID
    pub fn new(component_id: ComponentId) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            props: Props::default(),
            component_id,
            items: Vec::new(),
            list_state,
            focused: 0,
            multi_select: false,
            title: String::new(),
            empty_message: "No items".to_string(),
        }
    }

    /// Set the title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Enable multi-select mode
    pub fn multi_select(mut self, enabled: bool) -> Self {
        self.multi_select = enabled;
        self
    }

    /// Set empty state message
    pub fn empty_message(mut self, message: impl Into<String>) -> Self {
        self.empty_message = message.into();
        self
    }

    /// Set items
    pub fn set_items(&mut self, items: Vec<ListItem>) {
        self.items = items;
        if !self.items.is_empty() && self.focused >= self.items.len() {
            self.focused = self.items.len() - 1;
            self.list_state.select(Some(self.focused));
        }
    }

    /// Get items
    pub fn items(&self) -> &[ListItem] {
        &self.items
    }

    /// Get focused index
    pub fn focused_index(&self) -> usize {
        self.focused
    }

    /// Get focused item
    pub fn focused_item(&self) -> Option<&ListItem> {
        self.items.get(self.focused)
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get selected items (for multi-select)
    pub fn selected_items(&self) -> Vec<&ListItem> {
        self.items.iter().filter(|item| item.selected).collect()
    }

    /// Toggle selection of current item (multi-select only)
    pub fn toggle_selection(&mut self) {
        if self.multi_select {
            if let Some(item) = self.items.get_mut(self.focused) {
                item.selected = !item.selected;
            }
        }
    }

    /// Select current item (sets selected in multi-select, or just returns in single-select)
    pub fn select_current(&mut self) {
        if self.multi_select {
            if let Some(item) = self.items.get_mut(self.focused) {
                item.selected = true;
            }
        }
    }

    /// Deselect all items
    pub fn clear_selection(&mut self) {
        for item in &mut self.items {
            item.selected = false;
        }
    }

    /// Move focus up
    pub fn move_up(&mut self) {
        if self.focused > 0 {
            self.focused -= 1;
            self.list_state.select(Some(self.focused));
        }
    }

    /// Move focus down
    pub fn move_down(&mut self) {
        if !self.items.is_empty() && self.focused < self.items.len() - 1 {
            self.focused += 1;
            self.list_state.select(Some(self.focused));
        }
    }

    /// Move to top
    pub fn move_to_top(&mut self) {
        if !self.items.is_empty() {
            self.focused = 0;
            self.list_state.select(Some(self.focused));
        }
    }

    /// Move to bottom
    pub fn move_to_bottom(&mut self) {
        if !self.items.is_empty() {
            self.focused = self.items.len() - 1;
            self.list_state.select(Some(self.focused));
        }
    }

    /// Format a list item for display
    fn format_item<'a>(&self, item: &'a ListItem, is_focused: bool) -> RatatuiListItem<'a> {
        let mut spans = Vec::new();

        // Selection indicator (multi-select)
        if self.multi_select {
            let indicator = if item.selected { "[✓] " } else { "[ ] " };
            let style = if item.selected {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(indicator, style));
        }

        // Label
        let label_style = if is_focused {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(&item.label, label_style));

        // Description (if present)
        if let Some(ref desc) = item.description {
            spans.push(Span::styled(
                format!(" - {}", desc),
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ));
        }

        RatatuiListItem::new(Line::from(spans))
    }
}

impl MockComponent for SelectList {
    fn view(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        if self.items.is_empty() {
            // Show empty state
            let empty_text = Span::styled(
                &self.empty_message,
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            );
            let paragraph = ratatui::widgets::Paragraph::new(Line::from(empty_text)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.title.as_str())
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
            frame.render_widget(paragraph, area);
        } else {
            let focused = self.focused;
            let items: Vec<RatatuiListItem> = self
                .items
                .iter()
                .enumerate()
                .map(|(idx, item)| self.format_item(item, idx == focused))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(self.title.as_str())
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, area, &mut self.list_state);
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        if let Some(item) = self.focused_item() {
            State::One(StateValue::String(item.id.clone()))
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
            Cmd::GoTo(Position::Begin) => {
                self.move_to_top();
                CmdResult::Changed(self.state())
            }
            Cmd::GoTo(Position::End) => {
                self.move_to_bottom();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => CmdResult::Submit(self.state()),
            Cmd::Toggle => {
                if self.multi_select {
                    self.toggle_selection();
                    CmdResult::Changed(self.state())
                } else {
                    CmdResult::None
                }
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for SelectList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Up,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_up();
                Some(Msg::SelectionChanged {
                    component: self.component_id.clone(),
                    index: self.focused,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Down,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_down();
                Some(Msg::SelectionChanged {
                    component: self.component_id.clone(),
                    index: self.focused,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Home,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_to_top();
                Some(Msg::SelectionChanged {
                    component: self.component_id.clone(),
                    index: self.focused,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::End,
                modifiers: KeyModifiers::NONE,
            }) => {
                self.move_to_bottom();
                Some(Msg::SelectionChanged {
                    component: self.component_id.clone(),
                    index: self.focused,
                })
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter,
                modifiers: KeyModifiers::NONE,
            }) => {
                if let Some(item) = self.focused_item() {
                    if self.multi_select {
                        // In multi-select, Enter confirms selection
                        Some(Msg::FormSubmitted(self.component_id.clone()))
                    } else {
                        // In single-select, Enter selects the item
                        Some(Msg::ContactSelected(item.id.clone()))
                    }
                } else {
                    None
                }
            }
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                modifiers: KeyModifiers::NONE,
            }) => {
                if self.multi_select {
                    self.toggle_selection();
                    Some(Msg::SelectionChanged {
                        component: self.component_id.clone(),
                        index: self.focused,
                    })
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

    fn create_test_items() -> Vec<ListItem> {
        vec![
            ListItem::new("1", "Item 1"),
            ListItem::new("2", "Item 2").with_description("Description 2"),
            ListItem::new("3", "Item 3"),
        ]
    }

    #[test]
    fn test_list_item_creation() {
        let item = ListItem::new("id1", "Label");

        assert_eq!(item.id, "id1");
        assert_eq!(item.label, "Label");
        assert!(item.description.is_none());
        assert!(!item.selected);
    }

    #[test]
    fn test_list_item_with_description() {
        let item = ListItem::new("id1", "Label").with_description("Desc");

        assert_eq!(item.description, Some("Desc".to_string()));
    }

    #[test]
    fn test_list_item_with_selected() {
        let item = ListItem::new("id1", "Label").with_selected(true);

        assert!(item.selected);
    }

    #[test]
    fn test_select_list_creation() {
        let list = SelectList::new(ComponentId::ContactList);

        assert!(list.is_empty());
        assert_eq!(list.focused_index(), 0);
        assert!(list.focused_item().is_none());
        assert!(!list.multi_select);
    }

    #[test]
    fn test_builder_pattern() {
        let list = SelectList::new(ComponentId::ContactList)
            .title("Contacts")
            .multi_select(true)
            .empty_message("No contacts");

        assert_eq!(list.title, "Contacts");
        assert!(list.multi_select);
        assert_eq!(list.empty_message, "No contacts");
    }

    #[test]
    fn test_set_items() {
        let mut list = SelectList::new(ComponentId::ContactList);
        let items = create_test_items();

        list.set_items(items.clone());

        assert_eq!(list.items().len(), 3);
        assert!(!list.is_empty());
        assert_eq!(list.focused_index(), 0);
        assert_eq!(list.focused_item().unwrap().id, "1");
    }

    #[test]
    fn test_navigation_down() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        assert_eq!(list.focused_index(), 0);

        list.move_down();
        assert_eq!(list.focused_index(), 1);
        assert_eq!(list.focused_item().unwrap().id, "2");

        list.move_down();
        assert_eq!(list.focused_index(), 2);

        // Should not go beyond last item
        list.move_down();
        assert_eq!(list.focused_index(), 2);
    }

    #[test]
    fn test_navigation_up() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        list.move_to_bottom();
        assert_eq!(list.focused_index(), 2);

        list.move_up();
        assert_eq!(list.focused_index(), 1);

        list.move_up();
        assert_eq!(list.focused_index(), 0);

        // Should not go below 0
        list.move_up();
        assert_eq!(list.focused_index(), 0);
    }

    #[test]
    fn test_move_to_top() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        list.move_to_bottom();
        assert_eq!(list.focused_index(), 2);

        list.move_to_top();
        assert_eq!(list.focused_index(), 0);
    }

    #[test]
    fn test_move_to_bottom() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        list.move_to_bottom();
        assert_eq!(list.focused_index(), 2);
        assert_eq!(list.focused_item().unwrap().id, "3");
    }

    #[test]
    fn test_navigation_on_empty_list() {
        let mut list = SelectList::new(ComponentId::ContactList);

        list.move_down();
        assert_eq!(list.focused_index(), 0);

        list.move_up();
        assert_eq!(list.focused_index(), 0);

        list.move_to_top();
        assert_eq!(list.focused_index(), 0);

        list.move_to_bottom();
        assert_eq!(list.focused_index(), 0);
    }

    #[test]
    fn test_multi_select_toggle() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        list.set_items(create_test_items());

        assert!(!list.items()[0].selected);

        list.toggle_selection();
        assert!(list.items()[0].selected);

        list.toggle_selection();
        assert!(!list.items()[0].selected);
    }

    #[test]
    fn test_multi_select_disabled() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        list.toggle_selection();
        assert!(!list.items()[0].selected, "Toggle should not work in single-select mode");
    }

    #[test]
    fn test_select_current() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        list.set_items(create_test_items());

        list.select_current();
        assert!(list.items()[0].selected);
    }

    #[test]
    fn test_clear_selection() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        let mut items = create_test_items();
        items[0].selected = true;
        items[1].selected = true;
        list.set_items(items);

        assert!(list.items()[0].selected);
        assert!(list.items()[1].selected);

        list.clear_selection();

        assert!(!list.items()[0].selected);
        assert!(!list.items()[1].selected);
    }

    #[test]
    fn test_selected_items() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        let mut items = create_test_items();
        items[0].selected = true;
        items[2].selected = true;
        list.set_items(items);

        let selected = list.selected_items();

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, "1");
        assert_eq!(selected[1].id, "3");
    }

    #[test]
    fn test_perform_move_up() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());
        list.move_down();

        let result = list.perform(Cmd::Move(Direction::Up));

        assert_eq!(list.focused_index(), 0);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_move_down() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.perform(Cmd::Move(Direction::Down));

        assert_eq!(list.focused_index(), 1);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_goto_begin() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());
        list.move_to_bottom();

        let result = list.perform(Cmd::GoTo(Position::Begin));

        assert_eq!(list.focused_index(), 0);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_goto_end() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.perform(Cmd::GoTo(Position::End));

        assert_eq!(list.focused_index(), 2);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_submit() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.perform(Cmd::Submit);

        assert!(matches!(result, CmdResult::Submit(_)));
    }

    #[test]
    fn test_perform_toggle_multiselect() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        list.set_items(create_test_items());

        let result = list.perform(Cmd::Toggle);

        assert!(list.items()[0].selected);
        assert!(matches!(result, CmdResult::Changed(_)));
    }

    #[test]
    fn test_perform_toggle_singleselect() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.perform(Cmd::Toggle);

        assert!(!list.items()[0].selected);
        assert!(matches!(result, CmdResult::None));
    }

    #[test]
    fn test_state_returns_focused_id() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let state = list.state();

        if let State::One(StateValue::String(id)) = state {
            assert_eq!(id, "1");
        } else {
            panic!("Expected String state");
        }

        list.move_down();
        let state = list.state();

        if let State::One(StateValue::String(id)) = state {
            assert_eq!(id, "2");
        }
    }

    #[test]
    fn test_state_none_when_empty() {
        let list = SelectList::new(ComponentId::ContactList);

        let state = list.state();

        assert_eq!(state, State::None);
    }

    #[test]
    fn test_keyboard_up_event() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());
        list.move_down();

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Up,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(list.focused_index(), 0);
        assert!(result.is_some());
        if let Some(Msg::SelectionChanged { component, index }) = result {
            assert_eq!(component, ComponentId::ContactList);
            assert_eq!(index, 0);
        }
    }

    #[test]
    fn test_keyboard_down_event() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Down,
            modifiers: KeyModifiers::NONE,
        }));

        assert_eq!(list.focused_index(), 1);
        assert!(result.is_some());
    }

    #[test]
    fn test_keyboard_enter_singleselect() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }));

        assert!(result.is_some());
        if let Some(Msg::ContactSelected(id)) = result {
            assert_eq!(id, "1");
        } else {
            panic!("Expected ContactSelected");
        }
    }

    #[test]
    fn test_keyboard_enter_multiselect() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        list.set_items(create_test_items());

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }));

        assert!(result.is_some());
        if let Some(Msg::FormSubmitted(_)) = result {
            // Correct
        } else {
            panic!("Expected FormSubmitted for multi-select");
        }
    }

    #[test]
    fn test_keyboard_space_toggles_multiselect() {
        let mut list = SelectList::new(ComponentId::ContactList).multi_select(true);
        list.set_items(create_test_items());

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Char(' '),
            modifiers: KeyModifiers::NONE,
        }));

        assert!(list.items()[0].selected);
        assert!(result.is_some());
    }

    #[test]
    fn test_keyboard_space_ignored_in_singleselect() {
        let mut list = SelectList::new(ComponentId::ContactList);
        list.set_items(create_test_items());

        let result = list.on(Event::Keyboard(KeyEvent {
            code: Key::Char(' '),
            modifiers: KeyModifiers::NONE,
        }));

        assert!(!list.items()[0].selected);
        assert!(result.is_none());
    }
}
