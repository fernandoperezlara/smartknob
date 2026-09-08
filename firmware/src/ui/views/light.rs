use alloc::{
    format,
    string::{String, ToString},
};

use super::{AppState, Display, View};
use crate::peripherals::display::graphics::{
    Color, GraphicsError,
    text::{HorizontalAlignment, Text, VerticalAlignment},
};

pub struct LightView {
    _name: String,
}

impl View for LightView {
    fn new(name: &str) -> Self {
        Self {
            _name: name.to_string(),
        }
    }

    fn render(&self, state: &AppState, display: &mut Display) -> Result<(), GraphicsError> {
        let text = Text {
            content: format!("{:.1}", state.position),
            x: 120,
            y: 120,
            color: Color::WHITE,
            horizontal_align: HorizontalAlignment::Center,
            vertical_align: VerticalAlignment::Middle,
        };

        display.draw(&text)?;

        Ok(())
    }
}
