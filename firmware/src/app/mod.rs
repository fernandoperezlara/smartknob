mod encoder_sampling;
mod state;

use alloc::boxed::Box;

use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use log::{debug, error, info};

pub use self::state::AppState;
use crate::{
    error::SmartknobError,
    hardware::Hardware,
    peripherals::{
        display::{Display, graphics::Color},
        encoder::Encoder,
    },
    ui::{LightView, View, ViewManager},
};

const DISPLAY_PERIOD: Duration = Duration::from_millis(33);

type LatestState = Signal<CriticalSectionRawMutex, AppState>;

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

        self.display.clear(Color::BLACK).await;
        self.view.select(0, &self.state, &mut self.display).await?;
        self.display.render().await?;

        let latest_state = LatestState::new();
        info!("Starting encoder sampling (1 ms) and display refresh (33 ms)");

        let view = &self.view;
        let state = &mut self.state;
        match select(
            encoder_sampling::run(&mut self.encoder, |_position, delta_counts| {
                view.on_rotate(0, delta_counts, state);
                latest_state.signal(*state);
            }),
            Self::refresh(&mut self.display, view, &latest_state),
        )
        .await
        {
            Either::First(never) => match never {},
            Either::Second(result) => result,
        }
    }

    async fn refresh(
        display: &mut Display,
        view: &ViewManager,
        latest_state: &LatestState,
    ) -> Result<(), SmartknobError> {
        loop {
            let started = Instant::now();
            let state = latest_state.wait().await;
            display.clear(Color::BLACK).await;
            view.select(0, &state, display).await?;

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
