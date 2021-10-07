use std::string::{String, ToString};
use std::time::{Duration, SystemTime};

use serde::Deserialize;
use structopt::StructOpt;

pub use simplelog::{LevelFilter, TermLogger, TerminalMode};

pub mod error;
pub use error::HalError;

#[cfg(all(feature = "hal-linux", target_os = "linux"))]
pub mod linux;

#[cfg(feature = "hal-cp2130")]
pub mod cp2130;

use crate::*;

/// Generic device configuration structure for SPI drivers
#[derive(Debug, StructOpt, Deserialize)]
pub struct DeviceConfig {
    /// Linux SpiDev SPI device
    #[structopt(long, group = "spi-kind", env = "SPI_DEV")]
    pub spi_dev: Option<String>,

    /// CP2130 SPI device
    #[structopt(long, group = "spi-kind", env = "CP2130_DEV")]
    pub cp2130_dev: Option<usize>,

    #[structopt(flatten)]
    #[serde(flatten)]
    pub spi: SpiConfig,

    #[structopt(flatten)]
    #[serde(flatten)]
    pub pins: PinConfig,
}

/// SPI device configuration
#[derive(Debug, Clone, StructOpt, Deserialize)]
pub struct SpiConfig {
    /// Baud rate setting
    #[structopt(long = "spi-baud", default_value = "1000000", env = "SPI_BAUD")]
    pub baud: u32,

    /// SPI mode setting
    #[structopt(long = "spi-mode", default_value = "0", env = "SPI_MODE")]
    pub mode: u32,
}

/// Pin configuration object
#[derive(Debug, Clone, StructOpt, Deserialize)]
pub struct PinConfig {
    /// Chip Select (output) pin
    #[structopt(long = "cs-pin", default_value = "16", env = "CS_PIN")]
    pub chip_select: u64,

    /// Reset (output) pin
    #[structopt(long = "reset-pin", default_value = "17", env = "RESET_PIN")]
    pub reset: u64,

    /// Busy (input) pin
    #[structopt(long = "busy-pin", env = "BUSY_PIN")]
    pub busy: Option<u64>,

    /// Ready (input) pin
    #[structopt(long = "ready-pin", env = "READY_PIN")]
    pub ready: Option<u64>,

    /// LED 0 (output) pin
    #[structopt(long = "led0-pin", env = "LED0_PIN")]
    pub led0: Option<u64>,

    /// LED 1 (output) pin
    #[structopt(long = "led1-pin", env = "LED1_PIN")]
    pub led1: Option<u64>,
}

/// Log configuration object
#[derive(Debug, StructOpt)]
pub struct LogConfig {
    #[structopt(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    level: LevelFilter,
}

impl LogConfig {
    /// Initialise logging with the provided level
    pub fn init(&self) {
        TermLogger::init(self.level, simplelog::Config::default(), TerminalMode::Mixed).unwrap();
    }
}

/// HAL instance
pub struct HalInst {
    pub base: HalBase,
    pub spi: HalSpi,
    pub pins: HalPins,
}
impl HalInst {
    /// Load a hal instance from the provided configuration
    pub fn load(config: &DeviceConfig) -> Result<HalInst, HalError> {
        // Process HAL configuration options
        let hal = match (&config.spi_dev, &config.cp2130_dev) {
            (Some(_), Some(_)) => {
                error!("Only one of spi_dev and cp2130_dev may be specified");
                return Err(HalError::InvalidConfig);
            }
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            (Some(s), None) => {
                debug!("Creating linux hal driver");
                linux::LinuxDriver::new(s, &config.spi, &config.pins)?
            }
            #[cfg(all(feature = "hal-linux", not(target_os = "linux")))]
            (Some(s), None) => {
                error!("Linux HAL only supported on linux platforms");
                return Err(HalError::InvalidConfig);
            }
            #[cfg(feature = "hal-cp2130")]
            (None, Some(i)) => {
                debug!("Creating cp2130 hal driver");
                cp2130::Cp2130Driver::new(*i, &config.spi, &config.pins)?
            }
            _ => {
                error!("No SPI configuration provided or no matching implementation found");
                return Err(HalError::InvalidConfig);
            }
        };

        Ok(hal)
    }
}

