//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::spi::{self, Transfer, Write, Operation};
use embedded_hal::digital::{InputPin, OutputPin};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::{Error, Busy, PinState, Ready, Reset, ManagedChipSelect};


/// Wrapper provides a wrapper around an SPI object with Chip Select management
pub struct Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> {
    spi: Spi,

    cs: CsPin,
    reset: ResetPin,

    busy: BusyPin,
    ready: ReadyPin,

    delay: Delay,

    _e: core::marker::PhantomData<Error<SpiError, PinError, DelayError>>,
}

/// ManagedChipSelect indicates wrapper controls CS line
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>ManagedChipSelect for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>{}

impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> 
where 
    Spi: Write<u8, Error=SpiError> + Transfer<u8, Error=SpiError>,
    CsPin: OutputPin<Error=PinError>,
{

    /// Create a new wrapper with the provided chip select pin
    pub fn new(spi: Spi, cs: CsPin, reset: ResetPin, busy: BusyPin, ready: ReadyPin, delay: Delay) -> Self {
        Self{spi, cs, reset, busy, ready, delay, _e: core::marker::PhantomData}
    }

    /// Explicitly fetch the inner spi (non-CS controlling) object
    /// 
    /// (note that deref is also implemented for this)
    pub fn inner_spi(&mut self) -> &mut Spi {
        &mut self.spi
    }
}

impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> spi::Transfer<u8> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>
where 
    Spi: Transfer<u8, Error=SpiError>,
    CsPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError, DelayError>;

    fn try_transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.cs.try_set_low().map_err(Error::Pin)?;
        
        let r = self.spi.try_transfer(data).map_err(Error::Spi);

        self.cs.try_set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Write` implementation managing the CS pin
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> spi::Write<u8> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>
where 
    Spi: Write<u8, Error=SpiError>,
    CsPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError, DelayError>;

    fn try_write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.try_set_low().map_err(Error::Pin)?;
        
        let r = self.spi.try_write(data).map_err(Error::Spi);

        self.cs.try_set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Transactional` implementation managing CS pin
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> spi::Transactional<u8> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>
where
    Spi: spi::Transactional<u8, Error = SpiError>,
    CsPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError, DelayError>;

    fn try_exec<'a>(&mut self, operations: &mut [Operation<'a, u8>]) -> Result<(), Self::Error> {
        self.cs.try_set_low().map_err(Error::Pin)?;

        let r = spi::Transactional::try_exec(&mut self.spi, operations).map_err(Error::Spi);

        self.cs.try_set_high().map_err(Error::Pin)?;

        r
    }
}

/// Reset pin implementation for inner objects implementing `Reset`
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> Reset for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> 
where
    ResetPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError, DelayError>;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.reset.try_set_high().map_err(Error::Pin)?,
            PinState::Low => self.reset.try_set_low().map_err(Error::Pin)?,
        };
        Ok(())
    }
}

/// Busy pin implementation for inner objects implementing `Busy`
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>  Busy for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> 
where
    BusyPin: InputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError, DelayError>;

    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        match self.busy.try_is_high().map_err(Error::Pin)? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

/// Ready pin implementation for inner object implementing `Ready`
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>  Ready for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError> 
where
    ReadyPin: InputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError, DelayError>;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        match self.ready.try_is_high().map_err(Error::Pin)? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>DelayMs<u32> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>
where
    Delay: DelayMs<u32, Error=DelayError>,
{
    type Error = DelayError;

    fn try_delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        self.delay.try_delay_ms(ms)
    }
}


impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>DelayUs<u32> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay, DelayError>
where
    Delay: DelayUs<u32, Error=DelayError>,
{
    type Error = DelayError;

    fn try_delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.delay.try_delay_us(us)
    }
}
