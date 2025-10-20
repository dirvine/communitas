//! Modern Calendar widget component
//!
//! Provides calendar visualization with event support:
//! - Date selection and navigation
//! - Event management and display
//! - Multiple calendar views (month, week, day)
//! - Event tooltips and details

use crate::messages::{Msg, TaskResult, UserEvent};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Utc, Weekday};
use std::collections::HashMap;
use tuirealm::{
    Component, Frame, MockComponent, State,
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    ratatui::{
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    },
};

/// Calendar event
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub date: NaiveDate,
    pub time: Option<DateTime<Utc>>,
    pub category: EventCategory,
    pub importance: EventImportance,
    pub color: Color,
}

impl CalendarEvent {
    pub fn new(
        id: String,
        title: String,
        date: NaiveDate,
        category: EventCategory,
        importance: EventImportance,
    ) -> Self {
        let color = Self::get_color_for_category(&category, &importance);

        Self {
            id,
            title,
            description: None,
            date,
            time: None,
            category,
            importance,
            color,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_time(mut self, time: DateTime<Utc>) -> Self {
        self.time = Some(time);
        self
    }

    fn get_color_for_category(category: &EventCategory, importance: &EventImportance) -> Color {
        match category {
            EventCategory::Meeting => match importance {
                EventImportance::Low => Color::Blue,
                EventImportance::Normal => Color::Cyan,
                EventImportance::High => Color::Yellow,
                EventImportance::Critical => Color::Red,
            },
            EventCategory::Deadline => Color::Red,
            EventCategory::Appointment => Color::Green,
            EventCategory::Reminder => Color::Yellow,
            EventCategory::Holiday => Color::Magenta,
            EventCategory::Personal => Color::Blue,
            EventCategory::Work => Color::Cyan,
            EventCategory::Other => Color::Gray,
        }
    }
}

/// Event categories
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventCategory {
    Meeting,
    Deadline,
    Appointment,
    Reminder,
    Holiday,
    Personal,
    Work,
    Other,
}

/// Event importance levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventImportance {
    Low,
    Normal,
    High,
    Critical,
}

/// Calendar view modes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalendarView {
    Month,
    Week,
    Day,
}

/// Calendar configuration
#[derive(Debug, Clone)]
pub struct CalendarConfig {
    pub view_mode: CalendarView,
    pub show_week_numbers: bool,
    pub show_weekends: bool,
    pub start_date: Option<NaiveDate>,
    pub selected_date: Option<NaiveDate>,
    pub highlighted_dates: Vec<NaiveDate>,
    pub first_day_of_week: Weekday,
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            view_mode: CalendarView::Month,
            show_week_numbers: true,
            show_weekends: true,
            start_date: None,
            selected_date: Some(Local::now().date_naive()),
            highlighted_dates: Vec::new(),
            first_day_of_week: Weekday::Sun,
        }
    }
}

/// Modern calendar widget
pub struct ModernCalendar {
    props: Props,
    config: CalendarConfig,
    events: HashMap<NaiveDate, Vec<CalendarEvent>>,
    _state: TableState,
    current_month: NaiveDate, // First day of current month
    selected_date: NaiveDate,
    _hover_date: Option<NaiveDate>,
}

impl Default for ModernCalendar {
    fn default() -> Self {
        Self::new(CalendarConfig::default())
    }
}

impl ModernCalendar {
    pub fn new(config: CalendarConfig) -> Self {
        let today = Local::now().date_naive();
        let current_month =
            NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

        Self {
            props: Props::default(),
            config,
            events: HashMap::new(),
            _state: TableState::default(),
            current_month,
            selected_date: today,
            _hover_date: None,
        }
    }

    pub fn add_event(&mut self, event: CalendarEvent) {
        self.events.entry(event.date).or_default().push(event);
    }

