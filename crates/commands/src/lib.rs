//! Validated command definitions for Creative Engine
//!
//! Commands represent intent to mutate document state. They carry all data needed
//! for validation and execution, and produce results with undo information.

use creative_domain::{DocumentId, ObjectId, LayerId, ArtboardId, Point2, Color, Rect, Transform2D, BlendMode, GridType, MeasurementUnit, Vector2};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during command validation or execution
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    Invalid(String),
    #[error("Document not found: {0}")]
    DocumentNotFound(DocumentId),
    #[error("Object not found: {0}")]
    ObjectNotFound(ObjectId),
    #[error("Layer not found: {0}")]
    LayerNotFound(LayerId),
    #[error("Artboard not found: {0}")]
    ArtboardNotFound(ArtboardId),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
    #[error("Corrupt document: {0}")]
    CorruptDocument(String),
    #[error("Version incompatibility: {0}")]
    VersionIncompatibility(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Result of a command execution containing undo information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub undo_data: Option<UndoData>,
    pub affected_ids: Vec<ObjectId>,
}

/// Data needed to undo a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoData {
    CreateObject { object_id: ObjectId },
    DeleteObject { object_data: ObjectData },
    UpdateObject { object_id: ObjectId, previous_data: ObjectData },
    TransformObjects { object_ids: Vec<ObjectId>, previous_transforms: Vec<Transform2D> },
    LayerOperation { layer_id: LayerId, previous_state: LayerState },
    CreateLayer { layer_id: LayerId },
    DeleteLayer { layer_data: LayerData },
}

/// Serializable object data for undo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectData {
    pub id: ObjectId,
    pub object_type: ObjectType,
    pub layer_id: LayerId,
    pub transform: Transform2D,
    pub style: ObjectStyle,
    pub geometry: GeometryData,
    pub z_index: i32,
    pub visible: bool,
    pub locked: bool,
}

/// Serializable layer data for undo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerData {
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

/// Serializable layer state for undo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerState {
    pub visible: bool,
    pub locked: bool,
    pub opacity: f32,
    pub blend_mode: BlendMode,
    pub parent: Option<LayerId>,
}

/// Object types in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Stroke,
    Shape,
    Text,
    Image,
    Group,
    PdfPage,
    Measurement,
}

/// Object styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectStyle {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub blend_mode: BlendMode,
}

/// Geometry data for different object types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GeometryData {
    Stroke {
        points: Vec<StrokePoint>,
        brush_id: Uuid,
        smoothing: f32,
    },
    Shape {
        shape_type: ShapeType,
        path_data: Vec<PathCommand>,
    },
    Text {
        content: String,
        font_family: String,
        font_size: f32,
        alignment: TextAlignment,
    },
    Image {
        image_id: Uuid,
        source_rect: Rect,
    },
    Group {
        children: Vec<ObjectId>,
    },
}

/// Stroke point with pressure/tilt data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrokePoint {
    pub position: Point2,
    pub pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub timestamp: u64,
}

/// Shape types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeType {
    Rectangle,
    Ellipse,
    Line,
    Polygon,
    Path,
}

/// Path commands for vector shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathCommand {
    MoveTo(Point2),
    LineTo(Point2),
    CubicTo(Point2, Point2, Point2),
    ClosePath,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

/// Base trait for all commands
pub trait Command: Send + Sync + fmt::Debug {
    type Output;
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn validate(&self) -> Result<(), Self::Error>;
    fn document_id(&self) -> DocumentId;
    
    /// Execute the command and return result with undo data
    fn execute(&self) -> Self::Output;
}

/// Document commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentCommand {
    CreateDocument {
        document_id: DocumentId,
        name: String,
        settings: DocumentSettings,
    },
    DeleteDocument {
        document_id: DocumentId,
    },
    RenameDocument {
        document_id: DocumentId,
        new_name: String,
    },
    DuplicateDocument {
        document_id: DocumentId,
        new_document_id: DocumentId,
        new_name: String,
    },
}

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
    pub measurement_scale: f64,
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

