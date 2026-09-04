use core::option::Option;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy)]
pub struct ActuatorConfig {
    pub actuator_type: ActuatorType,
    pub nominal_max_mV: u16,
    pub absolute_max_mV: u16,
    pub max_current_mA: u16,
    pub impedance_mOhm: u16,
    pub inductance_uH: u16,
    pub frequency_Hz: u16,
    pub pid_Kp_Ki: Option<(u16, u16)>,
}

#[allow(nonstandard_style)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActuatorType {
    LRA = 0,
    ERM = 1,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TriggerMode {
    SingleSequence = 0,
    MultiSequence = 1,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TriggerPolarity {
    Rising = 0,
    Falling = 1,
    Both = 2,
}

#[derive(Debug, Clone, Copy)]
pub struct TriggerConfig {
    pub sequence_id: u8,
    pub polarity: TriggerPolarity,
    pub mode: TriggerMode,
}

#[derive(Debug, Clone, Copy)]
pub struct DeviceConfig {
    pub operation_mode: OperationMode,
    pub driving_mode: DrivingMode,
    pub gpi_triggers: [Option<TriggerConfig>; 3],
    pub acceleration: bool,
    pub rapid_stop: bool,
}

#[allow(nonstandard_style)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationMode {
    INACTIVE = 0,
    DRO_MODE = 1,
    PWM_MODE = 2,
    RTWM_MODE = 3,
    ETWM_MODE = 4,
}

/// According to 5.7 Advanced Operation
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(nonstandard_style)]
pub enum DrivingMode {
    FREQUENCY_TRACK,
    WIDEBAND,
    CUSTOM_WAVEFORM,
}
