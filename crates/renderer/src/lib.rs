//! Renderer abstraction for Creative Engine
//!
//! Provides a backend-agnostic rendering interface with support for
//! GPU (wgpu) and software (tiny-skia) backends.

use creative_domain::{Point2, Rect, Color, Camera};
use creative_geometry::{Geometry, GeometryId, BoundingBox, StrokeStyle};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

/// Renderer errors
#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Backend not initialized")]
    NotInitialized,
    #[error("Device lost: {0}")]
    DeviceLost(String),
    #[error("Out of memory: {0}")]
    OutOfMemory(String),
    #[error("Shader compilation failed: {0}")]
    ShaderError(String),
    #[error("Texture creation failed: {0}")]
    TextureError(String),
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Render backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderBackend {
    Wgpu,
    Software,
}

/// Render configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    pub backend: RenderBackend,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub background_color: Color,
    pub enable_grid: bool,
    pub grid_spacing: f64,
    pub grid_color: Color,
    pub grid_opacity: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            backend: RenderBackend::Wgpu,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
            vsync: true,
            msaa_samples: 4,
            background_color: Color::WHITE,
            enable_grid: true,
            grid_spacing: 50.0,
            grid_color: Color::new(0.5, 0.5, 0.5, 0.3),
            grid_opacity: 0.5,
        }
    }
}

/// Render statistics for profiling
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RenderStats {
    pub frame_time_ms: f64,
    pub draw_calls: u32,
    pub vertices: u32,
    pub triangles: u32,
    pub textures: u32,
    pub culled_objects: u32,
    pub visible_objects: u32,
}

/// Render layer types (document content vs editor overlays)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RenderLayer {
    /// Artboard/page backgrounds
    ArtboardBackground,
    /// Document geometry objects
    DocumentContent,
    /// Grids and guides
    Precision,
    /// Selection outlines and handles
    Selection,
    /// Cursor and stylus feedback
    CursorFeedback,
    /// Transient previews (stroke preview, transform handles)
    TransientPreview,
    /// AI proposed changes overlay
    AiOverlay,
}

/// Render item representing a drawable object
#[derive(Debug, Clone)]
pub struct RenderItem {
    pub object_id: creative_domain::ObjectId,
    pub geometry_id: GeometryId,
    pub layer: RenderLayer,
    pub z_index: i32,
    pub bounding_box: BoundingBox,
    pub transform: creative_domain::Transform2D,
    pub fill: Option<Color>,
    pub stroke: Option<StrokeStyle>,
    pub opacity: f32,
    pub blend_mode: creative_domain::BlendMode,
    pub clip_rect: Option<Rect>,
    pub is_selected: bool,
    pub is_hovered: bool,
}

/// Viewport for rendering
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f32,
    pub camera: Camera,
}

impl Viewport {
    pub fn new(width: u32, height: u32, scale_factor: f32, camera: Camera) -> Self {
        Self { width, height, scale_factor, camera }
    }
    
    pub fn world_to_screen(&self, point: Point2) -> Point2 {
        self.camera.world_to_screen(point, (self.width as f64, self.height as f64), self.scale_factor as f64)
    }
    
    pub fn screen_to_world(&self, point: Point2) -> Point2 {
        self.camera.screen_to_world(point, (self.width as f64, self.height as f64), self.scale_factor as f64)
    }
    
    pub fn visible_world_rect(&self) -> Rect {
        let top_left = self.screen_to_world(Point2::new(0.0, 0.0));
        let bottom_right = self.screen_to_world(Point2::new(self.width as f64, self.height as f64));
        Rect::new(top_left, bottom_right)
    }
}

/// Render context passed to backend implementations
pub struct RenderContext<'a> {
    pub viewport: &'a Viewport,
    pub config: &'a RenderConfig,
    pub stats: &'a mut RenderStats,
}

/// Trait for render backends
pub trait RenderBackendImpl: Send + Sync {
    /// Initialize the backend
    fn initialize(&mut self, config: &RenderConfig) -> Result<(), RenderError>;
    
