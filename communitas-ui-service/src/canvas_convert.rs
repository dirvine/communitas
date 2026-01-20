//! Type conversions between core canvas types and UI canvas types.
//!
//! This module provides conversion functions for translating between the internal
//! `communitas_core` command/response types used for persistence, and the
//! `communitas_ui_api` types used by UI components.
//!
//! Note: Rust's orphan rules prevent implementing `From` traits here since both
//! source and target types are defined in external crates. Use the explicit
//! conversion functions instead.

use communitas_core::command::{CanvasElementResponse, CanvasSnapshotResponse};
use communitas_ui_api::canvas::{
    CanvasElement, CanvasInfo, CanvasSnapshot, CanvasState, ElementType, Layer, ToolType, Transform,
};

/// Convert a core `CanvasSnapshotResponse` to a UI `CanvasSnapshot`.
///
/// Creates a UI snapshot with default collaborative state (no remote cursors,
/// empty history, no offline operations). The core response only contains
/// element and viewport data.
#[must_use]
pub fn core_snapshot_to_ui(snap: &CanvasSnapshotResponse) -> CanvasSnapshot {
    let elements: Vec<CanvasElement> = snap.elements.iter().map(core_element_to_ui).collect();

    // Create a default layer since core doesn't track layers
    let default_layer = Layer {
        id: "default".to_string(),
        name: "Layer 1".to_string(),
        visible: true,
        locked: false,
        opacity: 1.0,
        z_index: 0,
    };

    let canvas_info = CanvasInfo {
        id: snap.entity_id.clone(),
        entity_id: snap.entity_id.clone(),
        name: String::new(), // Core doesn't provide name
        width: snap.viewport_width as u32,
        height: snap.viewport_height as u32,
        created_at: 0, // Core doesn't track timestamps
        updated_at: 0,
    };

    let state = CanvasState {
        canvas_info,
        elements,
        layers: vec![default_layer],
        active_layer_id: "default".to_string(),
    };

    CanvasSnapshot {
        state,
        remote_cursors: Vec::new(),
        history: Vec::new(),
        offline_operations: Vec::new(),
        selected_tool: ToolType::default(),
        stroke_color: "#000000".to_string(),
        fill_color: None,
        stroke_width: 2.0,
    }
}

/// Convert a core `CanvasElementResponse` to a UI `CanvasElement`.
///
/// Maps the flat element response from the core to the structured UI element type.
/// The `element_type` string is parsed to determine the appropriate variant.
#[must_use]
pub fn core_element_to_ui(elem: &CanvasElementResponse) -> CanvasElement {
    let element_type = parse_element_type(elem);
    let transform = if elem.rotation != 0.0 {
        Some(Transform {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: elem.rotation,
        })
    } else {
        None
    };

    CanvasElement {
        id: elem.id.clone(),
        element_type,
        layer_id: "default".to_string(), // Core doesn't track layers
        z_index: elem.z_index,
        opacity: 1.0, // Core doesn't track opacity per element
        transform,
        created_by: String::new(), // Core doesn't track creator
        created_at: 0,             // Core doesn't track timestamps
    }
}

/// Parse the element type string and data to create the appropriate UI ElementType.
fn parse_element_type(elem: &CanvasElementResponse) -> ElementType {
    // Extract content/src from the data JSON value
    let content = elem
        .data
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let src = elem
        .data
        .get("src")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let font_size = elem
        .data
        .get("font_size")
        .and_then(|v| v.as_f64())
        .unwrap_or(16.0) as f32;
    let color = elem
        .data
        .get("color")
        .and_then(|v| v.as_str())
        .unwrap_or("#000000")
        .to_string();

    match elem.element_type.as_str() {
        "text" => ElementType::Text {
            x: elem.x,
            y: elem.y,
            content,
            font_size,
            color,
        },
        "image" => ElementType::Image {
            x: elem.x,
            y: elem.y,
            width: elem.width,
            height: elem.height,
            data_url: src,
        },
        "rectangle" | "rect" => ElementType::Rectangle {
            x: elem.x,
            y: elem.y,
            width: elem.width,
            height: elem.height,
            fill: Some("#ffffff".to_string()),
            stroke: Some("#000000".to_string()),
        },
        "ellipse" | "circle" => ElementType::Ellipse {
            cx: elem.x + elem.width / 2.0,
            cy: elem.y + elem.height / 2.0,
            rx: elem.width / 2.0,
            ry: elem.height / 2.0,
            fill: Some("#ffffff".to_string()),
            stroke: Some("#000000".to_string()),
        },
        _ => {
            // Default to text for unknown types
            ElementType::Text {
                x: elem.x,
                y: elem.y,
                content,
                font_size: 16.0,
                color: "#000000".to_string(),
            }
        }
    }
}

