//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::{InputPin, OutputPin};

use crate::{Busy, Error, PinState, Ready, Reset, Transaction, Transactional};

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
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
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

    /// Execute the provided transactions
    fn spi_exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error> {
        let mut res = Ok(());

        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e))?;

        for i in 0..transactions.len() {
            let mut t = &mut transactions[i];

            res = match &mut t {
                Transaction::Write(d) => self.spi.write(d),
                Transaction::Read(d) => self.spi.transfer(d).map(|_r| ()),
            }
            .map_err(|e| Error::Spi(e));

            if res.is_err() {
                break;
            }
        }

        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e))?;

        res
    }
}

use embedded_hal::blocking::spi;

pub struct H<Spi, SpiError, Cs, CsError> {
    inner: Spi,
    cs: Cs,

    _e: std::marker::PhantomData<Error<SpiError, CsError>>,
}

impl <Spi, SpiError, Cs, CsError> crate::CSManaged for H<Spi, SpiError, Cs, CsError> {}

impl <Spi, SpiError, Cs, CsError> H<Spi, SpiError, Cs, CsError>  {
    pub fn new(inner: Spi, cs: Cs) -> Self {
        Self{inner, cs, _e: std::marker::PhantomData}
    }

    /// Fetch the inner (non-CS controlling) object
    pub fn inner(&mut self) -> &mut Spi {
        &mut self.inner
    }
}

use core::ops::{Deref, DerefMut};

impl <Spi, SpiError, Cs, CsError> Deref for H<Spi, SpiError, Cs, CsError> {
    type Target = Spi;

    fn deref(&self) -> &Self::Target {
       &self.inner
    } 
}

impl <Spi, SpiError, Cs, CsError> DerefMut for H<Spi, SpiError, Cs, CsError> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
     } 
}

impl <Spi, SpiError, Cs, CsError> spi::Transfer<u8> for H<Spi, SpiError, Cs, CsError> 
where 
    Spi: Transfer<u8, Error=SpiError>,
    Cs: OutputPin<Error=CsError>
{
    type Error = Error<SpiError, CsError>;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;
        
        let r = self.inner.transfer(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

impl <Spi, SpiError, Cs, CsError> spi::Write<u8> for H<Spi, SpiError, Cs, CsError> 
where 
    Spi: Write<u8, Error=SpiError>,
    Cs: OutputPin<Error=CsError>
{
    type Error = Error<SpiError, CsError>;

    fn write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;
        
        let r = self.inner.write(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}


/// Helper to execute transactions over a non-transactional SPI device
fn spi_exec<Spi, SpiError>(spi: &mut Spi, transactions: &mut [Transaction]) -> Result<(), SpiError> where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
{
    for i in 0..transactions.len() {
        let mut t = &mut transactions[i];

        match &mut t {
            Transaction::Write(d) => spi.write(d)?,
            Transaction::Read(d) => spi.transfer(d).map(|_r| ())?,
        }

    }
    Ok(())
}

/// Helper to execute transactions over a non-transactional SPI device with CS
fn spi_exec_cs<Spi, SpiError, Pin, PinError>(spi: &mut Spi, cs: &mut Pin, transactions: &mut [Transaction]) -> Result<(), Error<SpiError, PinError>> where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Pin: OutputPin<Error = PinError>,
{
    // Assert CS
    cs.set_low().map_err(|e| Error::Pin(e))?;

    // Run transactions
    let res = spi_exec(spi, transactions).map_err(Error::Spi);

    // Assert CS
    cs.set_low().map_err(|e| Error::Pin(e))?;

    res
}
