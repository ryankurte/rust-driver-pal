//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi` 
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use crate::{Transaction, Transactional, Error};

/// Wrapper wraps an Spi and Pin object to support transactions
#[derive(Debug, Clone, PartialEq)]
pub struct Wrapper<Spi, SpiError, Pin, PinError> {
    spi: Spi,
    cs: Pin,

    pub(crate) err: Option<Error<SpiError, PinError>>,
}

impl <Spi, SpiError, Pin, PinError> Wrapper<Spi, SpiError, Pin, PinError> 
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Pin: OutputPin<Error = PinError>,
{
    pub fn new(spi: Spi, cs: Pin) -> Self {
        Self{spi, cs, err: None}
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

impl <Spi, SpiError, Pin, PinError> Transactional for Wrapper<Spi, SpiError, Pin, PinError> 
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Pin: OutputPin<Error = PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Read data from a specified address
    /// This consumes the provided input data array and returns a reference to this on success
    fn read<'a>(&mut self, prefix: &[u8], mut data: &'a mut [u8]) -> Result<(), Error<SpiError, PinError>> {
        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e) )?;

        // Write command
        let mut res = self.spi.write(&prefix);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.transfer(&mut data).map(|_r| () );
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Error::Pin(e) )?;

        // Return result (contains returned data)
        match res {
            Err(e) => Err(Error::Spi(e)),
            Ok(_) => Ok(()),
        }
    }

    /// Write data to a specified register address
    fn write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error> {
        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e) )?;

        // Write command
        let mut res = self.spi.write(&prefix);

        // Read incoming data
        if res.is_ok() {
            res = self.spi.write(&data);
        }

        // Clear CS
        self.cs.set_high().map_err(|e| Error::Pin(e) )?;

        // Return result
        match res {
            Err(e) => Err(Error::Spi(e)),
            Ok(_) => Ok(()),
        }
    }

    /// Execute the provided transactions
    fn exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error> {
        let mut res = Ok(());

        // Assert CS
        self.cs.set_low().map_err(|e| Error::Pin(e) )?;

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
        self.cs.set_low().map_err(|e| Error::Pin(e) )?;

        res
    }
}
