//! Scratch test simulating the desktop app's pointer flow through the engine.

use creative_application::{AppConfig, Application};
use creative_input::{InputDeviceType, Modifiers, PointerButton, PointerEvent, PointerEventType};
use creative_domain::Point2;

fn pointer(event_type: PointerEventType, x: f64, y: f64, buttons: u32) -> PointerEvent {
    PointerEvent {
        event_type,
        device_type: InputDeviceType::Mouse,
        device_id: 0,
        pointer_id: 0,
        position: Point2::new(x, y),
        pressure: if event_type == PointerEventType::Down { 1.0 } else { 0.5 },
        tangential_pressure: 0.0,
        tilt_x: 0.0,
        tilt_y: 0.0,
        twist: 0.0,
        altitude_angle: 0.0,
        azimuth_angle: 0.0,
        contact_size: Point2::new(1.0, 1.0),
        button: match event_type {
            PointerEventType::Down | PointerEventType::Up => Some(PointerButton::Left),
            _ => None,
        },
        buttons,
        modifiers: Modifiers::default(),
        timestamp: 0,
        is_primary: true,
        is_coalesced: false,
        predicted_points: Vec::new(),
    }
}

#[test]
fn desktop_pointer_flow_commits_stroke() {
    let mut app = Application::new(AppConfig::default()).unwrap();
    let doc = app.create_document("test", None).unwrap();
    app.resize(doc, 1280, 800, 1.0).unwrap();

    app.handle_pointer_event(doc, pointer(PointerEventType::Down, 600.0, 300.0, 1));
    for x in 610..=660 {
        app.handle_pointer_event(doc, pointer(PointerEventType::Move, x as f64, 320.0, 1));
    }
    app.handle_pointer_event(doc, pointer(PointerEventType::Up, 670.0, 330.0, 0));

    let state = app.document(doc).unwrap();
    let canvas = &state.canvas;
    println!("geometries: {}", canvas.geometries().len());

    let layer_ids = state.layer_tree.children(state.layer_tree.root_id());
    println!("layers: {} ({:?})", layer_ids.len(), layer_ids);
    for id in &layer_ids {
        let layer = state.layer_tree.get_layer(*id).unwrap();
        println!("layer '{}' objects: {:?}", layer.name, layer.object_ids);
    }

    for (id, geom) in canvas.geometries() {
        let bbox = geom.bounding_box();
        println!("geometry {:?} bbox min=({:.1},{:.1}) max=({:.1},{:.1})",
            id, bbox.min.x, bbox.min.y, bbox.max.x, bbox.max.y);
        let vp = creative_canvas::Viewport { width: 1280, height: 800, scale_factor: 1.0 };
        let s1 = canvas.world_to_screen(bbox.min, vp);
        let s2 = canvas.world_to_screen(bbox.max, vp);
        println!("  on screen: min=({:.1},{:.1}) max=({:.1},{:.1})", s1.x, s1.y, s2.x, s2.y);
    }

    assert!(!canvas.geometries().is_empty(), "stroke was not committed");
}

#[test]
fn pen_drag_does_not_pan_camera() {
    let mut app = Application::new(AppConfig::default()).unwrap();
    let doc = app.create_document("test", None).unwrap();
    app.resize(doc, 1280, 800, 1.0).unwrap();

    let before = app.document(doc).unwrap().canvas.state().camera.center;

    app.handle_pointer_event(doc, pointer(PointerEventType::Down, 500.0, 400.0, 1));
    for x in [510.0, 520.0, 530.0, 540.0] {
        app.handle_pointer_event(doc, pointer(PointerEventType::Move, x, 410.0, 1));
    }
    app.handle_pointer_event(doc, pointer(PointerEventType::Up, 550.0, 420.0, 0));

    let after = app.document(doc).unwrap().canvas.state().camera.center;
    assert!(
        (before.x - after.x).abs() < 1e-9 && (before.y - after.y).abs() < 1e-9,
        "pen drag moved the camera: {:?} -> {:?}",
        before,
        after
    );
}

#[test]
fn pan_tool_drag_pans_linearly() {
    let mut app = Application::new(AppConfig::default()).unwrap();
    let doc = app.create_document("test", None).unwrap();
    app.resize(doc, 1280, 800, 1.0).unwrap();
    app.set_tool(doc, creative_canvas::Tool::Pan).unwrap();

    let before = app.document(doc).unwrap().canvas.state().camera.center;

    app.handle_pointer_event(doc, pointer(PointerEventType::Down, 600.0, 400.0, 1));
    for x in 610..=700 {
        app.handle_pointer_event(doc, pointer(PointerEventType::Move, x as f64, 400.0, 1));
    }
    app.handle_pointer_event(doc, pointer(PointerEventType::Up, 710.0, 400.0, 0));

    let after = app.document(doc).unwrap().canvas.state().camera.center;
    // Dragged 100px right -> camera center should move 100px left (zoom 1)
    assert!(
        ((before.x - after.x) - 100.0).abs() < 1.0,
        "pan moved by {} instead of ~100",
        before.x - after.x
    );
}
