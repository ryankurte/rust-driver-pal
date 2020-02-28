
use embedded_hal::blocking::spi;

use super::{HalError, HalPin, HalPins, MaybeGpio, SpiConfig, PinConfig};

extern crate linux_embedded_hal;
pub use linux_embedded_hal::sysfs_gpio::{Direction, Error as PinError};
pub use linux_embedded_hal::{spidev, Delay, Pin as Pindev, Spidev, spidev::SpiModeFlags};

pub struct LinuxDriver {
    spi: Spidev,
}

impl LinuxDriver {
    /// Load an SPI device using the provided configuration
    pub fn new(path: &str, spi: &SpiConfig) -> Result<Self, HalError> {
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

        Ok(Self{ spi })
    }

    /// Load pins using the provided config
    pub fn load_pins(&mut self, pins: &PinConfig) -> Result<HalPins<Pindev, Pindev>, HalError> {

        let chip_select = load_pin(pins.chip_select, Direction::Out)?;

        let reset = load_pin(pins.reset, Direction::Out)?;

        let busy = match pins.busy {
            Some(p) => Some(load_pin(p, Direction::In)?),
            None => None,
        };

        let ready = match pins.ready {
            Some(p) => Some(load_pin(p, Direction::In)?),
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

impl spi::Transfer<u8> for LinuxDriver
{
    type Error = HalError;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        let r = self.spi.transfer(data)?;
        Ok(r)
    }
}

impl spi::Write<u8> for LinuxDriver
{
    type Error = HalError;

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(data)?;
        Ok(())
    }
}

impl spi::Transactional<u8> for LinuxDriver {
    type Error = HalError;

    fn exec<'b, O>(&mut self, operations: O) -> Result<(), Self::Error>
    where
        O: AsMut<[spi::Operation<'b, u8>]> 
    {
        crate::wrapper::spi_exec(self, operations)
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
