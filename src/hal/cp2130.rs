
use std::convert::{TryFrom, TryInto};

use driver_cp2130::prelude::*;

use embedded_hal::blocking::spi::{self};

use crate::*;
use super::{HalPins, HalPin, HalError, SpiConfig, PinConfig, MaybeGpio};

/// Convert a generic SPI config object into a CP2130 object
impl TryInto<driver_cp2130::SpiConfig> for SpiConfig {
    type Error = HalError;

    fn try_into(self) -> Result<driver_cp2130::SpiConfig, Self::Error> {
        Ok(driver_cp2130::SpiConfig {
            clock: SpiClock::try_from(self.baud as usize)?,
            ..driver_cp2130::SpiConfig::default()
        })
    }
}

/// CP2130 `Hal` implementation
pub struct Cp2130Driver<'a> {
    cp2130: Cp2130<'a>,
    pub spi: Spi<'a>,
}

impl <'a>Cp2130Driver<'a> {
    /// Load base CP2130 instance
    pub fn new(index: usize, spi: &SpiConfig) -> Result<Self, HalError> {
        // Fetch the matching device and descriptor
        let (device, descriptor) = Manager::device(Filter::default(), index)?;

        // Create CP2130 object
        let cp2130 = Cp2130::new(device, descriptor)?;

        // Connect SPI
        let spi_config = spi.clone().try_into()?;
        let spi = cp2130.spi(0, spi_config)?;

        // Return object
        Ok(Self{
            cp2130,
            spi,
        })
    }

    /// Fetch pin objects from the driver
    pub fn load_pins(&mut self, pins: &PinConfig) -> Result<HalPins<OutputPin<'a>, InputPin<'a>>, HalError> {
        // Connect pins

        let chip_select = self.cp2130.gpio_out(pins.chip_select as u8, GpioMode::PushPull, GpioLevel::High)?;

        let reset = self.cp2130.gpio_out(pins.reset as u8, GpioMode::PushPull, GpioLevel::High)?;

        let busy = match pins.busy {
            Some(p) => Some(self.cp2130.gpio_in(p as u8)?),
            None => None,
        };

        let ready = match pins.ready {
            Some(p) => Some(self.cp2130.gpio_in(p as u8)?),
            None => None,
        };

        let pins = HalPins{
            cs: HalPin(chip_select),
            reset: HalPin(reset),
            busy: MaybeGpio( busy.map(|p| HalPin(p)) ),
            ready: MaybeGpio( ready.map(|p| HalPin(p)) ),
        };
        
        Ok(pins)
    }
}

impl <'a> spi::Transfer<u8> for Cp2130Driver<'a>
{
    type Error = HalError;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let r = self.spi.transfer(data)?;
        Ok(r)
    }
}

impl <'a> spi::Write<u8> for Cp2130Driver<'a>
{
    type Error = HalError;

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(data)?;
        Ok(())
    }
}

impl <'a> spi::Transactional<u8> for Cp2130Driver<'a> {
    type Error = HalError;

    fn exec<'b, O>(&mut self, operations: O) -> Result<(), Self::Error>
    where
        O: AsMut<[spi::Operation<'b, u8>]> 
    {
        crate::wrapper::spi_exec(self, operations)
    }   
}
