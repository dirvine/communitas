//! Remote cursors overlay for showing collaborators on the canvas.

use communitas_ui_api::RemoteCursor;
use dioxus::prelude::*;

/// Props for the RemoteCursors component.
#[derive(Props, Clone, PartialEq)]
pub struct RemoteCursorsProps {
    /// List of remote cursors to display.
    pub cursors: Vec<RemoteCursor>,
    /// Viewport width for bounds checking.
    #[props(default = 800.0)]
    pub viewport_width: f32,
    /// Viewport height for bounds checking.
    #[props(default = 600.0)]
    pub viewport_height: f32,
    /// Current timestamp for activity filtering (Unix epoch ms).
    #[props(default = 0)]
    pub current_time: i64,
    /// Inactivity timeout in milliseconds (cursors fade after this).
    #[props(default = 5000)]
    pub inactivity_timeout: i64,
}

/// Overlay showing other users' cursor positions on the canvas.
#[component]
pub fn RemoteCursors(props: RemoteCursorsProps) -> Element {
    // Filter out inactive cursors (older than timeout)
    let active_cursors: Vec<_> = props
        .cursors
        .iter()
        .filter(|c| {
            if props.current_time == 0 {
                return true; // No time filtering if current_time not provided
            }
            props.current_time - c.last_active < props.inactivity_timeout
        })
        .cloned()
        .collect();

    if active_cursors.is_empty() {
        return rsx! {};
    }

    rsx! {
        svg {
            class: "remote-cursors absolute inset-0 pointer-events-none",
            view_box: format!("0 0 {} {}", props.viewport_width, props.viewport_height),
            xmlns: "http://www.w3.org/2000/svg",
            for cursor in active_cursors.iter() {
                RemoteCursorMarker {
                    key: "{cursor.user_id}",
                    cursor: cursor.clone(),
                    opacity: calculate_opacity(cursor.last_active, props.current_time, props.inactivity_timeout),
                }
            }
        }
        // Cursor list for accessibility
        div {
            class: "sr-only",
            role: "status",
            aria_live: "polite",
            {
                let count = active_cursors.len();
                let s = if count != 1 { "s" } else { "" };
                format!("{} collaborator{} active", count, s)
            }
        }
    }
}

/// Calculate opacity based on last activity time.
#[allow(dead_code)]
fn calculate_opacity(last_active: i64, current_time: i64, timeout: i64) -> f32 {
    if current_time == 0 {
        return 1.0;
    }

    let elapsed = current_time - last_active;
    if elapsed < 0 {
        return 1.0;
    }

    // Fade out over the last 2 seconds of the timeout
    let fade_start = timeout - 2000;
    if elapsed < fade_start {
        1.0
    } else {
        let fade_progress = (elapsed - fade_start) as f32 / 2000.0;
        (1.0 - fade_progress).max(0.2)
    }
}

/// Props for individual cursor marker.
#[derive(Props, Clone, PartialEq)]
struct RemoteCursorMarkerProps {
    cursor: RemoteCursor,
    opacity: f32,
}

/// Individual remote cursor display.
#[component]
fn RemoteCursorMarker(props: RemoteCursorMarkerProps) -> Element {
    let c = &props.cursor;

    // Cursor arrow path (pointing top-left)
    let cursor_path = "M0,0 L0,14 L4,10 L7,17 L10,16 L7,9 L12,9 Z";

    // Label offset from cursor tip
    let label_x = c.x + 15.0;
    let label_y = c.y + 20.0;

    rsx! {
        g {
            class: "remote-cursor-marker transition-opacity",
            opacity: "{props.opacity}",
            // Cursor pointer
            g {
                transform: format!("translate({}, {})", c.x, c.y),
                path {
                    d: "{cursor_path}",
                    fill: "{c.color}",
                    stroke: "#ffffff",
                    stroke_width: "1",
                }
            }
            // User name label
            g {
                transform: format!("translate({}, {})", label_x, label_y),
                rect {
                    x: "-2",
                    y: "-10",
                    width: "{c.user_name.len() as f32 * 7.0 + 8.0}",
                    height: "14",
                    rx: "3",
                    fill: "{c.color}",
                }
                text {
                    x: "2",
                    y: "0",
                    font_size: "10",
                    font_family: "system-ui, sans-serif",
                    fill: "#ffffff",
                    font_weight: "500",
                    "{c.user_name}"
                }
            }
            // Tool indicator
            if let Some(tool) = &c.tool {
                g {
                    transform: format!("translate({}, {})", c.x + 8.0, c.y - 5.0),
                    circle {
                        r: "6",
                        fill: "#1e293b",
                        stroke: "{c.color}",
                        stroke_width: "1",
                    }
                    text {
                        x: "0",
                        y: "3",
                        font_size: "8",
                        text_anchor: "middle",
                        fill: "#ffffff",
                        "{tool_abbreviation(tool)}"
                    }
                }
            }
        }
    }
}

