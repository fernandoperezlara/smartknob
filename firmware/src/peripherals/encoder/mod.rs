use core::f32::consts::PI;

use crate::hardware::spi::SpiDevice;

pub mod error;
mod frame;

pub use frame::Position;

const RESOLUTION_BITS: u8 = 14;
const MAX_VALUE: u16 = (1 << RESOLUTION_BITS) - 1;
pub const ANGLE_TO_DEGREES: f32 = 360.0 / MAX_VALUE as f32;
pub const ANGLE_TO_RADIANS: f32 = (2.0 * PI) / MAX_VALUE as f32;

pub struct Encoder {
    spi: SpiDevice,
}

impl Encoder {
    pub fn new(spi: SpiDevice) -> Self {
        Self { spi }
    }

    pub async fn read(&mut self) -> Result<Position, error::EncoderError> {
        let mut buffer = [0u8; 3];
        self.spi.read(&mut buffer).await?;
        frame::decode(buffer).map_err(error::EncoderError::from)
    }
}
