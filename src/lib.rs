//! Embedded SPI helper package
//! This defines a higher level `Transactional` SPI interface, as well as an SPI `Transaction` enumeration
//! that more closely map to the common uses of SPI peripherals.
//! 
//! An `embedded_spi::wrapper::Wrapper` type is provided to wrap existing SPI implementations in this
//! `embedded_spi::Transactional` interface, as well as a set of helpers for C compatibility enabled with
//! the `compat` feature, and a basic mocking adaptor enabled with the `mock` feature.
#![no_std]

pub mod wrapper;

#[cfg(feature = "mock")]
extern crate std;

#[cfg(feature = "mock")]
pub mod mock;

#[cfg(feature = "compat")]
extern crate libc;

#[cfg(feature = "compat")]
pub mod compat;

extern crate embedded_hal;


/// Transaction trait provides higher level, transaction-based, SPI constructs
/// These are executed in a single SPI transaction (without de-asserting CS).
pub trait Transactional {
    type Error;

    /// Read writes the prefix buffer then reads into the input buffer
    /// Note that the values of the input buffer will also be output, because, SPI...
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), Self::Error>;
    
    /// Write writes the prefix buffer then writes the output buffer
    fn spi_write(&mut self, prefix: &[u8], data: &[u8]) -> Result<(), Self::Error>;

    /// Transfer writes the outgoing buffer while reading into the incoming buffer
    /// note that outgoing and incoming must have the same length
    //fn transfer(&mut self, outgoing: &[u8], incoming: &mut [u8]) -> Result<(), Self::Error>;

    /// Exec allows 'Transaction' objects to be chained together into a single transaction
    fn spi_exec(&mut self, transactions: &mut [Transaction]) -> Result<(), Self::Error>;
}

/// Transaction enum defines possible SPI transactions
#[derive(Debug, PartialEq)]
pub enum Transaction<'a> {
    // Write the supplied buffer to the peripheral
    Write(&'a [u8]),
    // Read from the peripheral into the supplied buffer
    Read(&'a mut [u8]),
    // Write the first buffer while reading into the second
    // This behaviour is actually just the same as Read
    //Transfer((&'a [u8], &'a mut [u8]))
}
