# driver-pal

Previously known as `driver-pal`, new releases at [crates.io/crates/driver-pal](https://crates.io/crates/driver-pal). 
A helper package for rust-embedded driver traits and implementations to assist with constructing drivers for embedded devices.


This provides:

- a `CS` pin trait to communicate CS control for drivers
- a `Wrapper` type to provide this for an SPI and OutputPin implementation
- a `Hal` that abstracts over a number of SPI implementations to assist with writing driver utilities
- a `Mock` helper for testing drivers based on this
- a set of compatibility shims for c FFI use with dependency injected drivers


## Status

[![GitHub tag](https://img.shields.io/github/tag/ryankurte/rust-driver-pal.svg)](https://github.com/ryankurte/rust-driver-pal)
[![Build Status](https://travis-ci.com/ryankurte/rust-driver-pal.svg?branch=master)](https://travis-ci.com/ryankurte/rust-driver-pal)
[![Crates.io](https://img.shields.io/crates/v/driver-pal.svg)](https://crates.io/crates/driver-pal)
[![Docs.rs](https://docs.rs/driver-pal/badge.svg)](https://docs.rs/driver-pal)

[Open Issues](https://github.com/ryankurte/rust-driver-pal/issues)


Currently patched-to-heck waiting on `embedded-hal` version `v1.0.0-alpha.3` with transactional SPI, and a bunch of
downstream patches that depend on this.

