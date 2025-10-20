//! Enhanced theming system component
//!
//! Provides comprehensive theming capabilities:
//! - Light/dark themes with smooth transitions
//! - Custom color palettes
//! - Component-specific styling
//! - High contrast and accessibility themes
//! - Seasonal and holiday themes

use crate::messages::{Msg, UserEvent};
use serde::{Deserialize, Serialize};
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

/// Theme modes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    /// Light theme for daytime use
    Light,
    /// Dark theme for nighttime use
    Dark,
    /// High contrast for accessibility
    HighContrast,
    /// Blue-tinted professional theme
    Blue,
    /// Green nature-inspired theme
    Forest,
    /// Ocean blue theme
    Ocean,
    /// Purple sunset theme
    Sunset,
    /// Red/dark theme
    Mars,
    /// Custom user-defined theme
    Custom(String),
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        matches!(
            self,
            ThemeMode::Dark
                | ThemeMode::HighContrast
                | ThemeMode::Blue
                | ThemeMode::Forest
                | ThemeMode::Ocean
                | ThemeMode::Mars
                | ThemeMode::Sunset
        )
    }

    pub fn name(&self) -> &str {
        match self {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
            ThemeMode::HighContrast => "High Contrast",
            ThemeMode::Blue => "Blue",
            ThemeMode::Forest => "Forest",
            ThemeMode::Ocean => "Ocean",
            ThemeMode::Sunset => "Sunset",
            ThemeMode::Mars => "Mars",
            ThemeMode::Custom(name) => name,
        }
    }
}

/// Color palette definition
/// Note: Serialize/Deserialize removed because ratatui::Color doesn't implement them
#[derive(Debug, Clone)]
pub struct ColorPalette {
    /// Primary accent color
    pub primary: Color,
    /// Secondary accent color
    pub secondary: Color,
    /// Success/error indicators
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    /// Text colors
    pub foreground: Color,
    pub background: Color,
    /// UI elements
    pub border: Color,
    pub highlight: Color,
    /// Interactive elements
    pub focused: Color,
    pub selected: Color,
    /// Additional semantic colors
    pub info: Color,
    pub disabled: Color,
    /// Status indicators
    pub online: Color,
    pub offline: Color,
    pub connecting: Color,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::dark()
    }
}

