//! Input handling for Creative Engine
//!
//! Provides normalized pointer/stylus event handling with support for
//! mouse, touch, and stylus input with pressure, tilt, and azimuth.

use creative_domain::{Point2, Vector2};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;

/// Input errors
#[derive(Debug, Error)]
pub enum InputError {
    #[error("Invalid input state: {0}")]
    InvalidState(String),
    #[error("Unsupported input type: {0}")]
    Unsupported(String),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
}

/// Input device types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputDeviceType {
    Mouse,
    Touch,
    Stylus,
    Eraser,
    Keyboard,
}

/// Pointer button identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

/// Pointer event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerEventType {
    Down,
    Up,
    Move,
    Enter,
    Leave,
    Cancel,
}

/// Modifier key state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
    pub super_key: bool,
}

/// Normalized pointer event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerEvent {
    pub event_type: PointerEventType,
    pub device_type: InputDeviceType,
    pub device_id: u64,
    pub pointer_id: u64,
    pub position: Point2,
    pub pressure: f32,
    pub tangential_pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub twist: f32,
    pub altitude_angle: f32,
    pub azimuth_angle: f32,
    pub contact_size: Point2,
    pub button: Option<PointerButton>,
    pub buttons: u32, // Bitmask of pressed buttons
    pub modifiers: Modifiers,
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub is_primary: bool,
    pub is_coalesced: bool,
    pub predicted_points: Vec<PredictedPoint>,
}

/// Predicted point for latency compensation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedPoint {
    pub position: Point2,
    pub timestamp: u64, // Unix timestamp in milliseconds
    pub confidence: f32,
}

/// Gesture event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GestureEvent {
    Pan { delta: Vector2, velocity: Vector2 },
    Zoom { factor: f64, center: Point2 },
    Rotate { angle: f64, center: Point2 },
    Tap { count: u32, position: Point2 },
    LongPress { position: Point2 },
    Swipe { velocity: Vector2, direction: SwipeDirection },
}

/// Swipe direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwipeDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Input state for tracking
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub pointers: HashMap<u64, PointerState>,
    pub modifiers: Modifiers,
    pub hovered_element: Option<u64>,
    pub focused_element: Option<u64>,
    pub captured_pointer: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PointerState {
    pub device_type: InputDeviceType,
    pub position: Point2,
    pub pressure: f32,
    pub buttons: u32,
    pub last_position: Point2,
    pub last_timestamp: u64, // Unix timestamp in milliseconds
    pub is_down: bool,
    pub click_count: u32,
    pub last_click_time: u64, // Unix timestamp in milliseconds
    pub last_click_position: Point2,
}

/// Input processor for normalizing and managing input events
pub struct InputProcessor {
    state: InputState,
    config: InputConfig,
    gesture_recognizer: GestureRecognizer,
    event_queue: Vec<InputEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    Pointer(PointerEvent),
    Gesture(GestureEvent),
    Key(KeyEvent),
    Text(String),
    Focus(bool),
}

/// Keyboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: Key,
    pub physical_key: Option<String>,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub repeat: bool,
    pub timestamp: u64, // Unix timestamp in milliseconds
}

/// Key identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    Character(char),
    Enter,
    Tab,
    Space,
    Escape,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Control,
    Shift,
    Alt,
    Meta,
    Super,
    CapsLock,
    NumLock,
    ScrollLock,
    PrintScreen,
    Pause,
    Other(u32),
}

/// Key state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub double_click_time_ms: u64,
    pub double_click_distance: f64,
    pub long_press_time_ms: u64,
    pub long_press_distance: f64,
    pub swipe_min_velocity: f64,
    pub swipe_min_distance: f64,
    pub pan_threshold: f64,
    pub zoom_threshold: f64,
    pub rotate_threshold: f64,
    pub stylus_priority: bool, // Stylus takes precedence over touch
    pub palm_rejection: bool,
    pub prediction_enabled: bool,
    pub coalescing_enabled: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            double_click_time_ms: 300,
            double_click_distance: 5.0,
            long_press_time_ms: 500,
            long_press_distance: 10.0,
            swipe_min_velocity: 500.0,
            swipe_min_distance: 50.0,
            pan_threshold: 3.0,
            zoom_threshold: 0.01,
            rotate_threshold: 0.01,
            stylus_priority: true,
            palm_rejection: true,
            prediction_enabled: true,
            coalescing_enabled: true,
        }
    }
}

