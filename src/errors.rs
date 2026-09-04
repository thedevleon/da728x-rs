use core::fmt::{Debug, Display, Formatter, Result};
use embedded_hal::digital::ErrorKind as DigitalErrorKind;
use embedded_hal::i2c::ErrorKind as I2cErrorKind;

#[derive(Debug)]
pub enum Error {
    I2c(I2cErrorKind),
    Gpio(DigitalErrorKind),
    VariantMismatch,
    InvalidValue,
    NotSupported,
    NotConfigured,
    WrongMode,
    MissingTriggerConfigInEtwmMode,
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

#[cfg(feature = "debug")]
impl defmt::Format for Error {
    fn format(&self, f: defmt::Formatter) {
        match self {
            Error::I2c(err) => defmt::write!(f, "I2C error: {}", defmt::Debug2Format(err)),
            Error::Gpio(err) => defmt::write!(f, "GPIO error: {}", defmt::Debug2Format(err)),
            Error::VariantMismatch => defmt::write!(f, "Variant does not match chip ID"),
            Error::InvalidValue => defmt::write!(f, "Invalid value, most likely out of range."),
            Error::NotSupported => defmt::write!(f, "Not supported"),
            Error::NotConfigured => defmt::write!(f, "Configuration has not beed set yet."),
            Error::MissingTriggerConfigInEtwmMode => defmt::write!(f, "Missing trigger configuration in ETWM mode"),
            Error::WrongMode => defmt::write!(f, "Driver is not in the right mode to support this operation"),
            Error::WaveformMemoryFull => defmt::write!(f, "Waveform memory exceeds 100 bytes"),
            Error::TooManySnippets => defmt::write!(f, "Too many snippets (max 15)"),
            Error::TooManySequences => defmt::write!(f, "Too many sequences (max 16)"),
            Error::InvalidSnippetId => defmt::write!(f, "Invalid snippet ID"),
            Error::InvalidTimebase => defmt::write!(f, "Invalid timebase value"),
            Error::InvalidAmplitude => defmt::write!(f, "Invalid amplitude value"),
            Error::InvalidFrequency => defmt::write!(f, "Invalid frequency value"),
            Error::InvalidLoopCount => defmt::write!(f, "Invalid loop count"),
            Error::EmptySnippet => defmt::write!(f, "Snippet must contain at least one point"),
            Error::EmptySequence => defmt::write!(f, "Sequence must contain at least one frame"),
        }
    }
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Error::I2c(err) => write!(f, "I2C error: {}", err),
            Error::Gpio(err) => write!(f, "GPIO error: {}", err),
            Error::VariantMismatch => write!(f, "Variant does not match chip ID"),
            Error::InvalidValue => write!(f,  "Invalid value, most likely out of range."),
            Error::NotSupported => write!(f, "Not supported"),
            Error::NotConfigured => write!(f, "Configuration has not beed set yet."),
            Error::MissingTriggerConfigInEtwmMode => write!(f, "Missing trigger configuration in ETWM mode"),
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