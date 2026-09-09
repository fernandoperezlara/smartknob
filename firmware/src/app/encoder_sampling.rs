use core::convert::Infallible;

use embassy_time::{Duration, Instant, Timer};
use log::{info, warn};

use crate::peripherals::encoder::{Encoder, Position};

const ENCODER_PERIOD: Duration = Duration::from_millis(1);
const WARNING_INTERVAL: Duration = Duration::from_secs(1);
const UNAVAILABLE_AFTER: Duration = Duration::from_millis(500);

pub async fn run(encoder: &mut Encoder, mut on_sample: impl FnMut(&Position, i32)) -> Infallible {
    let mut previous: Option<Position> = None;
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
                let delta = previous
                    .as_ref()
                    .map_or(0, |last| position.delta_from(last));
                on_sample(&position, delta);
                previous = Some(position);
            },
            Err(err) => {
                previous = None;
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