/// Gesture recognizer
pub struct GestureRecognizer {
    config: InputConfig,
    pan_state: Option<PanState>,
    zoom_state: Option<ZoomState>,
    rotate_state: Option<RotateState>,
    tap_state: TapState,
    long_press_timer: Option<u64>, // Unix timestamp in milliseconds
}

#[derive(Debug, Clone)]
struct PanState {
    start_position: Point2,
    last_position: Point2,
    start_time: u64, // Unix timestamp in milliseconds
    last_timestamp: u64, // Unix timestamp in milliseconds
    is_panning: bool,
}

#[derive(Debug, Clone)]
struct ZoomState {
    start_distance: f64,
    start_center: Point2,
    start_time: u64, // Unix timestamp in milliseconds
}

#[derive(Debug, Clone)]
struct RotateState {
    start_angle: f64,
    start_center: Point2,
    start_time: u64, // Unix timestamp in milliseconds
}

#[derive(Debug, Clone, Default)]
struct TapState {
    last_tap_time: Option<u64>, // Unix timestamp in milliseconds
    last_tap_position: Option<Point2>,
    tap_count: u32,
}

impl InputProcessor {
    pub fn new(config: InputConfig) -> Self {
        Self {
            state: InputState::default(),
            gesture_recognizer: GestureRecognizer::new(config.clone()),
            config,
            event_queue: Vec::new(),
        }
    }
    
    pub fn process_pointer_event(&mut self, raw_event: RawPointerEvent) -> Vec<InputEvent> {
        let mut events = Vec::new();
        
        // Normalize the event
        let pointer_event = self.normalize_pointer_event(raw_event);
        
        // Update internal state
        self.update_pointer_state(&pointer_event);
        
        // Recognize gestures
        let gesture_events = self.gesture_recognizer.process(&pointer_event, &self.state);
        events.extend(gesture_events.into_iter().map(InputEvent::Gesture));
        
        // Add pointer event
        events.push(InputEvent::Pointer(pointer_event));
        
        events
    }
    
    pub fn process_key_event(&mut self, raw_event: RawKeyEvent) -> Vec<InputEvent> {
        let mut events = Vec::new();
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let key_event = KeyEvent {
            key: raw_event.key,
            physical_key: raw_event.physical_key,
            state: raw_event.state,
            modifiers: raw_event.modifiers,
            repeat: raw_event.repeat,
            timestamp: now,
        };
        
        // Update modifier state
        self.update_modifiers(&key_event);
        
        events.push(InputEvent::Key(key_event));
        
        events
    }
    
    pub fn process_text_input(&mut self, text: String) -> Vec<InputEvent> {
        vec![InputEvent::Text(text)]
    }
    
    pub fn process_focus_change(&mut self, focused: bool) -> Vec<InputEvent> {
        vec![InputEvent::Focus(focused)]
    }
    
    fn normalize_pointer_event(&self, raw: RawPointerEvent) -> PointerEvent {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        PointerEvent {
            event_type: raw.event_type,
            device_type: raw.device_type,
            device_id: raw.device_id,
            pointer_id: raw.pointer_id,
            position: raw.position,
            pressure: raw.pressure.clamp(0.0, 1.0),
            tangential_pressure: raw.tangential_pressure.clamp(-1.0, 1.0),
            tilt_x: raw.tilt_x.clamp(-1.0, 1.0),
            tilt_y: raw.tilt_y.clamp(-1.0, 1.0),
            twist: raw.twist,
            altitude_angle: raw.altitude_angle.clamp(0.0, std::f32::consts::PI / 2.0),
            azimuth_angle: raw.azimuth_angle,
            contact_size: raw.contact_size,
            button: raw.button,
            buttons: raw.buttons,
            modifiers: raw.modifiers,
            timestamp: now,
            is_primary: raw.is_primary,
            is_coalesced: false,
            predicted_points: Vec::new(),
        }
    }
    
