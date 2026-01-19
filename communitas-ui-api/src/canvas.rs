//! Canvas DTOs for collaborative drawing surfaces.

use serde::{Deserialize, Serialize};

/// Information about a canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasInfo {
    /// Unique canvas identifier.
    pub id: String,
    /// Entity this canvas belongs to.
    pub entity_id: String,
    /// Canvas display name.
    pub name: String,
    /// Canvas width in pixels.
    pub width: u32,
    /// Canvas height in pixels.
    pub height: u32,
    /// Creation timestamp (Unix epoch ms).
    pub created_at: i64,
    /// Last update timestamp (Unix epoch ms).
    pub updated_at: i64,
}

/// A 2D point with optional pressure data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Point {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
    /// Optional pressure value (0.0 to 1.0) for pen/stylus input.
    pub pressure: Option<f32>,
}

/// Transform applied to canvas elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    /// X translation offset.
    pub translate_x: f32,
    /// Y translation offset.
    pub translate_y: f32,
    /// X scale factor.
    pub scale_x: f32,
    /// Y scale factor.
    pub scale_y: f32,
    /// Rotation angle in radians.
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate_x: 0.0,
            translate_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }
}

/// Types of canvas elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementType {
    /// Freehand path/stroke.
    Path {
        /// Points along the path.
        points: Vec<Point>,
        /// Stroke width in pixels.
        stroke_width: f32,
        /// Stroke color (CSS color string).
        color: String,
    },
    /// Rectangle shape.
    Rectangle {
        /// X position of top-left corner.
        x: f32,
        /// Y position of top-left corner.
        y: f32,
        /// Width of rectangle.
        width: f32,
        /// Height of rectangle.
        height: f32,
        /// Fill color (CSS color string).
        fill: Option<String>,
        /// Stroke color (CSS color string).
        stroke: Option<String>,
    },
    /// Ellipse shape.
    Ellipse {
        /// Center X coordinate.
        cx: f32,
        /// Center Y coordinate.
        cy: f32,
        /// X radius.
        rx: f32,
        /// Y radius.
        ry: f32,
        /// Fill color (CSS color string).
        fill: Option<String>,
        /// Stroke color (CSS color string).
        stroke: Option<String>,
    },
    /// Text element.
    Text {
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Text content.
        content: String,
        /// Font size in pixels.
        font_size: f32,
        /// Text color (CSS color string).
        color: String,
    },
    /// Image element.
    Image {
        /// X position.
        x: f32,
        /// Y position.
        y: f32,
        /// Image width.
        width: f32,
        /// Image height.
        height: f32,
        /// Base64 data URL or external URL.
        data_url: String,
    },
}

/// A canvas element with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasElement {
    /// Unique element identifier.
    pub id: String,
    /// Element type and data.
    pub element_type: ElementType,
    /// Layer this element belongs to.
    pub layer_id: String,
    /// Z-order within the layer.
    pub z_index: i32,
    /// Element opacity (0.0 to 1.0).
    pub opacity: f32,
    /// Optional transform applied to the element.
    pub transform: Option<Transform>,
    /// User ID of the creator.
    pub created_by: String,
    /// Creation timestamp (Unix epoch ms).
    pub created_at: i64,
}

/// A canvas layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    /// Unique layer identifier.
    pub id: String,
    /// Layer display name.
    pub name: String,
    /// Whether the layer is visible.
    pub visible: bool,
    /// Whether the layer is locked for editing.
    pub locked: bool,
    /// Layer opacity (0.0 to 1.0).
    pub opacity: f32,
    /// Z-order of the layer.
    pub z_index: i32,
}

/// Complete canvas state for UI rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasState {
    /// Canvas metadata.
    pub canvas_info: CanvasInfo,
    /// All elements on the canvas.
    pub elements: Vec<CanvasElement>,
    /// All layers.
    pub layers: Vec<Layer>,
    /// Currently active layer ID.
    pub active_layer_id: String,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            canvas_info: CanvasInfo {
                id: String::new(),
                entity_id: String::new(),
                name: String::new(),
                width: 1920,
                height: 1080,
                created_at: 0,
                updated_at: 0,
            },
            elements: Vec::new(),
            layers: Vec::new(),
            active_layer_id: String::new(),
        }
    }
}

/// Remote user's cursor position for collaborative editing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteCursor {
    /// User ID of the cursor owner.
    pub user_id: String,
    /// User's display name.
    pub user_name: String,
    /// Cursor X position on canvas.
    pub x: f32,
    /// Cursor Y position on canvas.
    pub y: f32,
    /// User's assigned color (CSS color string).
    pub color: String,
    /// Last activity timestamp (Unix epoch ms).
    pub last_active: i64,
    /// Currently selected tool.
    pub tool: Option<String>,
}

