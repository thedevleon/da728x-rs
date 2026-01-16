//! Wideband DRO example playing the Tetris melody.
//!
//! This example demonstrates using WIDEBAND mode to play different frequencies,
//! creating a simple melody using the haptic actuator as a speaker/buzzer.
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
//! - The melody may not be audible
//! - You will see `ACTUATOR FAULT` warnings in the logs
//! - This is expected behavior - the DA7280 uses back-EMF sensing to detect
//!   abnormal actuator conditions
//!
//! To test properly, place the haptic motor between two solid objects (e.g.,
//! hold it pressed against a table with your finger).
//!
//! # WIDEBAND vs FREQUENCY_TRACK
//!
//! - **FREQUENCY_TRACK**: Locks onto the LRA's resonant frequency (~170 Hz) for
//!   maximum efficiency. Cannot play arbitrary frequencies.
//! - **WIDEBAND**: Allows driving the actuator at any frequency, enabling melodies
//!   but with less efficient energy transfer.
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example wideband_melody
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

/// Tetris Theme (Korobeiniki) melody - frequency in Hz and duration in ms.
const TETRIS_MELODY: &[(u16, u64)] = &[
    (659, 150), (494, 75), (523, 75), (587, 150), (523, 75), (494, 75),
    (440, 150), (440, 75), (523, 75), (659, 150), (587, 75), (523, 75),
    (494, 225), (523, 75), (587, 150), (659, 150),
    (523, 150), (440, 150), (440, 150),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== Wideband Melody Example (Tetris Theme) ===");
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

    // Configure actuator
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 1800,
        absolute_max_mV: 2000,
        max_current_mA: 120,
        impedance_mOhm: 15000,
        frequency_Hz: 170,
    };

    // Configure for DRO mode with WIDEBAND (allows arbitrary frequencies)
    let device_config = DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::WIDEBAND,
        acceleration: false,
        rapid_stop: false,
    };

    haptics.configure(actuator_config, device_config).await.unwrap();
    haptics.enable().await.unwrap();
    info!("Wideband DRO mode enabled.");

    loop {
        info!("Playing Tetris melody...");

        for &(freq, duration) in TETRIS_MELODY.iter() {
            // Set frequency for this note
            haptics.set_frequency(freq).await.unwrap();

            // Play note at full amplitude
            haptics.set_override_value(127).await.unwrap();
            Timer::after_millis(duration).await;

            // Brief silence between notes
            haptics.set_override_value(0).await.unwrap();
            Timer::after_millis(50).await;
        }

        info!("Melody complete!");

        // Check for errors
        let (events, warnings, _) = haptics.get_events().await.unwrap();
        if events.E_ACTUATOR_FAULT() {
            warn!("ACTUATOR FAULT - Is the actuator loaded?");
        }
        if events.E_WARNING() {
            warn!("Warning: {:?}", warnings);
        }

        // Wait before repeating
        Timer::after_millis(2000).await;
    }
}

embassy_rp::bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});
