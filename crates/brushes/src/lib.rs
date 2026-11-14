//! Brush engine for Creative Engine
//!
//! Provides brush preset system, stroke generation pipeline, and built-in brush families.
//! Pipeline: capture → normalize → smooth → resample → evaluate curves → generate outline

use creative_domain::{BrushId, Color, Point2, Vector2};
use creative_geometry::{Geometry, PathGeometry, PathSegment, StrokeStyle, StrokeCap, StrokeJoin, BoundingBox};
use creative_input::PointerEvent;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Brush errors
#[derive(Debug, Error)]
pub enum BrushError {
    #[error("Invalid brush configuration: {0}")]
    InvalidConfig(String),
    #[error("Brush not found: {0}")]
    NotFound(BrushId),
    #[error("Stroke generation failed: {0}")]
    StrokeGeneration(String),
    #[error("Tessellation failed: {0}")]
    Tessellation(String),
}

/// Brush family/category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BrushFamily {
    TechnicalPen,
    FixedWidth,
    PressurePen,
    Pencil,
    Marker,
    Airbrush,
    Eraser,
    Custom,
}

/// Pressure response curve
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PressureCurve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Custom([f32; 4]), // Cubic bezier control points
}

impl PressureCurve {
    pub fn evaluate(&self, pressure: f32) -> f32 {
        match self {
            PressureCurve::Linear => pressure,
            PressureCurve::EaseIn => pressure * pressure,
            PressureCurve::EaseOut => 1.0 - (1.0 - pressure) * (1.0 - pressure),
            PressureCurve::EaseInOut => {
                if pressure < 0.5 {
                    2.0 * pressure * pressure
                } else {
                    1.0 - 2.0 * (1.0 - pressure) * (1.0 - pressure)
                }
            }
            PressureCurve::Custom([_x1, y1, _x2, y2]) => {
                // Cubic bezier evaluation
                let t = pressure;
                let u = 1.0 - t;
                3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t
            }
        }
    }
}

/// Brush tip shape
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TipShape {
    Round,
    Square,
    Ellipse { aspect: f32, angle: f32 },
    Texture(Uuid), // Reference to texture asset
}

/// Brush blend mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrushBlendMode {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Erase, // Eraser mode
}

/// Brush dynamics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushDynamics {
    /// Size dynamics
    pub size_pressure: bool,
    pub size_velocity: bool,
    pub size_random: f32, // 0.0 - 1.0
    pub size_tilt: bool,
    
    /// Opacity dynamics
    pub opacity_pressure: bool,
    pub opacity_velocity: bool,
    pub opacity_random: f32,
    pub opacity_tilt: bool,
    
    /// Flow dynamics
    pub flow_pressure: bool,
    pub flow_velocity: bool,
    pub flow_random: f32,
    
    /// Spacing dynamics
    pub spacing_pressure: bool,
    pub spacing_velocity: bool,
    pub spacing_random: f32,
    
    /// Scatter dynamics
    pub scatter_radius: f32,
    pub scatter_pressure: bool,
    pub scatter_random: f32,
    
    /// Rotation dynamics
    pub rotation_velocity: bool,
    pub rotation_random: f32,
    pub rotation_tilt: bool,
}

impl Default for BrushDynamics {
    fn default() -> Self {
        Self {
            size_pressure: true,
            size_velocity: false,
            size_random: 0.0,
            size_tilt: false,
            opacity_pressure: true,
            opacity_velocity: false,
            opacity_random: 0.0,
            opacity_tilt: false,
            flow_pressure: true,
            flow_velocity: false,
            flow_random: 0.0,
            spacing_pressure: false,
            spacing_velocity: false,
            spacing_random: 0.0,
            scatter_radius: 0.0,
            scatter_pressure: false,
            scatter_random: 0.0,
            rotation_velocity: false,
            rotation_random: 0.0,
            rotation_tilt: false,
        }
    }
}

/// Brush texture for textured brushes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushTexture {
    pub id: Uuid,
    pub name: String,
    pub data: Vec<u8>, // Raw texture data (grayscale)
    pub width: u32,
    pub height: u32,
    pub scale: f32, // World units per texture pixel
}

/// Brush preset definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushPreset {
    pub id: BrushId,
    pub name: String,
    pub family: BrushFamily,
    pub version: u32,
    
    // Core properties
    pub base_size: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub color: Color,
    pub opacity: f32,
    pub flow: f32,
    pub spacing: f32, // 0.0 - 1.0, relative to size
    
    // Tip properties
    pub tip_shape: TipShape,
    pub tip_hardness: f32, // 0.0 - 1.0
    
    // Dynamics
    pub dynamics: BrushDynamics,
    pub pressure_curve: PressureCurve,
    
    // Blend mode
    pub blend_mode: BrushBlendMode,
    
    // Texture
    pub texture: Option<BrushTexture>,
    
    // Smoothing
    pub smoothing: f32, // 0.0 - 1.0
    pub stabilization: f32, // 0.0 - 1.0
    
    // Eraser specific
    pub erases_all_layers: bool,
    
    // Metadata
    pub tags: Vec<String>,
    pub description: String,
    pub author: String,
}

