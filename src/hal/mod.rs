use std::string::String;
use std::time::{Duration, SystemTime};

use clap::Parser;
use serde::Deserialize;

pub use simplelog::{LevelFilter, TermLogger, TerminalMode};

pub mod error;
pub use error::HalError;

#[cfg(all(feature = "hal-linux", target_os = "linux"))]
pub mod linux;

#[cfg(feature = "hal-cp2130")]
pub mod cp2130;

use crate::*;

/// Generic device configuration structure for SPI drivers
#[derive(Debug, Parser, Deserialize)]
pub struct DeviceConfig {
    /// Linux SpiDev SPI device
    #[clap(long, group = "spi-kind", env = "SPI_DEV")]
    pub spi_dev: Option<String>,

    /// CP2130 SPI device
    #[clap(long, group = "spi-kind", env = "CP2130_DEV")]
    pub cp2130_dev: Option<usize>,

    #[clap(flatten)]
    #[serde(flatten)]
    pub spi: SpiConfig,

    #[clap(flatten)]
    #[serde(flatten)]
    pub pins: PinConfig,
}

/// SPI device configuration
#[derive(Debug, Clone, Parser, Deserialize)]
pub struct SpiConfig {
    /// Baud rate setting
    #[clap(long = "spi-baud", default_value = "1000000", env = "SPI_BAUD")]
    pub baud: u32,

    /// SPI mode setting
    #[clap(long = "spi-mode", default_value = "0", env = "SPI_MODE")]
    pub mode: u32,
}

/// Pin configuration object
#[derive(Debug, Clone, Parser, Deserialize)]
pub struct PinConfig {
    /// Chip Select (output) pin
    #[clap(long = "cs-pin", default_value = "16", env = "CS_PIN")]
    pub chip_select: u64,

    /// Reset (output) pin
    #[clap(long = "reset-pin", default_value = "17", env = "RESET_PIN")]
    pub reset: u64,

    /// Busy (input) pin
    #[clap(long = "busy-pin", env = "BUSY_PIN")]
    pub busy: Option<u64>,

    /// Ready (input) pin
    #[clap(long = "ready-pin", env = "READY_PIN")]
    pub ready: Option<u64>,

    /// LED 0 (output) pin
    #[clap(long = "led0-pin", env = "LED0_PIN")]
    pub led0: Option<u64>,

    /// LED 1 (output) pin
    #[clap(long = "led1-pin", env = "LED1_PIN")]
    pub led1: Option<u64>,
}

/// Log configuration object
#[derive(Debug, Parser)]
pub struct LogConfig {
    #[clap(long = "log-level", default_value = "info")]
    /// Enable verbose logging
    level: LevelFilter,
}

impl LogConfig {
    /// Initialise logging with the provided level
    pub fn init(&self) {
        TermLogger::init(
            self.level,
            simplelog::Config::default(),
            TerminalMode::Mixed,
        )
        .unwrap();
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
#[non_exhaustive]
pub enum HalSpi {
    #[cfg(all(feature = "hal-linux", target_os = "linux"))]
    Linux(linux_embedded_hal::Spidev),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Spi),
}

impl embedded_hal::spi::SpiDevice<u8> for HalSpi {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.transaction(operations)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.transaction(operations)?,
            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoDriver),
        }
        Ok(())
    }

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.write(data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.write(data)?,
            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoDriver),
        }
        Ok(())
    }

    fn transfer<'w>(&mut self, buff: &'w mut [u8], data: &'w [u8]) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.transfer(buff, data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.transfer(buff, data)?,
            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoDriver),
        }
        Ok(())
    }

    fn transfer_in_place<'w>(&mut self, data: &'w mut [u8]) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalSpi::Linux(i) => i.transfer_in_place(data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.transfer_in_place(data)?,
            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoDriver),
        }
        Ok(())
    }
}

impl embedded_hal::spi::ErrorType for HalSpi {
    type Error = HalError;
}

/// Input pin hal wrapper
#[non_exhaustive]
pub enum HalInputPin {
    #[cfg(all(feature = "hal-linux", target_os = "linux"))]
    Linux(linux_embedded_hal::SysfsPin),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::InputPin),
    None,
}

impl embedded_hal::digital::InputPin for HalInputPin {
    fn is_high(&self) -> Result<bool, Self::Error> {
        let r = match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalInputPin::Linux(i) => i.is_high()?,

            #[cfg(feature = "hal-cp2130")]
            HalInputPin::Cp2130(i) => i.is_high()?,

            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoPin),
        };

        Ok(r)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

impl embedded_hal::digital::ErrorType for HalInputPin {
    type Error = HalError;
}

/// Output pin hal wrapper
#[non_exhaustive]
pub enum HalOutputPin {
    #[cfg(all(feature = "hal-linux", target_os = "linux"))]
    Linux(linux_embedded_hal::SysfsPin),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::OutputPin),
    None,
}

impl embedded_hal::digital::OutputPin for HalOutputPin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalOutputPin::Linux(i) => i.set_high()?,

            #[cfg(feature = "hal-cp2130")]
            HalOutputPin::Cp2130(i) => i.set_high()?,

            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoPin),
        }
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(all(feature = "hal-linux", target_os = "linux"))]
            HalOutputPin::Linux(i) => i.set_low()?,

            #[cfg(feature = "hal-cp2130")]
            HalOutputPin::Cp2130(i) => i.set_low()?,

            #[allow(unreachable_patterns)]
            _ => return Err(HalError::NoPin),
        }
        Ok(())
    }
}

impl embedded_hal::digital::ErrorType for HalOutputPin {
    type Error = HalError;
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

impl embedded_hal::delay::DelayUs for HalDelay {
    fn delay_us(&mut self, us: u32) {
        let n = SystemTime::now();
        let d = Duration::from_micros(us as u64);
        while n.elapsed().unwrap() < d {}
    }
}