    pub fn remove_event(&mut self, event_id: &str) -> bool {
        for events in self.events.values_mut() {
            if let Some(pos) = events.iter().position(|e| e.id == event_id) {
                events.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn get_events_for_date(&self, date: NaiveDate) -> Vec<&CalendarEvent> {
        self.events
            .get(&date)
            .map(|events| events.iter().collect())
            .unwrap_or_default()
    }

    pub fn select_date(&mut self, date: NaiveDate) {
        self.selected_date = date;
        self.config.selected_date = Some(date);
    }

    pub fn navigate_month(&mut self, direction: i32) {
        if direction > 0 {
            if let Some(new_month) = self
                .current_month
                .checked_add_months(chrono::Months::new(1))
            {
                self.current_month = new_month;
            }
        } else if direction < 0
            && let Some(new_month) = self
                .current_month
                .checked_sub_months(chrono::Months::new(1))
        {
            self.current_month = new_month;
        }
    }

    pub fn navigate_week(&mut self, direction: i32) {
        if direction > 0 {
            let date = self
                .selected_date
                .checked_add_signed(Duration::weeks(1))
                .unwrap_or(self.selected_date);
            self.select_date(date);
        } else if direction < 0 {
            let date = self
                .selected_date
                .checked_sub_signed(Duration::weeks(1))
                .unwrap_or(self.selected_date);
            self.select_date(date);
        }
    }

    pub fn navigate_day(&mut self, direction: i32) {
        if direction > 0 {
            if let Some(new_date) = self.selected_date.checked_add_signed(Duration::days(1)) {
                self.select_date(new_date);
            }
        } else if direction < 0
            && let Some(new_date) = self.selected_date.checked_sub_signed(Duration::days(1))
        {
            self.select_date(new_date);
        }
    }

    fn get_month_days(&self) -> Vec<Vec<Option<NaiveDate>>> {
        let first_day = self.current_month;
        let current_date = first_day;

        // Find the first day of the calendar grid (might be previous month)
        let weekday_offset = if self.config.first_day_of_week == Weekday::Sun {
            first_day.weekday().num_days_from_sunday() as i32
        } else {
            first_day.weekday().num_days_from_monday() as i32
        };

        let start_date = current_date
            .checked_sub_signed(Duration::days(weekday_offset as i64))
            .unwrap_or(first_day);

        // Create 6 weeks of dates (42 days total)
        let mut weeks = Vec::new();
        let mut date = start_date;

        for _ in 0..6 {
            let mut week = Vec::new();
            for _ in 0..7 {
                let is_same_month = date.month() == first_day.month();
                let date_opt = if is_same_month { Some(date) } else { None };
                week.push(date_opt);

                date = date.checked_add_signed(Duration::days(1)).unwrap_or(date);
            }
            weeks.push(week);
        }

        weeks
    }

    #[allow(dead_code)]
    fn get_day_header(&self, weekday: Weekday) -> &'static str {
        match weekday {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue",
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        }
    }

    fn render_month_view(&self, _area: Rect) -> Table<'_> {
        let weeks = self.get_month_days();
        let today = Local::now().date_naive();

        // Build rows for the table
        let mut rows = Vec::new();

        // Header row with weekday names
        let header_cells = if self.config.first_day_of_week == Weekday::Sun {
            vec!["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        } else {
            vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
        };

        let header_row = Row::new(
            header_cells
                .into_iter()
                .map(|day| {
                    Cell::from(Line::from(Span::styled(
                        day,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )))
                })
                .collect::<Vec<_>>(),
        );

        rows.push(header_row);

        // Calendar rows (weeks)
        for week in weeks {
            let week_row = Row::new(
                week.into_iter()
                    .map(|date_opt| {
                        if let Some(date) = date_opt {
                            let is_today = date == today;
                            let is_selected = date == self.selected_date;
                            let has_events = self.events.contains_key(&date);
                            let is_current_month = date.month() == self.current_month.month();

                            let mut style = Style::default();

                            if is_selected {
                                style = style
                                    .fg(Color::Black)
                                    .bg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD);
                            } else if is_today {
                                style = style
                                    .fg(Color::Black)
                                    .bg(Color::Green)
                                    .add_modifier(Modifier::BOLD);
                            } else if !is_current_month {
                                style = style.fg(Color::DarkGray);
                            } else {
                                style = style.fg(Color::White);
                            }

                            if has_events {
                                style = style.add_modifier(Modifier::UNDERLINED);
                            }

                            let mut line =
                                Line::from(vec![Span::styled(format!("{:2}", date.day()), style)]);

                            // Add event indicator
                            if has_events && !is_selected {
                                let events = self.get_events_for_date(date);
                                if !events.is_empty() {
                                    line.push_span(Span::styled(
                                        "●",
                                        Style::default().fg(events[0].color),
                                    ));
                                }
                            } else if has_events && is_selected {
                                let events = self.get_events_for_date(date);
                                if !events.is_empty() {
                                    line.push_span(Span::styled(
                                        "●",
                                        Style::default().fg(Color::Black),
                                    ));
                                }
                            }

                            Cell::from(line)
                        } else {
                            Cell::from("  ")
                        }
                    })
                    .collect::<Vec<_>>(),
            );

            rows.push(week_row);
        }

        Table::new(rows, [Constraint::Length(3); 7])
            .style(Style::default())
            .block(
                Block::default()
                    .title(format!(
                        "{} {}",
                        self.current_month.format("%B %Y"),
                        if self.events.contains_key(&self.selected_date) {
                            format!(
                                " • {} events",
                                self.get_events_for_date(self.selected_date).len()
                            )
                        } else {
                            String::new()
                        }
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .style(Style::default().fg(Color::White))
    }

    fn render_events_list(&self, _area: Rect) -> Paragraph<'_> {
        let events = self.get_events_for_date(self.selected_date);

        let lines = if events.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No events for this date",
                    Style::default()
                        .fg(Color::Gray)
                        .add_modifier(Modifier::ITALIC),
                )),
            ]
        } else {
            let mut lines = vec![
                Line::from(Span::styled(
                    "Events:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];

            for (i, event) in events.iter().enumerate() {
                let importance_indicator = match event.importance {
                    EventImportance::Low => "○",
                    EventImportance::Normal => "●",
                    EventImportance::High => "◉",
                    EventImportance::Critical => "⚡",
                };

                lines.push(Line::from(vec![
                    Span::styled(importance_indicator, Style::default().fg(event.color)),
                    Span::raw(" "),
                    Span::styled(
                        &event.title,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));

                if let Some(ref description) = event.description {
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(description, Style::default().fg(Color::Gray)),
                    ]));
                }

                if i < events.len() - 1 {
                    lines.push(Line::from(""));
                }
            }

            lines
        };

        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(
                        "Events for {}",
                        self.selected_date.format("%B %d, %Y")
                    ))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)),
            )
            .alignment(Alignment::Left)
    }

    fn render_help_text(&self, _area: Rect) -> Paragraph<'_> {
        let help_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Calendar Navigation:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("←→ Weeks", Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled("↑↓ Days", Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled("PgUp/PgDn Months", Style::default().fg(Color::Green)),
                Span::raw(" "),
                Span::styled("Today 't'", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Actions:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled("Enter Select", Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled("Esc Close", Style::default().fg(Color::Gray)),
            ]),
        ];

        Paragraph::new(help_lines).alignment(Alignment::Center)
    }
}

