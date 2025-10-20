//! Modern Tabs component using ratatui 0.30+ native Tabs widget
//!
//! Provides tabbed navigation with enhanced styling and accessibility.

use crate::messages::{Msg, UserEvent};
use tuirealm::ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Tabs},
};
use tuirealm::{
    Component, MockComponent, State,
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
};

/// Tab configuration
#[derive(Debug, Clone)]
pub struct TabConfig {
    pub id: String,
    pub title: String,
    pub badge: Option<String>,
}

impl TabConfig {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            badge: None,
        }
    }

    pub fn with_badge(mut self, badge: &str) -> Self {
        self.badge = Some(badge.to_string());
        self
    }
}

/// Modern Tabs component with accessibility support
pub struct ModernTabs {
    props: Props,
    tabs: Vec<TabConfig>,
    selected_index: usize,
    height: u16,
    show_badges: bool,
}

impl Default for ModernTabs {
    fn default() -> Self {
        Self::new()
    }
}

impl ModernTabs {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            tabs: Vec::new(),
            selected_index: 0,
            height: 3,
            show_badges: true,
        }
    }

    pub fn with_tabs(mut self, tabs: Vec<TabConfig>) -> Self {
        self.tabs = tabs;
        self
    }

    pub fn with_selected(mut self, index: usize) -> Self {
        self.selected_index = index.min(self.tabs.len().saturating_sub(1));
        self
    }

    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn show_badges(mut self, show: bool) -> Self {
        self.show_badges = show;
        self
    }

    pub fn add_tab(&mut self, tab: TabConfig) {
        self.tabs.push(tab);
    }

    pub fn select_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.selected_index = index;
            true
        } else {
            false
        }
    }

    pub fn select_next(&mut self) -> bool {
        if !self.tabs.is_empty() && self.selected_index < self.tabs.len() - 1 {
            self.selected_index += 1;
            true
        } else {
            false
        }
    }

    pub fn select_previous(&mut self) -> bool {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn get_selected_id(&self) -> Option<&str> {
        self.tabs
            .get(self.selected_index)
            .map(|tab| tab.id.as_str())
    }

    pub fn get_selected_title(&self) -> Option<&str> {
        self.tabs
            .get(self.selected_index)
            .map(|tab| tab.title.as_str())
    }

    fn render_tabs(&self) -> Vec<Span<'_>> {
        self.tabs
            .iter()
            .enumerate()
            .flat_map(|(i, tab)| {
                let is_selected = i == self.selected_index;

                // Base style for tabs
                let base_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                } else {
                    Style::default().fg(Color::Gray)
                };

                let mut spans = vec![Span::styled(tab.title.clone(), base_style)];

                // Add badge if enabled and present
                if self.show_badges
                    && !is_selected
                    && let Some(ref badge) = tab.badge
                {
                    spans.push(Span::styled(
                        format!("({})", badge),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::DIM),
                    ));
                }

                // Add separator
                if i < self.tabs.len() - 1 {
                    spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
                }

                spans
            })
            .collect()
    }
}

impl MockComponent for ModernTabs {
    fn view(&mut self, frame: &mut tuirealm::Frame<'_>, area: tuirealm::ratatui::layout::Rect) {
        let block = Block::default()
            .borders(Borders::ALL | Borders::BOTTOM)
            .border_style(Style::default().fg(Color::Blue));

        let tabs_area = block.inner(area);
        frame.render_widget(block, area);

        let spans = self.render_tabs();

        let tabs = Tabs::new(spans)
            .block(Block::default())
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                    .bg(Color::DarkGray),
            );

