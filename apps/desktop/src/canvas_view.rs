//! Canvas widget: bridges Iced pointer events into the engine's input
//! pipeline and draws the document through the camera transform.

use iced::mouse;
use iced::widget::canvas::{self, Canvas, Event, Frame, Geometry, Path, Stroke};
use iced::widget::canvas::path::arc;
use iced::widget::sensor;
use iced::{Element, Point, Radians, Rectangle, Renderer, Size};

use creative_canvas::{ShapeTool, ToolState, Viewport as CanvasViewport};
use creative_domain::Point2;
use creative_geometry::{Geometry as EngineGeometry, PathSegment, ShapeType as GeometryShapeType};

use crate::message::{Message, PointerMsg};
use crate::state::AppState;
use crate::theme;

/// The canvas rendering surface, bound to the application state
pub struct CanvasView<'a> {
    state: &'a AppState,
}

impl<'a> CanvasView<'a> {
    /// Build the canvas element for the workspace.
    ///
    /// A `sensor` wrapper reports the real layout size of the canvas so the
    /// engine's renderer viewport (and camera math) stay in sync.
    pub fn view(state: &'a AppState) -> Element<'a, Message> {
        sensor(
            Canvas::new(CanvasView { state })
                .width(iced::Fill)
                .height(iced::Fill),
        )
        .on_show(|size| Message::ViewportResized {
            width: size.width,
            height: size.height,
        })
        .on_resize(|size| Message::ViewportResized {
            width: size.width,
            height: size.height,
        })
        .into()
    }
}

impl canvas::Program<Message> for CanvasView<'_> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let position = cursor.position_in(bounds)?;
        let world = Point2::new(position.x as f64, position.y as f64);

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                Some(publish(PointerMsg::Down(world)).and_capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                Some(publish(PointerMsg::Move(world)))
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(publish(PointerMsg::Up(world)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        self.draw_document(&mut frame, bounds);
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            match self.state.active_tool {
                creative_canvas::Tool::Pan => mouse::Interaction::Grabbing,
                creative_canvas::Tool::Select => mouse::Interaction::default(),
                _ => mouse::Interaction::Crosshair,
            }
        } else {
            mouse::Interaction::default()
        }
    }
}

fn publish(msg: PointerMsg) -> canvas::Action<Message> {
    canvas::Action::publish(Message::Pointer(msg))
}

impl CanvasView<'_> {
    fn draw_document(&self, frame: &mut Frame, bounds: Rectangle) {
        let Some(document) = self.state.document else {
            return;
        };
        let Some(doc_state) = self.state.engine.document(document) else {
            return;
        };

        let viewport = CanvasViewport {
            width: bounds.width as u32,
            height: bounds.height as u32,
            scale_factor: 1.0,
        };
        let canvas = &doc_state.canvas;

        // Background
        let [r, g, b, a] = canvas.background_color();
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            iced::Color { r, g, b, a },
        );

        self.draw_grid(frame, canvas, viewport, bounds);

        // Draw objects in painter's order: render_order is top-to-bottom,
        // so paint from the bottom of the list upward.
        for layer_id in doc_state.layer_tree.render_order().iter().rev() {
            let Some(layer) = doc_state.layer_tree.get_layer(*layer_id) else {
                continue;
            };
            if !layer.visible {
                continue;
            }
            let layer_opacity = layer.opacity;

            for object_id in &layer.object_ids {
                let Some(geometry) = canvas.geometries().get(object_id) else {
                    continue;
                };
                let selected = doc_state.selection.is_selected(*object_id);
                self.draw_geometry(frame, canvas, viewport, geometry, layer_opacity, selected);
            }
        }

