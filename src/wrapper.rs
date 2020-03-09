//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::blocking::spi::{self, Transfer, Write, Operation};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use embedded_hal::blocking::delay::{DelayMs, DelayUs};

use crate::{Error, Busy, PinState, Ready, Reset, ManagedChipSelect};


/// Wrapper provides a wrapper around an SPI object with Chip Select management
pub struct Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> {
    spi: Spi,

    cs: CsPin,
    reset: ResetPin,

    busy: BusyPin,
    ready: ReadyPin,

    delay: Delay,

    _e: core::marker::PhantomData<Error<SpiError, PinError>>,
}

/// ManagedChipSelect indicates wrapper controls CS line
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>ManagedChipSelect for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>{}

impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> 
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

impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> spi::Transfer<u8> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where 
    Spi: Transfer<u8, Error=SpiError>,
    CsPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    fn transfer<'w>(&mut self, data: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;
        
        let r = self.spi.transfer(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Write` implementation managing the CS pin
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> spi::Write<u8> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where 
    Spi: Write<u8, Error=SpiError>,
    CsPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    fn write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;
        
        let r = self.spi.write(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Transactional` implementation managing CS pin
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> spi::Transactional<u8> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: spi::Transactional<u8, Error = SpiError>,
    CsPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    fn exec<'a>(&mut self, operations: &mut [spi::Operation<'a, u8>]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = spi::Transactional::exec(&mut self.spi, operations).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// Reset pin implementation for inner objects implementing `Reset`
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Reset for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> 
where
    ResetPin: OutputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.reset.set_high().map_err(Error::Pin)?,
            PinState::Low => self.reset.set_low().map_err(Error::Pin)?,
        };
        Ok(())
    }
}

/// Busy pin implementation for inner objects implementing `Busy`
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>  Busy for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> 
where
    BusyPin: InputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        match self.busy.is_high().map_err(Error::Pin)? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

/// Ready pin implementation for inner object implementing `Ready`
impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>  Ready for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> 
where
    ReadyPin: InputPin<Error=PinError>,
{
    type Error = Error<SpiError, PinError>;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        match self.ready.is_high().map_err(Error::Pin)? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>DelayMs<u32> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Delay: DelayMs<u32>,
{
    fn delay_ms(&mut self, ms: u32) {
        self.delay.delay_ms(ms);
    }
}


impl <Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>DelayUs<u32> for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Delay: DelayUs<u32>,
{
    fn delay_us(&mut self, us: u32) {
        self.delay.delay_us(us);
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