/// Get short abbreviation for tool name.
#[allow(dead_code)]
fn tool_abbreviation(tool: &str) -> &str {
    match tool.to_lowercase().as_str() {
        "select" => "S",
        "pen" => "P",
        "brush" => "B",
        "eraser" => "E",
        "rectangle" => "R",
        "ellipse" => "O",
        "text" => "T",
        "pan" => "H",
        "zoom" => "Z",
        _ => "?",
    }
}

/// Mini avatar list showing active collaborators.
#[derive(Props, Clone, PartialEq)]
pub struct CollaboratorListProps {
    /// List of remote cursors (users).
    pub cursors: Vec<RemoteCursor>,
    /// Maximum number of avatars to show.
    #[props(default = 5)]
    pub max_visible: usize,
}

#[component]
pub fn CollaboratorList(props: CollaboratorListProps) -> Element {
    let visible_cursors: Vec<_> = props.cursors.iter().take(props.max_visible).collect();
    let overflow_count = props.cursors.len().saturating_sub(props.max_visible);

    if props.cursors.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "collaborator-list flex items-center -space-x-2",
            role: "list",
            aria_label: "Active collaborators",
            for cursor in visible_cursors.iter() {
                div {
                    key: "{cursor.user_id}",
                    class: "w-8 h-8 rounded-full border-2 border-slate-800 flex items-center justify-center text-xs font-medium text-white",
                    style: "background-color: {cursor.color}",
                    title: "{cursor.user_name}",
                    role: "listitem",
                    "{initials(&cursor.user_name)}"
                }
            }
            if overflow_count > 0 {
                div {
                    class: "w-8 h-8 rounded-full border-2 border-slate-800 bg-slate-600 flex items-center justify-center text-xs font-medium text-white",
                    title: "{overflow_count} more collaborators",
                    "+{overflow_count}"
                }
            }
        }
    }
}

/// Get initials from a name.
#[allow(dead_code)]
fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn make_cursor(user_id: &str, name: &str, x: f32, y: f32, color: &str) -> RemoteCursor {
        RemoteCursor {
            user_id: user_id.to_string(),
            user_name: name.to_string(),
            x,
            y,
            color: color.to_string(),
            last_active: 1000,
            tool: None,
        }
    }

    #[test]
    fn initials_extraction() {
        assert_eq!(initials("Alice Smith"), "AS");
        assert_eq!(initials("Bob"), "B");
        assert_eq!(initials("Charlie David Evans"), "CD");
        assert_eq!(initials(""), "");
    }

    #[test]
    fn tool_abbreviations() {
        assert_eq!(tool_abbreviation("select"), "S");
        assert_eq!(tool_abbreviation("Pen"), "P");
        assert_eq!(tool_abbreviation("BRUSH"), "B");
        assert_eq!(tool_abbreviation("unknown"), "?");
    }

    #[test]
    fn opacity_calculation() {
        let timeout = 5000_i64;

        // Recent activity - full opacity
        let opacity = calculate_opacity(4000, 4500, timeout);
        assert!((opacity - 1.0).abs() < f32::EPSILON);

        // No current time - full opacity
        let opacity = calculate_opacity(1000, 0, timeout);
        assert!((opacity - 1.0).abs() < f32::EPSILON);

        // Old activity - faded
        let opacity = calculate_opacity(0, 5000, timeout);
        assert!(opacity < 1.0);
        assert!(opacity >= 0.2);
    }

    #[test]
    fn inactive_cursor_filtering() {
        let current_time = 10000_i64;
        let timeout = 5000_i64;

        let cursors = vec![
            RemoteCursor {
                user_id: "1".to_string(),
                user_name: "Active".to_string(),
                x: 0.0,
                y: 0.0,
                color: "#ff0000".to_string(),
                last_active: 8000, // 2 seconds ago
                tool: None,
            },
            RemoteCursor {
                user_id: "2".to_string(),
                user_name: "Inactive".to_string(),
                x: 0.0,
                y: 0.0,
                color: "#00ff00".to_string(),
                last_active: 2000, // 8 seconds ago
                tool: None,
            },
        ];

        let active: Vec<_> = cursors
            .iter()
            .filter(|c| current_time - c.last_active < timeout)
            .collect();

        assert_eq!(active.len(), 1);
        assert_eq!(active[0].user_name, "Active");
    }

    #[test]
    fn label_width_calculation() {
        let name = "Alice";
        let width = name.len() as f32 * 7.0 + 8.0;
        assert!((width - 43.0).abs() < f32::EPSILON);
    }
}
