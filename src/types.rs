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

/// Main configuration structure for the DA728x device
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    /// Device type (LRA, ERM Bar, ERM Coin)
    pub dev_type: Option<DeviceType>,
    /// Operation mode for constant effects
    pub const_op_mode: OperationMode,
    /// Operation mode for periodic effects
    pub periodic_op_mode: OperationMode,
    /// Nominal maximum voltage (in device units, or SKIP_INIT)
    pub nommax: u16,
    /// Absolute maximum voltage (in device units, or SKIP_INIT)
    pub absmax: u16,
    /// Maximum current in microamps
    pub imax: u32,
    /// Impedance in micro-ohms
    pub impd: u32,
    /// Resonant frequency high byte (or SKIP_INIT)
    pub resonant_freq_h: u16,
    /// Resonant frequency low byte (or SKIP_INIT)
    pub resonant_freq_l: u16,
    /// Enable back-EMF sensing
    pub bemf_sense_en: bool,
    /// Enable frequency tracking
    pub freq_track_en: bool,
    /// Enable acceleration
    pub acc_en: bool,
    /// Enable rapid stop
    pub rapid_stop_en: bool,
    /// Enable amplitude PID control
    pub amp_pid_en: bool,
    /// Pre-stored sequence ID
    pub ps_seq_id: u8,
    /// Pre-stored sequence loop count
    pub ps_seq_loop: u8,
    /// GPI configurations (3 GPIs)
    pub gpi_ctl: [GpiConfig; 3],
    /// Waveform memory data (if provided)
    pub mem_data: Option<[u8; SNP_MEM_SIZE]>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            dev_type: None,
            const_op_mode: OperationMode::DRO,
            periodic_op_mode: OperationMode::RTWM,
            nommax: SKIP_INIT,
            absmax: SKIP_INIT,
            imax: IMAX_DEFAULT as u32,
            impd: IMPD_DEFAULT,
            resonant_freq_h: SKIP_INIT,
            resonant_freq_l: SKIP_INIT,
            bemf_sense_en: false,
            freq_track_en: false,
            acc_en: false,
            rapid_stop_en: false,
            amp_pid_en: false,
            ps_seq_id: 0,
            ps_seq_loop: 0,
            gpi_ctl: [
                GpiConfig { seq_id: 0, ..Default::default() },
                GpiConfig { seq_id: 1, ..Default::default() },
                GpiConfig { seq_id: 2, ..Default::default() },
            ],
            mem_data: None,
        }
    }
}

impl DeviceConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the device type
    pub fn with_device_type(mut self, dev_type: DeviceType) -> Self {
        self.dev_type = Some(dev_type);
        self
    }

    /// Set the nominal maximum voltage in microvolts
    pub fn with_nom_microvolt(mut self, val: u32) -> Self {
        if val < VOLTAGE_RATE_MAX {
            self.nommax = (val / VOLTAGE_RATE_STEP + 1).min(0xFF) as u16;
        }
        self
    }

    /// Set the absolute maximum voltage in microvolts
    pub fn with_abs_max_microvolt(mut self, val: u32) -> Self {
        if val < VOLTAGE_RATE_MAX {
            self.absmax = (val / VOLTAGE_RATE_STEP + 1).min(0xFF) as u16;
        }
        self
    }

    /// Set the maximum current in microamps
    pub fn with_imax_microamp(mut self, val: u32) -> Self {
        if val < IMAX_LIMIT {
            self.imax = (val.saturating_sub(28_600)) / IMAX_STEP + 1;
        }
        self
    }

    /// Set the impedance in micro-ohms
    pub fn with_impd_micro_ohms(mut self, val: u32) -> Self {
        if val <= IMPD_MAX {
            self.impd = val;
        }
        self
    }

    /// Set the resonant frequency in Hz
    pub fn with_resonant_freq_hz(mut self, val: u32) -> Self {
        if val < MAX_RESONAT_FREQ_HZ && val > MIN_RESONAT_FREQ_HZ {
            let freq_val = 1_000_000_000 / (val * 1333);
            self.resonant_freq_h = ((freq_val >> 7) & 0xFF) as u16;
            self.resonant_freq_l = (freq_val & 0x7F) as u16;
        } else {
            self.resonant_freq_h = RESONT_FREQH_DFT as u16;
            self.resonant_freq_l = RESONT_FREQL_DFT as u16;
        }
        self
    }

    /// Enable back-EMF sensing
    pub fn with_bemf_sense(mut self, enable: bool) -> Self {
        self.bemf_sense_en = enable;
        self
    }

    /// Enable frequency tracking
    pub fn with_freq_track(mut self, enable: bool) -> Self {
        self.freq_track_en = enable;
        self
    }

    /// Enable acceleration
    pub fn with_acceleration(mut self, enable: bool) -> Self {
        self.acc_en = enable;
        self
    }

    /// Enable rapid stop
    pub fn with_rapid_stop(mut self, enable: bool) -> Self {
        self.rapid_stop_en = enable;
        self
    }

    /// Enable amplitude PID control
    pub fn with_amp_pid(mut self, enable: bool) -> Self {
        self.amp_pid_en = enable;
        self
    }

    /// Set pre-stored sequence ID
    pub fn with_ps_seq_id(mut self, id: u8) -> Self {
        if id <= SEQ_ID_MAX {
            self.ps_seq_id = id;
        }
        self
    }

    /// Set pre-stored sequence loop count
    pub fn with_ps_seq_loop(mut self, count: u8) -> Self {
        if count <= SEQ_LOOP_MAX {
            self.ps_seq_loop = count;
        }
        self
    }

    /// Set GPI configuration for a specific GPI pin
    pub fn with_gpi_config(mut self, gpi_num: usize, config: GpiConfig) -> Self {
        if gpi_num < 3 {
            self.gpi_ctl[gpi_num] = config;
        }
        self
    }

    /// Set waveform memory data
    pub fn with_mem_data(mut self, data: [u8; SNP_MEM_SIZE]) -> Self {
        self.mem_data = Some(data);
        self
    }
}
