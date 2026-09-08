pub mod error;
pub mod spi;

use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    peripherals::Peripherals,
    spi::Mode,
    timer::systimer::SystemTimer,
};
use log::{debug, info};

use self::{
    error::HardwareError,
    spi::{SharedSpiBus, SpiDevice},
};

pub struct Pins {
    pub display_dc: Output<'static>,
    pub display_rst: Output<'static>,
}

pub struct Hardware {
    pub spi_bus: SharedSpiBus,
    pub display_spi: SpiDevice,
    pub encoder_spi: SpiDevice,
    pub pins: Pins,
}

impl Hardware {
    pub async fn init() -> Result<Self, HardwareError> {
        info!("Initializing components");

        let peripherals = Self::init_peripherals()?;

        let timer = SystemTimer::new(peripherals.SYSTIMER);
        esp_rtos::start(timer.alarm0, peripherals.FROM_CPU_INTR0);

        debug!("Initializing shared SPI bus");
        let spi_bus = SharedSpiBus::new(
            peripherals.SPI2,
            peripherals.DMA_CH0,
            peripherals.GPIO19,
            peripherals.GPIO18,
            peripherals.GPIO20,
        )?;

        debug!("Creating SPI devices");
        let display_spi = SpiDevice::new(&spi_bus, peripherals.GPIO1, 80, Mode::_0);
        let encoder_spi = SpiDevice::new(&spi_bus, peripherals.GPIO21, 1, Mode::_1);

        let pins = Pins {
            display_dc: Output::new(peripherals.GPIO0, Level::High, OutputConfig::default()),
            display_rst: Output::new(peripherals.GPIO2, Level::High, OutputConfig::default()),
        };

        info!("Components initialized successfully");

        Ok(Self {
            spi_bus,
            display_spi,
            encoder_spi,
            pins,
        })
    }

    fn init_peripherals() -> Result<Peripherals, HardwareError> {
        debug!("Initializing ESP32 peripherals");
        let peripherals = esp_hal::init(esp_hal::Config::default());
        debug!("ESP32 peripherals initialized successfully");
        Ok(peripherals)
    }
}
