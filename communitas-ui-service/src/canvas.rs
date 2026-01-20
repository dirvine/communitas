//! Canvas service for collaborative drawing and visual surfaces.
//!
//! Wraps canvas-core Scene and provides reactive state updates via watch channels.
//! Uses LWW (Last-Write-Wins) conflict resolution for real-time visual operations
//! as documented in ADR-021.

use std::sync::Arc;

use canvas_core::{Element, ElementId, ElementKind, Scene, Transform};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::watch;
use tracing::instrument;

use crate::auth::{AuthController, AuthService, AuthStateSnapshot};
use communitas_core::app::CommunitasApp;

/// Errors that can occur during canvas operations.
#[derive(Debug, Error)]
pub enum CanvasError {
    /// Not authenticated - canvas requires valid session.
    #[error("not authenticated")]
    NotAuthenticated,

    /// Element not found in the scene.
    #[error("element not found: {0}")]
    ElementNotFound(String),

    /// Invalid transform values.
    #[error("invalid transform: {0}")]
    InvalidTransform(String),

    /// Scene serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Upstream canvas error.
    #[error("canvas error: {0}")]
    Canvas(String),
}

impl From<canvas_core::CanvasError> for CanvasError {
    fn from(err: canvas_core::CanvasError) -> Self {
        Self::Canvas(err.to_string())
    }
}

/// Serializable element view for snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementView {
    /// Element unique identifier.
    pub id: String,
    /// Element kind (chart, image, text, etc.).
    pub kind: ElementKindView,
    /// Position and size.
    pub transform: TransformView,
    /// Whether the element is selected.
    pub selected: bool,
    /// Whether the element is interactive.
    pub interactive: bool,
}

impl From<&Element> for ElementView {
    fn from(elem: &Element) -> Self {
        Self {
            id: elem.id.to_string(),
            kind: ElementKindView::from(&elem.kind),
            transform: TransformView::from(&elem.transform),
            selected: elem.selected,
            interactive: elem.interactive,
        }
    }
}

/// Serializable element kind for snapshots.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ElementKindView {
    /// Chart element.
    Chart {
        /// Chart type identifier.
        chart_type: String,
    },
    /// Image element.
    Image {
        /// Image source URI.
        src: String,
        /// Image format.
        format: String,
    },
    /// 3D model element.
    Model3D {
        /// Model source URI.
        src: String,
    },
    /// Video element.
    Video {
        /// Stream identifier.
        stream_id: String,
        /// Whether this is a live stream.
        is_live: bool,
    },
    /// Text element.
    Text {
        /// Text content.
        content: String,
        /// Font size in pixels.
        font_size: f32,
        /// Color as hex string.
        color: String,
    },
    /// Group container.
    Group {
        /// Number of children.
        child_count: usize,
    },
    /// Overlay layer.
    OverlayLayer {
        /// Number of children.
        child_count: usize,
    },
}

impl From<&ElementKind> for ElementKindView {
    fn from(kind: &ElementKind) -> Self {
        match kind {
            ElementKind::Chart { chart_type, .. } => Self::Chart {
                chart_type: chart_type.clone(),
            },
            ElementKind::Image { src, format } => Self::Image {
                src: src.clone(),
                format: format!("{format:?}").to_lowercase(),
            },
            ElementKind::Model3D { src, .. } => Self::Model3D { src: src.clone() },
            ElementKind::Video {
                stream_id, is_live, ..
            } => Self::Video {
                stream_id: stream_id.clone(),
                is_live: *is_live,
            },
            ElementKind::Text {
                content,
                font_size,
                color,
            } => Self::Text {
                content: content.clone(),
                font_size: *font_size,
                color: color.clone(),
            },
            ElementKind::Group { children } => Self::Group {
                child_count: children.len(),
            },
            ElementKind::OverlayLayer { children, .. } => Self::OverlayLayer {
                child_count: children.len(),
            },
        }
    }
}

/// Serializable transform for snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransformView {
    /// X position in pixels.
    pub x: f32,
    /// Y position in pixels.
    pub y: f32,
    /// Width in pixels.
    pub width: f32,
    /// Height in pixels.
    pub height: f32,
    /// Rotation in radians.
    pub rotation: f32,
    /// Z-index for layering.
    pub z_index: i32,
}

