//! Error recovery and user-friendly error messages component
//!
//! Provides comprehensive error handling with:
//! - User-friendly error messages
//! - Recovery suggestions
//! - Contextual help
//! - Error history and analytics

use crate::messages::{Msg, UserEvent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tuirealm::{
    Component, Frame, MockComponent, State,
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    ratatui::{
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    },
};

/// Helper function for serde default
fn instant_now() -> Instant {
    Instant::now()
}

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity (informational)
    Low,
    /// Medium severity (warning)
    Medium,
    /// High severity (important)
    High,
    /// Critical (system failure)
    Critical,
}

impl ErrorSeverity {
    pub fn color(&self) -> Color {
        match self {
            ErrorSeverity::Low => Color::Blue,
            ErrorSeverity::Medium => Color::Yellow,
            ErrorSeverity::High => Color::Red,
            ErrorSeverity::Critical => Color::Magenta,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ErrorSeverity::Low => "ℹ",
            ErrorSeverity::Medium => "⚠",
            ErrorSeverity::High => "⚡",
            ErrorSeverity::Critical => "☠",
        }
    }
}

/// Error categories for better organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network-related errors
    Network,
    /// Authentication errors
    Authentication,
    /// File system errors
    FileSystem,
    /// Data validation errors
    Validation,
    /// Configuration errors
    Configuration,
    /// Runtime errors
    Runtime,
    /// User input errors
    UserInput,
    /// System errors
    System,
    /// Uncategorized errors
    Other,
}

impl ErrorCategory {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorCategory::Network => "Network",
            ErrorCategory::Authentication => "Authentication",
            ErrorCategory::FileSystem => "FileSystem",
            ErrorCategory::Validation => "Validation",
            ErrorCategory::Configuration => "Configuration",
            ErrorCategory::Runtime => "Runtime",
            ErrorCategory::UserInput => "UserInput",
            ErrorCategory::System => "System",
            ErrorCategory::Other => "Other",
        }
    }

    pub fn recovery_suggestions(&self) -> Vec<&'static str> {
        match self {
            ErrorCategory::Network => vec![
                "Check your internet connection",
                "Try reconnecting to the network",
                "Verify the service status",
                "Restart the application",
            ],
            ErrorCategory::Authentication => vec![
                "Verify your credentials",
                "Clear stored passwords",
                "Try identity re-initialization",
                "Contact support if needed",
            ],
            ErrorCategory::FileSystem => vec![
                "Check file permissions",
                "Ensure disk space is available",
                "Verify the file path exists",
                "Try a different directory",
            ],
            ErrorCategory::Validation => vec![
                "Check input format",
                "Review required fields",
                "Remove special characters",
                "Contact support for validation rules",
            ],
            ErrorCategory::Configuration => vec![
                "Reset to default settings",
                "Check configuration file syntax",
                "Verify environment variables",
                "Review documentation for correct settings",
            ],
            ErrorCategory::Runtime => vec![
                "Restart the application",
                "Check system resources",
                "Update to the latest version",
                "Report the issue with system info",
            ],
            ErrorCategory::UserInput => vec![
                "Review the input format",
                "Check the help documentation",
                "Try a different approach",
                "Use suggested commands",
            ],
            ErrorCategory::System => vec![
                "Restart your computer",
                "Check system logs",
                "Update system dependencies",
                "Contact system administrator",
            ],
            ErrorCategory::Other => vec![
                "Try again later",
                "Restart the application",
                "Check system resources",
                "Contact support",
            ],
        }
    }
}

/// Individual error entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEntry {
    /// Unique error ID
    pub id: String,
    /// Error message
    pub message: String,
    /// Technical details (for logs/debug)
    pub details: String,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Timestamp when error occurred
    #[serde(skip, default = "instant_now")]
    pub timestamp: Instant,
    /// Whether the error is resolved
    pub resolved: bool,
    /// Recovery actions taken
    pub recovery_actions: Vec<String>,
    /// Whether user has been notified
    pub notified: bool,
}

impl ErrorEntry {
    pub fn new(
        message: String,
        details: String,
        severity: ErrorSeverity,
        category: ErrorCategory,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            message,
            details,
            severity,
            category,
            timestamp: Instant::now(),
            resolved: false,
            recovery_actions: Vec::new(),
            notified: false,
        }
    }

    pub fn mark_resolved(&mut self, action: &str) {
        self.resolved = true;
        self.recovery_actions.push(action.to_string());
    }

    pub fn mark_notified(&mut self) {
        self.notified = true;
    }

    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }

    pub fn format_for_display(&self) -> String {
        format!(
            "{} {} ({})",
            self.severity.icon(),
            self.message,
            self.category.name()
        )
    }
}

