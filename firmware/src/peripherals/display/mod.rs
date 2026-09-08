mod commands;
mod config;
pub mod error;
pub mod graphics;

use alloc::boxed::Box;

use embassy_futures::yield_now;
use embassy_time::{Duration, Timer};
use esp_hal::gpio::Output;
use log::{debug, info};

use self::{config::CONFIG, error::DisplayError, graphics::Color};
use crate::hardware::spi::SpiDevice;

const DISPLAY_WIDTH: u16 = 240;
const DISPLAY_HEIGHT: u16 = 240;
const BUFFER_SIZE: usize = (DISPLAY_WIDTH as usize) * (DISPLAY_HEIGHT as usize) * 2;
const RENDER_STRIPE_HEIGHT: u16 = 8;
const BYTES_PER_ROW: usize = DISPLAY_WIDTH as usize * 2;

enum Operation {
    Command(u8),
    Data(&'static [u8]),
    Delay(u64),
}

pub struct Display {
    spi: SpiDevice,
    dc: Output<'static>,
    rst: Output<'static>,
    buffer: Box<[u8; BUFFER_SIZE]>,
}

impl Display {
    pub fn new(spi: SpiDevice, dc: Output<'static>, rst: Output<'static>) -> Self {
        Self {
            spi,
            dc,
            rst,
            buffer: Box::new([0; BUFFER_SIZE]),
        }
    }

    pub async fn begin(&mut self) -> Result<(), DisplayError> {
        info!("Initializing display");

        self.hardware_reset().await;
        self.initialize_display().await?;

        info!("Display initialized successfully");
        Ok(())
    }

    async fn hardware_reset(&mut self) {
        debug!("Resetting display");

        self.rst.set_high();
        Timer::after(Duration::from_millis(10)).await;
        self.rst.set_low();
        Timer::after(Duration::from_millis(120)).await;
        self.rst.set_high();
        Timer::after(Duration::from_millis(120)).await;
    }

    async fn initialize_display(&mut self) -> Result<(), DisplayError> {
        debug!("Executing display initialization sequence");

        for operation in CONFIG.iter() {
            match operation {
                Operation::Command(command) => self.write_command(*command).await?,
                Operation::Data(data) => self.write_data(data).await?,
                Operation::Delay(delay) => Timer::after(Duration::from_millis(*delay)).await,
            }
        }

        Ok(())
    }

    async fn write_command(&mut self, command: u8) -> Result<(), DisplayError> {
        self.dc.set_low();
        self.spi.write(&[command]).await?;

        Ok(())
    }

    pub async fn write_data(&mut self, data: &[u8]) -> Result<(), DisplayError> {
        self.dc.set_high();
        self.spi.write(data).await?;

        Ok(())
    }

    pub async fn sleep(&mut self) -> Result<(), DisplayError> {
        debug!("Putting display to sleep");

        self.write_command(commands::DISPOFF).await?;
        self.write_command(commands::SLPIN).await?;

        Ok(())
    }

    pub async fn wake(&mut self) -> Result<(), DisplayError> {
        debug!("Waking display");

        self.write_command(commands::SLPOUT).await?;
        self.write_command(commands::DISPON).await?;

        Ok(())
    }

    pub async fn set_frame(
        &mut self,
        x1: u16,
        y1: u16,
        x2: u16,
        y2: u16,
    ) -> Result<(), DisplayError> {
        debug!(
            "Setting frame: x1: {}, y1: {}, x2: {}, y2: {}",
            x1, y1, x2, y2
        );

        if x1 > x2 || y1 > y2 {
            return Err(DisplayError::OutOfBounds { x1, x2, y1, y2 });
        }

        if x1 >= DISPLAY_WIDTH || y1 >= DISPLAY_HEIGHT {
            return Err(DisplayError::OutOfBounds {
                x1,
                y1,
                x2: DISPLAY_WIDTH,
                y2: DISPLAY_HEIGHT,
            });
        }

        if x2 >= DISPLAY_WIDTH || y2 >= DISPLAY_HEIGHT {
            return Err(DisplayError::OutOfBounds {
                x1: x2,
                y1: y2,
                x2: DISPLAY_WIDTH,
                y2: DISPLAY_HEIGHT,
            });
        }

        self.write_command(commands::CASET).await?;
        self.write_data(&[
            (x1 >> 8) as u8,
            (x1 & 0xFF) as u8,
            (x2 >> 8) as u8,
            (x2 & 0xFF) as u8,
        ])
        .await?;

        self.write_command(commands::RASET).await?;
        self.write_data(&[
            (y1 >> 8) as u8,
            (y1 & 0xFF) as u8,
            (y2 >> 8) as u8,
            (y2 & 0xFF) as u8,
        ])
        .await?;

        self.write_command(commands::RAMWR).await?;

        Ok(())
    }

    pub fn set_pixel(&mut self, x: u16, y: u16, color: u16) {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return;
        }

        let index = ((y as usize) * (DISPLAY_WIDTH as usize) + (x as usize)) * 2;

        self.buffer[index] = (color >> 8) as u8;
        self.buffer[index + 1] = (color & 0xFF) as u8;
    }

    pub fn get_pixel(&self, x: u16, y: u16) -> Option<u16> {
        if x >= DISPLAY_WIDTH || y >= DISPLAY_HEIGHT {
            return None;
        }

        let index = ((y as usize) * (DISPLAY_WIDTH as usize) + (x as usize)) * 2;

        Some(((self.buffer[index] as u16) << 8) | (self.buffer[index + 1] as u16))
    }

    pub fn clear(&mut self, color: Color) {
        debug!("Setting background color: {:?}", color);

        let color_u16: u16 = color.into();

        for i in 0..BUFFER_SIZE / 2 {
            self.buffer[i * 2] = (color_u16 >> 8) as u8;
            self.buffer[i * 2 + 1] = (color_u16 & 0xFF) as u8;
        }
    }

    pub async fn render(&mut self) -> Result<(), DisplayError> {
        debug!("Rendering buffer to display");

        for y_start in (0..DISPLAY_HEIGHT).step_by(RENDER_STRIPE_HEIGHT as usize) {
            let y_end = (y_start + RENDER_STRIPE_HEIGHT).min(DISPLAY_HEIGHT);
            self.set_frame(0, y_start, DISPLAY_WIDTH - 1, y_end - 1)
                .await?;

            let start = y_start as usize * BYTES_PER_ROW;
            let end = y_end as usize * BYTES_PER_ROW;
            self.dc.set_high();
            self.spi.write(&self.buffer[start..end]).await?;

            yield_now().await;
        }

        Ok(())
    }
}