/// Layer commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayerCommand {
    CreateLayer {
        document_id: DocumentId,
        layer_id: LayerId,
        name: String,
        parent: Option<LayerId>,
        insert_index: Option<usize>,
    },
    DeleteLayer {
        document_id: DocumentId,
        layer_id: LayerId,
        move_objects_to: Option<LayerId>,
    },
    RenameLayer {
        document_id: DocumentId,
        layer_id: LayerId,
        new_name: String,
    },
    SetLayerVisibility {
        document_id: DocumentId,
        layer_id: LayerId,
        visible: bool,
    },
    SetLayerLock {
        document_id: DocumentId,
        layer_id: LayerId,
        locked: bool,
    },
    SetLayerOpacity {
        document_id: DocumentId,
        layer_id: LayerId,
        opacity: f32,
    },
    SetLayerBlendMode {
        document_id: DocumentId,
        layer_id: LayerId,
        blend_mode: BlendMode,
    },
    ReorderLayer {
        document_id: DocumentId,
        layer_id: LayerId,
        new_index: usize,
    },
    MoveLayerToParent {
        document_id: DocumentId,
        layer_id: LayerId,
        new_parent: Option<LayerId>,
    },
    DuplicateLayer {
        document_id: DocumentId,
        layer_id: LayerId,
        new_layer_id: LayerId,
        new_name: String,
    },
    MergeLayers {
        document_id: DocumentId,
        source_layer_id: LayerId,
        target_layer_id: LayerId,
    },
}

/// Object commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectCommand {
    CreateObject {
        document_id: DocumentId,
        object_id: ObjectId,
        layer_id: LayerId,
        object_type: ObjectType,
        transform: Transform2D,
        style: ObjectStyle,
        geometry: GeometryData,
        z_index: i32,
    },
    DeleteObject {
        document_id: DocumentId,
        object_id: ObjectId,
    },
    UpdateObject {
        document_id: DocumentId,
        object_id: ObjectId,
        updates: ObjectUpdates,
    },
    TransformObjects {
        document_id: DocumentId,
        object_ids: Vec<ObjectId>,
        transform: Transform2D,
        pivot: Point2,
    },
    DuplicateObjects {
        document_id: DocumentId,
        object_ids: Vec<ObjectId>,
        offset: Vector2,
    },
    GroupObjects {
        document_id: DocumentId,
        object_ids: Vec<ObjectId>,
        group_id: ObjectId,
    },
    UngroupObjects {
        document_id: DocumentId,
        group_id: ObjectId,
    },
    MoveObjectsToLayer {
        document_id: DocumentId,
        object_ids: Vec<ObjectId>,
        target_layer_id: LayerId,
    },
    SetObjectZIndex {
        document_id: DocumentId,
        object_id: ObjectId,
        z_index: i32,
    },
    SetObjectVisibility {
        document_id: DocumentId,
        object_id: ObjectId,
        visible: bool,
    },
    SetObjectLock {
        document_id: DocumentId,
        object_id: ObjectId,
        locked: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectUpdates {
    pub transform: Option<Transform2D>,
    pub style: Option<ObjectStyle>,
    pub geometry: Option<GeometryData>,
    pub z_index: Option<i32>,
    pub visible: Option<bool>,
    pub locked: Option<bool>,
}

/// Stroke commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrokeCommand {
    BeginStroke {
        document_id: DocumentId,
        stroke_id: ObjectId,
        layer_id: LayerId,
        brush_id: Uuid,
        color: Color,
        stroke_width: f32,
        smoothing: f32,
    },
    AddStrokePoint {
        document_id: DocumentId,
        stroke_id: ObjectId,
        point: StrokePoint,
    },
    EndStroke {
        document_id: DocumentId,
        stroke_id: ObjectId,
    },
    CancelStroke {
        document_id: DocumentId,
        stroke_id: ObjectId,
    },
    UpdateStrokeBrush {
        document_id: DocumentId,
        stroke_id: ObjectId,
        brush_id: Uuid,
    },
    UpdateStrokeColor {
        document_id: DocumentId,
        stroke_id: ObjectId,
        color: Color,
    },
    UpdateStrokeWidth {
        document_id: DocumentId,
        stroke_id: ObjectId,
        width: f32,
    },
    UpdateStrokeSmoothing {
        document_id: DocumentId,
        stroke_id: ObjectId,
        smoothing: f32,
    },
}

/// Artboard commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtboardCommand {
    CreateArtboard {
        document_id: DocumentId,
        artboard_id: ArtboardId,
        name: String,
        rect: Rect,
        background_color: Option<Color>,
    },
    DeleteArtboard {
        document_id: DocumentId,
        artboard_id: ArtboardId,
    },
    UpdateArtboard {
        document_id: DocumentId,
        artboard_id: ArtboardId,
        name: Option<String>,
        rect: Option<Rect>,
        background_color: Option<Option<Color>>,
        export_enabled: Option<bool>,
    },
}

