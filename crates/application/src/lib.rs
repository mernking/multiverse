//! Creative Engine Application - Core application logic
//!
//! This crate provides the main application state, command dispatch,
//! and integration between all engine subsystems.

use creative_domain::{
    DocumentId, ObjectId, LayerId, BrushId, Color, Point2, Rect, Transform2D,
    Camera, BlendMode
};
use creative_canvas::{Canvas, CanvasConfig, Tool, ToolState, Viewport as CanvasViewport};
use creative_renderer::{Renderer, RenderConfig, RenderLayer as RenderLayerType, Viewport, RenderItem};
use creative_renderer_software::SoftwareBackend;
use creative_input::{InputProcessor, InputConfig, PointerEvent, RawPointerEvent, RawKeyEvent};
use creative_brushes::{BrushEngine, BrushPreset, Stroke, StrokeConfig};
use creative_layers::LayerTree;
use creative_selection::SelectionManager;
use creative_domain::DocumentSettings;
use creative_storage::SelectionState as StorageSelectionState;
use creative_history::{History, HistoryError};
use creative_commands::{CommandResult, CommandError, BoxedCommand};
use creative_geometry::{Geometry, PathGeometry, PathSegment, ShapeGeometry, ShapeType, BoundingBox, StrokeStyle};
use creative_storage::{DocumentStorage, Document as StorageDocument, StorageError};