/// Error recovery state
#[derive(Debug, Clone)]
pub struct ErrorRecoveryState {
    /// Current active error
    pub active_error: Option<ErrorEntry>,
    /// Error history (max 100 entries)
    pub error_history: VecDeque<ErrorEntry>,
    /// Unresolved errors count
    pub unresolved_count: usize,
    /// Last error timestamp
    pub last_error_time: Option<Instant>,
    /// Error statistics
    pub error_stats: ErrorStats,
}

#[derive(Debug, Clone, Default)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub unresolved_count: usize,
    pub network_errors: usize,
    pub auth_errors: usize,
    pub file_errors: usize,
    pub critical_errors: usize,
    pub resolved_errors: usize,
    pub avg_resolution_time_secs: f64,
}

impl Default for ErrorRecoveryState {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecoveryState {
    pub fn new() -> Self {
        Self {
            active_error: None,
            error_history: VecDeque::new(),
            unresolved_count: 0,
            last_error_time: None,
            error_stats: ErrorStats::default(),
        }
    }

    pub fn add_error(&mut self, error: ErrorEntry) {
        // Update statistics
        self.error_stats.total_errors += 1;
        self.error_stats.unresolved_count += 1;

        match error.category {
            ErrorCategory::Network => self.error_stats.network_errors += 1,
            ErrorCategory::Authentication => self.error_stats.auth_errors += 1,
            ErrorCategory::FileSystem => self.error_stats.file_errors += 1,
            _ => {}
        }

        if matches!(error.severity, ErrorSeverity::Critical) {
            self.error_stats.critical_errors += 1;
        }

        // Set as active error if critical
        if matches!(error.severity, ErrorSeverity::Critical) {
            self.active_error = Some(error.clone());
        }

        // Add to history
        self.error_history.push_back(error);

        // Keep history limited
        while self.error_history.len() > 100 {
            self.error_history.pop_front();
        }

        self.last_error_time = Some(Instant::now());
        self.unresolved_count = self.error_history.iter().filter(|e| !e.resolved).count();
    }

    pub fn resolve_error(&mut self, error_id: &str, action: &str) -> bool {
        for error in &mut self.error_history {
            if error.id == error_id {
                error.mark_resolved(action);
                self.error_stats.resolved_errors += 1;

                // Recalculate unresolved count
                self.unresolved_count = self.error_history.iter().filter(|e| !e.resolved).count();

                // Clear active error if this was it
                if let Some(ref active) = self.active_error
                    && active.id == error_id
                {
                    self.active_error = None;
                }

                return true;
            }
        }
        false
    }

    pub fn get_recent_errors(&self, count: usize) -> Vec<&ErrorEntry> {
        self.error_history.iter().rev().take(count).collect()
    }

    pub fn get_errors_by_category(&self, category: ErrorCategory) -> Vec<&ErrorEntry> {
        self.error_history
            .iter()
            .filter(|e| e.category == category)
            .collect()
    }

    pub fn get_unresolved_errors(&self) -> Vec<&ErrorEntry> {
        self.error_history.iter().filter(|e| !e.resolved).collect()
    }
}

/// Error recovery component
#[derive(Debug)]
pub struct ErrorRecovery {
    props: Props,
    state: ErrorRecoveryState,
    visible: bool,
    selected_suggestion: usize,
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorRecovery {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            state: ErrorRecoveryState::new(),
            visible: false,
            selected_suggestion: 0,
        }
    }

    pub fn show_error(&mut self, error: ErrorEntry) {
        self.state.add_error(error.clone());
        // Always set as active error when explicitly showing
        self.state.active_error = Some(error);
        self.visible = true;
        self.selected_suggestion = 0;
    }

    pub fn show_error_message(
        &mut self,
        message: String,
        details: String,
        severity: ErrorSeverity,
        category: ErrorCategory,
    ) {
        let error = ErrorEntry::new(message, details, severity, category);
        self.show_error(error);
    }

    pub fn dismiss_error(&mut self) {
        if let Some(ref mut error) = self.state.active_error {
            error.mark_notified();
        }
        self.visible = false;
    }

