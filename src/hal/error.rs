
#[derive(Debug)]
pub enum Error {
    InvalidConfig,

    #[cfg(feature = "hal-cp2130")]
    Cp2130(driver_cp2130::Error),

    #[cfg(feature = "hal-linux")]
    Io(std::io::Error),

    #[cfg(feature = "hal-linux")]
    Pin(linux_embedded_hal::sysfs_gpio::Error),
}

#[cfg(feature = "hal-cp2130")]
impl From<driver_cp2130::Error> for Error {
    fn from(e: driver_cp2130::Error) -> Self {
        Self::Cp2130(e)
    }
}

#[cfg(feature = "hal-linux")]
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(feature = "hal-linux")]
impl From<linux_embedded_hal::sysfs_gpio::Error> for Error {
    fn from(e: linux_embedded_hal::sysfs_gpio::Error) -> Self {
        Self::Pin(e)
    }
}