use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::warn;
/// Application errors
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Document not found: {0}")]
    DocumentNotFound(DocumentId),
    #[error("Layer not found: {0}")]
    LayerNotFound(LayerId),
    #[error("Object not found: {0}")]
    ObjectNotFound(ObjectId),
    #[error("Brush not found: {0}")]
    BrushNotFound(BrushId),
    #[error("Command error: {0}")]
    CommandError(#[from] CommandError),
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("Renderer error: {0}")]
    RendererError(#[from] creative_renderer::RenderError),
    #[error("Input error: {0}")]
    InputError(#[from] creative_input::InputError),
    #[error("Canvas error: {0}")]
    CanvasError(#[from] creative_canvas::CanvasError),
    #[error("Brush error: {0}")]
    BrushError(#[from] creative_brushes::BrushError),
    #[error("Layer error: {0}")]
    LayerError(#[from] creative_layers::LayerError),
    #[error("History error: {0}")]
    HistoryError(#[from] HistoryError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub render_config: RenderConfig,
    pub canvas_config: CanvasConfig,
    pub input_config: InputConfig,
    pub history_max_size: usize,
    pub autosave_interval_secs: u64,
    pub show_debug_overlay: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            render_config: RenderConfig::default(),
            canvas_config: CanvasConfig::default(),
            input_config: InputConfig::default(),
            history_max_size: 100,
            autosave_interval_secs: 30,
            show_debug_overlay: false,
        }
    }
}

/// Active document state
pub struct DocumentState {
    pub document: StorageDocument,
    pub canvas: Canvas,
    pub layer_tree: LayerTree,
    pub history: History,
    pub selection: SelectionManager,
    pub brush_engine: BrushEngine,
    pub renderer: Renderer,
    pub input_processor: InputProcessor,
    pub active_stroke: Option<Stroke>,
    pub active_brush: Option<BrushId>,
    pub modified: bool,
    pub last_autosave: std::time::Instant,
}

/// Main application state
pub struct Application {
    config: AppConfig,
    documents: HashMap<DocumentId, DocumentState>,
    active_document: Option<DocumentId>,
    storage: DocumentStorage,
    event_tx: mpsc::UnboundedSender<AppEvent>,
    event_rx: Mutex<Option<mpsc::UnboundedReceiver<AppEvent>>>,
    /// Reserved for custom brush-id generation (not yet wired up)
    #[allow(dead_code)]
    next_brush_id: u64,
}

/// Application events for async handling
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Document events
    DocumentCreated(DocumentId),
    DocumentOpened(DocumentId),
    DocumentClosed(DocumentId),
    DocumentModified(DocumentId),
    DocumentSaved(DocumentId),

    /// Layer events
    LayerAdded(DocumentId, LayerId),
    LayerRemoved(DocumentId, LayerId),
    LayerModified(DocumentId, LayerId),

    /// Object events
    ObjectCreated(DocumentId, ObjectId),
    ObjectModified(DocumentId, ObjectId),
    ObjectDeleted(DocumentId, ObjectId),

    /// Stroke events
    StrokeStarted(DocumentId, ObjectId),
    StrokePointAdded(DocumentId, ObjectId, Point2),
    StrokeEnded(DocumentId, ObjectId),
    StrokeCancelled(DocumentId, ObjectId),

    /// Selection events
    SelectionChanged(DocumentId),

    /// Viewport events
    ViewportChanged(DocumentId),

    /// Tool events
    ToolChanged(DocumentId, Tool),

    /// Autosave
    AutosaveTriggered,

    /// Error
    Error(String),
}

/// Result of command execution with app-level context
#[derive(Debug)]
pub struct AppCommandResult {
    pub command_result: CommandResult,
    pub document_id: DocumentId,
    pub events: Vec<AppEvent>,
}

impl Application {
    /// Create a new application instance
    pub fn new(config: AppConfig) -> Result<Self, AppError> {
        let autosave_dir = std::env::temp_dir().join("creative-engine-autosave");
        let storage = DocumentStorage::new(autosave_dir, config.autosave_interval_secs * 1000);

        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            config,
            documents: HashMap::new(),
            active_document: None,
            storage,
            event_tx,
            event_rx: Mutex::new(Some(event_rx)),
            next_brush_id: 1,
        })
    }

    /// Get the event receiver for the UI to poll
    pub fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<AppEvent>> {
        self.event_rx.lock().unwrap().take()
    }

    /// Get event sender for external components
    pub fn event_sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.event_tx.clone()
    }

    /// Get application configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get mutable application configuration
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Get all document IDs
    pub fn document_ids(&self) -> Vec<DocumentId> {
        self.documents.keys().copied().collect()
    }

    /// Get a document state by ID
    pub fn document(&self, id: DocumentId) -> Option<&DocumentState> {
        self.documents.get(&id)
    }

    /// Get mutable document state by ID
    pub fn document_mut(&mut self, id: DocumentId) -> Option<&mut DocumentState> {
        self.documents.get_mut(&id)
    }

    /// Get active document ID
    pub fn active_document(&self) -> Option<DocumentId> {
        self.active_document
    }

    /// Set active document
    pub fn set_active_document(&mut self, id: DocumentId) -> Result<(), AppError> {
        if self.documents.contains_key(&id) {
            self.active_document = Some(id);
            Ok(())
        } else {
            Err(AppError::DocumentNotFound(id))
        }
    }

    /// Create a new document
    pub fn create_document(&mut self, name: impl Into<String>, settings: Option<DocumentSettings>) -> Result<DocumentId, AppError> {
        let doc_id = DocumentId::new();
        let mut document = StorageDocument {
            meta: creative_domain::DocumentMeta {
                id: doc_id,
                name: name.into(),
                created: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                modified: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                version: 1,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
            },
            settings: settings.unwrap_or_default(),
            camera: Camera::new(),
            layers: Vec::new(),
            objects: HashMap::new(),
            artboards: Vec::new(),
            geometries: HashMap::new(),
            selection: StorageSelectionState::default(),
        };

        // Add root layer
        let root_layer = creative_domain::Layer::new("Root");
        document.layers.push(root_layer.clone());

        // Add a default layer
        let mut default_layer = creative_domain::Layer::new("Layer 1");
        default_layer.parent = Some(root_layer.id);
        document.layers.push(default_layer.clone());

        // Update root layer children
        if let Some(root) = document.layers.iter_mut().find(|l| l.id == root_layer.id) {
            root.children.push(default_layer.id);
        }

        let mut canvas = Canvas::new(doc_id, self.config.canvas_config.clone());
        canvas.set_tool(Tool::Pen);

        let journal_path = std::env::temp_dir().join(format!("creative-engine-history-{}.journal", doc_id.0));
        let history = History::new(doc_id, journal_path, self.config.history_max_size);

        let layer_tree = LayerTree::from_layers(document.layers.clone())?;

        let selection = SelectionManager::new(doc_id);

        let mut brush_engine = BrushEngine::new(StrokeConfig::default());
        // Built-in brushes are registered in BrushEngine::new()

        // Activate the first registered brush by default
        let first_brush = brush_engine.list_presets().first().map(|preset| preset.id);
        if let Some(brush_id) = first_brush {
            let _ = brush_engine.set_brush(brush_id);
        }

        let mut renderer = Renderer::with_backend(Box::new(SoftwareBackend::new()?));
        renderer.initialize(self.config.render_config.clone())?;

        let input_processor = InputProcessor::new(self.config.input_config.clone());

        let doc_state = DocumentState {
            document,
            canvas,
            layer_tree,
            history,
            selection,
            brush_engine,
            renderer,
            input_processor,
            active_stroke: None,
            active_brush: first_brush,
            modified: false,
            last_autosave: std::time::Instant::now(),
        };

        self.documents.insert(doc_id, doc_state);
        self.active_document = Some(doc_id);

        self.emit_event(AppEvent::DocumentCreated(doc_id));

        Ok(doc_id)
    }

    /// Open an existing document
    pub fn open_document(&mut self, path: &std::path::Path) -> Result<DocumentId, AppError> {
        let document = self.storage.load(path)?;
        let doc_id = document.meta.id;

        let canvas = Canvas::new(doc_id, self.config.canvas_config.clone());

        let journal_path = std::env::temp_dir().join(format!("creative-engine-history-{}.journal", doc_id.0));
        let history = History::new(doc_id, journal_path, self.config.history_max_size);

        let layer_tree = LayerTree::from_layers(document.layers.clone())?;

        let selection = SelectionManager::new(doc_id);

        let brush_engine = BrushEngine::new(StrokeConfig::default());

        let mut renderer = Renderer::with_backend(Box::new(SoftwareBackend::new()?));
        renderer.initialize(self.config.render_config.clone())?;

        let input_processor = InputProcessor::new(self.config.input_config.clone());

        let doc_state = DocumentState {
            document,
            canvas,
            layer_tree,
            history,
            selection,
            brush_engine,
            renderer,
            input_processor,
            active_stroke: None,
            active_brush: None,
            modified: false,
            last_autosave: std::time::Instant::now(),
        };

        self.documents.insert(doc_id, doc_state);
        self.active_document = Some(doc_id);

        self.emit_event(AppEvent::DocumentOpened(doc_id));

        Ok(doc_id)
    }

    /// Save a document
    pub fn save_document(&mut self, document_id: DocumentId) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        // Update document from layer tree
        doc_state.document.layers = doc_state.layer_tree.all_layers().into_iter().cloned().collect();

        // Save to default location
        let path = std::env::temp_dir().join(format!("{}.ceng", doc_state.document.meta.name));
        self.storage.save(&path, &doc_state.document)?;

        doc_state.modified = false;

        self.emit_event(AppEvent::DocumentSaved(document_id));

        Ok(())
    }

    /// Save document to specific path
    pub fn save_document_as(&mut self, document_id: DocumentId, path: &std::path::Path) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        // Update document from layer tree
        doc_state.document.layers = doc_state.layer_tree.all_layers().into_iter().cloned().collect();

        self.storage.save(path, &doc_state.document)?;

        doc_state.modified = false;

        self.emit_event(AppEvent::DocumentSaved(document_id));

        Ok(())
    }

    /// Close a document
    pub fn close_document(&mut self, document_id: DocumentId) -> Result<(), AppError> {
        if self.active_document == Some(document_id) {
            self.active_document = None;
        }
        self.documents.remove(&document_id);

        self.emit_event(AppEvent::DocumentClosed(document_id));

        Ok(())
    }

    /// Execute a command on a document
    pub fn execute_command(&mut self, document_id: DocumentId, command: BoxedCommand) -> Result<AppCommandResult, AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        // Execute command through history
        let result = doc_state.history.execute(command)?;

        if result.success {
            doc_state.modified = true;
            self.emit_event(AppEvent::DocumentModified(document_id));
        }

        Ok(AppCommandResult {
            command_result: result,
            document_id,
            events: Vec::new(),
        })
    }

    /// Undo last command
    pub fn undo(&mut self, document_id: DocumentId) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        doc_state.history.undo()?;
        doc_state.modified = true;
        self.emit_event(AppEvent::DocumentModified(document_id));
        Ok(())
    }

    /// Redo last undone command
    pub fn redo(&mut self, document_id: DocumentId) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        doc_state.history.redo()?;
        doc_state.modified = true;
        self.emit_event(AppEvent::DocumentModified(document_id));
        Ok(())
    }

    /// Handle pointer event
    pub fn handle_pointer_event(&mut self, document_id: DocumentId, event: PointerEvent) -> Result<(), AppError> {
        let (input_events, _viewport) = {
            let doc_state = self.documents.get_mut(&document_id)
                .ok_or(AppError::DocumentNotFound(document_id))?;

            // Convert to raw pointer event for input processor
            let raw_event = RawPointerEvent {
                event_type: event.event_type,
                device_type: event.device_type,
                device_id: event.device_id,
                pointer_id: event.pointer_id,
                position: event.position,
                pressure: event.pressure,
                tangential_pressure: event.tangential_pressure,
                tilt_x: event.tilt_x,
                tilt_y: event.tilt_y,
                twist: event.twist,
                altitude_angle: event.altitude_angle,
                azimuth_angle: event.azimuth_angle,
                contact_size: event.contact_size,
                button: event.button,
                buttons: event.buttons,
                modifiers: event.modifiers,
                is_primary: event.is_primary,
            };

            let events = doc_state.input_processor.process_pointer_event(raw_event);

            let viewport = CanvasViewport {
                width: doc_state.renderer.config().width,
                height: doc_state.renderer.config().height,
                scale_factor: doc_state.renderer.config().scale_factor,
            };

            (events, viewport)
        };

        self.process_input_events(document_id, input_events)
    }

    /// Handle a keyboard event through the input pipeline
    pub fn handle_key_event(&mut self, document_id: DocumentId, raw_event: RawKeyEvent) -> Result<(), AppError> {
        let input_events = {
            let doc_state = self.documents.get_mut(&document_id)
                .ok_or(AppError::DocumentNotFound(document_id))?;

            doc_state.input_processor.process_key_event(raw_event)
        };

        self.process_input_events(document_id, input_events)
    }

    /// Process normalized input events with scoped document borrows
    fn process_input_events(&mut self, document_id: DocumentId, input_events: Vec<creative_input::InputEvent>) -> Result<(), AppError> {
        // Process each input event with its own scoped document borrow
        let mut pending_events: Vec<AppEvent> = Vec::new();
        let mut pending_save = false;

        for input_event in input_events {
            let mut doc_state = self.documents.get_mut(&document_id)
                .ok_or(AppError::DocumentNotFound(document_id))?;

            // Viewport from the renderer configuration
            let viewport = CanvasViewport {
                width: doc_state.renderer.config().width,
                height: doc_state.renderer.config().height,
                scale_factor: doc_state.renderer.config().scale_factor,
            };

            match input_event {
                creative_input::InputEvent::Pointer(ptr_event) => {
                    pending_events.extend(Self::handle_pointer_input(&mut doc_state, ptr_event, viewport)?);
                }
                creative_input::InputEvent::Gesture(gesture) => {
                    pending_events.extend(Self::handle_gesture(&mut doc_state, gesture, viewport)?);
                }
                creative_input::InputEvent::Key(key_event) => {
                    let (events, save) = Self::handle_key(&mut doc_state, key_event)?;
                    pending_events.extend(events);
                    pending_save |= save;
                }
                creative_input::InputEvent::Text(text) => {
                    Self::handle_text(&mut doc_state, text)?;
                }
                creative_input::InputEvent::Focus(_focused) => {
                    // Handle focus change
                }
            }
        }

        // Perform actions that require the full application state
        if pending_save {
            self.save_document(document_id)?;
        }

        // Emit events after all document borrows are released
        for app_event in pending_events {
            self.emit_event(app_event);
        }

        Ok(())
    }

    /// Handle a normalized pointer input event against the active tool
    fn handle_pointer_input(doc_state: &mut DocumentState, event: creative_input::PointerEvent, viewport: CanvasViewport) -> Result<Vec<AppEvent>, AppError> {
        let document_id = doc_state.document.meta.id;
        let mut events = Vec::new();

        match event.event_type {
            creative_input::PointerEventType::Down => {
                doc_state.canvas.tool_mouse_down(event.position, viewport);
            }
            creative_input::PointerEventType::Move => {
                doc_state.canvas.tool_mouse_move(event.position, viewport);
            }
            creative_input::PointerEventType::Up => {
                if let Some(action) = doc_state.canvas.tool_mouse_up(event.position, viewport) {
                    match action {
                        creative_canvas::ToolAction::None
                        | creative_canvas::ToolAction::Pan
                        | creative_canvas::ToolAction::Zoom
                        | creative_canvas::ToolAction::Rotate => {}

                        creative_canvas::ToolAction::SelectSingle => {
                            // The canvas selection was applied during tool_mouse_down;
                            // sync it into the selection manager
                            let ids = doc_state.canvas.selected_objects().to_vec();
                            doc_state.selection.select_objects(ids, false);
                            events.push(AppEvent::SelectionChanged(document_id));
                        }

                        creative_canvas::ToolAction::SelectRect(rect) => {
                            let ids = doc_state.canvas.query_rect(rect);
                            doc_state.canvas.deselect_all();
                            for id in &ids {
                                doc_state.canvas.select_object(*id, true);
                            }
                            doc_state.selection.select_objects(ids, false);
                            events.push(AppEvent::SelectionChanged(document_id));
                        }

                        creative_canvas::ToolAction::DrawStroke { points } => {
                            let mut path = PathGeometry::new();
                            let mut points_iter = points.iter();
                            if let Some(first) = points_iter.next() {
                                path.segments.push(PathSegment::MoveTo(*first));
                            }
                            for point in points_iter {
                                path.segments.push(PathSegment::LineTo(*point));
                            }

                            // Use the active brush style when available
                            let (width, color) = doc_state.active_brush
                                .and_then(|id| doc_state.brush_engine.get_preset(id))
                                .map(|preset| (preset.base_size, preset.color))
                                .unwrap_or((2.0, Color::BLACK));

                            path.stroke = Some(StrokeStyle {
                                width,
                                color,
                                ..StrokeStyle::default()
                            });

                            let (layer_id, layer_events) = Self::get_active_layer(doc_state)?;
                            events.extend(layer_events);

                            let object_id = ObjectId::new();
                            doc_state.canvas.add_geometry(object_id, Geometry::Path(path));
                            doc_state.layer_tree.add_object(layer_id, object_id)?;
                            doc_state.modified = true;

                            events.push(AppEvent::ObjectCreated(document_id, object_id));
                            events.push(AppEvent::DocumentModified(document_id));
                        }

                        creative_canvas::ToolAction::CreateShape { shape_type, rect } => {
                            let geometry = Self::create_shape_geometry(shape_type, rect);

                            let (layer_id, layer_events) = Self::get_active_layer(doc_state)?;
                            events.extend(layer_events);

                            let object_id = ObjectId::new();
                            doc_state.canvas.add_geometry(object_id, geometry);
                            doc_state.layer_tree.add_object(layer_id, object_id)?;
                            doc_state.modified = true;

                            events.push(AppEvent::ObjectCreated(document_id, object_id));
                            events.push(AppEvent::DocumentModified(document_id));
                        }

                        creative_canvas::ToolAction::CreateMeasurement { start: _, end: _ } => {
                            // Measurement creation is handled by the precision subsystem
                        }
                    }
                }
            }
            creative_input::PointerEventType::Enter
            | creative_input::PointerEventType::Leave
            | creative_input::PointerEventType::Cancel => {}
        }

        Ok(events)
    }

    /// Handle gesture input
    fn handle_gesture(doc_state: &mut DocumentState, gesture: creative_input::GestureEvent, viewport: CanvasViewport) -> Result<Vec<AppEvent>, AppError> {
        let document_id = doc_state.document.meta.id;
        match gesture {
            creative_input::GestureEvent::Pan { delta, velocity: _ } => {
                // A single-pointer drag must not pan when a tool operation is
                // in progress (e.g. a pen stroke would have every recorded
                // point collapse onto the same world position, because the
                // camera keeps the cursor over the same point; the Pan tool
                // also pans through its own pointer path and would double up).
                // Pan gestures therefore only apply while no drag is active.
                let drag_in_progress = !matches!(
                    doc_state.canvas.state().tool_state,
                    creative_canvas::ToolState::Idle
                );

                if !drag_in_progress {
                    doc_state.canvas.pan(delta, viewport);
                }
            }
            creative_input::GestureEvent::Zoom { factor, center } => {
                doc_state.canvas.zoom_at(factor, center, viewport);
            }
            creative_input::GestureEvent::Rotate { angle, center } => {
                doc_state.canvas.rotate_at(angle, center, viewport);
            }
            creative_input::GestureEvent::Tap { count: _, position: _ } => {
                // Tap handled by pointer events
            }
            creative_input::GestureEvent::LongPress { position: _ } => {
                // Long press handled by pointer events
            }
            creative_input::GestureEvent::Swipe { velocity: _, direction: _ } => {
                // Could implement swipe navigation
            }
        }
        Ok(vec![AppEvent::ViewportChanged(document_id)])
    }

    /// Handle key input
    fn handle_key(doc_state: &mut DocumentState, event: creative_input::KeyEvent) -> Result<(Vec<AppEvent>, bool), AppError> {
        let document_id = doc_state.document.meta.id;
        let mut events = Vec::new();
        let mut save = false;

        // Handle keyboard shortcuts
        if event.state == creative_input::KeyState::Pressed {
            match event.key {
                creative_input::Key::Character('z') if event.modifiers.ctrl && !event.modifiers.shift => {
                    doc_state.history.undo()?;
                    doc_state.modified = true;
                    events.push(AppEvent::DocumentModified(document_id));
                }
                creative_input::Key::Character('z') if event.modifiers.ctrl && event.modifiers.shift => {
                    doc_state.history.redo()?;
                    doc_state.modified = true;
                    events.push(AppEvent::DocumentModified(document_id));
                }
                creative_input::Key::Character('s') if event.modifiers.ctrl => {
                    save = true;
                }
                creative_input::Key::Character('a') if event.modifiers.ctrl => {
                    let geometries: HashMap<ObjectId, (BoundingBox, Transform2D)> = doc_state.canvas.geometries()
                        .iter()
                        .map(|(id, geom)| {
                            let bbox = geom.bounding_box();
                            let geometry_bbox = BoundingBox::new(bbox.min, bbox.max);
                            (*id, (geometry_bbox, geom.transform()))
                        })
                        .collect();
                    doc_state.selection.select_all(&geometries);
                    for id in doc_state.selection.selected_ids() {
                        doc_state.canvas.select_object(id, true);
                    }
                    events.push(AppEvent::SelectionChanged(document_id));
                }
                creative_input::Key::Escape => {
                    doc_state.canvas.deselect_all();
                    doc_state.selection.deselect_all();
                    events.push(AppEvent::SelectionChanged(document_id));
                }
                creative_input::Key::Delete => {
                    // Delete selected objects
                    let selected = doc_state.selection.selected_ids();
                    for obj_id in selected {
                        doc_state.canvas.remove_geometry(obj_id);
                        // Remove from layer tree
                        for layer_id in doc_state.layer_tree.render_order() {
                            if let Some(layer) = doc_state.layer_tree.get_layer(*layer_id) {
                                if layer.object_ids.contains(&obj_id) {
                                    doc_state.layer_tree.remove_object(*layer_id, obj_id)?;
                                    break;
                                }
                            }
                        }
                        doc_state.modified = true;
                        events.push(AppEvent::ObjectDeleted(document_id, obj_id));
                    }
                    events.push(AppEvent::DocumentModified(document_id));
                }
                _ => {}
            }
        }
        Ok((events, save))
    }

    /// Handle text input
    fn handle_text(_doc_state: &mut DocumentState, _text: String) -> Result<(), AppError> {
        // Text input for text tool
        Ok(())
    }

    /// Create shape geometry
    fn create_shape_geometry(shape_type: creative_canvas::ShapeTool, rect: Rect) -> Geometry {
        match shape_type {
            creative_canvas::ShapeTool::Rectangle => {
                let shape = ShapeGeometry::new(ShapeType::Rectangle, rect);
                Geometry::Shape(shape)
            }
            creative_canvas::ShapeTool::Ellipse => {
                let shape = ShapeGeometry::new(ShapeType::Ellipse, rect);
                Geometry::Shape(shape)
            }
            creative_canvas::ShapeTool::Line => {
                let shape = ShapeGeometry::new(ShapeType::Line, rect);
                Geometry::Shape(shape)
            }
            creative_canvas::ShapeTool::Polygon => {
                let shape = ShapeGeometry::new(ShapeType::Polygon { sides: 6 }, rect);
                Geometry::Shape(shape)
            }
            creative_canvas::ShapeTool::Path => {
                let mut path = PathGeometry::new();
                path.segments.push(PathSegment::MoveTo(rect.min));
                path.segments.push(PathSegment::LineTo(Point2::new(rect.max.x, rect.min.y)));
                path.segments.push(PathSegment::LineTo(rect.max));
                path.segments.push(PathSegment::LineTo(Point2::new(rect.min.x, rect.max.y)));
                path.segments.push(PathSegment::ClosePath);
                Geometry::Path(path)
            }
        }
    }

    /// Get active layer ID (create one if none exists)
    fn get_active_layer(doc_state: &mut DocumentState) -> Result<(LayerId, Vec<AppEvent>), AppError> {
        let document_id = doc_state.document.meta.id;
        let root = doc_state.layer_tree.root_id();
        let children = doc_state.layer_tree.children(root);

        if let Some(layer_id) = children.first() {
            Ok((*layer_id, Vec::new()))
        } else {
            // Create a default layer
            let layer_id = doc_state.layer_tree.add_layer(root, "Layer 1")?;
            Ok((layer_id, vec![AppEvent::LayerAdded(document_id, layer_id)]))
        }
    }

    /// Check and trigger autosave
    pub fn check_autosave(&mut self, document_id: DocumentId) -> Result<(), AppError> {
        let should_autosave = {
            let doc_state = self.documents.get(&document_id)
                .ok_or(AppError::DocumentNotFound(document_id))?;

            doc_state.modified && doc_state.last_autosave.elapsed().as_secs() >= self.config.autosave_interval_secs
        };

        if should_autosave {
            // Trigger autosave in background
            self.emit_event(AppEvent::AutosaveTriggered);

            if let Some(doc_state) = self.documents.get_mut(&document_id) {
                doc_state.last_autosave = std::time::Instant::now();
            }
        }

        Ok(())
    }

    /// Render a document
    pub fn render(&mut self, document_id: DocumentId) -> Result<creative_renderer::RenderStats, AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        // Build render items from canvas geometries
        let mut render_items = Vec::new();

        // Get layer render info with effective properties
        let render_layers = doc_state.layer_tree.render_layers();

        // Build a map from object_id to layer info for quick lookup
        let mut object_to_layer = HashMap::new();
        for layer in &render_layers {
            for obj_id in &layer.object_ids {
                object_to_layer.insert(*obj_id, (layer.visible, layer.opacity, layer.blend_mode, layer.world_transform, layer.locked));
            }
        }

        for (object_id, geometry) in doc_state.canvas.geometries() {
            // Check if object is on a visible layer
            let (visible, opacity, blend_mode, world_transform, _locked) = object_to_layer.get(object_id)
                .copied()
                .unwrap_or((true, 1.0, BlendMode::Normal, Transform2D::IDENTITY, false));

            if !visible {
                continue;
            }

            // Determine render layer
            let render_layer = RenderLayerType::DocumentContent;

            // Get fill and stroke from geometry if available
            let (fill, stroke) = match geometry {
                creative_geometry::Geometry::Shape(s) => (s.fill, s.stroke.clone()),
                creative_geometry::Geometry::Path(p) => (p.fill, p.stroke.clone()),
                _ => (None, None),
            };

            // Convert BoundingBox from creative_geometry
            let bbox = geometry.bounding_box();
            let geometry_bbox = BoundingBox::new(bbox.min, bbox.max);

            // Create render item
            let item = RenderItem {
                object_id: *object_id,
                geometry_id: geometry.id(),
                layer: render_layer,
                z_index: 0,
                bounding_box: geometry_bbox,
                transform: world_transform,
                fill,
                stroke,
                opacity,
                blend_mode,
                clip_rect: None,
                is_selected: doc_state.selection.is_selected(*object_id),
                is_hovered: doc_state.canvas.state().hovered_object == Some(*object_id),
            };

            render_items.push(item);
        }

        // Create viewport for renderer
        let viewport = Viewport::new(
            doc_state.renderer.config().width,
            doc_state.renderer.config().height,
            doc_state.renderer.config().scale_factor,
            doc_state.canvas.camera().clone(),
        );

        // Render using the renderer
        let stats = doc_state.renderer.render(&viewport, &render_items, None)?;

        Ok(stats)
    }

    /// Resize renderer viewport
    pub fn resize(&mut self, document_id: DocumentId, width: u32, height: u32, scale_factor: f32) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        doc_state.renderer.resize(width, height, scale_factor)?;

        self.emit_event(AppEvent::ViewportChanged(document_id));
        Ok(())
    }

    /// Set active tool
    pub fn set_tool(&mut self, document_id: DocumentId, tool: Tool) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        doc_state.canvas.state_mut().active_tool = tool;
        doc_state.canvas.state_mut().tool_state = ToolState::Idle;

        self.emit_event(AppEvent::ToolChanged(document_id, tool));
        Ok(())
    }

    /// Set active brush
    pub fn set_brush(&mut self, document_id: DocumentId, brush_id: BrushId) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        doc_state.brush_engine.set_brush(brush_id)?;
        doc_state.active_brush = Some(brush_id);
        Ok(())
    }

    /// Update the size of the active brush
    pub fn set_brush_size(&mut self, document_id: DocumentId, size: f32) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        if let Some(brush_id) = doc_state.active_brush {
            if let Some(preset) = doc_state.brush_engine.get_preset_mut(brush_id) {
                preset.base_size = size;
            }
        }
        Ok(())
    }

    /// Update the colour of the active brush
    pub fn set_brush_color(&mut self, document_id: DocumentId, color: Color) -> Result<(), AppError> {
        let doc_state = self.documents.get_mut(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        if let Some(brush_id) = doc_state.active_brush {
            if let Some(preset) = doc_state.brush_engine.get_preset_mut(brush_id) {
                preset.color = color;
            }
        }
        Ok(())
    }

    /// Get list of available brushes
    pub fn list_brushes(&self, document_id: DocumentId) -> Result<Vec<&BrushPreset>, AppError> {
        let doc_state = self.documents.get(&document_id)
            .ok_or(AppError::DocumentNotFound(document_id))?;

        Ok(doc_state.brush_engine.list_presets())
    }

    /// Update application (call regularly for autosave, etc.)
    pub fn update(&mut self) -> Result<(), AppError> {
        // Check autosave for all modified documents
        let doc_ids: Vec<_> = self.documents.keys().copied().collect();
        for doc_id in doc_ids {
            if let Some(doc_state) = self.documents.get(&doc_id) {
                if doc_state.modified && doc_state.last_autosave.elapsed().as_secs() >= self.config.autosave_interval_secs {
                    // Try to autosave
                    if let Err(e) = self.save_document(doc_id) {
                        warn!("Autosave failed for document {}: {}", doc_id, e);
                        self.emit_event(AppEvent::Error(format!("Autosave failed: {}", e)));
                    }
                }
            }
        }

        Ok(())
    }

    /// Emit an application event
    fn emit_event(&self, event: AppEvent) {
        let _ = self.event_tx.send(event);
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new(AppConfig::default()).expect("Failed to create default application")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_creation() {
        let app = Application::new(AppConfig::default());
        assert!(app.is_ok());
    }

    #[test]
    fn test_create_document() {
        let mut app = Application::new(AppConfig::default()).unwrap();
        let doc_id = app.create_document("Test Document", None).unwrap();

        assert!(app.document(doc_id).is_some());
        assert_eq!(app.active_document(), Some(doc_id));
    }

    #[test]
    fn test_multiple_documents() {
        let mut app = Application::new(AppConfig::default()).unwrap();
        let doc1 = app.create_document("Doc 1", None).unwrap();
        let doc2 = app.create_document("Doc 2", None).unwrap();

        assert_eq!(app.document_ids().len(), 2);
        assert!(app.document(doc1).is_some());
        assert!(app.document(doc2).is_some());
    }

    #[test]
    fn test_switch_active_document() {
        let mut app = Application::new(AppConfig::default()).unwrap();
        let _doc1 = app.create_document("Doc 1", None).unwrap();
        let doc2 = app.create_document("Doc 2", None).unwrap();

        app.set_active_document(doc2).unwrap();
        assert_eq!(app.active_document(), Some(doc2));
    }
}