    /// Resize the render target
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) -> Result<(), RenderError>;
    
    /// Begin a new frame
    fn begin_frame(&mut self, context: &mut RenderContext) -> Result<(), RenderError>;
    
    /// End the current frame and present
    fn end_frame(&mut self) -> Result<(), RenderError>;
    
    /// Render a batch of items
    fn render_items(&mut self, items: &[RenderItem], context: &mut RenderContext) -> Result<(), RenderError>;
    
    /// Render grid
    fn render_grid(&mut self, context: &mut RenderContext) -> Result<(), RenderError>;
    
    /// Render selection overlay
    fn render_selection(&mut self, items: &[RenderItem], context: &mut RenderContext) -> Result<(), RenderError>;
    
    /// Render transient preview (e.g., stroke being drawn)
    fn render_preview(&mut self, preview: &PreviewData, context: &mut RenderContext) -> Result<(), RenderError>;
    
    /// Get backend type
    fn backend_type(&self) -> RenderBackend;
    
    /// Check if backend supports a feature
    fn supports_feature(&self, feature: RenderFeature) -> bool;
    
    /// Get device info
    fn device_info(&self) -> String;
}

/// Render features for capability querying
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderFeature {
    MSAA,
    ComputeShaders,
    StorageBuffers,
    TextureArrays,
    MultiDrawIndirect,
    RayTracing,
    MeshShaders,
}

/// Preview data for transient rendering
#[derive(Debug, Clone)]
pub struct PreviewData {
    pub points: Vec<Point2>,
    pub stroke_style: StrokeStyle,
    pub preview_type: PreviewType,
    pub transform: creative_domain::Transform2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewType {
    Stroke,
    Shape,
    Transform,
    Measurement,
}

/// Main renderer facade
pub struct Renderer {
    backend: Box<dyn RenderBackendImpl>,
    config: RenderConfig,
    stats: RenderStats,
    /// Cached render results, reserved for incremental rendering
    #[allow(dead_code)]
    item_cache: HashMap<GeometryId, CachedGeometry>,
    frame_count: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
struct CachedGeometry {
    geometry_id: GeometryId,
    revision: u64,
    // Backend-specific cached data would go here
    #[allow(dead_code)]
    data: Vec<u8>,
}

impl Renderer {
    /// Create a new renderer with the specified backend
    pub fn new(backend_type: RenderBackend) -> Result<Self, RenderError> {
        let _backend: Box<dyn RenderBackendImpl> = match backend_type {
            RenderBackend::Wgpu => {
                #[cfg(feature = "wgpu")]
                {
                    // This would use creative-renderer-wgpu crate
                    // For now, return error as it needs proper setup
                    return Err(RenderError::Unsupported("wgpu backend requires creative-renderer-wgpu crate".to_string()));
                }
                #[cfg(not(feature = "wgpu"))]
                {
                    return Err(RenderError::Unsupported("wgpu backend not compiled".to_string()));
                }
            }
            RenderBackend::Software => {
                // Use the software backend from creative-renderer-software crate
                // For now, return error - the app should create the backend directly
                return Err(RenderError::Unsupported("Software backend requires creative-renderer-software crate".to_string()));
            }
        };
    }
    
    /// Create a renderer with default backend (prefers wgpu)
    pub fn default() -> Result<Self, RenderError> {
        // Try wgpu first, fall back to software
        if cfg!(feature = "wgpu") {
            Self::new(RenderBackend::Wgpu)
        } else {
            Self::new(RenderBackend::Software)
        }
    }

    /// Create a renderer around a pre-constructed backend implementation.
    ///
    /// This lets composition roots (e.g. the application crate) supply backend
    /// implementations from optional crates such as `creative-renderer-software`
    /// without creating a circular dependency between the two renderer crates.
    pub fn with_backend(backend: Box<dyn RenderBackendImpl>) -> Self {
        Self {
            backend,
            config: RenderConfig::default(),
            stats: RenderStats::default(),
            item_cache: HashMap::new(),
            frame_count: 0,
        }
    }
    
    /// Initialize the renderer
    pub fn initialize(&mut self, config: RenderConfig) -> Result<(), RenderError> {
        self.config = config;
        self.backend.initialize(&self.config)
    }
    
    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f32) -> Result<(), RenderError> {
        self.config.width = width;
        self.config.height = height;
        self.config.scale_factor = scale_factor;
        self.backend.resize(width, height, scale_factor)
    }
    
