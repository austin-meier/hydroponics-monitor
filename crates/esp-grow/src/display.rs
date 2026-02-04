use esp_hal::Async;
use esp_hal::gpio::interconnect::{PeripheralInput, PeripheralOutput};
use esp_hal::i2c::master::{Config, ConfigError, I2c};
use esp_hal::peripherals::I2C0;
use ssd1306::mode::{BufferedGraphicsModeAsync, DisplayConfigAsync};
use ssd1306::prelude::I2CInterface;
use ssd1306::size::DisplaySize128x64;
use ssd1306::{I2CDisplayInterface, Ssd1306Async};

pub type OledDisplay<'d> = Ssd1306Async<
    I2CInterface<I2c<'d, Async>>,
    DisplaySize128x64,
    BufferedGraphicsModeAsync<DisplaySize128x64>,
>;

pub async fn setup_display<'d>(
    i2c_p: I2C0<'d>,
    scl: impl PeripheralOutput<'d> + PeripheralInput<'d>,
    sda: impl PeripheralOutput<'d> + PeripheralInput<'d>,
) -> Result<OledDisplay<'d>, ConfigError> {
    let i2c_bus = I2c::new(i2c_p, Config::default())?
        .with_scl(scl)
        .with_sda(sda)
        .into_async();

    let i2c_client = I2CDisplayInterface::new(i2c_bus);

    let mut display = Ssd1306Async::new(
        i2c_client,
        DisplaySize128x64,
        ssd1306::prelude::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    display.init().await.expect("Failed to initialize display");

    Ok(display)
}
