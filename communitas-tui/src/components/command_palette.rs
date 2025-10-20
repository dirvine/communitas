//! Command Palette Component - Fuzzy search for commands and actions
//!
//! Provides a searchable command palette similar to VS Code's Command Palette (Ctrl+K):
//! - Fuzzy search with intelligent scoring
//! - Recent command prioritization
//! - Command categories
//! - Keyboard shortcuts display
//! - Decoupled action execution (returns command ID)
//!
//! Phase 5a: Core Structure & Search
//! Phase 5b: UI & Interaction (future)
//! Phase 5c: Advanced Features (future)

use std::collections::HashMap;

/// A command that can be executed via the palette
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// Unique identifier for this command
    pub id: String,
    /// Display name shown in the palette
    pub name: String,
    /// Descriptive help text
    pub description: String,
    /// Category for grouping (e.g., "File", "Edit", "View")
    pub category: String,
    /// Keyboard shortcuts (e.g., ["Ctrl+S", "⌘+S"])
    pub shortcuts: Vec<String>,
}

impl Command {
    /// Create a new command
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            category: category.into(),
            shortcuts: Vec::new(),
        }
    }

    /// Add a keyboard shortcut
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcuts.push(shortcut.into());
        self
    }

    /// Add multiple keyboard shortcuts
    pub fn with_shortcuts(mut self, shortcuts: Vec<String>) -> Self {
        self.shortcuts.extend(shortcuts);
        self
    }
}

/// Command Palette component for fuzzy command search
#[derive(Debug)]
pub struct CommandPalette {
    /// All available commands
    commands: Vec<Command>,
    /// Current search query
    query: String,
    /// Filtered results with scores: (command_index, score)
    filtered_results: Vec<(usize, f32)>,
    /// Currently selected result index
    selected_index: usize,
    /// Whether the palette is visible
    visible: bool,
    /// Recently executed command IDs (most recent first)
    recent_command_ids: Vec<String>,
    /// Maximum number of recent commands to track
    max_recent: usize,
    /// Categories for grouping
    categories: HashMap<String, Vec<usize>>, // category -> command indices
}

