//! Embedded SPI helper package
//! This defines a higher level `Transactional` SPI interface, as well as an SPI `Transaction` enumeration
//! that more closely map to the common uses of SPI peripherals, as well as some other common driver helpers.
//!
//! An `embedded_spi::wrapper::Wrapper` type is provided to wrap existing SPI implementations in this
//! `embedded_spi::Transactional` interface, as well as a set of helpers for C compatibility enabled with
//! the `compat` feature, and a basic mocking adaptor enabled with the `mock` feature.


#![cfg_attr(not(feature = "hal"), no_std)]

#[macro_use]
extern crate log;

extern crate embedded_hal;

#[cfg(feature = "mock")]
extern crate std;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "ffi")]
extern crate libc;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "serde")]
extern crate serde;

#[cfg(feature = "toml")]
extern crate toml;

#[cfg(feature = "simplelog")]
extern crate simplelog;

#[cfg(feature = "hal-linux")]
extern crate linux_embedded_hal;

#[cfg(feature = "hal-cp2130")]
extern crate driver_cp2130;

#[cfg(feature = "hal")]
pub mod hal;


pub mod wrapper;


use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::spi;

/// ManagedChipSelect marker trait indicates CS is managed by the drivert
pub trait ManagedChipSelect {}

/// HAL trait abstracts commonly required functions for SPI peripherals
pub trait Hal<E>:
    PrefixWrite<Error=E> +
    PrefixRead<Error=E> +

    spi::Write<u8, Error=E> +
    spi::Transfer<u8, Error=E> +
    
    Busy<Error=E> + 
    Ready<Error=E> + 
    Reset<Error=E> + 
    
    DelayMs<u32> + 
    DelayUs<u32> {}

/// Default HAL trait impl over component traits
impl <T, E> Hal<E> for T where T: 
    PrefixWrite<Error=E> +
    PrefixRead<Error=E> +

    spi::Write<u8, Error=E> +
    spi::Transfer<u8, Error=E> + 
    
    Busy<Error=E> + 
    Ready<Error=E> + 
    Reset<Error=E> + 
    
    DelayMs<u32> + 
    DelayUs<u32> {}

/// PrefixRead trait provides a higher level, write then read function
pub trait PrefixRead {
    type Error;

    /// Read writes the prefix buffer then reads into the input buffer
    /// Note that the values of the input buffer will also be output, because, SPI...
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Self::Error>;
}

/// PrefixWrite trait provides higher level, writye then write function
pub trait PrefixWrite {
    type Error;

    /// Write writes the prefix buffer then writes the output buffer
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error>;
}

/// Transaction enum defines possible SPI transactions
pub type Transaction<'a> = embedded_hal::blocking::spi::Operation<'a, u8>;

/// Chip Select trait for peripherals supporting manual chip select
pub trait ChipSelect {
    type Error;

    /// Set the cs pin state if available
    fn set_cs(&mut self, state: PinState) -> Result<(), Self::Error>;
}

/// Busy trait for peripherals that support a busy signal
pub trait Busy {
    type Error;

    /// Returns the busy pin state if bound
    fn get_busy(&mut self) -> Result<PinState, Self::Error>;
}

/// Reset trait for peripherals that have a reset or shutdown pin
pub trait Reset {
    type Error;

    /// Set the reset pin state if available
    fn set_reset(&mut self, state: PinState) -> Result<(), Self::Error>;
}


/// Ready trait for peripherals that support a ready signal (or IRQ)
pub trait Ready {
    type Error;

    /// Returns the busy pin state if bound
    fn get_ready(&mut self) -> Result<PinState, Self::Error>;
}

/// Error type combining SPI and Pin errors for utility
#[derive(Debug, Clone, PartialEq)]
pub enum Error<SpiError, PinError> {
    Spi(SpiError),
    Pin(PinError),
    Aborted,
}

/// PinState enum used for busy indication
#[derive(Debug, Clone, PartialEq)]
pub enum PinState {
    Low,
    High,
}

/// Automatic `embedded_spi::PrefixWrite` implementation for objects implementing `embedded_hal::blocking::spi::Transactional`.
impl <T, E> PrefixWrite for T 
where
    T: spi::Transactional<u8, Error=E>, 
{
    type Error = E;

    /// Write data with the specified prefix
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error> {
        let mut ops = [
            spi::Operation::Write(prefix),
            spi::Operation::Write(data),
        ];

        self.exec(&mut ops)?;
        
        Ok(())
    }
}

/// Automatic `embedded_spi::PrefixRead` implementation for objects implementing `embedded_hal::blocking::spi::Transactional`.
impl <T, E> PrefixRead for T 
where
    T: spi::Transactional<u8, Error=E>, 
{
    type Error = E;

    /// Read data with the specified prefix
    fn spi_read<'a>(&mut self, prefix: &[u8], data: &'a mut [u8]) -> Result<(), Self::Error> {
        let mut ops = [
            spi::Operation::Write(prefix),
            spi::Operation::Transfer(data),
        ];

        self.exec(&mut ops)?;
        
        Ok(())
    }
}