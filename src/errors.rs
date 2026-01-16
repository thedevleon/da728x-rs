use core::fmt::{Debug, Display, Formatter, Result};
use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::i2c::ErrorKind as I2cErrorKind;

#[derive(Debug)]
pub enum Error {
    I2c(I2cErrorKind),
    Gpio(DigitalErrorKind),
    VariantMismatch,
    InvalidValue,
    NotConfigured,
    WrongMode,
    // Waveform memory errors
    WaveformMemoryFull,
    TooManySnippets,
    TooManySequences,
    InvalidSnippetId,
    InvalidTimebase,
    InvalidAmplitude,
    InvalidFrequency,
    InvalidLoopCount,
    EmptySnippet,
    EmptySequence,
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::I2c(err) => write!(f, "I2C error: {}", err),
            Error::Gpio(err) => write!(f, "GPIO error: {}", err),
            Error::VariantMismatch => write!(f, "Variant does not match chip ID"),
            Error::InvalidValue => write!(f,  "Invalid value, most likely out of range."),
            Error::NotConfigured => write!(f, "Configuration has not beed set yet."),
            Error::WrongMode => write!(f, "Driver is not in the right mode to support this operation"),
            Error::WaveformMemoryFull => write!(f, "Waveform memory exceeds 100 bytes"),
            Error::TooManySnippets => write!(f, "Too many snippets (max 15)"),
            Error::TooManySequences => write!(f, "Too many sequences (max 16)"),
            Error::InvalidSnippetId => write!(f, "Invalid snippet ID"),
            Error::InvalidTimebase => write!(f, "Invalid timebase value"),
            Error::InvalidAmplitude => write!(f, "Invalid amplitude value"),
            Error::InvalidFrequency => write!(f, "Invalid frequency value"),
            Error::InvalidLoopCount => write!(f, "Invalid loop count"),
            Error::EmptySnippet => write!(f, "Snippet must contain at least one point"),
            Error::EmptySequence => write!(f, "Sequence must contain at least one frame"),
        }
    }
}

impl core::error::Error for Error
{
}