//! Modern data visualization widgets
//!
//! Provides advanced data visualization components using ratatui 0.30+ capabilities:
//! - Interactive charts (line, bar, sparkline)
//! - Network diagrams
//! - Real-time metrics dashboards
//! - Statistical visualizations

use crate::messages::{Msg, UserEvent};
use std::time::{Duration, Instant};
use tuirealm::{
    Component, Frame, MockComponent, State,
    command::{Cmd, CmdResult},
    event::{Event, NoUserEvent},
    props::{AttrValue, Attribute, Props},
    ratatui::{
        layout::{Alignment, Rect},
        style::{Color, Modifier, Style},
        symbols::Marker,
        text::{Line, Span, Text},
        widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph, Sparkline},
    },
};

/// Chart data point
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub timestamp: Option<Instant>,
}

impl DataPoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            timestamp: None,
        }
    }

    pub fn with_timestamp(x: f64, y: f64, timestamp: Instant) -> Self {
        Self {
            x,
            y,
            timestamp: Some(timestamp),
        }
    }
}

/// Time series data collection
#[derive(Debug, Clone)]
pub struct TimeSeriesData {
    pub name: String,
    pub points: Vec<DataPoint>,
    pub color: Color,
    pub max_points: usize,
}

impl TimeSeriesData {
    pub fn new(name: String, color: Color, max_points: usize) -> Self {
        Self {
            name,
            points: Vec::new(),
            color,
            max_points,
        }
    }

    pub fn add_point(&mut self, y: f64) {
        let x = if let Some(last_point) = self.points.last() {
            last_point.x + 1.0
        } else {
            0.0
        };

        let point = DataPoint::with_timestamp(x, y, Instant::now());
        self.points.push(point);

        // Maintain max points
        if self.points.len() > self.max_points {
            self.points.remove(0);
            // Re-index x values
            for (i, point) in self.points.iter_mut().enumerate() {
                point.x = i as f64;
            }
        }
    }

    pub fn get_values(&self) -> Vec<(f64, f64)> {
        self.points.iter().map(|p| (p.x, p.y)).collect()
    }

    pub fn get_latest_value(&self) -> Option<f64> {
        self.points.last().map(|p| p.y)
    }

    pub fn get_average(&self) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.points.iter().map(|p| p.y).sum();
        sum / self.points.len() as f64
    }

    pub fn get_max(&self) -> f64 {
        self.points.iter().map(|p| p.y).fold(f64::MIN, f64::max)
    }

    pub fn get_min(&self) -> f64 {
        self.points.iter().map(|p| p.y).fold(f64::MAX, f64::min)
    }
}

/// Network performance data
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub upload_mbps: TimeSeriesData,
    pub download_mbps: TimeSeriesData,
    pub latency_ms: TimeSeriesData,
    pub packet_loss: TimeSeriesData,
    pub peer_count: TimeSeriesData,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMetrics {
    pub fn new() -> Self {
        let max_points = 60; // 60 data points

        Self {
            upload_mbps: TimeSeriesData::new("Upload".to_string(), Color::Green, max_points),
            download_mbps: TimeSeriesData::new("Download".to_string(), Color::Blue, max_points),
            latency_ms: TimeSeriesData::new("Latency".to_string(), Color::Yellow, max_points),
            packet_loss: TimeSeriesData::new("Packet Loss".to_string(), Color::Red, max_points),
            peer_count: TimeSeriesData::new("Peers".to_string(), Color::Cyan, max_points),
        }
    }

    pub fn update(&mut self, upload: f64, download: f64, latency: f64, loss: f64, peers: f64) {
        self.upload_mbps.add_point(upload);
        self.download_mbps.add_point(download);
        self.latency_ms.add_point(latency);
        self.packet_loss.add_point(loss);
        self.peer_count.add_point(peers);
    }

    pub fn simulate_update(&mut self) {
        // Simulate network metrics
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let upload = rng.gen_range(0.1..10.0);
        let download = rng.gen_range(1.0..100.0);
        let latency = rng.gen_range(5.0..500.0);
        let loss = rng.gen_range(0.0..5.0);
        let peers = rng.gen_range(1.0..50.0);

        self.update(upload, download, latency, loss, peers);
    }
}

/// System resource data
#[derive(Debug, Clone)]
pub struct SystemResources {
    pub cpu_usage: TimeSeriesData,
    pub memory_usage: TimeSeriesData,
    pub disk_usage: TimeSeriesData,
    pub network_io: TimeSeriesData,
}

