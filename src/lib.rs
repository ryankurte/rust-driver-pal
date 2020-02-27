//! Embedded SPI helper package
//! This defines a higher level `Transactional` SPI interface, as well as an SPI `Transaction` enumeration
//! that more closely map to the common uses of SPI peripherals, as well as some other common driver helpers.
//!
//! An `embedded_spi::wrapper::Wrapper` type is provided to wrap existing SPI implementations in this
//! `embedded_spi::Transactional` interface, as well as a set of helpers for C compatibility enabled with
//! the `compat` feature, and a basic mocking adaptor enabled with the `mock` feature.


#![cfg_attr(not(feature = "utils"), no_std)]

#[macro_use]
extern crate log;

extern crate embedded_hal;

pub mod wrapper;

#[cfg(feature = "mock")]
extern crate std;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "ffi")]
extern crate libc;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "utils")]
extern crate serde;

#[cfg(feature = "utils")]
extern crate toml;

#[cfg(feature = "utils")]
extern crate simplelog;

#[cfg(feature = "hal-linux")]
extern crate linux_embedded_hal;

#[cfg(feature = "hal-cp2130")]
extern crate driver_cp2130;

#[cfg(feature = "utils")]
pub mod utils;

pub mod hal;

use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::blocking::spi;

/// CSManaged marker trait indicates CS is managed by the drivert
pub trait CSManaged {}

/// HAL trait abstracts required functions for SPI peripherals
pub trait Hal<SpiError, PinError>: 
    spi::Write<u8, Error=SpiError> + 
    spi::Transfer<u8, Error=SpiError> + 
    Busy<Error=PinError> + 
    Ready<Error=PinError> + 
    Reset<Error=PinError> + 
    DelayMs<u32> + 
    DelayUs<u32> {}

/// Transaction trait provides higher level, transaction-based, SPI constructs
/// These are executed in a single SPI transaction (without de-asserting CS).
pub trait Transactional {
    type Error;

    /// Read writes the prefix buffer then reads into the input buffer
    /// Note that the values of the input buffer will also be output, because, SPI...
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Self::Error>;

    /// Write writes the prefix buffer then writes the output buffer
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error>;
}

/// Transaction enum defines possible SPI transactions
pub type Transaction<'a> = embedded_hal::blocking::spi::Operation<'a, u8>;

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

