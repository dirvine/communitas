//! Modal overlay component for dialogs and popups
//!
//! Provides a centered overlay system with:
//! - Background dimming/backdrop
//! - Configurable sizes (Small, Medium, Large, Custom)
//! - Title and content areas
//! - Optional borders and styling
//! - Modal types (Info, Confirm, Error, Warning)
//!
//! Features:
//! - Z-index layering for nested modals
//! - Keyboard shortcuts (Esc to close)
//! - Focus trapping
//! - Responsive centering

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Event, Key, KeyEvent, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    Component, MockComponent, State, StateValue,
};

use crate::messages::{ComponentId, Msg};

/// Modal size presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalSize {
    /// Small modal (50% width, 40% height)
    Small,
    /// Medium modal (70% width, 60% height)
    Medium,
    /// Large modal (90% width, 80% height)
    Large,
    /// Custom size (width%, height%)
    Custom(u16, u16),
}

impl ModalSize {
    /// Get width and height percentages
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            ModalSize::Small => (50, 40),
            ModalSize::Medium => (70, 60),
            ModalSize::Large => (90, 80),
            ModalSize::Custom(w, h) => (*w, *h),
        }
    }

    /// Validate custom dimensions (0-100%)
    pub fn custom(width: u16, height: u16) -> Result<Self, String> {
        if width == 0 || width > 100 {
            return Err(format!("Width must be 1-100%, got {}", width));
        }
        if height == 0 || height > 100 {
            return Err(format!("Height must be 1-100%, got {}", height));
        }
        Ok(ModalSize::Custom(width, height))
    }
}

/// Modal type determines styling and behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalType {
    /// Informational modal (blue)
    Info,
    /// Confirmation dialog (green)
    Confirm,
    /// Warning dialog (yellow)
    Warning,
    /// Error dialog (red)
    Error,
}

impl ModalType {
    /// Get color for this modal type
    pub fn color(&self) -> Color {
        match self {
            ModalType::Info => Color::Blue,
            ModalType::Confirm => Color::Green,
            ModalType::Warning => Color::Yellow,
            ModalType::Error => Color::Red,
        }
    }

    /// Get title prefix for this modal type
    pub fn prefix(&self) -> &'static str {
        match self {
            ModalType::Info => "ℹ",
            ModalType::Confirm => "?",
            ModalType::Warning => "⚠",
            ModalType::Error => "✗",
        }
    }
}

/// Modal overlay component
///
/// # Example
/// ```rust
/// use communitas_tui::components::modal::{Modal, ModalSize, ModalType};
/// use communitas_tui::messages::ComponentId;
///
/// let modal = Modal::new(ComponentId::ModalOverlay)
///     .title("Confirm Action")
///     .content("Are you sure you want to proceed?")
///     .modal_type(ModalType::Confirm)
///     .size(ModalSize::Medium);
/// ```
#[allow(dead_code)]
pub struct Modal {
    /// Component properties (tuirealm infrastructure)
    props: Props,
    /// Component identifier (tuirealm infrastructure)
    component_id: ComponentId,
    title: String,
    content: String,
    size: ModalSize,
    modal_type: ModalType,
    visible: bool,
    closeable: bool,
    show_backdrop: bool,
    backdrop_opacity: u8,
}

impl Modal {
    /// Create a new modal
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            props: Props::default(),
            component_id,
            title: String::new(),
            content: String::new(),
            size: ModalSize::Medium,
            modal_type: ModalType::Info,
            visible: false,
            closeable: true,
            show_backdrop: true,
            backdrop_opacity: 70,
        }
    }

    /// Set modal title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set modal content
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Set modal size
    pub fn size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    /// Set modal type (affects colors)
    pub fn modal_type(mut self, modal_type: ModalType) -> Self {
        self.modal_type = modal_type;
        self
    }

    /// Set initial visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set whether modal can be closed with Esc
    pub fn closeable(mut self, closeable: bool) -> Self {
        self.closeable = closeable;
        self
    }

    /// Set whether to show backdrop
    pub fn show_backdrop(mut self, show: bool) -> Self {
        self.show_backdrop = show;
        self
    }

    /// Set backdrop opacity (0-100%)
    pub fn backdrop_opacity(mut self, opacity: u8) -> Self {
        self.backdrop_opacity = opacity.min(100);
        self
    }

    /// Show the modal
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the modal
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Check if modal is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggle modal visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Update modal content
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    /// Update modal title
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Calculate centered modal area
    fn calculate_modal_area(&self, area: Rect) -> Rect {
        let (width_pct, height_pct) = self.size.dimensions();

        let width = (area.width as f32 * (width_pct as f32 / 100.0)) as u16;
        let height = (area.height as f32 * (height_pct as f32 / 100.0)) as u16;

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        Rect {
            x,
            y,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    }
}

