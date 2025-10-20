//! Accessibility features component
//!
//! Provides accessibility enhancements including:
//! - Screen reader support
//! - High contrast themes
//! - Keyboard navigation enhancement
//! - Text-to-speech capabilities
//! - Focus management

use crate::messages::{Msg, UserEvent};
use std::collections::HashMap;
use std::time::Duration;
use tuirealm::{
    Component, Frame, MockComponent, State,
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    },
};

/// Accessibility settings
#[derive(Debug, Clone)]
pub struct AccessibilitySettings {
    /// Enable screen reader announcements
    pub screen_reader: bool,
    /// High contrast mode
    pub high_contrast: bool,
    /// Large text mode
    pub large_text: bool,
    /// Announcement volume (0.0-1.0)
    pub announcement_volume: f32,
    /// Focus indicator mode
    pub focus_indicator: FocusIndicator,
    /// Announcement delay
    pub announcement_delay: Duration,
    /// Keyboard help mode
    pub keyboard_help: bool,
    /// Reduced motion animation
    pub reduced_motion: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            screen_reader: false,
            high_contrast: false,
            large_text: false,
            announcement_volume: 0.8,
            focus_indicator: FocusIndicator::Border,
            announcement_delay: Duration::from_millis(500),
            keyboard_help: false,
            reduced_motion: false,
        }
    }
}

/// Focus indicator styles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusIndicator {
    /// No focus indicator
    None,
    /// Border around focused element
    Border,
    /// Background highlight
    Background,
    /// Arrow pointer
    Arrow,
    /// Both border and background
    Combined,
}

/// Screen reader announcement
#[derive(Debug, Clone)]
pub struct Announcement {
    /// Announcement text
    pub text: String,
    /// Announcement priority
    pub priority: AnnouncementPriority,
    /// Timestamp
    pub timestamp: std::time::Instant,
    /// Whether to queue or interrupt current announcement
    pub interrupt: bool,
}

impl Announcement {
    pub fn new(text: String, priority: AnnouncementPriority) -> Self {
        Self {
            text,
            priority: priority.clone(),
            timestamp: std::time::Instant::now(),
            interrupt: matches!(priority, AnnouncementPriority::Critical),
        }
    }

    pub fn critical(text: String) -> Self {
        Self::new(text, AnnouncementPriority::Critical)
    }

    pub fn high_priority(text: String) -> Self {
        Self::new(text, AnnouncementPriority::High)
    }

    pub fn normal(text: String) -> Self {
        Self::new(text, AnnouncementPriority::Normal)
    }

    pub fn low(text: String) -> Self {
        Self::new(text, AnnouncementPriority::Low)
    }

    pub fn is_expired(&self, timeout: Duration) -> bool {
        self.timestamp.elapsed() > timeout
    }
}

/// Announcement priority levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnouncementPriority {
    /// Low priority (optional information)
    Low,
    /// Normal priority (important information)
    Normal,
    /// High priority (user needs to know)
    High,
    /// Critical (announcements that must be heard)
    Critical,
}

/// Focus tracking for accessibility
#[derive(Debug, Clone)]
pub struct FocusTracking {
    /// Currently focused component ID
    pub focused_component: Option<String>,
    /// Focus history for navigation
    pub focus_history: Vec<String>,
    /// Maximum history size
    pub max_history: usize,
}

impl Default for FocusTracking {
    fn default() -> Self {
        Self::new()
    }
}

impl FocusTracking {
    pub fn new() -> Self {
        Self {
            focused_component: None,
            focus_history: Vec::new(),
            max_history: 50,
        }
    }

    pub fn set_focus(&mut self, component_id: String) -> Option<String> {
        let previous = self.focused_component.clone();

        // Update history
        if let Some(ref current) = previous {
            self.focus_history.push(current.clone());

            // Limit history size
            if self.focus_history.len() > self.max_history {
                self.focus_history.remove(0);
            }
        }

        self.focused_component = Some(component_id);
        previous
    }

