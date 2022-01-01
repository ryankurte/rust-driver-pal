/// Error type combining SPI and Pin errors for utility
#[derive(Debug)]
pub enum HalError {
    InvalidConfig,
    InvalidSpiMode,
    NoPin,

    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Error),

    #[cfg(feature = "hal-linux")]
    Io(std::io::ErrorKind),

    #[cfg(feature = "hal-linux")]
    Sysfs(linux_embedded_hal::sysfs_gpio::Error),
}

impl HalError {
    /// Check whether the HalError contains an underlying error type
    pub fn is_inner(&self) -> bool {
        use HalError::*;

        match self {
            InvalidConfig | InvalidSpiMode | NoPin => false,
            _ => true,
        }
    }

    /// Check whether the HalError signals no pin is available
    pub fn is_no_pin(&self) -> bool {
        use HalError::*;

        match self {
            NoPin => true,
            _ => false,
        }
    }
}

#[cfg(feature = "hal-cp2130")]
impl From<driver_cp2130::Error> for HalError {
    fn from(e: driver_cp2130::Error) -> Self {
        Self::Cp2130(e)
    }
}

#[cfg(feature = "hal-linux")]
impl From<std::io::Error> for HalError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.kind())
    }
}

#[cfg(feature = "hal-linux")]
impl From<linux_embedded_hal::sysfs_gpio::Error> for HalError {
    fn from(e: linux_embedded_hal::sysfs_gpio::Error) -> Self {
        Self::Sysfs(e)
    }
}

impl embedded_hal::spi::Error for HalError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        embedded_hal::spi::ErrorKind::Other
    }
}