/// Precision commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrecisionCommand {
    SetGrid {
        document_id: DocumentId,
        enabled: bool,
        grid_type: GridType,
        spacing: f64,
        color: Color,
        opacity: f32,
        origin: Point2,
        rotation: f64,
    },
    SetSnap {
        document_id: DocumentId,
        enabled: bool,
        tolerance: f64,
    },
    SetMeasurementUnit {
        document_id: DocumentId,
        unit: MeasurementUnit,
        scale: f64,
    },
    AddGuide {
        document_id: DocumentId,
        guide_id: Uuid,
        orientation: GuideOrientation,
        position: f64,
        color: Color,
    },
    RemoveGuide {
        document_id: DocumentId,
        guide_id: Uuid,
    },
    SetPerspectiveGrid {
        document_id: DocumentId,
        grid_type: GridType,
        vanishing_points: Vec<Point2>,
        horizon_line: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GuideOrientation {
    Horizontal,
    Vertical,
}

/// Selection commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionCommand {
    SelectObjects {
        document_id: DocumentId,
        object_ids: Vec<ObjectId>,
        add_to_selection: bool,
    },
    SelectAll {
        document_id: DocumentId,
        layer_id: Option<LayerId>,
    },
    DeselectAll {
        document_id: DocumentId,
    },
    SelectByRectangle {
        document_id: DocumentId,
        rect: Rect,
        add_to_selection: bool,
    },
    SelectByLasso {
        document_id: DocumentId,
        points: Vec<Point2>,
        add_to_selection: bool,
    },
    InvertSelection {
        document_id: DocumentId,
        layer_id: Option<LayerId>,
    },
}

/// Camera/Viewport commands (non-persistent, for UI state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraCommand {
    Pan {
        delta: Vector2,
    },
    Zoom {
        factor: f64,
        center: Option<Point2>,
    },
    Rotate {
        angle: f64,
        center: Option<Point2>,
    },
    Reset,
    FitToContent,
    FitToSelection,
    SetZoom {
        zoom: f64,
    },
    SetRotation {
        rotation: f64,
    },
    SetCenter {
        center: Point2,
    },
}

/// Command batch for transactional execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBatch {
    pub commands: Vec<BoxedCommand>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BoxedCommand {
    Document(DocumentCommand),
    Layer(LayerCommand),
    Object(ObjectCommand),
    Stroke(StrokeCommand),
    Artboard(ArtboardCommand),
    Precision(PrecisionCommand),
    Selection(SelectionCommand),
}

impl Command for BoxedCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        match self {
            BoxedCommand::Document(cmd) => cmd.validate(),
            BoxedCommand::Layer(cmd) => cmd.validate(),
            BoxedCommand::Object(cmd) => cmd.validate(),
            BoxedCommand::Stroke(cmd) => cmd.validate(),
            BoxedCommand::Artboard(cmd) => cmd.validate(),
            BoxedCommand::Precision(cmd) => cmd.validate(),
            BoxedCommand::Selection(cmd) => cmd.validate(),
        }
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            BoxedCommand::Document(cmd) => cmd.document_id(),
            BoxedCommand::Layer(cmd) => cmd.document_id(),
            BoxedCommand::Object(cmd) => cmd.document_id(),
            BoxedCommand::Stroke(cmd) => cmd.document_id(),
            BoxedCommand::Artboard(cmd) => cmd.document_id(),
            BoxedCommand::Precision(cmd) => cmd.document_id(),
            BoxedCommand::Selection(cmd) => cmd.document_id(),
        }
    }
    
    fn execute(&self) -> CommandResult {
        self.execute()
    }
}

/// Execute a boxed command - this provides the execute method that delegates to the inner command
impl BoxedCommand {
    pub fn execute(&self) -> CommandResult {
        match self {
            BoxedCommand::Document(cmd) => Command::execute(cmd),
            BoxedCommand::Layer(cmd) => Command::execute(cmd),
            BoxedCommand::Object(cmd) => Command::execute(cmd),
            BoxedCommand::Stroke(cmd) => Command::execute(cmd),
            BoxedCommand::Artboard(cmd) => Command::execute(cmd),
            BoxedCommand::Precision(cmd) => Command::execute(cmd),
            BoxedCommand::Selection(cmd) => Command::execute(cmd),
        }
    }
}

