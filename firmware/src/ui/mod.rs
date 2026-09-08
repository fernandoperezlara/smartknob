mod views;

use alloc::{boxed::Box, vec::Vec};

pub use views::{LightView, RenderFuture, View};

use crate::{
    app::AppState,
    peripherals::display::{Display, graphics::GraphicsError},
};

#[derive(Default)]
pub struct ViewManager {
    views: Vec<Box<dyn View>>,
}

impl ViewManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, view: Box<dyn View>) {
        self.views.push(view);
    }

    pub fn on_rotate(&self, index: usize, delta_counts: i32, state: &mut AppState) {
        if let Some(view) = self.views.get(index) {
            view.on_rotate(delta_counts, state);
        }
    }

    pub async fn select(
        &self,
        index: usize,
        state: &AppState,
        display: &mut Display,
    ) -> Result<(), GraphicsError> {
        if let Some(view) = self.views.get(index) {
            view.render(state, display).await?;
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.views.len()
    }

    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }
}