/// Types of actions recorded in history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryActionType {
    /// Element added.
    AddElement,
    /// Element deleted.
    DeleteElement,
    /// Element modified.
    ModifyElement,
    /// Layer added.
    AddLayer,
    /// Layer deleted.
    DeleteLayer,
    /// Layer modified.
    ModifyLayer,
    /// Multiple operations grouped.
    BatchOperation,
}

impl HistoryActionType {
    /// Human-readable label for the action type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::AddElement => "Add Element",
            Self::DeleteElement => "Delete Element",
            Self::ModifyElement => "Modify Element",
            Self::AddLayer => "Add Layer",
            Self::DeleteLayer => "Delete Layer",
            Self::ModifyLayer => "Modify Layer",
            Self::BatchOperation => "Batch Operation",
        }
    }
}

/// A history entry for undo/redo.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique entry identifier.
    pub id: String,
    /// Type of action performed.
    pub action_type: HistoryActionType,
    /// Timestamp of the action (Unix epoch ms).
    pub timestamp: i64,
    /// User who performed the action.
    pub user_id: String,
    /// Human-readable description.
    pub description: String,
    /// Whether undo is available from this point.
    pub can_undo: bool,
    /// Whether redo is available from this point.
    pub can_redo: bool,
}

/// Types of operations for offline queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Add new element.
    AddElement,
    /// Update existing element.
    UpdateElement,
    /// Delete element.
    DeleteElement,
    /// Add new layer.
    AddLayer,
    /// Update existing layer.
    UpdateLayer,
    /// Delete layer.
    DeleteLayer,
}

impl OperationType {
    /// Human-readable label for the operation type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::AddElement => "Add Element",
            Self::UpdateElement => "Update Element",
            Self::DeleteElement => "Delete Element",
            Self::AddLayer => "Add Layer",
            Self::UpdateLayer => "Update Layer",
            Self::DeleteLayer => "Delete Layer",
        }
    }
}

/// Status of an offline operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OfflineStatus {
    /// Waiting to sync.
    Pending,
    /// Currently syncing.
    Syncing,
    /// Successfully synced.
    Synced,
    /// Failed to sync with error message.
    Failed(String),
}

impl OfflineStatus {
    /// Whether the operation is still pending.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Whether the operation is currently syncing.
    pub fn is_syncing(&self) -> bool {
        matches!(self, Self::Syncing)
    }

    /// Whether the operation has been synced.
    pub fn is_synced(&self) -> bool {
        matches!(self, Self::Synced)
    }

    /// Whether the operation has failed.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }
}

/// An operation queued for offline sync.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineOperation {
    /// Unique operation identifier.
    pub id: String,
    /// Type of operation.
    pub operation_type: OperationType,
    /// JSON-serialized payload.
    pub payload: String,
    /// Creation timestamp (Unix epoch ms).
    pub created_at: i64,
    /// Number of sync retry attempts.
    pub retry_count: u32,
    /// Current sync status.
    pub status: OfflineStatus,
}

/// Tool types for the canvas toolbar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ToolType {
    /// Selection tool.
    #[default]
    Select,
    /// Pen for freehand drawing.
    Pen,
    /// Brush for softer strokes.
    Brush,
    /// Eraser tool.
    Eraser,
    /// Rectangle shape tool.
    Rectangle,
    /// Ellipse shape tool.
    Ellipse,
    /// Text tool.
    Text,
    /// Pan/drag viewport.
    Pan,
    /// Zoom tool.
    Zoom,
}

impl ToolType {
    /// Human-readable label for the tool.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Select => "Select",
            Self::Pen => "Pen",
            Self::Brush => "Brush",
            Self::Eraser => "Eraser",
            Self::Rectangle => "Rectangle",
            Self::Ellipse => "Ellipse",
            Self::Text => "Text",
            Self::Pan => "Pan",
            Self::Zoom => "Zoom",
        }
    }

    /// Icon representation for the tool.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Select => "pointer",
            Self::Pen => "pen",
            Self::Brush => "brush",
            Self::Eraser => "eraser",
            Self::Rectangle => "square",
            Self::Ellipse => "circle",
            Self::Text => "type",
            Self::Pan => "move",
            Self::Zoom => "zoom-in",
        }
    }

    /// All available tools.
    pub fn all() -> &'static [ToolType] {
        &[
            Self::Select,
            Self::Pen,
            Self::Brush,
            Self::Eraser,
            Self::Rectangle,
            Self::Ellipse,
            Self::Text,
            Self::Pan,
            Self::Zoom,
        ]
    }
}

