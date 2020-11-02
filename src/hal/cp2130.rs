
use std::convert::{TryFrom, TryInto};

use driver_cp2130::prelude::*;

use crate::*;
use super::{HalInst, HalPins, HalBase, HalSpi, HalInputPin, HalOutputPin, HalError, SpiConfig, PinConfig};

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
pub struct Cp2130Driver;

impl Cp2130Driver {
    /// Load base CP2130 instance
    pub fn new<'a>(index: usize, spi: &SpiConfig, pins: &PinConfig) -> Result<HalInst<'a>, HalError> {
        // Fetch the matching device and descriptor
        let (device, descriptor) = Manager::device(Filter::default(), index)?;

        // Create CP2130 object
        let cp2130 = Cp2130::new(device, descriptor, UsbOptions::default())?;

        // Connect SPI
        let spi_config = spi.clone().try_into()?;
        let spi = cp2130.spi(0, spi_config)?;
        
        // Connect pins

        let chip_select = cp2130.gpio_out(pins.chip_select as u8, GpioMode::PushPull, GpioLevel::High)?;

        let reset = cp2130.gpio_out(pins.reset as u8, GpioMode::PushPull, GpioLevel::High)?;

        let busy = match pins.busy {
            Some(p) => HalInputPin::Cp2130(cp2130.gpio_in(p as u8)?),
            None => HalInputPin::None,
        };

        let ready = match pins.ready {
            Some(p) => HalInputPin::Cp2130(cp2130.gpio_in(p as u8)?),
            None => HalInputPin::None,
        };

        let led0 = match pins.led0 {
            Some(p) => HalOutputPin::Cp2130(cp2130.gpio_out(p as u8, GpioMode::PushPull, GpioLevel::Low)?),
            None => HalOutputPin::None,
        };

        let led1 = match pins.led1 {
            Some(p) => HalOutputPin::Cp2130(cp2130.gpio_out(p as u8, GpioMode::PushPull, GpioLevel::Low)?),
            None => HalOutputPin::None,
        };

        let pins = HalPins {
            cs: HalOutputPin::Cp2130(chip_select),
            reset: HalOutputPin::Cp2130(reset),
            busy,
            ready,
            led0,
            led1,
        };

        // Return object
        Ok(HalInst{
            base: HalBase::Cp2130(cp2130),
            spi: HalSpi::Cp2130(spi),
            pins,
        })
    }
}