impl Default for BrushPreset {
    fn default() -> Self {
        Self {
            id: BrushId::new(),
            name: "New Brush".to_string(),
            family: BrushFamily::Custom,
            version: 1,
            base_size: 10.0,
            min_size: 1.0,
            max_size: 1000.0,
            color: Color::BLACK,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.1,
            tip_shape: TipShape::Round,
            tip_hardness: 1.0,
            dynamics: BrushDynamics::default(),
            pressure_curve: PressureCurve::Linear,
            blend_mode: BrushBlendMode::Normal,
            texture: None,
            smoothing: 0.5,
            stabilization: 0.0,
            erases_all_layers: false,
            tags: Vec::new(),
            description: String::new(),
            author: "Creative Engine".to_string(),
        }
    }
}

/// Raw stroke point from input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStrokePoint {
    pub position: Point2,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub altitude_angle: f32,
    pub azimuth_angle: f32,
    pub timestamp: u64,
    pub velocity: f32,
    pub rotation: f32,
}

/// Processed stroke point with computed properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Point2,
    pub pressure: f32,
    pub size: f32,
    pub opacity: f32,
    pub flow: f32,
    pub rotation: f32,
    pub scatter_offset: Vector2,
    pub timestamp: u64,
}

/// Complete stroke data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub id: Uuid,
    pub brush_id: BrushId,
    pub points: Vec<StrokePoint>,
    pub start_time: u64,
    pub end_time: u64,
    pub bounds: BoundingBox,
}

/// Stroke generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokeConfig {
    pub min_points: usize,
    pub max_points: usize,
    pub smoothing_factor: f32,
    pub simplification_tolerance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
}

impl Default for StrokeConfig {
    fn default() -> Self {
        Self {
            min_points: 2,
            max_points: 10000,
            smoothing_factor: 0.5,
            simplification_tolerance: 0.5,
            min_distance: 0.1,
            max_distance: 50.0,
        }
    }
}

/// Brush engine for stroke generation
#[derive(Debug)]
pub struct BrushEngine {
    presets: HashMap<BrushId, BrushPreset>,
    config: StrokeConfig,
    stroke_buffer: Vec<RawStrokePoint>,
    current_brush: Option<BrushId>,
}

impl BrushEngine {
    pub fn new(config: StrokeConfig) -> Self {
        let mut engine = Self {
            presets: HashMap::new(),
            config,
            stroke_buffer: Vec::new(),
            current_brush: None,
        };
        
        // Register built-in brush presets
        engine.register_builtin_brushes();
        engine
    }
    
