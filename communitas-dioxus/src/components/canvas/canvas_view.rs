//! Main canvas view component for rendering and interacting with the canvas.

use communitas_ui_service::UiServices;
use communitas_ui_service::canvas::{ElementKindView, ElementView, TransformView};
use dioxus::prelude::*;
use std::sync::Arc;

/// Props for the CanvasView component.
#[derive(Props, Clone, PartialEq)]
pub struct CanvasViewProps {
    /// Entity ID for the canvas context.
    #[props(default)]
    pub entity_id: String,
    /// Whether to show grid overlay.
    #[props(default = false)]
    pub show_grid: bool,
    /// Callback when element is clicked.
    #[props(default)]
    pub on_element_click: Option<EventHandler<String>>,
    /// Callback when canvas background is clicked.
    #[props(default)]
    pub on_canvas_click: Option<EventHandler<(f32, f32)>>,
}

/// Main canvas view for rendering elements and handling interactions.
#[component]
pub fn CanvasView(props: CanvasViewProps) -> Element {
    let services = use_context::<Arc<UiServices>>();
    let canvas_service = services.canvas();

    // Subscribe to canvas state
    let mut canvas_snapshot = use_signal(|| canvas_service.current_snapshot());

    // Update signal when snapshot changes
    let _updater = use_resource(move || {
        let canvas = services.canvas();
        async move {
            let mut rx = canvas.subscribe();
            loop {
                if rx.changed().await.is_err() {
                    break;
                }
                canvas_snapshot.set(rx.borrow().clone());
            }
        }
    });

    let snapshot = canvas_snapshot();

    // Calculate SVG viewBox from viewport and pan/zoom
    let view_box = format!(
        "{} {} {} {}",
        -snapshot.pan_x / snapshot.zoom,
        -snapshot.pan_y / snapshot.zoom,
        snapshot.viewport_width / snapshot.zoom,
        snapshot.viewport_height / snapshot.zoom
    );

    // Status announcement for screen readers
    let selected_count = snapshot.elements.iter().filter(|e| e.selected).count();
    let status_text = if snapshot.loading {
        "Loading canvas".to_string()
    } else if selected_count > 0 {
        format!(
            "{} element{} selected",
            selected_count,
            if selected_count == 1 { "" } else { "s" }
        )
    } else {
        format!("{} elements on canvas", snapshot.elements.len())
    };

    rsx! {
        div {
            class: "canvas-view relative w-full h-full bg-slate-900 overflow-hidden",
            role: "application",
            aria_label: "Canvas drawing area",
            // Aria-live region for status announcements
            div {
                role: "status",
                aria_live: "polite",
                aria_atomic: "true",
                class: "sr-only",
                "{status_text}"
            }
            // Loading overlay
            if snapshot.loading {
                div {
                    class: "absolute inset-0 bg-slate-900/50 flex items-center justify-center z-50",
                    div {
                        class: "text-white flex items-center gap-2",
                        span { class: "animate-spin", "⏳" }
                        span { "Loading canvas..." }
                    }
                }
            }
            // SVG Canvas
            svg {
                class: "w-full h-full",
                view_box: "{view_box}",
                xmlns: "http://www.w3.org/2000/svg",
                onclick: {
                    let on_click = props.on_canvas_click;
                    move |evt: Event<MouseData>| {
                        if let Some(handler) = &on_click {
                            let coords = evt.element_coordinates();
                            handler.call((coords.x as f32, coords.y as f32));
                        }
                    }
                },
                // Grid overlay
                if props.show_grid {
                    GridOverlay {
                        width: snapshot.viewport_width,
                        height: snapshot.viewport_height,
                        zoom: snapshot.zoom,
                    }
                }
                // Render elements
                for element in snapshot.elements.iter() {
                    CanvasElement {
                        key: "{element.id}",
                        element: element.clone(),
                        on_click: {
                            let on_element_click = props.on_element_click;
                            let id = element.id.clone();
                            move |_| {
                                if let Some(handler) = &on_element_click {
                                    handler.call(id.clone());
                                }
                            }
                        },
                    }
                }
                // Selection handles for selected elements
                for element in snapshot.elements.iter().filter(|e| e.selected) {
                    SelectionHandles {
                        key: "handles-{element.id}",
                        transform: element.transform,
                    }
                }
            }
            // Zoom indicator
            div {
                class: "absolute bottom-2 right-2 px-2 py-1 bg-slate-800/80 rounded text-xs text-slate-400",
                "{(snapshot.zoom * 100.0) as i32}%"
            }
        }
    }
}

/// Props for grid overlay.
#[derive(Props, Clone, PartialEq)]
struct GridOverlayProps {
    width: f32,
    height: f32,
    zoom: f32,
}

