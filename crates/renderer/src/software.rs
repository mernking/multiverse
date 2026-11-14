//! Software renderer backend using tiny-skia

use super::*;
use tiny_skia::{Canvas, Color as SkiaColor, Paint, Path as SkiaPath, PathBuilder, Stroke, Transform as SkiaTransform, Pixmap, PixmapMut, Shader, SpreadMode};
use std::sync::Arc;

pub struct SoftwareBackend {
    pixmap: Option<Pixmap>,
    config: Option<RenderConfig>,
    width: u32,
    height: u32,
    scale_factor: f32,
}

impl SoftwareBackend {
    pub fn new() -> Result<Self, RenderError> {
        Ok(Self {
            pixmap: None,
            config: None,
            width: 0,
            height: 0,
            scale_factor: 1.0,
        })
    }
    
    fn ensure_pixmap(&mut self) -> Result<(), RenderError> {
        if self.pixmap.is_none() || self.pixmap.as_ref().unwrap().width() != self.width || self.pixmap.as_ref().unwrap().height() != self.height {
            self.pixmap = Some(Pixmap::new(self.width, self.height).ok_or(RenderError::OutOfMemory("Failed to create pixmap".to_string()))?);
        }
        Ok(())
    }
    
    fn skia_color(color: Color) -> SkiaColor {
        SkiaColor::from_rgba8(
            (color.r.clamp(0.0, 1.0) * 255.0) as u8,
            (color.g.clamp(0.0, 1.0) * 255.0) as u8,
            (color.b.clamp(0.0, 1.0) * 255.0) as u8,
            (color.a.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
    
    fn skia_transform(transform: &creative_domain::Transform2D) -> SkiaTransform {
        SkiaTransform::from_row(transform.m[0] as f32, transform.m[1] as f32, transform.m[2] as f32,
                                 transform.m[3] as f32, transform.m[4] as f32, transform.m[5] as f32)
    }
    
    fn world_to_canvas(&self, viewport: &Viewport, point: Point2) -> (f32, f32) {
        let screen = viewport.world_to_screen(point);
        (screen.x as f32, screen.y as f32)
    }
    
    fn build_skia_path(geometry: &creative_geometry::Geometry) -> Option<SkiaPath> {
        match geometry {
            creative_geometry::Geometry::Path(g) => {
                let mut pb = PathBuilder::new();
                for seg in &g.segments {
                    match seg {
                        creative_geometry::PathSegment::MoveTo(p) => pb.move_to(p.x as f32, p.y as f32),
                        creative_geometry::PathSegment::LineTo(p) => pb.line_to(p.x as f32, p.y as f32),
                        creative_geometry::PathSegment::CubicTo(c1, c2, p) => pb.cubic_to(c1.x as f32, c1.y as f32, c2.x as f32, c2.y as f32, p.x as f32, p.y as f32),
                        creative_geometry::PathSegment::ClosePath => pb.close(),
                    }
                }
                pb.finish()
            }
            creative_geometry::Geometry::Shape(g) => {
                let mut pb = PathBuilder::new();
                let rect = &g.rect;
                match g.shape_type {
                    creative_geometry::ShapeType::Rectangle => {
                        pb.move_to(rect.min.x as f32, rect.min.y as f32);
                        pb.line_to(rect.max.x as f32, rect.min.y as f32);
                        pb.line_to(rect.max.x as f32, rect.max.y as f32);
                        pb.line_to(rect.min.x as f32, rect.max.y as f32);
                        pb.close();
                    }
                    creative_geometry::ShapeType::Ellipse => {
                        let cx = (rect.min.x + rect.max.x) * 0.5;
                        let cy = (rect.min.y + rect.max.y) * 0.5;
                        let rx = (rect.max.x - rect.min.x) * 0.5;
                        let ry = (rect.max.y - rect.min.y) * 0.5;
                        // Approximate ellipse with cubic curves
                        let k = 0.5522847498;
                        pb.move_to(cx as f32, (cy - ry) as f32);
                        pb.cubic_to((cx + rx * k) as f32, (cy - ry) as f32, (cx + rx) as f32, (cy - ry * k) as f32, (cx + rx) as f32, cy as f32);
                        pb.cubic_to((cx + rx) as f32, (cy + ry * k) as f32, (cx + rx * k) as f32, (cy + ry) as f32, cx as f32, (cy + ry) as f32);
                        pb.cubic_to((cx - rx * k) as f32, (cy + ry) as f32, (cx - rx) as f32, (cy + ry * k) as f32, (cx - rx) as f32, cy as f32);
                        pb.cubic_to((cx - rx) as f32, (cy - ry * k) as f32, (cx - rx * k) as f32, (cy - ry) as f32, cx as f32, (cy - ry) as f32);
                        pb.close();
                    }
                    creative_geometry::ShapeType::Line => {
                        pb.move_to(rect.min.x as f32, rect.min.y as f32);
                        pb.line_to(rect.max.x as f32, rect.max.y as f32);
                    }
                    creative_geometry::ShapeType::RoundedRectangle { radius } => {
                        let r = radius as f32;
                        let min_x = rect.min.x as f32;
                        let min_y = rect.min.y as f32;
                        let max_x = rect.max.x as f32;
                        let max_y = rect.max.y as f32;
                        pb.move_to(min_x + r, min_y);
                        pb.line_to(max_x - r, min_y);
                        pb.quad_to(max_x, min_y, max_x, min_y + r);
                        pb.line_to(max_x, max_y - r);
                        pb.quad_to(max_x, max_y, max_x - r, max_y);
                        pb.line_to(min_x + r, max_y);
                        pb.quad_to(min_x, max_y, min_x, max_y - r);
                        pb.line_to(min_x, min_y + r);
                        pb.quad_to(min_x, min_y, min_x + r, min_y);
                        pb.close();
                    }
                    creative_geometry::ShapeType::Polygon { sides } => {
                        let cx = (rect.min.x + rect.max.x) * 0.5;
                        let cy = (rect.min.y + rect.max.y) * 0.5;
                        let r = ((rect.max.x - rect.min.x).min(rect.max.y - rect.min.y) * 0.5) as f32;
                        let sides = sides.max(3) as usize;
                        for i in 0..sides {
                            let angle = (i as f64 * 2.0 * std::f64::consts::PI / sides as f64) - std::f64::consts::PI / 2.0;
                            let x = cx + angle.cos() * r as f64;
                            let y = cy + angle.sin() * r as f64;
                            if i == 0 { pb.move_to(x as f32, y as f32); } else { pb.line_to(x as f32, y as f32); }
                        }
                        pb.close();
                    }
                    creative_geometry::ShapeType::Star { points, inner_radius } => {
                        let cx = (rect.min.x + rect.max.x) * 0.5;
                        let cy = (rect.min.y + rect.max.y) * 0.5;
                        let outer_r = ((rect.max.x - rect.min.x).min(rect.max.y - rect.min.y) * 0.5) as f32;
                        let inner_r = outer_r * (inner_radius as f32 / 100.0);
                        let points = points.max(3) as usize;
                        for i in 0..(points * 2) {
                            let angle = (i as f64 * std::f64::consts::PI / points as f64) - std::f64::consts::PI / 2.0;
                            let r = if i % 2 == 0 { outer_r } else { inner_r };
                            let x = cx + angle.cos() * r as f64;
                            let y = cy + angle.sin() * r as f64;
                            if i == 0 { pb.move_to(x as f32, y as f32); } else { pb.line_to(x as f32, y as f32); }
                        }
                        pb.close();
                    }
                }
                pb.finish()
            }
            creative_geometry::Geometry::Text(g) => {
                // Text rendering would need a font library - placeholder rect for now
                let mut pb = PathBuilder::new();
                let rect = g.bounding_box();
                pb.move_to(rect.min.x as f32, rect.min.y as f32);
                pb.line_to(rect.max.x as f32, rect.min.y as f32);
                pb.line_to(rect.max.x as f32, rect.max.y as f32);
                pb.line_to(rect.min.x as f32, rect.max.y as f32);
                pb.close();
                pb.finish()
            }
            creative_geometry::Geometry::Image(g) => {
                // Image rendering placeholder - draw rect
                let mut pb = PathBuilder::new();
                let rect = &g.rect;
                pb.move_to(rect.min.x as f32, rect.min.y as f32);
                pb.line_to(rect.max.x as f32, rect.min.y as f32);
                pb.line_to(rect.max.x as f32, rect.max.y as f32);
                pb.line_to(rect.min.x as f32, rect.max.y as f32);
                pb.close();
                pb.finish()
            }
            creative_geometry::Geometry::Group(_) => None,
        }
    }
    
    fn create_stroke(style: &StrokeStyle) -> Stroke {
        let mut stroke = Stroke::default();
        stroke.width = style.width;
        stroke.line_cap = match style.cap {
            creative_geometry::StrokeCap::Butt => tiny_skia::LineCap::Butt,
            creative_geometry::StrokeCap::Round => tiny_skia::LineCap::Round,
            creative_geometry::StrokeCap::Square => tiny_skia::LineCap::Square,
        };
        stroke.line_join = match style.join {
            creative_geometry::StrokeJoin::Miter => tiny_skia::LineJoin::Miter,
            creative_geometry::StrokeJoin::Round => tiny_skia::LineJoin::Round,
            creative_geometry::StrokeJoin::Bevel => tiny_skia::LineJoin::Bevel,
        };
        if !style.dash_pattern.is_empty() {
            stroke.dash_pattern = style.dash_pattern.iter().map(|&v| v).collect();
            stroke.dash_offset = style.dash_offset;
        }
        stroke
    }
    
    fn render_item(&self, canvas: &mut Canvas, item: &RenderItem, viewport: &Viewport) {
        // Apply object transform
        let transform = self.skia_transform(&item.transform);
        
        // Apply viewport transform (world to screen)
        let camera = &viewport.camera;
        let scale = (camera.zoom * viewport.scale_factor as f64) as f32;
        let viewport_transform = SkiaTransform::from_scale(scale, scale)
            .post_translate(viewport.width as f32 * 0.5, viewport.height as f32 * 0.5)
            .post_translate(-(camera.center.x as f32 * scale), -(camera.center.y as f32 * scale));
        
        let total_transform = viewport_transform * transform;
        
        if let Some(path) = Self::build_skia_path(&creative_geometry::Geometry::Group(creative_geometry::GroupGeometry::new(vec![]))) {
            // This is a workaround - we need the actual geometry
            // In real implementation, we'd look up geometry by ID
        }
        
        // For now, we can't render without the actual geometry
        // The RenderItem should carry the geometry or we need a geometry registry
    }
}

impl RenderBackendImpl for SoftwareBackend {
    fn initialize(&mut self, config: &RenderConfig) -> Result<(), RenderError> {
        self.config = Some(config.clone());
        self.width = config.width;
        self.height = config.height;
        self.scale_factor = config.scale_factor;
        self.ensure_pixmap()
    }
    
    fn resize(&mut self, width: u32, height: u32, scale_factor: f32) -> Result<(), RenderError> {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        if let Some(config) = &mut self.config {
            config.width = width;
            config.height = height;
            config.scale_factor = scale_factor;
        }
        self.pixmap = None;
        self.ensure_pixmap()
    }
    
    fn begin_frame(&mut self, context: &RenderContext) -> Result<(), RenderError> {
        self.ensure_pixmap()?;
        if let Some(pixmap) = &mut self.pixmap {
            let bg = &context.config.background_color;
            pixmap.fill(Self::skia_color(*bg));
        }
        Ok(())
    }
    
    fn end_frame(&mut self) -> Result<(), RenderError> {
        // Nothing special needed for software renderer
        Ok(())
    }
    
    fn render_items(&mut self, items: &[RenderItem], context: &RenderContext) -> Result<(), RenderError> {
        let Some(pixmap) = &mut self.pixmap else {
            return Err(RenderError::NotInitialized);
        };
        
        let mut canvas = Canvas::from(pixmap.as_mut()).ok_or(RenderError::OutOfMemory("Failed to create canvas".to_string()))?;
        
        for item in items {
            // Render fill
            if let Some(fill_color) = item.fill {
                if let Some(path) = Self::build_skia_path_for_item(item) {
                    let mut paint = Paint::default();
                    paint.set_color(Self::skia_color(fill_color));
                    paint.anti_alias = true;
                    paint.blend_mode = Self::skia_blend_mode(item.blend_mode);
                    canvas.fill_path(&path, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
                }
            }
            
            // Render stroke
            if let Some(stroke_style) = &item.stroke {
                if let Some(path) = Self::build_skia_path_for_item(item) {
                    let mut paint = Paint::default();
                    paint.set_color(Self::skia_color(stroke_style.color));
                    paint.anti_alias = true;
                    paint.blend_mode = Self::skia_blend_mode(item.blend_mode);
                    let stroke = Self::create_stroke(stroke_style);
                    canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
                }
            }
            
            // Render selection highlight
            if item.is_selected {
                if let Some(path) = Self::build_skia_path_for_item(item) {
                    let mut paint = Paint::default();
                    paint.set_color(SkiaColor::from_rgba8(0, 120, 255, 200));
                    paint.anti_alias = true;
                    let stroke = Stroke { width: 2.0 / (context.viewport.camera.zoom as f32 * context.viewport.scale_factor), ..Stroke::default() };
                    canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
                }
            }
            
            context.stats.draw_calls += 1;
        }
        
        Ok(())
    }
    
    fn render_grid(&mut self, context: &RenderContext) -> Result<(), RenderError> {
        let Some(pixmap) = &mut self.pixmap else {
            return Err(RenderError::NotInitialized);
        };
        
        let mut canvas = Canvas::from(pixmap.as_mut()).ok_or(RenderError::OutOfMemory("Failed to create canvas".to_string()))?;
        let config = context.config;
        let viewport = context.viewport;
        
        let spacing = config.grid_spacing * viewport.camera.zoom;
        if spacing < 10.0 { return Ok(()); } // Too small to see
        
        let visible_rect = viewport.visible_world_rect();
        let start_x = (visible_rect.min.x / config.grid_spacing).floor() * config.grid_spacing;
        let start_y = (visible_rect.min.y / config.grid_spacing).floor() * config.grid_spacing;
        let end_x = visible_rect.max.x;
        let end_y = visible_rect.max.y;
        
        let grid_color = Self::skia_color(config.grid_color);
        let mut paint = Paint::default();
        paint.set_color(grid_color);
        paint.anti_alias = true;
        let stroke = Stroke { width: 1.0 / (viewport.camera.zoom as f32 * viewport.scale_factor), ..Stroke::default() };
        
        let mut x = start_x;
        while x <= end_x {
            let p1 = viewport.world_to_screen(Point2::new(x, visible_rect.min.y));
            let p2 = viewport.world_to_screen(Point2::new(x, visible_rect.max.y));
            let mut pb = PathBuilder::new();
            pb.move_to(p1.x as f32, p1.y as f32);
            pb.line_to(p2.x as f32, p2.y as f32);
            if let Some(path) = pb.finish() {
                canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
            }
            x += config.grid_spacing;
        }
        
        let mut y = start_y;
        while y <= end_y {
            let p1 = viewport.world_to_screen(Point2::new(visible_rect.min.x, y));
            let p2 = viewport.world_to_screen(Point2::new(visible_rect.max.x, y));
            let mut pb = PathBuilder::new();
            pb.move_to(p1.x as f32, p1.y as f32);
            pb.line_to(p2.x as f32, p2.y as f32);
            if let Some(path) = pb.finish() {
                canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
            }
            y += config.grid_spacing;
        }
        
        Ok(())
    }
    
    fn render_selection(&mut self, items: &[RenderItem], context: &RenderContext) -> Result<(), RenderError> {
        let Some(pixmap) = &mut self.pixmap else {
            return Err(RenderError::NotInitialized);
        };
        
        let mut canvas = Canvas::from(pixmap.as_mut()).ok_or(RenderError::OutOfMemory("Failed to create canvas".to_string()))?;
        
        for item in items {
            if let Some(path) = Self::build_skia_path_for_item(item) {
                let mut paint = Paint::default();
                paint.set_color(SkiaColor::from_rgba8(0, 120, 255, 200));
                paint.anti_alias = true;
                let stroke = Stroke { width: 2.0 / (context.viewport.camera.zoom as f32 * context.viewport.scale_factor), ..Stroke::default() };
                canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
                
                // Draw bounding box handles
                let bbox = &item.bounding_box;
                let corners = [
                    Point2::new(bbox.min.x, bbox.min.y),
                    Point2::new(bbox.max.x, bbox.min.y),
                    Point2::new(bbox.max.x, bbox.max.y),
                    Point2::new(bbox.min.x, bbox.max.y),
                ];
                
                for corner in corners {
                    let screen = context.viewport.world_to_screen(corner);
                    let mut pb = PathBuilder::new();
                    let handle_size = 6.0 / (context.viewport.camera.zoom as f32 * context.viewport.scale_factor);
                    pb.move_to(screen.x as f32 - handle_size, screen.y as f32 - handle_size);
                    pb.line_to(screen.x as f32 + handle_size, screen.y as f32 - handle_size);
                    pb.line_to(screen.x as f32 + handle_size, screen.y as f32 + handle_size);
                    pb.line_to(screen.x as f32 - handle_size, screen.y as f32 + handle_size);
                    pb.close();
                    if let Some(handle_path) = pb.finish() {
                        paint.set_color(SkiaColor::from_rgba8(255, 255, 255, 255));
                        canvas.fill_path(&handle_path, &paint, tiny_skia::FillRule::Winding, tiny_skia::Transform::identity(), None);
                        paint.set_color(SkiaColor::from_rgba8(0, 120, 255, 255));
                        canvas.stroke_path(&handle_path, &paint, &Stroke { width: 1.0, ..Stroke::default() }, tiny_skia::Transform::identity(), None);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn render_preview(&mut self, preview: &PreviewData, context: &RenderContext) -> Result<(), RenderError> {
        let Some(pixmap) = &mut self.pixmap else {
            return Err(RenderError::NotInitialized);
        };
        
        if preview.points.len() < 2 {
            return Ok(());
        }
        
        let mut canvas = Canvas::from(pixmap.as_mut()).ok_or(RenderError::OutOfMemory("Failed to create canvas".to_string()))?;
        
        let mut pb = PathBuilder::new();
        pb.move_to(preview.points[0].x as f32, preview.points[0].y as f32);
        for p in &preview.points[1..] {
            pb.line_to(p.x as f32, p.y as f32);
        }
        
        if let Some(path) = pb.finish() {
            let mut paint = Paint::default();
            paint.set_color(Self::skia_color(preview.stroke_style.color));
            paint.anti_alias = true;
            let stroke = Self::create_stroke(&preview.stroke_style);
            canvas.stroke_path(&path, &paint, &stroke, tiny_skia::Transform::identity(), None);
        }
        
        Ok(())
    }
    
    fn backend_type(&self) -> RenderBackend {
        RenderBackend::Software
    }
    
    fn supports_feature(&self, feature: RenderFeature) -> bool {
        matches!(feature, RenderFeature::MSAA)
    }
    
    fn device_info(&self) -> String {
        format!("Software Renderer (tiny-skia) - {}x{}", self.width, self.height)
    }
}

impl SoftwareBackend {
    fn skia_blend_mode(mode: creative_domain::BlendMode) -> tiny_skia::BlendMode {
        match mode {
            creative_domain::BlendMode::Normal => tiny_skia::BlendMode::SourceOver,
            creative_domain::BlendMode::Multiply => tiny_skia::BlendMode::Multiply,
            creative_domain::BlendMode::Screen => tiny_skia::BlendMode::Screen,
            creative_domain::BlendMode::Overlay => tiny_skia::BlendMode::Overlay,
            creative_domain::BlendMode::Darken => tiny_skia::BlendMode::Darken,
            creative_domain::BlendMode::Lighten => tiny_skia::BlendMode::Lighten,
            creative_domain::BlendMode::ColorDodge => tiny_skia::BlendMode::ColorDodge,
            creative_domain::BlendMode::ColorBurn => tiny_skia::BlendMode::ColorBurn,
            creative_domain::BlendMode::HardLight => tiny_skia::BlendMode::HardLight,
            creative_domain::BlendMode::SoftLight => tiny_skia::BlendMode::SoftLight,
            creative_domain::BlendMode::Difference => tiny_skia::BlendMode::Difference,
            creative_domain::BlendMode::Exclusion => tiny_skia::BlendMode::Exclusion,
        }
    }
    
    fn build_skia_path_for_item(item: &RenderItem) -> Option<SkiaPath> {
        // We need to reconstruct path from the item's data
        // For now, create a simple rect path from bounding box
        let bbox = &item.bounding_box;
        let mut pb = PathBuilder::new();
        pb.move_to(bbox.min.x as f32, bbox.min.y as f32);
        pb.line_to(bbox.max.x as f32, bbox.min.y as f32);
        pb.line_to(bbox.max.x as f32, bbox.max.y as f32);
        pb.line_to(bbox.min.x as f32, bbox.max.y as f32);
        pb.close();
        pb.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderConfig, RenderBackend};
    
    #[test]
    fn test_software_backend_creation() {
        let backend = SoftwareBackend::new();
        assert!(backend.is_ok());
    }
    
    #[test]
    fn test_software_backend_initialize() {
        let mut backend = SoftwareBackend::new().unwrap();
        let config = RenderConfig::default();
        let result = backend.initialize(&config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_software_backend_resize() {
        let mut backend = SoftwareBackend::new().unwrap();
        backend.initialize(&RenderConfig::default()).unwrap();
        let result = backend.resize(1024, 768, 1.0);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_skia_color_conversion() {
        let color = Color::new(1.0, 0.5, 0.0, 0.75);
        let skia = SoftwareBackend::skia_color(color);
        assert_eq!(skia.red(), 255);
        assert_eq!(skia.green(), 128);
        assert_eq!(skia.blue(), 0);
        assert_eq!(skia.alpha(), 191);
    }
}