    /// Register built-in brush presets
    fn register_builtin_brushes(&mut self) {
        // Technical Pen - precise, no pressure variation
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Technical Pen".to_string(),
            family: BrushFamily::TechnicalPen,
            version: 1,
            base_size: 2.0,
            min_size: 0.5,
            max_size: 10.0,
            color: Color::BLACK,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.05,
            tip_shape: TipShape::Round,
            tip_hardness: 1.0,
            dynamics: BrushDynamics {
                size_pressure: false,
                size_velocity: false,
                opacity_pressure: false,
                flow_pressure: false,
                ..Default::default()
            },
            pressure_curve: PressureCurve::Linear,
            blend_mode: BrushBlendMode::Normal,
            texture: None,
            smoothing: 0.8,
            stabilization: 0.5,
            erases_all_layers: false,
            tags: vec!["technical".to_string(), "ink".to_string(), "precise".to_string()],
            description: "Precise technical pen with constant width".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Fixed Width - constant width, no dynamics
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Fixed Width".to_string(),
            family: BrushFamily::FixedWidth,
            version: 1,
            base_size: 5.0,
            min_size: 1.0,
            max_size: 50.0,
            color: Color::BLACK,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.1,
            tip_shape: TipShape::Round,
            tip_hardness: 1.0,
            dynamics: BrushDynamics {
                size_pressure: false,
                size_velocity: false,
                opacity_pressure: false,
                flow_pressure: false,
                ..Default::default()
            },
            pressure_curve: PressureCurve::Linear,
            blend_mode: BrushBlendMode::Normal,
            texture: None,
            smoothing: 0.5,
            stabilization: 0.3,
            erases_all_layers: false,
            tags: vec!["fixed".to_string(), "constant".to_string()],
            description: "Constant width brush, no pressure sensitivity".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Pressure Pen - standard pressure-sensitive pen
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Pressure Pen".to_string(),
            family: BrushFamily::PressurePen,
            version: 1,
            base_size: 8.0,
            min_size: 1.0,
            max_size: 30.0,
            color: Color::BLACK,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.1,
            tip_shape: TipShape::Round,
            tip_hardness: 0.9,
            dynamics: BrushDynamics {
                size_pressure: true,
                size_velocity: true,
                opacity_pressure: true,
                flow_pressure: true,
                ..Default::default()
            },
            pressure_curve: PressureCurve::EaseInOut,
            blend_mode: BrushBlendMode::Normal,
            texture: None,
            smoothing: 0.6,
            stabilization: 0.4,
            erases_all_layers: false,
            tags: vec!["pen".to_string(), "pressure".to_string(), "inking".to_string()],
            description: "Standard pressure-sensitive pen for inking".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Pencil - textured, tilt-sensitive
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Pencil".to_string(),
            family: BrushFamily::Pencil,
            version: 1,
            base_size: 6.0,
            min_size: 1.0,
            max_size: 20.0,
            color: Color::new(0.2, 0.2, 0.2, 1.0),
            opacity: 0.8,
            flow: 0.7,
            spacing: 0.15,
            tip_shape: TipShape::Ellipse { aspect: 0.7, angle: 45.0 },
            tip_hardness: 0.5,
            dynamics: BrushDynamics {
                size_pressure: true,
                size_tilt: true,
                opacity_pressure: true,
                opacity_tilt: true,
                flow_pressure: true,
                scatter_radius: 1.0,
                scatter_pressure: true,
                scatter_random: 0.3,
                rotation_tilt: true,
                ..Default::default()
            },
            pressure_curve: PressureCurve::EaseOut,
            blend_mode: BrushBlendMode::Multiply,
            texture: None,
            smoothing: 0.3,
            stabilization: 0.2,
            erases_all_layers: false,
            tags: vec!["pencil".to_string(), "sketch".to_string(), "tilt".to_string()],
            description: "Graphite pencil with tilt and texture response".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Marker - chisel tip, alcohol marker feel
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Marker".to_string(),
            family: BrushFamily::Marker,
            version: 1,
            base_size: 12.0,
            min_size: 3.0,
            max_size: 50.0,
            color: Color::BLUE,
            opacity: 0.9,
            flow: 0.8,
            spacing: 0.08,
            tip_shape: TipShape::Ellipse { aspect: 0.3, angle: 0.0 },
            tip_hardness: 0.7,
            dynamics: BrushDynamics {
                size_pressure: true,
                size_velocity: false,
                opacity_pressure: false,
                flow_pressure: true,
                scatter_radius: 0.5,
                scatter_random: 0.1,
                ..Default::default()
            },
            pressure_curve: PressureCurve::Linear,
            blend_mode: BrushBlendMode::Multiply,
            texture: None,
            smoothing: 0.4,
            stabilization: 0.3,
            erases_all_layers: false,
            tags: vec!["marker".to_string(), "chisel".to_string(), "alcohol".to_string()],
            description: "Chisel-tip alcohol marker".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Airbrush - soft, builds up with flow
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Airbrush".to_string(),
            family: BrushFamily::Airbrush,
            version: 1,
            base_size: 40.0,
            min_size: 10.0,
            max_size: 200.0,
            color: Color::BLACK,
            opacity: 0.3,
            flow: 0.2,
            spacing: 0.02,
            tip_shape: TipShape::Round,
            tip_hardness: 0.1,
            dynamics: BrushDynamics {
                size_pressure: true,
                size_velocity: false,
                opacity_pressure: true,
                flow_pressure: true,
                scatter_radius: 15.0,
                scatter_pressure: true,
                scatter_random: 0.5,
                ..Default::default()
            },
            pressure_curve: PressureCurve::EaseIn,
            blend_mode: BrushBlendMode::Normal,
            texture: None,
            smoothing: 0.7,
            stabilization: 0.6,
            erases_all_layers: false,
            tags: vec!["airbrush".to_string(), "soft".to_string(), "spray".to_string()],
            description: "Soft airbrush with buildup flow".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Eraser - removes strokes
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Eraser".to_string(),
            family: BrushFamily::Eraser,
            version: 1,
            base_size: 30.0,
            min_size: 5.0,
            max_size: 500.0,
            color: Color::WHITE,
            opacity: 1.0,
            flow: 1.0,
            spacing: 0.05,
            tip_shape: TipShape::Round,
            tip_hardness: 1.0,
            dynamics: BrushDynamics {
                size_pressure: true,
                ..Default::default()
            },
            pressure_curve: PressureCurve::Linear,
            blend_mode: BrushBlendMode::Erase,
            texture: None,
            smoothing: 0.5,
            stabilization: 0.3,
            erases_all_layers: true,
            tags: vec!["eraser".to_string(), "erase".to_string()],
            description: "Hard eraser, removes all layers".to_string(),
            author: "Creative Engine".to_string(),
        });
        
        // Soft Eraser
        self.register_preset(BrushPreset {
            id: BrushId::new(),
            name: "Soft Eraser".to_string(),
            family: BrushFamily::Eraser,
            version: 1,
            base_size: 40.0,
            min_size: 10.0,
            max_size: 500.0,
            color: Color::WHITE,
            opacity: 0.5,
            flow: 0.3,
            spacing: 0.02,
            tip_shape: TipShape::Round,
            tip_hardness: 0.2,
            dynamics: BrushDynamics {
                size_pressure: true,
                scatter_radius: 5.0,
                scatter_random: 0.3,
                ..Default::default()
            },
            pressure_curve: PressureCurve::EaseIn,
            blend_mode: BrushBlendMode::Erase,
            texture: None,
            smoothing: 0.6,
            stabilization: 0.4,
            erases_all_layers: false,
            tags: vec!["eraser".to_string(), "soft".to_string()],
            description: "Soft eraser with buildup".to_string(),
            author: "Creative Engine".to_string(),
        });
    }
    
    /// Register a brush preset
    pub fn register_preset(&mut self, preset: BrushPreset) {
        self.presets.insert(preset.id, preset);
    }
    
    /// Get a brush preset by ID
    pub fn get_preset(&self, id: BrushId) -> Option<&BrushPreset> {
        self.presets.get(&id)
    }
    
    /// Get mutable brush preset
    pub fn get_preset_mut(&mut self, id: BrushId) -> Option<&mut BrushPreset> {
        self.presets.get_mut(&id)
    }
    
    /// List all brush presets
    pub fn list_presets(&self) -> Vec<&BrushPreset> {
        self.presets.values().collect()
    }
    
    /// List brushes by family
    pub fn list_by_family(&self, family: BrushFamily) -> Vec<&BrushPreset> {
        self.presets.values().filter(|p| p.family == family).collect()
    }
    
    /// Set current brush
    pub fn set_brush(&mut self, brush_id: BrushId) -> Result<(), BrushError> {
        if self.presets.contains_key(&brush_id) {
            self.current_brush = Some(brush_id);
            Ok(())
        } else {
            Err(BrushError::NotFound(brush_id))
        }
    }
    
    /// Get current brush
    pub fn current_brush(&self) -> Option<BrushId> {
        self.current_brush
    }
    
    /// Start a new stroke
    pub fn begin_stroke(&mut self, brush_id: BrushId) -> Result<(), BrushError> {
        self.set_brush(brush_id)?;
        self.stroke_buffer.clear();
        Ok(())
    }
    
    /// Add a point to the current stroke
    pub fn add_stroke_point(&mut self, event: &PointerEvent) {
        let velocity = self.compute_velocity(event);
        let rotation = self.compute_rotation(event);
        
        self.stroke_buffer.push(RawStrokePoint {
            position: event.position,
            pressure: event.pressure,
            tilt_x: event.tilt_x,
            tilt_y: event.tilt_y,
            altitude_angle: event.altitude_angle,
            azimuth_angle: event.azimuth_angle,
            timestamp: event.timestamp,
            velocity,
            rotation,
        });
    }
    
    /// End stroke and generate geometry
    pub fn end_stroke(&mut self) -> Result<Stroke, BrushError> {
        if self.stroke_buffer.is_empty() {
            return Err(BrushError::StrokeGeneration("Empty stroke buffer".to_string()));
        }
        
        let brush_id = self.current_brush.ok_or_else(|| 
            BrushError::StrokeGeneration("No brush selected".to_string()))?;
        
        let preset = self.presets.get(&brush_id).ok_or_else(|| 
            BrushError::NotFound(brush_id))?;
        
        // Process stroke through pipeline
        let processed_points = self.process_stroke(preset)?;
        
        // Calculate bounds
        let bounds = self.compute_bounds(&processed_points);
        
        let stroke = Stroke {
            id: Uuid::now_v7(),
            brush_id,
            points: processed_points,
            start_time: self.stroke_buffer.first().map(|p| p.timestamp).unwrap_or(0),
            end_time: self.stroke_buffer.last().map(|p| p.timestamp).unwrap_or(0),
            bounds,
        };
        
        self.stroke_buffer.clear();
        Ok(stroke)
    }
    
    /// Process raw stroke points through the full pipeline
    fn process_stroke(&self, preset: &BrushPreset) -> Result<Vec<StrokePoint>, BrushError> {
        if self.stroke_buffer.len() < self.config.min_points {
            return Err(BrushError::StrokeGeneration("Too few points".to_string()));
        }
        
        // 1. Normalize (clamp values, handle device differences)
        let mut points = self.normalize_points(&self.stroke_buffer);
        
        // 2. Smooth (reduce jitter)
        points = self.smooth_points(points, preset.smoothing);
        
        // 3. Resample (even spacing)
        points = self.resample_points(points, preset.spacing, preset.base_size);
        
        // 4. Evaluate curves (pressure, velocity, tilt -> size, opacity, flow)
        let processed = self.evaluate_dynamics(points, preset)?;
        
        // 5. Apply stabilization (lazy mouse / rope)
        let stabilized = self.apply_stabilization(processed, preset.stabilization);
        
        // 6. Limit points
        if stabilized.len() > self.config.max_points {
            // Decimate while preserving shape
            return Ok(self.decimate_points(stabilized, self.config.max_points));
        }
        
        Ok(stabilized)
    }
    
    /// Normalize raw points
    fn normalize_points(&self, raw: &[RawStrokePoint]) -> Vec<RawStrokePoint> {
        raw.iter().map(|p| RawStrokePoint {
            pressure: p.pressure.clamp(0.0, 1.0),
            tilt_x: p.tilt_x.clamp(-1.0, 1.0),
            tilt_y: p.tilt_y.clamp(-1.0, 1.0),
            altitude_angle: p.altitude_angle.clamp(0.0, std::f32::consts::FRAC_PI_2),
            ..*p
        }).collect()
    }
    
    /// Smooth points using exponential moving average
    fn smooth_points(&self, points: Vec<RawStrokePoint>, factor: f32) -> Vec<RawStrokePoint> {
        if points.len() < 3 || factor <= 0.0 {
            return points;
        }
        
        let alpha = factor.clamp(0.0, 1.0);
        let mut smoothed = Vec::with_capacity(points.len());
        smoothed.push(points[0].clone());
        
        for i in 1..points.len() - 1 {
            let prev = &smoothed[i - 1];
            let curr = &points[i];
            let _next = &points[i + 1];
            
            let smooth_pos = Point2::new(
                prev.position.x * (1.0 - alpha as f64) + curr.position.x * alpha as f64,
                prev.position.y * (1.0 - alpha as f64) + curr.position.y * alpha as f64,
            );
            
            smoothed.push(RawStrokePoint {
                position: smooth_pos,
                pressure: prev.pressure * (1.0 - alpha) + curr.pressure * alpha,
                tilt_x: prev.tilt_x * (1.0 - alpha) + curr.tilt_x * alpha,
                tilt_y: prev.tilt_y * (1.0 - alpha) + curr.tilt_y * alpha,
                altitude_angle: curr.altitude_angle,
                azimuth_angle: curr.azimuth_angle,
                timestamp: curr.timestamp,
                velocity: curr.velocity,
                rotation: curr.rotation,
            });
        }
        
        smoothed.push(points.last().unwrap().clone());
        smoothed
    }
    
    /// Resample points to even spacing
    fn resample_points(&self, points: Vec<RawStrokePoint>, spacing: f32, base_size: f32) -> Vec<RawStrokePoint> {
        if points.len() < 2 {
            return points;
        }
        
        let target_distance = (base_size * spacing).max(self.config.min_distance).min(self.config.max_distance);
        let mut resampled = Vec::new();
        resampled.push(points[0].clone());
        
        let mut accum_distance = 0.0;
        
        for i in 1..points.len() {
            let prev = &points[i - 1];
            let curr = &points[i];
            let segment_dist = prev.position.distance_to(curr.position) as f32;
            
            accum_distance += segment_dist;
            
            while accum_distance >= target_distance {
                let t = (accum_distance - target_distance) / segment_dist.max(0.001);
                let interp_pos = Point2::new(
                    prev.position.x * (1.0 - t as f64) + curr.position.x * t as f64,
                    prev.position.y * (1.0 - t as f64) + curr.position.y * t as f64,
                );
                let interp_pressure = prev.pressure * (1.0 - t) + curr.pressure * t;
                let interp_tilt_x = prev.tilt_x * (1.0 - t) + curr.tilt_x * t;
                let interp_tilt_y = prev.tilt_y * (1.0 - t) + curr.tilt_y * t;
                let interp_timestamp = (prev.timestamp as f32 * (1.0 - t) + curr.timestamp as f32 * t) as u64;
                
                resampled.push(RawStrokePoint {
                    position: interp_pos,
                    pressure: interp_pressure,
                    tilt_x: interp_tilt_x,
                    tilt_y: interp_tilt_y,
                    altitude_angle: curr.altitude_angle,
                    azimuth_angle: curr.azimuth_angle,
                    timestamp: interp_timestamp,
                    velocity: curr.velocity,
                    rotation: curr.rotation,
                });
                
                accum_distance -= target_distance;
            }
        }
        
        // Always include last point
        if resampled.last().map(|p| p.timestamp) != points.last().map(|p| p.timestamp) {
            resampled.push(points.last().unwrap().clone());
        }
        
        resampled
    }
    
    /// Evaluate brush dynamics to compute stroke point properties
    fn evaluate_dynamics(&self, points: Vec<RawStrokePoint>, preset: &BrushPreset) -> Result<Vec<StrokePoint>, BrushError> {
        let dynamics = &preset.dynamics;
        let pressure_curve = &preset.pressure_curve;
        
        let processed: Vec<StrokePoint> = points.into_iter().enumerate().map(|(i, p)| {
            let pressure_factor = pressure_curve.evaluate(p.pressure);
            
            // Size calculation
            let size = if dynamics.size_pressure {
                lerp(preset.min_size as f64, preset.max_size as f64, pressure_factor as f64) as f32
            } else {
                preset.base_size
            };
            
            // Apply velocity to size
            let size = if dynamics.size_velocity {
                size * (1.0 - p.velocity.min(1.0) * 0.5)
            } else {
                size
            };
            
            // Apply random size variation
            let size = if dynamics.size_random > 0.0 {
                let rand = ((i as f32 * 12345.6789).sin() * 0.5 + 0.5) * 2.0 - 1.0;
                size * (1.0 + rand * dynamics.size_random)
            } else {
                size
            };
            
            // Opacity calculation
            let opacity = if dynamics.opacity_pressure {
                preset.opacity * pressure_factor
            } else {
                preset.opacity
            };
            
            let opacity = if dynamics.opacity_velocity {
                opacity * (1.0 - p.velocity.min(1.0) * 0.3)
            } else {
                opacity
            };
            
            // Flow calculation
            let flow = if dynamics.flow_pressure {
                preset.flow * pressure_factor
            } else {
                preset.flow
            };
            
            // Scatter offset
            let scatter_offset = if dynamics.scatter_radius > 0.0 {
                let angle = (i as f32 * 2.39996323).sin() * std::f32::consts::TAU;
                let radius = dynamics.scatter_radius * (1.0 + pressure_factor * 0.5);
                Vector2::new(angle.cos() as f64 * radius as f64, angle.sin() as f64 * radius as f64)
            } else {
                Vector2::ZERO
            };
            
            // Rotation
            let rotation = if dynamics.rotation_velocity {
                p.rotation + p.velocity * 0.1
            } else if dynamics.rotation_random > 0.0 {
                p.rotation + ((i as f32 * 9876.54321).sin() * dynamics.rotation_random)
            } else if dynamics.rotation_tilt {
                p.rotation + (p.tilt_x.atan2(p.tilt_y))
            } else {
                p.rotation
            };
            
            StrokePoint {
                position: p.position,
                pressure: p.pressure,
                size: size.max(0.1),
                opacity: opacity.clamp(0.0, 1.0),
                flow: flow.clamp(0.0, 1.0),
                rotation,
                scatter_offset,
                timestamp: p.timestamp,
            }
        }).collect();
        
        Ok(processed)
    }
    
    /// Apply stabilization (lazy mouse / rope algorithm)
    fn apply_stabilization(&self, points: Vec<StrokePoint>, stabilization: f32) -> Vec<StrokePoint> {
        if points.len() < 3 || stabilization <= 0.0 {
            return points;
        }
        
        let strength = stabilization.clamp(0.0, 1.0);
        let rope_length = 50.0 * strength; // Max rope length in world units
        
        let mut stabilized = Vec::with_capacity(points.len());
        stabilized.push(points[0].clone());
        
        let mut rope_end = points[0].position;
        
        for i in 1..points.len() {
            let curr = &points[i];
            let to_curr = curr.position - rope_end;
            let dist = to_curr.x.hypot(to_curr.y) as f32;
            
            if dist > rope_length {
                let dir = Vector2::new(to_curr.x / dist as f64, to_curr.y / dist as f64);
                rope_end = Point2::new(
                    curr.position.x - dir.x * rope_length as f64,
                    curr.position.y - dir.y * rope_length as f64,
                );
            } else {
                rope_end = curr.position;
            }
            
            let mut new_point = curr.clone();
            new_point.position = rope_end;
            stabilized.push(new_point);
        }
        
        stabilized
    }
    
    /// Decimate points while preserving shape (Douglas-Peucker simplified)
    fn decimate_points(&self, points: Vec<StrokePoint>, max_points: usize) -> Vec<StrokePoint> {
        if points.len() <= max_points {
            return points;
        }
        
        let step = points.len() as f32 / max_points as f32;
        let mut decimated = Vec::with_capacity(max_points);
        
        for i in 0..max_points {
            let idx = ((i as f32 * step) as usize).min(points.len() - 1);
            decimated.push(points[idx].clone());
        }
        
        decimated
    }
    
    /// Compute stroke bounds
    fn compute_bounds(&self, points: &[StrokePoint]) -> BoundingBox {
        let mut min = Point2::new(f64::INFINITY, f64::INFINITY);
        let mut max = Point2::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
        
        for p in points {
            let size = p.size as f64;
            let pos = p.position;
            let scatter = p.scatter_offset;
            
            let x = pos.x + scatter.x;
            let y = pos.y + scatter.y;
            
            min.x = min.x.min(x - size);
            min.y = min.y.min(y - size);
            max.x = max.x.max(x + size);
            max.y = max.y.max(y + size);
        }
        
        BoundingBox::new(min, max)
    }
    
    /// Compute velocity from stroke buffer
    fn compute_velocity(&self, event: &PointerEvent) -> f32 {
        if self.stroke_buffer.is_empty() {
            return 0.0;
        }
        
        let last = &self.stroke_buffer.last().unwrap();
        let dt = (event.timestamp.saturating_sub(last.timestamp)) as f32 / 1000.0;
        if dt <= 0.0 {
            return 0.0;
        }
        
        let dist = event.position.distance_to(last.position) as f32;
        dist / dt
    }
    
    /// Compute rotation from stroke buffer
    fn compute_rotation(&self, event: &PointerEvent) -> f32 {
        if self.stroke_buffer.len() < 2 {
            return 0.0;
        }
        
        let last = &self.stroke_buffer.last().unwrap();
        let _prev = &self.stroke_buffer[self.stroke_buffer.len() - 2];
        
        let dx = event.position.x - last.position.x;
        let dy = event.position.y - last.position.y;
        (dy.atan2(dx)) as f32
    }
    
    /// Generate geometry from stroke (outline or fill)
    pub fn stroke_to_geometry(&self, stroke: &Stroke) -> Result<Vec<Geometry>, BrushError> {
        let brush_id = stroke.brush_id;
        let preset = self.presets.get(&brush_id).ok_or_else(|| 
            BrushError::NotFound(brush_id))?;
        
        if preset.blend_mode == BrushBlendMode::Erase {
            // Eraser creates a special geometry type for subtraction
            return self.generate_eraser_geometry(stroke, preset);
        }
        
        // Generate stroke outline geometry
        let outline = self.generate_stroke_outline(stroke, preset)?;
        Ok(vec![Geometry::Path(outline)])
    }
    
    /// Generate stroke outline as a path geometry
    fn generate_stroke_outline(&self, stroke: &Stroke, preset: &BrushPreset) -> Result<PathGeometry, BrushError> {
        let mut path = PathGeometry::new();
        path.stroke = Some(StrokeStyle {
            width: 0.0, // We'll build the outline manually
            color: preset.color,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            dash_pattern: Vec::new(),
            dash_offset: 0.0,
        });
        path.fill = Some(preset.color);
        
        if stroke.points.len() < 2 {
            return Ok(path);
        }
        
        // Build left and right edges of stroke
        let mut left_points = Vec::new();
        let mut right_points = Vec::new();
        
        for (i, pt) in stroke.points.iter().enumerate() {
            // Compute normal direction
            let (prev_pt, next_pt) = if i == 0 {
                (pt, &stroke.points[1])
            } else if i == stroke.points.len() - 1 {
                (&stroke.points[i - 1], pt)
            } else {
                (&stroke.points[i - 1], &stroke.points[i + 1])
            };
            
            let dir = Vector2::new(
                next_pt.position.x - prev_pt.position.x,
                next_pt.position.y - prev_pt.position.y,
            );
            let len = dir.length().max(0.001);
            let normal = Vector2::new(-dir.y / len, dir.x / len);
            
            let half_size = (pt.size * 0.5) as f64;
            let scatter = pt.scatter_offset;
            
            let center = Point2::new(
                pt.position.x + scatter.x,
                pt.position.y + scatter.y,
            );
            
            left_points.push(Point2::new(
                center.x - normal.x * half_size,
                center.y - normal.y * half_size,
            ));
            right_points.push(Point2::new(
                center.x + normal.x * half_size,
                center.y + normal.y * half_size,
            ));
        }
        
        // Build path: left edge forward, right edge backward
        if !left_points.is_empty() {
            path.segments.push(PathSegment::MoveTo(left_points[0]));
            for p in &left_points[1..] {
                path.segments.push(PathSegment::LineTo(*p));
            }
            
            for p in right_points.iter().rev() {
                path.segments.push(PathSegment::LineTo(*p));
            }
            
            path.segments.push(PathSegment::ClosePath);
        }
        
        Ok(path)
    }
    
    /// Generate eraser geometry
    fn generate_eraser_geometry(&self, stroke: &Stroke, preset: &BrushPreset) -> Result<Vec<Geometry>, BrushError> {
        // Eraser creates a path that will be used for boolean subtraction
        let mut path = self.generate_stroke_outline(stroke, preset)?;
        path.fill = Some(Color::TRANSPARENT);
        path.stroke = None;
        Ok(vec![Geometry::Path(path)])
    }
}

