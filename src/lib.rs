#![no_std]

use embedded_graphics::{
    Drawable, mono_font::{MonoTextStyleBuilder, ascii::FONT_6X10}, pixelcolor::BinaryColor, prelude::Point, text::{Baseline, Text}
};
use esp_hal::{analog::adc::{AdcChannel, Instance, RegisterAccess, CalibrationAccess}, gpio::AnalogPin};
use heapless::{format, String};

use crate::tds::TdsDriver;
pub mod display;
pub mod tds;

pub struct Context<'d, PIN, ADCI> {
    display: display::OledDisplay<'d>,
    tds: TdsDriver<'d, PIN, ADCI>
}

impl<'d, PIN, ADCI> Context<'d, PIN, ADCI>
where
    PIN: AdcChannel + AnalogPin,
    ADCI: 'd + RegisterAccess + Instance + CalibrationAccess,
{
    pub fn new(display: display::OledDisplay<'d>, tds: TdsDriver<'d, PIN, ADCI>) -> Self {
        Self { display, tds }
    }

    pub async fn update(&mut self) {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        let tds_reading = self.tds.read_ppm(19.5).await;
        let tds_text: String<32> = format!("TDS: {} ppm", tds_reading).expect("Failed to format TDS text");
        self.display.clear_buffer();

        Text::with_baseline(
            &tds_text,
            Point::new(0, 32),
            text_style,
            Baseline::Top,
        ).draw(&mut self.display).expect("Failed to update text");

        self.display.flush().await.expect("Failed to flush display");

    }
}