/// Convert a UI `CanvasElement` to parameters for core canvas commands.
///
/// Returns a tuple of (element_type, x, y, width, height, content) for use
/// with core commands.
#[must_use]
pub fn ui_element_to_core_params(
    elem: &CanvasElement,
) -> (String, f32, f32, f32, f32, Option<String>) {
    match &elem.element_type {
        ElementType::Text {
            x,
            y,
            content,
            font_size: _,
            color: _,
        } => ("text".to_string(), *x, *y, 0.0, 0.0, Some(content.clone())),
        ElementType::Image {
            x,
            y,
            width,
            height,
            data_url,
        } => (
            "image".to_string(),
            *x,
            *y,
            *width,
            *height,
            Some(data_url.clone()),
        ),
        ElementType::Rectangle {
            x,
            y,
            width,
            height,
            ..
        } => ("rectangle".to_string(), *x, *y, *width, *height, None),
        ElementType::Ellipse { cx, cy, rx, ry, .. } => {
            // Convert center+radii to top-left+dimensions
            let x = cx - rx;
            let y = cy - ry;
            ("ellipse".to_string(), x, y, rx * 2.0, ry * 2.0, None)
        }
        ElementType::Path {
            points,
            stroke_width: _,
            color: _,
        } => {
            // Serialize points to JSON for content
            let content = serde_json::to_string(points).ok();
            ("path".to_string(), 0.0, 0.0, 0.0, 0.0, content)
        }
    }
}

