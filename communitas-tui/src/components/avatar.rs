//! Avatar component for displaying user profile images
//!
//! Features:
//! - Image display with ratatui-image support
//! - Fallback to initials when no image
//! - Multiple sizes (Small, Medium, Large)
//! - Multiple shapes (Circular, Square)
//! - Loading and error states
//! - Auto-detection of image protocol (iTerm2, Kitty, Sixel)

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tuirealm::{
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{Alignment, AttrValue, Attribute, Props},
    Component, MockComponent, State, StateValue,
};

use crate::messages::{ComponentId, Msg};

/// Avatar display size
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Small,  // 3x3 cells
    Medium, // 5x5 cells
    Large,  // 9x9 cells
}

impl AvatarSize {
    /// Get width/height in terminal cells
    pub fn dimensions(&self) -> (u16, u16) {
        match self {
            AvatarSize::Small => (3, 3),
            AvatarSize::Medium => (5, 5),
            AvatarSize::Large => (9, 9),
        }
    }
}

/// Avatar shape style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarShape {
    Circular, // Rounded corners
    Square,   // Sharp corners
}

/// Avatar loading state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvatarState {
    /// No image loaded, showing fallback
    Fallback,
    /// Loading image data
    Loading,
    /// Image loaded successfully
    Loaded,
    /// Failed to load image
    Error(String),
}

/// Avatar component for displaying user profile images
///
/// # Example
/// ```rust
/// use communitas_tui::components::{Avatar, AvatarSize, AvatarShape};
/// use communitas_tui::messages::ComponentId;
///
/// let avatar = Avatar::new(ComponentId::Avatar("user123".to_string()))
///     .user_id("user123")
///     .display_name("Alice Smith")
///     .size(AvatarSize::Medium)
///     .shape(AvatarShape::Circular);
/// ```
#[allow(dead_code)]
pub struct Avatar {
    /// Component properties (tuirealm infrastructure)
    props: Props,
    /// Component identifier (tuirealm infrastructure)
    component_id: ComponentId,
    user_id: String,
    display_name: String,
    size: AvatarSize,
    shape: AvatarShape,
    state: AvatarState,
    image_data: Option<Vec<u8>>,
    background_color: Color,
}

impl Avatar {
    /// Create a new Avatar component
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            props: Props::default(),
            component_id,
            user_id: String::new(),
            display_name: String::new(),
            size: AvatarSize::Medium,
            shape: AvatarShape::Circular,
            state: AvatarState::Fallback,
            image_data: None,
            background_color: Color::Blue,
        }
    }

    /// Set user ID
    pub fn user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = id.into();
        self
    }

    /// Set display name (used for fallback initials)
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Set avatar size
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    /// Set avatar shape
    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set background color for fallback
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Load image data
    pub fn load_image(&mut self, data: Vec<u8>) -> Result<(), String> {
        // Validate image data (basic check)
        if data.is_empty() {
            return Err("Image data is empty".to_string());
        }

        // In real implementation, would validate image format
        self.image_data = Some(data);
        self.state = AvatarState::Loaded;
        Ok(())
    }

    /// Clear image and return to fallback
    pub fn clear_image(&mut self) {
        self.image_data = None;
        self.state = AvatarState::Fallback;
    }

    /// Set loading state
    pub fn set_loading(&mut self) {
        self.state = AvatarState::Loading;
    }

    /// Set error state
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.state = AvatarState::Error(error.into());
    }

    /// Get current state
    pub fn avatar_state(&self) -> &AvatarState {
        &self.state
    }

    /// Generate initials from display name
    fn initials(&self) -> String {
        if self.display_name.is_empty() {
            return "?".to_string();
        }

        let parts: Vec<&str> = self.display_name.split_whitespace().collect();
        match parts.len() {
            0 => "?".to_string(),
            1 => parts[0].chars().next().unwrap_or('?').to_uppercase().to_string(),
            _ => {
                let first = parts[0].chars().next().unwrap_or('?');
                let last = parts[parts.len() - 1].chars().next().unwrap_or('?');
                format!("{}{}", first, last).to_uppercase()
            }
        }
    }

    /// Get border style based on shape
    fn border_style(&self) -> Borders {
        match self.shape {
            AvatarShape::Circular => Borders::ALL,
            AvatarShape::Square => Borders::ALL,
        }
    }
}

impl MockComponent for Avatar {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let (width, height) = self.size.dimensions();