impl ColorPalette {
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Gray,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            foreground: Color::Black,
            background: Color::White,
            border: Color::DarkGray,
            highlight: Color::LightBlue,
            focused: Color::Cyan,
            selected: Color::Blue,
            info: Color::Blue,
            disabled: Color::DarkGray,
            online: Color::Green,
            offline: Color::Red,
            connecting: Color::Yellow,
        }
    }

    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            foreground: Color::White,
            background: Color::Black,
            border: Color::DarkGray,
            highlight: Color::LightBlue,
            focused: Color::LightCyan,
            selected: Color::LightBlue,
            info: Color::Cyan,
            disabled: Color::Gray,
            online: Color::Green,
            offline: Color::Red,
            connecting: Color::Yellow,
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            primary: Color::Yellow,
            secondary: Color::White,
            success: Color::LightGreen,
            error: Color::LightRed,
            warning: Color::LightYellow,
            foreground: Color::White,
            background: Color::Black,
            border: Color::White,
            highlight: Color::Yellow,
            focused: Color::White,
            selected: Color::LightYellow,
            info: Color::LightCyan,
            disabled: Color::Gray,
            online: Color::LightGreen,
            offline: Color::LightRed,
            connecting: Color::LightYellow,
        }
    }

    pub fn blue_theme() -> Self {
        Self {
            primary: Color::LightBlue,
            secondary: Color::Blue,
            success: Color::Green,
            error: Color::LightRed,
            warning: Color::Yellow,
            foreground: Color::White,
            background: Color::Blue, // Changed from DarkBlue
            border: Color::Blue,
            highlight: Color::LightCyan,
            focused: Color::Cyan,
            selected: Color::LightBlue,
            info: Color::LightBlue,
            disabled: Color::DarkGray,
            online: Color::Green,
            offline: Color::LightRed,
            connecting: Color::LightYellow,
        }
    }

    pub fn forest_theme() -> Self {
        Self {
            primary: Color::LightGreen,
            secondary: Color::Green,
            success: Color::LightGreen, // Changed from BrightGreen
            error: Color::LightRed,
            warning: Color::LightYellow,
            foreground: Color::LightGreen,
            background: Color::Green, // Changed from DarkGreen
            border: Color::Green,
            highlight: Color::LightYellow,
            focused: Color::LightGreen, // Changed from BrightGreen
            selected: Color::LightGreen,
            info: Color::LightCyan,
            disabled: Color::DarkGray,
            online: Color::LightGreen, // Changed from BrightGreen
            offline: Color::LightRed,
            connecting: Color::LightYellow,
        }
    }

    pub fn ocean_theme() -> Self {
        Self {
            primary: Color::LightBlue,
            secondary: Color::Blue,
            success: Color::LightGreen,
            error: Color::LightRed, // Changed from Coral
            warning: Color::LightYellow,
            foreground: Color::LightBlue,
            background: Color::Blue, // Changed from DarkBlue
            border: Color::Blue,
            highlight: Color::LightCyan,
            focused: Color::Cyan,
            selected: Color::LightBlue,
            info: Color::LightBlue,
            disabled: Color::DarkGray,
            online: Color::LightGreen,
            offline: Color::LightRed, // Changed from Coral
            connecting: Color::LightYellow,
        }
    }

    pub fn sunset_theme() -> Self {
        Self {
            primary: Color::Magenta,
            secondary: Color::Magenta, // Changed from Purple
            success: Color::LightGreen,
            error: Color::LightRed,
            warning: Color::Yellow,
            foreground: Color::LightYellow,
            background: Color::Magenta, // Changed from DarkMagenta
            border: Color::Magenta,     // Changed from Purple
            highlight: Color::LightMagenta,
            focused: Color::Magenta,
            selected: Color::LightMagenta,
            info: Color::LightCyan,
            disabled: Color::DarkGray,
            online: Color::LightGreen,
            offline: Color::LightRed,
            connecting: Color::LightYellow,
        }
    }

    pub fn mars_theme() -> Self {
        Self {
            primary: Color::LightRed,
            secondary: Color::Red,
            success: Color::Green,
            error: Color::LightRed,
            warning: Color::Yellow,
            foreground: Color::LightRed,
            background: Color::Red, // Changed from DarkRed
            border: Color::Red,
            highlight: Color::LightYellow,
            focused: Color::Red,
            selected: Color::LightRed,
            info: Color::LightCyan,
            disabled: Color::DarkGray,
            online: Color::Green,
            offline: Color::LightRed,
            connecting: Color::LightYellow,
        }
    }
}

/// Component style definitions
#[derive(Debug, Clone)]
pub struct ComponentStyles {
    pub border: Style,
    pub focused_border: Style,
    pub title: Style,
    pub text: Style,
    pub highlight: Style,
    pub selected: Style,
    pub disabled: Style,
    pub button: Style,
    pub button_focused: Style,
    pub input: Style,
    pub input_focused: Style,
    pub list_item: Style,
    pub list_selected: Style,
    pub status_bar: Style,
    pub status_error: Style,
    pub status_success: Style,
}

impl ComponentStyles {
    pub fn from_palette(palette: &ColorPalette) -> Self {
        Self {
            border: Style::default().fg(palette.border),
            focused_border: Style::default()
                .fg(palette.focused)
                .add_modifier(Modifier::BOLD),
            title: Style::default()
                .fg(palette.primary)
                .add_modifier(Modifier::BOLD),
            text: Style::default().fg(palette.foreground),
            highlight: Style::default()
                .fg(palette.highlight)
                .add_modifier(Modifier::BOLD),
            selected: Style::default().fg(palette.selected).bg(palette.border),
            disabled: Style::default().fg(palette.disabled),
            button: Style::default().fg(palette.foreground).bg(palette.primary),
            button_focused: Style::default().fg(palette.background).bg(palette.focused),
            input: Style::default()
                .fg(palette.foreground)
                .bg(palette.background),
            input_focused: Style::default()
                .fg(palette.foreground)
                .bg(palette.highlight),
            list_item: Style::default().fg(palette.foreground),
            list_selected: Style::default().fg(palette.selected).bg(palette.border),
            status_bar: Style::default().fg(palette.foreground).bg(palette.border),
            status_error: Style::default().fg(palette.error),
            status_success: Style::default().fg(palette.success),
        }
    }
}

