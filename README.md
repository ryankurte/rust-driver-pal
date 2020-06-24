# spi-hal

Previously known as `embedded-spi`, new releases at [crates.io/crates/spi-hal](https://crates.io/crates/spi-hal). 
A helper package for rust-embedded SPI traits and implementations, including testing approactes prior to proposing additions to embedded-hal.


This provides:

- A Transactional SPI interface (https://github.com/rust-embedded/embedded-hal/pull/191)
- A `CS` pin trait to communicate CS control for drivers
- a `Wrapper` type to provide this for an SPI and OutputPin implementation
- a `Hal` that abstracts over a number of SPI implementations to assist with writing driver utilities
- a `Mock` helper for testing drivers based on this
- a set of compatibility shims for c FFI use with dependency injected drivers


## Status

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/rust-embedded-spi.svg)](https://github.com/ryankurte/rust-embedded-spi)
[![Build Status](https://travis-ci.com/ryankurte/rust-embedded-spi.svg?branch=master)](https://travis-ci.com/ryankurte/rust-embedded-spi)
[![Crates.io](https://img.shields.io/crates/v/embedded-spi.svg)](https://crates.io/crates/embedded-spi)
[![Docs.rs](https://docs.rs/embedded-spi/badge.svg)](https://docs.rs/embedded-spi)

[Open Issues](https://github.com/ryankurte/rust-embedded-spi/issues)

