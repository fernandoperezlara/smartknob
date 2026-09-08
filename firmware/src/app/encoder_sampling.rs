use embassy_time::{Duration, Instant, Timer};

use super::LatestPosition;
use crate::{error::SmartknobError, peripherals::encoder::Encoder};

const ENCODER_PERIOD: Duration = Duration::from_millis(1);

pub(super) async fn run(
    encoder: &mut Encoder,
    latest_position: &LatestPosition,
) -> Result<(), SmartknobError> {
    loop {
        let started = Instant::now();
        latest_position.signal(encoder.read().await?);

        let deadline = started + ENCODER_PERIOD;
        Timer::at(if deadline > Instant::now() {
            deadline
        } else {
            Instant::now() + ENCODER_PERIOD
        })
        .await;
    }
}
