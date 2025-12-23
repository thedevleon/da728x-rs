#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Timer;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

use da728x::{DA728x, Variant};
use da728x::config::{ActuatorConfig, ActuatorType, DeviceConfig, OperationMode, DrivingMode};

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let config = twim::Config::default();
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = Twim::new(
        p.SERIAL20,
        Irqs,
        p.P1_10,
        p.P1_11,
        config,
        RAM_BUFFER.take(),
    );

    info!("Setting up haptics IC...");
    let mut haptics = DA728x::new(twi, 0x4A, Variant::DA7280)
        .await
        .unwrap();

    info!("Haptics IC setup successfully.");

    // Configure the IC
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 2_106,
        absolute_max_mV: 2_260,
        max_current_mA: 165,
        impedance_mOhm: 13_800,
        frequency_Hz: 170,
    };

    let device_config = DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::WIDEBAND,
        acceleration: false,
        rapid_stop: false,
    };

    haptics.configure(actuator_config, device_config).await.unwrap();
    info!("Haptics IC configured successfully.");

    let status = haptics.get_status().await.unwrap();
    info!("Haptics Status: {:?}", status);

    haptics.enable().await.unwrap();

    loop {
        haptics.set_frequency(170).await.unwrap();

        info!("100%");
        haptics.set_override_value(127).await.unwrap();
        Timer::after_millis(800).await;
        info!("33%");
        haptics.set_override_value(42).await.unwrap();
        Timer::after_millis(800).await;
        info!("0%");
        haptics.set_override_value(0).await.unwrap();

        Timer::after_millis(2_000).await;

        // Tetris melody notes (frequency in Hz, duration in ms)
        // Is not 100% correct
        let tetris_melody = [
            (659, 150), (494, 75), (523, 75), (587, 150), (523, 75), (494, 75),
            (440, 150), (440, 75), (523, 75), (659, 150), (587, 75), (523, 75),
            (494, 225), (523, 75), (587, 150), (659, 150),
            (523, 150), (440, 150), (440, 150),
        ];

        for &(freq, dur) in tetris_melody.iter() {
            haptics.set_frequency(freq).await.unwrap();
            haptics.set_override_value(127).await.unwrap();
            Timer::after_millis(dur).await;
            haptics.set_override_value(0).await.unwrap();
            Timer::after_millis(90).await;
        }

        Timer::after_millis(2_000).await;

        let status = haptics.get_status().await.unwrap();
        info!("Haptics Status: {:?}", status);
    }
}