/// Snapshot of canvas state including collaboration data.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CanvasSnapshot {
    /// Current canvas state.
    pub state: CanvasState,
    /// Remote cursors from other users.
    pub remote_cursors: Vec<RemoteCursor>,
    /// Recent history entries.
    pub history: Vec<HistoryEntry>,
    /// Pending offline operations.
    pub offline_operations: Vec<OfflineOperation>,
    /// Currently selected tool.
    pub selected_tool: ToolType,
    /// Current stroke color.
    pub stroke_color: String,
    /// Current fill color.
    pub fill_color: Option<String>,
    /// Current stroke width.
    pub stroke_width: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_default_identity() {
        let t = Transform::default();
        assert_eq!(t.translate_x, 0.0);
        assert_eq!(t.translate_y, 0.0);
        assert_eq!(t.scale_x, 1.0);
        assert_eq!(t.scale_y, 1.0);
        assert_eq!(t.rotation, 0.0);
    }

    #[test]
    fn tool_type_labels() {
        assert_eq!(ToolType::Select.label(), "Select");
        assert_eq!(ToolType::Pen.label(), "Pen");
        assert_eq!(ToolType::Brush.label(), "Brush");
        assert_eq!(ToolType::Eraser.label(), "Eraser");
        assert_eq!(ToolType::Rectangle.label(), "Rectangle");
        assert_eq!(ToolType::Ellipse.label(), "Ellipse");
        assert_eq!(ToolType::Text.label(), "Text");
        assert_eq!(ToolType::Pan.label(), "Pan");
        assert_eq!(ToolType::Zoom.label(), "Zoom");
    }

    #[test]
    fn tool_type_icons() {
        assert_eq!(ToolType::Select.icon(), "pointer");
        assert_eq!(ToolType::Pen.icon(), "pen");
        assert_eq!(ToolType::Eraser.icon(), "eraser");
    }

    #[test]
    fn tool_type_all_variants() {
        let all = ToolType::all();
        assert_eq!(all.len(), 9);
        assert!(all.contains(&ToolType::Select));
        assert!(all.contains(&ToolType::Zoom));
    }

    #[test]
    fn history_action_type_labels() {
        assert_eq!(HistoryActionType::AddElement.label(), "Add Element");
        assert_eq!(HistoryActionType::DeleteElement.label(), "Delete Element");
        assert_eq!(HistoryActionType::ModifyElement.label(), "Modify Element");
        assert_eq!(HistoryActionType::AddLayer.label(), "Add Layer");
        assert_eq!(HistoryActionType::DeleteLayer.label(), "Delete Layer");
        assert_eq!(HistoryActionType::ModifyLayer.label(), "Modify Layer");
        assert_eq!(HistoryActionType::BatchOperation.label(), "Batch Operation");
    }

    #[test]
    fn operation_type_labels() {
        assert_eq!(OperationType::AddElement.label(), "Add Element");
        assert_eq!(OperationType::UpdateElement.label(), "Update Element");
        assert_eq!(OperationType::DeleteElement.label(), "Delete Element");
        assert_eq!(OperationType::AddLayer.label(), "Add Layer");
        assert_eq!(OperationType::UpdateLayer.label(), "Update Layer");
        assert_eq!(OperationType::DeleteLayer.label(), "Delete Layer");
    }

    #[test]
    fn offline_status_predicates() {
        assert!(OfflineStatus::Pending.is_pending());
        assert!(!OfflineStatus::Pending.is_syncing());

        assert!(OfflineStatus::Syncing.is_syncing());
        assert!(!OfflineStatus::Syncing.is_pending());

        assert!(OfflineStatus::Synced.is_synced());
        assert!(!OfflineStatus::Synced.is_failed());

        let failed = OfflineStatus::Failed("error".to_string());
        assert!(failed.is_failed());
        assert!(!failed.is_synced());
    }

    #[test]
    fn canvas_state_default() {
        let state = CanvasState::default();
        assert!(state.elements.is_empty());
        assert!(state.layers.is_empty());
        assert!(state.active_layer_id.is_empty());
        assert_eq!(state.canvas_info.width, 1920);
        assert_eq!(state.canvas_info.height, 1080);
    }

    #[test]
    fn point_serialization() {
        let point = Point {
            x: 10.5,
            y: 20.3,
            pressure: Some(0.8),
        };
        let json = serde_json::to_string(&point).unwrap();
        let deserialized: Point = serde_json::from_str(&json).unwrap();
        assert_eq!(point, deserialized);
    }

    #[test]
    fn element_type_path() {
        let path = ElementType::Path {
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
        };
        if let ElementType::Path {
            points,
            stroke_width,
            ..
        } = path
        {
            assert_eq!(points.len(), 2);
            assert_eq!(stroke_width, 2.0);
        } else {
            panic!("Expected Path variant");
        }
    }

    #[test]
    fn layer_creation() {
        let layer = Layer {
            id: "layer-1".to_string(),
            name: "Background".to_string(),
            visible: true,
            locked: false,
            opacity: 1.0,
            z_index: 0,
        };
        assert!(layer.visible);
        assert!(!layer.locked);
        assert_eq!(layer.opacity, 1.0);
    }
}
