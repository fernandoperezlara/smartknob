use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_hal_async::spi::SpiBus;
use esp_hal::{
    Async, dma_rx_buffer, dma_tx_buffer,
    gpio::{InputPin, Level, Output, OutputConfig, OutputPin},
    spi::{
        Mode,
        master::{Config, Instance, Spi, SpiDma},
    },
    time::Rate,
};
use log::debug;
use static_cell::StaticCell;

use super::error::SpiError;

const DMA_BUFFER_SIZE: usize = 4096;

static SPI_BUS: StaticCell<Mutex<CriticalSectionRawMutex, SpiDma<'static, Async>>> =
    StaticCell::new();

pub struct SharedSpiBus {
    inner: &'static Mutex<CriticalSectionRawMutex, SpiDma<'static, Async>>,
}

impl SharedSpiBus {
    pub fn new<SPI, SCLK, MOSI, MISO>(
        spi_instance: SPI,
        dma_channel: esp_hal::peripherals::DMA_CH0<'static>,
        sclk: SCLK,
        mosi: MOSI,
        miso: MISO,
    ) -> Result<Self, SpiError>
    where
        SPI: Instance + 'static,
        SCLK: OutputPin + 'static,
        MOSI: OutputPin + 'static,
        MISO: InputPin + 'static,
    {
        debug!("Initializing shared SPI bus");

        let spi_config = Config::default();

        let dma_rx_buf = dma_rx_buffer!(DMA_BUFFER_SIZE)?;
        let dma_tx_buf = dma_tx_buffer!(DMA_BUFFER_SIZE)?;

        let spi = Spi::new(spi_instance, spi_config)
            .map_err(SpiError::from)?
            .with_sck(sclk)
            .with_mosi(mosi)
            .with_miso(miso)
            .with_dma(dma_channel)
            .with_buffers(dma_rx_buf, dma_tx_buf)
            .into_async();

        debug!("Shared SPI interface initialized successfully");

        let inner = SPI_BUS.init(Mutex::new(spi));

        Ok(Self { inner })
    }

    pub fn bus(&self) -> &'static Mutex<CriticalSectionRawMutex, SpiDma<'static, Async>> {
        self.inner
    }
}

pub struct SpiDevice {
    bus: &'static Mutex<CriticalSectionRawMutex, SpiDma<'static, Async>>,
    cs: Output<'static>,
    frequency: u32,
    mode: Mode,
}

struct ChipSelectGuard<'a> {
    cs: &'a mut Output<'static>,
}

impl<'a> ChipSelectGuard<'a> {
    fn new(cs: &'a mut Output<'static>) -> Self {
        cs.set_low();
        Self { cs }
    }
}

impl Drop for ChipSelectGuard<'_> {
    fn drop(&mut self) {
        self.cs.set_high();
    }
}

impl SpiDevice {
    pub fn new<CS>(bus: &SharedSpiBus, cs_pin: CS, frequency: u32, mode: Mode) -> Self
    where
        CS: OutputPin + 'static,
    {
        debug!("Creating SPI device with CS pin at {}MHz", frequency);

        Self {
            bus: bus.bus(),
            cs: Output::new(cs_pin, Level::High, OutputConfig::default()),
            frequency,
            mode,
        }
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), SpiError> {
        if data.is_empty() {
            return Err(SpiError::invalid_parameters(
                "Write data buffer cannot be empty",
            ));
        }

        let mut bus = self.bus.lock().await;

        let config = Config::default()
            .with_frequency(Rate::from_mhz(self.frequency))
            .with_mode(self.mode);
        bus.apply_config(&config)?;

        let chip_select = ChipSelectGuard::new(&mut self.cs);
        let result = SpiBus::write(&mut *bus, data).await;
        drop(chip_select);

        result.map_err(|_| SpiError::write_failed("Failed to write data to SPI bus"))
    }

    pub async fn read(&mut self, data: &mut [u8]) -> Result<(), SpiError> {
        if data.is_empty() {
            return Err(SpiError::invalid_parameters(
                "Read data buffer cannot be empty",
            ));
        }

        let mut bus = self.bus.lock().await;

        let config = Config::default()
            .with_frequency(Rate::from_mhz(self.frequency))
            .with_mode(self.mode);
        bus.apply_config(&config)?;

        let chip_select = ChipSelectGuard::new(&mut self.cs);
        let result = SpiBus::read(&mut *bus, data).await;
        drop(chip_select);

        result.map_err(|_| SpiError::read_failed("Failed to read data from SPI bus"))
    }

    pub async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), SpiError> {
        if read.is_empty() && write.is_empty() {
            return Err(SpiError::invalid_parameters(
                "Read and write data buffers cannot be empty",
            ));
        }

        if read.len() != write.len() {
            return Err(SpiError::invalid_parameters(
                "Read and write data buffers must have the same length",
            ));
        }

        let mut bus = self.bus.lock().await;

        let config = Config::default()
            .with_frequency(Rate::from_mhz(self.frequency))
            .with_mode(self.mode);
        bus.apply_config(&config)?;

        let chip_select = ChipSelectGuard::new(&mut self.cs);
        let result = SpiBus::transfer(&mut *bus, read, write).await;
        drop(chip_select);

        result.map_err(|_| SpiError::transfer_failed("Failed to transfer data on SPI bus"))
    }

    pub async fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), SpiError> {
        if data.is_empty() {
            return Err(SpiError::invalid_parameters("Data buffer cannot be empty"));
        }

        let mut bus = self.bus.lock().await;

        let config = Config::default()
            .with_frequency(Rate::from_mhz(self.frequency))
            .with_mode(self.mode);
        bus.apply_config(&config)?;

        let chip_select = ChipSelectGuard::new(&mut self.cs);
        let result = SpiBus::transfer_in_place(&mut *bus, data).await;
        drop(chip_select);

        result.map_err(|_| SpiError::transfer_failed("Failed to transfer data in place on SPI bus"))
    }
}