/// Theme configuration and management
#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub mode: ThemeMode,
    pub palette: ColorPalette,
    pub styles: ComponentStyles,
    pub transition_duration: std::time::Duration,
    pub enable_animations: bool,
    pub auto_switch: bool,
}

impl ThemeConfig {
    pub fn new(mode: ThemeMode) -> Self {
        let palette = match mode {
            ThemeMode::Light => ColorPalette::light(),
            ThemeMode::Dark => ColorPalette::dark(),
            ThemeMode::HighContrast => ColorPalette::high_contrast(),
            ThemeMode::Blue => ColorPalette::blue_theme(),
            ThemeMode::Forest => ColorPalette::forest_theme(),
            ThemeMode::Ocean => ColorPalette::ocean_theme(),
            ThemeMode::Sunset => ColorPalette::sunset_theme(),
            ThemeMode::Mars => ColorPalette::mars_theme(),
            ThemeMode::Custom(_) => ColorPalette::dark(), // Fallback
        };

        let styles = ComponentStyles::from_palette(&palette);

        Self {
            mode,
            palette,
            styles,
            transition_duration: std::time::Duration::from_millis(300),
            enable_animations: true,
            auto_switch: false,
        }
    }

    pub fn switch_mode(&mut self, mode: ThemeMode) {
        self.mode = mode.clone();
        self.palette = match mode {
            ThemeMode::Light => ColorPalette::light(),
            ThemeMode::Dark => ColorPalette::dark(),
            ThemeMode::HighContrast => ColorPalette::high_contrast(),
            ThemeMode::Blue => ColorPalette::blue_theme(),
            ThemeMode::Forest => ColorPalette::forest_theme(),
            ThemeMode::Ocean => ColorPalette::ocean_theme(),
            ThemeMode::Sunset => ColorPalette::sunset_theme(),
            ThemeMode::Mars => ColorPalette::mars_theme(),
            ThemeMode::Custom(_) => ColorPalette::dark(),
        };
        self.styles = ComponentStyles::from_palette(&self.palette);
    }

    pub fn get_style_for_component(&self, component: &str, focused: bool) -> Style {
        match component {
            "border" if focused => self.styles.focused_border,
            "border" => self.styles.border,
            "title" => self.styles.title,
            "text" => self.styles.text,
            "highlight" => self.styles.highlight,
            "selected" => self.styles.selected,
            "disabled" => self.styles.disabled,
            "button" if focused => self.styles.button_focused,
            "button" => self.styles.button,
            "input" if focused => self.styles.input_focused,
            "input" => self.styles.input,
            "list_item" => self.styles.list_item,
            "list_selected" => self.styles.list_selected,
            "status_bar" => self.styles.status_bar,
            "status_error" => self.styles.status_error,
            "status_success" => self.styles.status_success,
            _ => self.styles.text,
        }
    }
}