impl MockComponent for ModernCalendar {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        // Split area into calendar and events
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // Calendar
                Constraint::Percentage(40), // Events list
            ])
            .split(area);

        // Render calendar
        let calendar = self.render_month_view(chunks[0]);
        frame.render_widget(calendar, chunks[0]);

        // Render events list
        let events_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Events list
                Constraint::Length(4), // Help text
            ])
            .split(chunks[1]);

        let events_list = self.render_events_list(events_area[0]);
        frame.render_widget(events_list, events_area[0]);

        let help_text = self.render_help_text(events_area[1]);
        frame.render_widget(help_text, events_area[1]);
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
            Cmd::Custom("navigate_month_forward") => {
                self.navigate_month(1);
                CmdResult::None
            }
            Cmd::Custom("navigate_month_backward") => {
                self.navigate_month(-1);
                CmdResult::None
            }
            Cmd::Custom("navigate_week_forward") => {
                self.navigate_week(1);
                CmdResult::None
            }
            Cmd::Custom("navigate_week_backward") => {
                self.navigate_week(-1);
                CmdResult::None
            }
            Cmd::Custom("navigate_day_forward") => {
                self.navigate_day(1);
                CmdResult::None
            }
            Cmd::Custom("navigate_day_backward") => {
                self.navigate_day(-1);
                CmdResult::None
            }
            Cmd::Custom("today") => {
                self.select_date(Local::now().date_naive());
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for ModernCalendar {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(event) => {
                use tuirealm::event::Key;

                match event.code {
                    Key::Left => {
                        self.navigate_day(-1);
                    }
                    Key::Right => {
                        self.navigate_day(1);
                    }
                    Key::Up => {
                        self.navigate_week(-1);
                    }
                    Key::Down => {
                        self.navigate_week(1);
                    }
                    Key::PageUp => {
                        self.navigate_month(-1);
                    }
                    Key::PageDown => {
                        self.navigate_month(1);
                    }
                    Key::Char('t') => {
                        self.select_date(Local::now().date_naive());
                    }
                    Key::Enter => {
                        // Return selected date
                        tracing::info!("Calendar date selected: {}", self.selected_date);
                        return Some(Msg::User(UserEvent::TaskCompleted {
                            task_id: "calendar_date_selected".to_string(),
                            result: TaskResult::Success(self.selected_date.to_string()),
                        }));
                    }
                    _ => {}
                }
                None
            }
            Event::Mouse(_) => {
                // Handle mouse clicks on calendar dates
                // TODO: Implement mouse interaction
                None
            }
            _ => None,
        }
    }
}

