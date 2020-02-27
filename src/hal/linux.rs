
use std::fs::read_to_string;
use std::string::{String, ToString};

use serde::{de::DeserializeOwned, Deserialize};
use structopt::StructOpt
;
pub use simplelog::{LevelFilter, TermLogger};

pub use embedded_hal::digital::v2::{InputPin, OutputPin};

use super::{SpiConfig, PinConfig, Error};

extern crate linux_embedded_hal;
pub use linux_embedded_hal::sysfs_gpio::{Direction, Error as PinError};
pub use linux_embedded_hal::{spidev, Delay, Pin as Pindev, Spidev, spidev::SpiModeFlags};

pub struct LinuxDevice {
    spi_dev: Spidev,
}

impl LinuxDevice {
    /// Load an SPI device using the provided configuration
    pub fn new(path: &str, spi: &SpiConfig, pin: &PinConfig) -> Result<Spidev, Error> {
        debug!(
            "Conecting to spi: {} at {} baud with mode: {:?}",
            path, baud, mode
        );

        let mut spi_dev = Spidev::open(path).expect("error opening spi device");

        let mut config = spidev::SpidevOptions::new();
        config.mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS);
        config.max_speed_hz(baud);
        spi_dev.configure(&config)
            .expect("error configuring spi device");

        Ok(Self{
            spi_dev,
        })
    }
}



