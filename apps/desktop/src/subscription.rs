//! Keyboard subscription: forwards global shortcuts to the engine.

use iced::event::{self, Status};
use iced::{keyboard, Subscription};

use crate::message::Message;

/// Global keyboard handling (Ctrl+Z/Y/S/A, Escape, Delete ...)
pub fn subscription(_state: &crate::state::AppState) -> Subscription<Message> {
    event::listen_with(|event, status, _window_id| {
        if status != Status::Ignored {
            return None;
        }

        let iced::Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event
        else {
            return None;
        };

        let key = match &key {
            keyboard::Key::Character(chars) => {
                let first = chars
                    .chars()
                    .next()
                    .and_then(|c| c.to_lowercase().next());
                match first {
                    Some(c) => creative_input::Key::Character(c),
                    None => return None,
                }
            }
            keyboard::Key::Named(named) => match named {
                keyboard::key::Named::Escape => creative_input::Key::Escape,
                keyboard::key::Named::Delete => creative_input::Key::Delete,
                keyboard::key::Named::Backspace => creative_input::Key::Backspace,
                keyboard::key::Named::Space => creative_input::Key::Space,
                keyboard::key::Named::Enter => creative_input::Key::Enter,
                keyboard::key::Named::ArrowUp => creative_input::Key::ArrowUp,
                keyboard::key::Named::ArrowDown => creative_input::Key::ArrowDown,
                keyboard::key::Named::ArrowLeft => creative_input::Key::ArrowLeft,
                keyboard::key::Named::ArrowRight => creative_input::Key::ArrowRight,
                _ => return None,
            },
            _ => return None,
        };

        Some(Message::Key {
            key,
            pressed: true,
            ctrl: modifiers.control(),
            shift: modifiers.shift(),
            alt: modifiers.alt(),
            meta: modifiers.logo(),
        })
    })
}