/// Linear interpolation
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Stroke tessellator for rendering (simplified - returns stroke geometry)
pub struct StrokeTessellator;

impl StrokeTessellator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn tessellate_stroke(&self, stroke: &Stroke, _preset: &BrushPreset) -> Result<Vec<f32>, BrushError> {
        // Return flattened stroke points for rendering
        if stroke.points.is_empty() {
            return Ok(Vec::new());
        }
        
        let mut vertices = Vec::new();
        for pt in &stroke.points {
            vertices.push(pt.position.x as f32);
            vertices.push(pt.position.y as f32);
        }
        
        Ok(vertices)
    }
}

impl Default for StrokeTessellator {
    fn default() -> Self {
        Self::new()
    }
}

/// Fill tessellator for filled shapes (simplified - returns path geometry)
pub struct FillTessellator;

impl FillTessellator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn tessellate_path(&self, path_geom: &PathGeometry) -> Result<Vec<f32>, BrushError> {
        let mut vertices = Vec::new();
        
        for seg in &path_geom.segments {
            match seg {
                PathSegment::MoveTo(p) | PathSegment::LineTo(p) => {
                    vertices.push(p.x as f32);
                    vertices.push(p.y as f32);
                }
                PathSegment::CubicTo(_, _, p) => {
                    vertices.push(p.x as f32);
                    vertices.push(p.y as f32);
                }
                PathSegment::ClosePath => {}
            }
        }
        
