//! Canvas Engine - Infinite canvas with camera and navigation
//!
//! This crate provides the infinite canvas system with world coordinates,
//! camera transforms, viewport management, and spatial indexing.

use creative_domain::{DocumentId, ObjectId, Point2, Rect, Camera, Vector2};
use creative_geometry::{Geometry, GeometryId, BoundingBox};
use rstar::{RTree, RTreeObject, AABB, PointDistance};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Canvas errors
#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("Invalid viewport: {0}")]
    InvalidViewport(String),
    #[error("Camera error: {0}")]
    CameraError(String),
    #[error("Coordinate transform error: {0}")]
    TransformError(String),
    #[error("Object not found: {0}")]
    ObjectNotFound(ObjectId),
    #[error("Document not found: {0}")]
    DocumentNotFound(DocumentId),
}

/// Canvas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasConfig {
    pub min_zoom: f64,
    pub max_zoom: f64,
    pub default_zoom: f64,
    pub show_grid: bool,
    pub grid_spacing: f64,
    pub grid_color: [f32; 4],
    pub background_color: [f32; 4],
    pub infinite: bool,
    pub bounds: Option<Rect>,
}

impl Default for CanvasConfig {
    fn default() -> Self {
        Self {
            min_zoom: 0.01,
            max_zoom: 100.0,
            default_zoom: 1.0,
            show_grid: true,
            grid_spacing: 50.0,
            grid_color: [0.5, 0.5, 0.5, 0.3],
            background_color: [1.0, 1.0, 1.0, 1.0],
            infinite: true,
            bounds: None,
        }
    }
}

/// Canvas state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasState {
    pub camera: Camera,
    pub config: CanvasConfig,
    pub selected_objects: Vec<ObjectId>,
    pub hovered_object: Option<ObjectId>,
    pub is_panning: bool,
    pub pan_start: Option<Point2>,
    pub is_rotating: bool,
    pub rotation_start: Option<f64>,
    pub active_tool: Tool,
    pub tool_state: ToolState,
}

/// Available tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tool {
    Select,
    Pan,
    Zoom,
    Rotate,
    Pen,
    Pencil,
    Brush,
    Eraser,
    Shape(ShapeTool),
    Text,
    Eyedropper,
    Measure,
}

/// Shape tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeTool {
    Rectangle,
    Ellipse,
    Line,
    Polygon,
    Path,
}

/// Tool state for tracking in-progress operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolState {
    Idle,
    Drawing { start: Point2, current: Point2, points: Vec<Point2> },
    Selecting { start: Point2, current: Point2 },
    Panning { start: Point2, camera_center: Point2 },
    Zooming { center: Point2, start_zoom: f64 },
    Rotating { center: Point2, start_angle: f64 },
    CreatingShape { start: Point2, current: Point2, shape_type: ShapeTool },
    EditingText { object_id: ObjectId, cursor_position: usize },
    Measuring { start: Point2, current: Point2 },
}

impl Default for ToolState {
    fn default() -> Self { Self::Idle }
}

/// Object in spatial index
#[derive(Debug, Clone)]
struct SpatialObject {
    id: ObjectId,
    bbox: creative_geometry::BoundingBox,
    geometry_id: creative_geometry::GeometryId,
}

impl rstar::RTreeObject for SpatialObject {
    type Envelope = rstar::AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope {
        rstar::AABB::from_corners([self.bbox.min.x, self.bbox.min.y], [self.bbox.max.x, self.bbox.max.y])
    }
}

impl rstar::PointDistance for SpatialObject {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        let dx = self.bbox.center().x - point[0];
        let dy = self.bbox.center().y - point[1];
        dx * dx + dy * dy
    }
}

/// Infinite canvas with spatial indexing
pub struct Canvas {
    state: CanvasState,
    spatial_index: RTree<SpatialObject>,
    geometries: HashMap<ObjectId, creative_geometry::Geometry>,
    document_id: DocumentId,
    spatial_dirty: bool,
}

impl Canvas {
    pub fn new(document_id: DocumentId, config: CanvasConfig) -> Self {
        let mut state = CanvasState {
            camera: Camera::new(), config,
            selected_objects: Vec::new(), hovered_object: None,
            is_panning: false, pan_start: None,
            is_rotating: false, rotation_start: None,
            active_tool: Tool::Select, tool_state: ToolState::Idle,
        };
        state.camera.zoom = state.config.default_zoom;
        Self { state, spatial_index: RTree::new(), geometries: HashMap::new(), document_id, spatial_dirty: false }
    }
    
