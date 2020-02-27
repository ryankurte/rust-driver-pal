//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::spi::{self, Transfer, Write, Operation};
use embedded_hal::digital::v2::{OutputPin};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::{Busy, PinState, Ready, Reset, ManagedChipSelect};

/// Wrapper provides a wrapper around an SPI object with Chip Select management
pub struct Wrapper<Inner, Cs, E> {
    inner: Inner,
    cs: Cs,

    _e: std::marker::PhantomData<E>,
}

impl <Inner, Cs, E> ManagedChipSelect for Wrapper<Inner, Cs, E> {}

impl <Inner, Cs, E> Wrapper<Inner, Cs, E>  {

    /// Create a new wrapper with the provided chip select pin
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
#[cfg(feature="deref")]
impl <Inner, SpiError, Cs, PinError> core::ops::Deref for Wrapper<Inner, Cs, E> {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
       &self.inner
    } 
}

#[cfg(feature="deref")]
impl <Inner, SpiError, Cs, PinError> core::ops::DerefMut for Wrapper<Inner, Cs, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
     } 
}


impl <Inner, Cs, E, SpiError, PinError> spi::Transfer<u8> for Wrapper<Inner, Cs, E> 
where 
    Inner: Transfer<u8, Error=SpiError>,
    Cs: OutputPin<Error=PinError>,
    SpiError: Into<E>,
    PinError: Into<E>,
{
    type Error = E;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.cs.set_low().map_err(PinError::into)?;
        
        let r = self.inner.transfer(data).map_err(SpiError::into);

        self.cs.set_high().map_err(PinError::into)?;

        r
    }
}

/// `spi::Write` implementation managing the CS pin
impl <Inner, Cs, E, SpiError, PinError> spi::Write<u8> for Wrapper<Inner, Cs, E> 
where 
    Inner: Write<u8, Error=SpiError>,
    Cs: OutputPin<Error=PinError>,
    SpiError: Into<E>,
    PinError: Into<E>,
{
    type Error = E;

    fn write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(PinError::into)?;
        
        let r = self.inner.write(data).map_err(SpiError::into);

        self.cs.set_high().map_err(PinError::into)?;

        r
    }
}

/// `spi::Transactional` implementation managing CS pin
impl <Inner, Cs, E, SpiError, PinError> spi::Transactional<u8> for Wrapper<Inner, Cs, E>
where
    Inner: spi::Transactional<u8, Error = SpiError>,
    Cs: OutputPin<Error=PinError>,
    SpiError: Into<E>,
    PinError: Into<E>,
{
    type Error = E;

    fn exec<'a, O>(&mut self, operations: O) -> Result<(), Self::Error>
    where
        O: AsMut<[Operation<'a, u8>]> 
    {
        self.cs.set_low().map_err(PinError::into)?;

        let r = spi::Transactional::exec(&mut self.inner, operations).map_err(SpiError::into);

        self.cs.set_high().map_err(PinError::into)?;

        r
    }
}

/// Reset pin implementation for inner objects implementing `Reset`
impl <Inner, Cs, E, PinError> Reset for Wrapper<Inner, Cs, E>  
where
    Inner: Reset<Error=PinError>,
    PinError: Into<E>,
{
    type Error = E;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        Reset::set_reset(&mut self.inner, state).map_err(PinError::into)
    }
}

/// Busy pin implementation for inner objects implementing `Busy`
impl <Inner, Cs, E, PinError> Busy for Wrapper<Inner, Cs, E>
where
    Inner: Busy<Error=PinError>,
    PinError: Into<E>,
{
    type Error = E;

    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        Busy::get_busy(&mut self.inner).map_err(PinError::into)
    }
}

/// Ready pin implementation for inner object implementing `Ready`
impl <Inner, Cs, E, PinError> Ready for Wrapper<Inner, Cs, E> 
where
    Inner: Ready<Error=PinError>,
    PinError: Into<E>,
{
    type Error = E;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        Ready::get_ready(&mut self.inner).map_err(PinError::into)
    }
}

impl <Inner, Cs, E> DelayMs<u32> for Wrapper<Inner, Cs, E> 
where
    Inner: DelayMs<u32>,
{
    fn delay_ms(&mut self, _ms: u32) {
        unimplemented!();
    }
}


impl <Inner, Cs, E> DelayUs<u32> for Wrapper<Inner, Cs, E> 
where
    Inner: DelayUs<u32>,
{
    fn delay_us(&mut self, _us: u32) {
        unimplemented!();
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
            Operation::WriteRead(d) => spi.transfer(d).map(|_| ())?,
        }
    }
    Ok(())
}