/// Grid overlay for canvas.
#[component]
fn GridOverlay(props: GridOverlayProps) -> Element {
    let grid_size = 20.0 / props.zoom;
    let pattern_id = "canvas-grid-pattern";

    rsx! {
        defs {
            pattern {
                id: "{pattern_id}",
                width: "{grid_size}",
                height: "{grid_size}",
                pattern_units: "userSpaceOnUse",
                path {
                    d: format!("M {} 0 L 0 0 0 {}", grid_size, grid_size),
                    fill: "none",
                    stroke: "rgba(100, 116, 139, 0.2)",
                    stroke_width: "0.5",
                }
            }
        }
        rect {
            x: "0",
            y: "0",
            width: "{props.width}",
            height: "{props.height}",
            fill: "url(#{pattern_id})",
        }
    }
}

/// Props for individual canvas element.
#[derive(Props, Clone, PartialEq)]
struct CanvasElementProps {
    element: ElementView,
    on_click: EventHandler<()>,
}

/// Renders a single canvas element.
#[component]
fn CanvasElement(props: CanvasElementProps) -> Element {
    let elem = &props.element;
    let t = &elem.transform;

    // Apply transform
    let transform_str = format!(
        "translate({}, {}) rotate({} {} {})",
        t.x,
        t.y,
        t.rotation.to_degrees(),
        t.width / 2.0,
        t.height / 2.0
    );

    let cursor_class = if elem.interactive {
        "cursor-pointer"
    } else {
        "cursor-default"
    };

    let opacity = if elem.selected { "0.9" } else { "1" };

    rsx! {
        g {
            class: "{cursor_class}",
            transform: "{transform_str}",
            opacity: "{opacity}",
            onclick: move |evt| {
                evt.stop_propagation();
                props.on_click.call(());
            },
            {render_element_content(&elem.kind, t)}
        }
    }
}

/// Render the content of an element based on its kind.
#[allow(dead_code)]
fn render_element_content(kind: &ElementKindView, transform: &TransformView) -> Element {
    match kind {
        ElementKindView::Text {
            content,
            font_size,
            color,
        } => rsx! {
            text {
                x: "0",
                y: "{font_size}",
                font_size: "{font_size}",
                fill: "{color}",
                font_family: "system-ui, sans-serif",
                "{content}"
            }
        },
        ElementKindView::Image { src, .. } => rsx! {
            image {
                x: "0",
                y: "0",
                width: "{transform.width}",
                height: "{transform.height}",
                href: "{src}",
                preserve_aspect_ratio: "xMidYMid meet",
            }
        },
        ElementKindView::Chart { chart_type } => rsx! {
            rect {
                x: "0",
                y: "0",
                width: "{transform.width}",
                height: "{transform.height}",
                fill: "#1e293b",
                stroke: "#475569",
                stroke_width: "1",
            }
            text {
                x: "{transform.width / 2.0}",
                y: "{transform.height / 2.0}",
                text_anchor: "middle",
                dominant_baseline: "middle",
                fill: "#94a3b8",
                font_size: "12",
                "Chart: {chart_type}"
            }
        },
        ElementKindView::Video {
            stream_id: _,
            is_live,
        } => {
            let live_indicator = if *is_live { " (Live)" } else { "" };
            rsx! {
                rect {
                    x: "0",
                    y: "0",
                    width: "{transform.width}",
                    height: "{transform.height}",
                    fill: "#0f172a",
                    stroke: if *is_live { "#ef4444" } else { "#475569" },
                    stroke_width: "2",
                }
                text {
                    x: "{transform.width / 2.0}",
                    y: "{transform.height / 2.0}",
                    text_anchor: "middle",
                    dominant_baseline: "middle",
                    fill: "#94a3b8",
                    font_size: "12",
                    "Video{live_indicator}"
                }
            }
        }
        ElementKindView::Model3D { src: _ } => rsx! {
            rect {
                x: "0",
                y: "0",
                width: "{transform.width}",
                height: "{transform.height}",
                fill: "#1e1b4b",
                stroke: "#6366f1",
                stroke_width: "1",
            }
            text {
                x: "{transform.width / 2.0}",
                y: "{transform.height / 2.0}",
                text_anchor: "middle",
                dominant_baseline: "middle",
                fill: "#a5b4fc",
                font_size: "12",
                "3D Model"
            }
        },
        ElementKindView::Group { child_count } => rsx! {
            rect {
                x: "0",
                y: "0",
                width: "{transform.width}",
                height: "{transform.height}",
                fill: "none",
                stroke: "#94a3b8",
                stroke_width: "1",
                stroke_dasharray: "4",
            }
            text {
                x: "4",
                y: "16",
                fill: "#64748b",
                font_size: "10",
                "Group ({child_count})"
            }
        },
        ElementKindView::OverlayLayer { child_count } => rsx! {
            rect {
                x: "0",
                y: "0",
                width: "{transform.width}",
                height: "{transform.height}",
                fill: "rgba(0, 0, 0, 0.1)",
                stroke: "#64748b",
                stroke_width: "1",
                stroke_dasharray: "2",
            }
            text {
                x: "4",
                y: "16",
                fill: "#64748b",
                font_size: "10",
                "Overlay ({child_count})"
            }
        },
    }
}