    /// Render a frame
    pub fn render(
        &mut self,
        viewport: &Viewport,
        items: &[RenderItem],
        preview: Option<&PreviewData>,
    ) -> Result<RenderStats, RenderError> {
        self.frame_count += 1;
        
        // Cull items outside viewport (do this before borrowing stats)
        let visible_items = self.cull_items(items, viewport);
        let culled_count = items.len() as u32 - visible_items.len() as u32;
        let visible_count = visible_items.len() as u32;
        
        // Sort items by layer then z-index
        let mut sorted_items: Vec<RenderItem> = visible_items.iter().map(|&item| item.clone()).collect();
        sorted_items.sort_by(|a, b| {
            (a.layer as u8).cmp(&(b.layer as u8))
                .then(a.z_index.cmp(&b.z_index))
        });
        
        // Now create context with mutable stats
        self.stats = RenderStats::default();
        self.stats.culled_objects = culled_count;
        self.stats.visible_objects = visible_count;
        
        let mut context = RenderContext {
            viewport,
            config: &self.config,
            stats: &mut self.stats,
        };
        
        // Begin frame
        self.backend.begin_frame(&mut context)?;
        
        // Clear with background
        // (Backend handles clear in begin_frame)
        
        // Render grid if enabled
        if self.config.enable_grid {
            self.backend.render_grid(&mut context)?;
        }
        
        // Render document content
        self.backend.render_items(&sorted_items, &mut context)?;
        
        // Render preview if present
        if let Some(preview_data) = preview {
            self.backend.render_preview(preview_data, &mut context)?;
        }
        
        // End frame and present
        self.backend.end_frame()?;
        
        Ok(self.stats.clone())
    }
    
    /// Render selection overlay separately (for editor)
    pub fn render_selection_overlay(
        &mut self,
        viewport: &Viewport,
        selected_items: &[RenderItem],
    ) -> Result<(), RenderError> {
        let mut context = RenderContext {
            viewport,
            config: &self.config,
            stats: &mut self.stats,
        };
        
        self.backend.begin_frame(&mut context)?;
        self.backend.render_selection(selected_items, &mut context)?;
        self.backend.end_frame()
    }
    
    /// Cull items outside the visible viewport
    fn cull_items<'a>(&self, items: &'a [RenderItem], viewport: &Viewport) -> Vec<&'a RenderItem> {
        let visible_rect = viewport.visible_world_rect();
        let margin = 50.0 / viewport.camera.zoom; // Extra margin for strokes/effects
        
        let cull_rect = Rect::new(
            Point2::new(visible_rect.min.x - margin, visible_rect.min.y - margin),
            Point2::new(visible_rect.max.x + margin, visible_rect.max.y + margin),
        );
        
        items
            .iter()
            .filter(|item| item.bounding_box.intersects(creative_geometry::BoundingBox::new(cull_rect.min, cull_rect.max)))
            .collect()
    }
    
    /// Get current render config
    pub fn config(&self) -> &RenderConfig {
        &self.config
    }
    
    /// Get current render stats
    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }
    
    /// Render items directly (for when caller handles culling/sorting)
    pub fn render_items_direct(
        &mut self,
        viewport: &Viewport,
        items: &[RenderItem],
        preview: Option<&PreviewData>,
    ) -> Result<RenderStats, RenderError> {
        self.frame_count += 1;
        self.stats = RenderStats::default();
        
        let mut context = RenderContext {
            viewport,
            config: &self.config,
            stats: &mut self.stats,
        };
        
        self.backend.begin_frame(&mut context)?;
        
        if self.config.enable_grid {
            self.backend.render_grid(&mut context)?;
        }
        
        self.backend.render_items(items, &mut context)?;
        
        if let Some(preview_data) = preview {
            self.backend.render_preview(preview_data, &mut context)?;
        }
        
        self.backend.end_frame()?;
        
        Ok(self.stats.clone())
    }
    
    /// Get backend type
    pub fn backend_type(&self) -> RenderBackend {
        self.backend.backend_type()
    }
    
    /// Get device info
    pub fn device_info(&self) -> String {
        self.backend.device_info()
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: RenderConfig) -> Result<(), RenderError> {
        let needs_reinit = config.backend != self.config.backend
            || config.msaa_samples != self.config.msaa_samples;
        
        self.config = config;
        
        if needs_reinit {
            self.backend.initialize(&self.config)?;
        }
        
        Ok(())
    }
    
    /// Check if a feature is supported
    pub fn supports_feature(&self, feature: RenderFeature) -> bool {
        self.backend.supports_feature(feature)
    }
}

// Re-export backend modules
#[cfg(feature = "wgpu")]
// Backend implementations are in separate crates:
// - creative-renderer-wgpu for GPU rendering
// - creative-renderer-software for CPU rendering

