//! Constants and type definitions for DA728x driver

/// Maximum voltage rating in microvolts (6V)
pub const VOLTAGE_RATE_MAX: u32 = 6_000_000;
/// Voltage rating step in microvolts (23.4mV)
pub const VOLTAGE_RATE_STEP: u32 = 23_400;
/// Default nominal maximum voltage
pub const NOMMAX_DFT: u8 = 0x6B;
/// Default absolute maximum voltage
pub const ABSMAX_DFT: u8 = 0x78;

/// Maximum impedance in micro-ohms (1500Ω)
pub const IMPD_MAX: u32 = 1_500_000_000;
/// Default impedance in micro-ohms (22Ω)
pub const IMPD_DEFAULT: u32 = 22_000_000;

/// Default IMAX value
pub const IMAX_DEFAULT: u8 = 0x0E;
/// IMAX step in microamps (7.2mA)
pub const IMAX_STEP: u32 = 7_200;
/// IMAX limit in microamps (252mA)
pub const IMAX_LIMIT: u32 = 252_000;

/// Default resonant frequency high byte
pub const RESONT_FREQH_DFT: u8 = 0x39;
/// Default resonant frequency low byte
pub const RESONT_FREQL_DFT: u8 = 0x32;
/// Minimum resonant frequency in Hz
pub const MIN_RESONAT_FREQ_HZ: u32 = 50;
/// Maximum resonant frequency in Hz
pub const MAX_RESONAT_FREQ_HZ: u32 = 300;

/// Maximum sequence ID value
pub const SEQ_ID_MAX: u8 = 15;
/// Maximum sequence loop value
pub const SEQ_LOOP_MAX: u8 = 15;
/// Default GPI sequence ID
pub const GPI_SEQ_ID_DFT: u8 = 0;
/// Maximum GPI sequence ID
pub const GPI_SEQ_ID_MAX: u8 = 2;

/// Size of the waveform memory in bytes
pub const SNP_MEM_SIZE: usize = 100;

/// Skip initialization marker
pub const SKIP_INIT: u16 = 0x100;

/// Haptic device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    /// Linear Resonant Actuator
    LRA = 0,
    /// Eccentric Rotating Mass (Bar)
    ErmBar = 1,
    /// Eccentric Rotating Mass (Coin)
    ErmCoin = 2,
}

impl DeviceType {
    /// Convert to the ACTUATOR_TYPE bit value
    pub fn to_bit(&self) -> bool {
        match self {
            DeviceType::LRA => false,
            DeviceType::ErmBar | DeviceType::ErmCoin => true,
        }
    }
}

/// Operation mode for the haptic driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Inactive mode
    Inactive = 0,
    /// Direct Register Override mode
    DRO = 1,
    /// PWM mode
    PWM = 2,
    /// Real-Time Waveform Memory mode
    RTWM = 3,
    /// External Trigger Waveform Memory mode
    ETWM = 4,
}

impl OperationMode {
    /// Convert to register value
    pub fn to_register_value(&self) -> u8 {
        *self as u8
    }
}

/// GPI polarity configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpiPolarity {
    /// Rising edge
    RisingEdge = 0,
    /// Falling edge
    FallingEdge = 1,
    /// Both edges
    BothEdge = 2,
}

impl GpiPolarity {
    /// Convert to register value
    pub fn to_register_value(&self) -> u8 {
        *self as u8
    }
}

/// GPI mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpiMode {
    /// Single pattern mode
    SinglePattern = 0,
    /// Multi pattern mode
    MultiPattern = 1,
}

impl GpiMode {
    /// Convert to register value
    pub fn to_register_value(&self) -> bool {
        match self {
            GpiMode::SinglePattern => false,
            GpiMode::MultiPattern => true,
        }
    }
}

/// Configuration for a single GPI pin
#[derive(Debug, Clone, Copy)]
pub struct GpiConfig {
    /// Sequence ID to trigger
    pub seq_id: u8,
    /// GPI mode
    pub mode: GpiMode,
    /// GPI polarity
    pub polarity: GpiPolarity,
}

impl Default for GpiConfig {
    fn default() -> Self {
        Self {
            seq_id: GPI_SEQ_ID_DFT,
            mode: GpiMode::SinglePattern,
            polarity: GpiPolarity::RisingEdge,
        }
    }
}