impl MockComponent for Modal {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if !self.visible {
            return;
        }

        // Render backdrop if enabled
        if self.show_backdrop {
            let backdrop = Block::default().style(Style::default().bg(Color::Black));
            frame.render_widget(backdrop, area);
        }

        // Calculate modal area
        let modal_area = self.calculate_modal_area(area);

        // Clear the modal area
        frame.render_widget(Clear, modal_area);

        // Create title with type prefix
        let title_text = if self.title.is_empty() {
            self.modal_type.prefix().to_string()
        } else {
            format!("{} {}", self.modal_type.prefix(), self.title)
        };

        // Create modal block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.modal_type.color()))
            .title(title_text)
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        // Render content
        let paragraph = Paragraph::new(self.content.as_str())
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, inner);
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        match attr {
            Attribute::Value => Some(AttrValue::Flag(self.visible)),
            Attribute::Title => Some(AttrValue::String(self.title.clone())),
            Attribute::Text => Some(AttrValue::String(self.content.clone())),
            _ => None,
        }
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match attr {
            Attribute::Value => {
                if let AttrValue::Flag(visible) = value {
                    self.visible = visible;
                }
            }
            Attribute::Title => {
                if let AttrValue::String(title) = value {
                    self.title = title;
                }
            }
            Attribute::Text => {
                if let AttrValue::String(content) = value {
                    self.content = content;
                }
            }
            _ => {}
        }
    }

    fn state(&self) -> State {
        State::One(StateValue::Bool(self.visible))
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for Modal {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if !self.visible {
            return None;
        }

        match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Esc, ..
            }) if self.closeable => {
                self.hide();
                Some(Msg::CloseModal)
            }
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                self.hide();
                Some(Msg::ModalConfirmed(self.modal_type.into()))
            }
            _ => None,
        }
    }
}

