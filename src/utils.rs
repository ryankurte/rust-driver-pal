use std::fs::read_to_string;
use std::string::{String, ToString};

pub use serde::{de::DeserializeOwned, Deserialize};
pub use structopt::StructOpt;

pub use simplelog::{LevelFilter, TermLogger};

pub use embedded_hal::digital::v2::{InputPin, OutputPin};
pub use linux_embedded_hal::sysfs_gpio::{Direction, Error as PinError};
pub use linux_embedded_hal::{spidev, spidev::SpiModeFlags, Delay, Pin as Pindev, Spidev};

use crate::wrapper::Wrapper;

/// Generic device configuration structure for SPI drivers
#[derive(Debug, StructOpt, Deserialize)]
pub struct DeviceConfig {
    /// Spi device
    #[structopt(
        short = "d",
        long = "spi-dev",
        default_value = "/dev/spidev0.0",
        env = "SPI_DEV"
    )]
    spi: String,

    /// Baud rate setting
    #[structopt(
        short = "b",
        long = "spi-baud",
        default_value = "1000000",
        env = "SPI_BAUD"
    )]
    baud: u32,

    /// Chip Select (output) pin
    #[structopt(long = "cs-pin", default_value = "16", env = "CS_PIN")]
    chip_select: u64,

    /// Reset (output) pin
    #[structopt(long = "reset-pin", default_value = "17", env = "RESET_PIN")]
    reset: u64,

    /// Busy (input) pin
    #[structopt(long = "busy-pin", env = "BUSY_PIN")]
    busy: Option<u64>,

    /// Ready (input) pin
    #[structopt(long = "ready-pin")]
    ready: Option<u64>,
}

impl DeviceConfig {
    /// Load without busy or ready pins
    pub fn load_base(
        &self,
    ) -> Wrapper<Spidev, std::io::Error, Pindev, (), (), Pindev, PinError, Delay> {
        // Load SPI peripheral
        let spi = load_spi(&self.spi, self.baud, SpiModeFlags::SPI_MODE_0);

        // Setup CS pin
        let mut cs = load_pin(self.chip_select, Direction::Out);
        cs.set_high().unwrap();

        // Setup reset pin
        let reset = load_pin(self.reset, Direction::Out);

        Wrapper::new(spi, cs, (), (), reset, Delay {})
    }

    /// Load with busy pin
    pub fn load_with_busy(
        &self,
    ) -> Wrapper<Spidev, std::io::Error, Pindev, Pindev, (), Pindev, PinError, Delay> {
        // Load SPI peripheral
        let spi = load_spi(&self.spi, self.baud, SpiModeFlags::SPI_MODE_0);

        // Setup CS pin
        let mut cs = load_pin(self.chip_select, Direction::Out);
        cs.set_high().unwrap();

        // Setup reset pin
        let reset = load_pin(self.reset, Direction::Out);

        // Setup optional pins
        let busy = self.busy.map(|p| load_pin(p, Direction::Out)).unwrap();

        Wrapper::new(spi, cs, busy, (), reset, Delay {})
    }

    /// Load with ready pin
    pub fn load_with_ready(
        &self,
    ) -> Wrapper<Spidev, std::io::Error, Pindev, (), Pindev, Pindev, PinError, Delay> {
        // Load SPI peripheral
        let spi = load_spi(&self.spi, self.baud, SpiModeFlags::SPI_MODE_0);

        // Setup CS pin
        let mut cs = load_pin(self.chip_select, Direction::Out);
        cs.set_high().unwrap();

        // Setup reset pin
        let reset = load_pin(self.reset, Direction::Out);

        // Setup optional pins
        let ready = self.ready.map(|p| load_pin(p, Direction::In)).unwrap();

        Wrapper::new(spi, cs, (), ready, reset, Delay {})
    }

    /// Load with busy and ready pins
    pub fn load_with_busy_ready(
        &self,
    ) -> Wrapper<Spidev, std::io::Error, Pindev, Pindev, Pindev, Pindev, PinError, Delay> {
        // Load SPI peripheral
        let spi = load_spi(&self.spi, self.baud, SpiModeFlags::SPI_MODE_0);

        // Setup CS pin
        let mut cs = load_pin(self.chip_select, Direction::Out);
        cs.set_high().unwrap();

        // Setup reset pin
        let reset = load_pin(self.reset, Direction::Out);

        // Setup optional pins
        let busy = self.busy.map(|p| load_pin(p, Direction::Out)).unwrap();
        let ready = self.ready.map(|p| load_pin(p, Direction::In)).unwrap();

        Wrapper::new(spi, cs, busy, ready, reset, Delay {})
    }
}

#[derive(Debug, StructOpt)]
pub struct LogConfig {
    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    level: LevelFilter,
}

/// Load an SPI device using the provided configuration
pub fn load_spi(path: &str, baud: u32, mode: spidev::SpiModeFlags) -> Spidev {
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
pub fn load_pin(index: u64, direction: Direction) -> Pindev {
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

/// Initialise logging
pub fn init_logging(level: LevelFilter) {
    TermLogger::init(level, simplelog::Config::default()).unwrap();
}

/// Load a configuration file
pub fn load_config<T>(file: &str) -> T
where
    T: DeserializeOwned,
{
    let d = read_to_string(file).expect("error reading file");
    toml::from_str(&d).expect("error parsing toml file")
}

pub fn delay() -> Delay {
    Delay {}
}