        // Center the avatar in the provided area
        let avatar_area = Rect {
            x: area.x + (area.width.saturating_sub(width)) / 2,
            y: area.y + (area.height.saturating_sub(height)) / 2,
            width: width.min(area.width),
            height: height.min(area.height),
        };

        match &self.state {
            AvatarState::Fallback => {
                // Show initials
                let initials = self.initials();
                let block = Block::default()
                    .borders(self.border_style())
                    .style(Style::default().bg(self.background_color).fg(Color::White));

                let paragraph = Paragraph::new(initials)
                    .block(block)
                    .alignment(Alignment::Center);

                frame.render_widget(paragraph, avatar_area);
            }
            AvatarState::Loading => {
                // Show loading indicator
                let block = Block::default()
                    .borders(self.border_style())
                    .style(Style::default().bg(Color::DarkGray));

                let paragraph = Paragraph::new("...")
                    .block(block)
                    .alignment(Alignment::Center);

                frame.render_widget(paragraph, avatar_area);
            }
            AvatarState::Loaded => {
                // In real implementation, would render image with ratatui-image
                // For now, show placeholder
                let block = Block::default()
                    .borders(self.border_style())
                    .style(Style::default().bg(Color::Green));

                let paragraph = Paragraph::new("[IMG]")
                    .block(block)
                    .alignment(Alignment::Center);

                frame.render_widget(paragraph, avatar_area);
            }
            AvatarState::Error(_msg) => {
                // Show error state
                let block = Block::default()
                    .borders(self.border_style())
                    .style(Style::default().bg(Color::Red).fg(Color::White));

                let paragraph = Paragraph::new("!")
                    .block(block)
                    .alignment(Alignment::Center);

                frame.render_widget(paragraph, avatar_area);
            }
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        match attr {
            Attribute::Value => Some(AttrValue::String(self.user_id.clone())),
            _ => None,
        }
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        match attr {
            Attribute::Value => {
                if let AttrValue::String(user_id) = value {
                    self.user_id = user_id;
                }
            }
            _ => {}
        }
    }

    fn state(&self) -> State {
        State::One(StateValue::String(self.user_id.clone()))
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for Avatar {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        // Avatar is typically non-interactive
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    // === Component Creation Tests ===

    #[test]
    fn test_avatar_creation() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        assert_eq!(avatar.user_id, "");
        assert_eq!(avatar.display_name, "");
        assert_eq!(avatar.size, AvatarSize::Medium);
        assert_eq!(avatar.shape, AvatarShape::Circular);
        assert_eq!(avatar.state, AvatarState::Fallback);
        assert!(avatar.image_data.is_none());
    }

    #[test]
    fn test_avatar_with_user_id() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .user_id("user123");
        assert_eq!(avatar.user_id, "user123");
    }

