use std::str::Split;

use crossterm::event::KeyCode;
use tui::layout::Rect;

use crate::app::config::Key;

// borrowed from https://github.com/extrawurst/gitui
pub fn centered_rect_absolute(width: u16, height: u16, r: Rect) -> Rect {
    Rect::new(
        (r.width.saturating_sub(width)) / 2,
        (r.height.saturating_sub(height)) / 2,
        width.min(r.width),
        height.min(r.height),
    )
}

pub fn stream_input_to<F: FnMut(Key)>(input: String, mut to: F) {
    for char in input.chars() {
        let key = Key::for_key_code(KeyCode::Char(char));
        to(key)
    }
}
