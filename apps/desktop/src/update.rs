//! Message handling (update) for the Creative Engine desktop shell.

use creative_canvas::{ShapeTool, Tool};
use creative_domain::Point2;
use creative_input::{
    InputDeviceType, Modifiers, PointerButton, PointerEvent, PointerEventType, RawKeyEvent,
    KeyState,
};

use crate::message::{Message, PointerMsg};
use crate::state::AppState;

/// Update the application state in response to a message
pub fn update(state: &mut AppState, message: Message) {
    let Some(document) = state.document else {
        return;
    };

    match message {
        Message::Pointer(pointer) => {
            sync_viewport(state, document);

            let event = pointer_event(pointer);
            if let Err(e) = state.engine.handle_pointer_event(document, event) {
                state.status = format!("Error: {e}");
            }
        }
        Message::ViewportResized { width, height } => {
            state.canvas_size = (width, height);
            let _ = state.engine.resize(document, width as u32, height as u32, 1.0);
        }
        Message::Tool(tool) => {
            state.active_tool = tool;
            let _ = state.engine.set_tool(document, tool);
        }
        Message::Color(color) => {
            state.brush_color = color;
            let _ = state.engine.set_brush_color(document, color);
        }
        Message::BrushSize(size) => {
            state.brush_size = size;
            let _ = state.engine.set_brush_size(document, size);
        }
        Message::Undo => {
            if let Err(e) = state.engine.undo(document) {
                state.status = format!("Undo: {e}");
            }
        }
        Message::Redo => {
            if let Err(e) = state.engine.redo(document) {
                state.status = format!("Redo: {e}");
            }
        }
        Message::Save => match state.engine.save_document(document) {
            Ok(()) => state.status = String::from("Saved"),
            Err(e) => state.status = format!("Save failed: {e}"),
        },
        Message::ZoomIn => zoom(state, document, 1.25),
        Message::ZoomOut => zoom(state, document, 0.8),
        Message::ResetView => {
            if let Some(doc_state) = state.engine.document_mut(document) {
                doc_state.canvas.reset_view();
            }
        }
        Message::FitAll => {
            let vp = viewport(state);
            if let Some(doc_state) = state.engine.document_mut(document) {
                doc_state.canvas.fit_all(vp);
            }
        }
        Message::ToggleLayersPanel => {
            state.layers_panel_open = !state.layers_panel_open;
        }
        Message::ToggleLayerVisible(layer_id) => {
            if let Some(doc_state) = state.engine.document_mut(document) {
                let visible = doc_state
                    .layer_tree
                    .get_layer(layer_id)
                    .map(|layer| layer.visible)
                    .unwrap_or(true);
                let _ = doc_state
                    .layer_tree
                    .set_visibility(layer_id, !visible);
            }
        }
        Message::AddLayer => {
            if let Some(doc_state) = state.engine.document_mut(document) {
                let root = doc_state.layer_tree.root_id();
                let name = format!("Layer {}", doc_state.layer_tree.layer_count());
                if let Err(e) = doc_state.layer_tree.add_layer(root, name) {
                    state.status = format!("Add layer failed: {e}");
                }
            }
        }
        Message::Key {
            key,
            pressed,
            ctrl,
            shift,
            alt,
            meta,
        } => {
            let raw = RawKeyEvent {
                key,
                physical_key: None,
                state: if pressed {
                    KeyState::Pressed
                } else {
                    KeyState::Released
                },
                modifiers: Modifiers {
                    shift,
                    ctrl,
                    alt,
                    meta,
                    super_key: false,
                },
                repeat: false,
            };

            if let Err(e) = state.engine.handle_key_event(document, raw) {
                state.status = format!("Key: {e}");
            }
        }
    }
}

