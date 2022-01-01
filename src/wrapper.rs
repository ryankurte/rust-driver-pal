//! Transactional SPI wrapper implementation
//! This provides a `Wrapper` type that is generic over an `embedded_hal::blocking::spi`
//! and `embedded_hal::digital::v2::OutputPin` to provide a transactional API for SPI transactions.

use embedded_hal::delay::blocking::DelayUs;
use embedded_hal::digital::blocking::{InputPin, OutputPin};
use embedded_hal::spi::blocking::{self as spi, Operation, Transfer, TransferInplace, Write};

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
    Spi: Write<u8> + Transfer<u8>,
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

impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> TransferInplace<u8>
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::TransferInplace<u8>,
    <Spi as spi::TransferInplace<u8>>::Error: core::fmt::Debug,
    CsPin: OutputPin,
    <CsPin as OutputPin>::Error: core::fmt::Debug,
    Delay: DelayUs,
    <Delay as DelayUs>::Error: core::fmt::Debug,
{
    type Error = Error<
        <Spi as spi::TransferInplace<u8>>::Error,
        <CsPin as OutputPin>::Error,
        <Delay as DelayUs>::Error,
    >;

    fn transfer_inplace<'w>(&mut self, data: &'w mut [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        self.spi.transfer_inplace(data).map_err(Error::Spi)?;

        self.cs.set_high().map_err(Error::Pin)?;

        Ok(())
    }
}

/// `spi::Write` implementation managing the CS pin
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> spi::Write<u8>
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::Write<u8>,
    <Spi as spi::Write<u8>>::Error: core::fmt::Debug,
    CsPin: OutputPin,
    <CsPin as OutputPin>::Error: core::fmt::Debug,
    Delay: DelayUs,
    <Delay as DelayUs>::Error: core::fmt::Debug,
{
    type Error = Error<
        <Spi as spi::Write<u8>>::Error,
        <CsPin as OutputPin>::Error,
        <Delay as DelayUs>::Error,
    >;

    fn write<'w>(&mut self, data: &'w [u8]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = self.spi.write(data).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// `spi::Transactional` implementation managing CS pin
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> spi::Transactional<u8>
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    Spi: spi::Transactional<u8>,
    <Spi as spi::Transactional<u8>>::Error: core::fmt::Debug,
    CsPin: OutputPin,
    <CsPin as OutputPin>::Error: core::fmt::Debug,
    Delay: DelayUs,
    <Delay as DelayUs>::Error: core::fmt::Debug,
{
    type Error = Error<
        <Spi as spi::Transactional<u8>>::Error,
        <CsPin as OutputPin>::Error,
        <Delay as DelayUs>::Error,
    >;

    fn exec<'a>(&mut self, operations: &mut [Operation<'a, u8>]) -> Result<(), Self::Error> {
        self.cs.set_low().map_err(Error::Pin)?;

        let r = spi::Transactional::exec(&mut self.spi, operations).map_err(Error::Spi);

        self.cs.set_high().map_err(Error::Pin)?;

        r
    }
}

/// Reset pin implementation for inner objects implementing `Reset`
impl<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay> Reset
    for Wrapper<Spi, CsPin, BusyPin, ReadyPin, ResetPin, Delay>
where
    ResetPin: OutputPin,
    <ResetPin as OutputPin>::Error: core::fmt::Debug,
{
    type Error = <ResetPin as OutputPin>::Error;

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
    <BusyPin as InputPin>::Error: core::fmt::Debug,
{
    type Error = <BusyPin as InputPin>::Error;

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
    <ReadyPin as InputPin>::Error: core::fmt::Debug,
{
    type Error = <ReadyPin as InputPin>::Error;

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
