#![deny(unsafe_code)]
#![allow(unused)]
#![no_std]
#![doc = include_str!("../README.md")]

pub mod config;
pub mod errors;
pub mod registers;
pub mod waveform;

use maybe_async::maybe_async;

#[cfg(not(feature = "blocking"))]
use embedded_hal_async::i2c::I2c;
#[cfg(feature = "blocking")]
use embedded_hal::i2c::I2c;

use embedded_hal::i2c::Error as I2cError;

#[cfg(feature = "debug")]
use defmt::{debug, info};

use config::{ActuatorConfig, DeviceConfig, DrivingMode, OperationMode};
use errors::Error;
use registers::Register;
use registers::{
    ACTUATOR1, ACTUATOR2, ACTUATOR3, CALIB_V2I_H, CALIB_V2I_L, CHIP_REV, FRQ_LRA_PER_H,
    FRQ_LRA_PER_L, FRQ_PHASE_H, FRQ_PHASE_L, IRQ_EVENT_SEQ_DIAG, IRQ_EVENT_WARNING_DIAG,
    IRQ_EVENT1, IRQ_STATUS1, TOP_CFG1, TOP_CTL1,
};

use crate::registers::MEM_CTL2;
use crate::registers::SEQ_CTL1;
use crate::registers::SEQ_CTL2;
use crate::registers::TOP_CFG2;
use crate::registers::TOP_CFG4;
use crate::registers::TOP_CFG5;
use crate::registers::TOP_CTL2;
use crate::registers::TOP_INT_CFG6_H;
use crate::registers::TOP_INT_CFG6_L;
use crate::registers::TOP_INT_CFG7_H;
use crate::registers::TOP_INT_CFG7_L;
use crate::registers::TOP_INT_CFG8;
use crate::registers::TRIM4;
use crate::waveform::WaveformMemory;
use crate::waveform::WaveformMemoryTimebase;

pub enum Variant {
    DA7280 = 0xBA,
    DA7281 = 0xCA,
    DA7282 = 0xDA,
}

pub struct DA728x<I2C> {
    i2c: I2C,
    address: u8,
    variant: Variant,
    actuator_config: Option<ActuatorConfig>,
    device_config: Option<DeviceConfig>,
}

