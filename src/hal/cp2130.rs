
use std::convert::{TryFrom, TryInto};

use driver_cp2130::prelude::*;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin as _, OutputPin as _};
use embedded_hal::blocking::spi::{Write as SpiWrite, Transfer as SpiTransfer, Transactional as SpiTransactional};

use crate::*;
use super::{Error, SpiConfig, PinConfig};


impl TryInto<driver_cp2130::SpiConfig> for SpiConfig {
    type Error = Error;

    fn try_into(self) -> Result<driver_cp2130::SpiConfig, Self::Error> {
        Ok(driver_cp2130::SpiConfig {
            clock: SpiClock::try_from(self.baud as usize)?,
            ..driver_cp2130::SpiConfig::default()
        })
    }
}

pub struct Cp2130Driver<'a> {
    _cp2130: Cp2130<'a>,

    spi: Spi<'a>,

    chip_select: OutputPin<'a>,
    reset: OutputPin<'a>,
    busy: Option<InputPin<'a>>,
    ready: Option<InputPin<'a>>,
}

impl <'a>Cp2130Driver<'a> {
    /// Load base CP2130 instance
    pub fn new(index: usize, spi: &SpiConfig, pins: &PinConfig) -> Result<Self, Error> {
        // Fetch the matching device and descriptor
        let (device, descriptor) = Manager::device(Filter::default(), index)?;

        // Create CP2130 object
        let cp2130 = Cp2130::new(device, descriptor)?;

        // Connect SPI
        let spi_config = spi.clone().try_into()?;
        let spi = cp2130.spi(0, spi_config)?;

        // Connect pins

        let chip_select = cp2130.gpio_out(pins.chip_select as u8, GpioMode::PushPull, GpioLevel::High)?;

        let reset = cp2130.gpio_out(pins.reset as u8, GpioMode::PushPull, GpioLevel::High)?;

        let busy = match pins.busy {
            Some(p) => Some(cp2130.gpio_in(p as u8)?),
            None => None,
        };

        let ready = match pins.ready {
            Some(p) => Some(cp2130.gpio_in(p as u8)?),
            None => None,
        };

        // Return object
        Ok(Self{
            _cp2130: cp2130,
            spi,
            chip_select,
            reset,
            busy,
            ready,
        })
    }
}

impl <'a> Reset for Cp2130Driver<'a> {
    type Error = Error;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.reset.set_high()?,
            PinState::Low => self.reset.set_low()?,
        };

        Ok(())
    }
}

impl <'a> Busy for Cp2130Driver<'a> {
    type Error = Error;

    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        let v = self.busy.as_ref().unwrap().is_high()?;
        match v {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl <'a> Ready for Cp2130Driver<'a> {
    type Error = Error;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        let v = self.ready.as_ref().unwrap().is_high()?;
        match v {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl <'a> DelayMs<u32> for Cp2130Driver<'a> {
    fn delay_ms(&mut self, _ms: u32) {
        unimplemented!();
    }
}


impl <'a> DelayUs<u32> for Cp2130Driver<'a> {
    fn delay_us(&mut self, _us: u32) {
        unimplemented!();
    }
}


impl <'a> SpiTransfer<u8> for Cp2130Driver<'a>
{
    type Error = Error;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let r = self.spi.transfer(data)?;
        Ok(r)
    }
}
impl <'a> SpiWrite<u8> for Cp2130Driver<'a>
{
    type Error = Error;

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(data)?;
        Ok(())
    }
}

#[cfg(nope)]
impl <'a> Transactional for Cp2130Driver<'a> {
    type Error = Error;

    
}
