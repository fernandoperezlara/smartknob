mod color;
pub mod error;
mod primitives;
pub mod text;

use core::future::Future;

use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{Dimensions, DrawTarget, IntoStorage, OriginDimensions, Pixel, Size},
};

pub use self::{color::Color, error::GraphicsError, primitives::FilledCircle};
use crate::peripherals::display::{DISPLAY_HEIGHT, DISPLAY_WIDTH, Display, error::DisplayError};

impl OriginDimensions for Display {
    fn size(&self) -> Size {
        Size::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32)
    }
}

impl DrawTarget for Display {
    type Color = Rgb565;
    type Error = DisplayError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let bb = self.bounding_box();

        pixels
            .into_iter()
            .filter(|&Pixel(pos, _color)| bb.contains(pos))
            .for_each(|Pixel(position, color)| {
                let raw_color = color.into_storage();

                self.set_pixel(position.x as u16, position.y as u16, raw_color);
            });

        Ok(())
    }
}

pub trait Graphic {
    fn draw(&self, display: &mut Display) -> impl Future<Output = Result<(), GraphicsError>>;
}

impl Display {
    pub async fn draw<T>(&mut self, shape: &T) -> Result<(), GraphicsError>
    where
        T: Graphic,
    {
        shape.draw(self).await
    }
}
