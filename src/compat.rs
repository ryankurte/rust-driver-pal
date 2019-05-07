//! Compatibility shims to allow C use of rust SPI peripherals
//! This module provides mechanisms to convert an abstract `Wrapper` object to and from c void pointers,
//! as well as C ffi compatible spi_read and spi_write functions using these context pointers.

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

use crate::{Transactional};
use crate::wrapper::Wrapper;

impl <Spi, SpiError, Pin, PinError> Wrapper<Spi, SpiError, Pin, PinError> 
where
    Spi: Transfer<u8, Error = SpiError> + Write<u8, Error = SpiError>,
    Pin: OutputPin<Error = PinError>,
{
    /// Generate a C void pointer that can be re-cast into this object
    pub fn to_c_ptr(&mut self) -> *mut libc::c_void {
        self as *mut Self as *mut libc::c_void
    }

    /// Cast a C void pointer created by to_c_ptr back into this object
    pub(crate) fn from_c<'a>(ctx: *mut libc::c_void) -> &'a mut Self {
        unsafe {
            //assert!(ctx == ptr::null());
            let s = ctx as *mut Self;
            &mut *s
        }
    }

    /// C FFI compatible spi_write function for dependency injection
    pub extern fn spi_write(ctx: *mut libc::c_void, prefix: *mut u8, prefix_len: u16, data: *mut u8, data_len: u16) -> isize {
        // Coerce back into rust
        let s = Self::from_c(ctx);

        // Parse buffers
        let prefix: &[u8] = unsafe { core::slice::from_raw_parts(prefix, prefix_len as usize) };
        let data: &[u8] = unsafe { core::slice::from_raw_parts(data, data_len as usize) };

        // Execute command and handle errors
        match s.write(&prefix, &data) {
            Ok(_) => 0,
            Err(e) => {
                s.err = Some(e);
                -1
            },
        }
    }

    /// C FFI compatible spi_read function for dependency injection
    pub extern fn spi_read(ctx: *mut libc::c_void, prefix: *mut u8, prefix_len: u16, data: *mut u8, data_len: u16) -> isize {
        // Coerce back into rust
        let s = Self::from_c(ctx);

        // Parse buffers
        let prefix: &[u8] = unsafe { core::slice::from_raw_parts(prefix, prefix_len as usize) };
        let mut data: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(data, data_len as usize) };

        // Execute command and handle errors
        match s.read(&prefix, &mut data) {
            Ok(_) => 0,
            Err(e) => {
                s.err = Some(e);
                -1
            },
        }
    }
}

#[cfg(test)]
mod test {
    // TODO: test for this  c u r s e d  thing require a viable mock Spi and Pin impl
    // which are not yet part of embedded-hal-mock
}
