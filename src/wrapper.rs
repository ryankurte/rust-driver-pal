//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{self, Transfer, Write, Operation};
use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::{Busy, Error, PinState, Ready, Reset, Transactional};

/// Wrapper wraps an Spi and Pin object to support transactions
#[derive(Debug, Clone, PartialEq)]
pub struct Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> {
    /// SPI device
    spi: Spi,

    /// Chip select pin (required)
    cs: CsPin,

    /// Delay implementation
    delay: Delay,

    /// Busy input pin (optional)
    busy: BusyPin,

    /// Ready input pin (optional)
    ready: ReadyPin,

    /// Reset output pin (optional)
    reset: ResetPin,

    pub(crate) err: Option<Error<SpiError, PinError>>,
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
    Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError> + spi::Transactional<Error = SpiError>,
    CsPin: OutputPin<Error = PinError>,
    Delay: DelayMs<u32>,
{
    /// Create a new wrapper object with the provided SPI and pin
    pub fn new(
        spi: Spi,
        cs: CsPin,
        busy: BusyPin,
        ready: ReadyPin,
        reset: ResetPin,
        delay: Delay,
    ) -> Self {
        Self {
            spi,
            cs,
            delay,
            busy,
            ready,
            reset,
            err: None,
        }
    }

    /// Write to a Pin instance while wrapping and storing the error internally
    /// This returns 0 for success or -1 for errors
    pub fn pin_write<P>(&mut self, pin: &mut P, value: bool) -> i32
    where
        P: OutputPin<Error = PinError>,
    {
        let r = match value {
            true => pin.set_high(),
            false => pin.set_low(),
        };
        match r {
            Ok(_) => 0,
            Err(e) => {
                self.err = Some(Error::Pin(e));
                -1
            }
        }
    }

    /// Write to a Pin instance while wrapping and storing the error internally
    /// This returns 0 for low, 1 for high, and -1 for errors
    pub fn pin_read<P>(&mut self, pin: &mut P) -> i32
    where
        P: InputPin<Error = PinError>,
    {
        let r = pin.is_high();
        match r {
            Ok(true) => 1,
            Ok(false) => 0,
            Err(e) => {
                self.err = Some(Error::Pin(e));
                -1
            }
        }
    }

    /// Check the internal error state of the peripheral
    /// This provides a mechanism to retrieve the rust error if an error occurs
    /// during an FFI call, and clears the internal error state
    pub fn check_error(&mut self) -> Result<(), Error<SpiError, PinError>> {
        match self.err.take() {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// Return hardware resources for reuse
    pub fn free(self) -> (Spi, CsPin, BusyPin, ReadyPin, ResetPin) {
        (self.spi, self.cs, self.busy, self.ready, self.reset)
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Transactional
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    CsPin: OutputPin<Error = PinError>,
    Delay: DelayMs<u32>,
{
    type Error = Error<SpiError, PinError>;

    /// Read data from a specified address
    /// This consumes the provided input data array and returns a reference to this on success
    fn spi_read<'a>(&mut self, prefix: &[u8], mut data: &'a mut [u8]) -> Result<(), Self::Error> {
        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e))?;

        // Write command
        let mut res = self.spi.write(&prefix);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.transfer(&mut data).map(|_r| ());
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Error::Pin(e))?;

        trace!("[spi_read] prefix: {:x?} received: {:x?}", prefix, data);

        // Return result (contains returned data)
        match res {
            Err(e) => Err(Error::Spi(e)),
            Ok(_) => Ok(()),
        }
    }

    /// Write data to a specified register address
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error> {
        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e))?;

        trace!("[spi_write] prefix: {:x?} writing: {:x?}", prefix, data);

        // Write command
        let mut res = self.spi.write(&prefix);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.write(&data);
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Error::Pin(e))?;

        // Return result
        match res {
            Err(e) => Err(Error::Spi(e)),
            Ok(_) => Ok(()),
        }
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Busy
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    BusyPin: InputPin<Error = PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        let v = self.busy.is_high().map_err(|e| Error::Pin(e))?;
        match v {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Ready
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    ReadyPin: InputPin<Error = PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        let v = self.ready.is_high().map_err(|e| Error::Pin(e))?;
        match v {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Reset
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    ResetPin: OutputPin<Error = PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.reset.set_high().map_err(|e| Error::Pin(e)),
            PinState::Low => self.reset.set_low().map_err(|e| Error::Pin(e)),
        }
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> DelayMs<u32>
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Delay: DelayMs<u32>,
{
    /// Set the reset pin state
    fn delay_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms);
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Transfer<u8>
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: Transfer<u8, Error = SpiError>,
{
    type Error = SpiError;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        trace!("[spi::Transfer] writing: {:x?}", &data);
        Transfer::transfer(&mut self.spi, data).map(|r| {
            trace!("[spi::Transfer] read: {:x?}", &r);
            r
        })
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Write<u8>
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: Write<u8, Error = SpiError>,
{
    type Error = SpiError;

    fn write<'w>(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        trace!("[spi::Write] writing: {:x?}", &data);
        Write::write(&mut self.spi, data)
    }
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> spi::Transactional
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: spi::Transactional<Error = SpiError>,
{
    type Error = SpiError;

    fn exec<'a, O>(&mut self, operations: O) -> Result<(), Self::Error>
    where
        O: AsMut<[Operation<'a>]> 
    {
        spi::Transactional::exec(&mut self.spi, operations)
    }
}
