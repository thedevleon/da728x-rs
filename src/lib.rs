#![deny(unsafe_code)]
#![no_std]
#![doc = include_str!("../README.md")]

pub mod errors;
pub mod registers;
pub mod types;

// Re-export commonly used types at the crate root
pub use errors::Error;
pub use types::{
    DeviceConfig, DeviceType, GpiConfig, GpiMode, GpiPolarity, OperationMode,
    SNP_MEM_SIZE,
};

use embedded_hal::i2c::Error as I2cError;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::I2c;

use registers::*;
use types::*;

#[cfg(feature = "debug")]
use log::{debug, info};

/// DA728x chip variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    DA7280 = 0xBA,
    DA7281 = 0xCA,
    DA7282 = 0xDA,
}

/// Main driver structure for DA728x haptic driver
pub struct DA728x<I2C, INT, DELAY> {
    i2c: I2C,
    address: u8,
    #[allow(dead_code)]
    int_pin: INT,
    #[allow(dead_code)]
    delay: DELAY,
    variant: Variant,
    config: DeviceConfig,
    active: bool,
}

impl<I2C, INT, DELAY> DA728x<I2C, INT, DELAY>
where
    I2C: I2c,
    INT: Wait,
    DELAY: DelayNs,
{
    /// Create a new DA728x driver instance
    ///
    /// This initializes the device and verifies the chip variant matches expectations.
    ///
    /// # Arguments
    /// * `i2c` - I2C bus interface
    /// * `address` - I2C device address (typically 0x4A or 0x4B)
    /// * `int_pin` - Interrupt pin
    /// * `delay` - Delay provider
    /// * `variant` - Expected chip variant
    /// * `config` - Device configuration
    pub async fn new(
        i2c: I2C,
        address: u8,
        int_pin: INT,
        delay: DELAY,
        variant: Variant,
        config: DeviceConfig,
    ) -> Result<Self, Error> {
        let mut da728x = DA728x {
            i2c,
            address,
            int_pin,
            delay,
            variant,
            config,
            active: false,
        };

        // Verify chip revision matches variant
        let chip_rev = da728x.get_chip_rev().await?;

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

        // Initialize the device with the configuration
        da728x.init().await?;

        Ok(da728x)
    }

    /// Get the chip revision
    pub async fn get_chip_rev(&mut self) -> Result<CHIP_REV, Error> {
        let reg = self.read_register(Register::CHIP_REV).await?;
        Ok(CHIP_REV::from(reg))
    }

    /// Initialize the device with the current configuration
    async fn init(&mut self) -> Result<(), Error> {
        // If device type is not set, read it from the chip
        let mut dev_type = self.config.dev_type;
        if dev_type.is_none() {
            let val = self.read_register(Register::TOP_CFG1).await?;
            let cfg = TOP_CFG1::from(val);
            dev_type = Some(if cfg.ACTUATOR_TYPE() {
                DeviceType::ErmCoin
            } else {
                DeviceType::LRA
            });
            self.config.dev_type = dev_type;
        }

        let dev_type = dev_type.unwrap();

        // Apply LRA-specific settings
        if dev_type == DeviceType::LRA && self.config.resonant_freq_l != SKIP_INIT {
            self.write_register(Register::FRQ_LRA_PER_H, self.config.resonant_freq_h as u8)
                .await?;
            self.write_register(Register::FRQ_LRA_PER_L, self.config.resonant_freq_l as u8)
                .await?;
        } else if dev_type == DeviceType::ErmCoin {
            // ERM coin specific configuration
            self.update_register(Register::TOP_INT_CFG1, |val| {
                let mut cfg = TOP_INT_CFG1::from(val);
                cfg.set_BEMF_FAULT_LIM(0);
                cfg.into()
            })
            .await?;

            self.update_register(Register::TOP_CFG4, |val| {
                let mut cfg = TOP_CFG4::from(val);
                cfg.set_TST_CALIB_IMPEDANCE_DIS(true);
                cfg.set_V2I_FACTOR_FREEZE(true);
                cfg.into()
            })
            .await?;

            // Disable certain features for ERM coin
            self.config.acc_en = false;
            self.config.rapid_stop_en = false;
            self.config.amp_pid_en = false;
        }

        // Configure TOP_CFG1
        let bemf_sense_en = self.config.bemf_sense_en;
        let freq_track_en = self.config.freq_track_en;
        let acc_en = self.config.acc_en;
        let rapid_stop_en = self.config.rapid_stop_en;
        let amp_pid_en = self.config.amp_pid_en;

        self.update_register(Register::TOP_CFG1, |val| {
            let mut cfg = TOP_CFG1::from(val);
            cfg.set_ACTUATOR_TYPE(dev_type.to_bit());
            cfg.set_BEMF_SENSE_EN(bemf_sense_en);
            cfg.set_FREQ_TRACK_EN(freq_track_en);
            cfg.set_ACCELERATION_EN(acc_en);
            cfg.set_RAPID_STOP_EN(rapid_stop_en);
            cfg.set_AMP_PID_EN(amp_pid_en);
            cfg.into()
        })
        .await?;

        // Configure TOP_CFG5
        self.update_register(Register::TOP_CFG5, |val| {
            let mut cfg = TOP_CFG5::from(val);
            cfg.set_V2I_FACTOR_OFFSET_EN(acc_en);
            cfg.into()
        })
        .await?;

        // Configure TOP_CFG2
        self.update_register(Register::TOP_CFG2, |val| {
            let mut cfg = TOP_CFG2::from(val);
            cfg.set_MEM_DATA_SIGNED(!acc_en);
            cfg.into()
        })
        .await?;

        // Set nominal maximum voltage
        if self.config.nommax != SKIP_INIT {
            self.write_register(Register::ACTUATOR1, self.config.nommax as u8)
                .await?;
        }

        // Set absolute maximum voltage
        if self.config.absmax != SKIP_INIT {
            self.write_register(Register::ACTUATOR2, self.config.absmax as u8)
                .await?;
        }

        // Set IMAX
        let imax_val = self.config.imax.min(0x1F);
        self.update_register(Register::ACTUATOR3, |val| {
            let mut cfg = ACTUATOR3::from(val);
            cfg.set_IMAX(imax_val as u8);
            cfg.into()
        })
        .await?;

        // Calculate and set V2I factor
        let v2i_factor = (self.config.impd * (self.config.imax + 4)) / 1_610_400;
        self.write_register(Register::CALIB_V2I_L, (v2i_factor & 0xFF) as u8)
            .await?;
        self.write_register(Register::CALIB_V2I_H, (v2i_factor >> 8) as u8)
            .await?;

        // Enable standby
        self.update_register(Register::TOP_CTL1, |val| {
            let mut cfg = TOP_CTL1::from(val);
            cfg.set_STANDBY_EN(true);
            cfg.into()
        })
        .await?;

        // Update memory if provided
        let mem_data = self.config.mem_data;
        if let Some(mem_data) = mem_data {
            self.mem_update(&mem_data).await?;
        }

        // Set PS_SEQ_ID and PS_SEQ_LOOP
        let mut seq_ctl = SEQ_CTL2::new();
        seq_ctl.set_PS_SEQ_ID(self.config.ps_seq_id);
        seq_ctl.set_PS_SEQ_LOOP(self.config.ps_seq_loop);
        self.write_register(Register::SEQ_CTL2, seq_ctl.into())
            .await?;

        // Configure GPI pins
        for i in 0..3 {
            let mut gpi_ctl = GPI_CTL::new();
            gpi_ctl.set_SEQUENCE_ID(self.config.gpi_ctl[i].seq_id);
            gpi_ctl.set_MODE(self.config.gpi_ctl[i].mode.to_register_value());
            gpi_ctl.set_POLARITY(self.config.gpi_ctl[i].polarity.to_register_value());

            let reg = match i {
                0 => Register::GPI_0_CTL,
                1 => Register::GPI_1_CTL,
                2 => Register::GPI_2_CTL,
                _ => unreachable!(),
            };
            self.write_register(reg, gpi_ctl.into()).await?;
        }

        // Mask ADC_SAT_M bit as default
        self.update_register(Register::IRQ_MASK2, |val| {
            let mut cfg = IRQ_MASK2::from(val);
            cfg.set_ADC_SAT_M(true);
            cfg.into()
        })
        .await?;

        // Clear interrupts
        self.write_register(Register::IRQ_EVENT1, 0xFF).await?;

        // Unmask SEQ_FAULT and SEQ_DONE interrupts
        self.update_register(Register::IRQ_MASK1, |val| {
            let mut cfg = IRQ_MASK1::from(val);
            cfg.set_SEQ_FAULT_M(false);
            cfg.set_SEQ_DONE_M(false);
            cfg.into()
        })
        .await?;

        self.active = false;

        Ok(())
    }

    /// Update waveform memory
    async fn mem_update(&mut self, mem_data: &[u8; SNP_MEM_SIZE]) -> Result<(), Error> {
        // Check if device is busy
        let status = self.read_register(Register::IRQ_STATUS1).await?;
        let status = IRQ_STATUS1::from(status);
        if status.STA_WARNING() {
            return Err(Error::DeviceBusy);
        }

        // Check if memory is locked (lock bit should be 1 to allow updates)
        let mem_ctl2 = self.read_register(Register::MEM_CTL2).await?;
        let mem_ctl2 = MEM_CTL2::from(mem_ctl2);
        if !mem_ctl2.WAV_MEM_LOCK() {
            return Err(Error::MemoryLocked);
        }

        // Set to inactive mode for safety
        self.update_register(Register::TOP_CTL1, |val| {
            let mut cfg = TOP_CTL1::from(val);
            cfg.set_OPERATION_MODE(OperationMode::Inactive.to_register_value());
            cfg.into()
        })
        .await?;

        // Get memory start address
        let mem_ctl1 = self.read_register(Register::MEM_CTL1).await?;

        // Write memory data
        self.bulk_write(mem_ctl1, mem_data).await?;

        Ok(())
    }

    /// Set the operation mode and activate the device
    pub async fn activate(&mut self, mode: OperationMode) -> Result<(), Error> {
        if self.active {
            return Ok(());
        }

        // Set operation mode
        self.update_register(Register::TOP_CTL1, |val| {
            let mut cfg = TOP_CTL1::from(val);
            cfg.set_OPERATION_MODE(mode.to_register_value());
            cfg.into()
        })
        .await?;

        // Start sequence for PWM and RTWM modes
        if mode == OperationMode::PWM || mode == OperationMode::RTWM {
            self.update_register(Register::TOP_CTL1, |val| {
                let mut cfg = TOP_CTL1::from(val);
                cfg.set_SEQ_START(true);
                cfg.into()
            })
            .await?;
        }

        self.active = true;

        Ok(())
    }

    /// Deactivate the device
    pub async fn deactivate(&mut self) -> Result<(), Error> {
        if !self.active {
            return Ok(());
        }

        // Set to inactive mode
        self.update_register(Register::TOP_CTL1, |val| {
            let mut cfg = TOP_CTL1::from(val);
            cfg.set_OPERATION_MODE(OperationMode::Inactive.to_register_value());
            cfg.into()
        })
        .await?;

        // Clear sequence start bit for RTWM/ETWM
        self.update_register(Register::TOP_CTL1, |val| {
            let mut cfg = TOP_CTL1::from(val);
            cfg.set_SEQ_START(false);
            cfg.into()
        })
        .await?;

        self.active = false;

        Ok(())
    }

    /// Set the DRO mode level (0-255 or 0-127 if acceleration is enabled)
    pub async fn set_dro_level(&mut self, level: u8) -> Result<(), Error> {
        let level = if self.config.acc_en {
            level.min(0x7F)
        } else {
            level
        };

        self.write_register(Register::TOP_CTL2, level).await?;

        Ok(())
    }

    /// Set the pre-stored sequence parameters
    pub async fn set_ps_sequence(&mut self, seq_id: u8, seq_loop: u8) -> Result<(), Error> {
        if seq_id > SEQ_ID_MAX || seq_loop > SEQ_LOOP_MAX {
            return Err(Error::InvalidParameter);
        }

        let mut seq_ctl = SEQ_CTL2::new();
        seq_ctl.set_PS_SEQ_ID(seq_id);
        seq_ctl.set_PS_SEQ_LOOP(seq_loop);
        self.write_register(Register::SEQ_CTL2, seq_ctl.into())
            .await?;

        Ok(())
    }

    /// Set GPI sequence ID
    pub async fn set_gpi_sequence(&mut self, gpi_num: u8, seq_id: u8) -> Result<(), Error> {
        if gpi_num > GPI_SEQ_ID_MAX || seq_id > SEQ_ID_MAX {
            return Err(Error::InvalidParameter);
        }

        let reg = match gpi_num {
            0 => Register::GPI_0_CTL,
            1 => Register::GPI_1_CTL,
            2 => Register::GPI_2_CTL,
            _ => return Err(Error::InvalidParameter),
        };

        self.update_register(reg, |val| {
            let mut cfg = GPI_CTL::from(val);
            cfg.set_SEQUENCE_ID(seq_id);
            cfg.into()
        })
        .await?;

        Ok(())
    }

    /// Read IRQ events
    pub async fn read_irq_events(&mut self) -> Result<(IRQ_EVENT1, IRQ_EVENT_WARNING_DIAG, IRQ_EVENT_SEQ_DIAG), Error> {
        let mut events = [0u8; 3];
        self.bulk_read(Register::IRQ_EVENT1 as u8, &mut events)
            .await?;

        Ok((
            IRQ_EVENT1::from(events[0]),
            IRQ_EVENT_WARNING_DIAG::from(events[1]),
            IRQ_EVENT_SEQ_DIAG::from(events[2]),
        ))
    }

    /// Clear IRQ event 1 register
    /// 
    /// Note: This only clears the IRQ_EVENT1 register. In the C driver,
    /// only this register is cleared as the other event registers are
    /// automatically cleared or are diagnostic status registers.
    pub async fn clear_irq_events(&mut self) -> Result<(), Error> {
        self.write_register(Register::IRQ_EVENT1, 0xFF).await
    }

    /// Read IRQ status
    pub async fn read_irq_status(&mut self) -> Result<IRQ_STATUS1, Error> {
        let val = self.read_register(Register::IRQ_STATUS1).await?;
        Ok(IRQ_STATUS1::from(val))
    }

    /// Check if device is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get current configuration
    pub fn config(&self) -> &DeviceConfig {
        &self.config
    }

    // Low-level I2C operations

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

    async fn update_register<F>(&mut self, register: Register, f: F) -> Result<(), Error>
    where
        F: FnOnce(u8) -> u8,
    {
        let val = self.read_register(register).await?;
        let new_val = f(val);
        self.write_register(register, new_val).await
    }

    async fn bulk_read(&mut self, start_addr: u8, buffer: &mut [u8]) -> Result<(), Error> {
        self.i2c
            .write_read(self.address, &[start_addr], buffer)
            .await
            .map_err(|e| Error::I2c(e.kind()))
    }

    async fn bulk_write(&mut self, start_addr: u8, data: &[u8]) -> Result<(), Error> {
        // Create a temporary buffer with address + data
        let mut buffer = [0u8; SNP_MEM_SIZE + 1];
        buffer[0] = start_addr;
        buffer[1..=data.len()].copy_from_slice(data);

        self.i2c
            .write(self.address, &buffer[..=data.len()])
            .await
            .map_err(|e| Error::I2c(e.kind()))
    }
}