    pub fn can_go_back(&self) -> bool {
        !self.focus_history.is_empty()
    }

    pub fn go_back(&mut self) -> Option<String> {
        if let Some(previous) = self.focus_history.pop() {
            self.focused_component = Some(previous.clone());
            Some(previous)
        } else {
            None
        }
    }

    pub fn clear_history(&mut self) {
        self.focus_history.clear();
    }
}

/// Accessibility manager component
#[derive(Debug)]
pub struct AccessibilityManager {
    props: Props,
    settings: AccessibilitySettings,
    _pending_announcements: Vec<Announcement>,
    focus_tracking: FocusTracking,
    keyboard_help_visible: bool,
    announcement_queue: Vec<Announcement>,
    last_announcement_time: Option<std::time::Instant>,
}

impl Default for AccessibilityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccessibilityManager {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            settings: AccessibilitySettings::default(),
            _pending_announcements: Vec::new(),
            focus_tracking: FocusTracking::new(),
            keyboard_help_visible: false,
            announcement_queue: Vec::new(),
            last_announcement_time: None,
        }
    }

    pub fn settings(&self) -> &AccessibilitySettings {
        &self.settings
    }

    pub fn toggle_setting(&mut self, setting: AccessibilitySetting) {
        match setting {
            AccessibilitySetting::ScreenReader => {
                self.settings.screen_reader = !self.settings.screen_reader;
            }
            AccessibilitySetting::HighContrast => {
                self.settings.high_contrast = !self.settings.high_contrast;
            }
            AccessibilitySetting::LargeText => {
                self.settings.large_text = !self.settings.large_text;
            }
            AccessibilitySetting::KeyboardHelp => {
                self.settings.keyboard_help = !self.settings.keyboard_help;
                self.keyboard_help_visible = self.settings.keyboard_help;
            }
            AccessibilitySetting::ReducedMotion => {
                self.settings.reduced_motion = !self.settings.reduced_motion;
            }
        }
    }

    pub fn announce(&mut self, text: String) {
        if self.settings.screen_reader {
            self.queue_announcement(Announcement::normal(text));
        }
    }

    pub fn announce_important(&mut self, text: String) {
        if self.settings.screen_reader {
            self.queue_announcement(Announcement::high_priority(text));
        }
    }

    pub fn announce_critical(&mut self, text: String) {
        if self.settings.screen_reader {
            self.queue_announcement(Announcement::new(text, AnnouncementPriority::Critical));
        }
    }

    fn queue_announcement(&mut self, announcement: Announcement) {
        if announcement.interrupt || self.announcement_queue.is_empty() {
            // Interrupt current or start new announcement
            self.announcement_queue.clear();
            self.announcement_queue.push(announcement);
        } else {
            // Add to queue
            self.announcement_queue.push(announcement);
        }
    }

    fn process_announcement_queue(&mut self) -> Option<String> {
        if !self.settings.screen_reader || self.announcement_queue.is_empty() {
            return None;
        }

        // Check if we should process the next announcement
        let now = std::time::Instant::now();
        if let Some(last_time) = self.last_announcement_time
            && now.duration_since(last_time) < self.settings.announcement_delay
        {
            return None;
        }

        // Get next announcement
        if !self.announcement_queue.is_empty() {
            let announcement = self.announcement_queue.remove(0);
            self.last_announcement_time = Some(now);

            // In a real implementation, this would call a screen reader library
            tracing::debug!("Screen reader: {}", announcement.text);

            Some(announcement.text.clone())
        } else {
            None
        }
    }

    pub fn set_focus(&mut self, component_id: String) -> Option<String> {
        let previous = self.focus_tracking.set_focus(component_id.clone());

        // Announce focus change if screen reader is enabled
        if self.settings.screen_reader {
            let announcement = if let Some(ref _prev) = previous {
                format!("Focused {}", component_id)
            } else {
                format!("Focused {}", component_id)
            };
            self.queue_announcement(Announcement::normal(announcement));
        }

        previous
    }

    pub fn is_keyboard_help_visible(&self) -> bool {
        self.keyboard_help_visible
    }

    pub fn get_high_contrast_theme(&self) -> HashMap<Color, Color> {
        if !self.settings.high_contrast {
            return HashMap::new();
        }

        // High contrast color mappings
        let mut theme = HashMap::new();
        theme.insert(Color::Black, Color::White);
        theme.insert(Color::White, Color::Black);
        theme.insert(Color::Gray, Color::DarkGray);
        theme.insert(Color::DarkGray, Color::Gray);
        theme.insert(Color::White, Color::Blue); // LightGray -> White as closest match
        theme.insert(Color::Red, Color::LightRed);
        theme.insert(Color::Green, Color::LightGreen);
        theme.insert(Color::Yellow, Color::LightYellow);
        theme.insert(Color::Blue, Color::LightBlue);
        theme.insert(Color::Magenta, Color::LightMagenta);
        theme.insert(Color::Cyan, Color::LightCyan);
        theme
    }

    fn render_keyboard_help(&self) -> Vec<Line<'_>> {
        vec![
            Line::from(Span::styled(
                "Keyboard Shortcuts & Navigation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Navigation:"),
            Line::from("  Tab, Shift+Tab  : Move focus between sections"),
            Line::from("  ↑↓←→ or hjkl : Navigate within lists"),
            Line::from("  1-9            : Quick jump to numbered items"),
            Line::from("  g, G           : Go to top/bottom of list"),
            Line::from(""),
            Line::from("Actions:"),
            Line::from("  Enter          : Activate/Select focused item"),
            Line::from("  Space          : Toggle selection"),
            Line::from("  n              : Create new entity"),
            Line::from("  i              : Initialize identity"),
            Line::from("  q              : Quit application"),
            Line::from(""),
            Line::from("Accessibility:"),
            Line::from("  Alt+A          : Toggle accessibility settings"),
            Line::from("  Ctrl+S         : Toggle screen reader"),
            Line::from("  Ctrl+H         : Toggle high contrast"),
            Line::from("  Ctrl+L         : Toggle large text"),
            Line::from("  Ctrl+F1        : Toggle this help"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Current Settings:", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(
                        " ScreenReader: {}",
                        if self.settings.screen_reader {
                            "ON"
                        } else {
                            "OFF"
                        }
                    ),
                    if self.settings.screen_reader {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(
                    format!(
                        " HighContrast: {}",
                        if self.settings.high_contrast {
                            "ON"
                        } else {
                            "OFF"
                        }
                    ),
                    if self.settings.high_contrast {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "                 LargeText: ",
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    (if self.settings.large_text {
                        "ON"
                    } else {
                        "OFF"
                    })
                    .to_string(),
                    if self.settings.large_text {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Esc to close help",
                Style::default().fg(Color::Gray),
            )]),
        ]
    }
}