#[maybe_async]
impl<I2C> DA728x<I2C>
where
    I2C: I2c,
{
    pub async fn new(i2c: I2C, address: u8, variant: Variant) -> Result<Self, Error> {
        let mut da728x = DA728x {
            i2c,
            address,
            variant,
            actuator_config: None,
            device_config: None,
        };

        // Check that CHIP_REV matches with selected Variant
        let chip_rev = da728x.get_chip_rev().await?;

        #[cfg(feature = "debug")]
        debug!(
            "CHIP_REV = 0x{:X}{:X}",
            chip_rev.CHIP_REV_MINOR(),
            chip_rev.CHIP_REV_MAJOR()
        );

        match da728x.variant {
            Variant::DA7280 => {
                if chip_rev.CHIP_REV_MINOR() != 0xB || chip_rev.CHIP_REV_MAJOR() != 0xA {
                    return Err(Error::VariantMismatch);
                }
            }
            Variant::DA7281 => {
                if chip_rev.CHIP_REV_MINOR() != 0xC || chip_rev.CHIP_REV_MAJOR() != 0xA {
                    return Err(Error::VariantMismatch);
                }
            }
            Variant::DA7282 => {
                if chip_rev.CHIP_REV_MINOR() != 0xD || chip_rev.CHIP_REV_MAJOR() != 0xA {
                    return Err(Error::VariantMismatch);
                }
            }
        }

        Ok(da728x)
    }

    /// Configure the device with the supplied ActuatorConfig and DeviceConfig.
    ///
    /// There are a lot of inter-dependencies between the actuator config and the device config,
    /// so they need to be set together so that we can figure out if everything can work like configured
    /// And to deal with different value ranges for the registers depending on the driving modes
    pub async fn configure(
        &mut self,
        actuator_config: ActuatorConfig,
        device_config: DeviceConfig,
    ) -> Result<(), Error> {
        // Check for invalid combinations
        if device_config.driving_mode != DrivingMode::FREQUENCY_TRACK
            && (device_config.acceleration || device_config.rapid_stop)
        {
            return Err(Error::WrongMode);
        }

        // Check ranges of values before we set any registers
        if actuator_config.nominal_max_mV > 6000 {
            return Err(Error::InvalidValue);
        }
        if actuator_config.absolute_max_mV > 6000 {
            return Err(Error::InvalidValue);
        }
        if actuator_config.max_current_mA > 252 {
            return Err(Error::InvalidValue);
        }
        if !(4000..50_000).contains(&actuator_config.impedance_mOhm) {
            return Err(Error::InvalidValue);
        }
        match device_config.driving_mode {
            DrivingMode::FREQUENCY_TRACK => {
                if !(50..=300).contains(&actuator_config.frequency_Hz) {
                    return Err(Error::InvalidValue);
                }
            }
            DrivingMode::WIDEBAND | DrivingMode::CUSTOM_WAVEFORM => {
                if !(25..=1024).contains(&actuator_config.frequency_Hz) {
                    return Err(Error::InvalidValue);
                }
            }
        }

        // Figure out feature flags depending on actuator type and driving mode
        let (bemf_sense_en, frequency_track_en, acceleration_en, rapid_stop_en);

        match device_config.driving_mode {
            DrivingMode::FREQUENCY_TRACK => {
                bemf_sense_en = true;
                frequency_track_en = true;
                acceleration_en = device_config.acceleration;
                rapid_stop_en = device_config.rapid_stop;
            }
            DrivingMode::WIDEBAND | DrivingMode::CUSTOM_WAVEFORM => {
                bemf_sense_en = false;
                frequency_track_en = false;
                acceleration_en = false;
                rapid_stop_en = false;
            }
        }

        // TOP_CFG1 register (type and features)
        // EMBEDDED_MODE enables automatic fault clearing when entering IDLE state
        let top_cfg1 = TOP_CFG1::new()
            .with_ACTUATOR_TYPE(actuator_config.actuator_type as u8)
            .with_BEMF_SENSE_EN(bemf_sense_en)
            .with_FREQ_TRACK_EN(frequency_track_en)
            .with_ACCELERATION_EN(acceleration_en)
            .with_RAPID_STOP_EN(rapid_stop_en)
            .with_AMP_PID_EN(false) // Only supported with ERMs, disable for now.
            .with_EMBEDDED_MODE(true); // Auto-clear faults when entering IDLE
        self.write_register(Register::TOP_CFG1, top_cfg1.into())
            .await?;

        // TOP_CFG2 - MEM_DATA_SIGNED must match acceleration config:
        // MEM_DATA_SIGNED = 1 when ACCELERATION_EN = 0
        // MEM_DATA_SIGNED = 0 when ACCELERATION_EN = 1
        // Read-modify-write to preserve FULL_BRAKE_THR
        let top_cfg2_val = self.read_register(Register::TOP_CFG2).await?;
        let top_cfg2 = TOP_CFG2::from(top_cfg2_val).with_MEM_DATA_SIGNED(!acceleration_en);
        self.write_register(Register::TOP_CFG2, top_cfg2.into())
            .await?;

        // TOP_CFG5 - V2I_FACTOR_OFFSET_EN must match acceleration
        // V2I_FACTOR_OFFSET_EN = 0 when ACCELERATION_EN = 0
        // V2I_FACTOR_OFFSET_EN = 1 when ACCELERATION_EN = 1
        let top_cfg5 = TOP_CFG5::new().with_V2I_FACTOR_OFFSET_EN(acceleration_en);
        self.write_register(Register::TOP_CFG5, top_cfg5.into())
            .await?;

        // ACTUATOR1 (nom max volt)
        let volt_converted = ((actuator_config.nominal_max_mV as u32 * 1000) / 23400) as u8;
        let actuator1 = ACTUATOR1::from(volt_converted);
        self.write_register(Register::ACTUATOR1, actuator1.into())
            .await?;

        // ACTUATOR2 (abs max volt)
        let volt_converted = ((actuator_config.absolute_max_mV as u32 * 1000) / 23400) as u8; // +1?
        let actuator2 = ACTUATOR2::from(volt_converted);
        self.write_register(Register::ACTUATOR2, actuator2.into())
            .await?;

        // ACTUATOR3 (imax)
        let current_converted =
            ((actuator_config.max_current_mA as u32 * 1000 - 28600) / 7200) as u8; // +1?
        let current_converted_clone = current_converted as u32;
        let actuator3 = ACTUATOR3::new().with_IMAX(current_converted);
        self.write_register(Register::ACTUATOR3, actuator3.into())
            .await?;

        // CALIB_V2I_L / CALIB_V2I_H (impedance)
        let impedance_converted =
            ((actuator_config.impedance_mOhm as u32 * 1000 * (current_converted_clone + 4))
                / 1610400) as u16;
        let bytes: [u8; 2] = impedance_converted.to_be_bytes();
        let calib_v2i_h = CALIB_V2I_H::from(bytes[0]);
        let calib_v2i_l = CALIB_V2I_L::from(bytes[1]);
        self.write_register(Register::CALIB_V2I_H, calib_v2i_h.into())
            .await?;
        self.write_register(Register::CALIB_V2I_L, calib_v2i_l.into())
            .await?;

        // Setting TRIM4: LOOP_FILT_RES_TRIM and LOOP_FILT_CAP_TRIM
        // See Table 15 and Table 16 in Section 5.7.9 of the Datasheet
        // TODO: If inductance_uH > 1000 (1mH), we should also set LOOP_FILT_LOW_BW to 1 in TRIM3
        let cap_trim = match actuator_config.inductance_uH {
            0..=17 => 3,
            18..=27 => 2,
            28..=40 => 1,
            _ => 0,
        };
        // Table 16: LOOP_FILT_RES_TRIM lookup
        // Bin Lseries into 25 μH steps: ≤25, 50, 75, …, ≥250
        let l_idx = (actuator_config.inductance_uH.saturating_sub(13) / 25).min(9) as usize;
        // ≤25 50  75 100 125 150 175 200 225 ≥250
        let res_trim: u8 = match actuator_config.impedance_mOhm * 1000 {
            0..=5 => [2, 2, 3, 3, 3, 3, 3, 3, 3, 3][l_idx],
            6..=7 => [1, 2, 2, 3, 3, 3, 3, 3, 3, 3][l_idx],
            8..=9 => [1, 2, 2, 2, 3, 3, 3, 3, 3, 3][l_idx],
            10..=11 => [0, 1, 2, 2, 2, 3, 3, 3, 3, 3][l_idx],
            12..=13 => [0, 1, 2, 2, 2, 2, 3, 3, 3, 3][l_idx],
            14..=15 => [0, 1, 1, 2, 2, 2, 2, 3, 3, 3][l_idx],
            16..=17 => [0, 1, 1, 2, 2, 2, 2, 2, 3, 3][l_idx],
            18..=19 => [0, 0, 1, 1, 2, 2, 2, 2, 2, 3][l_idx],
            20..=21 => [0, 1, 1, 2, 2, 2, 3, 3, 3, 3][l_idx],
            22..=23 => [0, 1, 1, 2, 2, 2, 2, 3, 3, 3][l_idx],
            24..=25 => [0, 1, 1, 2, 2, 2, 2, 3, 3, 3][l_idx],
            26..=27 => [0, 1, 1, 2, 2, 2, 2, 2, 3, 3][l_idx],
            28..=31 => [0, 1, 2, 2, 2, 3, 3, 3, 3, 3][l_idx],
            32..=33 => [0, 1, 2, 2, 2, 2, 3, 3, 3, 3][l_idx],
            34..=35 => [0, 1, 1, 2, 2, 2, 3, 3, 3, 3][l_idx],
            36..=41 => [0, 1, 1, 2, 2, 2, 2, 3, 3, 3][l_idx],
            42..=45 => [0, 1, 2, 2, 3, 3, 3, 3, 3, 3][l_idx],
            _ => [0, 1, 2, 2, 2, 3, 3, 3, 3, 3][l_idx], // 46-50+
        };

        let trim4 = TRIM4::new()
            .with_LOOP_FILT_CAP_TRIM(cap_trim)
            .with_LOOP_FILT_RES_TRIM(res_trim);
        self.write_register(Register::TRIM4, trim4.into()).await?;

        // Set PID coefficients if provided
        if let Some((pid_kp, pid_ki)) = actuator_config.pid_Kp_Ki {
            let pid_kp_h = ((pid_kp >> 7) & 0xFF) as u8;
            let pid_kp_l = (pid_kp & 0x7F) as u8;
            let pid_ki_h = ((pid_ki >> 7) & 0xFF) as u8;
            let pid_ki_l = (pid_ki & 0x7F) as u8;

            let top_int_cfg6_h = TOP_INT_CFG6_H::from(pid_kp_h);
            let top_int_cfg6_l = TOP_INT_CFG6_L::from(pid_kp_l);
            let top_int_cfg7_h = TOP_INT_CFG7_H::from(pid_ki_h);
            let top_int_cfg7_l = TOP_INT_CFG7_L::from(pid_ki_l);

            self.write_register(Register::TOP_INT_CFG6_H, top_int_cfg6_h.into())
                .await?;
            self.write_register(Register::TOP_INT_CFG6_L, top_int_cfg6_l.into())
                .await?;
            self.write_register(Register::TOP_INT_CFG7_H, top_int_cfg7_h.into())
                .await?;
            self.write_register(Register::TOP_INT_CFG7_L, top_int_cfg7_l.into())
                .await?;
        }

        // Default resonant frequency
        let frequency_converted =
            (1_000_000_000 as u64 * 100 / (actuator_config.frequency_Hz as u64 * 133332)) as u16;
        let frequency_converted_h: u8 = ((frequency_converted >> 7) & 0xFF) as u8;
        let frequency_converted_l: u8 = (frequency_converted & 0x7F) as u8;
        let frq_lra_per_h = FRQ_LRA_PER_H::from(frequency_converted_h);
        let frq_lra_per_l = FRQ_LRA_PER_L::new().with_LRA_PER_L(frequency_converted_l);
        self.write_register(Register::FRQ_LRA_PER_H, frq_lra_per_h.into())
            .await?;
        self.write_register(Register::FRQ_LRA_PER_L, frq_lra_per_l.into())
            .await?;

        // Rapid Stop
        // TOP_INT_CFG8 -> TST_AMP_RAPID_STOP_LIM = 7 with rapid stop OFF
        // With rapid stop on, tune TOP_INT_CFG8 and TOP_INT_CFG1
        if !device_config.rapid_stop {
            let top_int_cfg8_val = self.read_register(Register::TOP_INT_CFG8).await?;
            let top_int_cfg8 = TOP_INT_CFG8::from(top_int_cfg8_val).with_RAPID_STOP_LIM(7);
            self.write_register(Register::TOP_INT_CFG8, top_int_cfg8.into())
                .await?;
        } else {
            todo!(
                "TOP_INT_CFG8 and TOP_INT_CFG1 are not yet set correctly with rapid stop enabled."
            );
        }

        // Additional configuration depending on DrivingMode

        // WIDEBAND MODE
        // FREQ_TRACK_EN = 0, ACCELERATION_EN = 0, RAPID_STOP_EN = 0, BEMF_SENSE_EN = 0
        // DELAY_H = 0, DELAY_SHIFT_L = 0, DELAY_FREEZE = 1

        // CUSTOM WAVEFORM MODE
        // FREQ_TRACK_EN = 0, ACCELERATION_EN = 0, RAPID_STOP_EN = 0, BEMF_SENSE_EN = 0, AMP_PID_EN = 0
        // DELAY_H = 0, DELAY_SHIFT_L = 0, DELAY_FREEZE = 1
        // WAVEGEN_MODE = 1, V2I_FACTOR_FREEZE = 1

        if device_config.driving_mode == DrivingMode::WIDEBAND
            || device_config.driving_mode == DrivingMode::CUSTOM_WAVEFORM
        {
            let frq_phase_h = FRQ_PHASE_H::from(0x00);
            let frq_phase_l = FRQ_PHASE_L::new()
                .with_DELAY_SHIFT_L(0x00)
                .with_DELAY_FREEZE(true);
            self.write_register(Register::FRQ_PHASE_H, frq_phase_h.into())
                .await?;
            self.write_register(Register::FRQ_PHASE_L, frq_phase_l.into())
                .await?;
        }

        if device_config.driving_mode == DrivingMode::CUSTOM_WAVEFORM {
            let seq_ctl1 = SEQ_CTL1::new().with_WAVEGEN_MODE(true);
            let top_cfg4 = TOP_CFG4::new().with_V2I_FACTOR_FREEZE(true); // Unclear if TST_CALIB_IMPEDANCE_DIS should be true/false.
            self.write_register(Register::SEQ_CTL1, seq_ctl1.into())
                .await?;
            self.write_register(Register::TOP_CFG4, top_cfg4.into())
                .await?;
        }

        self.actuator_config = Some(actuator_config);
        self.device_config = Some(device_config);
        Ok(())
    }

    pub async fn get_chip_rev(&mut self) -> Result<registers::CHIP_REV, Error> {
        let reg = self.read_register(Register::CHIP_REV).await?;
        Ok(CHIP_REV::from(reg))
    }

    /// This gets all system events (and also clears them...)
    pub async fn get_events(
        &mut self,
    ) -> Result<(IRQ_EVENT1, IRQ_EVENT_WARNING_DIAG, IRQ_EVENT_SEQ_DIAG), Error> {
        let irq_event1 = IRQ_EVENT1::from(self.read_register(Register::IRQ_EVENT1).await?);
        let irq_event_warning_diag = IRQ_EVENT_WARNING_DIAG::from(
            self.read_register(Register::IRQ_EVENT_WARNING_DIAG).await?,
        );
        let irq_event_seq_diag =
            IRQ_EVENT_SEQ_DIAG::from(self.read_register(Register::IRQ_EVENT_SEQ_DIAG).await?);
        //TODO: IRQ_EVENT_ACTUATOR_FAULT could be read as well. However, it is already cleared with writing to IRQ_EVENT->E_ACTUATOR.

        // Clear events (only IRQ_EVENT1)
        self.write_register(Register::IRQ_EVENT1, 0xFF).await?;

        Ok((irq_event1, irq_event_warning_diag, irq_event_seq_diag))
    }

    pub async fn get_status(&mut self) -> Result<IRQ_STATUS1, Error> {
        let status = self.read_register(Register::IRQ_STATUS1).await?;
        Ok(IRQ_STATUS1::from(status))
    }

    pub async fn set_frequency(&mut self, frequency_hz: u16) -> Result<(), Error> {
        if self.actuator_config.is_none() || self.device_config.is_none() {
            return Err(Error::NotConfigured);
        }

        let device_config = self.device_config.as_ref().unwrap();

        // Different frequency ranges with normal mode and wide-band/custom waveform mode
        match device_config.driving_mode {
            DrivingMode::FREQUENCY_TRACK => {
                if !(50..=300).contains(&frequency_hz) {
                    return Err(Error::InvalidValue);
                }
            }
            DrivingMode::WIDEBAND | DrivingMode::CUSTOM_WAVEFORM => {
                if !(25..=1024).contains(&frequency_hz) {
                    return Err(Error::InvalidValue);
                }
            }
        }

        let frequency_converted =
            (1_000_000_000 as u64 * 100 / (frequency_hz as u64 * 133332)) as u16;
        let frequency_converted_h: u8 = ((frequency_converted >> 7) & 0xFF) as u8;
        let frequency_converted_l: u8 = (frequency_converted & 0x7F) as u8;
        let frq_lra_per_h = FRQ_LRA_PER_H::from(frequency_converted_h);
        let frq_lra_per_l = FRQ_LRA_PER_L::new().with_LRA_PER_L(frequency_converted_l);
        self.write_register(Register::FRQ_LRA_PER_H, frq_lra_per_h.into())
            .await?;
        self.write_register(Register::FRQ_LRA_PER_L, frq_lra_per_l.into())
            .await?;

        Ok(())
    }

    /// Direct register override
    ///
    /// This sets the amplitude in the DRO_MODE
    /// With acceleration enabled, this has a range of 0..127
    /// With acceleration disabled, this has a range of -127..127
    pub async fn set_override_value(&mut self, value: i8) -> Result<(), Error> {
        if self.actuator_config.is_none() || self.device_config.is_none() {
            return Err(Error::NotConfigured);
        }

        if self.device_config.unwrap().acceleration && value < 0 {
            return Err(Error::InvalidValue);
        }

        let device_config = self.device_config.unwrap();

        if device_config.operation_mode != OperationMode::DRO_MODE {
            return Err(Error::WrongMode);
        }

        let top_ctl_2 = TOP_CTL2::from(value as u8);
        self.write_register(Register::TOP_CTL2, top_ctl_2.into())
            .await?;

        Ok(())
    }

    pub async fn set_timebase(&mut self, timebase: WaveformMemoryTimebase) -> Result<(), Error> {
        let seq_ctl_1_val = SEQ_CTL1::from(self.read_register(Register::SEQ_CTL1).await? as u8);
        let seq_ctl_1 = seq_ctl_1_val.with_FREQ_WAVEFORM_TIMEBASE(timebase as u8);
        self.write_register(Register::SEQ_CTL1, seq_ctl_1.into())
            .await?;

        Ok(())
    }

    /// Enable the configured operation mode
    ///
    /// Note: For RTWM/ETWM modes, this enables the mode but does NOT start sequence playback.
    /// Call `play_sequence()` or `start_sequence()` separately after enabling.
    pub async fn enable(&mut self) -> Result<(), Error> {
        if self.actuator_config.is_none() || self.device_config.is_none() {
            return Err(Error::NotConfigured);
        }
        let device_config = self.device_config.unwrap();

        // Build new TOP_CTL1 value - explicitly clear SEQ_START to avoid
        // accidentally starting a sequence before memory is ready
        let top_ctl1 = TOP_CTL1::new()
            .with_OPERATION_MODE(device_config.operation_mode as u8)
            .with_STANDBY_EN(false)
            .with_SEQ_START(false);

        #[cfg(feature = "debug")]
        debug!("TOP_CTL1: {:?}", top_ctl1);

        self.write_register(Register::TOP_CTL1, top_ctl1.into())
            .await?;

        Ok(())
    }

    /// Disable the configured Operation Mode (also stopping haptic feedback)
    pub async fn disable(&mut self) -> Result<(), Error> {
        if self.actuator_config.is_none() || self.device_config.is_none() {
            return Err(Error::NotConfigured);
        }
        let device_config = self.device_config.unwrap();

        let mut top_ctl1 = TOP_CTL1::from(self.read_register(Register::TOP_CTL1).await?);
        top_ctl1 = top_ctl1.with_OPERATION_MODE(OperationMode::INACTIVE as u8);
        self.write_register(Register::TOP_CTL1, top_ctl1.into())
            .await?;

        Ok(())
    }

    /// Sets a custom drive waveform, see 5.7.6 Custom Waveform Operation
    /// Device needs to be in the CUSTOM_WAVEFORM mode.
    pub async fn set_custom_drive_waveform(&mut self, points: [u8; 3]) -> Result<(), Error> {
        if self.actuator_config.is_none() || self.device_config.is_none() {
            return Err(Error::NotConfigured);
        }
        let device_config = self.device_config.as_ref().unwrap();

        if device_config.driving_mode != DrivingMode::CUSTOM_WAVEFORM {
            return Err(Error::WrongMode);
        }

        // TODO

        Ok(())
    }

    async fn read_register(&mut self, register: Register) -> Result<u8, Error> {
        let mut buffer = [0u8; 1];

        self.i2c
            .write_read(self.address, &[register as u8], &mut buffer)
            .await
            .map_err(|e| Error::I2c(e.kind()))?;

        Ok(buffer[0])
    }

    async fn write_register(&mut self, register: Register, data: u8) -> Result<(), Error> {
        self.i2c
            .write(self.address, &[register as u8, data])
            .await
            .map_err(|e| Error::I2c(e.kind()))
    }

    /// Write multiple bytes to consecutive memory addresses starting at SNP_MEM_0.
    ///
    /// This is used for uploading waveform memory data.
    async fn write_memory_bytes(&mut self, data: &[u8]) -> Result<(), Error> {
        // Write in chunks to avoid buffer overflow
        // Most I2C implementations have limited buffer sizes
        const CHUNK_SIZE: usize = 32;

        let mut offset = 0;
        while offset < data.len() {
            let chunk_end = (offset + CHUNK_SIZE).min(data.len());
            let chunk = &data[offset..chunk_end];

            // Create buffer with register address followed by data
            // We use a fixed-size buffer since we're no_std
            // Waveform memory starts at SNP_MEM_0 (0x84)
            let mut buffer = [0u8; CHUNK_SIZE + 1];
            buffer[0] = Register::SNP_MEM_0 as u8 + offset as u8;
            buffer[1..1 + chunk.len()].copy_from_slice(chunk);

            self.i2c
                .write(self.address, &buffer[..1 + chunk.len()])
                .await
                .map_err(|e| Error::I2c(e.kind()))?;

            offset = chunk_end;
        }

        Ok(())
    }

    /// Unlock waveform memory for writing.
    ///
    /// Must be called before uploading waveform memory.
    /// Per datasheet: WAV_MEM_LOCK = 1 means unlocked, 0 means locked.
    pub async fn unlock_waveform_memory(&mut self) -> Result<(), Error> {
        let mem_ctl2 = MEM_CTL2::new().with_WAV_MEM_LOCK(true); // 1 = unlocked
        self.write_register(Register::MEM_CTL2, mem_ctl2.into())
            .await
    }

    /// Lock waveform memory to prevent accidental writes.
    ///
    /// Should be called after uploading waveform memory.
    /// Per datasheet: WAV_MEM_LOCK = 0 means locked, 1 means unlocked.
    pub async fn lock_waveform_memory(&mut self) -> Result<(), Error> {
        let mem_ctl2 = MEM_CTL2::new().with_WAV_MEM_LOCK(false); // 0 = locked
        self.write_register(Register::MEM_CTL2, mem_ctl2.into())
            .await
    }

    /// Upload waveform memory to the device.
    ///
    /// # Arguments
    /// * `memory` - The waveform memory to upload
    /// * `lock_after` - Whether to lock memory after upload
    ///
    /// # Errors
    /// Returns an I2C error if communication fails.
    pub async fn upload_waveform_memory(
        &mut self,
        memory: &WaveformMemory,
        lock_after: bool,
    ) -> Result<(), Error> {
        // Unlock memory first
        self.unlock_waveform_memory().await?;

        // Write the waveform data
        self.write_memory_bytes(memory.as_bytes()).await?;

        // Optionally lock memory
        if lock_after {
            self.lock_waveform_memory().await?;
        }

        Ok(())
    }

    /// Read back waveform memory contents.
    ///
    /// # Arguments
    /// * `len` - Number of bytes to read (max 100)
    /// * `buffer` - Buffer to store the read data
    ///
    /// # Returns
    /// Number of bytes actually read.
    pub async fn read_waveform_memory(
        &mut self,
        len: usize,
        buffer: &mut [u8],
    ) -> Result<usize, Error> {
        let read_len = len.min(100).min(buffer.len());
        for i in 0..read_len {
            let addr = Register::SNP_MEM_0 as u8 + i as u8;
            let mut data = [0u8; 1];
            self.i2c
                .write_read(self.address, &[addr], &mut data)
                .await
                .map_err(|e| Error::I2c(e.kind()))?;
            buffer[i] = data[0];
        }
        Ok(read_len)
    }

    /// Select a sequence for playback.
    ///
    /// # Arguments
    /// * `sequence_id` - Sequence ID (0-15)
    /// * `loops` - Number of times to loop (0-15, where 0 means play once)
    ///
    /// # Errors
    /// Returns `InvalidValue` if parameters are out of range.
    pub async fn select_sequence(&mut self, sequence_id: u8, loops: u8) -> Result<(), Error> {
        if sequence_id > 15 || loops > 15 {
            return Err(Error::InvalidValue);
        }

        let seq_ctl2 = SEQ_CTL2::new()
            .with_PS_SEQ_ID(sequence_id)
            .with_PS_SEQ_LOOP(loops);
        self.write_register(Register::SEQ_CTL2, seq_ctl2.into())
            .await
    }

    /// Start playback of the selected sequence.
    ///
    /// The device must be in RTWM_MODE and enabled for this to work.
    pub async fn start_sequence(&mut self) -> Result<(), Error> {
        let mut top_ctl1 = TOP_CTL1::from(self.read_register(Register::TOP_CTL1).await?);
        top_ctl1 = top_ctl1.with_SEQ_START(true);
        self.write_register(Register::TOP_CTL1, top_ctl1.into())
            .await
    }

    /// Select and immediately start playing a sequence.
    ///
    /// This is a convenience method that combines `select_sequence` and `start_sequence`.
    ///
    /// # Arguments
    /// * `sequence_id` - Sequence ID (0-15)
    /// * `loops` - Number of times to loop (0-15, where 0 means play once)
    pub async fn play_sequence(&mut self, sequence_id: u8, loops: u8) -> Result<(), Error> {
        self.select_sequence(sequence_id, loops).await?;
        self.start_sequence().await
    }
}