impl Default for SystemResources {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemResources {
    pub fn new() -> Self {
        let max_points = 120; // More points for system monitoring

        Self {
            cpu_usage: TimeSeriesData::new("CPU %".to_string(), Color::Red, max_points),
            memory_usage: TimeSeriesData::new("Memory %".to_string(), Color::Green, max_points),
            disk_usage: TimeSeriesData::new("Disk %".to_string(), Color::Blue, max_points),
            network_io: TimeSeriesData::new("Network MB/s".to_string(), Color::Yellow, max_points),
        }
    }

    pub fn update(&mut self, cpu: f64, memory: f64, disk: f64, network_io: f64) {
        self.cpu_usage.add_point(cpu);
        self.memory_usage.add_point(memory);
        self.disk_usage.add_point(disk);
        self.network_io.add_point(network_io);
    }

    pub fn simulate_update(&mut self) {
        // Simulate system metrics
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let cpu = rng.gen_range(5.0..95.0);
        let memory = rng.gen_range(20.0..85.0);
        let disk = rng.gen_range(15.0..75.0);
        let network_io = rng.gen_range(0.0..25.0);

        self.update(cpu, memory, disk, network_io);
    }
}

/// Data visualization widget options
#[derive(Debug, Clone)]
pub struct DataVisOptions {
    pub show_grid: bool,
    pub show_labels: bool,
    pub show_legend: bool,
    pub auto_scale: bool,
    pub color_scheme: ColorScheme,
    pub refresh_interval: Duration,
}

impl Default for DataVisOptions {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_labels: true,
            show_legend: true,
            auto_scale: true,
            color_scheme: ColorScheme::Default,
            refresh_interval: Duration::from_millis(1000),
        }
    }
}

/// Color schemes for charts
#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    Dark,
    Colorful,
    Monochrome,
    HighContrast,
}

impl ColorScheme {
    pub fn get_colors(&self) -> Vec<Color> {
        match self {
            ColorScheme::Default => vec![
                Color::Blue,
                Color::Green,
                Color::Red,
                Color::Yellow,
                Color::Magenta,
                Color::Cyan,
            ],
            ColorScheme::Dark => vec![
                Color::DarkGray,
                Color::Gray,
                Color::White,
                Color::LightBlue,
                Color::LightGreen,
                Color::LightRed,
            ],
            ColorScheme::Colorful => vec![
                Color::Blue,
                Color::Green,
                Color::Magenta,
                Color::Yellow,
                Color::Red,
                Color::Cyan,
            ],
            ColorScheme::Monochrome => vec![
                Color::White,
                Color::Gray,
                Color::DarkGray,
                Color::White,
                Color::LightRed,
                Color::LightBlue, // LightGray → White
            ],
            ColorScheme::HighContrast => vec![
                Color::White,
                Color::Yellow,
                Color::Cyan,
                Color::Magenta,
                Color::Green,
                Color::Red,
            ],
        }
    }
}

/// Modern chart widget
pub struct ModernChart {
    props: Props,
    title: String,
    datasets: Vec<TimeSeriesData>,
    options: DataVisOptions,
    last_update: Option<Instant>,
}

impl ModernChart {
    pub fn new(title: String, options: DataVisOptions) -> Self {
        Self {
            props: Props::default(),
            title,
            datasets: Vec::new(),
            options,
            last_update: None,
        }
    }

    pub fn add_dataset(&mut self, dataset: TimeSeriesData) {
        self.datasets.push(dataset);
    }

    pub fn update_dataset(&mut self, index: usize, value: f64) -> bool {
        if index < self.datasets.len() {
            self.datasets[index].add_point(value);
            self.last_update = Some(Instant::now());
            true
        } else {
            false
        }
    }

    pub fn get_y_bounds(&self) -> (f64, f64) {
        if self.datasets.is_empty() {
            return (0.0, 100.0);
        }

        let mut min = f64::MAX;
        let mut max = f64::MIN;

        for dataset in &self.datasets {
            if !dataset.points.is_empty() {
                min = min.min(dataset.get_min());
                max = max.max(dataset.get_max());
            }
        }

        // Add some padding
        if min == max {
            min -= 1.0;
            max += 1.0;
        } else {
            let padding = (max - min) * 0.1;
            min -= padding;
            max += padding;
        }

        (min.clamp(0.0, 100.0), max)
    }

    pub fn get_x_bounds(&self) -> (f64, f64) {
        if self.datasets.is_empty() {
            return (0.0, 1.0);
        }

        let max_x = self
            .datasets
            .iter()
            .map(|d| d.points.iter().map(|p| p.x).fold(f64::MIN, f64::max))
            .fold(f64::MIN, f64::max);

        (0.0, max_x.max(1.0))
    }
}