    fn update_pointer_state(&mut self, event: &PointerEvent) {
        let pointer_state = self.state.pointers.entry(event.pointer_id).or_insert(PointerState {
            device_type: event.device_type,
            position: event.position,
            pressure: event.pressure,
            buttons: event.buttons,
            last_position: event.position,
            last_timestamp: event.timestamp,
            is_down: false,
            click_count: 0,
            last_click_time: event.timestamp,
            last_click_position: event.position,
        });
        
        pointer_state.device_type = event.device_type;
        pointer_state.last_position = pointer_state.position;
        pointer_state.last_timestamp = event.timestamp;
        pointer_state.position = event.position;
        pointer_state.pressure = event.pressure;
        pointer_state.buttons = event.buttons;
        
        match event.event_type {
            PointerEventType::Down => {
                pointer_state.is_down = true;
                // Check for double click
                let time_diff = event.timestamp.saturating_sub(pointer_state.last_click_time);
                let dist = event.position.distance_to(pointer_state.last_click_position);
                
                if time_diff <= self.config.double_click_time_ms && dist <= self.config.double_click_distance {
                    pointer_state.click_count += 1;
                } else {
                    pointer_state.click_count = 1;
                }
                pointer_state.last_click_time = event.timestamp;
                pointer_state.last_click_position = event.position;
            }
            PointerEventType::Up => {
                pointer_state.is_down = false;
            }
            PointerEventType::Cancel => {
                pointer_state.is_down = false;
                pointer_state.buttons = 0;
            }
            _ => {}
        }
        
        // Update captured pointer
        if event.event_type == PointerEventType::Down {
            self.state.captured_pointer = Some(event.pointer_id);
        } else if event.event_type == PointerEventType::Up || event.event_type == PointerEventType::Cancel {
            if self.state.captured_pointer == Some(event.pointer_id) {
                self.state.captured_pointer = None;
            }
        }
    }
    
    fn update_modifiers(&mut self, event: &KeyEvent) {
        match event.key {
            Key::Control => self.state.modifiers.ctrl = event.state == KeyState::Pressed,
            Key::Shift => self.state.modifiers.shift = event.state == KeyState::Pressed,
            Key::Alt => self.state.modifiers.alt = event.state == KeyState::Pressed,
            Key::Meta | Key::Super => self.state.modifiers.meta = event.state == KeyState::Pressed,
            _ => {}
        }
    }
    
    pub fn state(&self) -> &InputState {
        &self.state
    }
    
    pub fn config(&self) -> &InputConfig {
        &self.config
    }
    
    pub fn set_config(&mut self, config: InputConfig) {
        self.config = config.clone();
        self.gesture_recognizer = GestureRecognizer::new(config);
    }
    
    /// Get pending events and clear queue
    pub fn take_events(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.event_queue)
    }
}

impl GestureRecognizer {
    pub fn new(config: InputConfig) -> Self {
        Self {
            config,
            pan_state: None,
            zoom_state: None,
            rotate_state: None,
            tap_state: TapState::default(),
            long_press_timer: None,
        }
    }
    