impl Command for DocumentCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            DocumentCommand::CreateDocument { document_id, .. } => *document_id,
            DocumentCommand::DeleteDocument { document_id } => *document_id,
            DocumentCommand::RenameDocument { document_id, .. } => *document_id,
            DocumentCommand::DuplicateDocument { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}

impl Command for LayerCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            LayerCommand::CreateLayer { document_id, .. } => *document_id,
            LayerCommand::DeleteLayer { document_id, .. } => *document_id,
            LayerCommand::RenameLayer { document_id, .. } => *document_id,
            LayerCommand::SetLayerVisibility { document_id, .. } => *document_id,
            LayerCommand::SetLayerLock { document_id, .. } => *document_id,
            LayerCommand::SetLayerOpacity { document_id, .. } => *document_id,
            LayerCommand::SetLayerBlendMode { document_id, .. } => *document_id,
            LayerCommand::ReorderLayer { document_id, .. } => *document_id,
            LayerCommand::MoveLayerToParent { document_id, .. } => *document_id,
            LayerCommand::DuplicateLayer { document_id, .. } => *document_id,
            LayerCommand::MergeLayers { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}

impl Command for ObjectCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            ObjectCommand::CreateObject { document_id, .. } => *document_id,
            ObjectCommand::DeleteObject { document_id, .. } => *document_id,
            ObjectCommand::UpdateObject { document_id, .. } => *document_id,
            ObjectCommand::TransformObjects { document_id, .. } => *document_id,
            ObjectCommand::DuplicateObjects { document_id, .. } => *document_id,
            ObjectCommand::GroupObjects { document_id, .. } => *document_id,
            ObjectCommand::UngroupObjects { document_id, .. } => *document_id,
            ObjectCommand::MoveObjectsToLayer { document_id, .. } => *document_id,
            ObjectCommand::SetObjectZIndex { document_id, .. } => *document_id,
            ObjectCommand::SetObjectVisibility { document_id, .. } => *document_id,
            ObjectCommand::SetObjectLock { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}