    pub fn state(&self) -> &CanvasState { &self.state }
    pub fn state_mut(&mut self) -> &mut CanvasState { &mut self.state }
    pub fn camera(&self) -> &Camera { &self.state.camera }
    pub fn camera_mut(&mut self) -> &mut Camera { &mut self.state.camera }
    pub fn document_id(&self) -> DocumentId { self.document_id }
    
    pub fn add_geometry(&mut self, object_id: ObjectId, geometry: creative_geometry::Geometry) {
        let rect = geometry.bounding_box();
        let bbox = creative_geometry::BoundingBox::new(rect.min, rect.max);
        let geometry_id = geometry.id();
        self.geometries.insert(object_id, geometry);
        let spatial_obj = SpatialObject { id: object_id, bbox, geometry_id };
        self.spatial_index.insert(spatial_obj);
        self.spatial_dirty = true;
    }
    
    pub fn remove_geometry(&mut self, object_id: ObjectId) -> Option<creative_geometry::Geometry> {
        self.geometries.remove(&object_id);
        self.spatial_dirty = true;
        None
    }
    
    pub fn update_geometry(&mut self, object_id: ObjectId, geometry: creative_geometry::Geometry) {
        let rect = geometry.bounding_box();
        let bbox = creative_geometry::BoundingBox::new(rect.min, rect.max);
        let geometry_id = geometry.id();
        self.geometries.insert(object_id, geometry);
        let spatial_obj = SpatialObject { id: object_id, bbox, geometry_id };
        self.spatial_index.insert(spatial_obj);
        self.spatial_dirty = true;
    }
    
    pub fn get_geometry(&self, object_id: ObjectId) -> Option<&creative_geometry::Geometry> { self.geometries.get(&object_id) }
    pub fn geometries(&self) -> &HashMap<ObjectId, creative_geometry::Geometry> { &self.geometries }
    
    pub fn query_rect(&self, rect: Rect) -> Vec<ObjectId> {
        if self.spatial_dirty { return self.geometries.keys().copied().collect(); }
        let aabb = rstar::AABB::from_corners([rect.min.x, rect.min.y], [rect.max.x, rect.max.y]);
        self.spatial_index.locate_in_envelope_intersecting(aabb).map(|obj| obj.id).collect()
    }
    
    pub fn query_point(&self, point: Point2, tolerance: f64) -> Vec<ObjectId> {
        let search_rect = Rect::new(Point2::new(point.x - tolerance, point.y - tolerance), Point2::new(point.x + tolerance, point.y + tolerance));
        self.query_rect(search_rect)
    }
    
    pub fn hit_test(&self, screen_point: Point2, viewport: Viewport) -> Option<ObjectId> {
        let world_point = self.screen_to_world(screen_point, viewport);
        let objects = self.query_point(world_point, 5.0 / self.state.camera.zoom);
        objects.into_iter().last()
    }
    
    pub fn world_to_screen(&self, world: Point2, viewport: Viewport) -> Point2 {
        let camera = &self.state.camera;
        let viewport_size = (viewport.width as f64, viewport.height as f64);
        let device_scale = viewport.scale_factor as f64;
        camera.world_to_screen(world, viewport_size, device_scale)
    }
    
    pub fn screen_to_world(&self, screen: Point2, viewport: Viewport) -> Point2 {
        let camera = &self.state.camera;
        let viewport_size = (viewport.width as f64, viewport.height as f64);
        let device_scale = viewport.scale_factor as f64;
        camera.screen_to_world(screen, viewport_size, device_scale)
    }
    
    pub fn pan(&mut self, delta: Vector2, viewport: Viewport) {
        let world_delta = Vector2::new(delta.x / (self.state.camera.zoom * viewport.scale_factor as f64), delta.y / (self.state.camera.zoom * viewport.scale_factor as f64));
        let cos = self.state.camera.rotation.cos(); let sin = self.state.camera.rotation.sin();
        let rotated = Vector2::new(world_delta.x * cos - world_delta.y * sin, world_delta.x * sin + world_delta.y * cos);
        self.state.camera.center.x -= rotated.x; self.state.camera.center.y -= rotated.y;
    }
    
