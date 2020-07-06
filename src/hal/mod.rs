
use std::string::{String, ToString};
use std::time::{SystemTime, Duration};
use std::marker::PhantomData;
use std::convert::Infallible;

use serde::{Deserialize};
use structopt::StructOpt;

use embedded_hal::digital;

pub use simplelog::{LevelFilter, TermLogger};

pub mod error;
pub use error::HalError;

#[cfg(feature = "hal-linux")]
pub mod linux;

#[cfg(feature = "hal-cp2130")]
pub mod cp2130;

use crate::*;

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
        TermLogger::init(self.level, simplelog::Config::default()).unwrap();
    }
}

/// HAL instance
pub struct HalInst<'a> {
    pub base: HalBase<'a>,
    pub spi: HalSpi<'a>,
    pub pins: HalPins<'a>,
}

/// Base storage for Hal instances
pub enum HalBase<'a> {
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Cp2130<'a>),
    None,

    _Fake(PhantomData<&'a ()>),
}

impl DeviceConfig {

    /// Load a hal instance from the provided configuration
    pub fn load<'a>(&self) -> Result<HalInst<'a>, HalError> {

        // Process HAL configuration options
        let hal = match (&self.spi_dev, &self.cp2130_dev) {
            (Some(_), Some(_)) => {
                error!("Only one of spi_dev and cp2130_dev may be specified");
                return Err(HalError::InvalidConfig)
            },
            #[cfg(feature = "hal-linux")]
            (Some(s), None) => {
                linux::LinuxDriver::new(s, &self.spi, &self.pins)?
            },
            #[cfg(feature = "hal-cp2130")]
            (None, Some(i)) => {
                cp2130::Cp2130Driver::new(*i, &self.spi, &self.pins)?
            },
            _ => {
                error!("No SPI configuration provided or no matching implementation found");
                return Err(HalError::InvalidConfig)
            }
        };

        Ok(hal)
    }
}

/// SPI hal wrapper
pub enum HalSpi<'a> {
    #[cfg(feature = "hal-linux")]
    Linux(linux_embedded_hal::Spidev),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Spi<'a>),

    _Fake(PhantomData<&'a ()>),
}

impl <'a> spi::Transfer<u8> for HalSpi<'a>
{
    type Error = HalError;

    fn try_transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let r = match self {
            #[cfg(feature = "hal-linux")]
            HalSpi::Linux(i) => i.try_transfer(data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.try_transfer(data)?,
            _ => unreachable!(),
        };
        Ok(r)
    }
}

impl <'a> spi::Write<u8> for HalSpi<'a>
{
    type Error = HalError;

    fn try_write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "hal-linux")]
            HalSpi::Linux(i) => i.try_write(data)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) => i.try_write(data)?,
            _ => unreachable!(),
        };
        Ok(())
    }
}

impl <'a> spi::Transactional<u8> for HalSpi<'a> {
    type Error = HalError;

    fn try_exec<'b>(&mut self, operations: &mut [spi::Operation<'b, u8>]) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "hal-linux")]
            HalSpi::Linux(i) =>  i.try_exec(operations)?,
            #[cfg(feature = "hal-cp2130")]
            HalSpi::Cp2130(i) =>  i.try_exec(operations)?,
            _ => unreachable!(),
        };
        Ok(())
    }   
}

/// Input pin hal wrapper
pub enum HalInputPin<'a> {
    #[cfg(feature = "hal-linux")]
    Linux(linux_embedded_hal::SysfsPin),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::InputPin<'a>),
    None,

    _Fake(PhantomData<&'a ()>),
}


impl <'a> digital::InputPin for HalInputPin<'a> {
    type Error = HalError;

    fn try_is_high(&self) -> Result<bool, Self::Error> {
        let r = match self {
            #[cfg(feature = "hal-linux")]
            HalInputPin::Linux(i) => i.try_is_high()?,
            #[cfg(feature = "hal-cp2130")]
            HalInputPin::Cp2130(i) => i.try_is_high()?,
            HalInputPin::None => return Err(HalError::NoPin),
            _ => unreachable!(),
        };

        Ok(r)
    }

    fn try_is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.try_is_high()?)
    }
}

/// Output pin hal wrapper
pub enum HalOutputPin<'a> {
    #[cfg(feature = "hal-linux")]
    Linux(linux_embedded_hal::SysfsPin),
    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::OutputPin<'a>),
    None,

    _Fake(PhantomData<&'a ()>),
}

impl <'a> digital::OutputPin for HalOutputPin<'a> {
    type Error = HalError;

    fn try_set_high(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "hal-linux")]
            HalOutputPin::Linux(i) => i.try_set_high()?,
            #[cfg(feature = "hal-cp2130")]
            HalOutputPin::Cp2130(i) => i.try_set_high()?,
            HalOutputPin::None => return Err(HalError::NoPin),
            _ => unreachable!(),
        }
        Ok(())
    }

    fn try_set_low(&mut self) -> Result<(), Self::Error> {
        match self {
            #[cfg(feature = "hal-linux")]
            HalOutputPin::Linux(i) => i.try_set_low()?,
            #[cfg(feature = "hal-cp2130")]
            HalOutputPin::Cp2130(i) => i.try_set_low()?,
            HalOutputPin::None => return Err(HalError::NoPin),
            _ => unreachable!(),
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
pub struct HalPins<'a> {
   pub cs: HalOutputPin<'a>,
   pub reset: HalOutputPin<'a>,
   pub busy: HalInputPin<'a>,
   pub ready: HalInputPin<'a>, 
}


/// HalDelay object based on blocking SystemTime::elapsed calls
pub struct HalDelay;

impl DelayMs<u32> for HalDelay {
    type Error = Infallible;
    
    fn try_delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        let n = SystemTime::now();
        let d = Duration::from_millis(ms as u64);
        while n.elapsed().unwrap() < d {};
        Ok(())
    }
}
impl DelayUs<u32> for HalDelay {
    type Error = Infallible;
    
    fn try_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        let n = SystemTime::now();
        let d = Duration::from_micros(us as u64);
        while n.elapsed().unwrap() < d {};
        Ok(())
    }
}
