#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use bitfield_struct::bitfield;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Register {
    CHIP_REV = 0x00,
}

#[bitfield(u8)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
pub struct CHIP_REV {
    #[bits(4, access = RO)]
    pub CHIP_REV_MINOR: u8,
    #[bits(4, access = RO)]
    pub CHIP_REV_MAJOR: u8,
}