impl MockComponent for ModernChart {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        if self.datasets.is_empty() {
            // Show empty chart message
            let empty_message = Paragraph::new(Text::from(vec![Line::from(Span::styled(
                "No data available",
                Style::default()
                    .fg(Color::Gray)
                    .add_modifier(Modifier::ITALIC),
            ))]))
            .block(
                Block::default()
                    .title(self.title.as_str())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Center);

            frame.render_widget(empty_message, area);
            return;
        }

        // Create chart
        let (y_min, y_max) = self.get_y_bounds();
        let (x_min, x_max) = self.get_x_bounds();

        // Collect all data points first to avoid lifetime issues
        let dataset_data: Vec<(TimeSeriesData, Vec<(f64, f64)>)> = self
            .datasets
            .iter()
            .filter(|ds| !ds.points.is_empty())
            .map(|ds| (ds.clone(), ds.get_values()))
            .collect();

        let chart = Chart::new(
            dataset_data
                .iter()
                .map(|(dataset, data)| {
                    Dataset::default()
                        .name(dataset.name.as_str())
                        .marker(Marker::Dot)
                        .graph_type(GraphType::Line)
                        .style(Style::default().fg(dataset.color))
                        .data(data)
                })
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .title(self.title.as_str())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([x_min, x_max])
                .labels(if self.options.show_labels {
                    vec![
                        "0".into(),
                        format!("{}", (x_max / 2.0) as i32),
                        format!("{}", x_max as i32),
                    ]
                } else {
                    vec![]
                }),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([y_min, y_max])
                .labels(if self.options.show_labels {
                    vec![
                        format!("{:.0}", y_min),
                        format!("{:.0}", (y_min + y_max) / 2.0),
                        format!("{:.0}", y_max),
                    ]
                } else {
                    vec![]
                }),
        );

        if self.options.show_legend {
            let legend_area = area;
            frame.render_widget(chart, legend_area);

            // Render legend as separate overlay
            let legend_width = area.width.min(30);
            let legend_height = self.datasets.len() as u16 + 2;
            let legend_rect = Rect {
                x: area.x + area.width - legend_width - 1,
                y: area.y + 1,
                width: legend_width,
                height: legend_height,
            };

            let legend_lines: Vec<Line> = self
                .datasets
                .iter()
                .map(|dataset| {
                    Line::from(vec![
                        Span::styled("●", Style::default().fg(dataset.color)),
                        Span::raw(format!(" {}", dataset.name)),
                    ])
                })
                .collect();

            let legend_paragraph = Paragraph::new(Text::from(legend_lines))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                )
                .alignment(Alignment::Left);

