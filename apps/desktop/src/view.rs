//! Workspace layout: canvas fills the window, floating panels overlay it.

use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, slider, space, stack, text,
    toggler,
};
use iced::{Element, Fill, Length, alignment};

use crate::message::Message;
use crate::state::AppState;
use crate::theme;
use crate::update::{SWATCHES, TOOLBAR};

/// The whole workspace
pub fn view(state: &AppState) -> Element<'_, Message> {
    stack![
        crate::canvas_view::CanvasView::view(state),
        top_bar(state),
        middle_overlay(state),
        bottom_bar(state),
    ]
    .into()
}

/// Title bar: document name on the left, actions on the right
fn top_bar(state: &AppState) -> Element<'_, Message> {
    let document_name = state
        .document
        .and_then(|id| state.engine.document(id))
        .map(|doc_state| doc_state.document.meta.name.clone())
        .unwrap_or_else(|| String::from("No document"));

    let left = row![text(document_name).size(13)]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    let right = row![
        action_button("Undo", Message::Undo),
        action_button("Redo", Message::Redo),
        action_button("Save", Message::Save),
        action_button(
            if state.layers_panel_open {
                "Hide Layers"
            } else {
                "Layers"
            },
            Message::ToggleLayersPanel
        ),
    ]
    .spacing(4);

    let bar = container(
        row![
            container(left).width(Length::Fill).align_x(iced::alignment::Horizontal::Left),
            right,
        ]
        .align_y(alignment::Vertical::Center),
    )
    .padding([6, 8])
    .width(Length::Fill)
    .style(theme::panel_style);

    container(opaque(bar))
        .padding(8)
        .width(Length::Fill)
        .into()
}

/// Middle band: tool bar on the left, layers panel on the right
fn middle_overlay(state: &AppState) -> Element<'_, Message> {
    row![
        container(opaque(tool_bar(state)))
            .height(Fill)
            .align_y(alignment::Vertical::Center)
            .padding(iced::Padding {
                left: 12.0,
                ..iced::Padding::default()
            }),
        space::horizontal().width(Fill),
        container(opaque(layers_panel(state)))
            .height(Fill)
            .align_y(alignment::Vertical::Center)
            .padding(iced::Padding {
                right: 12.0,
                ..iced::Padding::default()
            }),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

/// Floating vertical tool bar
fn tool_bar(state: &AppState) -> Element<'_, Message> {
    let buttons = TOOLBAR.iter().map(|(label, tool)| {
        let tool = *tool;
        button(
            text(*label)
                .size(11)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(76)
        .padding([6, 4])
        .style(theme::tool_button_style(state.active_tool == tool))
        .on_press(Message::Tool(tool))
        .into()
    });

    container(
        column(buttons.collect::<Vec<_>>())
            .spacing(2)
            .padding(6),
    )
    .style(theme::panel_style)
    .into()
}

/// Floating layers panel
fn layers_panel(state: &AppState) -> Element<'_, Message> {
    if !state.layers_panel_open {
        return space::horizontal().width(0.0).into();
    }

    let Some(document) = state.document else {
        return space::horizontal().width(0.0).into();
    };
    let Some(doc_state) = state.engine.document(document) else {
        return space::horizontal().width(0.0).into();
    };

    // Display top layer first (render_order is top-to-bottom)
    let rows: Vec<Element<'_, Message>> = doc_state
        .layer_tree
        .render_order()
        .iter()
        .filter_map(|layer_id| {
            let layer = doc_state.layer_tree.get_layer(*layer_id)?;
            if layer.parent.is_none() {
                // Skip the invisible root container
                return None;
            }
            let layer_id = *layer_id;
            let row = row![
                toggler(layer.visible)
                    .size(14)
                    .on_toggle(move |_| Message::ToggleLayerVisible(layer_id)),
                text(layer.name.clone())
                    .size(12)
                    .width(Length::Fill),
                text(format!("{}", layer.object_ids.len()))
                    .size(10),
            ]
            .spacing(6)
            .align_y(alignment::Vertical::Center);

            Some(container(row).padding([2, 4]).into())
        })
        .collect();

    let panel = container(
        column![
            text("Layers").size(12),
            scrollable(column(rows).spacing(2)).height(Fill),
            button(
                text("Add Layer")
                    .size(11)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center)
            )
            .padding([5, 4])
            .style(theme::small_button_style())
            .on_press(Message::AddLayer),
        ]
        .spacing(6),
    )
    .padding(8)
    .width(230)
    .height(320)
    .style(theme::panel_style);

    panel.into()
}

/// Bottom bar: colour swatches, brush size, zoom controls, status
fn bottom_bar(state: &AppState) -> Element<'_, Message> {
    let swatches = SWATCHES.iter().map(|color| {
        let color = *color;
        let selected = color == state.brush_color;
        let swatch = container(space::horizontal().height(20))
            .width(24)
            .style(move |_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(theme::to_iced(color))),
                border: iced::Border {
                    color: if selected {
                        theme::ACCENT
                    } else {
                        theme::PANEL_BORDER
                    },
                    width: if selected { 2.0 } else { 1.0 },
                    radius: 5.0.into(),
                },
                text_color: None,
                shadow: Default::default(),
                snap: false,
            });

        mouse_area(swatch).on_press(Message::Color(color)).into()
    });

    let zoom_percent = state
        .document
        .and_then(|id| state.engine.document(id))
        .map(|doc_state| doc_state.canvas.camera().zoom * 100.0)
        .unwrap_or(100.0);

    let status = state
        .document
        .and_then(|id| state.engine.document(id))
        .map(|doc_state| {
            let modified = if doc_state.modified {
                " • modified"
            } else {
                ""
            };
            format!("{}{}", state.status, modified)
        })
        .unwrap_or_default();

    let bar = container(
        row![
            row(swatches.collect::<Vec<_>>()).spacing(4),
            text(format!("{:.0}px", state.brush_size)).size(11),
            slider(1.0..=64.0, state.brush_size, Message::BrushSize).width(120),
            space::horizontal().width(Fill),
            action_button("−", Message::ZoomOut),
            text(format!("{:.0}%", zoom_percent)).size(11),
            action_button("+", Message::ZoomIn),
            action_button("Fit", Message::FitAll),
            action_button("Reset", Message::ResetView),
            space::horizontal().width(Fill),
            text(status).size(11),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center),
    )
    .padding([6, 10])
    .width(Length::Fill)
    .style(theme::panel_style);

    container(opaque(bar))
        .padding(8)
        .width(Length::Fill)
        .into()
}

/// Small labelled action button
fn action_button(label: &str, message: Message) -> Element<'static, Message> {
    button(
        text(label.to_string())
            .size(11)
            .align_x(iced::alignment::Horizontal::Center),
    )
    .padding([5, 8])
    .style(theme::small_button_style())
    .on_press(message)
    .into()
}
