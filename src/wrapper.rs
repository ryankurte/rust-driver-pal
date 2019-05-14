//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi` 
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::{OutputPin, InputPin};

use crate::{Transaction, Transactional, PinState, Error};

/// Wrapper wraps an Spi and Pin object to support transactions
#[derive(Debug, Clone, PartialEq)]
pub struct Wrapper<Spi, SpiError, OutputPin, InputPin, PinError> {
    spi: Spi,
    cs: OutputPin,

    busy: Option<InputPin>,

    pub(crate) err: Option<Error<SpiError, PinError>>,
}

impl <Spi, SpiError, Output, Input, PinError> Wrapper<Spi, SpiError, Output, Input, PinError>  
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
{
    /// Create a new wrapper object with the provided SPI and pin
    pub fn new(spi: Spi, cs: Output) -> Self {
        Self{spi, cs, err: None, busy: None}
    }

    /// Add a busy input to the wrapper object
    pub fn with_busy(&mut self, busy: Input) {
        self.busy = Some(busy);
    }

    /// Write to a Pin instance while wrapping and storing the error internally
    /// This returns 0 for success or -1 for errors
    pub fn pin_write<P>(&mut self, pin: &mut P, value: bool) -> i32 
    where P: OutputPin<Error = PinError>
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
    where P: InputPin<Error = PinError>
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
            None => Ok(())
        }
    }
}

impl <Spi, SpiError, Output, Input, PinError> Transactional for Wrapper<Spi, SpiError, Output, Input, PinError>  
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Read data from a specified address
    /// This consumes the provided input data array and returns a reference to this on success
    fn spi_read<'a>(&mut self, prefix: &[u8], mut data: &'a mut [u8]) -> Result<(), Self::Error> {
        // Assert CS
        self.cs.set_low().map_err(|e| { Error::Pin(e) })?;

        // Write command
        let mut res = self.spi.write(&prefix);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.transfer(&mut data).map(|_r| () );
        }

        // Clear CS
        self.cs.set_high().map_err(|e| { Error::Pin(e) })?;

        // Return result (contains returned data)
        match res {
            Err(e) => Err( Error::Spi(e) ),
            Ok(_) => Ok(()),
        }
    }

    /// Write data to a specified register address
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error> {
        // Assert CS
        self.cs.set_low().map_err(|e| { Error::Pin(e) })?;

        // Write command
        let mut res = self.spi.write(&prefix);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.write(&data);
        }

        // Clear CS
        self.cs.set_high().map_err(|e| { Error::Pin(e) })?;

        // Return result
        match res {
            Err(e) => Err( Error::Spi(e) ),
            Ok(_) => Ok(()),
        }
    }

    /// Execute the provided transactions
    fn spi_exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error> {
        let mut res = Ok(());

        // Assert CS
        self.cs.set_low().map_err(|e| { Error::Pin(e) })?;

        for i in 0..transactions.len() {
            let mut t = &mut transactions[i];

            res = match &mut t {
                Transaction::Write(d) => self.spi.write(d),
                Transaction::Read(d) =>  self.spi.transfer(d).map(|_r| () ),
            }.map_err(|e| Error::Spi(e) );

            if res.is_err() {
                break;
            }
        }

        // Assert CS
        self.cs.set_low().map_err(|e| { Error::Pin(e) })?;

        res
    }

    /// Check if the peripheral is currently busy
    /// This will panic if not busy pin is bound
    fn spi_busy(&mut self) -> Result<PinState, Self::Error> {
        match &self.busy {
            // TODO: should this be an error?
            None => Ok(PinState::Low),
            Some(b) => {
                let v = b.is_high().map_err(|e| Error::Pin(e) )?;
                match v {
                    true => Ok(PinState::High),
                    false => Ok(PinState::Low),
                }
            }
        }
    }
}

impl <Spi, SpiError, Output, Input, PinError> Transfer<u8> for Wrapper<Spi, SpiError, Output, Input, PinError>  
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
    
{
    type Error = SpiError;

    fn transfer<'w>(&mut self, buffer: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Transfer::transfer(&mut self.spi, buffer)
    }
}

impl <Spi, SpiError, Output, Input, PinError> Write<u8> for Wrapper<Spi, SpiError, Output, Input, PinError>  
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Output: OutputPin<Error = PinError>,
    Input: InputPin<Error = PinError>,
{
    type Error = SpiError;
    
    fn write<'w>(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        Write::write(&mut self.spi, buffer)
    }
}