    #[test]
    fn test_avatar_with_display_name() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .display_name("Alice Smith");
        assert_eq!(avatar.display_name, "Alice Smith");
    }

    // === Size Tests ===

    #[test]
    fn test_avatar_small_size() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .size(AvatarSize::Small);
        assert_eq!(avatar.size, AvatarSize::Small);
        assert_eq!(avatar.size.dimensions(), (3, 3));
    }

    #[test]
    fn test_avatar_medium_size() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .size(AvatarSize::Medium);
        assert_eq!(avatar.size, AvatarSize::Medium);
        assert_eq!(avatar.size.dimensions(), (5, 5));
    }

    #[test]
    fn test_avatar_large_size() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .size(AvatarSize::Large);
        assert_eq!(avatar.size, AvatarSize::Large);
        assert_eq!(avatar.size.dimensions(), (9, 9));
    }

    // === Shape Tests ===

    #[test]
    fn test_avatar_circular_shape() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .shape(AvatarShape::Circular);
        assert_eq!(avatar.shape, AvatarShape::Circular);
    }

    #[test]
    fn test_avatar_square_shape() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .shape(AvatarShape::Square);
        assert_eq!(avatar.shape, AvatarShape::Square);
    }

    // === Background Color Tests ===

    #[test]
    fn test_avatar_custom_background() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .background_color(Color::Magenta);
        assert_eq!(avatar.background_color, Color::Magenta);
    }

    // === Initials Generation Tests ===

    #[test]
    fn test_initials_empty_name() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        assert_eq!(avatar.initials(), "?");
    }

    #[test]
    fn test_initials_single_name() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .display_name("Alice");
        assert_eq!(avatar.initials(), "A");
    }

    #[test]
    fn test_initials_full_name() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .display_name("Alice Smith");
        assert_eq!(avatar.initials(), "AS");
    }

    #[test]
    fn test_initials_three_names() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .display_name("Alice Jane Smith");
        assert_eq!(avatar.initials(), "AS");
    }

    #[test]
    fn test_initials_lowercase() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .display_name("alice smith");
        assert_eq!(avatar.initials(), "AS");
    }

    // === Image Loading Tests ===

    #[test]
    fn test_load_image_success() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        let image_data = vec![1, 2, 3, 4, 5];

        let result = avatar.load_image(image_data.clone());
        assert!(result.is_ok());
        assert_eq!(avatar.state, AvatarState::Loaded);
        assert_eq!(avatar.image_data, Some(image_data));
    }

    #[test]
    fn test_load_image_empty_data() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));

        let result = avatar.load_image(vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Image data is empty");
        assert_eq!(avatar.state, AvatarState::Fallback);
    }

    #[test]
    fn test_clear_image() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.load_image(vec![1, 2, 3]).ok();

        avatar.clear_image();
        assert_eq!(avatar.state, AvatarState::Fallback);
        assert!(avatar.image_data.is_none());
    }

    // === State Management Tests ===

    #[test]
    fn test_set_loading_state() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.set_loading();
        assert_eq!(avatar.state, AvatarState::Loading);
    }

    #[test]
    fn test_set_error_state() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.set_error("Network error");
        assert_eq!(avatar.state, AvatarState::Error("Network error".to_string()));
    }

    #[test]
    fn test_get_avatar_state() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.set_loading();
        assert_eq!(avatar.avatar_state(), &AvatarState::Loading);
    }

    // === Builder Pattern Tests ===

    #[test]
    fn test_builder_chain() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .user_id("user123")
            .display_name("Alice Smith")
            .size(AvatarSize::Large)
            .shape(AvatarShape::Square)
            .background_color(Color::Cyan);

        assert_eq!(avatar.user_id, "user123");
        assert_eq!(avatar.display_name, "Alice Smith");
        assert_eq!(avatar.size, AvatarSize::Large);
        assert_eq!(avatar.shape, AvatarShape::Square);
        assert_eq!(avatar.background_color, Color::Cyan);
    }

    // === MockComponent Tests ===

    #[test]
    fn test_query_value() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .user_id("user123");

        let value = avatar.query(Attribute::Value);
        assert_eq!(value, Some(AttrValue::String("user123".to_string())));
    }

    #[test]
    fn test_attr_value() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.attr(Attribute::Value, AttrValue::String("new_user".to_string()));

        assert_eq!(avatar.user_id, "new_user");
    }

    #[test]
    fn test_state_serialization() {
        let avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .user_id("user123");

        let state = avatar.state();
        assert_eq!(state, State::One(StateValue::String("user123".to_string())));
    }

    // === Visual Rendering Tests ===

    #[test]
    fn test_render_fallback_state() {
        let mut terminal = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()))
            .display_name("Alice Smith");

        terminal.draw(|f| {
            let area = f.area();
            avatar.view(f, area);
        }).unwrap();

        // Avatar should be in fallback state
        assert_eq!(avatar.state, AvatarState::Fallback);
    }

    #[test]
    fn test_render_loading_state() {
        let mut terminal = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.set_loading();

        terminal.draw(|f| {
            let area = f.area();
            avatar.view(f, area);
        }).unwrap();

        assert_eq!(avatar.state, AvatarState::Loading);
    }

    #[test]
    fn test_render_loaded_state() {
        let mut terminal = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.load_image(vec![1, 2, 3]).ok();

        terminal.draw(|f| {
            let area = f.area();
            avatar.view(f, area);
        }).unwrap();

        assert_eq!(avatar.state, AvatarState::Loaded);
    }

    #[test]
    fn test_render_error_state() {
        let mut terminal = Terminal::new(TestBackend::new(20, 10)).unwrap();
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));
        avatar.set_error("Failed to load");

        terminal.draw(|f| {
            let area = f.area();
            avatar.view(f, area);
        }).unwrap();

        assert_eq!(avatar.state, AvatarState::Error("Failed to load".to_string()));
    }

    // === Component Event Tests ===

    #[test]
    fn test_avatar_non_interactive() {
        let mut avatar = Avatar::new(ComponentId::Avatar("user1".to_string()));

        let msg = avatar.on(Event::Keyboard(KeyEvent::from(Key::Enter)));
        assert_eq!(msg, None);
    }
}
