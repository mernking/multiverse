//! Storage layer for Creative Engine
//!
//! Provides atomic save/load, autosave journal, and crash recovery
//! for native `.ceng` document format (ZIP container with manifest + JSON).

use creative_domain::{DocumentId, DocumentMeta, DocumentSettings, Layer, LayerId, ObjectId, Artboard, ArtboardId, Camera};
use creative_geometry::Geometry;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

/// Storage errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Document not found: {0}")]
    DocumentNotFound(DocumentId),
    #[error("Corrupt document: {0}")]
    CorruptDocument(String),
    #[error("Version incompatibility: {0}")]
    VersionIncompatibility(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Autosave error: {0}")]
    Autosave(String),
}

/// Document format version
pub const DOCUMENT_VERSION: u32 = 1;
pub const DOCUMENT_MAGIC: &[u8] = b"CENG";

/// Native document format (in-memory representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
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
    pub transform: creative_domain::Transform2D,
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
    pub fill: Option<creative_domain::Color>,
    pub stroke: Option<creative_domain::Color>,
    pub stroke_width: f32,
    pub opacity: f32,
    pub blend_mode: creative_domain::BlendMode,
}

/// Selection state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionState {
    pub object_ids: Vec<ObjectId>,
    pub layer_id: Option<LayerId>,
}

/// Manifest for the ZIP container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    pub document_id: DocumentId,
    pub created: u64,
    pub modified: u64,
    pub app_version: String,
    pub object_count: usize,
    pub layer_count: usize,
    pub geometry_count: usize,
    pub artboard_count: usize,
    pub schema_version: u32,
}

/// Document storage handler
pub struct DocumentStorage {
    autosave_dir: PathBuf,
    autosave_interval_ms: u64,
}

impl DocumentStorage {
    pub fn new(autosave_dir: PathBuf, autosave_interval_ms: u64) -> Self {
        std::fs::create_dir_all(&autosave_dir).ok();
        Self {
            autosave_dir,
            autosave_interval_ms,
        }
    }
    
    /// Save document to .ceng file (ZIP container)
    pub fn save(&self, path: &Path, document: &Document) -> Result<(), StorageError> {
        // Create temporary file for atomic write
        let temp_path = path.with_extension("ceng.tmp");
        
        self.save_to_file(&temp_path, document)?;
        
        // Atomic rename
        std::fs::rename(&temp_path, path)?;
        
        Ok(())
    }
    
    /// Save document to a specific file path (used for temp/autosave)
    fn save_to_file(&self, path: &Path, document: &Document) -> Result<(), StorageError> {
        let file = std::fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .compression_level(Some(6));
        
        // Write magic header
        zip.start_file("magic", options)?;
        zip.write_all(DOCUMENT_MAGIC)?;
        
        // Write manifest
        let manifest = Manifest {
            version: DOCUMENT_VERSION,
            document_id: document.meta.id,
            created: document.meta.created,
            modified: document.meta.modified,
            app_version: document.meta.app_version.clone(),
            object_count: document.objects.len(),
            layer_count: document.layers.len(),
            geometry_count: document.geometries.len(),
            artboard_count: document.artboards.len(),
            schema_version: document.meta.version,
        };
        let manifest_json = serde_json::to_vec(&manifest)?;
        zip.start_file("manifest.json", options)?;
        zip.write_all(&manifest_json)?;
        
        // Write document data
        let doc_json = serde_json::to_vec(document)?;
        zip.start_file("document.json", options)?;
        zip.write_all(&doc_json)?;
        
        zip.finish()?;
        Ok(())
    }
    
    /// Load document from .ceng file
    pub fn load(&self, path: &Path) -> Result<Document, StorageError> {
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(file)?;
        
        // Verify magic header
        {
            let mut magic_file = zip.by_name("magic")?;
            let mut magic = vec![0u8; DOCUMENT_MAGIC.len()];
            magic_file.read_exact(&mut magic)?;
            if magic != DOCUMENT_MAGIC {
                return Err(StorageError::CorruptDocument("Invalid magic header".to_string()));
            }
        }
        
        // Read manifest
        let manifest: Manifest = {
            let mut manifest_file = zip.by_name("manifest.json")?;
            let mut manifest_json = String::new();
            std::io::Read::read_to_string(&mut manifest_file, &mut manifest_json)?;
            serde_json::from_str(&manifest_json)?
        };
        
        // Check version compatibility
        if manifest.version > DOCUMENT_VERSION {
            return Err(StorageError::VersionIncompatibility(
                format!("Document version {} > supported version {}", manifest.version, DOCUMENT_VERSION)
            ));
        }
        
        // Read document
        let document: Document = {
            let mut doc_file = zip.by_name("document.json")?;
            let mut doc_json = String::new();
            std::io::Read::read_to_string(&mut doc_file, &mut doc_json)?;
            serde_json::from_str(&doc_json)?
        };
        
        // Verify document ID matches manifest
        if document.meta.id != manifest.document_id {
            return Err(StorageError::CorruptDocument("Document ID mismatch".to_string()));
        }
        
        Ok(document)
    }
    
