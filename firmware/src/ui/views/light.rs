use alloc::{
    format,
    string::{String, ToString},
};
use core::f32::consts::TAU;

use libm::{cosf, sinf};

use super::{AppState, Display, View};
use crate::peripherals::{
    display::graphics::{
        Color, FilledCircle, GraphicsError,
        text::{HorizontalAlignment, Text, VerticalAlignment},
    },
    encoder::COUNTS_PER_REVOLUTION,
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

    fn on_rotate(&self, delta_counts: i32, state: &mut AppState) {
        state.rotation_counts += i64::from(delta_counts);
    }

    fn render(&self, state: &AppState, display: &mut Display) -> Result<(), GraphicsError> {
        let value = state.rotation_counts as f64 * (360.0 / f64::from(COUNTS_PER_REVOLUTION));
        let text = Text {
            content: format!("{value:.1}"),
            x: 120,
            y: 120,
            color: Color::WHITE,
            horizontal_align: HorizontalAlignment::Center,
            vertical_align: VerticalAlignment::Middle,
        };

        display.draw(&text)?;

        let within_turn = state
            .rotation_counts
            .rem_euclid(i64::from(COUNTS_PER_REVOLUTION));
        let angle = -TAU * (within_turn as f32 / f32::from(COUNTS_PER_REVOLUTION));
        display.draw(&FilledCircle {
            x: (120.0 + 105.0 * sinf(angle)) as u16,
            y: (120.0 - 105.0 * cosf(angle)) as u16,
            diameter: 12,
            color: Color::WHITE,
        })?;

        Ok(())
    }
}
