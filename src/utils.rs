

use std::fs::read_to_string;
use std::string::{String, ToString};

pub use serde::{Deserialize, de::DeserializeOwned};
pub use structopt::StructOpt;

pub use simplelog::{TermLogger, LevelFilter};

pub use linux_embedded_hal::{spidev, Spidev, Pin as Pindev, Delay};
pub use linux_embedded_hal::sysfs_gpio::Direction;

use crate::wrapper::Wrapper;

/// Generic device configuration structure for SPI drivers
#[derive(Debug, StructOpt, Deserialize)]
pub struct DeviceConfig {
    /// Spi device
    #[structopt(short = "d", long = "spi-dev", default_value = "/dev/spidev0.0")]
    spi: String,

    /// Baud rate setting
    #[structopt(short = "b", long = "spi-baud", default_value = "1000000", env = "SX127X_BAUD")]
    baud: u32,

    /// Chip Select (output) pin
    #[structopt(long = "cs-pin", default_value = "16")]
    chip_select: u64,

    /// Reset (output) pin
    #[structopt(long = "reset-pin")]
    reset: Option<u64>,

    /// Busy (input) pin
    #[structopt(long = "busy-pin")]
    busy: Option<u64>,

    /// Ready (input) pin
    #[structopt(long = "ready-pin")]
    ready: Option<u64>,
}

impl DeviceConfig {
    pub fn load(&self) -> Wrapper<Spidev, std::io::Error, Pindev, Pindev, ()> {
        let spi = load_spi(&self.spi, self.baud, spidev::SPI_MODE_0);
        let cs = load_pin(self.chip_select, Direction::Out);

        let mut w = Wrapper::new(spi, cs);

        if let Some(v) = self.busy {
            w.with_busy(load_pin(v, Direction::In));
        }

        if let Some(v) = self.ready {
            w.with_ready(load_pin(v, Direction::In));
        }

        if let Some(v) = self.reset {
            w.with_reset(load_pin(v, Direction::Out));
        }

        w
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
    debug!("Conecting to spi: {} at {} baud with mode: {:?}", path, baud, mode);

    let mut spi = Spidev::open(path).expect("error opening spi device");
    
    let mut config = spidev::SpidevOptions::new();
    config.mode(spidev::SPI_MODE_0);
    config.max_speed_hz(baud);
    spi.configure(&config).expect("error configuring spi device");

    spi
}

/// Load a Pin using the provided configuration
pub fn load_pin(index: u64, direction: Direction) -> Pindev {
    debug!("Connecting to pin: {} with direction: {:?}", index, direction);

    let p = Pindev::new(index);
    p.export().expect("error exporting cs pin");
    p.set_direction(direction).expect("error setting cs pin direction");

    p
}

/// Initialise logging
pub fn init_logging(level: LevelFilter) {
    TermLogger::init(level, simplelog::Config::default()).unwrap();
}

/// Load a configuration file
pub fn load_config<T>(file: &str) -> T
where T: DeserializeOwned {
    let d = read_to_string(file).expect("error reading file");
    toml::from_str(&d).expect("error parsing toml file")
}

pub fn delay() -> Delay {
    Delay{}
}
