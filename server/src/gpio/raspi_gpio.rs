use rppal::gpio::Gpio as RpGpio;
use embedded_hal::digital::{InputPin, OutputPin};
use std::time::Duration;

pub fn create_pins(close_pin: u8, open_pin: u8, coupler_pin: u8, _poll_interval: Duration, _expected_shut_time: Duration) -> Result<(impl InputPin, impl InputPin, impl OutputPin), Box<dyn std::error::Error>> {
    let gpio = RpGpio::new()?;
    let close_limit = gpio.get(close_pin)?.into_input();
    let open_limit = gpio.get(open_pin)?.into_input();
    let coupler = gpio.get(coupler_pin)?.into_output();
    Ok((close_limit, open_limit, coupler))
}