    pub fn zoom_at(&mut self, factor: f64, screen_point: Point2, viewport: Viewport) {
        let world_point = self.screen_to_world(screen_point, viewport);
        let new_zoom = (self.state.camera.zoom * factor).clamp(self.state.config.min_zoom, self.state.config.max_zoom);
        if new_zoom == self.state.camera.zoom { return; }
        let zoom_ratio = new_zoom / self.state.camera.zoom;
        let viewport_size = (viewport.width as f64, viewport.height as f64);
        let device_scale = viewport.scale_factor as f64;
        let screen_center = Point2::new(viewport_size.0 * 0.5, viewport_size.1 * 0.5);
        let screen_offset = Point2::new(screen_point.x - screen_center.x, screen_point.y - screen_center.y);
        let world_offset = screen_offset / (self.state.camera.zoom * device_scale);
        self.state.camera.center.x -= world_offset.x * (1.0 - 1.0 / zoom_ratio);
        self.state.camera.center.y -= world_offset.y * (1.0 - 1.0 / zoom_ratio);
        self.state.camera.zoom = new_zoom;
    }
    
    pub fn rotate_at(&mut self, angle_delta: f64, screen_point: Point2, viewport: Viewport) {
        self.state.camera.rotation += angle_delta;
    }
    
    pub fn set_zoom(&mut self, zoom: f64) { self.state.camera.zoom = zoom.clamp(self.state.config.min_zoom, self.state.config.max_zoom); }
    pub fn set_rotation(&mut self, rotation: f64) { self.state.camera.rotation = rotation; }
    pub fn set_center(&mut self, center: Point2) { self.state.camera.center = center; }
    
    pub fn fit_all(&mut self, viewport: Viewport) {
        if self.geometries.is_empty() { self.state.camera.center = Point2::new(0.0, 0.0); self.state.camera.zoom = self.state.config.default_zoom; return; }
        let mut min_x = f64::INFINITY; let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY; let mut max_y = f64::NEG_INFINITY;
        for geom in self.geometries.values() {
            let bbox = geom.bounding_box();
            min_x = min_x.min(bbox.min.x); min_y = min_y.min(bbox.min.y);
            max_x = max_x.max(bbox.max.x); max_y = max_y.max(bbox.max.y);
        }
        let width = max_x - min_x; let height = max_y - min_y;
        let center = Point2::new(min_x + width * 0.5, min_y + height * 0.5);
        let viewport_size = (viewport.width as f64, viewport.height as f64);
        let device_scale = viewport.scale_factor as f64;
        let zoom_x = (viewport_size.0 * 0.9) / (width * device_scale);
        let zoom_y = (viewport_size.1 * 0.9) / (height * device_scale);
        let zoom = zoom_x.min(zoom_y).clamp(self.state.config.min_zoom, self.state.config.max_zoom);
        self.state.camera.center = center; self.state.camera.zoom = zoom;
    }
    
    pub fn fit_selection(&mut self, viewport: Viewport) {
        let selected: Vec<_> = self.state.selected_objects.iter().filter_map(|id| self.geometries.get(id)).collect();
        if selected.is_empty() { self.fit_all(viewport); return; }
        let mut min_x = f64::INFINITY; let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY; let mut max_y = f64::NEG_INFINITY;
        for geom in selected {
            let bbox = geom.bounding_box();
            min_x = min_x.min(bbox.min.x); min_y = min_y.min(bbox.min.y);
            max_x = max_x.max(bbox.max.x); max_y = max_y.max(bbox.max.y);
        }
        let width = max_x - min_x; let height = max_y - min_y;
        let center = Point2::new(min_x + width * 0.5, min_y + height * 0.5);
        let viewport_size = (viewport.width as f64, viewport.height as f64);
        let device_scale = viewport.scale_factor as f64;
        let zoom_x = (viewport_size.0 * 0.9) / (width * device_scale);
        let zoom_y = (viewport_size.1 * 0.9) / (height * device_scale);
        let zoom = zoom_x.min(zoom_y).clamp(self.state.config.min_zoom, self.state.config.max_zoom);
        self.state.camera.center = center; self.state.camera.zoom = zoom;
    }
    
    pub fn reset_view(&mut self) { self.state.camera = Camera::new(); self.state.camera.zoom = self.state.config.default_zoom; }
    
    pub fn select_object(&mut self, object_id: ObjectId, add: bool) { if !add { self.state.selected_objects.clear(); } if !self.state.selected_objects.contains(&object_id) { self.state.selected_objects.push(object_id); } }
    pub fn deselect_object(&mut self, object_id: ObjectId) { self.state.selected_objects.retain(|id| *id != object_id); }
    pub fn deselect_all(&mut self) { self.state.selected_objects.clear(); }
    pub fn selected_objects(&self) -> &[ObjectId] { &self.state.selected_objects }
    
    pub fn set_tool(&mut self, tool: Tool) { self.state.active_tool = tool; self.state.tool_state = ToolState::Idle; }
    pub fn active_tool(&self) -> Tool { self.state.active_tool }
    
