//! Core domain entities and invariants for Creative Engine
//!
//! This crate contains the fundamental types that represent the document model.
//! It has no dependencies on UI frameworks, rendering backends, or platform specifics.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(pub Uuid);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for any object in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub Uuid);

impl ObjectId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayerId(pub Uuid);

impl LayerId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl fmt::Display for LayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a brush preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BrushId(pub Uuid);

impl BrushId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl fmt::Display for BrushId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an artboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtboardId(pub Uuid);

impl ArtboardId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl fmt::Display for ArtboardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Color representation in sRGB space with alpha
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err("Invalid hex color length".to_string());
        }
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex")? as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex")? as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex")? as f32 / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| "Invalid hex")? as f32 / 255.0
        } else {
            1.0
        };
        Ok(Self { r, g, b, a })
    }

    pub fn to_hex(&self) -> String {
        let r = (self.r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (self.a.clamp(0.0, 1.0) * 255.0).round() as u8;
        if a == 255 {
            format!("#{:02X}{:02X}{:02X}", r, g, b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
        }
    }

    pub fn with_alpha(&self, a: f32) -> Self {
        Self { a, ..*self }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// 2D point in world coordinates (f64 for precision)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point2 {
    pub x: f64,
    pub y: f64,
}

impl Point2 {
    pub const ZERO: Point2 = Point2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: Point2) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Point2 {
    fn default() -> Self {
        Self::ZERO
    }
}

impl std::ops::Add for Point2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

impl std::ops::Sub for Point2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self { x: self.x - other.x, y: self.y - other.y }
    }
}

impl std::ops::Mul<f64> for Point2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar }
    }
}

impl std::ops::Div<f64> for Point2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self { x: self.x / scalar, y: self.y / scalar }
    }
}

/// 2D vector in world coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub const ZERO: Vector2 = Vector2 { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self { x: self.x / len, y: self.y / len }
        } else {
            Self::ZERO
        }
    }
}

impl std::ops::Add for Vector2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self { x: self.x + other.x, y: self.y + other.y }
    }
}

impl std::ops::Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self { x: self.x * scalar, y: self.y * scalar }
    }
}

impl std::ops::Sub<Vector2> for Point2 {
    type Output = Point2;
    fn sub(self, other: Vector2) -> Point2 {
        Point2::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Add<Vector2> for Point2 {
    type Output = Point2;
    fn add(self, other: Vector2) -> Point2 {
        Point2::new(self.x + other.x, self.y + other.y)
    }
}

/// Rectangle in world coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub min: Point2,
    pub max: Point2,
}

impl Rect {
    pub fn new(min: Point2, max: Point2) -> Self {
        Self {
            min: Point2::new(min.x.min(max.x), min.y.min(max.y)),
            max: Point2::new(min.x.max(max.x), min.y.max(max.y)),
        }
    }

    pub fn from_center_size(center: Point2, size: Vector2) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Point2 {
        Point2::new((self.min.x + self.max.x) * 0.5, (self.min.y + self.max.y) * 0.5)
    }

    pub fn contains(&self, point: Point2) -> bool {
        point.x >= self.min.x && point.x <= self.max.x && point.y >= self.min.y && point.y <= self.max.y
    }

    pub fn intersects(&self, other: Rect) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y
    }

    pub fn union(&self, other: Rect) -> Rect {
        Rect::new(
            Point2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            Point2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        )
    }
}

/// Affine transformation in 2D (stored as 3x3 matrix in row-major order)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub m: [f64; 6], // [a, b, c, d, e, f] for [a b c; d e f; 0 0 1]
}