impl From<&Transform> for TransformView {
    fn from(t: &Transform) -> Self {
        Self {
            x: t.x,
            y: t.y,
            width: t.width,
            height: t.height,
            rotation: t.rotation,
            z_index: t.z_index,
        }
    }
}

impl From<TransformView> for Transform {
    fn from(v: TransformView) -> Self {
        Self {
            x: v.x,
            y: v.y,
            width: v.width,
            height: v.height,
            rotation: v.rotation,
            z_index: v.z_index,
        }
    }
}

/// Current state snapshot of the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSnapshot {
    /// All elements in the scene.
    pub elements: Vec<ElementView>,
    /// Currently selected element IDs.
    pub selected_ids: Vec<String>,
    /// Viewport width in pixels.
    pub viewport_width: f32,
    /// Viewport height in pixels.
    pub viewport_height: f32,
    /// Current zoom level (1.0 = 100%).
    pub zoom: f32,
    /// Pan offset X.
    pub pan_x: f32,
    /// Pan offset Y.
    pub pan_y: f32,
    /// Whether a canvas operation is in progress.
    pub loading: bool,
}

impl Default for CanvasSnapshot {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            selected_ids: Vec::new(),
            viewport_width: 800.0,
            viewport_height: 600.0,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            loading: false,
        }
    }
}

/// Canvas service for collaborative drawing and visual surfaces.
///
/// Follows the same pattern as other UiServices (KanbanService, MessagingService):
/// - watch channels for reactive state updates
/// - tracing instrumentation on all public methods
/// - authentication checks where required
pub struct CanvasService {
    auth: Arc<AuthController>,
    app: Arc<CommunitasApp>,
    scene: std::sync::RwLock<Scene>,
    tx: watch::Sender<CanvasSnapshot>,
    rx: watch::Receiver<CanvasSnapshot>,
}

impl CanvasService {
    /// Create a new canvas service with the given auth controller and app.
    #[must_use]
    pub fn new(auth: Arc<AuthController>, app: Arc<CommunitasApp>) -> Self {
        let scene = Scene::new(800.0, 600.0);
        let snapshot = Self::scene_to_snapshot(&scene, false);
        let (tx, rx) = watch::channel(snapshot);
        Self {
            auth,
            app,
            scene: std::sync::RwLock::new(scene),
            tx,
            rx,
        }
    }

    /// Returns a clone of the Communitas app handle.
    #[must_use]
    pub fn app(&self) -> Arc<CommunitasApp> {
        self.app.clone()
    }

