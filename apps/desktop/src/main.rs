//! Creative Engine — desktop application entry point.
//!
//! Iced workspace shell around the shared creative engine: a full-window
//! canvas with floating tool, layer and colour panels, Concepts-style.

mod canvas_view;
mod message;
mod state;
mod subscription;
mod theme;
mod update;
mod view;

use state::AppState;
pub fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    iced::application(AppState::new, update::update, view::view)
        .title("Creative Engine")
        .theme(theme::theme)
        .subscription(subscription::subscription)
        .window_size(iced::Size::new(1280.0, 800.0))
        .centered()
        .run()
}
