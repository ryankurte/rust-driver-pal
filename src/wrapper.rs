//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::delay::blocking::DelayUs;
use embedded_hal::digital::blocking::{InputPin, OutputPin};
use embedded_hal::spi::blocking::{self as spi, SpiBus, SpiBusWrite};

use crate::{Busy, Error, ManagedChipSelect, PinState, Ready, Reset};

/// Wrapper provides a wrapper around an SPI object with Chip Select management
pub struct Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> {
    spi: Spi,

    cs: CsPin,
    reset: ResetPin,

    busy: BusyPin,
    ready: ReadyPin,

    delay: Delay,
}

/// ManagedChipSelect indicates wrapper controls CS line
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> ManagedChipSelect
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
{
}

impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
    Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: SpiBusWrite<u8> + SpiBus<u8>,
    CsPin: OutputPin,
{
    /// Create a new wrapper with the provided chip select pin
    pub fn new(
        spi: Spi,
        cs: CsPin,
        reset: ResetPin,
        busy: BusyPin,
        ready: ReadyPin,
        delay: Delay,
    ) -> Self {
        Self {
            spi,
            cs,
            reset,
            busy,
            ready,
            delay,
        }
    }

    /// Explicitly fetch the inner spi (non-CS controlling) object
    ///
    /// (note that deref is also implemented for this)
    pub fn inner_spi(&mut self) -> &mut Spi {
        &mut self.spi
    }
}

impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> embedded_hal::spi::ErrorType
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> 
where
    Spi: embedded_hal::spi::ErrorType,
    CsPin: embedded_hal::digital::ErrorType,
    Delay: DelayUs,
    {

    type Error = Error<
        <Spi as embedded_hal::spi::ErrorType>::Error,
        <CsPin as embedded_hal::digital::ErrorType>::Error,
        <Delay as DelayUs>::Error,
    >;
}

impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> embedded_hal::digital::ErrorType
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> 
where
    Spi: embedded_hal::spi::ErrorType,
    CsPin: embedded_hal::digital::ErrorType,
    Delay: DelayUs,
    {

    type Error = Error<
        <Spi as embedded_hal::spi::ErrorType>::Error,
        <CsPin as embedded_hal::digital::ErrorType>::Error,
        <Delay as DelayUs>::Error,
    >;
}

impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> SpiBus<u8>
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::SpiBus<u8>,
    CsPin: OutputPin,
    Delay: DelayUs,
{
    fn transfer_in_place<'w>(&mut self, data: &'w mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        self.spi.transfer_in_place(data).map_err(Error::Spi)?;

        self.cs.set_high().map_err(Error::Pin)?;

        Ok(())
    }

    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        self.spi.transfer(read, write).map_err(Error::Spi)?;

        self.cs.set_high().map_err(Error::Pin)?;

        Ok(())
    }
}

/// `spi::Read` implementation managing the CS pin
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> spi::SpiBusFlush
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::SpiBusFlush,
    CsPin: OutputPin,
    Delay: DelayUs,
{
    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// `spi::Read` implementation managing the CS pin
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> spi::SpiBusRead<u8>
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::SpiBusRead<u8>,
    CsPin: OutputPin,
    Delay: DelayUs,
{
    fn read<'w>(&mut self, data: &'w mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = self.spi.read(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Write` implementation managing the CS pin
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> spi::SpiBusWrite<u8>
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::SpiBusWrite<u8>,
    CsPin: OutputPin,
    Delay: DelayUs,
{
    fn write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = self.spi.write(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// Reset pin implementation for inner objects implementing `Reset`
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> Reset
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    ResetPin: OutputPin,
{
    type Error = <ResetPin as embedded_hal::digital::ErrorType>::Error;

    /// Set the reset pin state
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.reset.set_high()?,
            PinState::Low => self.reset.set_low()?,
        };
        Ok(())
    }
}

/// Busy pin implementation for inner objects implementing `Busy`
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> Busy
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    BusyPin: InputPin,
{
    type Error = <BusyPin as embedded_hal::digital::ErrorType>::Error;
    
    /// Fetch the busy pin state
    fn get_busy(&mut self) -> Result<PinState, Self::Error> {
        match self.busy.is_high()? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

/// Ready pin implementation for inner object implementing `Ready`
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> Ready
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    ReadyPin: InputPin,
{
    type Error = <ReadyPin as embedded_hal::digital::ErrorType>::Error;

    /// Fetch the ready pin state
    fn get_ready(&mut self) -> Result<PinState, Self::Error> {
        match self.ready.is_high()? {
            true => Ok(PinState::High),
            false => Ok(PinState::Low),
        }
    }
}

impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> DelayUs
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Delay: DelayUs,
    <Delay as DelayUs>::Error: core::fmt::Debug,
{
    type Error = <Delay as DelayUs>::Error;

    fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        self.delay.delay_us(us)
    }
}
