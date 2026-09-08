use embassy_futures::yield_now;
use embedded_graphics::{
    prelude::{IntoStorage, Pixel, Point, Primitive},
    primitives::{Circle as EgCircle, PrimitiveStyle},
};
use log::debug;

use super::{Color, Display, Graphic, GraphicsError};

pub struct FilledCircle {
    pub x: u16,
    pub y: u16,
    pub diameter: u16,
    pub color: Color,
}

impl Graphic for FilledCircle {
    async fn draw(&self, display: &mut Display) -> Result<(), GraphicsError> {
        debug!(
            "Drawing filled circle at ({}, {}) with radius {} and color {:?}",
            self.x, self.y, self.diameter, self.color
        );

        let center = Point::new(self.x as i32, self.y as i32);
        let color: embedded_graphics::pixelcolor::Rgb565 = self.color.into();

        let circle = EgCircle::with_center(center, self.diameter as u32)
            .into_styled(PrimitiveStyle::with_fill(color));

        for (index, Pixel(point, color)) in circle.pixels().enumerate() {
            if let (Ok(x), Ok(y)) = (u16::try_from(point.x), u16::try_from(point.y)) {
                display.set_pixel(x, y, color.into_storage());
            }
            if (index + 1) % 128 == 0 {
                yield_now().await;
            }
        }
        yield_now().await;

        Ok(())
    }
}
