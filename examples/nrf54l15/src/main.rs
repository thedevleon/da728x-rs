#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

use da728x::DA728x;

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let config = twim::Config::default();
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = Twim::new(p.SERIAL20, Irqs, p.P1_10, p.P1_11, config, RAM_BUFFER.take());

    info!("Setting up haptics IC...");
    let mut haptics = DA728x::new(twi, 0x4A, da728x::Variant::DA7280).await.unwrap();
    
    info!("Haptics IC setup successfully.");

    let rev = haptics.get_chip_rev().await.unwrap();
    info!("Chip ID: {:x} {:x}", rev.CHIP_REV_MAJOR(), rev.CHIP_REV_MINOR());
}