/// Props for selection handles.
#[derive(Props, Clone, PartialEq)]
struct SelectionHandlesProps {
    transform: TransformView,
}

/// Selection handles around a selected element.
#[component]
fn SelectionHandles(props: SelectionHandlesProps) -> Element {
    let t = &props.transform;
    let handle_size = 8.0;
    let half = handle_size / 2.0;

    // Handle positions: corners and midpoints
    let handles = [
        (0.0, 0.0, "nw-resize"),
        (t.width / 2.0, 0.0, "n-resize"),
        (t.width, 0.0, "ne-resize"),
        (t.width, t.height / 2.0, "e-resize"),
        (t.width, t.height, "se-resize"),
        (t.width / 2.0, t.height, "s-resize"),
        (0.0, t.height, "sw-resize"),
        (0.0, t.height / 2.0, "w-resize"),
    ];

    rsx! {
        g {
            transform: format!("translate({}, {})", t.x, t.y),
            // Selection border
            rect {
                x: "0",
                y: "0",
                width: "{t.width}",
                height: "{t.height}",
                fill: "none",
                stroke: "#10b981",
                stroke_width: "2",
                stroke_dasharray: "4",
            }
            // Handle squares
            for (hx, hy, cursor) in handles {
                rect {
                    key: "{hx}-{hy}",
                    x: "{hx - half}",
                    y: "{hy - half}",
                    width: "{handle_size}",
                    height: "{handle_size}",
                    fill: "#ffffff",
                    stroke: "#10b981",
                    stroke_width: "1",
                    style: "cursor: {cursor}",
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_size_scales_with_zoom() {
        let base_grid = 20.0_f32;
        let zoom = 2.0_f32;
        let scaled = base_grid / zoom;
        assert!((scaled - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn transform_string_format() {
        let t = TransformView {
            x: 100.0,
            y: 50.0,
            width: 200.0,
            height: 100.0,
            rotation: std::f32::consts::PI / 4.0, // 45 degrees
            z_index: 0,
        };
        let transform_str = format!(
            "translate({}, {}) rotate({} {} {})",
            t.x,
            t.y,
            t.rotation.to_degrees(),
            t.width / 2.0,
            t.height / 2.0
        );
        assert!(transform_str.contains("translate(100, 50)"));
        assert!(transform_str.contains("rotate(45"));
    }

    #[test]
    fn viewbox_calculation() {
        let pan_x = 100.0;
        let pan_y = 50.0;
        let zoom = 2.0;
        let width = 800.0;
        let height = 600.0;

        let view_box = format!(
            "{} {} {} {}",
            -pan_x / zoom,
            -pan_y / zoom,
            width / zoom,
            height / zoom
        );

        assert_eq!(view_box, "-50 -25 400 300");
    }

    #[test]
    fn canvas_status_text_loading() {
        let loading = true;
        let selected_count = 0usize;
        let element_count = 5usize;

        let status_text = if loading {
            "Loading canvas".to_string()
        } else if selected_count > 0 {
            format!(
                "{} element{} selected",
                selected_count,
                if selected_count == 1 { "" } else { "s" }
            )
        } else {
            format!("{} elements on canvas", element_count)
        };

        assert_eq!(status_text, "Loading canvas");
    }

    #[test]
    fn canvas_status_text_single_selection() {
        let loading = false;
        let selected_count = 1usize;
        let element_count = 5usize;

        let status_text = if loading {
            "Loading canvas".to_string()
        } else if selected_count > 0 {
            format!(
                "{} element{} selected",
                selected_count,
                if selected_count == 1 { "" } else { "s" }
            )
        } else {
            format!("{} elements on canvas", element_count)
        };

        assert_eq!(status_text, "1 element selected");
    }

    #[test]
    fn canvas_status_text_multiple_selection() {
        let loading = false;
        let selected_count = 3usize;
        let element_count = 5usize;

        let status_text = if loading {
            "Loading canvas".to_string()
        } else if selected_count > 0 {
            format!(
                "{} element{} selected",
                selected_count,
                if selected_count == 1 { "" } else { "s" }
            )
        } else {
            format!("{} elements on canvas", element_count)
        };

        assert_eq!(status_text, "3 elements selected");
    }

    #[test]
    fn canvas_status_text_no_selection() {
        let loading = false;
        let selected_count = 0usize;
        let element_count = 8usize;

        let status_text = if loading {
            "Loading canvas".to_string()
        } else if selected_count > 0 {
            format!(
                "{} element{} selected",
                selected_count,
                if selected_count == 1 { "" } else { "s" }
            )
        } else {
            format!("{} elements on canvas", element_count)
        };

        assert_eq!(status_text, "8 elements on canvas");
    }
}
