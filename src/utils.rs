
use std::fs::read_to_string;

use structopt::StructOpt;
use serde::de::Deserialize;
use simplelog::{TermLogger, LevelFilter};

use linux_embedded_hal::{spidev, Spidev, Pin as Pindev, Delay};
use linux_embedded_hal::sysfs_gpio::Direction;

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
    cs: u64,

    /// Reset (output) pin
    #[structopt(long = "rst-pin", default_value = "17")]
    rst: u64,

    /// Busy (input) pin
    #[structopt(long = "busy-pin")]
    busy: Option<u64>,

    /// Ready (input) pin
    #[structopt(long = "ready-pin")]
    ready: Option<u64>,
}

#[derive(Debug, StructOpt, Deserialize)]
pub struct LogConfig {
    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    level: LevelFilter,
}

pub fn load_spi(path: &str, baud: u32, mode: spidev::Mode) -> Spidev {
    debug!("Conecting to spi: {} at {} baud with mode: {:?}", path, baud, mode);

    let mut spi = Spidev::open(opts.spi).expect("error opening spi device");
    
    let mut config = spidev::SpidevOptions::new();
    config.mode(spidev::SPI_MODE_0);
    config.max_speed_hz(baud);
    spi.configure(&config).expect("error configuring spi device");

    spi
}

pub fn load_pin(index: u64, direction: Direction) -> Pindev {
    debug!("Connecting to pin: {} with direction: {:?}", pin, direction);

    let p = PinDev::new(index);
    p.export().expect("error exporting cs pin");
    p.set_direction(direction).expect("error setting cs pin direction");

    p
}

pub fn init_logging(level: LevelFilter) {
    TermLogger::init(opts.level, simplelog::Config::default()).unwrap();
}

pub fn load_config<T>(file: &str) -> T
where T: Deserialize {
    let d = read_to_string(file).expect("error reading file");
    toml::from_str(d).expect("error parsing toml file")
}