/// Accessibility setting flags
#[derive(Debug, Clone)]
pub enum AccessibilitySetting {
    ScreenReader,
    HighContrast,
    LargeText,
    KeyboardHelp,
    ReducedMotion,
}

impl MockComponent for AccessibilityManager {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Process announcement queue
        self.process_announcement_queue();

        // Render keyboard help if visible
        if self.is_keyboard_help_visible() {
            let help_width = area.width.min(80);
            let help_height = area.height.min(30);
            let help_x = (area.width - help_width) / 2;
            let help_y = (area.height - help_height) / 2;

            let help_area = Rect {
                x: area.x + help_x,
                y: area.y + help_y,
                width: help_width,
                height: help_height,
            };

            let help_lines = self.render_keyboard_help();
            let help_paragraph = Paragraph::new(help_lines)
                .block(
                    Block::default()
                        .title(" Accessibility Help ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .alignment(Alignment::Left);

            frame.render_widget(help_paragraph, help_area);
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Custom("toggle_screen_reader") => {
                self.toggle_setting(AccessibilitySetting::ScreenReader);
                CmdResult::None
            }
            Cmd::Custom("toggle_high_contrast") => {
                self.toggle_setting(AccessibilitySetting::HighContrast);
                CmdResult::None
            }
            Cmd::Custom("toggle_large_text") => {
                self.toggle_setting(AccessibilitySetting::LargeText);
                CmdResult::None
            }
            Cmd::Custom("toggle_keyboard_help") => {
                self.toggle_setting(AccessibilitySetting::KeyboardHelp);
                CmdResult::None
            }
            Cmd::Custom("announce") => {
                if let Some(text) = self.query(Attribute::Text)
                    && let AttrValue::String(text) = text
                {
                    self.announce(text);
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for AccessibilityManager {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(event) => {
                use tuirealm::event::{Key, KeyModifiers};

                match (event.code, event.modifiers) {
                    (Key::Char('a'), KeyModifiers::ALT) => {
                        // Toggle accessibility settings panel (would be implemented)
                        Some(Msg::User(UserEvent::TaskCompleted {
                            task_id: "accessibility_toggle".to_string(),
                            result: TaskResult::Success(
                                "Accessibility settings toggled".to_string(),
                            ),
                        }))
                    }
                    (Key::Char('s'), KeyModifiers::CONTROL) => {
                        self.toggle_setting(AccessibilitySetting::ScreenReader);
                        self.announce_important(format!(
                            "Screen reader {}",
                            if self.settings.screen_reader {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        ));
                        None
                    }
                    (Key::Char('h'), KeyModifiers::CONTROL) => {
                        self.toggle_setting(AccessibilitySetting::HighContrast);
                        self.announce_important(format!(
                            "High contrast {}",
                            if self.settings.high_contrast {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        ));
                        None
                    }
                    (Key::Char('l'), KeyModifiers::CONTROL) => {
                        self.toggle_setting(AccessibilitySetting::LargeText);
                        self.announce_important(format!(
                            "Large text {}",
                            if self.settings.large_text {
                                "enabled"
                            } else {
                                "disabled"
                            }
                        ));
                        None
                    }
                    (Key::Function(1), KeyModifiers::CONTROL) => {
                        self.toggle_setting(AccessibilitySetting::KeyboardHelp);
                        None
                    }
                    (Key::Esc, _) => {
                        if self.is_keyboard_help_visible() {
                            self.keyboard_help_visible = false;
                            None
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            Event::FocusGained => {
                // Handle focus gained events for accessibility
                if let Some(ref component) = self.focus_tracking.focused_component {
                    self.announce(format!("Focused {}", component));
                }
                None
            }
            Event::FocusLost => {
                // Handle focus lost events
                None
            }
            _ => None,
        }
    }
}

// Re-export for convenience
use crate::messages::TaskResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_settings_default() {
        let settings = AccessibilitySettings::default();
        assert!(!settings.screen_reader);
        assert!(!settings.high_contrast);
        assert!(!settings.large_text);
        assert_eq!(settings.announcement_volume, 0.8);
        assert_eq!(settings.focus_indicator, FocusIndicator::Border);
    }

    #[test]
    fn test_announcement_creation() {
        let ann = Announcement::normal("Test".to_string());
        assert_eq!(ann.text, "Test");
        assert_eq!(ann.priority, AnnouncementPriority::Normal);
        assert!(!ann.interrupt);

        let critical = Announcement::critical("Critical".to_string());
        assert!(critical.interrupt);
    }

    #[test]
    fn test_focus_tracking() {
        let mut tracking = FocusTracking::new();

        let previous = tracking.set_focus("component1".to_string());
        assert!(previous.is_none());
        assert_eq!(tracking.focused_component.as_ref().unwrap(), "component1");

        let previous = tracking.set_focus("component2".to_string());
        assert_eq!(previous.unwrap(), "component1");
        assert_eq!(tracking.focus_history.len(), 1);
    }

    #[test]
    fn test_focus_tracking_navigation() {
        let mut tracking = FocusTracking::new();

        // Initially can't go back
        assert!(!tracking.can_go_back());

        tracking.set_focus("component1".to_string());
        tracking.set_focus("component2".to_string());

        // Now can go back
        assert!(tracking.can_go_back());

        let previous = tracking.go_back();
        assert!(previous.is_some());
        assert_eq!(tracking.focused_component.as_ref().unwrap(), "component1");
    }

    #[test]
    fn test_accessibility_manager_creation() {
        let manager = AccessibilityManager::new();
        assert!(!manager.settings.screen_reader);
        assert!(!manager.is_keyboard_help_visible());
    }

    #[test]
    fn test_accessibility_manager_toggle_settings() {
        let mut manager = AccessibilityManager::new();

        // Toggle screen reader
        manager.toggle_setting(AccessibilitySetting::ScreenReader);
        assert!(manager.settings.screen_reader);

        // Toggle high contrast
        manager.toggle_setting(AccessibilitySetting::HighContrast);
        assert!(manager.settings.high_contrast);

        // Toggle keyboard help
        manager.toggle_setting(AccessibilitySetting::KeyboardHelp);
        assert!(manager.is_keyboard_help_visible());
    }

    #[test]
    fn test_accessibility_manager_announce() {
        let mut manager = AccessibilityManager::new();

        // Should not queue announcement when screen reader is off
        manager.announce("Test".to_string());
        assert!(manager.announcement_queue.is_empty());

        // Enable screen reader
        manager.toggle_setting(AccessibilitySetting::ScreenReader);

        // Now should queue announcement
        manager.announce("Test".to_string());
        assert_eq!(manager.announcement_queue.len(), 1);
    }

    #[test]
    fn test_accessibility_manager_focus_tracking() {
        let mut manager = AccessibilityManager::new();

        // Enable screen reader for focus announcements
        manager.toggle_setting(AccessibilitySetting::ScreenReader);

        manager.set_focus("component1".to_string());
        assert_eq!(
            manager.focus_tracking.focused_component.as_ref().unwrap(),
            "component1"
        );
        assert!(!manager.announcement_queue.is_empty()); // Should have focus announcement
    }

    #[test]
    fn test_accessibility_component_events() {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};

        let mut manager = AccessibilityManager::new();

        // Test Ctrl+S toggle screen reader
        let _msg = manager.on(Event::Keyboard(KeyEvent::new(
            Key::Char('s'),
            KeyModifiers::CONTROL,
        )));

        // Should have screen reader enabled now
        assert!(manager.settings.screen_reader);

        // Test Ctrl+H toggle high contrast
        let _msg = manager.on(Event::Keyboard(KeyEvent::new(
            Key::Char('h'),
            KeyModifiers::CONTROL,
        )));

        assert!(manager.settings.high_contrast);

        // Test Alt+A
        let msg = manager.on(Event::Keyboard(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::ALT,
        )));

        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));
    }

    #[test]
    fn test_mock_component_perform() {
        let mut manager = AccessibilityManager::new();

        // Test toggle screen reader
        let result = manager.perform(Cmd::Custom("toggle_screen_reader"));
        assert!(matches!(result, CmdResult::None));
        assert!(manager.settings.screen_reader);

        // Test announce - need to set Text attribute first
        manager.attr(
            Attribute::Text,
            AttrValue::String("Test announcement".to_string()),
        );
        manager.perform(Cmd::Custom("announce"));
        assert!(!manager.announcement_queue.is_empty()); // Should have announcement
    }
}
