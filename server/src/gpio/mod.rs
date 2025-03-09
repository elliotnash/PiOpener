#[cfg(feature = "raspberry_pi")]
pub mod raspi_gpio;
#[cfg(feature = "raspberry_pi")]
pub use raspi_gpio::create_pins;

#[cfg(not(feature = "raspberry_pi"))]
pub mod mock_gpio;
#[cfg(not(feature = "raspberry_pi"))]
pub use mock_gpio::create_pins;