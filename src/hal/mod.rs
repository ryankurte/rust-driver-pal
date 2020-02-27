
use std::string::{String, ToString};
use std::boxed::Box;

use serde::{Deserialize};
use structopt::StructOpt;

pub mod error;
pub use error::HalError;

#[cfg(feature = "hal-linux")]
pub mod linux;

#[cfg(feature = "hal-cp2130")]
pub mod cp2130;

use crate::*;
use crate::wrapper::Wrapper;


/// Generic device configuration structure for SPI drivers
#[derive(Debug, StructOpt, Deserialize)]
pub struct DeviceConfig {
    /// Linux SpiDev SPI device
    #[structopt(long, group = "spi-kind", env = "SPI_DEV")]
    spi_dev: Option<String>,

    /// CP2130 SPI device
    #[structopt(long, group = "spi-kind", env = "CP2130_DEV")]
    cp2130_dev: Option<usize>,

    #[structopt(flatten)]
    #[serde(flatten)]
    spi: SpiConfig,

    #[structopt(flatten)]
    #[serde(flatten)]
    pins: PinConfig,
}

/// SPI device configuration
#[derive(Debug, Clone, StructOpt, Deserialize)]
pub struct SpiConfig {
    /// Baud rate setting
    #[structopt(long = "spi-baud",
        default_value = "1000000",
        env = "SPI_BAUD"
    )]
    baud: u32,

    /// SPI mode setting
    #[structopt(long = "spi-mode", default_value="0", env="SPI_MODE")]
    mode: u32,
}

/// Pin configuration object
#[derive(Debug, Clone, StructOpt, Deserialize)]
pub struct PinConfig {
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
    #[structopt(long = "ready-pin", env = "READY_PIN")]
    ready: Option<u64>,
}

/// Load a hal instance from the provided configuration
pub fn load_hal(config: &DeviceConfig) -> Result<Box<dyn Hal<HalError>>, HalError> {

    match (&config.spi_dev, &config.cp2130_dev) {
        (Some(_), Some(_)) => {
            error!("Only one of spi_dev and cp2130_dev may be specified");
            return Err(HalError::InvalidConfig)
        },
        (Some(s), None) => {
            unimplemented!()
        },
        (None, Some(i)) => {
            let mut d = cp2130::Cp2130Driver::new(*i, &config.spi, &config.pins)?;
            let cs = d.chip_select.take().unwrap();
            let w = Wrapper::new(d, cs);
            return Ok(Box::new(w))
        },
        _ => {
            error!("No SPI configuration provided");
            return Err(HalError::InvalidConfig)
        }
    }

    unimplemented!()
}