        self.draw_preview(frame, canvas, viewport);
    }

    /// Draw the grid, if enabled and dense enough to be readable
    fn draw_grid(
        &self,
        frame: &mut Frame,
        canvas: &creative_canvas::Canvas,
        viewport: CanvasViewport,
        bounds: Rectangle,
    ) {
        if !canvas.show_grid() {
            return;
        }

        let spacing = canvas.grid_spacing();
        let zoom = canvas.camera().zoom;

        // Skip the grid when each cell would be smaller than a few pixels
        let screen_spacing = spacing * zoom;
        if screen_spacing < 6.0 {
            return;
        }

        let [r, g, b, a] = canvas.grid_color();
        let color = iced::Color { r, g, b, a };
        let stroke = Stroke::default().with_width(1.0).with_color(color);

        // Visible world-space rectangle
        let top_left = canvas.screen_to_world(Point2::new(0.0, 0.0), viewport);
        let bottom_right = canvas.screen_to_world(
            Point2::new(bounds.width as f64, bounds.height as f64),
            viewport,
        );
        let min_x = top_left.x.min(bottom_right.x);
        let max_x = top_left.x.max(bottom_right.x);
        let min_y = top_left.y.min(bottom_right.y);
        let max_y = top_left.y.max(bottom_right.y);

        let to_screen = |p: Point2| to_screen(canvas, viewport, p);

        let mut x = (min_x / spacing).floor() * spacing;
        while x <= max_x {
            let a = to_screen(Point2::new(x, min_y));
            let b = to_screen(Point2::new(x, max_y));
            frame.stroke(&Path::line(a, b), stroke);
            x += spacing;
        }

        let mut y = (min_y / spacing).floor() * spacing;
        while y <= max_y {
            let a = to_screen(Point2::new(min_x, y));
            let b = to_screen(Point2::new(max_x, y));
            frame.stroke(&Path::line(a, b), stroke);
            y += spacing;
        }
    }

    /// Draw a single geometry object
    fn draw_geometry(
        &self,
        frame: &mut Frame,
        canvas: &creative_canvas::Canvas,
        viewport: CanvasViewport,
        geometry: &EngineGeometry,
        layer_opacity: f32,
        selected: bool,
    ) {
        let zoom = canvas.camera().zoom as f32;

        let stroke_of = |style: &creative_geometry::StrokeStyle| {
            let mut color = theme::to_iced(style.color);
            color.a *= layer_opacity;
            Stroke::default()
                .with_width(style.width * zoom)
                .with_color(color)
        };

        match geometry {
            EngineGeometry::Path(path) => {
                let iced_path = path_to_iced(canvas, viewport, &path.segments);
                if let Some(fill) = path.fill {
                    let mut color = theme::to_iced(fill);
                    color.a *= layer_opacity;
                    frame.fill(&iced_path, color);
                }
                if let Some(style) = &path.stroke {
                    frame.stroke(&iced_path, stroke_of(style));
                }
            }
            EngineGeometry::Shape(shape) => {
                let iced_path = shape_to_iced(canvas, viewport, shape.shape_type, shape.rect);
                if let Some(fill) = shape.fill {
                    let mut color = theme::to_iced(fill);
                    color.a *= layer_opacity;
                    frame.fill(&iced_path, color);
                }
                if let Some(style) = &shape.stroke {
                    frame.stroke(&iced_path, stroke_of(style));
                }
            }
            _ => {}
        }

        // Selection highlight: outline the object's world bounding box
        if selected {
            let bbox = geometry.bounding_box();
            let p1 = to_screen(canvas, viewport, bbox.min);
            let p2 = to_screen(canvas, viewport, bbox.max);
            let origin = Point::new(p1.x.min(p2.x) - 4.0, p1.y.min(p2.y) - 4.0);
            let size = iced::Size::new((p2.x - p1.x).abs() + 8.0, (p2.y - p1.y).abs() + 8.0);
            frame.stroke(
                &Path::rectangle(origin, size),
                Stroke::default()
                    .with_width(1.5)
                    .with_color(theme::ACCENT),
            );
        }
    }

    /// Draw the in-progress tool state (stroke preview, shape preview,
    /// selection rectangle, measurement line)
    fn draw_preview(
        &self,
        frame: &mut Frame,
        canvas: &creative_canvas::Canvas,
        viewport: CanvasViewport,
    ) {
        let zoom = canvas.camera().zoom as f32;
        let accent = Stroke::default()
            .with_width(1.5)
            .with_color(theme::ACCENT);

        match &canvas.state().tool_state {
            ToolState::Drawing { points, .. } => {
                if points.len() < 2 {
                    return;
                }
                let brush_color = theme::to_iced(self.state.brush_color);
                let path = Path::new(|builder| {
                    for (index, point) in points.iter().enumerate() {
                        let screen = to_screen(canvas, viewport, *point);
                        if index == 0 {
                            builder.move_to(screen);
                        } else {
                            builder.line_to(screen);
                        }
                    }
                });
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_width(self.state.brush_size * zoom)
                        .with_color(brush_color),
                );
            }
            ToolState::CreatingShape {
                start,
                current,
                shape_type,
            } => {
                let rect = creative_domain::Rect::new(*start, *current);
                let path = shape_tool_to_iced(canvas, viewport, *shape_type, rect);
                frame.stroke(&path, accent);
            }
            ToolState::Selecting { start, current } => {
                let p1 = to_screen(canvas, viewport, *start);
                let p2 = to_screen(canvas, viewport, *current);
                let origin = Point::new(p1.x.min(p2.x), p1.y.min(p2.y));
                let size = iced::Size::new((p2.x - p1.x).abs(), (p2.y - p1.y).abs());
                frame.stroke(&Path::rectangle(origin, size), accent);
            }
            ToolState::Measuring { start, current } => {
                let a = to_screen(canvas, viewport, *start);
                let b = to_screen(canvas, viewport, *current);
                frame.stroke(&Path::line(a, b), accent);
            }
            _ => {}
        }
    }
}