    pub fn tool_mouse_down(&mut self, screen_point: Point2, viewport: Viewport) {
        let world_point = self.screen_to_world(screen_point, viewport);
        match self.state.active_tool {
            Tool::Select => { if let Some(obj) = self.hit_test(screen_point, viewport) { self.select_object(obj, false); self.state.tool_state = ToolState::Selecting { start: world_point, current: world_point }; } else { self.deselect_all(); self.state.tool_state = ToolState::Selecting { start: world_point, current: world_point }; } }
            Tool::Pan => { self.state.is_panning = true; self.state.pan_start = Some(screen_point); self.state.tool_state = ToolState::Panning { start: screen_point, camera_center: self.state.camera.center }; }
            Tool::Zoom => { self.state.tool_state = ToolState::Zooming { center: screen_point, start_zoom: self.state.camera.zoom }; }
            Tool::Rotate => { self.state.is_rotating = true; self.state.rotation_start = Some(self.state.camera.rotation); self.state.tool_state = ToolState::Rotating { center: self.screen_to_world(screen_point, viewport), start_angle: self.state.camera.rotation }; }
            Tool::Pen | Tool::Pencil | Tool::Brush => { self.state.tool_state = ToolState::Drawing { start: world_point, current: world_point, points: vec![world_point] }; }
            Tool::Shape(shape_type) => { self.state.tool_state = ToolState::CreatingShape { start: world_point, current: world_point, shape_type }; }
            Tool::Eraser => { self.state.tool_state = ToolState::Drawing { start: world_point, current: world_point, points: vec![world_point] }; }
            Tool::Text => {}
            Tool::Eyedropper => {}
            Tool::Measure => { self.state.tool_state = ToolState::Measuring { start: world_point, current: world_point }; }
        }
    }
    
    pub fn tool_mouse_move(&mut self, screen_point: Point2, viewport: Viewport) {
        let world_point = self.screen_to_world(screen_point, viewport);
        match &mut self.state.tool_state {
            ToolState::Selecting { start, current } => { *current = world_point; }
            ToolState::Panning { start, camera_center } => { if let Some(pan_start) = self.state.pan_start { let delta = Vector2::new(screen_point.x - pan_start.x, screen_point.y - pan_start.y); self.pan(delta, viewport); } }
            ToolState::Zooming { center, start_zoom } => {}
            ToolState::Rotating { center, start_angle } => {}
            ToolState::Drawing { start, current, points } => { *current = world_point; points.push(world_point); }
            ToolState::CreatingShape { start, current, .. } => { *current = world_point; }
            ToolState::Measuring { start, current } => { *current = world_point; }
            _ => {}
        }
        self.state.hovered_object = self.hit_test(screen_point, viewport);
    }
    
    pub fn tool_mouse_up(&mut self, screen_point: Point2, viewport: Viewport) -> Option<ToolAction> {
        let world_point = self.screen_to_world(screen_point, viewport);
        let action = match std::mem::take(&mut self.state.tool_state) {
            ToolState::Selecting { start, current } => { self.state.is_panning = false; self.state.pan_start = None; if start.distance_to(current) < 10.0 { ToolAction::SelectSingle } else { ToolAction::SelectRect(Rect::new(start, current)) } }
            ToolState::Panning { .. } => { self.state.is_panning = false; self.state.pan_start = None; ToolAction::Pan }
            ToolState::Zooming { .. } => ToolAction::Zoom,
            ToolState::Rotating { .. } => { self.state.is_rotating = false; self.state.rotation_start = None; ToolAction::Rotate }
            ToolState::Drawing { start, current, points } => { if points.len() > 1 { ToolAction::DrawStroke { points } } else { ToolAction::None } }
            ToolState::CreatingShape { start, current, shape_type } => { ToolAction::CreateShape { shape_type, rect: Rect::new(start, current) } }
            ToolState::Measuring { start, current } => { ToolAction::CreateMeasurement { start, end: current } }
            _ => ToolAction::None,
        };
        self.state.tool_state = ToolState::Idle;
        Some(action)
    }
    
    pub fn hovered_object(&self) -> Option<ObjectId> { self.state.hovered_object }
    pub fn grid_spacing(&self) -> f64 { self.state.config.grid_spacing }
    pub fn grid_color(&self) -> [f32; 4] { self.state.config.grid_color }
    pub fn background_color(&self) -> [f32; 4] { self.state.config.background_color }
    pub fn show_grid(&self) -> bool { self.state.config.show_grid }
    pub fn set_show_grid(&mut self, show: bool) { self.state.config.show_grid = show; }
    pub fn set_grid_spacing(&mut self, spacing: f64) { self.state.config.grid_spacing = spacing; }
}

