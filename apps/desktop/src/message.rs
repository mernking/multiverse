//! Messages for the Creative Engine desktop application.

use creative_canvas::Tool;
use creative_domain::{Color, LayerId, Point2};

/// Pointer interaction forwarded from the canvas widget
#[derive(Debug, Clone, Copy)]
pub enum PointerMsg {
    Down(Point2),
    Move(Point2),
    Up(Point2),
}

/// Application message
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Pointer(PointerMsg),
    ViewportResized {
        width: f32,
        height: f32,
    },
    /// Select a tool
    Tool(Tool),
    /// Pick a drawing colour
    Color(Color),
    /// Change the active brush size
    BrushSize(f32),
    Undo,
    Redo,
    Save,
    ZoomIn,
    ZoomOut,
    ResetView,
    FitAll,
    ToggleLayersPanel,
    ToggleLayerVisible(LayerId),
    AddLayer,
    /// A normalized keyboard event forwarded to the engine
    Key {
        key: creative_input::Key,
        pressed: bool,
        ctrl: bool,
        shift: bool,
        alt: bool,
        meta: bool,
    },
}
