#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use bitfield_struct::bitfield;

/// Register addresses for DA728x devices
#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    CHIP_REV = 0x00,
    IRQ_EVENT1 = 0x03,
    IRQ_EVENT_WARNING_DIAG = 0x04,
    IRQ_EVENT_SEQ_DIAG = 0x05,
    IRQ_STATUS1 = 0x06,
    IRQ_MASK1 = 0x07,
    FRQ_LRA_PER_H = 0x0A,
    FRQ_LRA_PER_L = 0x0B,
    ACTUATOR1 = 0x0C,
    ACTUATOR2 = 0x0D,
    ACTUATOR3 = 0x0E,
    CALIB_V2I_H = 0x0F,
    CALIB_V2I_L = 0x10,
    TOP_CFG1 = 0x13,
    TOP_CFG2 = 0x14,
    TOP_CFG4 = 0x16,
    TOP_INT_CFG1 = 0x17,
    TOP_CTL1 = 0x22,
    TOP_CTL2 = 0x23,
    SEQ_CTL1 = 0x24,
    SWG_C1 = 0x25,
    SWG_C2 = 0x26,
    SWG_C3 = 0x27,
    SEQ_CTL2 = 0x28,
    GPI_0_CTL = 0x29,
    GPI_1_CTL = 0x2A,
    GPI_2_CTL = 0x2B,
    MEM_CTL1 = 0x2C,
    MEM_CTL2 = 0x2D,
    POLARITY = 0x43,
    FRQ_PHASE_H = 0x48,
    FRQ_PHASE_L = 0x49,
    TOP_CFG5 = 0x6E,
    IRQ_MASK2 = 0x83,
    SNP_MEM_0 = 0x84,
    SNP_MEM_99 = 0xE7,
}

/// Chip revision register
#[bitfield(u8)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct CHIP_REV {
    #[bits(4, access = RO)]
    pub CHIP_REV_MAJOR: u8,
    #[bits(4, access = RO)]
    pub CHIP_REV_MINOR: u8,
}

/// IRQ Event 1 register (0x03)
#[cfg_attr(feature = "debug", bitfield(u8, defmt = true))]
#[cfg_attr(not(feature = "debug"), bitfield(u8))]
pub struct IRQ_EVENT1 {
    pub E_SEQ_CONTINUE: bool,
    pub E_UVLO: bool,
    pub E_SEQ_DONE: bool,
    pub E_OVERTEMP_CRIT: bool,
    pub E_SEQ_FAULT: bool,
    pub E_WARNING: bool,
    pub E_ACTUATOR_FAULT: bool,
    pub E_OC_FAULT: bool,
}

/// IRQ Event Warning Diag register (0x04)
#[cfg_attr(feature = "debug", bitfield(u8, defmt = true))]
#[cfg_attr(not(feature = "debug"), bitfield(u8))]
pub struct IRQ_EVENT_WARNING_DIAG {
    #[bits(3)]
    __: u8,
    pub E_OVERTEMP_WARN: bool,
    pub E_MEM_TYPE: bool,
    #[bits(1)]
    __: u8,
    pub E_LIM_DRIVE_ACC: bool,
    pub E_LIM_DRIVE: bool,
}

/// IRQ Event Sequence Diag register (0x05)
#[cfg_attr(feature = "debug", bitfield(u8, defmt = true))]
#[cfg_attr(not(feature = "debug"), bitfield(u8))]
pub struct IRQ_EVENT_SEQ_DIAG {
    #[bits(5)]
    __: u8,
    pub E_PWM_FAULT: bool,
    pub E_MEM_FAULT: bool,
    pub E_SEQ_ID_FAULT: bool,
}

/// IRQ Status 1 register (0x06)
#[cfg_attr(feature = "debug", bitfield(u8, defmt = true))]
#[cfg_attr(not(feature = "debug"), bitfield(u8))]
pub struct IRQ_STATUS1 {
    pub STA_SEQ_CONTINUE: bool,
    pub STA_UVLO_VBAT_OK: bool,
    pub STA_SEQ_DONE: bool,
    pub STA_OVERTEMP_CRIT: bool,
    pub STA_SEQ_FAULT: bool,
    pub STA_WARNING: bool,
    pub STA_ACTUATOR: bool,
    pub STA_OC: bool,
}

/// IRQ Mask 1 register (0x07)
#[bitfield(u8)]
pub struct IRQ_MASK1 {
    pub SEQ_CONTINUE_M: bool,
    pub E_UVLO_M: bool,
    pub SEQ_DONE_M: bool,
    pub OVERTEMP_CRIT_M: bool,
    pub SEQ_FAULT_M: bool,
    pub WARNING_M: bool,
    pub ACTUATOR_M: bool,
    pub OC_M: bool,
}

/// FRQ_LRA_PER_H register (0x0A)
#[bitfield(u8)]
pub struct FRQ_LRA_PER_H {
    pub LRA_PER_H: u8
}

/// FRQ_LRA_PER_L register (0x0B)
#[bitfield(u8)]
pub struct FRQ_LRA_PER_L {
    #[bits(7)]
    pub LRA_PER_L: u8,
    #[bits(1)]
    __: u8
}

/// ACTUATOR1 register (0x0C)
#[bitfield(u8)]
pub struct ACTUATOR1 {
    pub ACTUATOR_NOMMAX: u8
}

