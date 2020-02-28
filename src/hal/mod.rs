
use std::string::{String, ToString};
use std::boxed::Box;

use serde::{Deserialize};
use structopt::StructOpt;

use embedded_hal::digital::v2::{self as digital, InputPin, OutputPin};

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

pub type HalInst = Box<dyn Hal<Error<HalError, HalError>>>;
//pub type HalInst = Box<dyn Hal<HalError>>;


/// Load a hal instance from the provided configuration
pub fn load_hal(config: &DeviceConfig) -> Result<HalInst, HalError> {

    // Process HAL configuration options
    match (&config.spi_dev, &config.cp2130_dev) {
        (Some(_), Some(_)) => {
            error!("Only one of spi_dev and cp2130_dev may be specified");
            return Err(HalError::InvalidConfig)
        },
        (Some(_s), None) => {
            unimplemented!()
        },
        (None, Some(i)) => {
            let mut spi = cp2130::Cp2130Driver::new(*i, &config.spi)?;
            let HalPins{cs, reset, busy, ready} = spi.load_pins(&config.pins)?;

            let w = Wrapper::new(spi, cs, reset, busy, ready, HalDelay);
            return Ok(Box::new(w))
        },
        _ => {
            error!("No SPI configuration provided");
            return Err(HalError::InvalidConfig)
        }
    }
}

/// HalPins object for conveniently returning bound pins
pub struct HalPins<OutputPin, InputPin> where
    OutputPin: digital::OutputPin,
    InputPin: digital::InputPin,
{
   cs: HalPin<OutputPin>,
   reset: HalPin<OutputPin>,
   busy: MaybeGpio<HalPin<InputPin>>,
   ready: MaybeGpio<HalPin<InputPin>>, 
}


/// HalPin object automatically wraps pin objects with errors that
/// can be coerced to HalError
pub struct HalPin<T> (T);

impl <'a, T, E> digital::OutputPin for HalPin<T> where
    T: digital::OutputPin<Error = E>,
    E: Into<HalError>,
{
    type Error = HalError;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.0.set_high().map_err(|e| e.into())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.0.set_low().map_err(|e| e.into())
    }
}

impl <'a, T, E> digital::InputPin for HalPin<T> where
    T: digital::InputPin<Error = E>,
    E: Into<HalError>,
{
    type Error = HalError;

    fn is_high(&self) -> Result<bool, Self::Error> {
        self.0.is_high().map_err(|e| e.into())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.0.is_low().map_err(|e| e.into())
    }
}

/// MaybeGpio wraps a GPIO option to allow for unconfigured pins
pub struct MaybeGpio<T>(Option<T>);

impl <T> From<Option<T>> for MaybeGpio<T> {
    fn from(v: Option<T>) -> Self {
        Self(v)
    }
}

impl <T> OutputPin for MaybeGpio<T> 
where 
    T: OutputPin<Error=HalError>,
{
    type Error = HalError;

    fn set_high(&mut self) -> Result<(), Self::Error> {
        let p = match self.0.as_mut() {
            Some(v) => v,
            None => return Err(HalError::NoPin),
        };

        p.set_high()
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        let p = match self.0.as_mut() {
            Some(v) => v,
            None => return Err(HalError::NoPin),
        };

        p.set_low()
    }
}

impl <T> InputPin for MaybeGpio<T> 
where 
    T: InputPin<Error=HalError>,
{
    type Error = HalError;

    fn is_high(&self) -> Result<bool, Self::Error> {
        let p = match self.0.as_ref() {
            Some(v) => v,
            None => return Err(HalError::NoPin),
        };

        p.is_high()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.is_high()?)
    }
}

/// HalDelay object based on blocking SystemTime::elapsed calls
pub struct HalDelay;

use std::time::{SystemTime, Duration};

impl DelayMs<u32> for HalDelay {
    fn delay_ms(&mut self, ms: u32) {
        let n = SystemTime::now();
        let d = Duration::from_millis(ms as u64);
        while n.elapsed().unwrap() < d {}
    }
}
impl DelayUs<u32> for HalDelay {
    fn delay_us(&mut self, us: u32) {
        let n = SystemTime::now();
        let d = Duration::from_micros(us as u64);
        while n.elapsed().unwrap() < d {}
    }
}
