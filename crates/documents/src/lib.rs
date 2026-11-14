//! Document model and migrations for Creative Engine
//!
//! Provides the native file model, schema versioning, and migration framework
//! for the `.ceng` document format.

use creative_domain::{DocumentId, DocumentMeta, DocumentSettings, Layer, LayerId, ObjectId, Artboard, ArtboardId, Camera, Color, BlendMode, Transform2D};
use creative_geometry::Geometry;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

/// Document model errors
#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    #[error("Unsupported schema version: {0}")]
    UnsupportedVersion(u32),
    #[error("Invalid document state: {0}")]
    InvalidState(String),
    #[error("Schema validation error: {0}")]
    SchemaValidation(String),
}

/// Current schema version
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Document model with versioned schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentModel {
    pub schema_version: u32,
    pub meta: DocumentMeta,
    pub settings: DocumentSettings,
    pub camera: Camera,
    pub layers: Vec<Layer>,
    pub objects: HashMap<ObjectId, DocumentObject>,
    pub artboards: Vec<Artboard>,
    pub geometries: HashMap<creative_geometry::GeometryId, Geometry>,
    pub selection: SelectionState,
}

/// Document object entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentObject {
    pub id: ObjectId,
    pub layer_id: LayerId,
    pub object_type: ObjectType,
    pub transform: Transform2D,
    pub style: ObjectStyle,
    pub geometry_id: creative_geometry::GeometryId,
    pub z_index: i32,
    pub visible: bool,
    pub locked: bool,
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

/// Selection state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionState {
    pub object_ids: Vec<ObjectId>,
    pub layer_id: Option<LayerId>,
}

/// Migration trait for schema version upgrades
pub trait Migration: Send + Sync {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, model: DocumentModel) -> Result<DocumentModel, DocumentError>;
}

/// Migration registry
pub struct MigrationRegistry {
    migrations: Vec<Box<dyn Migration>>,
}

impl MigrationRegistry {
    pub fn new() -> Self {
        Self { migrations: Vec::new() }
    }
    
    pub fn register(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
    }
    
    /// Migrate a document from its current version to the latest
    pub fn migrate_to_latest(&self, mut model: DocumentModel) -> Result<DocumentModel, DocumentError> {
        let mut current_version = model.schema_version;
        
        while current_version < CURRENT_SCHEMA_VERSION {
            // Find migration for current version
            let migration = self.migrations.iter()
                .find(|m| m.from_version() == current_version)
                .ok_or_else(|| DocumentError::MigrationFailed(
                    format!("No migration from version {} to {}", current_version, current_version + 1)
                ))?;
            
            model = migration.migrate(model)?;
            current_version = model.schema_version;
        }
        
        Ok(model)
    }
    
    /// Get available migration path
    pub fn get_migration_path(&self, from_version: u32) -> Vec<u32> {
        let mut path = Vec::new();
        let mut current = from_version;
        
        while current < CURRENT_SCHEMA_VERSION {
            if self.migrations.iter().any(|m| m.from_version() == current) {
                current += 1;
                path.push(current);
            } else {
                break;
            }
        }
        
        path
    }
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        let registry = Self::new();
        
        // Register migrations here as versions increase
        // registry.register(Box::new(V1ToV2Migration));
        
        registry
    }
}

/// Document builder for creating new documents
pub struct DocumentBuilder {
    name: String,
    settings: DocumentSettings,
    artboards: Vec<Artboard>,
}

impl DocumentBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            settings: DocumentSettings::default(),
            artboards: Vec::new(),
        }
    }
    
    pub fn with_settings(mut self, settings: DocumentSettings) -> Self {
        self.settings = settings;
        self
    }
    
    pub fn with_artboard(mut self, artboard: Artboard) -> Self {
        self.artboards.push(artboard);
        self
    }
    
    pub fn with_artboards(mut self, artboards: Vec<Artboard>) -> Self {
        self.artboards = artboards;
        self
    }
    
    pub fn build(self) -> DocumentModel {
        let doc_id = DocumentId::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        // Create default layer if no artboards
        let mut layers = Vec::new();
        if self.artboards.is_empty() {
            let layer = Layer::new("Layer 1");
            layers.push(layer);
        } else {
            // Create a layer per artboard
            for artboard in &self.artboards {
                let layer = Layer::new(&format!("{} Layer", artboard.name));
                layers.push(layer);
            }
        }
        
        DocumentModel {
            schema_version: CURRENT_SCHEMA_VERSION,
            meta: DocumentMeta {
                id: doc_id,
                name: self.name,
                created: now,
                modified: now,
                version: CURRENT_SCHEMA_VERSION,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            settings: self.settings,
            camera: Camera::new(),
            layers,
            objects: HashMap::new(),
            artboards: self.artboards,
            geometries: HashMap::new(),
            selection: SelectionState::default(),
        }
    }
}