/// Extract viewport parameters from a UI `CanvasSnapshot`.
#[must_use]
pub fn ui_snapshot_viewport(snap: &CanvasSnapshot) -> (f32, f32, f32, f32, f32) {
    let info = &snap.state.canvas_info;
    (
        info.width as f32,
        info.height as f32,
        1.0, // Default zoom
        0.0, // Default pan_x
        0.0, // Default pan_y
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use communitas_ui_api::canvas::Point;
    use serde_json::json;

    fn make_test_element_response() -> CanvasElementResponse {
        CanvasElementResponse {
            id: "elem-123".to_string(),
            element_type: "text".to_string(),
            x: 100.0,
            y: 200.0,
            width: 150.0,
            height: 50.0,
            rotation: 0.0,
            z_index: 1,
            selected: false,
            interactive: true,
            data: json!({"content": "Hello, World!", "font_size": 16.0, "color": "#000000"}),
        }
    }

    fn make_test_snapshot_response() -> CanvasSnapshotResponse {
        CanvasSnapshotResponse {
            entity_id: "entity-456".to_string(),
            elements: vec![make_test_element_response()],
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            loading: false,
        }
    }

    #[test]
    fn test_core_element_to_ui_text() {
        let core_elem = make_test_element_response();
        let ui_elem = core_element_to_ui(&core_elem);

        assert_eq!(ui_elem.id, "elem-123");
        assert_eq!(ui_elem.z_index, 1);
        assert_eq!(ui_elem.layer_id, "default");

        if let ElementType::Text { x, y, content, .. } = ui_elem.element_type {
            assert_eq!(x, 100.0);
            assert_eq!(y, 200.0);
            assert_eq!(content, "Hello, World!");
        } else {
            panic!("Expected Text element type");
        }
    }

    #[test]
    fn test_core_element_to_ui_image() {
        let core_elem = CanvasElementResponse {
            id: "img-1".to_string(),
            element_type: "image".to_string(),
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 100.0,
            rotation: 0.0,
            z_index: 2,
            selected: false,
            interactive: true,
            data: json!({"src": "data:image/png;base64,abc123"}),
        };
        let ui_elem = core_element_to_ui(&core_elem);

        if let ElementType::Image {
            x,
            y,
            width,
            height,
            data_url,
        } = ui_elem.element_type
        {
            assert_eq!(x, 50.0);
            assert_eq!(y, 50.0);
            assert_eq!(width, 200.0);
            assert_eq!(height, 100.0);
            assert_eq!(data_url, "data:image/png;base64,abc123");
        } else {
            panic!("Expected Image element type");
        }
    }

    #[test]
    fn test_core_element_to_ui_rectangle() {
        let core_elem = CanvasElementResponse {
            id: "rect-1".to_string(),
            element_type: "rectangle".to_string(),
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            rotation: 0.0,
            z_index: 0,
            selected: false,
            interactive: true,
            data: json!({}),
        };
        let ui_elem = core_element_to_ui(&core_elem);

        if let ElementType::Rectangle {
            x,
            y,
            width,
            height,
            ..
        } = ui_elem.element_type
        {
            assert_eq!(x, 10.0);
            assert_eq!(y, 20.0);
            assert_eq!(width, 100.0);
            assert_eq!(height, 50.0);
        } else {
            panic!("Expected Rectangle element type");
        }
    }

    #[test]
    fn test_core_element_to_ui_ellipse() {
        let core_elem = CanvasElementResponse {
            id: "ellipse-1".to_string(),
            element_type: "ellipse".to_string(),
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 80.0,
            rotation: 0.0,
            z_index: 0,
            selected: false,
            interactive: true,
            data: json!({}),
        };
        let ui_elem = core_element_to_ui(&core_elem);

        if let ElementType::Ellipse { cx, cy, rx, ry, .. } = ui_elem.element_type {
            // Center should be at (50, 40) for element at (0,0) with size (100, 80)
            assert_eq!(cx, 50.0);
            assert_eq!(cy, 40.0);
            assert_eq!(rx, 50.0);
            assert_eq!(ry, 40.0);
        } else {
            panic!("Expected Ellipse element type");
        }
    }

    #[test]
    fn test_core_element_to_ui_with_rotation() {
        let core_elem = CanvasElementResponse {
            id: "rotated-1".to_string(),
            element_type: "text".to_string(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            rotation: 1.57, // ~90 degrees
            z_index: 0,
            selected: false,
            interactive: true,
            data: json!({"content": "Rotated"}),
        };
        let ui_elem = core_element_to_ui(&core_elem);

        assert!(ui_elem.transform.is_some());
        let transform = ui_elem.transform.unwrap();
        assert!((transform.rotation - 1.57).abs() < 0.01);
    }

    #[test]
    fn test_core_element_to_ui_no_rotation() {
        let core_elem = make_test_element_response();
        let ui_elem = core_element_to_ui(&core_elem);

        // No rotation means no transform needed
        assert!(ui_elem.transform.is_none());
    }

    #[test]
    fn test_core_snapshot_to_ui() {
        let core_snap = make_test_snapshot_response();
        let ui_snap = core_snapshot_to_ui(&core_snap);

        assert_eq!(ui_snap.state.canvas_info.entity_id, "entity-456");
        assert_eq!(ui_snap.state.canvas_info.width, 1920);
        assert_eq!(ui_snap.state.canvas_info.height, 1080);
        assert_eq!(ui_snap.state.elements.len(), 1);
        assert_eq!(ui_snap.state.layers.len(), 1);
        assert_eq!(ui_snap.state.active_layer_id, "default");
        assert!(ui_snap.remote_cursors.is_empty());
        assert!(ui_snap.history.is_empty());
        assert!(ui_snap.offline_operations.is_empty());
    }

    #[test]
    fn test_core_snapshot_to_ui_empty() {
        let core_snap = CanvasSnapshotResponse {
            entity_id: "empty-canvas".to_string(),
            elements: vec![],
            viewport_width: 800.0,
            viewport_height: 600.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            loading: false,
        };
        let ui_snap = core_snapshot_to_ui(&core_snap);

        assert!(ui_snap.state.elements.is_empty());
        assert_eq!(ui_snap.state.canvas_info.width, 800);
        assert_eq!(ui_snap.state.canvas_info.height, 600);
    }

    #[test]
    fn test_ui_element_to_core_params_text() {
        let elem = CanvasElement {
            id: "text-1".to_string(),
            element_type: ElementType::Text {
                x: 10.0,
                y: 20.0,
                content: "Test text".to_string(),
                font_size: 14.0,
                color: "#ff0000".to_string(),
            },
            layer_id: "default".to_string(),
            z_index: 0,
            opacity: 1.0,
            transform: None,
            created_by: String::new(),
            created_at: 0,
        };
        let (elem_type, x, y, w, h, content) = ui_element_to_core_params(&elem);

        assert_eq!(elem_type, "text");
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(w, 0.0);
        assert_eq!(h, 0.0);
        assert_eq!(content, Some("Test text".to_string()));
    }

    #[test]
    fn test_ui_element_to_core_params_rectangle() {
        let elem = CanvasElement {
            id: "rect-1".to_string(),
            element_type: ElementType::Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 50.0,
                fill: Some("#ffffff".to_string()),
                stroke: Some("#000000".to_string()),
            },
            layer_id: "default".to_string(),
            z_index: 0,
            opacity: 1.0,
            transform: None,
            created_by: String::new(),
            created_at: 0,
        };
        let (elem_type, _x, _y, w, h, content) = ui_element_to_core_params(&elem);

        assert_eq!(elem_type, "rectangle");
        assert_eq!(w, 100.0);
        assert_eq!(h, 50.0);
        assert!(content.is_none());
    }

    #[test]
    fn test_ui_element_to_core_params_ellipse() {
        let elem = CanvasElement {
            id: "ellipse-1".to_string(),
            element_type: ElementType::Ellipse {
                cx: 100.0,
                cy: 100.0,
                rx: 50.0,
                ry: 30.0,
                fill: None,
                stroke: Some("#0000ff".to_string()),
            },
            layer_id: "default".to_string(),
            z_index: 0,
            opacity: 1.0,
            transform: None,
            created_by: String::new(),
            created_at: 0,
        };
        let (elem_type, x, y, w, h, _content) = ui_element_to_core_params(&elem);

        assert_eq!(elem_type, "ellipse");
        // Converted from center+radii to top-left+dimensions
        assert_eq!(x, 50.0); // cx - rx = 100 - 50
        assert_eq!(y, 70.0); // cy - ry = 100 - 30
        assert_eq!(w, 100.0); // rx * 2
        assert_eq!(h, 60.0); // ry * 2
    }

    #[test]
    fn test_ui_element_to_core_params_path() {
        let elem = CanvasElement {
            id: "path-1".to_string(),
            element_type: ElementType::Path {
                points: vec![
                    Point {
                        x: 0.0,
                        y: 0.0,
                        pressure: None,
                    },
                    Point {
                        x: 100.0,
                        y: 100.0,
                        pressure: Some(0.5),
                    },
                ],
                stroke_width: 2.0,
                color: "#000000".to_string(),
            },
            layer_id: "default".to_string(),
            z_index: 0,
            opacity: 1.0,
            transform: None,
            created_by: String::new(),
            created_at: 0,
        };
        let (elem_type, _, _, _, _, content) = ui_element_to_core_params(&elem);

        assert_eq!(elem_type, "path");
        assert!(content.is_some());
        // Content should be JSON serialized points
        let json = content.unwrap();
        assert!(json.contains("100.0"));
    }

    #[test]
    fn test_ui_snapshot_viewport() {
        let snap = CanvasSnapshot {
            state: CanvasState {
                canvas_info: CanvasInfo {
                    id: "canvas-1".to_string(),
                    entity_id: "entity-1".to_string(),
                    name: "Test Canvas".to_string(),
                    width: 1920,
                    height: 1080,
                    created_at: 0,
                    updated_at: 0,
                },
                elements: vec![],
                layers: vec![],
                active_layer_id: String::new(),
            },
            remote_cursors: vec![],
            history: vec![],
            offline_operations: vec![],
            selected_tool: ToolType::default(),
            stroke_color: String::new(),
            fill_color: None,
            stroke_width: 2.0,
        };

        let (width, height, zoom, pan_x, pan_y) = ui_snapshot_viewport(&snap);

        assert_eq!(width, 1920.0);
        assert_eq!(height, 1080.0);
        assert_eq!(zoom, 1.0);
        assert_eq!(pan_x, 0.0);
        assert_eq!(pan_y, 0.0);
    }

    #[test]
    fn test_unknown_element_type_defaults_to_text() {
        let core_elem = CanvasElementResponse {
            id: "unknown-1".to_string(),
            element_type: "unknown_type".to_string(),
            x: 10.0,
            y: 20.0,
            width: 0.0,
            height: 0.0,
            rotation: 0.0,
            z_index: 0,
            selected: false,
            interactive: true,
            data: json!({"content": "Fallback content", "font_size": 16.0, "color": "#000000"}),
        };
        let ui_elem = core_element_to_ui(&core_elem);

        if let ElementType::Text { content, .. } = ui_elem.element_type {
            assert_eq!(content, "Fallback content");
        } else {
            panic!("Expected Text fallback for unknown type");
        }
    }

    #[test]
    fn test_multiple_elements_preserve_order() {
        let core_snap = CanvasSnapshotResponse {
            entity_id: "multi".to_string(),
            elements: vec![
                CanvasElementResponse {
                    id: "first".to_string(),
                    element_type: "text".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    rotation: 0.0,
                    z_index: 0,
                    selected: false,
                    interactive: true,
                    data: json!({"content": "First", "font_size": 16.0, "color": "#000000"}),
                },
                CanvasElementResponse {
                    id: "second".to_string(),
                    element_type: "rectangle".to_string(),
                    x: 100.0,
                    y: 100.0,
                    width: 50.0,
                    height: 50.0,
                    rotation: 0.0,
                    z_index: 1,
                    selected: false,
                    interactive: true,
                    data: json!({}),
                },
            ],
            viewport_width: 800.0,
            viewport_height: 600.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            loading: false,
        };

        let ui_snap = core_snapshot_to_ui(&core_snap);

        assert_eq!(ui_snap.state.elements.len(), 2);
        assert_eq!(ui_snap.state.elements[0].id, "first");
        assert_eq!(ui_snap.state.elements[1].id, "second");
    }
}
