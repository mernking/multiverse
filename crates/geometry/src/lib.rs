//! Geometry crate for Creative Engine
//!
//! Provides core geometric types: paths, shapes, bounding boxes, and spatial operations.

use creative_domain::Point2;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a geometry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GeometryId(pub uuid::Uuid);

impl GeometryId {
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl Default for GeometryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GeometryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Bounding box for spatial queries
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Point2,
    pub max: Point2,
}

impl BoundingBox {
    pub fn new(min: Point2, max: Point2) -> Self {
        Self {
            min: Point2::new(min.x.min(max.x), min.y.min(max.y)),
            max: Point2::new(min.x.max(max.x), min.y.max(max.y)),
        }
    }

    pub fn from_points(points: &[Point2]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut min = points[0];
        let mut max = points[0];
        for p in points {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Some(Self::new(min, max))
    }

    pub fn width(&self) -> f64 { self.max.x - self.min.x }
    pub fn height(&self) -> f64 { self.max.y - self.min.y }
    pub fn center(&self) -> Point2 {
        Point2::new((self.min.x + self.max.x) * 0.5, (self.min.y + self.max.y) * 0.5)
    }
    pub fn contains(&self, point: Point2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x && point.y >= self.min.y && point.y <= self.max.y
    }
    pub fn intersects(&self, other: BoundingBox) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }
    pub fn union(&self, other: BoundingBox) -> BoundingBox {
        BoundingBox::new(
            Point2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            Point2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        )
    }
    pub fn expand(&self, margin: f64) -> BoundingBox {
        BoundingBox::new(
            Point2::new(self.min.x - margin, self.min.y - margin),
            Point2::new(self.max.x + margin, self.max.y + margin),
        )
    }
}

/// Path segment types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathSegment {
    MoveTo(Point2),
    LineTo(Point2),
    CubicTo(Point2, Point2, Point2),
    ClosePath,
}

/// Fill rule for paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

/// Stroke style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeStyle {
    pub width: f32,
    pub color: creative_domain::Color,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub dash_pattern: Vec<f32>,
    pub dash_offset: f32,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: creative_domain::Color::BLACK,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            dash_pattern: Vec::new(),
            dash_offset: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeCap { Butt, Round, Square }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeJoin { Miter, Round, Bevel }

/// Geometry types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Geometry {
    Path(PathGeometry),
    Shape(ShapeGeometry),
    Text(TextGeometry),
    Image(ImageGeometry),
    Group(GroupGeometry),
}

impl Geometry {
    pub fn id(&self) -> GeometryId {
        match self {
            Geometry::Path(g) => g.id,
            Geometry::Shape(g) => g.id,
            Geometry::Text(g) => g.id,
            Geometry::Image(g) => g.id,
            Geometry::Group(g) => g.id,
        }
    }

    pub fn bounding_box(&self) -> creative_domain::Rect {
        match self {
            Geometry::Path(g) => g.bounding_box(),
            Geometry::Shape(g) => g.bounding_box(),
            Geometry::Text(g) => g.bounding_box(),
            Geometry::Image(g) => g.bounding_box(),
            Geometry::Group(g) => g.bounding_box(),
        }
    }

    pub fn transform(&mut self, transform: creative_domain::Transform2D) {
        match self {
            Geometry::Path(g) => g.transform(transform),
            Geometry::Shape(g) => g.transform(transform),
            Geometry::Text(g) => g.transform(transform),
            Geometry::Image(g) => g.transform(transform),
            Geometry::Group(g) => g.transform(transform),
        }
    }
}

/// Path geometry (vector paths)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathGeometry {
    pub id: GeometryId,
    pub segments: Vec<PathSegment>,
    pub fill: Option<creative_domain::Color>,
    pub stroke: Option<StrokeStyle>,
    pub fill_rule: FillRule,
    pub transform: creative_domain::Transform2D,
}

