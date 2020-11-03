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
