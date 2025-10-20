//! Performance monitoring and metrics component
//!
//! Provides real-time performance monitoring including:
//! - Frame rate tracking
//! - Memory usage
//! - Network latency
//! - Component render times

use crate::messages::{Msg, UserEvent};
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
        widgets::{Block, Borders, Gauge, Paragraph},
    },
};

/// Performance metrics data
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub fps: f64,
    pub frame_time: Duration,
    pub memory_usage_mb: f64,
    pub network_latency_ms: f64,
    pub component_render_times: Vec<String>,
    pub last_updated: Instant,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            frame_time: Duration::ZERO,
            memory_usage_mb: 0.0,
            network_latency_ms: 0.0,
            component_render_times: Vec::new(),
            last_updated: Instant::now(),
        }
    }
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_fps(&mut self, fps: f64, frame_time: Duration) {
        self.fps = fps;
        self.frame_time = frame_time;
        self.last_updated = Instant::now();
    }

    pub fn update_memory(&mut self, usage_mb: f64) {
        self.memory_usage_mb = usage_mb;
    }

    pub fn update_network_latency(&mut self, latency_ms: f64) {
        self.network_latency_ms = latency_ms;
    }

    pub fn add_component_time(&mut self, component: &str, time: Duration) {
        let entry = format!("{}: {:?}ms", component, time.as_millis());
        self.component_render_times.push(entry);

        // Keep only last 10 components
        if self.component_render_times.len() > 10 {
            self.component_render_times.remove(0);
        }
    }

    pub fn is_stale(&self) -> bool {
        self.last_updated.elapsed() > Duration::from_secs(5)
    }
}

/// Performance monitoring component
#[derive(Debug)]
pub struct PerformanceMonitor {
    props: Props,
    metrics: PerformanceMetrics,
    visible: bool,
    fps_counter: u32,
    last_fps_update: Instant,
    frame_times: Vec<Instant>, // Changed to Vec<Instant> to track frame timestamps
    max_frame_times: usize,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            props: Props::default(),
            metrics: PerformanceMetrics::new(),
            visible: false,
            fps_counter: 0,
            last_fps_update: Instant::now(),
            frame_times: Vec::new(),
            max_frame_times: 60, // Store 60 frames for FPS calculation
        }
    }

    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn record_frame(&mut self) {
        let now = Instant::now();

        // Add current frame timestamp
        self.frame_times.push(now);

        // Keep only recent frame times
        if self.frame_times.len() > self.max_frame_times {
            self.frame_times.remove(0);
        }

        // Update FPS counter
        self.fps_counter += 1;

        // Update FPS display every second
        if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            let fps = self.fps_counter as f64;

            // Calculate average frame time from timestamps
            let avg_frame_time = if self.frame_times.len() >= 2 {
                let first = self.frame_times.first();
                let last = self.frame_times.last();
                match (first, last) {
                    (Some(first), Some(last)) => {
                        let total_duration = last.duration_since(*first);
                        total_duration / (self.frame_times.len() - 1) as u32
                    }
                    _ => Duration::ZERO,
                }
            } else {
                Duration::ZERO
            };

            self.metrics.update_fps(fps, avg_frame_time);
            self.fps_counter = 0;
            self.last_fps_update = now;
        }

        // Update memory usage (simulate for demo)
        if self.metrics.is_stale() {
            // In a real app, this would query actual memory usage
            let memory_mb = simulate_memory_usage();
            self.metrics.update_memory(memory_mb);

            // Simulate network latency
            let latency = simulate_network_latency();
            self.metrics.update_network_latency(latency);
        }
    }

    fn get_status_color(&self, value: f64, good: f64, warning: f64) -> Color {
        if value >= good {
            Color::Green
        } else if value >= warning {
            Color::Yellow
        } else {
            Color::Red
        }
    }

    fn format_line<'a>(&self, label: &str, value: &'a str, color: Color) -> Line<'a> {
        Line::from(vec![
            Span::styled(format!("{}: ", label), Style::default().fg(Color::Gray)),
            Span::styled(value, Style::default().fg(color)),
        ])
    }
}

impl MockComponent for PerformanceMonitor {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if !self.is_visible() {
            return;
        }

