use super::{Position, error::FrameError};

pub(super) fn decode(bytes: [u8; 3]) -> Result<Position, FrameError> {
    let frame = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
    let payload = frame >> 6;
    let received = (frame & 0x3F) as u8;
    let expected = crc6(payload);
    if received != expected {
        return Err(FrameError::CrcMismatch { expected, received });
    }

    let status = (payload & 0x0F) as u8;
    if status & 0x08 != 0 {
        return Err(FrameError::LossOfTrack);
    }
    match status & 0x03 {
        1 => return Err(FrameError::MagneticFieldTooStrong),
        2 => return Err(FrameError::MagneticFieldTooWeak),
        3 => return Err(FrameError::InvalidMagneticFieldStatus),
        _ => {},
    }

    Ok(Position {
        value: (payload >> 4) as u16,
        status,
    })
}

fn crc6(payload: u32) -> u8 {
    let mut crc = 0u8;
    for bit in (0..18).rev() {
        let feedback = ((crc >> 5) ^ ((payload >> bit) as u8)) & 1;
        crc = (crc << 1) & 0x3F;
        if feedback != 0 {
            crc ^= 0x03;
        }
    }
    crc
}