    pub fn process(&mut self, event: &PointerEvent, state: &InputState) -> Vec<GestureEvent> {
        let mut gestures = Vec::new();
        
        // Count active pointers
        let active_pointers: Vec<_> = state.pointers.values()
            .filter(|p| p.is_down)
            .collect();
        
        match (active_pointers.len(), event.event_type) {
            (1, PointerEventType::Down) => {
                // Single pointer down - potential pan, tap, or long press
                self.pan_state = Some(PanState {
                    start_position: event.position,
                    last_position: event.position,
                    start_time: event.timestamp,
                    last_timestamp: event.timestamp,
                    is_panning: false,
                });
                self.long_press_timer = Some(event.timestamp);
            }
            (1, PointerEventType::Move) => {
                if let Some(pan) = &mut self.pan_state {
                    let delta_point = event.position - pan.last_position;
                    let distance = delta_point.distance_to(Point2::ZERO);
                    
                    if !pan.is_panning && distance > self.config.pan_threshold {
                        pan.is_panning = true;
                        self.long_press_timer = None; // Cancel long press
                    }
                    
                    if pan.is_panning {
                        let time_diff = event.timestamp.saturating_sub(pan.last_timestamp) as f64 / 1000.0;
                        let velocity = if time_diff > 0.0 { 
                            Vector2::new(delta_point.x / time_diff, delta_point.y / time_diff) 
                        } else { 
                            Vector2::ZERO 
                        };
                        
                        gestures.push(GestureEvent::Pan { delta: Vector2::new(delta_point.x, delta_point.y), velocity });
                    }
                    
                    pan.last_position = event.position;
                    pan.last_timestamp = event.timestamp;
                }
            }
            (1, PointerEventType::Up) => {
                if let Some(pan) = &self.pan_state {
                    if !pan.is_panning {
                        // Check for tap
                        let time_diff = event.timestamp.saturating_sub(pan.start_time);
                        let distance = event.position.distance_to(pan.start_position);
                        
                        if time_diff <= self.config.double_click_time_ms && distance <= self.config.double_click_distance {
                            // Handle tap counting
                            if let Some(last_tap) = self.tap_state.last_tap_time {
                                let tap_interval = event.timestamp.saturating_sub(last_tap);
                                let tap_distance = event.position.distance_to(self.tap_state.last_tap_position.unwrap_or(event.position));
                                
                                if tap_interval <= self.config.double_click_time_ms && tap_distance <= self.config.double_click_distance {
                                    self.tap_state.tap_count += 1;
                                } else {
                                    self.tap_state.tap_count = 1;
                                }
                            } else {
                                self.tap_state.tap_count = 1;
                            }
                            
                            self.tap_state.last_tap_time = Some(event.timestamp);
                            self.tap_state.last_tap_position = Some(event.position);
                            
                            gestures.push(GestureEvent::Tap { count: self.tap_state.tap_count, position: event.position });
                        }
                    }
                }
                
                self.pan_state = None;
                self.long_press_timer = None;
            }
            (2, PointerEventType::Down) => {
                // Two pointers down - potential zoom/rotate
                let positions: Vec<Point2> = active_pointers.iter().map(|p| p.position).collect();
                if positions.len() == 2 {
                    let center = Point2::new(
                        (positions[0].x + positions[1].x) * 0.5,
                        (positions[0].y + positions[1].y) * 0.5,
                    );
                    let distance = positions[0].distance_to(positions[1]);
                    let angle = (positions[1].y - positions[0].y).atan2(positions[1].x - positions[0].x);
                    
                    self.zoom_state = Some(ZoomState {
                        start_distance: distance,
                        start_center: center,
                        start_time: event.timestamp,
                    });
                    self.rotate_state = Some(RotateState {
                        start_angle: angle,
                        start_center: center,
                        start_time: event.timestamp,
                    });
                    self.pan_state = None;
                }
            }
            (2, PointerEventType::Move) => {
                let positions: Vec<Point2> = active_pointers.iter().map(|p| p.position).collect();
                if positions.len() == 2 {
                    let center = Point2::new(
                        (positions[0].x + positions[1].x) * 0.5,
                        (positions[0].y + positions[1].y) * 0.5,
                    );
                    let distance = positions[0].distance_to(positions[1]);
                    let angle = (positions[1].y - positions[0].y).atan2(positions[1].x - positions[0].x);
                    
                    // Zoom
                    if let Some(zoom) = &self.zoom_state {
                        let factor = distance / zoom.start_distance.max(0.001);
                        if (factor - 1.0).abs() > self.config.zoom_threshold {
                            gestures.push(GestureEvent::Zoom { factor, center });
                        }
                    }
                    
                    // Rotate
                    if let Some(rotate) = &self.rotate_state {
                        let angle_delta = angle - rotate.start_angle;
                        if angle_delta.abs() > self.config.rotate_threshold {
                            gestures.push(GestureEvent::Rotate { angle: angle_delta, center });
                        }
                    }
                    
                    // Pan with two fingers
                    if let Some(pan) = &mut self.pan_state {
                        let delta_point = center - pan.last_position;
                        let time_diff = event.timestamp.saturating_sub(pan.last_timestamp) as f64 / 1000.0;
                        let velocity = if time_diff > 0.0 { 
                            Vector2::new(delta_point.x / time_diff, delta_point.y / time_diff) 
                        } else { 
                            Vector2::ZERO 
                        };
                        gestures.push(GestureEvent::Pan { delta: Vector2::new(delta_point.x, delta_point.y), velocity });
                        pan.last_position = center;
                        pan.last_timestamp = event.timestamp;
                    } else {
                        self.pan_state = Some(PanState {
                            start_position: center,
                            last_position: center,
                            start_time: event.timestamp,
                            last_timestamp: event.timestamp,
                            is_panning: true,
                        });
                    }
                }
            }
            (2, PointerEventType::Up) => {
                self.zoom_state = None;
                self.rotate_state = None;
            }
            _ => {}
        }
        
        // Check for long press
        if let Some(timer) = self.long_press_timer {
            let elapsed = event.timestamp.saturating_sub(timer);
            if elapsed >= self.config.long_press_time_ms {
                if let Some(pan) = &self.pan_state {
                    let distance = event.position.distance_to(pan.start_position);
                    if distance <= self.config.long_press_distance {
                        gestures.push(GestureEvent::LongPress { position: pan.start_position });
                        self.long_press_timer = None;
                    }
                }
            }
        }
        
        gestures
    }
}