    /// Autosave document to journal
    pub fn autosave(&self, document: &Document) -> Result<PathBuf, StorageError> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let filename = format!("autosave_{}_{}.ceng", document.meta.id.0, timestamp);
        let path = self.autosave_dir.join(filename);
        
        self.save_to_file(&path, document)?;
        
        // Clean old autosaves (keep last 10)
        self.clean_autosaves(document.meta.id)?;
        
        Ok(path)
    }
    
    /// Find latest autosave for a document
    pub fn find_latest_autosave(&self, document_id: DocumentId) -> Option<PathBuf> {
        let prefix = format!("autosave_{}_", document_id.0);
        let mut entries: Vec<_> = std::fs::read_dir(&self.autosave_dir).ok()?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .collect();
        
        entries.sort_by_key(|e| e.file_name());
        entries.last().map(|e| e.path())
    }
    
    /// Recover from autosave
    pub fn recover(&self, document_id: DocumentId) -> Result<Document, StorageError> {
        let autosave_path = self.find_latest_autosave(document_id)
            .ok_or(StorageError::DocumentNotFound(document_id))?;
        
        self.load(&autosave_path)
    }
    
    /// Clean old autosaves (keep last N)
    fn clean_autosaves(&self, document_id: DocumentId) -> Result<(), StorageError> {
        let prefix = format!("autosave_{}_", document_id.0);
        let mut entries: Vec<_> = std::fs::read_dir(&self.autosave_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .collect();
        
        entries.sort_by_key(|e| e.file_name());
        
        // Keep last 10
        if entries.len() > 10 {
            for entry in entries[..entries.len() - 10].iter() {
                std::fs::remove_file(entry.path()).ok();
            }
        }
        
        Ok(())
    }
    
    /// Validate document integrity
    pub fn validate(&self, document: &Document) -> Result<(), StorageError> {
        // Check required fields
        if document.objects.is_empty() && document.layers.is_empty() {
            return Err(StorageError::ValidationFailed("Empty document".to_string()));
        }
        
        // Check all objects reference valid layers
        for obj in document.objects.values() {
            if !document.layers.iter().any(|l| l.id == obj.layer_id) {
                return Err(StorageError::ValidationFailed(
                    format!("Object {} references non-existent layer {}", obj.id, obj.layer_id)
                ));
            }
        }
        
        // Check all objects reference valid geometries
        for obj in document.objects.values() {
            if !document.geometries.contains_key(&obj.geometry_id) {
                return Err(StorageError::ValidationFailed(
                    format!("Object {} references non-existent geometry {}", obj.id, obj.geometry_id)
                ));
            }
        }
        
        // Check layer hierarchy consistency
        let layer_ids: std::collections::HashSet<_> = document.layers.iter().map(|l| l.id).collect();
        for layer in &document.layers {
            if let Some(parent) = layer.parent {
                if !layer_ids.contains(&parent) {
                    return Err(StorageError::ValidationFailed(
                        format!("Layer {} references non-existent parent {}", layer.id, parent)
                    ));
                }
            }
            for child in &layer.children {
                if !layer_ids.contains(child) {
                    return Err(StorageError::ValidationFailed(
                        format!("Layer {} references non-existent child {}", layer.id, child)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Export document to JSON (for debugging/interop)
    pub fn export_json(&self, document: &Document) -> Result<String, StorageError> {
        serde_json::to_string_pretty(document).map_err(StorageError::Serialization)
    }
    
    /// Import document from JSON
    pub fn import_json(&self, json: &str) -> Result<Document, StorageError> {
        serde_json::from_str(json).map_err(StorageError::Serialization)
    }
}

/// Background autosave service
pub struct AutosaveService {
    storage: Arc<DocumentStorage>,
    interval_ms: u64,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl AutosaveService {
    pub fn new(storage: Arc<DocumentStorage>, interval_ms: u64) -> Self {
        Self {
            storage,
            interval_ms,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    pub fn start(&self, document: Arc<std::sync::RwLock<Document>>) {
        self.running.store(true, std::sync::atomic::Ordering::Relaxed);
        
        let storage = self.storage.clone();
        let interval = self.interval_ms;
        let running = self.running.clone();
        
        std::thread::spawn(move || {
            while running.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(interval));
                
                if !running.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                
                let doc = document.read().unwrap();
                if let Err(e) = storage.autosave(&doc) {
                    eprintln!("Autosave failed: {}", e);
                }
            }
        });
    }
    
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::{DocumentId, DocumentMeta, DocumentSettings, Layer, LayerId, ObjectId, Artboard, ArtboardId, Camera, Color, Point2, Rect, BlendMode, Transform2D, GridType, MeasurementUnit};
    use creative_geometry::{Geometry, ShapeGeometry, ShapeType, GeometryId};
    use tempfile::TempDir;
    use std::collections::HashMap;
    
    fn create_test_document() -> Document {
        let doc_id = DocumentId::new();
        let layer_id = LayerId::new();
        let obj_id = ObjectId::new();
        let geom_id = GeometryId::new();
        let artboard_id = ArtboardId::new();
        
        let mut objects = HashMap::new();
        let mut geometries = HashMap::new();
        
        let rect = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        let shape = ShapeGeometry::new(ShapeType::Rectangle, rect);
        let geometry = Geometry::Shape(shape);
        geometries.insert(geom_id, geometry);
        
        objects.insert(obj_id, DocumentObject {
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
        });
        
        Document {
            meta: DocumentMeta {
                id: doc_id,
                name: "Test Document".to_string(),
                created: 1000,
                modified: 2000,
                version: 1,
                app_version: "0.1.0".to_string(),
            },
            settings: DocumentSettings::default(),
            camera: Camera::new(),
            layers: vec![Layer { id: layer_id, name: "Layer 1".to_string(), visible: true, locked: false, opacity: 1.0, blend_mode: BlendMode::Normal, object_ids: vec![], parent: None, children: vec![] }],
            objects,
            artboards: vec![Artboard::new("Artboard 1", Rect::new(Point2::ZERO, Point2::new(1920.0, 1080.0)))],
            geometries,
            selection: SelectionState::default(),
        }
    }
    
    #[test]
    fn test_save_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DocumentStorage::new(temp_dir.path().join("autosave"), 1000);
        let document = create_test_document();
        let path = temp_dir.path().join("test.ceng");
        
        storage.save(&path, &document).unwrap();
        let loaded = storage.load(&path).unwrap();
        
        assert_eq!(loaded.meta.id, document.meta.id);
        assert_eq!(loaded.meta.name, document.meta.name);
        assert_eq!(loaded.objects.len(), document.objects.len());
        assert_eq!(loaded.layers.len(), document.layers.len());
        assert_eq!(loaded.geometries.len(), document.geometries.len());
    }
    
    #[test]
    fn test_autosave_and_recover() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DocumentStorage::new(temp_dir.path().join("autosave"), 1000);
        let document = create_test_document();
        let doc_id = document.meta.id;
        
        let autosave_path = storage.autosave(&document).unwrap();
        assert!(autosave_path.exists());
        
        let recovered = storage.recover(doc_id).unwrap();
        assert_eq!(recovered.meta.id, document.meta.id);
        assert_eq!(recovered.objects.len(), document.objects.len());
    }
    
    #[test]
    fn test_validate_document() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DocumentStorage::new(temp_dir.path().join("autosave"), 1000);
        let document = create_test_document();
        
        assert!(storage.validate(&document).is_ok());
    }
    
    #[test]
    fn test_validate_fails_on_bad_layer_ref() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DocumentStorage::new(temp_dir.path().join("autosave"), 1000);
        let mut document = create_test_document();
        
        // Add object with invalid layer reference
        let bad_layer_id = LayerId::new();
        let bad_obj_id = ObjectId::new();
        let bad_geom_id = GeometryId::new();
        
        let rect = Rect::new(Point2::new(0.0, 0.0), Point2::new(100.0, 100.0));
        let shape = ShapeGeometry::new(ShapeType::Rectangle, rect);
        let geometry = Geometry::Shape(shape);
        document.geometries.insert(bad_geom_id, geometry);
        
        document.objects.insert(bad_obj_id, DocumentObject {
            id: bad_obj_id,
            layer_id: bad_layer_id, // Invalid layer
            object_type: ObjectType::Shape,
            transform: Transform2D::IDENTITY,
            style: ObjectStyle {
                fill: Some(Color::RED),
                stroke: None,
                stroke_width: 1.0,
                opacity: 1.0,
                blend_mode: BlendMode::Normal,
            },
            geometry_id: bad_geom_id,
            z_index: 0,
            visible: true,
            locked: false,
        });
        
        assert!(storage.validate(&document).is_err());
    }
    
    #[test]
    fn test_export_import_json() {
        let temp_dir = TempDir::new().unwrap();
        let storage = DocumentStorage::new(temp_dir.path().join("autosave"), 1000);
        let document = create_test_document();
        
        let json = storage.export_json(&document).unwrap();
        let imported = storage.import_json(&json).unwrap();
        
        assert_eq!(imported.meta.id, document.meta.id);
        assert_eq!(imported.objects.len(), document.objects.len());
    }
}