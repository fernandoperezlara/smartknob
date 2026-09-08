use core::f32::consts::PI;

use crate::hardware::spi::SpiDevice;

pub mod error;
mod frame;

const RESOLUTION_BITS: u8 = 14;
pub const COUNTS_PER_REVOLUTION: u16 = 1 << RESOLUTION_BITS;
pub const ANGLE_TO_DEGREES: f32 = 360.0 / COUNTS_PER_REVOLUTION as f32;
pub const ANGLE_TO_RADIANS: f32 = (2.0 * PI) / COUNTS_PER_REVOLUTION as f32;

#[derive(Debug, PartialEq, Eq)]
pub struct Position {
    pub value: u16,
    pub status: u8,
}

impl Position {
    pub fn delta_from(&self, previous: &Self) -> i32 {
        let counts = i32::from(COUNTS_PER_REVOLUTION);
        let half = counts / 2;
        (i32::from(self.value) - i32::from(previous.value) + half).rem_euclid(counts) - half
    }
}

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
