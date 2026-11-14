//! Visual styling helpers for the Creative Engine desktop shell.
//!
//! Dark, canvas-first look inspired by the Concepts workspace: floating
//! rounded panels over a full-window canvas.

use iced::widget::{button, container};
use iced::{Background, Border, Color, Theme};

/// Panel background (slightly translucent dark)
pub const PANEL_BG: Color = Color::from_rgba(0.09, 0.09, 0.11, 0.95);
/// Panel border colour
pub const PANEL_BORDER: Color = Color::from_rgba(0.25, 0.25, 0.30, 1.0);
/// Accent colour for active tools and selection highlights
pub const ACCENT: Color = Color::from_rgb8(0x4F, 0xA8, 0xFF);

/// Default theme: dark workspace
pub fn theme(_state: &crate::state::AppState) -> Theme {
    Theme::CatppuccinMocha
}

/// Style for floating workspace panels
pub fn panel_style(theme: &Theme) -> container::Style {
    let _ = theme;
    container::Style {
        background: Some(Background::Color(PANEL_BG)),
        border: Border {
            color: PANEL_BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: None,
        shadow: Default::default(),
        snap: false,
    }
}

/// Style for toolbar buttons; highlighted when `active`
pub fn tool_button_style(active: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, _status| {
        if active {
            button::Style {
                background: Some(Background::Color(ACCENT)),
                text_color: Color::WHITE,
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                snap: false,
            }
        } else {
            button::Style {
                background: Some(Background::Color(Color::from_rgba(0.16, 0.16, 0.20, 1.0))),
                text_color: Color::from_rgba8(0xD8, 0xD8, 0xDC, 1.0),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 6.0.into(),
                },
                shadow: Default::default(),
                snap: false,
            }
        }
    }
}

/// Style for small neutral buttons on dark panels
pub fn small_button_style() -> impl Fn(&Theme, button::Status) -> button::Style {
    |_theme, _status| button::Style {
        background: Some(Background::Color(Color::from_rgba(0.16, 0.16, 0.20, 1.0))),
        text_color: Color::from_rgba8(0xD8, 0xD8, 0xDC, 1.0),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 6.0.into(),
        },
        shadow: Default::default(),
        snap: false,
    }
}

/// Convert a domain colour into an iced colour
pub fn to_iced(color: creative_domain::Color) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}
