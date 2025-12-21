use core::fmt::{Debug, Display, Formatter, Result};
use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::i2c::ErrorKind as I2cErrorKind;

#[derive(Debug)]
pub enum Error {
    I2c(I2cErrorKind),
    Gpio(DigitalErrorKind),
    VariantMismatch,
    DeviceBusy,
    MemoryLocked,
    InvalidParameter,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::I2c(err) => write!(f, "I2C error: {}", err),
            Error::Gpio(err) => write!(f, "GPIO error: {}", err),
            Error::VariantMismatch => write!(f, "Variant does not match chip ID"),
            Error::DeviceBusy => write!(f, "Device is busy"),
            Error::MemoryLocked => write!(f, "Memory is locked"),
            Error::InvalidParameter => write!(f, "Invalid parameter"),
        }
    }
}

impl core::error::Error for Error {
}