impl PathGeometry {
    pub fn new() -> Self {
        Self {
            id: GeometryId::new(),
            segments: Vec::new(),
            fill: None,
            stroke: Some(StrokeStyle::default()),
            fill_rule: FillRule::NonZero,
            transform: creative_domain::Transform2D::IDENTITY,
        }
    }

    pub fn bounding_box(&self) -> creative_domain::Rect {
        let mut points = Vec::new();
        for seg in &self.segments {
            match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => points.push(*p),
                PathSegment::CubicTo(_, _, p) => points.push(*p),
                PathSegment::ClosePath => {},
            }
        }
        BoundingBox::from_points(&points)
            .map(|bb| creative_domain::Rect::new(bb.min, bb.max))
            .unwrap_or(creative_domain::Rect::new(creative_domain::Point2::new(0.0, 0.0), creative_domain::Point2::new(0.0, 0.0)))
    }

    pub fn transform(&mut self, transform: creative_domain::Transform2D) {
        self.transform = transform * self.transform;
        for seg in &mut self.segments {
            match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => *p = transform.transform_point(*p),
                PathSegment::CubicTo(c1, c2, p) => {
                    *c1 = transform.transform_point(*c1);
                    *c2 = transform.transform_point(*c2);
                    *p = transform.transform_point(*p);
                }
                PathSegment::ClosePath => {},
            }
        }
    }
}

impl Default for PathGeometry {
    fn default() -> Self { Self::new() }
}

/// Shape geometry (primitive shapes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeGeometry {
    pub id: GeometryId,
    pub shape_type: ShapeType,
    pub rect: creative_domain::Rect,
    pub fill: Option<creative_domain::Color>,
    pub stroke: Option<StrokeStyle>,
    pub transform: creative_domain::Transform2D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    RoundedRectangle { radius: u32 },
    Polygon { sides: u32 },
    Star { points: u32, inner_radius: u32 },
}

impl ShapeGeometry {
    pub fn new(shape_type: ShapeType, rect: creative_domain::Rect) -> Self {
        Self {
            id: GeometryId::new(),
            shape_type,
            rect,
            fill: None,
            stroke: Some(StrokeStyle::default()),
            transform: creative_domain::Transform2D::IDENTITY,
        }
    }

    pub fn bounding_box(&self) -> creative_domain::Rect {
        self.rect
    }

    pub fn transform(&mut self, transform: creative_domain::Transform2D) {
        self.transform = transform * self.transform;
        let center = self.rect.center();
        let new_center = transform.transform_point(center);
        let half_w = self.rect.width() * 0.5;
        let half_h = self.rect.height() * 0.5;
        self.rect = creative_domain::Rect::new(
            creative_domain::Point2::new(new_center.x - half_w, new_center.y - half_h),
            creative_domain::Point2::new(new_center.x + half_w, new_center.y + half_h),
        );
    }
}

/// Text geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGeometry {
    pub id: GeometryId,
    pub content: String,
    pub font_family: String,
    pub font_size: f32,
    pub position: creative_domain::Point2,
    pub fill: Option<creative_domain::Color>,
    pub stroke: Option<StrokeStyle>,
    pub transform: creative_domain::Transform2D,
}

impl TextGeometry {
    pub fn new(content: String, position: creative_domain::Point2) -> Self {
        Self {
            id: GeometryId::new(),
            content,
            font_family: "sans-serif".to_string(),
            font_size: 16.0,
            position,
            fill: Some(creative_domain::Color::BLACK),
            stroke: None,
            transform: creative_domain::Transform2D::IDENTITY,
        }
    }

    pub fn bounding_box(&self) -> creative_domain::Rect {
        let width = self.content.len() as f64 * self.font_size as f64 * 0.6;
        let height = self.font_size as f64 * 1.2;
        creative_domain::Rect::new(
            self.position,
            creative_domain::Point2::new(self.position.x + width, self.position.y + height),
        )
    }

    pub fn transform(&mut self, transform: creative_domain::Transform2D) {
        self.transform = transform * self.transform;
        self.position = transform.transform_point(self.position);
    }
}

