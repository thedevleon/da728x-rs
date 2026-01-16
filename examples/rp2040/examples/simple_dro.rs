//! Simple DRO (Direct Register Override) example for DA7280.
//!
//! This example demonstrates basic haptic pulses using DRO mode with
//! frequency tracking. This is the simplest way to generate haptic feedback.
//!
//! # Hardware Setup
//!
//! This example is designed for the **SparkFun Qwiic Haptic Driver (DA7280)**
//! connected to an RP2040 board via I2C.
//!
//! - DA7280 connected via I2C0 (SDA: GP16, SCL: GP17)
//! - I2C address: 0x4A (default)
//!
//! # Important: Actuator Loading
//!
//! The LRA actuator **must be mechanically loaded** (compressed between two
//! surfaces) for proper operation. When unloaded:
//!
//! - The actuator may not vibrate properly
//! - You will see `ACTUATOR FAULT` warnings in the logs
//! - This is expected behavior - the DA7280 uses back-EMF sensing to detect
//!   abnormal actuator conditions
//!
//! To test properly, place the haptic motor between two solid objects (e.g.,
//! hold it pressed against a table with your finger).
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example simple_dro
//! ```

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::i2c::{self, Config, I2c};
use embassy_rp::peripherals::I2C0;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use da728x::config::{ActuatorConfig, ActuatorType, DeviceConfig, DrivingMode, OperationMode};
use da728x::{Variant, DA728x};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== Simple DRO Example ===");
    info!("Make sure the actuator is loaded (pressed between two surfaces)!");

    // Initialize I2C
    info!("Initializing I2C...");
    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let mut config = Config::default();
    config.frequency = 400_000;

    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, config);

    // Initialize DA7280
    info!("Setting up DA7280 haptics driver...");
    let mut haptics = DA728x::new(i2c, 0x4A, Variant::DA7280)
        .await
        .unwrap();
    info!("DA7280 initialized successfully.");

    // Configure actuator - these values work well with the SparkFun board's LRA
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 1800,
        absolute_max_mV: 2000,
        max_current_mA: 120,
        impedance_mOhm: 15000,
        frequency_Hz: 170,
    };

    // Configure for DRO mode with frequency tracking
    let device_config = DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,
        rapid_stop: false,
    };

    haptics.configure(actuator_config, device_config).await.unwrap();
    haptics.enable().await.unwrap();
    info!("DRO mode enabled.");

    // Main loop - generate simple pulses
    loop {
        info!("Pulse!");

        // Set amplitude to maximum (127 = 100%)
        haptics.set_override_value(127).await.unwrap();
        Timer::after_millis(100).await;

        // Turn off
        haptics.set_override_value(0).await.unwrap();
        Timer::after_millis(400).await;

        // Check for errors
        // Note: EMBEDDED_MODE is enabled, so faults auto-clear when going to IDLE.
        // If a fault occurs, disable briefly to trigger auto-clear, then re-enable.
        let (events, warnings, _) = haptics.get_events().await.unwrap();
        if events.E_ACTUATOR_FAULT() {
            warn!("ACTUATOR FAULT - Is the actuator loaded? Auto-recovering...");

            // Disable to enter IDLE state (triggers auto-clear via EMBEDDED_MODE)
            haptics.disable().await.unwrap();
            Timer::after_millis(50).await;
            haptics.enable().await.unwrap();

            info!("Recovery complete - pulses will resume when actuator is loaded");
        }
        if events.E_WARNING() {
            warn!("Warning: {:?}", warnings);
        }
    }
}

embassy_rp::bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});