/// Build a full-circle elliptical path from an origin (top-left) and size
fn ellipse_path(origin: Point, size: Size) -> Path {
    let center = Point::new(origin.x + size.width / 2.0, origin.y + size.height / 2.0);
    Path::new(|builder| {
        builder.ellipse(arc::Elliptical {
            center,
            radii: iced::Vector::new(size.width / 2.0, size.height / 2.0),
            rotation: Radians(0.0),
            start_angle: Radians(0.0),
            end_angle: Radians(std::f32::consts::TAU),
        });
    })
}

/// World -> screen conversion for a canvas
fn to_screen(
    canvas: &creative_canvas::Canvas,
    viewport: CanvasViewport,
    world: Point2,
) -> Point {
    let screen = canvas.world_to_screen(world, viewport);
    Point::new(screen.x as f32, screen.y as f32)
}

/// Convert engine path segments into an iced path
fn path_to_iced(
    canvas: &creative_canvas::Canvas,
    viewport: CanvasViewport,
    segments: &[PathSegment],
) -> Path {
    Path::new(|builder| {
        for segment in segments {
            match segment {
                PathSegment::MoveTo(point) => builder.move_to(to_screen(canvas, viewport, *point)),
                PathSegment::LineTo(point) => builder.line_to(to_screen(canvas, viewport, *point)),
                PathSegment::CubicTo(c1, c2, end) => {
                    let screen_c1 = to_screen(canvas, viewport, *c1);
                    let screen_c2 = to_screen(canvas, viewport, *c2);
                    let screen_end = to_screen(canvas, viewport, *end);
                    builder.bezier_curve_to(screen_c1, screen_c2, screen_end);
                }
                PathSegment::ClosePath => builder.close(),
            }
        }
    })
}

/// Convert a shape geometry into an iced path
fn shape_to_iced(
    canvas: &creative_canvas::Canvas,
    viewport: CanvasViewport,
    shape_type: GeometryShapeType,
    rect: creative_domain::Rect,
) -> Path {
    let p1 = to_screen(canvas, viewport, rect.min);
    let p2 = to_screen(canvas, viewport, rect.max);
    let origin = Point::new(p1.x.min(p2.x), p1.y.min(p2.y));
    let size = iced::Size::new((p2.x - p1.x).abs(), (p2.y - p1.y).abs());

    match shape_type {
        GeometryShapeType::Ellipse => ellipse_path(origin, size),
        GeometryShapeType::Line => Path::line(p1, p2),
        _ => Path::rectangle(origin, size),
    }
}

/// Convert a shape tool preview into an iced path
fn shape_tool_to_iced(
    canvas: &creative_canvas::Canvas,
    viewport: CanvasViewport,
    shape_type: ShapeTool,
    rect: creative_domain::Rect,
) -> Path {
    let p1 = to_screen(canvas, viewport, rect.min);
    let p2 = to_screen(canvas, viewport, rect.max);

    match shape_type {
        ShapeTool::Line => Path::line(p1, p2),
        ShapeTool::Ellipse => {
            let origin = Point::new(p1.x.min(p2.x), p1.y.min(p2.y));
            let size = Size::new((p2.x - p1.x).abs(), (p2.y - p1.y).abs());
            ellipse_path(origin, size)
        }
        _ => {
            let origin = Point::new(p1.x.min(p2.x), p1.y.min(p2.y));
            let size = iced::Size::new((p2.x - p1.x).abs(), (p2.y - p1.y).abs());
            Path::rectangle(origin, size)
        }
    }
}