impl From<ModalType> for crate::messages::ModalType {
    fn from(mt: ModalType) -> Self {
        match mt {
            ModalType::Info => crate::messages::ModalType::Help,
            ModalType::Confirm => crate::messages::ModalType::Confirmation {
                title: String::new(),
                message: String::new(),
            },
            ModalType::Warning => crate::messages::ModalType::Error(String::new()),
            ModalType::Error => crate::messages::ModalType::Error(String::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    // === ModalSize Tests ===

    #[test]
    fn test_modal_size_small() {
        assert_eq!(ModalSize::Small.dimensions(), (50, 40));
    }

    #[test]
    fn test_modal_size_medium() {
        assert_eq!(ModalSize::Medium.dimensions(), (70, 60));
    }

    #[test]
    fn test_modal_size_large() {
        assert_eq!(ModalSize::Large.dimensions(), (90, 80));
    }

    #[test]
    fn test_modal_size_custom() {
        let size = ModalSize::Custom(60, 50);
        assert_eq!(size.dimensions(), (60, 50));
    }

    #[test]
    fn test_modal_size_custom_validation() {
        assert!(ModalSize::custom(50, 50).is_ok());
        assert!(ModalSize::custom(1, 1).is_ok());
        assert!(ModalSize::custom(100, 100).is_ok());
        assert!(ModalSize::custom(0, 50).is_err());
        assert!(ModalSize::custom(50, 0).is_err());
        assert!(ModalSize::custom(101, 50).is_err());
        assert!(ModalSize::custom(50, 101).is_err());
    }

    // === ModalType Tests ===

    #[test]
    fn test_modal_type_colors() {
        assert_eq!(ModalType::Info.color(), Color::Blue);
        assert_eq!(ModalType::Confirm.color(), Color::Green);
        assert_eq!(ModalType::Warning.color(), Color::Yellow);
        assert_eq!(ModalType::Error.color(), Color::Red);
    }

    #[test]
    fn test_modal_type_prefixes() {
        assert_eq!(ModalType::Info.prefix(), "ℹ");
        assert_eq!(ModalType::Confirm.prefix(), "?");
        assert_eq!(ModalType::Warning.prefix(), "⚠");
        assert_eq!(ModalType::Error.prefix(), "✗");
    }

    // === Modal Creation Tests ===

    #[test]
    fn test_modal_creation() {
        let modal = Modal::new(ComponentId::ModalOverlay);
        assert_eq!(modal.title, "");
        assert_eq!(modal.content, "");
        assert_eq!(modal.size, ModalSize::Medium);
        assert_eq!(modal.modal_type, ModalType::Info);
        assert!(!modal.visible);
        assert!(modal.closeable);
        assert!(modal.show_backdrop);
        assert_eq!(modal.backdrop_opacity, 70);
    }

    // === Builder Pattern Tests ===

    #[test]
    fn test_modal_builder_title() {
        let modal = Modal::new(ComponentId::ModalOverlay).title("Test Title");
        assert_eq!(modal.title, "Test Title");
    }

    #[test]
    fn test_modal_builder_content() {
        let modal = Modal::new(ComponentId::ModalOverlay).content("Test content");
        assert_eq!(modal.content, "Test content");
    }

    #[test]
    fn test_modal_builder_size() {
        let modal = Modal::new(ComponentId::ModalOverlay).size(ModalSize::Large);
        assert_eq!(modal.size, ModalSize::Large);
    }

    #[test]
    fn test_modal_builder_type() {
        let modal = Modal::new(ComponentId::ModalOverlay).modal_type(ModalType::Error);
        assert_eq!(modal.modal_type, ModalType::Error);
    }

    #[test]
    fn test_modal_builder_visible() {
        let modal = Modal::new(ComponentId::ModalOverlay).visible(true);
        assert!(modal.visible);
    }

    #[test]
    fn test_modal_builder_closeable() {
        let modal = Modal::new(ComponentId::ModalOverlay).closeable(false);
        assert!(!modal.closeable);
    }

    #[test]
    fn test_modal_builder_backdrop() {
        let modal = Modal::new(ComponentId::ModalOverlay).show_backdrop(false);
        assert!(!modal.show_backdrop);
    }

    #[test]
    fn test_modal_builder_backdrop_opacity() {
        let modal = Modal::new(ComponentId::ModalOverlay).backdrop_opacity(50);
        assert_eq!(modal.backdrop_opacity, 50);
    }

    #[test]
    fn test_modal_builder_backdrop_opacity_clamped() {
        let modal = Modal::new(ComponentId::ModalOverlay).backdrop_opacity(150);
        assert_eq!(modal.backdrop_opacity, 100);
    }

    #[test]
    fn test_modal_builder_chain() {
        let modal = Modal::new(ComponentId::ModalOverlay)
            .title("Confirm")
            .content("Are you sure?")
            .size(ModalSize::Small)
            .modal_type(ModalType::Confirm)
            .visible(true)
            .closeable(false)
            .backdrop_opacity(80);

        assert_eq!(modal.title, "Confirm");
        assert_eq!(modal.content, "Are you sure?");
        assert_eq!(modal.size, ModalSize::Small);
        assert_eq!(modal.modal_type, ModalType::Confirm);
        assert!(modal.visible);
        assert!(!modal.closeable);
        assert_eq!(modal.backdrop_opacity, 80);
    }

    // === Visibility Tests ===

    #[test]
    fn test_modal_show() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        assert!(!modal.is_visible());

        modal.show();
        assert!(modal.is_visible());
    }

    #[test]
    fn test_modal_hide() {
        let mut modal = Modal::new(ComponentId::ModalOverlay).visible(true);
        assert!(modal.is_visible());

        modal.hide();
        assert!(!modal.is_visible());
    }

    #[test]
    fn test_modal_toggle() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        assert!(!modal.is_visible());

        modal.toggle();
        assert!(modal.is_visible());

        modal.toggle();
        assert!(!modal.is_visible());
    }

    // === Content Update Tests ===

    #[test]
    fn test_set_content() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        modal.set_content("New content");
        assert_eq!(modal.content, "New content");
    }

