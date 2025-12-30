#![deny(unsafe_code)]
#![allow(unused)]
#![no_std]
#![doc = include_str!("../README.md")]

pub mod config;
pub mod errors;
pub mod registers;

use embedded_hal_async::i2c::Error as I2cError;
use embedded_hal_async::i2c::I2c;

#[cfg(feature = "debug")]
use defmt::{debug, info};

use config::{ActuatorConfig, DeviceConfig, DrivingMode, OperationMode};
use errors::Error;
use registers::Register;
use registers::{CHIP_REV, ACTUATOR1, ACTUATOR2, ACTUATOR3, TOP_CTL1, TOP_CFG1, CALIB_V2I_H, CALIB_V2I_L, FRQ_LRA_PER_H, FRQ_LRA_PER_L, IRQ_STATUS1, IRQ_EVENT1, IRQ_EVENT_WARNING_DIAG, IRQ_EVENT_SEQ_DIAG, FRQ_PHASE_H, FRQ_PHASE_L};

use crate::registers::SEQ_CTL1;
use crate::registers::TOP_CFG4;
use crate::registers::TOP_CTL2;

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

impl<I2C> DA728x<I2C>
where
    I2C: I2c,
{
    pub async fn new(i2c: I2C, address: u8, variant: Variant) -> Result<Self, Error>
    where
        I2C: I2c,
    {
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
        if device_config.driving_mode != DrivingMode::FREQUENCY_TRACK &&
            (device_config.acceleration || device_config.rapid_stop) {
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
                if !(50..300).contains(&actuator_config.frequency_Hz) {
                    return Err(Error::InvalidValue);
                }
            }
            DrivingMode::WIDEBAND | DrivingMode::CUSTOM_WAVEFORM => {
                if !(50..300).contains(&actuator_config.frequency_Hz) {
                    return Err(Error::InvalidValue);
                }
            }
        }

        // Figure out feature flags depending on actuator type and driving mode
        let (bemf_sense_en,frequency_track_en, acceleration_en, rapid_stop_en);

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
        let top_cfg1 = TOP_CFG1::new()
        .with_ACTUATOR_TYPE(actuator_config.actuator_type as u8)
        .with_BEMF_SENSE_EN(bemf_sense_en)
        .with_FREQ_TRACK_EN(frequency_track_en)
        .with_ACCELERATION_EN(acceleration_en)
        .with_RAPID_STOP_EN(rapid_stop_en)
        .with_AMP_PID_EN(false); // Only supported with ERMs, disable for now.
        self.write_register(Register::TOP_CFG1, top_cfg1.into()).await?;

        // ACTUATOR1 (nom max volt)
        let volt_converted = ((actuator_config.nominal_max_mV as u32 * 1000) / 23400) as u8;
        let actuator1 = ACTUATOR1::from(volt_converted);
        self.write_register(Register::ACTUATOR1, actuator1.into()).await?;


        // ACTUATOR2 (as max volt)
        let volt_converted = ((actuator_config.absolute_max_mV as u32 * 1000) / 23400) as u8; // +1?
        let actuator2 = ACTUATOR2::from(volt_converted);
        self.write_register(Register::ACTUATOR2, actuator2.into()).await?;

        // ACTUATOR3 (imax)
        let current_converted = ((actuator_config.max_current_mA as u32 * 1000 - 28600) / 7200) as u8; // +1?
        let current_converted_clone = current_converted as u32;
        let actuator3 = ACTUATOR3::new().with_IMAX(current_converted);
        self.write_register(Register::ACTUATOR3, actuator3.into()).await?;

        // CALIB_V2I_L / CALIB_V2I_H (impedance)
        let impedance_converted = ((actuator_config.impedance_mOhm as u32 * 1000 * (current_converted_clone + 4)) / 1610400) as u16;
        let bytes: [u8; 2] = impedance_converted.to_be_bytes();
        let calib_v2i_h = CALIB_V2I_H::from(bytes[0]);
        let calib_v2i_l = CALIB_V2I_L::from(bytes[1]);
        self.write_register(Register::CALIB_V2I_H, calib_v2i_h.into()).await?;
        self.write_register(Register::CALIB_V2I_L, calib_v2i_l.into()).await?;

        // Default resonant frequency
        let frequency_converted =  (1000000000 / (actuator_config.frequency_Hz as u32 * 1333)) as u16;
        let frequency_converted_h: u8 = ((frequency_converted >> 7) & 0xFF) as u8;
        let frequency_converted_l: u8 = (frequency_converted & 0x7F) as u8;
        let frq_lra_per_h = FRQ_LRA_PER_H::from(frequency_converted_h);
        let frq_lra_per_l = FRQ_LRA_PER_L::new().with_LRA_PER_L(frequency_converted_l);
        self.write_register(Register::FRQ_LRA_PER_H, frq_lra_per_h.into()).await?;
        self.write_register(Register::FRQ_LRA_PER_L, frq_lra_per_l.into()).await?;

        // Additional configuration depending on DrivingMode

        // WIDEBAND MODE
        // FREQ_TRACK_EN = 0, ACCELERATION_EN = 0, RAPID_STOP_EN = 0, BEMF_SENSE_EN = 0
        // DELAY_H = 0, DELAY_SHIFT_L = 0, DELAY_FREEZE = 1

        // CUSTOM WAVEFORM MODE
        // FREQ_TRACK_EN = 0, ACCELERATION_EN = 0, RAPID_STOP_EN = 0, BEMF_SENSE_EN = 0, AMP_PID_EN = 0
        // DELAY_H = 0, DELAY_SHIFT_L = 0, DELAY_FREEZE = 1
        // WAVEGEN_MODE = 1, V2I_FACTOR_FREEZE = 1

        if device_config.driving_mode == DrivingMode::WIDEBAND || device_config.driving_mode == DrivingMode::CUSTOM_WAVEFORM {
            let frq_phase_h = FRQ_PHASE_H::from(0x00);
            let frq_phase_l = FRQ_PHASE_L::new().with_DELAY_SHIFT_L(0x00).with_DELAY_FREEZE(true);
            self.write_register(Register::FRQ_PHASE_H, frq_phase_h.into());
            self.write_register(Register::FRQ_PHASE_L, frq_phase_l.into());
        }

        if device_config.driving_mode == DrivingMode::CUSTOM_WAVEFORM {
            let seq_ctl1 = SEQ_CTL1::new().with_WAVEGEN_MODE(true);
            let top_cfg4 = TOP_CFG4::new().with_V2I_FACTOR_FREEZE(true); // Unclear if TST_CALIB_IMPEDANCE_DIS should be true/false.
            self.write_register(Register::SEQ_CTL1, seq_ctl1.into());
            self.write_register(Register::TOP_CFG4, top_cfg4.into());
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
    pub async fn get_events(&mut self) -> Result<(IRQ_EVENT1, IRQ_EVENT_WARNING_DIAG, IRQ_EVENT_SEQ_DIAG), Error> {
        let irq_event1 = IRQ_EVENT1::from(self.read_register(Register::IRQ_EVENT1).await?);
        let irq_event_warning_diag = IRQ_EVENT_WARNING_DIAG::from(self.read_register(Register::IRQ_EVENT_WARNING_DIAG).await?);
        let irq_event_seq_diag = IRQ_EVENT_SEQ_DIAG::from(self.read_register(Register::IRQ_EVENT_SEQ_DIAG).await?);

        // Clear events (only IRQ_EVENT1)
        self.write_register(Register::IRQ_EVENT1, 0xFF);

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
                if !(50..300).contains(&frequency_hz) {
                    return Err(Error::InvalidValue);
                }
            }
            DrivingMode::WIDEBAND | DrivingMode::CUSTOM_WAVEFORM => {
                if !(25..1024).contains(&frequency_hz) {
                    return Err(Error::InvalidValue);
                }
            }
        }

        let frequency_converted =  (1000000000 / (frequency_hz as u32 * 1333)) as u16;
        let frequency_converted_h: u8 = ((frequency_converted >> 7) & 0xFF) as u8;
        let frequency_converted_l: u8 = (frequency_converted & 0x7F) as u8;
        let frq_lra_per_h = FRQ_LRA_PER_H::from(frequency_converted_h);
        let frq_lra_per_l = FRQ_LRA_PER_L::new().with_LRA_PER_L(frequency_converted_l);
        self.write_register(Register::FRQ_LRA_PER_H, frq_lra_per_h.into()).await?;
        self.write_register(Register::FRQ_LRA_PER_L, frq_lra_per_l.into()).await?;

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
        self.write_register(Register::TOP_CTL2, top_ctl_2.into()).await?;

        Ok(())
    }

    /// Enable the configured operation mode
    pub async fn enable(&mut self) -> Result<(), Error> {
        if self.actuator_config.is_none() || self.device_config.is_none() {
            return Err(Error::NotConfigured);
        }
        let device_config = self.device_config.unwrap();

        let mut top_ctl1 = TOP_CTL1::from(self.read_register(Register::TOP_CTL1).await?);
        #[cfg(feature = "debug")]
        debug!("TOP_CTL1: {:?}", top_ctl1);
        top_ctl1 = top_ctl1.with_OPERATION_MODE(device_config.operation_mode as u8);
        self.write_register(Register::TOP_CTL1, top_ctl1.into()).await?;

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
        self.write_register(Register::TOP_CTL1, top_ctl1.into()).await?;

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
}