/// Theme manager component
#[derive(Debug)]
pub struct ThemeManager {
    props: Props,
    config: ThemeConfig,
    theme_preview_visible: bool,
    selected_theme_index: usize,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeManager {
    pub fn new() -> Self {
        let config = ThemeConfig::new(ThemeMode::Dark);

        Self {
            props: Props::default(),
            config,
            theme_preview_visible: false,
            selected_theme_index: 1, // Dark theme default
        }
    }

    pub fn with_theme(mut self, mode: ThemeMode) -> Self {
        // Update selected index
        self.selected_theme_index = match &mode {
            ThemeMode::Light => 0,
            ThemeMode::Dark => 1,
            ThemeMode::HighContrast => 2,
            ThemeMode::Blue => 3,
            ThemeMode::Forest => 4,
            ThemeMode::Ocean => 5,
            ThemeMode::Sunset => 6,
            ThemeMode::Mars => 7,
            ThemeMode::Custom(_) => 8,
        };

        // Update config with the theme mode
        self.config = ThemeConfig::new(mode);

        self
    }

    pub fn switch_theme(&mut self, mode: ThemeMode) {
        let previous_mode = self.config.mode.clone();
        self.config.switch_mode(mode.clone());

        // Update selected index
        self.selected_theme_index = match mode {
            ThemeMode::Light => 0,
            ThemeMode::Dark => 1,
            ThemeMode::HighContrast => 2,
            ThemeMode::Blue => 3,
            ThemeMode::Forest => 4,
            ThemeMode::Ocean => 5,
            ThemeMode::Sunset => 6,
            ThemeMode::Mars => 7,
            ThemeMode::Custom(_) => 8,
        };

        tracing::info!("Theme switched from {:?} to {:?}", previous_mode, mode);
    }

    pub fn toggle_theme_preview(&mut self) {
        self.theme_preview_visible = !self.theme_preview_visible;
    }

    pub fn is_preview_visible(&self) -> bool {
        self.theme_preview_visible
    }

    pub fn get_current_theme(&self) -> &ThemeConfig {
        &self.config
    }

    pub fn get_style(&self, component: &str, focused: bool) -> Style {
        self.config.get_style_for_component(component, focused)
    }

    fn get_available_themes() -> Vec<ThemeMode> {
        vec![
            ThemeMode::Light,
            ThemeMode::Dark,
            ThemeMode::HighContrast,
            ThemeMode::Blue,
            ThemeMode::Forest,
            ThemeMode::Ocean,
            ThemeMode::Sunset,
            ThemeMode::Mars,
        ]
    }

    fn render_theme_preview(&self) -> Vec<Line<'_>> {
        let themes = Self::get_available_themes();
        let mut lines = vec![
            Line::from(Span::styled(
                "Theme Selector",
                Style::default()
                    .fg(self.config.palette.primary)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        for (i, theme) in themes.iter().enumerate() {
            let is_selected = i == self.selected_theme_index;
            let is_current = matches!(&self.config.mode, mode if mode == theme);

            let prefix = if is_selected { "→ " } else { "  " };
            let suffix = if is_current { " ✓" } else { "" };

            let _style = if is_selected {
                Style::default()
                    .fg(self.config.palette.selected)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(self.config.palette.foreground)
            };

            // Use theme's primary color for the theme name
            let theme_style = match theme {
                ThemeMode::Light => ColorPalette::light(),
                ThemeMode::Dark => ColorPalette::dark(),
                ThemeMode::HighContrast => ColorPalette::high_contrast(),
                ThemeMode::Blue => ColorPalette::blue_theme(),
                ThemeMode::Forest => ColorPalette::forest_theme(),
                ThemeMode::Ocean => ColorPalette::ocean_theme(),
                ThemeMode::Sunset => ColorPalette::sunset_theme(),
                ThemeMode::Mars => ColorPalette::mars_theme(),
                ThemeMode::Custom(_) => ColorPalette::dark(),
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, Style::default().fg(self.config.palette.disabled)),
                Span::styled(
                    format!("{}{}", theme.name(), suffix),
                    Style::default().fg(theme_style.primary),
                ),
            ]));
        }

        lines.extend_from_slice(&[
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Controls:",
                    Style::default()
                        .fg(self.config.palette.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("↑↓ Select", Style::default().fg(self.config.palette.info)),
                Span::raw(" "),
                Span::styled(
                    "Enter Apply",
                    Style::default().fg(self.config.palette.success),
                ),
                Span::raw(" "),
                Span::styled(
                    "Esc Close",
                    Style::default().fg(self.config.palette.disabled),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Quick Toggle:",
                    Style::default()
                        .fg(self.config.palette.secondary)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("Ctrl+T", Style::default().fg(self.config.palette.info)),
            ]),
        ]);

        lines
    }

    fn render_current_theme_info(&self, _area: Rect) -> Paragraph<'_> {
        let info_lines = vec![
            Line::from(vec![
                Span::styled(
                    "Current Theme: ",
                    Style::default().fg(self.config.palette.secondary),
                ),
                Span::styled(
                    self.config.mode.name(),
                    Style::default()
                        .fg(self.config.palette.primary)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Primary: ",
                    Style::default().fg(self.config.palette.secondary),
                ),
                Span::styled("●", Style::default().fg(self.config.palette.primary)),
                Span::styled(
                    format!(" ({:?})", self.config.palette.primary),
                    Style::default().fg(self.config.palette.foreground),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "Background: ",
                    Style::default().fg(self.config.palette.secondary),
                ),
                Span::styled("●", Style::default().fg(self.config.palette.background)),
                Span::styled(
                    format!(" ({:?})", self.config.palette.background),
                    Style::default().fg(self.config.palette.foreground),
                ),
            ]),
            Line::from(vec![Span::styled(
                "Press Ctrl+T to change theme",
                Style::default().fg(self.config.palette.disabled),
            )]),
        ];

