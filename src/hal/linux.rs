extern crate linux_embedded_hal;
pub use linux_embedded_hal::sysfs_gpio::{Direction, Error as PinError};
pub use linux_embedded_hal::{spidev, spidev::SpiModeFlags, Delay, Spidev, SysfsPin as Pindev};

use super::*;

pub struct LinuxDriver;

impl LinuxDriver {
    /// Load an SPI device using the provided configuration
    pub fn new(path: &str, spi: &SpiConfig, pins: &PinConfig) -> Result<HalInst, HalError> {
        let mut flags = match spi.mode {
            0 => SpiModeFlags::SPI_MODE_0,
            1 => SpiModeFlags::SPI_MODE_1,
            2 => SpiModeFlags::SPI_MODE_2,
            3 => SpiModeFlags::SPI_MODE_3,
            _ => return Err(HalError::InvalidSpiMode),
        };

        flags |= SpiModeFlags::SPI_NO_CS;

        debug!(
            "Conecting to spi: {} at {} baud with mode: {:?}",
            path, spi.baud, flags
        );

        let spi = load_spi(path, spi.baud, flags)?;

        let pins = Self::load_pins(pins)?;

        Ok(HalInst {
            base: HalBase::None,
            spi: HalSpi::Linux(spi),
            pins,
        })
    }

    /// Load pins using the provided config
    fn load_pins(pins: &PinConfig) -> Result<HalPins, HalError> {
        let chip_select = load_pin(pins.chip_select, Direction::Out)?;

        let reset = load_pin(pins.reset, Direction::Out)?;

        let busy = match pins.busy {
            Some(p) => HalInputPin::Linux(load_pin(p, Direction::In)?),
            None => HalInputPin::None,
        };

        let ready = match pins.ready {
            Some(p) => HalInputPin::Linux(load_pin(p, Direction::In)?),
            None => HalInputPin::None,
        };

        let led0 = match pins.led0 {
            Some(p) => HalOutputPin::Linux(load_pin(p, Direction::Out)?),
            None => HalOutputPin::None,
        };

        let led1 = match pins.led1 {
            Some(p) => HalOutputPin::Linux(load_pin(p, Direction::Out)?),
            None => HalOutputPin::None,
        };

        let pins = HalPins {
            cs: HalOutputPin::Linux(chip_select),
            reset: HalOutputPin::Linux(reset),
            busy,
            ready,
            led0,
            led1,
        };

        Ok(pins)
    }
}

/// Load an SPI device using the provided configuration
fn load_spi(path: &str, baud: u32, mode: spidev::SpiModeFlags) -> Result<Spidev, HalError> {
    debug!(
        "Conecting to spi: {} at {} baud with mode: {:?}",
        path, baud, mode
    );

    let mut spi = Spidev::open(path)?;

    let mut config = spidev::SpidevOptions::new();
    config.mode(SpiModeFlags::SPI_MODE_0 | SpiModeFlags::SPI_NO_CS);
    config.max_speed_hz(baud);
    spi.configure(&config)?;

    Ok(spi)
}

/// Load a Pin using the provided configuration
fn load_pin(index: u64, direction: Direction) -> Result<Pindev, HalError> {
    debug!(
        "Connecting to pin: {} with direction: {:?}",
        index, direction
    );

    let p = Pindev::new(index);
    p.export()?;
    p.set_direction(direction)?;

    Ok(p)
}