/// Document operations
impl DocumentModel {
    /// Create a new document with default settings
    pub fn new(name: impl Into<String>) -> Self {
        DocumentBuilder::new(name).build()
    }
    
    /// Create a new document with artboards
    pub fn new_with_artboards(name: impl Into<String>, artboards: Vec<Artboard>) -> Self {
        DocumentBuilder::new(name).with_artboards(artboards).build()
    }
    
    /// Get object by ID
    pub fn get_object(&self, id: ObjectId) -> Option<&DocumentObject> {
        self.objects.get(&id)
    }
    
    /// Get mutable object by ID
    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut DocumentObject> {
        self.objects.get_mut(&id)
    }
    
    /// Add object to document
    pub fn add_object(&mut self, object: DocumentObject, geometry: Geometry) {
        self.geometries.insert(object.geometry_id, geometry);
        self.objects.insert(object.id, object);
        self.meta.modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Remove object from document
    pub fn remove_object(&mut self, id: ObjectId) -> Option<DocumentObject> {
        let obj = self.objects.remove(&id);
        if let Some(obj) = &obj {
            self.geometries.remove(&obj.geometry_id);
            self.meta.modified = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
        }
        obj
    }
    
    /// Get layer by ID
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }
    
    /// Get mutable layer by ID
    pub fn get_layer_mut(&mut self, id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id == id)
    }
    
    /// Add layer to document
    pub fn add_layer(&mut self, layer: Layer) {
        self.layers.push(layer);
        self.meta.modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Remove layer from document
    pub fn remove_layer(&mut self, id: LayerId) -> Option<Layer> {
        let index = self.layers.iter().position(|l| l.id == id)?;
        let layer = self.layers.remove(index);
        self.meta.modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Some(layer)
    }
    
    /// Get artboard by ID
    pub fn get_artboard(&self, id: ArtboardId) -> Option<&Artboard> {
        self.artboards.iter().find(|a| a.id == id)
    }
    
    /// Get all objects on a layer
    pub fn get_objects_on_layer(&self, layer_id: LayerId) -> Vec<&DocumentObject> {
        self.objects.values().filter(|o| o.layer_id == layer_id).collect()
    }
    
    /// Get visible objects (for rendering)
    pub fn get_visible_objects(&self) -> Vec<&DocumentObject> {
        self.objects.values().filter(|o| o.visible).collect()
    }
    
    /// Update modified timestamp
    pub fn touch(&mut self) {
        self.meta.modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
    }
    
    /// Get object count
    pub fn object_count(&self) -> usize {
        self.objects.len()
    }
    
    /// Get layer count
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
    
    /// Validate document integrity
    pub fn validate(&self) -> Result<(), DocumentError> {
        // Check all objects reference valid layers
        for obj in self.objects.values() {
            if !self.layers.iter().any(|l| l.id == obj.layer_id) {
                return Err(DocumentError::SchemaValidation(
                    format!("Object {} references non-existent layer {}", obj.id, obj.layer_id)
                ));
            }
        }
        
        // Check all objects reference valid geometries
        for obj in self.objects.values() {
            if !self.geometries.contains_key(&obj.geometry_id) {
                return Err(DocumentError::SchemaValidation(
                    format!("Object {} references non-existent geometry {}", obj.id, obj.geometry_id)
                ));
            }
        }
        
        // Check layer hierarchy consistency
        let layer_ids: std::collections::HashSet<_> = self.layers.iter().map(|l| l.id).collect();
        for layer in &self.layers {
            if let Some(parent) = layer.parent {
                if !layer_ids.contains(&parent) {
                    return Err(DocumentError::SchemaValidation(
                        format!("Layer {} references non-existent parent {}", layer.id, parent)
                    ));
                }
            }
            for child in &layer.children {
                if !layer_ids.contains(child) {
                    return Err(DocumentError::SchemaValidation(
                        format!("Layer {} references non-existent child {}", layer.id, child)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

impl Default for DocumentModel {
    fn default() -> Self {
        Self::new("Untitled")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{LayerId, ObjectId, Color, Point2, Rect, Transform2D, BlendMode};
    use creative_geometry::{Geometry, ShapeGeometry, ShapeType, GeometryId};
    
    fn create_test_object() -> (DocumentObject, Geometry) {
        let obj_id = ObjectId::new();
        let layer_id = LayerId::new();
        let geom_id = GeometryId::new();
        
        let rect = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        let shape = ShapeGeometry::new(ShapeType::Rectangle, rect);
        let geometry = Geometry::Shape(shape);
        
        let object = DocumentObject {
            id: obj_id,
            layer_id,
            object_type: ObjectType::Shape,
            transform: Transform2D::IDENTITY,
            style: ObjectStyle {
                fill: Some(Color::RED),
                stroke: Some(Color::BLACK),
                stroke_width: 2.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            },
            geometry_id: geom_id,
            z_index: 0,
            visible: true,
            locked: false,
        };
        
        (object, geometry)
    }
    
    #[test]
    fn test_document_creation() {
        let doc = DocumentModel::new("Test Document");
        
        assert_eq!(doc.meta.name, "Test Document");
        assert_eq!(doc.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(doc.layers.len(), 1);
        assert_eq!(doc.layers[0].name, "Layer 1");
    }
    
    #[test]
    fn test_document_with_artboards() {
        let artboard = Artboard::new("Artboard 1", Rect::new(Point2::ZERO, Point2::new(1920.0, 1080.0)));
        let doc = DocumentModel::new_with_artboards("Test", vec![artboard]);
        
        assert_eq!(doc.artboards.len(), 1);
        assert_eq!(doc.layers.len(), 1);
    }
    
    #[test]
    fn test_add_remove_object() {
        let mut doc = DocumentModel::new("Test");
        let (object, geometry) = create_test_object();
        let obj_id = object.id;
        
        doc.add_object(object, geometry);
        assert_eq!(doc.objects.len(), 1);
        assert_eq!(doc.geometries.len(), 1);
        
        let retrieved = doc.get_object(obj_id);
        assert!(retrieved.is_some());
        
        let removed = doc.remove_object(obj_id);
        assert!(removed.is_some());
        assert_eq!(doc.objects.len(), 0);
        assert_eq!(doc.geometries.len(), 0);
    }
    
    #[test]
    fn test_add_remove_layer() {
        let mut doc = DocumentModel::new("Test");
        let layer = Layer::new("Test Layer");
        let layer_id = layer.id;
        
        doc.add_layer(layer);
        assert_eq!(doc.layers.len(), 2); // Default + new
        
        let retrieved = doc.get_layer(layer_id);
        assert!(retrieved.is_some());
        
        let removed = doc.remove_layer(layer_id);
        assert!(removed.is_some());
        assert_eq!(doc.layers.len(), 1);
    }
    
    #[test]
    fn test_get_objects_on_layer() {
        let mut doc = DocumentModel::new("Test");
        let (object, geometry) = create_test_object();
        let layer_id = object.layer_id;
        
        doc.add_object(object, geometry);
        
        let objects = doc.get_objects_on_layer(layer_id);
        assert_eq!(objects.len(), 1);
    }
    
    #[test]
    fn test_get_visible_objects() {
        let mut doc = DocumentModel::new("Test");
        let (mut object, geometry) = create_test_object();
        object.visible = false;
        
        doc.add_object(object, geometry);
        
        let visible = doc.get_visible_objects();
        assert_eq!(visible.len(), 0);
    }
    
    #[test]
    fn test_validate_good_document() {
        let mut doc = DocumentModel::new("Test");
        let (object, geometry) = create_test_object();
        
        // Add the layer the object references
        let layer = Layer::new("Test Layer");
        let layer_id = layer.id;
        
        // Update object to reference this layer
        let mut obj = object;
        obj.layer_id = layer_id;
        
        doc.add_layer(layer);
        doc.add_object(obj, geometry);
        
        assert!(doc.validate().is_ok());
    }
    
    #[test]
    fn test_validate_fails_bad_layer() {
        let mut doc = DocumentModel::new("Test");
        let (object, geometry) = create_test_object();
        
        // Don't add the layer the object references
        doc.add_object(object, geometry);
        
        assert!(doc.validate().is_err());
    }
    
    #[test]
    fn test_document_builder() {
        let doc = DocumentBuilder::new("Built Document")
            .with_settings(DocumentSettings::default())
            .build();
        
        assert_eq!(doc.meta.name, "Built Document");
        assert_eq!(doc.schema_version, CURRENT_SCHEMA_VERSION);
    }
}