/// Base storage for Hal instances
pub enum HalBase {
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Cp2130),
    None,
}

/// SPI hal wrapper
pub enum HalSpi {
    #[cfg(all(feature = "hal-linux", target_os = "linux"))]
    Linux(linux_embedded_hal::Spidev),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Spi),
}

impl embedded_hal::spi::blocking::Transfer<u8> for HalSpi {
    type Error = HalError;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.transfer(data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.transfer(data)?,
        }
        Ok(())
    }
}

impl embedded_hal::spi::blocking::Write<u8> for HalSpi {
    type Error = HalError;

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.write(data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.write(data)?,
        }
        Ok(())
    }
}

use embedded_hal::spi::blocking::Operation;

impl embedded_hal::spi::blocking::Transactional<u8> for HalSpi {
    type Error = HalError;

    fn exec<'b>(
        &mut self,
        operations: &mut [Operation<'b, u8>],
    ) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.exec(operations)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.exec(operations)?,
        }
        Ok(())
    }
}

/// Input pin hal wrapper
pub enum HalInputPin {
    #[cfg(all(feature = "hal-linux", target_os = "linux"))]
    Linux(linux_embedded_hal::SysfsPin),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::InputPin),
    None,
}

impl embedded_hal::digital::blocking::InputPin for HalInputPin {
    type Error = HalError;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let r = match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalInputPin::Linux(i) => i.is_high()?,
            #[cfg(feature = "hal-cp2130")]
            HalInputPin::Cp2130(i) => i.is_high()?,
            HalInputPin::None => return Err(HalError::NoPin),
        };

        Ok(r)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

/// Output pin hal wrapper
pub enum HalOutputPin {
    #[cfg(all(feature = "hal-linux", target_os = "linux"))]
    Linux(linux_embedded_hal::SysfsPin),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::OutputPin),
    None,
}

impl embedded_hal::digital::blocking::OutputPin for HalOutputPin {
    type Error = HalError;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalOutputPin::Linux(i) => i.set_high()?,
            #[cfg(feature = "hal-cp2130")]
            HalOutputPin::Cp2130(i) => i.set_high()?,
            HalOutputPin::None => return Err(HalError::NoPin),
        }
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalOutputPin::Linux(i) => i.set_low()?,
            #[cfg(feature = "hal-cp2130")]
            HalOutputPin::Cp2130(i) => i.set_low()?,
            HalOutputPin::None => return Err(HalError::NoPin),
        }
        Ok(())
    }
}

/// Load a configuration file
pub fn load_config<T>(file: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    let d = std::fs::read_to_string(file).expect("error reading file");
    toml::from_str(&d).expect("error parsing toml file")
}

/// HalPins object for conveniently returning bound pins
pub struct HalPins {
    pub cs: HalOutputPin,
    pub reset: HalOutputPin,
    pub busy: HalInputPin,
    pub ready: HalInputPin,
    pub led0: HalOutputPin,
    pub led1: HalOutputPin,
}

/// HalDelay object based on blocking SystemTime::elapsed calls
pub struct HalDelay;

impl embedded_hal::delay::blocking::DelayMs<u32> for HalDelay {
    type Error = HalError;

    fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        let n = SystemTime::now();
        let d = Duration::from_millis(ms as u64);
        while n.elapsed().unwrap() < d {}
        Ok(())
    }
}
impl embedded_hal::delay::blocking::DelayUs<u32> for HalDelay {
    type Error = HalError;

    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        let n = SystemTime::now();
        let d = Duration::from_micros(us as u64);
        while n.elapsed().unwrap() < d {}
        Ok(())
    }
}