// Re-export the traits and types needed by backends
pub use crate::{RenderBackendImpl, RenderBackend, RenderConfig, RenderContext, RenderError, RenderItem, PreviewData, Viewport, RenderLayer, RenderFeature, RenderStats};

/// Builder for RenderItems from geometry
pub struct RenderItemBuilder;

impl RenderItemBuilder {
    pub fn from_geometry(
        object_id: creative_domain::ObjectId,
        geometry: &Geometry,
        layer: RenderLayer,
        z_index: i32,
        is_selected: bool,
        is_hovered: bool,
        opacity: f32,
        blend_mode: creative_domain::BlendMode,
    ) -> RenderItem {
        let (fill, stroke, transform, bbox) = match geometry {
            Geometry::Path(g) => (g.fill, g.stroke.clone(), g.transform, g.bounding_box()),
            Geometry::Shape(g) => (g.fill, g.stroke.clone(), g.transform, g.bounding_box()),
            Geometry::Text(g) => (g.fill, g.stroke.clone(), g.transform, g.bounding_box()),
            Geometry::Image(g) => (None, None, g.transform, g.bounding_box()),
            Geometry::Group(g) => (None, None, g.transform, g.bounding_box()),
        };
        
        // Convert creative_domain::Rect to BoundingBox
        let geometry_bbox = creative_geometry::BoundingBox::new(bbox.min, bbox.max);
        
        RenderItem {
            object_id,
            geometry_id: geometry.id(),
            layer,
            z_index,
            bounding_box: geometry_bbox,
            transform,
            fill,
            stroke,
            opacity,
            blend_mode,
            clip_rect: None,
            is_selected,
            is_hovered,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{ObjectId, Point2, Rect};
    use creative_geometry::{Geometry, ShapeGeometry, ShapeType};

    #[test]
    fn test_render_config_default() {
        let config = RenderConfig::default();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.backend, RenderBackend::Wgpu);
    }
    
    #[test]
    fn test_viewport_transforms() {
        let camera = Camera::new();
        let viewport = Viewport::new(800, 600, 1.0, camera);
        
        let world = Point2::new(100.0, 100.0);
        let screen = viewport.world_to_screen(world);
        let back = viewport.screen_to_world(screen);
        
        assert!((back.x - world.x).abs() < 0.001);
        assert!((back.y - world.y).abs() < 0.001);
    }
    
    #[test]
    fn test_viewport_visible_rect() {
        let camera = Camera::new();
        let viewport = Viewport::new(800, 600, 1.0, camera);
        let rect = viewport.visible_world_rect();
        
        // With default camera at (0,0) zoom 1.0, visible rect should be centered
        assert!((rect.center().x - 0.0).abs() < 1.0);
        assert!((rect.center().y - 0.0).abs() < 1.0);
        assert!((rect.width() - 800.0).abs() < 1.0);
        assert!((rect.height() - 600.0).abs() < 1.0);
    }
    
    #[test]
    fn test_render_item_builder() {
        let obj_id = ObjectId::new();
        let rect = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        let shape = ShapeGeometry::new(ShapeType::Rectangle, rect);
        let geometry = Geometry::Shape(shape);
        
        let item = RenderItemBuilder::from_geometry(
            obj_id,
            &geometry,
            RenderLayer::DocumentContent,
            0,
            false,
            false,
            1.0,
            creative_domain::BlendMode::Normal,
        );
        
        assert_eq!(item.object_id, obj_id);
        assert_eq!(item.layer, RenderLayer::DocumentContent);
        assert_eq!(item.z_index, 0);
        assert!(!item.is_selected);
    }
    
    #[test]
    fn test_render_layer_ordering() {
        assert!((RenderLayer::ArtboardBackground as u8) < (RenderLayer::DocumentContent as u8));
        assert!((RenderLayer::DocumentContent as u8) < (RenderLayer::Precision as u8));
        assert!((RenderLayer::Precision as u8) < (RenderLayer::Selection as u8));
        assert!((RenderLayer::Selection as u8) < (RenderLayer::CursorFeedback as u8));
        assert!((RenderLayer::CursorFeedback as u8) < (RenderLayer::TransientPreview as u8));
        assert!((RenderLayer::TransientPreview as u8) < (RenderLayer::AiOverlay as u8));
    }
}