/// ACTUATOR2 register (0x0D)
#[bitfield(u8)]
pub struct ACTUATOR2 {
    pub ACTUATOR_ABSMAX: u8
}

/// ACTUATOR3 register (0x0E)
#[bitfield(u8)]
pub struct ACTUATOR3 {
    #[bits(5)]
    pub IMAX: u8,
    #[bits(3)]
    __: u8,
}

/// CALIB_V2I_H register (0x0F)
#[bitfield(u8)]
pub struct CALIB_V2I_H {
    pub V2I_FACTOR_H: u8,
}

/// CALIB_V2I_L register (0x10)
#[bitfield(u8)]
pub struct CALIB_V2I_L {
    pub V2I_FACTOR_L: u8,
}

/// TOP_CFG1 register (0x13)
#[bitfield(u8)]
pub struct TOP_CFG1 {
    pub AMP_PID_EN: bool,
    pub RAPID_STOP_EN: bool,
    pub ACCELERATION_EN: bool,
    pub FREQ_TRACK_EN: bool,
    pub BEMF_SENSE_EN: bool,
    #[bits(1)]
    pub ACTUATOR_TYPE: u8,
    #[bits(2)]
    __: u8,
}

/// TOP_CFG2 register (0x14)
#[bitfield(u8)]
pub struct TOP_CFG2 {
    #[bits(4)]
    pub FULL_BRAKE_THR: u8,
    pub MEM_DATA_SIGNED: bool,
    #[bits(3)]
    __: u8,
}

/// TOP_CFG4 register (0x16)
#[bitfield(u8)]
pub struct TOP_CFG4 {
    #[bits(6)]
    __: u8,
    pub TST_CALIB_IMPEDANCE_DIS: bool,
    pub V2I_FACTOR_FREEZE: bool,
}

/// TOP_INT_CFG1 register (0x17)
#[bitfield(u8)]
pub struct TOP_INT_CFG1 {
    #[bits(2)]
    pub BEMF_FAULT_LIM: u8,
    #[bits(6)]
    __: u8,
}

/// TOP_CTL1 register (0x22)
#[cfg_attr(feature = "debug", bitfield(u8, defmt = true))]
#[cfg_attr(not(feature = "debug"), bitfield(u8))]
pub struct TOP_CTL1 {
    #[bits(3)]
    pub OPERATION_MODE: u8,
    pub STANDBY_EN: bool,
    pub SEQ_START: bool,
    #[bits(3)]
    __: u8,
}

/// TOP_CTL2 register (0x23)
#[bitfield(u8)]
pub struct TOP_CTL2 {
    OVERRIDE_VAL: u8
}

/// SEQ_CTL1 register (0x24)
#[bitfield(u8)]
pub struct SEQ_CTL1 {
    #[bits(1)]
    pub SEQ_CONTINUE: bool,
    #[bits(1)]
    pub WAVEGEN_MODE: bool,
    #[bits(1)]
    pub FREQ_WAVEFORM_TIMEBASE: u8,
    #[bits(5)]
    __: u8
}

/// SWG_C1 register (0x25)
#[bitfield(u8)]
pub struct SWG_C1 {
    pub CUSTOM_WAVE_GEN_COEFF1: u8,
}

/// SWG_C2 register (0x26)
#[bitfield(u8)]
pub struct SWG_C2 {
    pub CUSTOM_WAVE_GEN_COEFF2: u8,
}

/// SWG_C3 register (0x27)
#[bitfield(u8)]
pub struct SWG_C3 {
    pub CUSTOM_WAVE_GEN_COEFF3: u8,
}

/// SEQ_CTL2 register (0x28)
#[bitfield(u8)]
pub struct SEQ_CTL2 {
    #[bits(4)]
    pub PS_SEQ_ID: u8,
    #[bits(4)]
    pub PS_SEQ_LOOP: u8,
}

/// GPI Control register (0x29, 0x2A, 0x2B)
#[bitfield(u8)]
pub struct GPI_CTL {
    #[bits(2)]
    pub POLARITY: u8,
    pub MODE: bool,
    #[bits(4)]
    pub SEQUENCE_ID: u8,
    #[bits(1)]
    __: u8,
}

/// MEM_CTL1 register (0x2C)
#[bitfield(u8)]
pub struct MEM_CTL1 {
    #[bits(8, access = RO)]
    pub WAV_MEM_BASE_ADDR: u8,
}

/// MEM_CTL2 register (0x2D)
#[bitfield(u8)]
pub struct MEM_CTL2 {
    #[bits(7)]
    __: u8,
    pub WAV_MEM_LOCK: bool,
}

/// FRQ_PHASE_H register (0x48)
#[bitfield(u8)]
pub struct FRQ_PHASE_H {
    pub DELAY_H: u8,
}

/// FRQ_PHASE_L register (0x49)
#[bitfield(u8)]
pub struct FRQ_PHASE_L {
    #[bits(3)]
    pub DELAY_SHIFT_L: u8,
    #[bits(4)]
    __: u8,
    pub DELAY_FREEZE: bool
}

/// TOP_CFG5 register (0x6E)
#[bitfield(u8)]
pub struct TOP_CFG5 {
    pub V2I_FACTOR_OFFSET_EN: bool,
    #[bits(7)]
    __: u8,
}

/// IRQ_MASK2 register (0x83)
#[bitfield(u8)]
pub struct IRQ_MASK2 {
    #[bits(7)]
    __: u8,
    pub ADC_SAT_M: bool,
}