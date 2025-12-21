#![deny(unsafe_code)]
#![no_std]
#![doc = include_str!("../README.md")]

pub mod errors;
pub mod registers;

use embedded_hal_async::i2c::Error as I2cError;
use embedded_hal_async::i2c::I2c;

#[cfg(feature = "debug")]
use defmt::{debug, info};

use errors::Error;
use registers::Register;
use registers::CHIP_REV;

pub enum Variant {
    DA7280 = 0xBA,
    DA7281 = 0xCA,
    DA7282 = 0xDA,
}

pub struct DA728x<I2C> {
    i2c: I2C,
    address: u8,
    variant: Variant,
}

impl<I2C> DA728x<I2C>
where
    I2C: I2c,
{
    pub async fn new(
        i2c: I2C,
        address: u8,
        variant: Variant,
    ) -> Result<Self, Error>
    where
        I2C: I2c,
    {
        let mut da728x = DA728x {
            i2c,
            address,
            variant,
        };

        let chip_rev = da728x.get_chip_rev().await?;

        #[cfg(feature = "debug")]
        debug!("CHIP_REV = {:x} {:x}", chip_rev.CHIP_REV_MAJOR(), chip_rev.CHIP_REV_MINOR());

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
            .write(self.address, &[register as u8, data])
            .await
            .map_err(|e| Error::I2c(e.kind()))
    }
}
