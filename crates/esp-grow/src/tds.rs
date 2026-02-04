use defmt::info;
use embassy_time::{Duration, Timer};
use esp_hal::{
    Async,
    analog::adc::{Adc, AdcChannel, AdcConfig, AdcPin, Attenuation, CalibrationAccess, Instance, RegisterAccess},
    gpio::AnalogPin,
};
use heapless::Vec;
use micromath::F32Ext;

pub struct TdsSamplingConfig {
    pub vref: f32,               // ADC reference voltage (2.45V for 11dB attenuation)
    pub calibration_factor: f32, // sensor-specific calibration multiplier
    pub num_samples: u16,        // num of samples to take
    pub interval: Duration,      // delay between samples
}

impl TdsSamplingConfig {
    pub fn new(num_samples: u16, interval: Duration, vref: f32, calibration_factor: f32) -> Self {
        Self {
            num_samples,
            interval,
            vref,
            calibration_factor,
        }
    }
}

impl Default for TdsSamplingConfig {
    fn default() -> Self {
        Self {
            num_samples: 30,
            interval: Duration::from_millis(40),
            vref: 2.2,
            calibration_factor: 1.0,
        }
    }
}

pub struct TdsDriver<'d, PIN, ADCI> {
    config: TdsSamplingConfig,
    adc_pin: AdcPin<PIN, ADCI>,
    adc_driver: Adc<'d, ADCI, Async>,
}

impl<'d, PIN, ADCI> TdsDriver<'d, PIN, ADCI>
where
    PIN: AdcChannel + AnalogPin,
    ADCI: 'd + RegisterAccess + Instance + CalibrationAccess
{
    pub fn new(pin: PIN, adc_instance: ADCI, config: TdsSamplingConfig) -> Self {
        let mut adc1_config = AdcConfig::new();
        let adc_pin = adc1_config.enable_pin_with_cal(pin, Attenuation::_6dB);
        let adc1_driver = Adc::new(adc_instance, adc1_config).into_async();
        Self {
            config,
            adc_pin,
            adc_driver: adc1_driver,
        }
    }

    pub async fn read_ppm(&mut self, temperature: f32) -> u32 {
        let num_samples = self.config.num_samples;
        if (num_samples as usize) > 256 {
            panic!("Number of samples exceeds the maximum buffer size of 256");
        }

        let mut samples = Vec::<u16, 256>::new();

        for _ in 0..self.config.num_samples {
            let _ = samples.push(self.adc_driver.read_oneshot(&mut self.adc_pin).await);
            info!("sample: {}", samples.last().unwrap());
            Timer::after(self.config.interval).await;
        }

        samples.sort_unstable();

        let lower: usize = 5;
        let upper: usize = num_samples as usize - 5;
        let sum: u32 = samples[lower..upper].iter().map(|&x| x as u32).sum();
        let avg: u32 = sum / ((upper - lower) as u32);

        let voltage = (avg as f32 * self.config.vref * self.config.calibration_factor) / 4095.0;
        info!("voltage: {}", voltage);

        let compensation_coefficient = 1.0 + 0.02 * (temperature - 25.0);
        let compensated_voltage = voltage / compensation_coefficient;

        let tds_value = (133.42 * compensated_voltage.powi(3)
            - 255.86 * compensated_voltage.powi(2)
            + 857.39 * compensated_voltage)
            * 0.5;

        tds_value as u32
    }
}