impl Transform2D {
    pub const IDENTITY: Transform2D = Transform2D { m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0] };

    pub fn new(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64) -> Self {
        Self { m: [a, b, c, d, e, f] }
    }

    pub fn translate(x: f64, y: f64) -> Self {
        Self { m: [1.0, 0.0, x, 0.0, 1.0, y] }
    }

    pub fn scale(sx: f64, sy: f64) -> Self {
        Self { m: [sx, 0.0, 0.0, 0.0, sy, 0.0] }
    }

    pub fn uniform_scale(s: f64) -> Self {
        Self::scale(s, s)
    }

    pub fn rotate(angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self { m: [cos, -sin, 0.0, sin, cos, 0.0] }
    }

    pub fn transform_point(&self, p: Point2) -> Point2 {
        Point2::new(
            self.m[0] * p.x + self.m[1] * p.y + self.m[2],
            self.m[3] * p.x + self.m[4] * p.y + self.m[5],
        )
    }

    pub fn transform_vector(&self, v: Vector2) -> Vector2 {
        Vector2::new(
            self.m[0] * v.x + self.m[1] * v.y,
            self.m[3] * v.x + self.m[4] * v.y,
        )
    }

    pub fn inverse(&self) -> Option<Self> {
        let det = self.m[0] * self.m[4] - self.m[1] * self.m[3];
        if det.abs() < f64::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            m: [
                self.m[4] * inv_det,
                -self.m[1] * inv_det,
                (self.m[1] * self.m[5] - self.m[2] * self.m[4]) * inv_det,
                -self.m[3] * inv_det,
                self.m[0] * inv_det,
                (self.m[2] * self.m[3] - self.m[0] * self.m[5]) * inv_det,
            ],
        })
    }
}

impl std::ops::Mul for Transform2D {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Self {
            m: [
                self.m[0] * other.m[0] + self.m[1] * other.m[3],
                self.m[0] * other.m[1] + self.m[1] * other.m[4],
                self.m[0] * other.m[2] + self.m[1] * other.m[5] + self.m[2],
                self.m[3] * other.m[0] + self.m[4] * other.m[3],
                self.m[3] * other.m[1] + self.m[4] * other.m[4],
                self.m[3] * other.m[2] + self.m[4] * other.m[5] + self.m[5],
            ],
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Layer blend modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlendMode {
    Normal = 0,
    Multiply = 1,
    Screen = 2,
    Overlay = 3,
    Darken = 4,
    Lighten = 5,
    ColorDodge = 6,
    ColorBurn = 7,
    HardLight = 8,
    SoftLight = 9,
    Difference = 10,
    Exclusion = 11,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Normal
    }
}

/// Layer in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub object_ids: Vec<ObjectId>,
    pub parent: Option<LayerId>,
    pub children: Vec<LayerId>,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            visible: true,
            locked: false,
            opacity: 1.0,
            blend_mode: BlendMode::Normal,
            object_ids: Vec::new(),
            parent: None,
            children: Vec::new(),
        }
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    pub id: DocumentId,
    pub name: String,
    pub created: u64, // Unix timestamp
    pub modified: u64,
    pub version: u32, // Schema version
    pub app_version: String,
}

/// Camera state for the canvas
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Camera {
    pub center: Point2,
    pub zoom: f64,
    pub rotation: f64,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            center: Point2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
        }
    }

    pub fn world_to_screen(&self, point: Point2, viewport_size: (f64, f64), device_scale: f64) -> Point2 {
        let translated = point - self.center;
        let rotated = Point2::new(
            translated.x * self.rotation.cos() - translated.y * self.rotation.sin(),
            translated.x * self.rotation.sin() + translated.y * self.rotation.cos(),
        );
        let scaled = rotated * self.zoom * device_scale;
        Point2::new(
            scaled.x + viewport_size.0 * 0.5,
            scaled.y + viewport_size.1 * 0.5,
        )
    }

    pub fn screen_to_world(&self, point: Point2, viewport_size: (f64, f64), device_scale: f64) -> Point2 {
        let centered = Point2::new(
            point.x - viewport_size.0 * 0.5,
            point.y - viewport_size.1 * 0.5,
        );
        let scaled = centered / (self.zoom * device_scale);
        let rotated = Point2::new(
            scaled.x * self.rotation.cos() + scaled.y * self.rotation.sin(),
            -scaled.x * self.rotation.sin() + scaled.y * self.rotation.cos(),
        );
        rotated + self.center
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// Document settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSettings {
    pub background_color: Color,
    pub grid_enabled: bool,
    pub grid_type: GridType,
    pub grid_spacing: f64,
    pub grid_color: Color,
    pub grid_opacity: f32,
    pub snap_enabled: bool,
    pub snap_tolerance: f64,
    pub measurement_unit: MeasurementUnit,
    pub measurement_scale: f64, // 1 unit = N world units
}

