mod encoder_sampling;
mod state;

use alloc::boxed::Box;

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use libm::{cosf, sinf};
use log::{debug, error, info};

pub use self::state::AppState;
use crate::{
    error::SmartknobError,
    hardware::Hardware,
    peripherals::{
        display::{
            Display,
            graphics::{Color, FilledCircle},
        },
        encoder::{ANGLE_TO_RADIANS, Encoder, Position},
    },
    ui::{LightView, View, ViewManager},
};

const DISPLAY_PERIOD: Duration = Duration::from_millis(33);

type LatestPosition = Signal<CriticalSectionRawMutex, Position>;

pub struct App {
    display: Display,
    encoder: Encoder,
    view: ViewManager,
    state: AppState,
}

impl App {
    pub async fn new() -> Result<Self, SmartknobError> {
        info!("Starting application");

        let hardware = Hardware::init().await?;
        debug!("Components initialized successfully");

        let display = Display::new(
            hardware.display_spi,
            hardware.pins.display_dc,
            hardware.pins.display_rst,
        );
        debug!("Display interface created successfully");

        let encoder = Encoder::new(hardware.encoder_spi);
        debug!("Encoder interface created successfully");

        let mut view = ViewManager::new();
        view.add(Box::new(LightView::new("Light 1")));

        let state = AppState::new();

        Ok(Self {
            display,
            encoder,
            view,
            state,
        })
    }

    pub async fn run(&mut self) -> Result<(), SmartknobError> {
        match self.display.begin().await {
            Ok(_) => info!("Display initialized successfully"),
            Err(e) => {
                error!("Failed to initialize display: {}", e);
                return Err(e.into());
            },
        }

        self.display.clear(Color::BLACK);
        self.view.select(0, &self.state, &mut self.display)?;
        self.display.render().await?;

        let latest_position = LatestPosition::new();
        info!("Starting encoder sampling (1 ms) and display refresh (33 ms)");

        match select(
            encoder_sampling::run(&mut self.encoder, &latest_position),
            Self::refresh(
                &mut self.display,
                &self.view,
                &mut self.state,
                &latest_position,
            ),
        )
        .await
        {
            Either::First(result) | Either::Second(result) => result,
        }
    }

    async fn refresh(
        display: &mut Display,
        view: &ViewManager,
        state: &mut AppState,
        latest_position: &LatestPosition,
    ) -> Result<(), SmartknobError> {
        loop {
            let started = Instant::now();
            let position = latest_position.wait().await;
            let angle = position.value as f32 * -ANGLE_TO_RADIANS;

            state.position = ((position.value as u32 * 100) / 16383) as f32;

            let x = 120.0 + 105.0 * sinf(angle);
            let y = 120.0 - 105.0 * cosf(angle);

            display.clear(Color::BLACK);

            view.select(0, state, display)?;

            display.draw(&FilledCircle {
                x: x as u16,
                y: y as u16,
                diameter: 12,
                color: Color::WHITE,
            })?;

            display.render().await?;

            let deadline = started + DISPLAY_PERIOD;
            Timer::at(if deadline > Instant::now() {
                deadline
            } else {
                Instant::now() + DISPLAY_PERIOD
            })
            .await;
        }
    }
}