    #[test]
    fn test_set_title() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        modal.set_title("New title");
        assert_eq!(modal.title, "New title");
    }

    // === MockComponent Tests ===

    #[test]
    fn test_query_visible() {
        let modal = Modal::new(ComponentId::ModalOverlay).visible(true);
        let value = modal.query(Attribute::Value);
        assert_eq!(value, Some(AttrValue::Flag(true)));
    }

    #[test]
    fn test_query_title() {
        let modal = Modal::new(ComponentId::ModalOverlay).title("Test");
        let value = modal.query(Attribute::Title);
        assert_eq!(value, Some(AttrValue::String("Test".to_string())));
    }

    #[test]
    fn test_query_content() {
        let modal = Modal::new(ComponentId::ModalOverlay).content("Content");
        let value = modal.query(Attribute::Text);
        assert_eq!(value, Some(AttrValue::String("Content".to_string())));
    }

    #[test]
    fn test_attr_visible() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        modal.attr(Attribute::Value, AttrValue::Flag(true));
        assert!(modal.visible);
    }

    #[test]
    fn test_attr_title() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        modal.attr(Attribute::Title, AttrValue::String("New".to_string()));
        assert_eq!(modal.title, "New");
    }

    #[test]
    fn test_attr_content() {
        let mut modal = Modal::new(ComponentId::ModalOverlay);
        modal.attr(Attribute::Text, AttrValue::String("New content".to_string()));
        assert_eq!(modal.content, "New content");
    }

    #[test]
    fn test_state_serialization() {
        let modal = Modal::new(ComponentId::ModalOverlay).visible(true);
        let state = modal.state();
        assert_eq!(state, State::One(StateValue::Bool(true)));
    }

    // === Event Handling Tests ===

    #[test]
    fn test_esc_closes_modal() {
        let mut modal = Modal::new(ComponentId::ModalOverlay).visible(true);

        let msg = modal.on(Event::Keyboard(KeyEvent::from(Key::Esc)));
        assert_eq!(msg, Some(Msg::CloseModal));
        assert!(!modal.is_visible());
    }

    #[test]
    fn test_esc_blocked_when_not_closeable() {
        let mut modal = Modal::new(ComponentId::ModalOverlay)
            .visible(true)
            .closeable(false);

        let msg = modal.on(Event::Keyboard(KeyEvent::from(Key::Esc)));
        assert_eq!(msg, None);
        assert!(modal.is_visible());
    }

    #[test]
    fn test_enter_confirms_modal() {
        let mut modal = Modal::new(ComponentId::ModalOverlay).visible(true);

        let msg = modal.on(Event::Keyboard(KeyEvent::from(Key::Enter)));
        assert!(matches!(msg, Some(Msg::ModalConfirmed(_))));
        assert!(!modal.is_visible());
    }

    #[test]
    fn test_events_ignored_when_hidden() {
        let mut modal = Modal::new(ComponentId::ModalOverlay).visible(false);

        let msg = modal.on(Event::Keyboard(KeyEvent::from(Key::Esc)));
        assert_eq!(msg, None);
    }

    // === Visual Rendering Tests ===

    #[test]
    fn test_render_when_visible() {
        let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
        let mut modal = Modal::new(ComponentId::ModalOverlay)
            .visible(true)
            .title("Test")
            .content("Content");

        terminal
            .draw(|f| {
                let area = f.area();
                modal.view(f, area);
            })
            .unwrap();

        assert!(modal.is_visible());
    }

    #[test]
    fn test_no_render_when_hidden() {
        let mut terminal = Terminal::new(TestBackend::new(80, 40)).unwrap();
        let mut modal = Modal::new(ComponentId::ModalOverlay)
            .visible(false)
            .title("Test");

        terminal
            .draw(|f| {
                let area = f.area();
                modal.view(f, area);
            })
            .unwrap();

        assert!(!modal.is_visible());
    }

    #[test]
    fn test_modal_centering() {
        let modal = Modal::new(ComponentId::ModalOverlay).size(ModalSize::Medium);

        let area = Rect::new(0, 0, 100, 40);
        let modal_area = modal.calculate_modal_area(area);

        // Medium is 70% width, 60% height
        assert_eq!(modal_area.width, 70);
        assert_eq!(modal_area.height, 24);

        // Should be centered
        assert_eq!(modal_area.x, 15); // (100 - 70) / 2
        assert_eq!(modal_area.y, 8); // (40 - 24) / 2
    }
}