    pub fn apply_suggestion(&mut self, suggestion_index: usize) -> bool {
        let (error_id, action) = if let Some(ref error) = self.state.active_error {
            if let Some(suggestions) = self.get_recovery_suggestions() {
                if suggestion_index < suggestions.len() {
                    let action = suggestions[suggestion_index];
                    (error.id.clone(), action)
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } else {
            return false;
        };

        self.state.resolve_error(&error_id, action);
        if self.state.unresolved_count == 0 {
            self.visible = false;
        }
        true
    }

    pub fn get_recovery_suggestions(&self) -> Option<Vec<&'static str>> {
        self.state
            .active_error
            .as_ref()
            .map(|error| error.category.recovery_suggestions())
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    fn format_error_header(&self, error: &ErrorEntry) -> Vec<Line<'_>> {
        vec![
            Line::from(vec![
                Span::styled(
                    error.severity.icon(),
                    Style::default()
                        .fg(error.severity.color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(
                    "Error Recovery",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Message: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    error.message.clone(),
                    Style::default()
                        .fg(error.severity.color())
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Category: ", Style::default().fg(Color::Gray)),
                Span::styled(error.category.name(), Style::default().fg(Color::Blue)),
            ]),
            Line::from(vec![
                Span::styled("Severity: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:?}", error.severity),
                    Style::default().fg(error.severity.color()),
                ),
            ]),
            Line::from(""),
        ]
    }

    fn format_suggestions(&self, error: &ErrorEntry) -> Vec<Line<'_>> {
        let mut lines = vec![
            Line::from(Span::styled(
                "Recovery Suggestions:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        let suggestions = error.category.recovery_suggestions();
        if !suggestions.is_empty() {
            for (i, suggestion) in suggestions.iter().enumerate() {
                let is_selected = i == self.selected_suggestion;
                let prefix = if is_selected { "→ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let numbered = format!("{}{}. ", prefix, i + 1);

                lines.push(Line::from(vec![
                    Span::styled(numbered, Style::default().fg(Color::Gray)),
                    Span::styled(*suggestion, style),
                ]));
            }
        }

        lines.extend_from_slice(&[
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Actions:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("Enter=Dismiss", Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled("↑↓=Select", Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled("Esc=Cancel", Style::default().fg(Color::Gray)),
            ]),
        ]);

        lines
    }
}

impl MockComponent for ErrorRecovery {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if !self.is_visible() {
            return;
        }

        let Some(error) = self.state.active_error.as_ref() else {
            return;
        };

        // Create overlay area for error dialog
        let dialog_width = area.width.min(80);
        let dialog_height = area.height.min(25);
        let dialog_x = (area.width - dialog_width) / 2;
        let dialog_y = (area.height - dialog_height) / 2;

        let dialog_area = Rect {
            x: area.x + dialog_x,
            y: area.y + dialog_y,
            width: dialog_width,
            height: dialog_height,
        };

        // Split dialog into header, content, and actions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(6), // Error header
                Constraint::Min(10),   // Suggestions
                Constraint::Length(1), // Instructions
            ])
            .split(dialog_area);

        // Render dialog background
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(error.severity.color()))
            .title(" Error Recovery ")
            .title_style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(block, dialog_area);

        // Render error header
        let header_lines = self.format_error_header(error);
        let header_paragraph = Paragraph::new(header_lines).alignment(Alignment::Left);

        frame.render_widget(header_paragraph, chunks[0]);

        // Render recovery suggestions
        let suggestion_lines = self.format_suggestions(error);
        let suggestions_paragraph = Paragraph::new(suggestion_lines).alignment(Alignment::Left);

        frame.render_widget(suggestions_paragraph, chunks[1]);

        // Render instructions
        let instructions = Paragraph::new(Line::from(Span::styled(
            "Press Enter to dismiss, ↑↓ to select suggestions, or Esc to cancel",
            Style::default().fg(Color::Gray),
        )))
        .alignment(Alignment::Center);

        frame.render_widget(instructions, chunks[2]);
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
            Cmd::Custom("show_error") => {
                // This would be triggered from external code
                CmdResult::None
            }
            Cmd::Custom("dismiss") => {
                self.dismiss_error();
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for ErrorRecovery {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(event) => {
                use tuirealm::event::Key;

                match event.code {
                    Key::Enter => {
                        if self.is_visible() {
                            if self.selected_suggestion == 0 {
                                self.dismiss_error();
                                Some(Msg::User(UserEvent::TaskCompleted {
                                    task_id: "error_dismissed".to_string(),
                                    result: TaskResult::Success(
                                        "Error dismissed by user".to_string(),
                                    ),
                                }))
                            } else if self.apply_suggestion(self.selected_suggestion) {
                                Some(Msg::User(UserEvent::TaskCompleted {
                                    task_id: "error_recovery_applied".to_string(),
                                    result: TaskResult::Success(
                                        "Recovery action applied".to_string(),
                                    ),
                                }))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Key::Esc => {
                        if self.is_visible() {
                            self.dismiss_error();
                            Some(Msg::User(UserEvent::TaskCompleted {
                                task_id: "error_cancelled".to_string(),
                                result: TaskResult::Success("Error dialog cancelled".to_string()),
                            }))
                        } else {
                            None
                        }
                    }
                    Key::Up => {
                        if self.is_visible() && self.selected_suggestion > 0 {
                            self.selected_suggestion -= 1;
                        }
                        None
                    }
                    Key::Down => {
                        if self.is_visible()
                            && let Some(suggestions) = self.get_recovery_suggestions()
                            && self.selected_suggestion < suggestions.len()
                        {
                            self.selected_suggestion += 1;
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
    fn test_error_entry_creation() {
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );

        assert_eq!(error.message, "Test error");
        assert_eq!(error.severity, ErrorSeverity::Medium);
        assert_eq!(error.category, ErrorCategory::Network);
        assert!(!error.resolved);
        assert!(!error.notified);
        assert!(!error.id.is_empty());
    }

    #[test]
    fn test_error_entry_mark_resolved() {
        let mut error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );

        error.mark_resolved("Fixed it");
        assert!(error.resolved);
        assert_eq!(error.recovery_actions.len(), 1);
        assert_eq!(error.recovery_actions[0], "Fixed it");
    }

    #[test]
    fn test_error_severity_color() {
        assert_eq!(ErrorSeverity::Low.color(), Color::Blue);
        assert_eq!(ErrorSeverity::Medium.color(), Color::Yellow);
        assert_eq!(ErrorSeverity::High.color(), Color::Red);
        assert_eq!(ErrorSeverity::Critical.color(), Color::Magenta);
    }

    #[test]
    fn test_error_category_recoveries() {
        let network_suggestions = ErrorCategory::Network.recovery_suggestions();
        assert!(!network_suggestions.is_empty());
        assert!(
            network_suggestions
                .iter()
                .any(|s| s.contains("internet connection"))
        );
    }

    #[test]
    fn test_error_recovery_state_creation() {
        let state = ErrorRecoveryState::new();
        assert!(state.active_error.is_none());
        assert!(state.error_history.is_empty());
        assert_eq!(state.unresolved_count, 0);
    }

    #[test]
    fn test_error_recovery_state_add_error() {
        let mut state = ErrorRecoveryState::new();
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );

        state.add_error(error.clone());

        assert_eq!(state.error_history.len(), 1);
        assert_eq!(state.unresolved_count, 1);
        assert!(state.last_error_time.is_some());
        assert_eq!(state.error_stats.total_errors, 1);
        assert_eq!(state.error_stats.network_errors, 1);
    }

    #[test]
    fn test_error_recovery_state_resolve_error() {
        let mut state = ErrorRecoveryState::new();
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );

        state.add_error(error.clone());
        let resolved = state.resolve_error(&error.id, "Fixed it");

        assert!(resolved);
        assert_eq!(state.unresolved_count, 0);
        assert_eq!(state.error_stats.resolved_errors, 1);
    }

    #[test]
    fn test_error_recovery_creation() {
        let recovery = ErrorRecovery::new();
        assert!(!recovery.is_visible());
        assert_eq!(recovery.selected_suggestion, 0);
    }

    #[test]
    fn test_error_recovery_show_error() {
        let mut recovery = ErrorRecovery::new();
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );

        recovery.show_error(error);
        assert!(recovery.is_visible());
        assert!(recovery.state.active_error.is_some());
    }

    #[test]
    fn test_error_recovery_apply_suggestion() {
        let mut recovery = ErrorRecovery::new();
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );

        recovery.show_error(error);
        let applied = recovery.apply_suggestion(0);
        assert!(applied);
        assert!(!recovery.is_visible());
    }

    #[test]
    fn test_component_events() {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};

        let mut recovery = ErrorRecovery::new();
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::Medium,
            ErrorCategory::Network,
        );
        recovery.show_error(error.clone());

        // Test Enter dismiss
        let msg = recovery.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));
        assert!(!recovery.is_visible());

        // Test Esc cancel
        recovery.show_error(error);
        let msg = recovery.on(Event::Keyboard(KeyEvent::new(Key::Esc, KeyModifiers::NONE)));
        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));
        assert!(!recovery.is_visible());
    }

    #[test]
    fn test_format_error_header() {
        let recovery = ErrorRecovery::new();
        let error = ErrorEntry::new(
            "Test error".to_string(),
            "Test details".to_string(),
            ErrorSeverity::High,
            ErrorCategory::Network,
        );

        let lines = recovery.format_error_header(&error);
        assert!(!lines.is_empty());
        assert_eq!(lines.len(), 6); // Icon + title + blank + message + category + severity + blank
    }
}