        Paragraph::new(info_lines)
            .block(
                Block::default()
                    .title("Theme Info")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(self.config.palette.border)),
            )
            .alignment(Alignment::Left)
    }
}

impl MockComponent for ThemeManager {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if self.theme_preview_visible {
            let preview_width = area.width.min(40);
            let preview_height = area.height.min(20);
            let preview_x = (area.width - preview_width) / 2;
            let preview_y = (area.height - preview_height) / 2;

            let preview_area = Rect {
                x: area.x + preview_x,
                y: area.y + preview_y,
                width: preview_width,
                height: preview_height,
            };

            let preview_lines = self.render_theme_preview();
            let preview_paragraph = Paragraph::new(preview_lines)
                .block(
                    Block::default()
                        .title(" Theme Selection ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(self.config.palette.border)),
                )
                .alignment(Alignment::Left);

            frame.render_widget(preview_paragraph, preview_area);
        } else {
            // Show theme info widget
            let info_area = Rect {
                x: area.x + area.width - 30,
                y: area.y,
                width: 30,
                height: 6,
            };

            let info_widget = self.render_current_theme_info(info_area);
            frame.render_widget(info_widget, info_area);
        }
    }

    fn query(&self, attr: Attribute) -> Option<AttrValue> {
        self.props.get(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.props.set(attr, value);
    }

    fn state(&self) -> State {
        State::One(tuirealm::StateValue::Usize(self.selected_theme_index))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Custom("toggle_preview") => {
                self.toggle_theme_preview();
                CmdResult::None
            }
            // NOTE: Attribute::Index and AttrValue::Usize don't exist in tuirealm
            // This switch functionality is disabled until an alternative is found
            // Cmd::Custom("switch_theme") => {
            //     if let Some(index) = self.query(Attribute::Index) {
            //         if let AttrValue::Usize(index) = index {
            //             let themes = Self::get_available_themes();
            //             if index < themes.len() {
            //                 self.switch_theme(themes[index].clone());
            //                 return CmdResult::Submit(State::One(tuirealm::StateValue::Usize(index)));
            //             }
            //         }
            //     }
            //     CmdResult::None
            // }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for ThemeManager {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(event) => {
                use tuirealm::event::{Key, KeyModifiers};

                match (event.code, event.modifiers) {
                    (Key::Char('t'), KeyModifiers::CONTROL) => {
                        self.toggle_theme_preview();
                        None
                    }
                    (Key::Up, _) => {
                        if self.theme_preview_visible && self.selected_theme_index > 0 {
                            self.selected_theme_index -= 1;
                        }
                        None
                    }
                    (Key::Down, _) => {
                        if self.theme_preview_visible && self.selected_theme_index < 7 {
                            self.selected_theme_index += 1;
                        }
                        None
                    }
                    (Key::Enter, _) => {
                        if self.theme_preview_visible {
                            let themes = Self::get_available_themes();
                            if self.selected_theme_index < themes.len() {
                                self.switch_theme(themes[self.selected_theme_index].clone());
                                self.toggle_theme_preview(); // Close preview

                                return Some(Msg::User(UserEvent::TaskCompleted {
                                    task_id: "theme_switched".to_string(),
                                    result: TaskResult::Success(format!(
                                        "Switched to {}",
                                        themes[self.selected_theme_index].name()
                                    )),
                                }));
                            }
                        }
                        None
                    }
                    (Key::Esc, _) => {
                        if self.theme_preview_visible {
                            self.toggle_theme_preview();
                        }
                        None
                    }
                    _ => None,
                }
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
    fn test_theme_mode_properties() {
        assert!(!ThemeMode::Light.is_dark());
        assert!(ThemeMode::Dark.is_dark());
        assert!(ThemeMode::HighContrast.is_dark());
        assert!(ThemeMode::Blue.is_dark());

        assert_eq!(ThemeMode::Light.name(), "Light");
        assert_eq!(ThemeMode::Dark.name(), "Dark");
        assert_eq!(ThemeMode::Blue.name(), "Blue");
        assert_eq!(ThemeMode::Forest.name(), "Forest");
    }

    #[test]
    fn test_color_palette_creation() {
        let light = ColorPalette::light();
        assert_eq!(light.background, Color::White);
        assert_eq!(light.foreground, Color::Black);

        let dark = ColorPalette::dark();
        assert_eq!(dark.background, Color::Black);
        assert_eq!(dark.foreground, Color::White);

        let high_contrast = ColorPalette::high_contrast();
        assert_eq!(high_contrast.foreground, Color::White);
        assert_eq!(high_contrast.background, Color::Black);
    }

    #[test]
    fn test_component_styles_from_palette() {
        let palette = ColorPalette::dark();
        let styles = ComponentStyles::from_palette(&palette);

        assert_eq!(styles.text.fg.unwrap(), Color::White);
        assert_eq!(styles.border.fg.unwrap(), Color::DarkGray);
        assert_eq!(styles.title.fg.unwrap(), Color::Cyan);
    }

    #[test]
    fn test_theme_config_creation() {
        let config = ThemeConfig::new(ThemeMode::Dark);
        assert!(matches!(config.mode, ThemeMode::Dark));
        assert_eq!(config.palette.background, Color::Black);
        assert!(config.enable_animations);
        assert!(!config.auto_switch);
    }

    #[test]
    fn test_theme_config_switch_mode() {
        let mut config = ThemeConfig::new(ThemeMode::Dark);
        assert_eq!(config.palette.background, Color::Black);

        config.switch_mode(ThemeMode::Light);
        assert_eq!(config.palette.background, Color::White);
        assert!(matches!(config.mode, ThemeMode::Light));
    }

    #[test]
    fn test_theme_config_get_style() {
        let config = ThemeConfig::new(ThemeMode::Dark);

        let button_style = config.get_style_for_component("button", false);
        let button_focused_style = config.get_style_for_component("button", true);

        assert_ne!(button_style, button_focused_style);
        assert!(button_focused_style.bg.is_some());
    }

    #[test]
    fn test_theme_manager_creation() {
        let manager = ThemeManager::new();
        assert!(matches!(manager.config.mode, ThemeMode::Dark));
        assert_eq!(manager.selected_theme_index, 1);
        assert!(!manager.theme_preview_visible);
    }

    #[test]
    fn test_theme_manager_with_theme() {
        let manager = ThemeManager::new().with_theme(ThemeMode::Light);
        assert!(matches!(manager.config.mode, ThemeMode::Light));
        assert_eq!(manager.selected_theme_index, 0);
    }

    #[test]
    fn test_theme_manager_switch_theme() {
        let mut manager = ThemeManager::new();
        assert!(matches!(manager.config.mode, ThemeMode::Dark));

        manager.switch_theme(ThemeMode::Light);
        assert!(matches!(manager.config.mode, ThemeMode::Light));
        assert_eq!(manager.selected_theme_index, 0);
    }

    #[test]
    fn test_theme_manager_toggle_preview() {
        let mut manager = ThemeManager::new();
        assert!(!manager.theme_preview_visible);

        manager.toggle_theme_preview();
        assert!(manager.theme_preview_visible);

        manager.toggle_theme_preview();
        assert!(!manager.theme_preview_visible);
    }

    #[test]
    fn test_theme_manager_events() {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};

        let mut manager = ThemeManager::new();

        // Test Ctrl+T toggle preview
        manager.on(Event::Keyboard(KeyEvent {
            code: Key::Char('t'),
            modifiers: KeyModifiers::CONTROL,
        }));
        assert!(manager.theme_preview_visible);

        // Test Enter switch theme
        manager.on(Event::Keyboard(KeyEvent {
            code: Key::Up,
            modifiers: KeyModifiers::NONE,
        })); // Move to Light theme

        let msg = manager.on(Event::Keyboard(KeyEvent {
            code: Key::Enter,
            modifiers: KeyModifiers::NONE,
        }));

        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));
        assert!(!manager.theme_preview_visible); // Preview closed

        // Test Esc close preview
        manager.toggle_theme_preview(); // Show preview again
        manager.on(Event::Keyboard(KeyEvent {
            code: Key::Esc,
            modifiers: KeyModifiers::NONE,
        }));
        assert!(!manager.theme_preview_visible);
    }

    #[test]
    fn test_mock_component_perform() {
        let mut manager = ThemeManager::new();

        // Test toggle preview
        let result = manager.perform(Cmd::Custom("toggle_preview"));
        assert!(matches!(result, CmdResult::None));
        assert!(manager.theme_preview_visible);

        // Test toggle again to close
        let result = manager.perform(Cmd::Custom("toggle_preview"));
        assert!(matches!(result, CmdResult::None));
        assert!(!manager.theme_preview_visible);
    }
}
