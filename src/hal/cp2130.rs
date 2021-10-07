use std::convert::{TryFrom, TryInto};

use driver_cp2130::prelude::*;

use super::{
    HalBase, HalError, HalInputPin, HalInst, HalOutputPin, HalPins, HalSpi, PinConfig, SpiConfig,
};
use crate::*;

/// Convert a generic SPI config object into a CP2130 object
impl TryFrom<super::SpiConfig> for driver_cp2130::SpiConfig {
    type Error = HalError;

    fn try_from(c: super::SpiConfig) -> Result<driver_cp2130::SpiConfig, Self::Error> {
        Ok(driver_cp2130::SpiConfig {
            clock: SpiClock::try_from(c.baud as usize)?,
            ..driver_cp2130::SpiConfig::default()
        })
    }
}

/// CP2130 `Hal` implementation
pub struct Cp2130Driver;

impl Cp2130Driver {
    /// Load base CP2130 instance
    pub fn new(
        index: usize,
        spi_config: &SpiConfig,
        pins: &PinConfig,
    ) -> Result<HalInst, HalError> {
        // Fetch the matching device and descriptor
        let (device, descriptor) = Manager::device(Filter::default(), index)?;

        // Create CP2130 object
        let cp2130 = Cp2130::new(device, descriptor, UsbOptions::default())?;

        // Connect SPI
        let spi = cp2130.spi(0, spi_config.clone().try_into()?)?;

        // Connect pins

        let chip_select =
            cp2130.gpio_out(pins.chip_select as u8, GpioMode::PushPull, GpioLevel::High)?;

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
            Some(p) => HalOutputPin::Cp2130(cp2130.gpio_out(
                p as u8,
                GpioMode::PushPull,
                GpioLevel::Low,
            )?),
            None => HalOutputPin::None,
        };

        let led1 = match pins.led1 {
            Some(p) => HalOutputPin::Cp2130(cp2130.gpio_out(
                p as u8,
                GpioMode::PushPull,
                GpioLevel::Low,
            )?),
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
        Ok(HalInst {
            base: HalBase::Cp2130(cp2130),
            spi: HalSpi::Cp2130(spi),
            pins,
        })
    }
}
