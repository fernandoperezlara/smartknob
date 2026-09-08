mod light;

pub use light::LightView;

use super::{AppState, Display, GraphicsError};

pub trait View {
    fn new(name: &str) -> Self
    where
        Self: Sized;
    fn on_rotate(&self, delta_counts: i32, state: &mut AppState);
    fn render(&self, state: &AppState, display: &mut Display) -> Result<(), GraphicsError>;
}