impl Default for DocumentSettings {
    fn default() -> Self {
        Self {
            background_color: Color::WHITE,
            grid_enabled: false,
            grid_type: GridType::Dot,
            grid_spacing: 20.0,
            grid_color: Color::new(0.5, 0.5, 0.5, 0.3),
            grid_opacity: 0.5,
            snap_enabled: true,
            snap_tolerance: 5.0,
            measurement_unit: MeasurementUnit::Millimeters,
            measurement_scale: 1.0,
        }
    }
}

/// Grid types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridType {
    Dot,
    Square,
    Graph,
    Lined,
    Isometric,
    Triangular,
    Perspective1Point,
    Perspective2Point,
    Perspective3Point,
}

/// Measurement units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementUnit {
    Pixels,
    Points,
    Millimeters,
    Centimeters,
    Meters,
    Kilometers,
    Inches,
    Feet,
    Yards,
    Miles,
}

/// Artboard/page in the document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artboard {
    pub id: ArtboardId,
    pub name: String,
    pub rect: Rect,
    pub background_color: Option<Color>,
    pub export_enabled: bool,
}

impl Artboard {
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            id: ArtboardId::new(),
            name: name.into(),
            rect,
            background_color: None,
            export_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_id_generation() {
        let id1 = DocumentId::new();
        let id2 = DocumentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
        assert_eq!(color.to_hex(), "#FF0000");
    }

    #[test]
    fn test_color_hex_with_alpha() {
        let color = Color::from_hex("#FF000080").unwrap();
        assert!((color.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_point_operations() {
        let p1 = Point2::new(1.0, 2.0);
        let p2 = Point2::new(3.0, 4.0);
        assert_eq!(p1 + p2, Point2::new(4.0, 6.0));
        assert_eq!(p2 - p1, Point2::new(2.0, 2.0));
        assert_eq!(p1 * 2.0, Point2::new(2.0, 4.0));
        assert!((p1.distance_to(p2) - 2.828).abs() < 0.01);
    }

    #[test]
    fn test_transform() {
        let t = Transform2D::translate(10.0, 20.0);
        let p = t.transform_point(Point2::new(5.0, 5.0));
        assert_eq!(p, Point2::new(15.0, 25.0));
    }

    #[test]
    fn test_transform_composition() {
        let t1 = Transform2D::translate(10.0, 0.0);
        let t2 = Transform2D::scale(2.0, 2.0);
        let combined = t2 * t1; // scale then translate
        let p = combined.transform_point(Point2::new(5.0, 5.0));
        // translate first: (15, 5), then scale: (30, 10)
        assert_eq!(p, Point2::new(30.0, 10.0));
    }

    #[test]
    fn test_camera() {
        let camera = Camera::new();
        let world = Point2::new(100.0, 100.0);
        let screen = camera.world_to_screen(world, (800.0, 600.0), 1.0);
        let back = camera.screen_to_world(screen, (800.0, 600.0), 1.0);
        assert!((back.x - world.x).abs() < 0.001);
        assert!((back.y - world.y).abs() < 0.001);
    }

    #[test]
    fn test_rect() {
        let r = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        assert!(r.contains(Point2::new(50.0, 50.0)));
        assert!(!r.contains(Point2::new(150.0, 50.0)));
        assert_eq!(r.width(), 100.0);
        assert_eq!(r.height(), 100.0);
        assert_eq!(r.center(), Point2::new(50.0, 50.0));
    }
}
