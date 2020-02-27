//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use core::ops::{Deref, DerefMut};

use embedded_hal::blocking::spi::{self, Transfer, Write, Operation};
use embedded_hal::digital::v2::{OutputPin};

use crate::{Busy, Error, PinState, Ready, Reset};

/// Wrapper provides a wrapper around an SPI object with Chip Select management
pub struct Wrapper<Inner, SpiError, Cs, PinError> {
    inner: Inner,
    cs: Cs,

    _e: std::marker::PhantomData<Error<SpiError, PinError>>,
}

impl <Inner, SpiError, Cs, PinError> crate::CSManaged for Wrapper<Inner, SpiError, Cs, PinError> {}

impl <Inner, SpiError, Cs, PinError> Wrapper<Inner, SpiError, Cs, PinError>  {
    pub fn new(inner: Inner, cs: Cs) -> Self {
        Self{inner, cs, _e: std::marker::PhantomData}
    }

    /// Explicitly fetch the inner (non-CS controlling) object
    /// 
    /// (note that deref is also implemented for this)
    pub fn inner(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

/// Derefs to allow accessing methods on inner
impl <Inner, SpiError, Cs, PinError> Deref for Wrapper<Inner, SpiError, Cs, PinError> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
       &self.inner
    } 
}

impl <Inner, SpiError, Cs, PinError> DerefMut for Wrapper<Inner, SpiError, Cs, PinError> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
     } 
}


impl <Inner, SpiError, Cs, PinError> spi::Transfer<u8> for Wrapper<Inner, SpiError, Cs, PinError> 
where 
    Inner: Transfer<u8, Error=SpiError>,
    Cs: OutputPin<Error=PinError>
{
    type Error = Error<SpiError, PinError>;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;
        
        let r = self.inner.transfer(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Write` implementation managing the CS pin
impl <Inner, SpiError, Cs, PinError> spi::Write<u8> for Wrapper<Inner, SpiError, Cs, PinError> 
where 
    Inner: Write<u8, Error=SpiError>,
    Cs: OutputPin<Error=PinError>
{
    type Error = Error<SpiError, PinError>;

    fn write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;
        
        let r = self.inner.write(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Transactional` implementation managing CS pin
impl<Inner, SpiError, Cs, PinError> spi::Transactional<u8> for Wrapper<Inner, SpiError, Cs, PinError>
where
    Inner: spi::Transactional<u8, Error = SpiError>,
    Cs: OutputPin<Error=PinError>
{
    type Error = Error<SpiError, PinError>;

    fn exec<'a, O>(&mut self, operations: O) -> Result<(), Self::Error>
    where
        O: AsMut<[Operation<'a, u8>]> 
    {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = spi::Transactional::exec(&mut self.inner, operations).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// Reset pin implementation for inner objects implementing `Reset`
impl <Inner, SpiError, Cs, PinError> Reset for Wrapper<Inner, SpiError, Cs, PinError>  
where
    Inner: Reset<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        Reset::set_reset(&mut self.inner, state);

        Ok(())
    }
}

/// Busy pin implementation for inner objects implementing `Busy`
impl <Inner, SpiError, Cs, PinError> Busy for Wrapper<Inner, SpiError, Cs, PinError>
where
    Inner: Busy<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        Busy::get_busy(&mut self.inner).map_err(Error::Pin)
    }
}

/// Ready pin implementation for inner object implementing `Ready`
impl <Inner, SpiError, Cs, PinError> Ready for Wrapper<Inner, SpiError, Cs, PinError> 
where
    Inner: Ready<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        Ready::get_ready(&mut self.inner).map_err(Error::Pin)
    }
}


/// Helper to execute transactions over a non-transactional SPI device
pub fn spi_exec<'a, Spi, SpiError, Operations>(spi: &mut Spi, mut operations: Operations) -> Result<(), SpiError> where
    Spi: spi::Transfer<u8, Error = SpiError> + spi::Write<u8, Error = SpiError> +,
    Operations: AsMut<[Operation<'a, u8>]> 
{
    let o = operations.as_mut();

    for i in 0..o.len() {
        let mut t = &mut o[i];

        match &mut t {
            Operation::Write(d) => spi.write(d)?,
            Operation::WriteRead(d_out, d_in) => {
                // Write output data to mutable input vec
                d_in.copy_from_slice(d_out);
                // Execute transfer
                spi.transfer(d_in)?;
            },
        }

    }
    Ok(())
}
