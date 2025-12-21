#![deny(unsafe_code)]
#![no_std]
#![doc = include_str!("../README.md")]

pub mod errors;
pub mod registers;

use embedded_hal::digital::Error as DigitalError;
use embedded_hal::digital::InputPin;
use embedded_hal_async::delay::DelayNs;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::i2c::Error as I2cError;
use embedded_hal_async::i2c::I2c;

use errors::Error;
use registers::Register;

#[cfg(feature = "debug")]
use log::{debug, info};

use crate::registers::CHIP_REV;

pub enum Variant {
    DA7280 = 0xBA,
    DA7281 = 0xCA,
    DA7282 = 0xDA,
}

pub struct DA728x<I2C, INT, DELAY> {
    i2c: I2C,
    address: u8,
    int_pin: INT,
    delay: DELAY,
    variant: Variant,
}

impl<I2C, INT, DELAY> DA728x<I2C, INT, DELAY>
where
    I2C: I2c,
    INT: Wait,
    DELAY: DelayNs,
{
    pub async fn new(
        mut i2c: I2C,
        address: u8,
        mut int_pin: INT,
        delay: DELAY,
        variant: Variant,
    ) -> Result<Self, Error>
    where
        I2C: I2c,
        INT: Wait,
        DELAY: DelayNs,
    {
        let mut da728x = DA728x {
            i2c,
            address,
            int_pin,
            delay,
            variant,
        };

        let chip_rev = da728x.get_chip_rev().await?;

        match da728x.variant {
            Variant::DA7280 => {
                if chip_rev.CHIP_REV_MINOR() != 0xB && chip_rev.CHIP_REV_MAJOR() != 0xA {
                    return Err(Error::VariantMismatch);
                }
            },
            Variant::DA7281 => {
                if chip_rev.CHIP_REV_MINOR() != 0xC && chip_rev.CHIP_REV_MAJOR() != 0xA {
                    return Err(Error::VariantMismatch);
                }
            },
            Variant::DA7282 => {
                if chip_rev.CHIP_REV_MINOR() != 0xD && chip_rev.CHIP_REV_MAJOR() != 0xA {
                    return Err(Error::VariantMismatch);
                }
            },
        }

        Ok(da728x)
    }

    pub async fn get_chip_rev(&mut self) -> Result<registers::CHIP_REV, Error> {
        let reg = self.read_register(Register::CHIP_REV).await?;
        Ok(CHIP_REV::from(reg))
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
            .write(self.address, &[register as u8])
            .await
            .map_err(|e| Error::I2c(e.kind()))
    }
}