    /// Subscribe to canvas state changes.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<CanvasSnapshot> {
        self.rx.clone()
    }

    /// Get the current canvas snapshot.
    #[must_use]
    pub fn current_snapshot(&self) -> CanvasSnapshot {
        self.rx.borrow().clone()
    }

    /// Add a text element to the canvas.
    #[instrument(skip(self), fields(content_len = content.len()))]
    pub async fn add_text(
        &self,
        content: String,
        x: f32,
        y: f32,
        font_size: f32,
        color: String,
    ) -> Result<String, CanvasError> {
        self.require_auth()?;

        let element = Element::new(ElementKind::Text {
            content,
            font_size,
            color,
        })
        .with_transform(Transform {
            x,
            y,
            width: 200.0,
            height: font_size * 1.5,
            rotation: 0.0,
            z_index: 0,
        });

        let id = {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.add_element(element)
        };

        self.publish_snapshot(false);
        tracing::debug!(element_id = %id, "added text element");
        Ok(id.to_string())
    }

    /// Add an image element to the canvas.
    #[instrument(skip(self), fields(src_len = src.len()))]
    pub async fn add_image(
        &self,
        src: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<String, CanvasError> {
        self.require_auth()?;

        let format = if src.ends_with(".svg") {
            canvas_core::ImageFormat::Svg
        } else if src.ends_with(".webp") {
            canvas_core::ImageFormat::WebP
        } else if src.ends_with(".jpg") || src.ends_with(".jpeg") {
            canvas_core::ImageFormat::Jpeg
        } else {
            canvas_core::ImageFormat::Png
        };

        let element = Element::new(ElementKind::Image { src, format }).with_transform(Transform {
            x,
            y,
            width,
            height,
            rotation: 0.0,
            z_index: 0,
        });

        let id = {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.add_element(element)
        };

        self.publish_snapshot(false);
        tracing::debug!(element_id = %id, "added image element");
        Ok(id.to_string())
    }

    /// Add a chart element to the canvas.
    #[instrument(skip(self, data))]
    pub async fn add_chart(
        &self,
        chart_type: String,
        data: serde_json::Value,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<String, CanvasError> {
        self.require_auth()?;

        let element =
            Element::new(ElementKind::Chart { chart_type, data }).with_transform(Transform {
                x,
                y,
                width,
                height,
                rotation: 0.0,
                z_index: 0,
            });

        let id = {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.add_element(element)
        };

        self.publish_snapshot(false);
        tracing::debug!(element_id = %id, "added chart element");
        Ok(id.to_string())
    }

    /// Remove an element from the canvas.
    #[instrument(skip(self))]
    pub async fn remove_element(&self, element_id: &str) -> Result<(), CanvasError> {
        self.require_auth()?;

        let id = ElementId::parse(element_id)
            .map_err(|e| CanvasError::ElementNotFound(e.to_string()))?;

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.remove_element(&id)?;
        }

        self.publish_snapshot(false);
        tracing::debug!(element_id, "removed element");
        Ok(())
    }

    /// Update an element's transform (position, size, rotation).
    #[instrument(skip(self))]
    pub async fn update_transform(
        &self,
        element_id: &str,
        transform: TransformView,
    ) -> Result<(), CanvasError> {
        self.require_auth()?;

        // Validate transform values
        if transform.width <= 0.0 || transform.height <= 0.0 {
            return Err(CanvasError::InvalidTransform(
                "width and height must be positive".to_string(),
            ));
        }

        let id = ElementId::parse(element_id)
            .map_err(|e| CanvasError::ElementNotFound(e.to_string()))?;

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            let element = scene
                .get_element_mut(id)
                .ok_or_else(|| CanvasError::ElementNotFound(element_id.to_string()))?;
            element.transform = Transform::from(transform);
        }

        self.publish_snapshot(false);
        tracing::debug!(element_id, "updated transform");
        Ok(())
    }

    /// Select an element.
    #[instrument(skip(self))]
    pub async fn select_element(&self, element_id: &str) -> Result<(), CanvasError> {
        self.require_auth()?;

        let id = ElementId::parse(element_id)
            .map_err(|e| CanvasError::ElementNotFound(e.to_string()))?;

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.select(id)?;
        }

        self.publish_snapshot(false);
        tracing::debug!(element_id, "selected element");
        Ok(())
    }

    /// Deselect all elements.
    #[instrument(skip(self))]
    pub async fn deselect_all(&self) -> Result<(), CanvasError> {
        self.require_auth()?;

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.deselect_all();
        }

        self.publish_snapshot(false);
        tracing::debug!("deselected all elements");
        Ok(())
    }

    /// Set the viewport dimensions.
    #[instrument(skip(self))]
    pub async fn set_viewport(&self, width: f32, height: f32) -> Result<(), CanvasError> {
        if width <= 0.0 || height <= 0.0 {
            return Err(CanvasError::InvalidTransform(
                "viewport dimensions must be positive".to_string(),
            ));
        }

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.set_viewport(width, height);
        }

        self.publish_snapshot(false);
        tracing::debug!(width, height, "set viewport");
        Ok(())
    }

    /// Set zoom and pan.
    #[instrument(skip(self))]
    pub async fn set_view(&self, zoom: f32, pan_x: f32, pan_y: f32) -> Result<(), CanvasError> {
        if zoom <= 0.0 {
            return Err(CanvasError::InvalidTransform(
                "zoom must be positive".to_string(),
            ));
        }

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.zoom = zoom;
            scene.pan_x = pan_x;
            scene.pan_y = pan_y;
        }

        self.publish_snapshot(false);
        tracing::debug!(zoom, pan_x, pan_y, "set view");
        Ok(())
    }

    /// Clear all elements from the canvas.
    #[instrument(skip(self))]
    pub async fn clear(&self) -> Result<(), CanvasError> {
        self.require_auth()?;

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            scene.clear();
        }

        self.publish_snapshot(false);
        tracing::debug!("cleared canvas");
        Ok(())
    }

    /// Export the scene as JSON.
    #[instrument(skip(self))]
    pub async fn export_json(&self) -> Result<String, CanvasError> {
        let scene = self
            .scene
            .read()
            .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
        scene
            .to_json()
            .map_err(|e| CanvasError::Serialization(e.to_string()))
    }

    /// Import a scene from JSON.
    #[instrument(skip(self, json))]
    pub async fn import_json(&self, json: &str) -> Result<(), CanvasError> {
        self.require_auth()?;

        let new_scene =
            Scene::from_json(json).map_err(|e| CanvasError::Serialization(e.to_string()))?;

        {
            let mut scene = self
                .scene
                .write()
                .map_err(|_| CanvasError::Canvas("failed to acquire scene lock".to_string()))?;
            *scene = new_scene;
        }

        self.publish_snapshot(false);
        tracing::debug!("imported scene from JSON");
        Ok(())
    }

    /// Get the element at the given canvas coordinates.
    #[must_use]
    pub fn element_at(&self, x: f32, y: f32) -> Option<String> {
        let scene = self.scene.read().ok()?;
        scene.element_at(x, y).map(|id| id.to_string())
    }

    // --- Private helpers ---

    fn require_auth(&self) -> Result<(), CanvasError> {
        if !self.is_authenticated() {
            return Err(CanvasError::NotAuthenticated);
        }
        Ok(())
    }

    fn is_authenticated(&self) -> bool {
        matches!(
            &*self.auth.subscribe().borrow(),
            AuthStateSnapshot::Authenticated(_)
        )
    }

    fn scene_to_snapshot(scene: &Scene, loading: bool) -> CanvasSnapshot {
        let elements: Vec<ElementView> = scene.elements().map(ElementView::from).collect();
        let selected_ids: Vec<String> = scene
            .selected_elements()
            .map(|e| e.id.to_string())
            .collect();

        CanvasSnapshot {
            elements,
            selected_ids,
            viewport_width: scene.viewport_width,
            viewport_height: scene.viewport_height,
            zoom: scene.zoom,
            pan_x: scene.pan_x,
            pan_y: scene.pan_y,
            loading,
        }
    }

    fn publish_snapshot(&self, loading: bool) {
        if let Ok(scene) = self.scene.read() {
            let snapshot = Self::scene_to_snapshot(&scene, loading);
            let _ = self.tx.send(snapshot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::UiStorage;
    use communitas_core::app::CommunitasApp;
    use tempfile::TempDir;

    fn make_auth(temp: &TempDir) -> Arc<AuthController> {
        let storage = UiStorage::from_path(temp.path()).unwrap();
        Arc::new(AuthController::new(storage).unwrap())
    }

    fn make_authenticated_auth(temp: &TempDir) -> Arc<AuthController> {
        let auth = make_auth(temp);
        // Simulate login by setting demo mode
        auth.enable_demo_mode();
        auth
    }

    async fn make_app(temp: &TempDir) -> Arc<CommunitasApp> {
        Arc::new(
            CommunitasApp::new(
                "ocean-forest-moon-star".to_string(),
                "TestUser".to_string(),
                "TestDevice".to_string(),
                temp.path()
                    .join("app_storage")
                    .to_string_lossy()
                    .to_string(),
            )
            .await
            .unwrap(),
        )
    }

    async fn make_service(temp: &TempDir) -> CanvasService {
        let auth = make_auth(temp);
        let app = make_app(temp).await;
        CanvasService::new(auth, app)
    }

    async fn make_authenticated_service(temp: &TempDir) -> CanvasService {
        let auth = make_authenticated_auth(temp);
        let app = make_app(temp).await;
        CanvasService::new(auth, app)
    }

    #[test]
    fn enable_demo_mode_sets_authenticated_state() {
        let temp = TempDir::new().unwrap();
        let auth = make_auth(&temp);

        // Initially not authenticated
        let rx = auth.subscribe();
        assert!(
            matches!(&*rx.borrow(), AuthStateSnapshot::LoggedOut),
            "should start logged out"
        );

        // Enable demo mode
        auth.enable_demo_mode();

        // Now should be authenticated
        let rx2 = auth.subscribe();
        assert!(
            matches!(&*rx2.borrow(), AuthStateSnapshot::Authenticated(_)),
            "should be authenticated after enable_demo_mode"
        );
    }

    #[tokio::test]
    async fn canvas_service_is_authenticated_after_demo_mode() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        // Verify the canvas sees authenticated state
        assert!(
            canvas.is_authenticated(),
            "canvas.is_authenticated() should return true"
        );
    }

    #[tokio::test]
    async fn canvas_service_starts_empty() {
        let temp = TempDir::new().unwrap();
        let canvas = make_service(&temp).await;

        let snap = canvas.current_snapshot();
        assert!(snap.elements.is_empty());
        assert!(snap.selected_ids.is_empty());
        assert!(!snap.loading);
    }

    #[tokio::test]
    async fn canvas_service_default_viewport() {
        let temp = TempDir::new().unwrap();
        let canvas = make_service(&temp).await;

        let snap = canvas.current_snapshot();
        assert!((snap.viewport_width - 800.0).abs() < f32::EPSILON);
        assert!((snap.viewport_height - 600.0).abs() < f32::EPSILON);
        assert!((snap.zoom - 1.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn add_text_requires_auth() {
        let temp = TempDir::new().unwrap();
        let canvas = make_service(&temp).await;

        let result = canvas
            .add_text("Hello".to_string(), 10.0, 20.0, 16.0, "#000000".to_string())
            .await;
        assert!(matches!(result, Err(CanvasError::NotAuthenticated)));
    }

    #[tokio::test]
    async fn add_text_with_auth_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let id = canvas
            .add_text("Hello".to_string(), 10.0, 20.0, 16.0, "#000000".to_string())
            .await
            .unwrap();

        assert!(!id.is_empty());
        let snap = canvas.current_snapshot();
        assert_eq!(snap.elements.len(), 1);
        assert!(matches!(
            snap.elements[0].kind,
            ElementKindView::Text { .. }
        ));
    }

    #[tokio::test]
    async fn add_image_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let id = canvas
            .add_image("test.png".to_string(), 0.0, 0.0, 100.0, 100.0)
            .await
            .unwrap();

        assert!(!id.is_empty());
        let snap = canvas.current_snapshot();
        assert_eq!(snap.elements.len(), 1);
    }

    #[tokio::test]
    async fn add_chart_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let data = serde_json::json!({"values": [1, 2, 3]});
        let id = canvas
            .add_chart("bar".to_string(), data, 0.0, 0.0, 200.0, 150.0)
            .await
            .unwrap();

        assert!(!id.is_empty());
        let snap = canvas.current_snapshot();
        assert_eq!(snap.elements.len(), 1);
    }

    #[tokio::test]
    async fn remove_element_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let id = canvas
            .add_text("Remove me".to_string(), 0.0, 0.0, 14.0, "#fff".to_string())
            .await
            .unwrap();
        assert_eq!(canvas.current_snapshot().elements.len(), 1);

        canvas.remove_element(&id).await.unwrap();
        assert!(canvas.current_snapshot().elements.is_empty());
    }

    #[tokio::test]
    async fn remove_nonexistent_element_fails() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let result = canvas
            .remove_element("00000000-0000-0000-0000-000000000000")
            .await;
        assert!(matches!(result, Err(CanvasError::Canvas(_))));
    }

    #[tokio::test]
    async fn update_transform_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let id = canvas
            .add_text("Move me".to_string(), 0.0, 0.0, 14.0, "#000".to_string())
            .await
            .unwrap();

        let new_transform = TransformView {
            x: 50.0,
            y: 100.0,
            width: 150.0,
            height: 30.0,
            rotation: 0.5,
            z_index: 1,
        };

        canvas.update_transform(&id, new_transform).await.unwrap();

        let snap = canvas.current_snapshot();
        let elem = &snap.elements[0];
        assert!((elem.transform.x - 50.0).abs() < f32::EPSILON);
        assert!((elem.transform.y - 100.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn update_transform_invalid_size_fails() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let id = canvas
            .add_text("Test".to_string(), 0.0, 0.0, 14.0, "#000".to_string())
            .await
            .unwrap();

        let invalid_transform = TransformView {
            x: 0.0,
            y: 0.0,
            width: -10.0,
            height: 20.0,
            rotation: 0.0,
            z_index: 0,
        };

        let result = canvas.update_transform(&id, invalid_transform).await;
        assert!(matches!(result, Err(CanvasError::InvalidTransform(_))));
    }

    #[tokio::test]
    async fn select_and_deselect_elements() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let id = canvas
            .add_text("Select me".to_string(), 0.0, 0.0, 14.0, "#000".to_string())
            .await
            .unwrap();

        canvas.select_element(&id).await.unwrap();
        let snap = canvas.current_snapshot();
        assert_eq!(snap.selected_ids.len(), 1);
        assert!(snap.elements[0].selected);

        canvas.deselect_all().await.unwrap();
        let snap = canvas.current_snapshot();
        assert!(snap.selected_ids.is_empty());
        assert!(!snap.elements[0].selected);
    }

    #[tokio::test]
    async fn set_viewport_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_service(&temp).await;

        canvas.set_viewport(1920.0, 1080.0).await.unwrap();

        let snap = canvas.current_snapshot();
        assert!((snap.viewport_width - 1920.0).abs() < f32::EPSILON);
        assert!((snap.viewport_height - 1080.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn set_viewport_invalid_fails() {
        let temp = TempDir::new().unwrap();
        let canvas = make_service(&temp).await;

        let result = canvas.set_viewport(0.0, 600.0).await;
        assert!(matches!(result, Err(CanvasError::InvalidTransform(_))));
    }

    #[tokio::test]
    async fn set_view_succeeds() {
        let temp = TempDir::new().unwrap();
        let canvas = make_service(&temp).await;

        canvas.set_view(2.0, 100.0, 50.0).await.unwrap();

        let snap = canvas.current_snapshot();
        assert!((snap.zoom - 2.0).abs() < f32::EPSILON);
        assert!((snap.pan_x - 100.0).abs() < f32::EPSILON);
        assert!((snap.pan_y - 50.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn clear_removes_all_elements() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        canvas
            .add_text("One".to_string(), 0.0, 0.0, 14.0, "#000".to_string())
            .await
            .unwrap();
        canvas
            .add_text("Two".to_string(), 10.0, 10.0, 14.0, "#000".to_string())
            .await
            .unwrap();
        assert_eq!(canvas.current_snapshot().elements.len(), 2);

        canvas.clear().await.unwrap();
        assert!(canvas.current_snapshot().elements.is_empty());
    }

    #[tokio::test]
    async fn export_import_json_roundtrip() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        canvas
            .add_text(
                "Export me".to_string(),
                50.0,
                50.0,
                18.0,
                "#ff0000".to_string(),
            )
            .await
            .unwrap();

        let json = canvas.export_json().await.unwrap();
        assert!(!json.is_empty());

        canvas.clear().await.unwrap();
        assert!(canvas.current_snapshot().elements.is_empty());

        canvas.import_json(&json).await.unwrap();
        let snap = canvas.current_snapshot();
        assert_eq!(snap.elements.len(), 1);
    }

    #[tokio::test]
    async fn element_at_finds_element() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        // Add element via the async API
        canvas
            .add_text(
                "Click me".to_string(),
                100.0,
                100.0,
                14.0,
                "#000".to_string(),
            )
            .await
            .unwrap();

        // Update the transform to have specific dimensions
        let snap = canvas.current_snapshot();
        let id = &snap.elements[0].id;
        let new_transform = TransformView {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 50.0,
            rotation: 0.0,
            z_index: 0,
        };
        canvas.update_transform(id, new_transform).await.unwrap();

        // Point inside element
        let found = canvas.element_at(150.0, 125.0);
        assert!(found.is_some());

        // Point outside element
        let not_found = canvas.element_at(50.0, 50.0);
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn subscribe_receives_updates() {
        let temp = TempDir::new().unwrap();
        let canvas = make_authenticated_service(&temp).await;

        let mut rx = canvas.subscribe();

        canvas
            .add_text("Watch me".to_string(), 0.0, 0.0, 14.0, "#000".to_string())
            .await
            .unwrap();

        rx.changed().await.unwrap();
        let snap = rx.borrow().clone();
        assert_eq!(snap.elements.len(), 1);
    }
}
