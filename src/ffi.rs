//! Compatibility shims to allow C use of rust SPI peripherals
//! This module provides mechanisms to convert an abstract `Wrapper` object to and from c void pointers,
//! as well as C ffi compatible spi_read and spi_write functions using these context pointers.

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::{Transactional};
use embedded_hal::digital::v2::OutputPin;

use crate::{PrefixWrite, PrefixRead};
use crate::wrapper::Wrapper;


/// Mark traits as cursed to provide a `Conv` implementation for FFI use
pub trait Cursed {}

/// Conv provides methods to convert rust types to and from c pointers
pub trait Conv {
    /// Generate a C void pointer that can later be re-cast into this object
    fn to_c_ptr(&mut self) -> *mut libc::c_void;
    /// Cast a C void pointer created by to_c_ptr back into this object
    fn from_c_ptr<'a>(ctx: *mut libc::c_void) -> &'a mut Self;
}

impl<T> Conv for T
where
    T: Cursed,
{
    /// Generate a C void pointer that can be re-cast into this object
    fn to_c_ptr(&mut self) -> *mut libc::c_void {
        self as *mut Self as *mut libc::c_void
    }

    /// Cast a C void pointer created by to_c_ptr back into this object
    fn from_c_ptr<'a>(ctx: *mut libc::c_void) -> &'a mut Self {
        unsafe {
            //assert!(ctx == ptr::null());
            let s = ctx as *mut Self;
            &mut *s
        }
    }
}

/// Mark Wrapper as a  c u r s e d  type to allow C coercion
impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay> Cursed
    for Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
{
}

impl<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
    Wrapper<Spi, SpiError, CsPin, BusyPin, ReadyPin, ResetPin, PinError, Delay>
where
    Spi: Transactional<u8, Error = SpiError>,
    CsPin: OutputPin<Error = PinError>,
    Delay: DelayMs<u32>,
{
    /// C FFI compatible spi_write function for dependency injection
    pub extern "C" fn ffi_spi_write(
        ctx: *mut libc::c_void,
        prefix: *mut u8,
        prefix_len: u16,
        data: *mut u8,
        data_len: u16,
    ) -> isize {
        // Coerce back into rust
        let s = Self::from_c_ptr(ctx);

        // Parse buffers
        let prefix: &[u8] = unsafe { core::slice::from_raw_parts(prefix, prefix_len as usize) };
        let data: &[u8] = unsafe { core::slice::from_raw_parts(data, data_len as usize) };

        // Execute command and handle errors
        match s.spi_write(&prefix, &data) {
            Ok(_) => 0,
            Err(_e) => {
                // TODO: removed this from wrapper
                //s.err = Some(e);
                -1
            }
        }
    }

    /// C FFI compatible spi_read function for dependency injection
    pub extern "C" fn ffi_spi_read(
        ctx: *mut libc::c_void,
        prefix: *mut u8,
        prefix_len: u16,
        data: *mut u8,
        data_len: u16,
    ) -> isize {
        // Coerce back into rust
        let s = Self::from_c_ptr(ctx);

        // Parse buffers
        let prefix: &[u8] = unsafe { core::slice::from_raw_parts(prefix, prefix_len as usize) };
        let mut data: &mut [u8] =
            unsafe { core::slice::from_raw_parts_mut(data, data_len as usize) };

        // Execute command and handle errors
        match s.spi_read(&prefix, &mut data) {
            Ok(_) => 0,
            Err(_e) => {
                // TODO: removed this from wrapper
                //s.err = Some(e);
                -1
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Something(bool);
    impl Cursed for Something {}

    #[test]
    fn test_compat() {
        let mut s = Something(true);
        let p = s.to_c_ptr();
        let r = Something::from_c_ptr(p);

        assert_eq!(&s, r);
    }
}