/// Raw pointer event from platform
#[derive(Debug, Clone)]
pub struct RawPointerEvent {
    pub event_type: PointerEventType,
    pub device_type: InputDeviceType,
    pub device_id: u64,
    pub pointer_id: u64,
    pub position: Point2,
    pub pressure: f32,
    pub tangential_pressure: f32,
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub twist: f32,
    pub altitude_angle: f32,
    pub azimuth_angle: f32,
    pub contact_size: Point2,
    pub button: Option<PointerButton>,
    pub buttons: u32,
    pub modifiers: Modifiers,
    pub is_primary: bool,
}

/// Raw key event from platform
#[derive(Debug, Clone)]
pub struct RawKeyEvent {
    pub key: Key,
    pub physical_key: Option<String>,
    pub state: KeyState,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

/// Platform-independent input traits
pub trait InputBackend: Send + Sync {
    fn initialize(&mut self) -> Result<(), InputError>;
    fn poll_events(&mut self, processor: &mut InputProcessor) -> Result<(), InputError>;
    fn set_cursor(&mut self, cursor: CursorIcon) -> Result<(), InputError>;
    fn set_cursor_position(&mut self, position: Point2) -> Result<(), InputError>;
    fn get_cursor_position(&self) -> Result<Point2, InputError>;
}

/// Cursor icons
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorIcon {
    Default,
    Crosshair,
    Pointer,
    Move,
    Grab,
    Grabbing,
    Text,
    VerticalText,
    Cell,
    ContextMenu,
    Help,
    Progress,
    Wait,
    NotAllowed,
    ZoomIn,
    ZoomOut,
    ResizeN,
    ResizeS,
    ResizeE,
    ResizeW,
    ResizeNE,
    ResizeNW,
    ResizeSE,
    ResizeSW,
    ResizeNS,
    ResizeEW,
    ResizeNESW,
    ResizeNWSE,
    ColResize,
    RowResize,
    AllResize,
    None,
}

#[cfg(test)]
mod tests {
    use super::*;
    use creative_domain::Point2;
    
    #[test]
    fn test_input_processor_creation() {
        let processor = InputProcessor::new(InputConfig::default());
        assert_eq!(processor.config().double_click_time_ms, 300);
    }
    
