mod light;

use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

pub use light::LightView;

use super::{AppState, Display, GraphicsError};

pub type RenderFuture<'a> = Pin<Box<dyn Future<Output = Result<(), GraphicsError>> + 'a>>;

pub trait View {
    fn new(name: &str) -> Self
    where
        Self: Sized;
    fn on_rotate(&self, delta_counts: i32, state: &mut AppState);
    fn render<'a>(&'a self, state: &'a AppState, display: &'a mut Display) -> RenderFuture<'a>;
}
