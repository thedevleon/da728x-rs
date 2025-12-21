//! Device configuration structures and builders

use crate::types::*;

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
            imax: u32::from(IMAX_DEFAULT),
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
        if (28_600..IMAX_LIMIT).contains(&val) {
            self.imax = (val - 28_600) / IMAX_STEP + 1;
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

/// Builder for creating haptic effect patterns
#[derive(Debug, Clone)]
pub struct PatternBuilder {
    data: [u8; SNP_MEM_SIZE],
    length: usize,
}

impl PatternBuilder {
    /// Create a new pattern builder
    pub fn new() -> Self {
        Self {
            data: [0u8; SNP_MEM_SIZE],
            length: 0,
        }
    }

    /// Add a level value to the pattern (0-255)
    ///
    /// The level represents the drive strength at this step.
    /// For DRO mode with acceleration disabled: 0-255
    /// For DRO mode with acceleration enabled: 0-127
    pub fn add_level(mut self, level: u8) -> Result<Self, &'static str> {
        if self.length >= SNP_MEM_SIZE {
            return Err("Pattern memory full");
        }
        self.data[self.length] = level;
        self.length += 1;
        Ok(self)
    }

    /// Add multiple level values to the pattern
    pub fn add_levels(mut self, levels: &[u8]) -> Result<Self, &'static str> {
        if self.length + levels.len() > SNP_MEM_SIZE {
            return Err("Pattern memory would overflow");
        }
        self.data[self.length..self.length + levels.len()].copy_from_slice(levels);
        self.length += levels.len();
        Ok(self)
    }

    /// Repeat the current pattern a number of times
    pub fn repeat(mut self, times: usize) -> Result<Self, &'static str> {
        if self.length == 0 {
            return Err("Cannot repeat empty pattern");
        }
        
        let original_length = self.length;
        for _ in 0..times {
            if self.length + original_length > SNP_MEM_SIZE {
                return Err("Pattern memory would overflow");
            }
            self.data.copy_within(0..original_length, self.length);
            self.length += original_length;
        }
        Ok(self)
    }

    /// Build the pattern and return the memory data
    pub fn build(self) -> Result<[u8; SNP_MEM_SIZE], &'static str> {
        if self.length == 0 {
            return Err("Pattern is empty");
        }
        if self.length < 4 {
            return Err("Pattern too short (minimum 4 bytes)");
        }
        Ok(self.data)
    }

    /// Get the current length of the pattern
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the pattern is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl Default for PatternBuilder {
    fn default() -> Self {
        Self::new()
    }
}