    #[test]
    fn test_pointer_event_normalization() {
        let mut processor = InputProcessor::new(InputConfig::default());
        
        let raw = RawPointerEvent {
            event_type: PointerEventType::Down,
            device_type: InputDeviceType::Mouse,
            device_id: 1,
            pointer_id: 1,
            position: Point2::new(100.0, 100.0),
            pressure: 1.5, // Should clamp to 1.0
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
            is_primary: true,
        };
        
        let events = processor.process_pointer_event(raw);
        assert_eq!(events.len(), 1);
        
        if let InputEvent::Pointer(pe) = &events[0] {
            assert_eq!(pe.pressure, 1.0); // Clamped
            assert_eq!(pe.position, Point2::new(100.0, 100.0));
            assert_eq!(pe.button, Some(PointerButton::Left));
        }
    }
    
    #[test]
    fn test_gesture_recognizer_pan() {
        let mut processor = InputProcessor::new(InputConfig::default());
        
        // Pointer down
        let down = RawPointerEvent {
            event_type: PointerEventType::Down,
            device_type: InputDeviceType::Mouse,
            device_id: 1,
            pointer_id: 1,
            position: Point2::new(100.0, 100.0),
            pressure: 1.0,
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
            is_primary: true,
        };
        processor.process_pointer_event(down);
        
        // Pointer move (beyond pan threshold)
        let move_event = RawPointerEvent {
            event_type: PointerEventType::Move,
            device_type: InputDeviceType::Mouse,
            device_id: 1,
            pointer_id: 1,
            position: Point2::new(110.0, 110.0),
            pressure: 1.0,
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
            is_primary: true,
        };
        let events = processor.process_pointer_event(move_event);
        
        // Should have pointer event + pan gesture
        let has_pan = events.iter().any(|e| matches!(e, InputEvent::Gesture(GestureEvent::Pan { .. })));
        assert!(has_pan);
    }
    
    #[test]
    fn test_gesture_recognizer_zoom() {
        let mut processor = InputProcessor::new(InputConfig::default());
        
        // Two pointers down
        let down1 = RawPointerEvent {
            event_type: PointerEventType::Down,
            device_type: InputDeviceType::Touch,
            device_id: 1,
            pointer_id: 1,
            position: Point2::new(100.0, 100.0),
            pressure: 1.0,
            tangential_pressure: 0.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            twist: 0.0,
            altitude_angle: 1.57,
            azimuth_angle: 0.0,
            contact_size: Point2::new(1.0, 1.0),
            button: None,
            buttons: 0,
            modifiers: Modifiers::default(),
            is_primary: true,
        };
        processor.process_pointer_event(down1);
        
        let down2 = RawPointerEvent {
            event_type: PointerEventType::Down,
            device_type: InputDeviceType::Touch,
            device_id: 1,
            pointer_id: 2,
            position: Point2::new(200.0, 100.0),
            pressure: 1.0,
            tangential_pressure: 0.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            twist: 0.0,
            altitude_angle: 1.57,
            azimuth_angle: 0.0,
            contact_size: Point2::new(1.0, 1.0),
            button: None,
            buttons: 0,
            modifiers: Modifiers::default(),
            is_primary: false,
        };
        processor.process_pointer_event(down2);
        
        // Move pointers apart (zoom in)
        let move1 = RawPointerEvent {
            event_type: PointerEventType::Move,
            device_type: InputDeviceType::Touch,
            device_id: 1,
            pointer_id: 1,
            position: Point2::new(50.0, 100.0),
            pressure: 1.0,
            tangential_pressure: 0.0,
            tilt_x: 0.0,
            tilt_y: 0.0,
            twist: 0.0,
            altitude_angle: 1.57,
            azimuth_angle: 0.0,
            contact_size: Point2::new(1.0, 1.0),
            button: None,
            buttons: 0,
            modifiers: Modifiers::default(),
            is_primary: true,
        };
        let events = processor.process_pointer_event(move1);
        
        // Should have zoom gesture
        let has_zoom = events.iter().any(|e| matches!(e, InputEvent::Gesture(GestureEvent::Zoom { .. })));
        assert!(has_zoom);
    }
}