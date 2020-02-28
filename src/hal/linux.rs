
use std::fs::read_to_string;
use std::string::{String, ToString};

use serde::{de::DeserializeOwned, Deserialize};
use structopt::StructOpt
;
pub use simplelog::{LevelFilter, TermLogger};

pub use embedded_hal::digital::v2::{InputPin, OutputPin};

use super::{HalError, HalPins, SpiConfig, PinConfig, Error};

extern crate linux_embedded_hal;
pub use linux_embedded_hal::sysfs_gpio::{Direction, Error as PinError};
pub use linux_embedded_hal::{spidev, Delay, Pin as Pindev, Spidev, spidev::SpiModeFlags};

pub struct LinuxDevice {
    spi_dev: Spidev,
}

impl LinuxDevice {
    /// Load an SPI device using the provided configuration
    pub fn new(path: &str, spi: &SpiConfig) -> Result<Self, HalError> {
        debug!(
            "Conecting to spi: {} at {} baud with mode: {:?}",
            path, baud, mode
        );

        let mut spi_dev = load_spi(path, spi.baud, spi.mode)?;

        Ok(Self{
            spi_dev,
        })
    }

    pub fn load_pins(&mut self, pin: &PinConfig) -> Result<HalPins<Pindev, Pindev>, HalError> {
        let chip_select = load_pin(pins.chip_select as u8, Direction::Out)?;

        let reset = load_pin(pins.reset as u8, Direction::Out)?;

        let busy = match pins.busy {
            Some(p) => Some(load_pin(p, Direction::In)?),
            None => None,
        };

        let ready = match pins.ready {
            Some(p) => Some(load_pin(p, Direction::In)?),
            None => None,
        };

        let pins = HalPins{
            cs: Cp2130OutputPin(chip_select),
            reset: Cp2130OutputPin(reset),
            busy: MaybeGpio( busy.map(|p| Cp2130InputPin(p)) ),
            ready: MaybeGpio( ready.map(|p| Cp2130InputPin(p)) ),
        };
        
        Ok(pins)
    }
}



/// Load an SPI device using the provided configuration
fn load_spi(path: &str, baud: u32, mode: spidev::SpiModeFlags) -> Result<Spidev, HalError> {
    debug!(
        "Conecting to spi: {} at {} baud with mode: {:?}",
        path, baud, mode
    );

    let mut spi = Spidev::open(path).expect("error opening spi device");

    let mut config = spidev::SpidevOptions::new();
    config.mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS);
    config.max_speed_hz(baud);
    spi.configure(&config)
        .expect("error configuring spi device");

    spi
}

/// Load a Pin using the provided configuration
fn load_pin(index: u64, direction: Direction) -> Result<Pindev, HalError> {
    debug!(
        "Connecting to pin: {} with direction: {:?}",
        index, direction
    );

    let p = Pindev::new(index);
    p.export().expect("error exporting cs pin");
    p.set_direction(direction)
        .expect("error setting cs pin direction");

    p
}