            frame.render_widget(legend_paragraph, legend_rect);
        } else {
            frame.render_widget(chart, area);
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
            // NOTE: AttrValue::Usize, Float, and Attribute::Index don't exist in tuirealm
            // This update functionality is disabled until an alternative is found
            // Cmd::Custom("update") => {
            //     if let Some(index) = self.query(Attribute::Index) {
            //         if let AttrValue::Usize(index) = index {
            //             if let Some(value) = self.query(Attribute::Value) {
            //                 if let AttrValue::Float(value) = value {
            //                     self.update_dataset(index, value);
            //                     return CmdResult::None;
            //                 }
            //             }
            //         }
            //     }
            //     CmdResult::None
            // }
            Cmd::Custom("clear") => {
                for dataset in &mut self.datasets {
                    dataset.points.clear();
                }
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}

impl Component<Msg, NoUserEvent> for ModernChart {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(event) => {
                use tuirealm::event::Key;

                match event.code {
                    Key::Char('c') => {
                        // Clear all data
                        for dataset in &mut self.datasets {
                            dataset.points.clear();
                        }
                        Some(Msg::User(UserEvent::TaskCompleted {
                            task_id: "chart_cleared".to_string(),
                            result: TaskResult::Success("Chart data cleared".to_string()),
                        }))
                    }
                    Key::Function(8) => {
                        // Toggle legend
                        self.options.show_legend = !self.options.show_legend;
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Sparkline widget for mini metrics
pub struct ModernSparkline {
    props: Props,
    data: Vec<u64>,
    title: String,
    max: u64,
    color: Color,
}

impl ModernSparkline {
    pub fn new(title: String, max: u64, color: Color) -> Self {
        Self {
            props: Props::default(),
            data: Vec::new(),
            title,
            max,
            color,
        }
    }

    pub fn add_data(&mut self, value: u64) {
        self.data.push(value);

        // Keep only last 40 values (typical sparkline width)
        if self.data.len() > 40 {
            self.data.remove(0);
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl MockComponent for ModernSparkline {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(self.title.as_str())
                    .borders(Borders::ALL),
            )
            .data(&self.data)
            .max(self.max)
            .style(Style::default().fg(self.color));

        frame.render_widget(sparkline, area);
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
            // NOTE: AttrValue::U64 doesn't exist in tuirealm
            // This add functionality is disabled until an alternative is found
            // Cmd::Custom("add") => {
            //     if let Some(value) = self.query(Attribute::Value) {
            //         if let AttrValue::U64(value) = value {
            //             self.add_data(value);
            //             return CmdResult::None;
            //         }
            //     }
            //     CmdResult::None
            // }
            Cmd::Custom("clear") => {
                self.data.clear();
                CmdResult::None
            }
            _ => CmdResult::None,
        }
    }
}

// Re-export for convenience
use crate::messages::TaskResult;

#[cfg(test)]
mod tests {
    use super::*;
    use tuirealm::event::{Key, KeyEvent, KeyModifiers};

    #[test]
    fn test_data_point_creation() {
        let point = DataPoint::new(1.0, 2.0);
        assert_eq!(point.x, 1.0);
        assert_eq!(point.y, 2.0);
        assert!(point.timestamp.is_none());
    }

    #[test]
    fn test_time_series_data_creation() {
        let series = TimeSeriesData::new("Test".to_string(), Color::Blue, 10);
        assert_eq!(series.name, "Test");
        assert_eq!(series.color, Color::Blue);
        assert_eq!(series.max_points, 10);
        assert!(series.points.is_empty());
    }

    #[test]
    fn test_time_series_add_point() {
        let mut series = TimeSeriesData::new("Test".to_string(), Color::Blue, 5);

        series.add_point(1.0);
        assert_eq!(series.points.len(), 1);
        assert_eq!(series.points[0].x, 0.0);
        assert_eq!(series.points[0].y, 1.0);

        series.add_point(2.0);
        assert_eq!(series.points.len(), 2);
        assert_eq!(series.points[1].x, 1.0);
        assert_eq!(series.points[1].y, 2.0);
    }

    #[test]
    fn test_time_series_max_points() {
        let mut series = TimeSeriesData::new("Test".to_string(), Color::Blue, 3);

        series.add_point(1.0);
        series.add_point(2.0);
        series.add_point(3.0);
        series.add_point(4.0); // Should remove first point

        assert_eq!(series.points.len(), 3);
        assert_eq!(series.points[0].x, 0.0); // Re-indexed
        assert_eq!(series.points[0].y, 2.0); // Shifted
    }

    #[test]
    fn test_time_series_statistics() {
        let mut series = TimeSeriesData::new("Test".to_string(), Color::Blue, 10);

        series.add_point(10.0);
        series.add_point(20.0);
        series.add_point(30.0);

        assert_eq!(series.get_latest_value().unwrap(), 30.0);
        assert_eq!(series.get_average(), 20.0);
        assert_eq!(series.get_max(), 30.0);
        assert_eq!(series.get_min(), 10.0);
    }

    #[test]
    fn test_network_metrics_creation() {
        let metrics = NetworkMetrics::new();
        assert_eq!(metrics.upload_mbps.name, "Upload");
        assert_eq!(metrics.download_mbps.name, "Download");
        assert_eq!(metrics.latency_ms.name, "Latency");
        assert_eq!(metrics.packet_loss.name, "Packet Loss");
        assert_eq!(metrics.peer_count.name, "Peers");
    }

    #[test]
    fn test_network_metrics_update() {
        let mut metrics = NetworkMetrics::new();

        metrics.update(1.0, 2.0, 3.0, 4.0, 5.0);

        assert!(metrics.upload_mbps.get_latest_value().is_some());
        assert!(metrics.download_mbps.get_latest_value().is_some());
        assert!(metrics.latency_ms.get_latest_value().is_some());
        assert!(metrics.packet_loss.get_latest_value().is_some());
        assert!(metrics.peer_count.get_latest_value().is_some());
    }

    #[test]
    fn test_system_resources_creation() {
        let resources = SystemResources::new();
        assert_eq!(resources.cpu_usage.name, "CPU %");
        assert_eq!(resources.memory_usage.name, "Memory %");
        assert_eq!(resources.disk_usage.name, "Disk %");
        assert_eq!(resources.network_io.name, "Network MB/s");
    }

    #[test]
    fn test_data_vis_options_default() {
        let options = DataVisOptions::default();
        assert!(options.show_grid);
        assert!(options.show_labels);
        assert!(options.show_legend);
        assert!(options.auto_scale);
        assert!(matches!(options.color_scheme, ColorScheme::Default));
        assert_eq!(options.refresh_interval, Duration::from_millis(1000));
    }

    #[test]
    fn test_color_scheme_colors() {
        let default_colors = ColorScheme::Default.get_colors();
        assert_eq!(default_colors.len(), 6);
        assert!(default_colors.contains(&Color::Blue));
        assert!(default_colors.contains(&Color::Green));
    }

    #[test]
    fn test_modern_chart_creation() {
        let options = DataVisOptions::default();
        let chart = ModernChart::new("Test Chart".to_string(), options);
        assert_eq!(chart.title, "Test Chart");
        assert!(chart.datasets.is_empty());
    }

    #[test]
    fn test_modern_chart_add_dataset() {
        let mut chart = ModernChart::new("Test Chart".to_string(), DataVisOptions::default());

        let dataset = TimeSeriesData::new("Dataset1".to_string(), Color::Blue, 10);
        chart.add_dataset(dataset);

        assert_eq!(chart.datasets.len(), 1);
        assert_eq!(chart.datasets[0].name, "Dataset1");
    }

    #[test]
    fn test_modern_chart_update_dataset() {
        let mut chart = ModernChart::new("Test Chart".to_string(), DataVisOptions::default());

        let dataset = TimeSeriesData::new("Dataset1".to_string(), Color::Blue, 10);
        chart.add_dataset(dataset);

        let updated = chart.update_dataset(0, 42.0);
        assert!(updated);
        assert_eq!(chart.datasets[0].points.len(), 1);
        assert_eq!(chart.datasets[0].points[0].y, 42.0);
    }

    #[test]
    fn test_modern_chart_bounds() {
        let chart = ModernChart::new("Test Chart".to_string(), DataVisOptions::default());

        // Empty chart should return default bounds
        let (y_min, y_max) = chart.get_y_bounds();
        assert_eq!(y_min, 0.0);
        assert_eq!(y_max, 100.0);

        let (x_min, x_max) = chart.get_x_bounds();
        assert_eq!(x_min, 0.0);
        assert_eq!(x_max, 1.0);
    }

    #[test]
    fn test_modern_chart_events() {
        let mut chart = ModernChart::new("Test Chart".to_string(), DataVisOptions::default());

        // Test clear command
        let msg = chart.on(Event::Keyboard(KeyEvent::new(
            Key::Char('c'),
            KeyModifiers::NONE,
        )));

        assert!(matches!(
            msg,
            Some(Msg::User(UserEvent::TaskCompleted { .. }))
        ));

        // Test toggle legend
        let initial_legend = chart.options.show_legend;
        chart.on(Event::Keyboard(KeyEvent::new(
            Key::Function(8),
            KeyModifiers::NONE,
        )));
        assert_ne!(chart.options.show_legend, initial_legend);
    }

    #[test]
    fn test_modern_sparkline_creation() {
        let sparkline = ModernSparkline::new("Test".to_string(), 100, Color::Green);
        assert_eq!(sparkline.title, "Test");
        assert_eq!(sparkline.max, 100);
        assert_eq!(sparkline.color, Color::Green);
        assert!(sparkline.data.is_empty());
    }

    #[test]
    fn test_modern_sparkline_add_data() {
        let mut sparkline = ModernSparkline::new("Test".to_string(), 100, Color::Green);

        sparkline.add_data(50);
        assert_eq!(sparkline.data.len(), 1);
        assert_eq!(sparkline.data[0], 50);

        sparkline.add_data(75);
        assert_eq!(sparkline.data.len(), 2);
        assert_eq!(sparkline.data[1], 75);
    }

    #[test]
    fn test_modern_sparkline_max_points() {
        let mut sparkline = ModernSparkline::new("Test".to_string(), 100, Color::Green);

        // Add more than 40 points
        for i in 0..45 {
            sparkline.add_data(i as u64);
        }

        assert_eq!(sparkline.data.len(), 40);
        assert_eq!(sparkline.data[0], 5); // First 5 points removed
    }

    #[test]
    fn test_mock_component_perform() {
        let mut sparkline = ModernSparkline::new("Test".to_string(), 100, Color::Green);

        // Test add command
        let result = sparkline.perform(Cmd::Custom("add"));
        // Should not add without setting value attribute
        assert!(matches!(result, CmdResult::None));

        // Test clear command
        sparkline.add_data(50);
        sparkline.perform(Cmd::Custom("clear"));
        assert!(sparkline.data.is_empty());
    }
}