/// Actions that can result from tool input
#[derive(Debug, Clone)]
pub enum ToolAction { None, SelectSingle, SelectRect(Rect), Pan, Zoom, Rotate, DrawStroke { points: Vec<Point2> }, CreateShape { shape_type: ShapeTool, rect: Rect }, CreateMeasurement { start: Point2, end: Point2 } }

/// Viewport information
#[derive(Debug, Clone, Copy)]
pub struct Viewport { pub width: u32, pub height: u32, pub scale_factor: f32 }

impl Default for Viewport { fn default() -> Self { Self { width: 800, height: 600, scale_factor: 1.0 } } }

#[cfg(test)]
mod tests {
    use super::*; use creative_domain::{DocumentId, Point2, Vector2};
    #[test] fn test_canvas_creation() { let doc_id = DocumentId::new(); let canvas = Canvas::new(doc_id, CanvasConfig::default()); assert_eq!(canvas.document_id(), doc_id); assert_eq!(canvas.state().camera.zoom, 1.0); assert_eq!(canvas.state().camera.center, Point2::new(0.0, 0.0)); }
    #[test] fn test_pan() { let doc_id = DocumentId::new(); let mut canvas = Canvas::new(doc_id, CanvasConfig::default()); let viewport = Viewport { width: 800, height: 600, scale_factor: 1.0 }; let initial_center = canvas.state().camera.center; canvas.pan(Vector2::new(100.0, 50.0), viewport); assert_ne!(canvas.state().camera.center, initial_center); }
    #[test] fn test_zoom() { let doc_id = DocumentId::new(); let mut canvas = Canvas::new(doc_id, CanvasConfig::default()); let viewport = Viewport { width: 800, height: 600, scale_factor: 1.0 }; let screen_center = Point2::new(400.0, 300.0); canvas.zoom_at(2.0, screen_center, viewport); assert_eq!(canvas.state().camera.zoom, 2.0); }
    #[test] fn test_zoom_clamping() { let doc_id = DocumentId::new(); let mut canvas = Canvas::new(doc_id, CanvasConfig::default()); let viewport = Viewport { width: 800, height: 600, scale_factor: 1.0 }; canvas.zoom_at(200.0, Point2::new(400.0, 300.0), viewport); assert_eq!(canvas.state().camera.zoom, 100.0); canvas.set_zoom(0.01); assert_eq!(canvas.state().camera.zoom, 0.01); canvas.set_zoom(0.001); assert_eq!(canvas.state().camera.zoom, 0.01); canvas.set_zoom(200.0); assert_eq!(canvas.state().camera.zoom, 100.0); }
    #[test] fn test_fit_all() { let doc_id = DocumentId::new(); let mut canvas = Canvas::new(doc_id, CanvasConfig::default()); let viewport = Viewport { width: 800, height: 600, scale_factor: 1.0 }; canvas.fit_all(viewport); assert!(canvas.state().camera.zoom > 0.0); }
    #[test] fn test_tool_selection() { let doc_id = DocumentId::new(); let mut canvas = Canvas::new(doc_id, CanvasConfig::default()); canvas.set_tool(Tool::Pen); assert_eq!(canvas.active_tool(), Tool::Pen); canvas.set_tool(Tool::Select); assert_eq!(canvas.active_tool(), Tool::Select); }
    #[test] fn test_selection() { let doc_id = DocumentId::new(); let mut canvas = Canvas::new(doc_id, CanvasConfig::default()); let obj1 = ObjectId::new(); let obj2 = ObjectId::new(); canvas.select_object(obj1, false); assert_eq!(canvas.selected_objects(), &[obj1]); canvas.select_object(obj2, true); assert_eq!(canvas.selected_objects(), &[obj1, obj2]); canvas.deselect_object(obj1); assert_eq!(canvas.selected_objects(), &[obj2]); canvas.deselect_all(); assert!(canvas.selected_objects().is_empty()); }
    #[test] fn test_world_screen_transform() { let doc_id = DocumentId::new(); let canvas = Canvas::new(doc_id, CanvasConfig::default()); let viewport = Viewport { width: 800, height: 600, scale_factor: 1.0 }; let world = Point2::new(100.0, 100.0); let screen = canvas.world_to_screen(world, viewport); let back = canvas.screen_to_world(screen, viewport); assert!((back.x - world.x).abs() < 0.001); assert!((back.y - world.y).abs() < 0.001); }
}
