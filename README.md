# embedded-spi

A helper / testing package for rust-embedded SPI traits and implementations, to try out some more interesting approaches prior to proposing additions to embedded-hal.
This provides a Transactional SPI interface, as well as a `Wrapper` type to provide this for an SPI and OutputPin implementation, a `Mock` helper for testing drivers based on this, and a set of compatibility shims for c FFI use with dependency injected drivers.

## Status

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/rust-embedded-spi.svg)](https://github.com/ryankurte/rust-embedded-spi)
[![Build Status](https://travis-ci.com/ryankurte/rust-embedded-spi.svg?branch=master)](https://travis-ci.com/ryankurte/rust-embedded-spi)
[![Crates.io](https://img.shields.io/crates/v/embedded-spi.svg)](https://crates.io/crates/embedded-spi)
[![Docs.rs](https://docs.rs/embedded-spi/badge.svg)](https://docs.rs/embedded-spi)

[Open Issues](https://github.com/ryankurte/rust-embedded-spi/issues)