impl CommandPalette {
    /// Create a new command palette
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            query: String::new(),
            filtered_results: Vec::new(),
            selected_index: 0,
            visible: false,
            recent_command_ids: Vec::new(),
            max_recent: 10,
            categories: HashMap::new(),
        }
    }

    /// Create a palette with initial commands
    pub fn with_commands(commands: Vec<Command>) -> Self {
        let mut palette = Self::new();
        palette.set_commands(commands);
        palette
    }

    /// Set the available commands
    pub fn set_commands(&mut self, commands: Vec<Command>) {
        self.commands = commands;
        self.rebuild_categories();
        self.refresh_results();
    }

    /// Add a command to the palette
    pub fn add_command(&mut self, command: Command) {
        self.commands.push(command);
        self.rebuild_categories();
        self.refresh_results();
    }

    /// Rebuild category index
    fn rebuild_categories(&mut self) {
        self.categories.clear();
        for (idx, command) in self.commands.iter().enumerate() {
            self.categories
                .entry(command.category.clone())
                .or_default()
                .push(idx);
        }
    }

    /// Get all commands
    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    /// Get the current query
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Set the search query and refresh results
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.selected_index = 0;
        self.refresh_results();
    }

    /// Clear the query
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.selected_index = 0;
        self.refresh_results();
    }

    /// Show the palette
    pub fn show(&mut self) {
        self.visible = true;
    }

    /// Hide the palette
    pub fn hide(&mut self) {
        self.visible = false;
    }

    /// Toggle visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Check if the palette is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Get the filtered results (command indices and scores)
    pub fn results(&self) -> &[(usize, f32)] {
        &self.filtered_results
    }

    /// Get the number of results
    pub fn result_count(&self) -> usize {
        self.filtered_results.len()
    }

    /// Get the selected result index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get the selected command (if any)
    pub fn selected_command(&self) -> Option<&Command> {
        if self.filtered_results.is_empty() {
            return None;
        }
        let (cmd_idx, _) = self.filtered_results.get(self.selected_index)?;
        self.commands.get(*cmd_idx)
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if !self.filtered_results.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.filtered_results.is_empty()
            && self.selected_index + 1 < self.filtered_results.len()
        {
            self.selected_index += 1;
        }
    }

    /// Execute the selected command (returns command ID)
    pub fn execute_selected(&mut self) -> Option<String> {
        if let Some(command) = self.selected_command() {
            let id = command.id.clone();
            self.add_to_recent(&id);
            self.hide();
            self.clear_query();
            Some(id)
        } else {
            None
        }
    }

    /// Add a command to recent history
    fn add_to_recent(&mut self, command_id: &str) {
        // Remove if already present
        self.recent_command_ids.retain(|id| id != command_id);

        // Add to front
        self.recent_command_ids.insert(0, command_id.to_string());

        // Trim to max size
        if self.recent_command_ids.len() > self.max_recent {
            self.recent_command_ids.truncate(self.max_recent);
        }

        // Refresh results to apply new recency scores
        self.refresh_results();
    }

    /// Check if a command is recent
    fn is_recent(&self, command_id: &str) -> bool {
        self.recent_command_ids.contains(&command_id.to_string())
    }

    /// Get recent commands
    pub fn recent_commands(&self) -> &[String] {
        &self.recent_command_ids
    }

    // ===== FUZZY SEARCH =====

    /// Refresh filtered results based on current query
    fn refresh_results(&mut self) {
        if self.query.is_empty() {
            // Show all commands, sorted by category and recent
            self.filtered_results = self
                .commands
                .iter()
                .enumerate()
                .map(|(idx, cmd)| {
                    let score = if self.is_recent(&cmd.id) { 20.0 } else { 0.0 };
                    (idx, score)
                })
                .collect();
        } else {
            // Fuzzy search
            self.filtered_results = self.fuzzy_search(&self.query);
        }

        // Sort by score (descending)
        self.filtered_results
            .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Reset selection
        if self.selected_index >= self.filtered_results.len() {
            self.selected_index = 0;
        }
    }

    /// Perform fuzzy search and return scored results
    fn fuzzy_search(&self, query: &str) -> Vec<(usize, f32)> {
        let query_lower = query.to_lowercase();

        self.commands
            .iter()
            .enumerate()
            .filter_map(|(idx, cmd)| {
                let score = self.calculate_match_score(&query_lower, cmd);
                if score > 0.0 {
                    Some((idx, score))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Calculate match score for a command
    fn calculate_match_score(&self, query: &str, command: &Command) -> f32 {
        let name_lower = command.name.to_lowercase();
        let desc_lower = command.description.to_lowercase();

        let mut score = 0.0;

        // Exact match (very high score)
        if name_lower == query {
            score += 100.0;
        }
        // Prefix match (high score)
        else if name_lower.starts_with(query) {
            score += 90.0;
        }
        // Substring match (good score)
        else if name_lower.contains(query) {
            score += 70.0;
        }
        // Acronym match (e.g., "ocn" matches "Open Channel Names")
        else if self.matches_acronym(query, &name_lower) {
            score += 60.0;
        }
        // Fuzzy character match (lower score based on proximity)
        else if let Some(fuzzy_score) = self.fuzzy_character_match(query, &name_lower) {
            score += fuzzy_score;
        }

        // Also check description (lower weight)
        if desc_lower.contains(query) {
            score += 20.0;
        }

        // Bonus for recent commands
        if self.is_recent(&command.id) {
            score += 20.0;
        }

        score
    }

    /// Check if query matches as an acronym
    fn matches_acronym(&self, query: &str, text: &str) -> bool {
        let query_chars: Vec<char> = query.chars().collect();
        let mut query_idx = 0;

        for word in text.split_whitespace() {
            if query_idx >= query_chars.len() {
                break;
            }
            if let Some(first_char) = word.chars().next()
                && first_char.to_lowercase().to_string()
                    == query_chars[query_idx].to_lowercase().to_string()
            {
                query_idx += 1;
            }
        }

        query_idx == query_chars.len()
    }

    /// Fuzzy character-by-character matching
    fn fuzzy_character_match(&self, query: &str, text: &str) -> Option<f32> {
        let query_chars: Vec<char> = query.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();

        let mut query_idx = 0;
        let mut last_match_pos = 0;
        let mut total_distance = 0;

        for (pos, ch) in text_chars.iter().enumerate() {
            if query_idx >= query_chars.len() {
                break;
            }

            if ch.to_lowercase().to_string() == query_chars[query_idx].to_lowercase().to_string() {
                total_distance += pos.saturating_sub(last_match_pos);
                last_match_pos = pos;
                query_idx += 1;
            }
        }

        if query_idx == query_chars.len() {
            // All characters matched - score based on proximity
            let proximity_score = 50.0 / (1.0 + total_distance as f32 / 10.0);
            Some(proximity_score)
        } else {
            None
        }
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

// ===== PHASE 5b: UI & INTERACTION =====

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::{Component, Event, MockComponent, State, StateValue};

impl MockComponent for CommandPalette {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::ratatui::layout::Rect) {
        use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
        use tuirealm::ratatui::style::{Color, Modifier, Style};
        use tuirealm::ratatui::text::{Line, Span};
        use tuirealm::ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};

        if !self.visible {
            return; // Don't render if hidden
        }

        // Split area: input at top, results below
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Input field
                Constraint::Min(0),    // Results list
            ])
            .split(area);

        // Render search input
        let input_text = format!("> {}", self.query);
        let input = Paragraph::new(input_text)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Command Palette (Ctrl+K)")
                    .style(Style::default().fg(Color::Cyan)),
            );
        frame.render_widget(input, chunks[0]);

        // Render results
        let items: Vec<ListItem> = self
            .filtered_results
            .iter()
            .enumerate()
            .map(|(idx, (cmd_idx, score))| {
                let command = &self.commands[*cmd_idx];
                let is_selected = idx == self.selected_index;

                // Build display text
                let mut spans = Vec::new();

                // Category badge
                spans.push(Span::styled(
                    format!("[{}] ", command.category),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ));

                // Command name (highlighted if selected)
                let name_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                spans.push(Span::styled(command.name.clone(), name_style));

                // Shortcuts (if any)
                if !command.shortcuts.is_empty() {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        format!("({})", command.shortcuts[0]),
                        Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
                    ));
                }

                // Description
                spans.push(Span::raw(" - "));
                spans.push(Span::styled(
                    command.description.clone(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::DIM),
                ));

                // Score (for debugging - can be removed)
                if cfg!(debug_assertions) {
                    spans.push(Span::styled(
                        format!(" [{:.1}]", score),
                        Style::default().fg(Color::DarkGray),
                    ));
                }

                let line = Line::from(spans);
                ListItem::new(line)
            })
            .collect();

        let results_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("{} results", self.filtered_results.len()))
            .style(Style::default().fg(Color::White));

        let list = List::new(items).block(results_block);

        frame.render_widget(list, chunks[1]);
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {
        // Not used for this component
    }

    fn state(&self) -> State {
        if let Some(cmd) = self.selected_command() {
            State::One(StateValue::String(cmd.id.clone()))
        } else {
            State::None
        }
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(tuirealm::command::Direction::Up) => {
                self.select_previous();
                CmdResult::Changed(self.state())
            }
            Cmd::Move(tuirealm::command::Direction::Down) => {
                self.select_next();
                CmdResult::Changed(self.state())
            }
            Cmd::Submit => {
                if let Some(id) = self.execute_selected() {
                    CmdResult::Submit(State::One(StateValue::String(id)))
                } else {
                    CmdResult::None
                }
            }
            Cmd::Cancel => {
                self.hide();
                self.clear_query();
                CmdResult::Changed(State::None)
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<tuirealm::NoUserEvent, tuirealm::NoUserEvent> for CommandPalette {
    fn on(&mut self, ev: Event<tuirealm::NoUserEvent>) -> Option<tuirealm::NoUserEvent> {
        match ev {
            // Navigation
            Event::Keyboard(ke) if ke.code == Key::Up && ke.modifiers == KeyModifiers::NONE => {
                self.select_previous();
                None
            }
            Event::Keyboard(ke) if ke.code == Key::Down && ke.modifiers == KeyModifiers::NONE => {
                self.select_next();
                None
            }

            // Execute selected command
            Event::Keyboard(ke) if ke.code == Key::Enter && ke.modifiers == KeyModifiers::NONE => {
                self.execute_selected();
                None
            }

            // Close palette
            Event::Keyboard(ke) if ke.code == Key::Esc && ke.modifiers == KeyModifiers::NONE => {
                self.hide();
                self.clear_query();
                None
            }

            // Backspace - delete last character
            Event::Keyboard(ke)
                if ke.code == Key::Backspace && ke.modifiers == KeyModifiers::NONE =>
            {
                if !self.query.is_empty() {
                    self.query.pop();
                    self.refresh_results();
                    self.selected_index = 0;
                }
                None
            }

            // Character input - add to query
            Event::Keyboard(KeyEvent {
                code: Key::Char(c),
                modifiers,
                ..
            }) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                self.query.push(c);
                self.refresh_results();
                self.selected_index = 0;
                None
            }

            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::event::{Key, KeyEvent, KeyModifiers};

    // Test helper: Create sample commands
    fn create_test_commands() -> Vec<Command> {
        vec![
            Command::new("save", "Save File", "Save the current file", "File")
                .with_shortcut("Ctrl+S"),
            Command::new("open", "Open File", "Open a file", "File").with_shortcut("Ctrl+O"),
            Command::new(
                "close",
                "Close Window",
                "Close the current window",
                "Window",
            )
            .with_shortcut("Ctrl+W"),
            Command::new("copy", "Copy", "Copy selection to clipboard", "Edit")
                .with_shortcut("Ctrl+C"),
            Command::new("paste", "Paste", "Paste from clipboard", "Edit").with_shortcut("Ctrl+V"),
            Command::new(
                "settings",
                "Open Settings",
                "Open application settings",
                "View",
            )
            .with_shortcut("Ctrl+,"),
        ]
    }

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "Test Command", "A test", "Test");
        assert_eq!(cmd.id, "test");
        assert_eq!(cmd.name, "Test Command");
        assert_eq!(cmd.description, "A test");
        assert_eq!(cmd.category, "Test");
        assert!(cmd.shortcuts.is_empty());
    }

    #[test]
    fn test_command_with_shortcut() {
        let cmd = Command::new("test", "Test", "A test", "Test").with_shortcut("Ctrl+T");
        assert_eq!(cmd.shortcuts, vec!["Ctrl+T"]);
    }

    #[test]
    fn test_command_with_multiple_shortcuts() {
        let cmd = Command::new("test", "Test", "A test", "Test")
            .with_shortcuts(vec!["Ctrl+T".to_string(), "⌘+T".to_string()]);
        assert_eq!(cmd.shortcuts, vec!["Ctrl+T", "⌘+T"]);
    }

    #[test]
    fn test_palette_creation() {
        let palette = CommandPalette::new();
        assert_eq!(palette.commands().len(), 0);
        assert_eq!(palette.query(), "");
        assert!(!palette.is_visible());
    }

    #[test]
    fn test_palette_with_commands() {
        let commands = create_test_commands();
        let palette = CommandPalette::with_commands(commands);
        assert_eq!(palette.commands().len(), 6);
    }

    #[test]
    fn test_add_command() {
        let mut palette = CommandPalette::new();
        palette.add_command(Command::new("test", "Test", "A test", "Test"));
        assert_eq!(palette.commands().len(), 1);
    }

    #[test]
    fn test_visibility_toggle() {
        let mut palette = CommandPalette::new();
        assert!(!palette.is_visible());

        palette.show();
        assert!(palette.is_visible());

        palette.hide();
        assert!(!palette.is_visible());

        palette.toggle();
        assert!(palette.is_visible());
    }

    #[test]
    fn test_empty_query_shows_all() {
        let commands = create_test_commands();
        let palette = CommandPalette::with_commands(commands);

        assert_eq!(palette.result_count(), 6); // All commands shown
    }

    #[test]
    fn test_exact_match_scoring() {
        let commands = vec![Command::new("save", "Save", "Save file", "File")];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("save");
        assert_eq!(palette.result_count(), 1);

        let (_, score) = palette.results()[0];
        assert!(score >= 100.0); // Exact match score
    }

    #[test]
    fn test_prefix_match() {
        let commands = vec![Command::new("save", "Save File", "Save the file", "File")];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("sav");
        assert_eq!(palette.result_count(), 1);

        let (_, score) = palette.results()[0];
        assert!(score >= 90.0); // Prefix match score
    }

    #[test]
    fn test_substring_match() {
        let commands = vec![Command::new("open", "Open File", "Open a file", "File")];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("file");
        assert_eq!(palette.result_count(), 1);

        let (_, score) = palette.results()[0];
        assert!(score >= 70.0); // Substring match score
    }

    #[test]
    fn test_acronym_match() {
        let commands = vec![Command::new(
            "ocn",
            "Open Channel Names",
            "Open channel list",
            "View",
        )];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("ocn");
        assert_eq!(palette.result_count(), 1);
    }

    #[test]
    fn test_no_match() {
        let commands = vec![Command::new("save", "Save File", "Save file", "File")];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("xyz");
        assert_eq!(palette.result_count(), 0);
    }

    #[test]
    fn test_case_insensitive_search() {
        let commands = vec![Command::new("save", "Save File", "Save file", "File")];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("SAVE");
        assert_eq!(palette.result_count(), 1);
    }

    #[test]
    fn test_selection_navigation() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        assert_eq!(palette.selected_index(), 0);

        palette.select_next();
        assert_eq!(palette.selected_index(), 1);

        palette.select_next();
        assert_eq!(palette.selected_index(), 2);

        palette.select_previous();
        assert_eq!(palette.selected_index(), 1);

        palette.select_previous();
        assert_eq!(palette.selected_index(), 0);

        // Can't go below 0
        palette.select_previous();
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_selection_bounds() {
        let commands = vec![
            Command::new("cmd1", "Command 1", "First", "Test"),
            Command::new("cmd2", "Command 2", "Second", "Test"),
        ];
        let mut palette = CommandPalette::with_commands(commands);

        palette.select_next();
        palette.select_next();
        palette.select_next(); // Try to go past end

        assert_eq!(palette.selected_index(), 1); // Should stop at last
    }

    #[test]
    fn test_selected_command() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        let selected = palette.selected_command();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, palette.commands()[0].id);

        palette.select_next();
        let selected = palette.selected_command();
        assert!(selected.is_some());
    }

    #[test]
    fn test_execute_selected() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        palette.show();
        let executed = palette.execute_selected();

        assert!(executed.is_some());
        assert!(!palette.is_visible()); // Should hide after execution
        assert_eq!(palette.query(), ""); // Should clear query
    }

    #[test]
    fn test_recent_commands() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        // Execute some commands
        palette.execute_selected(); // Execute first command
        assert_eq!(palette.recent_commands().len(), 1);

        palette.select_next();
        palette.execute_selected(); // Execute second command
        assert_eq!(palette.recent_commands().len(), 2);

        // Most recent should be first
        let recent = palette.recent_commands();
        assert_eq!(recent[0], palette.commands()[1].id);
    }

    #[test]
    fn test_recent_bonus_scoring() {
        let commands = vec![
            Command::new("cmd1", "Command One", "First command", "Test"),
            Command::new("cmd2", "Command Two", "Second command", "Test"),
        ];
        let mut palette = CommandPalette::with_commands(commands);

        // Mark cmd2 as recent
        palette.add_to_recent("cmd2");

        // Search for "command" - both match, but cmd2 should score higher
        palette.set_query("command");

        let results = palette.results();
        assert_eq!(results.len(), 2);

        // First result should be cmd2 (recent)
        let (idx, _) = results[0];
        assert_eq!(palette.commands()[idx].id, "cmd2");
    }

    #[test]
    fn test_fuzzy_character_match() {
        let commands = vec![Command::new("test", "Save Document", "Save doc", "File")];
        let mut palette = CommandPalette::with_commands(commands);

        // "sdc" should fuzzy match "Save Document"
        palette.set_query("sdc");
        assert_eq!(palette.result_count(), 1);
    }

    #[test]
    fn test_description_match() {
        let commands = vec![Command::new(
            "test",
            "Test Command",
            "clipboard operations",
            "Edit",
        )];
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("clipboard");
        assert_eq!(palette.result_count(), 1);
    }

    #[test]
    fn test_query_clear() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("save");
        assert!(palette.result_count() < palette.commands().len());

        palette.clear_query();
        assert_eq!(palette.query(), "");
        assert_eq!(palette.result_count(), palette.commands().len());
    }

    #[test]
    fn test_categories() {
        let commands = create_test_commands();
        let palette = CommandPalette::with_commands(commands);

        // Should have File, Window, Edit, View categories
        assert!(palette.categories.len() >= 4);
    }

    // ===== PHASE 5b: UI & INTERACTION TESTS =====

    #[test]
    fn test_keyboard_char_input() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.show();

        // Simulate typing "sav"
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Char('s'),
            KeyModifiers::NONE,
        )));
        assert_eq!(palette.query(), "s");

        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Char('a'),
            KeyModifiers::NONE,
        )));
        assert_eq!(palette.query(), "sa");

        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Char('v'),
            KeyModifiers::NONE,
        )));
        assert_eq!(palette.query(), "sav");
    }

    #[test]
    fn test_keyboard_backspace() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.set_query("save");

        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));

        assert_eq!(palette.query(), "sav");
    }

    #[test]
    fn test_keyboard_backspace_empty() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        // Backspace on empty query should do nothing
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));

        assert_eq!(palette.query(), "");
    }

    #[test]
    fn test_keyboard_navigation_up_down() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        assert_eq!(palette.selected_index(), 0);

        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Down,
            KeyModifiers::NONE,
        )));
        assert_eq!(palette.selected_index(), 1);

        palette.on(Event::Keyboard(KeyEvent::new(Key::Up, KeyModifiers::NONE)));
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_keyboard_enter_executes() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.show();

        let initial_visible = palette.is_visible();
        assert!(initial_visible);

        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));

        // Should be hidden after execution
        assert!(!palette.is_visible());
        // Query should be cleared
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn test_keyboard_esc_closes() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.show();
        palette.set_query("test");

        palette.on(Event::Keyboard(KeyEvent::new(Key::Esc, KeyModifiers::NONE)));

        assert!(!palette.is_visible());
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn test_mockcomponent_state() {
        let commands = create_test_commands();
        let palette = CommandPalette::with_commands(commands);

        match palette.state() {
            State::One(StateValue::String(id)) => {
                // Should return ID of first command
                assert_eq!(id, palette.commands()[0].id);
            }
            _ => panic!("Expected State::One with command ID"),
        }
    }

    #[test]
    fn test_mockcomponent_perform_up_down() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        palette.perform(Cmd::Move(tuirealm::command::Direction::Down));
        assert_eq!(palette.selected_index(), 1);

        palette.perform(Cmd::Move(tuirealm::command::Direction::Up));
        assert_eq!(palette.selected_index(), 0);
    }

    #[test]
    fn test_mockcomponent_perform_submit() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.show();

        let result = palette.perform(Cmd::Submit);

        match result {
            CmdResult::Submit(State::One(StateValue::String(id))) => {
                assert!(!id.is_empty());
            }
            _ => panic!("Expected CmdResult::Submit with command ID"),
        }

        assert!(!palette.is_visible());
    }

    #[test]
    fn test_mockcomponent_perform_cancel() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.show();
        palette.set_query("test");

        palette.perform(Cmd::Cancel);

        assert!(!palette.is_visible());
        assert_eq!(palette.query(), "");
    }

    #[test]
    fn test_rendering_when_visible() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.show();

        use tuirealm::ratatui::Terminal;
        use tuirealm::ratatui::backend::TestBackend;
        use tuirealm::ratatui::layout::Rect;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 24);
                palette.view(frame, area);
            })
            .unwrap();

        // Rendering should succeed without panics
    }

    #[test]
    fn test_no_rendering_when_hidden() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);
        palette.hide(); // Explicitly hide

        use tuirealm::ratatui::Terminal;
        use tuirealm::ratatui::backend::TestBackend;
        use tuirealm::ratatui::layout::Rect;

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|frame| {
                let area = Rect::new(0, 0, 80, 24);
                palette.view(frame, area);
            })
            .unwrap();

        // Should render nothing (early return)
    }

    #[test]
    fn test_typing_updates_results() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        let initial_count = palette.result_count();

        // Type 's' - should filter results
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Char('s'),
            KeyModifiers::NONE,
        )));

        // Results should be filtered
        assert!(palette.result_count() <= initial_count);
        assert!(palette.result_count() > 0); // "save" and "settings" match
    }

    #[test]
    fn test_backspace_updates_results() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        palette.set_query("xyz"); // No matches
        assert_eq!(palette.result_count(), 0);

        // Backspace three times to clear query
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Backspace,
            KeyModifiers::NONE,
        )));

        // Empty query should show all commands
        assert!(palette.result_count() > 0);
    }

    #[test]
    fn test_selection_resets_on_query_change() {
        let commands = create_test_commands();
        let mut palette = CommandPalette::with_commands(commands);

        palette.select_next();
        palette.select_next();
        assert_eq!(palette.selected_index(), 2);

        // Typing should reset selection
        palette.on(Event::Keyboard(KeyEvent::new(
            Key::Char('s'),
            KeyModifiers::NONE,
        )));

        assert_eq!(palette.selected_index(), 0);
    }
}