/// Keep the engine renderer viewport in sync with the canvas widget size
fn sync_viewport(state: &mut AppState, document: creative_domain::DocumentId) {
    let config_size = state
        .engine
        .document(document)
        .map(|doc_state| {
            let config = doc_state.renderer.config();
            (config.width, config.height)
        });

    if let Some((width, height)) = config_size {
        let (target_w, target_h) = (state.canvas_size.0 as u32, state.canvas_size.1 as u32);
        if width != target_w || height != target_h {
            let _ = state.engine.resize(document, target_w, target_h, 1.0);
        }
    }
}

/// The canvas viewport used for camera math
pub fn viewport(state: &AppState) -> creative_canvas::Viewport {
    creative_canvas::Viewport {
        width: state.canvas_size.0 as u32,
        height: state.canvas_size.1 as u32,
        scale_factor: 1.0,
    }
}

/// Zoom around the centre of the canvas
fn zoom(state: &mut AppState, document: creative_domain::DocumentId, factor: f64) {
    let vp = viewport(state);
    if let Some(doc_state) = state.engine.document_mut(document) {
        let center = Point2::new(vp.width as f64 / 2.0, vp.height as f64 / 2.0);
        doc_state.canvas.zoom_at(factor, center, vp);
    }
}

/// Translate a pointer message from the canvas widget into an engine event
fn pointer_event(msg: PointerMsg) -> PointerEvent {
    let (event_type, position, button, pressure) = match msg {
        PointerMsg::Down(position) => (
            PointerEventType::Down,
            position,
            Some(PointerButton::Left),
            1.0,
        ),
        PointerMsg::Move(position) => (PointerEventType::Move, position, None, 0.5),
        PointerMsg::Up(position) => (
            PointerEventType::Up,
            position,
            Some(PointerButton::Left),
            0.0,
        ),
    };

    PointerEvent {
        event_type,
        device_type: InputDeviceType::Mouse,
        device_id: 0,
        pointer_id: 0,
        position: Point2::new(position.x as f64, position.y as f64),
        pressure,
        tangential_pressure: 0.0,
        tilt_x: 0.0,
        tilt_y: 0.0,
        twist: 0.0,
        altitude_angle: 0.0,
        azimuth_angle: 0.0,
        contact_size: Point2::new(1.0, 1.0),
        button,
        buttons: 0,
        modifiers: Modifiers::default(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0),
        is_primary: true,
        is_coalesced: false,
        predicted_points: Vec::new(),
    }
}

/// Tools offered in the floating toolbar: (label, tool)
pub const TOOLBAR: &[(&str, Tool)] = &[
    ("Select", Tool::Select),
    ("Pan", Tool::Pan),
    ("Pen", Tool::Pen),
    ("Pencil", Tool::Pencil),
    ("Brush", Tool::Brush),
    ("Eraser", Tool::Eraser),
    ("Rect", Tool::Shape(ShapeTool::Rectangle)),
    ("Ellipse", Tool::Shape(ShapeTool::Ellipse)),
    ("Line", Tool::Shape(ShapeTool::Line)),
    ("Measure", Tool::Measure),
];

/// Colour swatches shown in the bottom bar
pub const SWATCHES: &[creative_domain::Color] = &[
    creative_domain::Color {
        r: 0.10,
        g: 0.10,
        b: 0.12,
        a: 1.0,
    },
    creative_domain::Color::WHITE,
    creative_domain::Color {
        r: 0.86,
        g: 0.20,
        b: 0.20,
        a: 1.0,
    },
    creative_domain::Color {
        r: 0.96,
        g: 0.56,
        b: 0.10,
        a: 1.0,
    },
    creative_domain::Color {
        r: 0.94,
        g: 0.80,
        b: 0.10,
        a: 1.0,
    },
    creative_domain::Color {
        r: 0.20,
        g: 0.72,
        b: 0.32,
        a: 1.0,
    },
    creative_domain::Color {
        r: 0.20,
        g: 0.48,
        b: 0.90,
        a: 1.0,
    },
    creative_domain::Color {
        r: 0.58,
        g: 0.32,
        b: 0.88,
        a: 1.0,
    },
];