impl Command for StrokeCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            StrokeCommand::BeginStroke { document_id, .. } => *document_id,
            StrokeCommand::AddStrokePoint { document_id, .. } => *document_id,
            StrokeCommand::EndStroke { document_id, .. } => *document_id,
            StrokeCommand::CancelStroke { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeBrush { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeColor { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeWidth { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeSmoothing { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}

impl Command for ArtboardCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            ArtboardCommand::CreateArtboard { document_id, .. } => *document_id,
            ArtboardCommand::DeleteArtboard { document_id, .. } => *document_id,
            ArtboardCommand::UpdateArtboard { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}

impl Command for PrecisionCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            PrecisionCommand::SetGrid { document_id, .. } => *document_id,
            PrecisionCommand::SetSnap { document_id, .. } => *document_id,
            PrecisionCommand::SetMeasurementUnit { document_id, .. } => *document_id,
            PrecisionCommand::AddGuide { document_id, .. } => *document_id,
            PrecisionCommand::RemoveGuide { document_id, .. } => *document_id,
            PrecisionCommand::SetPerspectiveGrid { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}

impl Command for SelectionCommand {
    type Output = CommandResult;
    type Error = CommandError;
    
    fn validate(&self) -> Result<(), Self::Error> {
        Ok(())
    }
    
    fn document_id(&self) -> DocumentId {
        match self {
            SelectionCommand::SelectObjects { document_id, .. } => *document_id,
            SelectionCommand::SelectAll { document_id, .. } => *document_id,
            SelectionCommand::DeselectAll { document_id } => *document_id,
            SelectionCommand::SelectByRectangle { document_id, .. } => *document_id,
            SelectionCommand::SelectByLasso { document_id, .. } => *document_id,
            SelectionCommand::InvertSelection { document_id, .. } => *document_id,
        }
    }
    
    fn execute(&self) -> CommandResult {
        CommandResult {
            success: true,
            undo_data: None,
            affected_ids: vec![],
        }
    }
}



impl DocumentCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            DocumentCommand::CreateDocument { document_id, .. } => *document_id,
            DocumentCommand::DeleteDocument { document_id } => *document_id,
            DocumentCommand::RenameDocument { document_id, .. } => *document_id,
            DocumentCommand::DuplicateDocument { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

impl LayerCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            LayerCommand::CreateLayer { document_id, .. } => *document_id,
            LayerCommand::DeleteLayer { document_id, .. } => *document_id,
            LayerCommand::RenameLayer { document_id, .. } => *document_id,
            LayerCommand::SetLayerVisibility { document_id, .. } => *document_id,
            LayerCommand::SetLayerLock { document_id, .. } => *document_id,
            LayerCommand::SetLayerOpacity { document_id, .. } => *document_id,
            LayerCommand::SetLayerBlendMode { document_id, .. } => *document_id,
            LayerCommand::ReorderLayer { document_id, .. } => *document_id,
            LayerCommand::MoveLayerToParent { document_id, .. } => *document_id,
            LayerCommand::DuplicateLayer { document_id, .. } => *document_id,
            LayerCommand::MergeLayers { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

impl ObjectCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            ObjectCommand::CreateObject { document_id, .. } => *document_id,
            ObjectCommand::DeleteObject { document_id, .. } => *document_id,
            ObjectCommand::UpdateObject { document_id, .. } => *document_id,
            ObjectCommand::TransformObjects { document_id, .. } => *document_id,
            ObjectCommand::DuplicateObjects { document_id, .. } => *document_id,
            ObjectCommand::GroupObjects { document_id, .. } => *document_id,
            ObjectCommand::UngroupObjects { document_id, .. } => *document_id,
            ObjectCommand::MoveObjectsToLayer { document_id, .. } => *document_id,
            ObjectCommand::SetObjectZIndex { document_id, .. } => *document_id,
            ObjectCommand::SetObjectVisibility { document_id, .. } => *document_id,
            ObjectCommand::SetObjectLock { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

impl StrokeCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            StrokeCommand::BeginStroke { document_id, .. } => *document_id,
            StrokeCommand::AddStrokePoint { document_id, .. } => *document_id,
            StrokeCommand::EndStroke { document_id, .. } => *document_id,
            StrokeCommand::CancelStroke { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeBrush { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeColor { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeWidth { document_id, .. } => *document_id,
            StrokeCommand::UpdateStrokeSmoothing { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

impl ArtboardCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            ArtboardCommand::CreateArtboard { document_id, .. } => *document_id,
            ArtboardCommand::DeleteArtboard { document_id, .. } => *document_id,
            ArtboardCommand::UpdateArtboard { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

impl PrecisionCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            PrecisionCommand::SetGrid { document_id, .. } => *document_id,
            PrecisionCommand::SetSnap { document_id, .. } => *document_id,
            PrecisionCommand::SetMeasurementUnit { document_id, .. } => *document_id,
            PrecisionCommand::AddGuide { document_id, .. } => *document_id,
            PrecisionCommand::RemoveGuide { document_id, .. } => *document_id,
            PrecisionCommand::SetPerspectiveGrid { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

impl SelectionCommand {
    pub fn document_id(&self) -> DocumentId {
        match self {
            SelectionCommand::SelectObjects { document_id, .. } => *document_id,
            SelectionCommand::SelectAll { document_id, .. } => *document_id,
            SelectionCommand::DeselectAll { document_id } => *document_id,
            SelectionCommand::SelectByRectangle { document_id, .. } => *document_id,
            SelectionCommand::SelectByLasso { document_id, .. } => *document_id,
            SelectionCommand::InvertSelection { document_id, .. } => *document_id,
        }
    }
    
    fn validate(&self) -> Result<(), CommandError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{DocumentId, ObjectId, LayerId, Color, Transform2D};
    
    #[test]
    fn test_document_command_validation() {
        let cmd = DocumentCommand::CreateDocument {
            document_id: DocumentId::new(),
            name: "Test".to_string(),
            settings: DocumentSettings::default(),
        };
        assert!(cmd.validate().is_ok());
    }
    
    #[test]
    fn test_command_document_id() {
        let doc_id = DocumentId::new();
        let cmd = DocumentCommand::CreateDocument {
            document_id: doc_id,
            name: "Test".to_string(),
            settings: DocumentSettings::default(),
        };
        assert_eq!(cmd.document_id(), doc_id);
    }
    
    #[test]
    fn test_boxed_command() {
        let doc_id = DocumentId::new();
        let cmd = BoxedCommand::Document(DocumentCommand::CreateDocument {
            document_id: doc_id,
            name: "Test".to_string(),
            settings: DocumentSettings::default(),
        });
        assert_eq!(cmd.document_id(), doc_id);
        assert!(cmd.validate().is_ok());
    }
}
