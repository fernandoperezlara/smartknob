use embassy_time::{Duration, Instant, Timer};
use log::{info, warn};

use super::LatestPosition;
use crate::{error::SmartknobError, peripherals::encoder::Encoder};

const ENCODER_PERIOD: Duration = Duration::from_millis(1);
const WARNING_INTERVAL: Duration = Duration::from_secs(1);
const UNAVAILABLE_AFTER: Duration = Duration::from_millis(500);

pub(super) async fn run(
    encoder: &mut Encoder,
    latest_position: &LatestPosition,
) -> Result<(), SmartknobError> {
    let mut failure_since = None;
    let mut last_warning = None;
    let mut unavailable = false;

    loop {
        let started = Instant::now();
        match encoder.read().await {
            Ok(position) => {
                failure_since = None;
                if unavailable {
                    info!("Encoder readings recovered");
                    unavailable = false;
                }
                latest_position.signal(position);
            },
            Err(err) => {
                let now = Instant::now();
                let first_failure = *failure_since.get_or_insert(now);
                if last_warning.is_none_or(|last| now - last >= WARNING_INTERVAL) {
                    warn!("Discarding encoder reading: {}", err);
                    last_warning = Some(now);
                }
                if !unavailable && now - first_failure >= UNAVAILABLE_AFTER {
                    unavailable = true;
                }
            },
        }

        let deadline = started + ENCODER_PERIOD;
        Timer::at(if deadline > Instant::now() {
            deadline
        } else {
            Instant::now() + ENCODER_PERIOD
        })
        .await;
    }
}