        frame.render_widget(tabs, tabs_area);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(tuirealm::StateValue::Usize(self.selected_index))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Custom("select_tab") => {
                if let Some(index) = self.query(Attribute::Value)
                    && let AttrValue::Size(index) = index
                {
                    let index_usize: usize = index.into();
                    if self.select_tab(index_usize) {
                        return CmdResult::Submit(State::One(tuirealm::StateValue::Usize(
                            index_usize,
                        )));
                    }
                }
            }
            Cmd::Move(dir) => match dir {
                tuirealm::command::Direction::Left => {
                    if self.select_previous() {
                        return CmdResult::Submit(State::One(tuirealm::StateValue::Usize(
                            self.selected_index,
                        )));
                    }
                }
                tuirealm::command::Direction::Right => {
                    if self.select_next() {
                        return CmdResult::Submit(State::One(tuirealm::StateValue::Usize(
                            self.selected_index,
                        )));
                    }
                }
                _ => {}
            },
            _ => {}
        }
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for ModernTabs {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if let Event::Keyboard(event) = ev {
            use tuirealm::event::{Key, KeyModifiers};

            match event.code {
                Key::Left | Key::Char('h') => {
                    if self.select_previous() {
                        return Some(Msg::User(UserEvent::TabChanged(self.selected_index)));
                    }
                }
                Key::Right | Key::Char('l') => {
                    if self.select_next() {
                        return Some(Msg::User(UserEvent::TabChanged(self.selected_index)));
                    }
                }
                Key::Tab => {
                    if event.modifiers.contains(KeyModifiers::SHIFT) {
                        if self.select_previous() {
                            return Some(Msg::User(UserEvent::TabChanged(self.selected_index)));
                        }
                    } else if self.select_next() {
                        return Some(Msg::User(UserEvent::TabChanged(self.selected_index)));
                    }
                }
                Key::Enter => {
                    if let Some(id) = self.get_selected_id() {
                        return Some(Msg::User(UserEvent::TabActivated(id.to_string())));
                    }
                }
                _ => {}
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::StateValue;

    #[test]
    fn test_tab_config_creation() {
        let tab = TabConfig::new("test", "Test Tab");
        assert_eq!(tab.id, "test");
        assert_eq!(tab.title, "Test Tab");
        assert!(tab.badge.is_none());
    }

    #[test]
    fn test_tab_config_with_badge() {
        let tab = TabConfig::new("test", "Test Tab").with_badge("5");
        assert_eq!(tab.badge, Some("5".to_string()));
    }

    #[test]
    fn test_modern_tabs_creation() {
        let tabs = ModernTabs::new();
        assert!(tabs.tabs.is_empty());
        assert_eq!(tabs.selected_index, 0);
        assert_eq!(tabs.height, 3);
        assert!(tabs.show_badges);
    }

    #[test]
    fn test_add_tab() {
        let mut tabs = ModernTabs::new();
        tabs.add_tab(TabConfig::new("tab1", "Tab 1"));
        assert_eq!(tabs.tabs.len(), 1);
    }

    #[test]
    fn test_select_tab() {
        let mut tabs = ModernTabs::new().with_tabs(vec![
            TabConfig::new("tab1", "Tab 1"),
            TabConfig::new("tab2", "Tab 2"),
        ]);

        assert!(tabs.select_tab(1));
        assert_eq!(tabs.selected_index, 1);
        assert!(!tabs.select_tab(2)); // Out of bounds
    }

    #[test]
    fn test_select_navigation() {
        let mut tabs = ModernTabs::new().with_tabs(vec![
            TabConfig::new("tab1", "Tab 1"),
            TabConfig::new("tab2", "Tab 2"),
            TabConfig::new("tab3", "Tab 3"),
        ]);

        // Navigate right
        assert!(tabs.select_next());
        assert_eq!(tabs.selected_index, 1);
        assert!(tabs.select_next());
        assert_eq!(tabs.selected_index, 2);
        assert!(!tabs.select_next()); // Can't go past end

        // Navigate left
        assert!(tabs.select_previous());
        assert_eq!(tabs.selected_index, 1);
        assert!(tabs.select_previous());
        assert_eq!(tabs.selected_index, 0);
        assert!(!tabs.select_previous()); // Can't go before start
    }

    #[test]
    fn test_get_selected_methods() {
        let mut tabs = ModernTabs::new().with_tabs(vec![
            TabConfig::new("tab1", "Tab 1"),
            TabConfig::new("tab2", "Tab 2"),
        ]);

        tabs.select_tab(1);
        assert_eq!(tabs.get_selected_id(), Some("tab2"));
        assert_eq!(tabs.get_selected_title(), Some("Tab 2"));
    }

    #[test]
    fn test_render_tabs() {
        let tabs = ModernTabs::new()
            .with_tabs(vec![
                TabConfig::new("tab1", "Tab 1"),
                TabConfig::new("tab2", "Tab 2"),
            ])
            .with_selected(1);

        let spans = tabs.render_tabs();

        // Should have tab titles and separator
        let combined: String = spans.iter().map(|s| s.content.to_string()).collect();
        assert!(combined.contains("Tab 1"));
        assert!(combined.contains("Tab 2"));
        assert!(combined.contains("│"));
    }

    #[test]
    fn test_mock_component_perform() {
        use tuirealm::command::Direction;

        let mut tabs = ModernTabs::new().with_tabs(vec![
            TabConfig::new("tab1", "Tab 1"),
            TabConfig::new("tab2", "Tab 2"),
        ]);

        // Test left navigation
        tabs.selected_index = 1;
        let result = tabs.perform(Cmd::Move(Direction::Left));
        assert!(matches!(
            result,
            CmdResult::Submit(State::One(StateValue::Usize(0)))
        ));

        // Test right navigation
        let result = tabs.perform(Cmd::Move(Direction::Right));
        assert!(matches!(
            result,
            CmdResult::Submit(State::One(StateValue::Usize(1)))
        ));

        // Test invalid navigation
        let result = tabs.perform(Cmd::Move(Direction::Right));
        assert!(matches!(result, CmdResult::None));
    }

    #[test]
    fn test_component_events() {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};

        let mut tabs = ModernTabs::new().with_tabs(vec![
            TabConfig::new("tab1", "Tab 1"),
            TabConfig::new("tab2", "Tab 2"),
        ]);

        // Test left key
        let _msg = tabs.on(Event::Keyboard(KeyEvent::new(
            Key::Left,
            KeyModifiers::NONE,
        )));
        // Should be None since we're at index 0

        // Test enter key
        tabs.selected_index = 1;
        let msg = tabs.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert!(matches!(msg, Some(Msg::User(UserEvent::TabActivated(_)))));
    }
}
