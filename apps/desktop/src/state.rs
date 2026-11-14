//! Application state for the Creative Engine desktop shell.

use creative_application::{AppConfig, Application};
use creative_canvas::Tool;
use creative_domain::{Color, DocumentId};

/// The desktop application state.
///
/// Wraps the shared engine (`creative_application::Application`) together
/// with the workspace UI state (active tool, brush, panels).
pub struct AppState {
    /// The engine: document model, canvas, tools, history, renderer
    pub engine: Application,
    /// The document being edited (one at a time in the workspace)
    pub document: Option<DocumentId>,
    /// Currently active tool (mirrors the canvas state for highlighting)
    pub active_tool: Tool,
    /// Active brush size (world units)
    pub brush_size: f32,
    /// Active brush colour
    pub brush_color: Color,
    /// Whether the layers panel is open
    pub layers_panel_open: bool,
    /// Size of the canvas area in logical pixels
    pub canvas_size: (f32, f32),
    /// Last status message shown in the title bar
    pub status: String,
}

impl AppState {
    /// Create the initial application state
    pub fn new() -> Self {
        let mut engine = Application::new(AppConfig::default())
            .expect("failed to initialize the creative engine");

        let document = engine
            .create_document("Untitled", None)
            .expect("failed to create the initial document");

        // Match the default window size declared in main.rs
        let _ = engine.resize(document, 1280, 800, 1.0);

        let doc_state = engine
            .document(document)
            .expect("document just created");

        let active_tool = doc_state.canvas.active_tool();
        let brush_size = doc_state
            .active_brush
            .and_then(|id| doc_state.brush_engine.get_preset(id))
            .map(|preset| preset.base_size)
            .unwrap_or(2.0);
        let brush_color = doc_state
            .active_brush
            .and_then(|id| doc_state.brush_engine.get_preset(id))
            .map(|preset| preset.color)
            .unwrap_or(Color::BLACK);

        Self {
            engine,
            document: Some(document),
            active_tool,
            brush_size,
            brush_color,
            layers_panel_open: true,
            canvas_size: (1280.0, 800.0),
            status: String::from("Ready"),
        }
    }
}