// Re-export for convenience

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::UserEvent;

    #[test]
    fn test_calendar_event_creation() {
        let today = Local::now().date_naive();
        let event = CalendarEvent::new(
            "1".to_string(),
            "Test Event".to_string(),
            today,
            EventCategory::Meeting,
            EventImportance::Normal,
        );

        assert_eq!(event.id, "1");
        assert_eq!(event.title, "Test Event");
        assert_eq!(event.category, EventCategory::Meeting);
        assert_eq!(event.importance, EventImportance::Normal);
        assert!(event.description.is_none());
        assert!(event.time.is_none());
    }

    #[test]
    fn test_calendar_event_with_fields() {
        let today = Local::now().date_naive();
        let event = CalendarEvent::new(
            "1".to_string(),
            "Test Event".to_string(),
            today,
            EventCategory::Meeting,
            EventImportance::Normal,
        )
        .with_description("Description".to_string())
        .with_time(Utc::now());

        assert_eq!(event.description.as_ref().unwrap(), "Description");
        assert!(event.time.is_some());
    }

    #[test]
    fn test_calendar_config_default() {
        let config = CalendarConfig::default();
        assert!(matches!(config.view_mode, CalendarView::Month));
        assert!(config.show_week_numbers);
        assert!(config.show_weekends);
        assert!(config.start_date.is_none());
        assert!(config.selected_date.is_some());
        assert!(config.highlighted_dates.is_empty());
    }

    #[test]
    fn test_modern_calendar_creation() {
        let config = CalendarConfig::default();
        let calendar = ModernCalendar::new(config);

        assert!(calendar.events.is_empty());
        assert_eq!(calendar.selected_date, Local::now().date_naive());
        assert_eq!(
            calendar.current_month.month(),
            Local::now().date_naive().month()
        );
    }

    #[test]
    fn test_modern_calendar_add_event() {
        let mut calendar = ModernCalendar::new(CalendarConfig::default());
        let today = Local::now().date_naive();

        let event = CalendarEvent::new(
            "1".to_string(),
            "Test Event".to_string(),
            today,
            EventCategory::Meeting,
            EventImportance::Normal,
        );

        calendar.add_event(event);
        assert_eq!(calendar.get_events_for_date(today).len(), 1);
    }

    #[test]
    fn test_modern_calendar_select_date() {
        let mut calendar = ModernCalendar::new(CalendarConfig::default());
        let new_date = Local::now()
            .date_naive()
            .checked_add_days(chrono::Days::new(1))
            .unwrap();

        calendar.select_date(new_date);
        assert_eq!(calendar.selected_date, new_date);
        assert_eq!(calendar.config.selected_date.unwrap(), new_date);
    }

    #[test]
    fn test_modern_calendar_navigation() {
        let mut calendar = ModernCalendar::new(CalendarConfig::default());
        let original_date = calendar.selected_date;

        // Navigate day forward
        calendar.navigate_day(1);
        assert_ne!(calendar.selected_date, original_date);

        // Navigate day backward
        calendar.navigate_day(-1);
        assert_eq!(calendar.selected_date, original_date);

        // Navigate week forward
        let one_week_later = calendar
            .selected_date
            .checked_add_signed(Duration::weeks(1))
            .unwrap();
        calendar.navigate_week(1);
        assert_eq!(calendar.selected_date, one_week_later);

        // Navigate month forward
        let original_month = calendar.current_month.month();
        calendar.navigate_month(1);
        assert_ne!(calendar.current_month.month(), original_month);
    }

    #[test]
    fn test_modern_calendar_get_month_days() {
        let calendar = ModernCalendar::new(CalendarConfig::default());
        let weeks = calendar.get_month_days();

        // Should have 6 weeks
        assert_eq!(weeks.len(), 6);

        // Each week should have 7 days
        for week in &weeks {
            assert_eq!(week.len(), 7);
        }
    }

    #[test]
    fn test_modern_calendar_events() {
        let mut calendar = ModernCalendar::new(CalendarConfig::default());
        let today = Local::now().date_naive();

        // Add event
        let event = CalendarEvent::new(
            "1".to_string(),
            "Test Event".to_string(),
            today,
            EventCategory::Meeting,
            EventImportance::Normal,
        );
        calendar.add_event(event);

        // Check event exists
        let events = calendar.get_events_for_date(today);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].title, "Test Event");

        // Remove event
        assert!(calendar.remove_event("1"));
        let events = calendar.get_events_for_date(today);
        assert_eq!(events.len(), 0);

        // Try to remove non-existent event
        assert!(!calendar.remove_event("999"));
    }

    #[test]
    fn test_modern_calendar_components() {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};

        let mut calendar = ModernCalendar::new(CalendarConfig::default());
        let original_date = calendar.selected_date;

        // Test left/right navigation
        calendar.on(Event::Keyboard(KeyEvent::new(
            Key::Right,
            KeyModifiers::NONE,
        )));
        assert_ne!(calendar.selected_date, original_date);

        calendar.on(Event::Keyboard(KeyEvent::new(
            Key::Left,
            KeyModifiers::NONE,
        )));
        assert_eq!(calendar.selected_date, original_date);

        // Test today ('t')
        let test_date = original_date
            .checked_add_days(chrono::Days::new(5))
            .unwrap();
        calendar.select_date(test_date);
        calendar.on(Event::Keyboard(KeyEvent::new(
            Key::Char('t'),
            KeyModifiers::NONE,
        )));
        assert_eq!(calendar.selected_date, original_date);

        // Test Enter selection
        let msg = calendar.on(Event::Keyboard(KeyEvent::new(
            Key::Enter,
            KeyModifiers::NONE,
        )));
        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));
    }

    #[test]
    fn test_mock_component_perform() {
        let mut calendar = ModernCalendar::new(CalendarConfig::default());

        // Test month navigation
        let original_month = calendar.current_month.month();
        calendar.perform(Cmd::Custom("navigate_month_forward"));
        assert_ne!(calendar.current_month.month(), original_month);

        calendar.perform(Cmd::Custom("navigate_month_backward"));
        assert_eq!(calendar.current_month.month(), original_month);

        // Test today command
        let test_date = Local::now()
            .date_naive()
            .checked_add_days(chrono::Days::new(5))
            .unwrap();
        calendar.select_date(test_date);
        calendar.perform(Cmd::Custom("today"));
        assert_eq!(calendar.selected_date, Local::now().date_naive());
    }
}