        Ok(vertices)
    }
}

impl Default for FillTessellator {
    fn default() -> Self {
        Self::new()
    }
}

/// Brush preset library for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrushLibrary {
    pub name: String,
    pub version: u32,
    pub brushes: Vec<BrushPreset>,
}

impl BrushLibrary {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: 1,
            brushes: Vec::new(),
        }
    }
    
    pub fn add_brush(&mut self, brush: BrushPreset) {
        self.brushes.push(brush);
    }
    
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    pub fn to_binary(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
    
    pub fn from_binary(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::Point2;
    use creative_input::{PointerEvent, PointerEventType, InputDeviceType, PointerButton, Modifiers};
    
    fn create_test_pointer_event(x: f64, y: f64, pressure: f32, timestamp: u64) -> PointerEvent {
        PointerEvent {
            event_type: PointerEventType::Move,
            device_type: InputDeviceType::Stylus,
            device_id: 1,
            pointer_id: 1,
            position: Point2::new(x, y),
            pressure,
            tangential_pressure: 0.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            twist: 0.0,
            altitude_angle: 1.57,
            azimuth_angle: 0.0,
            contact_size: Point2::new(1.0, 1.0),
            button: Some(PointerButton::Left),
            buttons: 1,
            modifiers: Modifiers::default(),
            timestamp,
            is_primary: true,
            is_coalesced: false,
            predicted_points: Vec::new(),
        }
    }
    
    #[test]
    fn test_brush_engine_creation() {
        let engine = BrushEngine::new(StrokeConfig::default());
        assert!(!engine.list_presets().is_empty());
    }
    
    #[test]
    fn test_builtin_brushes_registered() {
        let engine = BrushEngine::new(StrokeConfig::default());
        let presets = engine.list_presets();
        
        let families: std::collections::HashSet<_> = presets.iter().map(|p| p.family).collect();
        assert!(families.contains(&BrushFamily::TechnicalPen));
        assert!(families.contains(&BrushFamily::FixedWidth));
        assert!(families.contains(&BrushFamily::PressurePen));
        assert!(families.contains(&BrushFamily::Pencil));
        assert!(families.contains(&BrushFamily::Marker));
        assert!(families.contains(&BrushFamily::Airbrush));
        assert!(families.contains(&BrushFamily::Eraser));
    }
    
    #[test]
    fn test_pressure_curve_evaluation() {
        assert_eq!(PressureCurve::Linear.evaluate(0.5), 0.5);
        assert!((PressureCurve::EaseIn.evaluate(0.5) - 0.25).abs() < 0.01);
        assert!((PressureCurve::EaseOut.evaluate(0.5) - 0.75).abs() < 0.01);
    }
    
    #[test]
    fn test_stroke_generation() {
        let mut engine = BrushEngine::new(StrokeConfig::default());
        let brushes = engine.list_by_family(BrushFamily::PressurePen);
        assert!(!brushes.is_empty());
        
        let brush_id = brushes[0].id;
        engine.begin_stroke(brush_id).unwrap();
        
        // Add stroke points
        for i in 0..10 {
            let event = create_test_pointer_event(
                i as f64 * 10.0,
                (i as f64 * 0.5).sin() * 10.0,
                0.5 + (i as f32 * 0.05),
                1000 + i as u64 * 16,
            );
            engine.add_stroke_point(&event);
        }
        
        let stroke = engine.end_stroke().unwrap();
        assert_eq!(stroke.brush_id, brush_id);
        assert!(!stroke.points.is_empty());
        assert!(stroke.bounds.width() > 0.0);
        assert!(stroke.bounds.height() > 0.0);
    }
    
    #[test]
    fn test_eraser_brush() {
        let engine = BrushEngine::new(StrokeConfig::default());
        let erasers = engine.list_by_family(BrushFamily::Eraser);
        assert!(!erasers.is_empty());
        
        let eraser = erasers.iter().find(|e| e.name == "Eraser").unwrap();
        assert_eq!(eraser.blend_mode, BrushBlendMode::Erase);
        assert!(eraser.erases_all_layers);
    }
    
    #[test]
    fn test_brush_library_serialization() {
        let mut library = BrushLibrary::new("Test Library");
        let engine = BrushEngine::new(StrokeConfig::default());
        
        for preset in engine.list_presets() {
            library.add_brush(preset.clone());
        }
        
        let json = library.to_json().unwrap();
        let loaded = BrushLibrary::from_json(&json).unwrap();
        
        assert_eq!(loaded.name, "Test Library");
        assert_eq!(loaded.brushes.len(), library.brushes.len());
    }
    
    #[test]
    fn test_stroke_to_geometry() {
        let mut engine = BrushEngine::new(StrokeConfig::default());
        let brushes = engine.list_by_family(BrushFamily::PressurePen);
        let brush_id = brushes[0].id;
        
        engine.begin_stroke(brush_id).unwrap();
        
        for i in 0..5 {
            let event = create_test_pointer_event(
                i as f64 * 20.0,
                0.0,
                0.8,
                1000 + i as u64 * 16,
            );
            engine.add_stroke_point(&event);
        }
        
        let stroke = engine.end_stroke().unwrap();
        let geometries = engine.stroke_to_geometry(&stroke).unwrap();
        
        assert_eq!(geometries.len(), 1);
        if let Geometry::Path(path) = &geometries[0] {
            assert!(!path.segments.is_empty());
        } else {
            panic!("Expected Path geometry");
        }
    }
}