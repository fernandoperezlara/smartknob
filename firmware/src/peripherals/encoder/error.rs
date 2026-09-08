use core::fmt;

use crate::hardware::error::SpiError;

#[derive(Debug, PartialEq, Eq)]
pub enum FrameError {
    CrcMismatch { expected: u8, received: u8 },
    LossOfTrack,
    MagneticFieldTooStrong,
    MagneticFieldTooWeak,
    InvalidMagneticFieldStatus,
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CrcMismatch { expected, received } => write!(
                f,
                "CRC mismatch: expected {expected:#04x}, received {received:#04x}"
            ),
            Self::LossOfTrack => write!(f, "Encoder lost tracking"),
            Self::MagneticFieldTooStrong => write!(f, "Magnetic field is too strong"),
            Self::MagneticFieldTooWeak => write!(f, "Magnetic field is too weak"),
            Self::InvalidMagneticFieldStatus => write!(f, "Reserved magnetic field status"),
        }
    }
}

#[derive(Debug)]
pub enum EncoderError {
    Spi(SpiError),
    Frame(FrameError),
}

impl From<FrameError> for EncoderError {
    fn from(err: FrameError) -> Self {
        Self::Frame(err)
    }
}

impl From<SpiError> for EncoderError {
    fn from(err: SpiError) -> Self {
        Self::Spi(err)
    }
}

impl fmt::Display for EncoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spi(err) => write!(f, "SPI error in encoder: {}", err),
            Self::Frame(err) => write!(f, "Invalid encoder frame: {}", err),
        }
    }
}