/// Image geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGeometry {
    pub id: GeometryId,
    pub image_id: uuid::Uuid,
    pub rect: creative_domain::Rect,
    pub opacity: f32,
    pub transform: creative_domain::Transform2D,
}

impl ImageGeometry {
    pub fn new(image_id: uuid::Uuid, rect: creative_domain::Rect) -> Self {
        Self {
            id: GeometryId::new(),
            image_id,
            rect,
            opacity: 1.0,
            transform: creative_domain::Transform2D::IDENTITY,
        }
    }

    pub fn bounding_box(&self) -> creative_domain::Rect { self.rect }

    pub fn transform(&mut self, transform: creative_domain::Transform2D) {
        self.transform = transform * self.transform;
        let center = self.rect.center();
        let new_center = transform.transform_point(center);
        let half_w = self.rect.width() * 0.5;
        let half_h = self.rect.height() * 0.5;
        self.rect = creative_domain::Rect::new(
            creative_domain::Point2::new(new_center.x - half_w, new_center.y - half_h),
            creative_domain::Point2::new(new_center.x + half_w, new_center.y + half_h),
        );
    }
}

/// Group geometry (container for other geometries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupGeometry {
    pub id: GeometryId,
    pub children: Vec<GeometryId>,
    pub transform: creative_domain::Transform2D,
}

impl GroupGeometry {
    pub fn new(children: Vec<GeometryId>) -> Self {
        Self {
            id: GeometryId::new(),
            children,
            transform: creative_domain::Transform2D::IDENTITY,
        }
    }

    pub fn bounding_box(&self) -> creative_domain::Rect {
        creative_domain::Rect::new(creative_domain::Point2::new(0.0, 0.0), creative_domain::Point2::new(0.0, 0.0))
    }

    pub fn transform(&mut self, transform: creative_domain::Transform2D) {
        self.transform = transform * self.transform;
    }
}

/// Geometry registry for managing geometry objects
#[derive(Debug, Default)]
pub struct GeometryRegistry {
    geometries: HashMap<GeometryId, Geometry>,
}

impl GeometryRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn insert(&mut self, geometry: Geometry) -> GeometryId {
        let id = geometry.id();
        self.geometries.insert(id, geometry);
        id
    }

    pub fn get(&self, id: GeometryId) -> Option<&Geometry> {
        self.geometries.get(&id)
    }

    pub fn get_mut(&mut self, id: GeometryId) -> Option<&mut Geometry> {
        self.geometries.get_mut(&id)
    }

    pub fn remove(&mut self, id: GeometryId) -> Option<Geometry> {
        self.geometries.remove(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&GeometryId, &Geometry)> {
        self.geometries.iter()
    }

    pub fn len(&self) -> usize { self.geometries.len() }
    pub fn is_empty(&self) -> bool { self.geometries.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{Point2, Rect};

    #[test]
    fn test_geometry_id() {
        let id1 = GeometryId::new();
        let id2 = GeometryId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_bounding_box() {
        let bb = BoundingBox::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        assert_eq!(bb.width(), 100.0);
        assert_eq!(bb.height(), 100.0);
        assert_eq!(bb.center(), Point2::new(50.0, 50.0));
        assert!(bb.contains(Point2::new(50.0, 50.0)));
        assert!(!bb.contains(Point2::new(150.0, 50.0)));
    }

    #[test]
    fn test_path_geometry() {
        let mut path = PathGeometry::new();
        path.segments.push(PathSegment::MoveTo(Point2::new(0.0, 0.0)));
        path.segments.push(PathSegment::LineTo(Point2::new(100.0, 100.0)));
        let bbox = path.bounding_box();
        assert!(bbox.width() > 0.0);
        assert!(bbox.height() > 0.0);
    }

    #[test]
    fn test_shape_geometry() {
        let rect = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 50.0));
        let shape = ShapeGeometry::new(ShapeType::Rectangle, rect);
        assert_eq!(shape.rect, rect);
        assert_eq!(shape.shape_type, ShapeType::Rectangle);
    }
}
