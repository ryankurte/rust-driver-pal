//! Embedded SPI helper and testing package
//! This is intended to try out some possible trait improvements prior to proposing chanegs
//! to embedded-hal

extern crate embedded_hal;

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

/// Transaction trait provides higher level, transaction-based, SPI constructs
/// These are executed in a single SPI transaction (without de-asserting CS).
trait Transactional {
    type Error;

    /// Read writes the prefix buffer then reads into the input buffer 
    fn read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Self::Error>;
    /// Write writes the prefix buffer then writes the output buffer
    fn write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error>;
    /// Exec allows 'Transaction' objects to be chained together into a single transaction
    fn exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error>;
}

/// Convenience error type combining SPI and Pin errors
#[derive(Debug, Clone, PartialEq)]
pub enum Error<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
    Aborted,
}

//#[derive(Debug, PartialEq)]
pub enum Transaction<'a> {
    Write(&'a [u8]),
    Read(&'a mut [u8]),
}

/// TransactionalSpi wraps an Spi and Pin object to support transactions
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionalSpi<Spi, Pin> {
    spi: Spi,
    cs: Pin,
}

impl <Spi, SpiError, Pin, PinError> TransactionalSpi<Spi, Pin> 
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Pin: OutputPin<Error = PinError>,
{
    pub fn new(spi: Spi, cs: Pin) -> Self {
        Self{spi, cs}
    }
}

impl <Spi, SpiError, Pin, PinError> Transactional for TransactionalSpi<Spi, Pin> 
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