        // Split area into header and content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Header with FPS gauge
                Constraint::Min(10),   // Metrics content
            ])
            .split(area);

        // Render FPS gauge in header
        let fps_normalized = (self.metrics.fps / 60.0).min(1.0);
        let fps_color = self.get_status_color(self.metrics.fps, 30.0, 15.0);

        let fps_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(Style::default().fg(fps_color).bg(Color::DarkGray))
            .percent(fps_normalized as u16 * 100)
            .label(format!("FPS: {:.1}", self.metrics.fps));

        frame.render_widget(fps_gauge, chunks[0]);

        // Render detailed metrics
        // Bind formatted strings to variables so they live long enough
        let frame_time_str = format!("{:.1}ms", self.metrics.frame_time.as_millis() as f64);
        let memory_str = format!("{:.1} MB", self.metrics.memory_usage_mb);
        let network_str = format!("{:.1} ms", self.metrics.network_latency_ms);

        let metrics_text = vec![
            self.format_line(
                "Frame Time",
                &frame_time_str,
                self.get_status_color(self.metrics.frame_time.as_millis() as f64, 16.7, 33.3),
            ),
            self.format_line(
                "Memory",
                &memory_str,
                self.get_status_color(self.metrics.memory_usage_mb, 100.0, 200.0),
            ),
            self.format_line(
                "Network",
                &network_str,
                self.get_status_color(self.metrics.network_latency_ms, 50.0, 200.0),
            ),
            Line::from(""),
            Line::from(Span::styled(
                "Component Render Times:",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        // Add component render times
        let mut content = metrics_text;
        for component_time in &self.metrics.component_render_times {
            content.push(Line::from(Span::styled(
                component_time.clone(),
                Style::default().fg(Color::Gray),
            )));
        }

        let metrics_paragraph = Paragraph::new(content)
            .block(
                Block::default()
                    .title("Performance Monitor")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            )
            .alignment(Alignment::Left);

        frame.render_widget(metrics_paragraph, chunks[1]);
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
            Cmd::Custom("toggle") => {
                self.toggle_visibility();
                CmdResult::Submit(State::None)
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for PerformanceMonitor {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(event) => {
                use tuirealm::event::Key;

                match event.code {
                    Key::Function(12) => {
                        self.toggle_visibility();
                        Some(Msg::User(UserEvent::TaskCompleted {
                            task_id: "perf_monitor_toggle".to_string(),
                            result: TaskResult::Success("Performance monitor toggled".to_string()),
                        }))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

// Helper functions (in a real app, these would query actual system metrics)
fn simulate_memory_usage() -> f64 {
    // Simulate memory usage between 50-250 MB
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(50.0..250.0)
}

fn simulate_network_latency() -> f64 {
    // Simulate network latency between 10-500ms
    use rand::Rng;
    let mut rng = rand::thread_rng();
    rng.gen_range(10.0..500.0)
}

// Re-export for convenience
use crate::messages::TaskResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_creation() {
        let metrics = PerformanceMetrics::new();
        assert_eq!(metrics.fps, 0.0);
        assert_eq!(metrics.memory_usage_mb, 0.0);
        assert_eq!(metrics.network_latency_ms, 0.0);
        assert!(metrics.component_render_times.is_empty());
    }

    #[test]
    fn test_performance_metrics_update_fps() {
        let mut metrics = PerformanceMetrics::new();
        metrics.update_fps(60.0, Duration::from_millis(16));

        assert_eq!(metrics.fps, 60.0);
        assert_eq!(metrics.frame_time, Duration::from_millis(16));
    }

    #[test]
    fn test_performance_metrics_update_memory() {
        let mut metrics = PerformanceMetrics::new();
        metrics.update_memory(128.5);

        assert_eq!(metrics.memory_usage_mb, 128.5);
    }

    #[test]
    fn test_performance_metrics_add_component_time() {
        let mut metrics = PerformanceMetrics::new();
        metrics.add_component_time("test_component", Duration::from_millis(5));

        assert_eq!(metrics.component_render_times.len(), 1);
        assert!(metrics.component_render_times[0].contains("test_component"));
        assert!(metrics.component_render_times[0].contains("5ms"));
    }

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(!monitor.is_visible());
        assert_eq!(monitor.max_frame_times, 60);
    }

    #[test]
    fn test_performance_monitor_toggle_visibility() {
        let mut monitor = PerformanceMonitor::new();
        monitor.toggle_visibility();
        assert!(monitor.is_visible());

        monitor.toggle_visibility();
        assert!(!monitor.is_visible());
    }

    #[test]
    fn test_performance_monitor_set_visible() {
        let mut monitor = PerformanceMonitor::new();
        monitor.set_visible(true);
        assert!(monitor.is_visible());

        monitor.set_visible(false);
        assert!(!monitor.is_visible());
    }

    #[test]
    fn test_performance_monitor_record_frame() {
        let mut monitor = PerformanceMonitor::new();

        // Record first frame
        monitor.record_frame();
        assert_eq!(monitor.frame_times.len(), 1);
        assert_eq!(monitor.fps_counter, 1); // Counter increments on each frame
    }

    #[test]
    fn test_performance_monitor_status_colors() {
        let monitor = PerformanceMonitor::new();

        // Test good performance
        assert_eq!(monitor.get_status_color(60.0, 30.0, 15.0), Color::Green);

        // Test warning performance
        assert_eq!(monitor.get_status_color(20.0, 30.0, 15.0), Color::Yellow);

        // Test poor performance
        assert_eq!(monitor.get_status_color(10.0, 30.0, 15.0), Color::Red);
    }

    #[test]
    fn test_performance_monitor_format_line() {
        let monitor = PerformanceMonitor::new();
        let line = monitor.format_line("Test", "Value", Color::Blue);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Test: ");
        assert_eq!(line.spans[0].style.fg.unwrap(), Color::Gray);
        assert_eq!(line.spans[1].content, "Value");
        assert_eq!(line.spans[1].style.fg.unwrap(), Color::Blue);
    }

    #[test]
    fn test_component_events() {
        use tuirealm::event::{Key, KeyEvent, KeyModifiers};

        let mut monitor = PerformanceMonitor::new();

        // Test F12 toggle
        let msg = monitor.on(Event::Keyboard(KeyEvent::new(
            Key::Function(12),
            KeyModifiers::NONE,
        )));

        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));
        assert!(monitor.is_visible());
    }

    #[test]
    fn test_mock_component_perform() {
        let mut monitor = PerformanceMonitor::new();

        // Test toggle command
        let result = monitor.perform(Cmd::Custom("toggle"));
        assert!(matches!(result, CmdResult::Submit(State::None)));
        assert!(monitor.is_visible());

        // Test other commands
        let result = monitor.perform(Cmd::Submit);
        assert!(matches!(result, CmdResult::None));
    }
}
