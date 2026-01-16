//! Simple RTWM (Register-Triggered Waveform Memory) example.
//!
//! This example demonstrates the basics of waveform memory:
//! - Creating a simple snippet (waveform shape)
//! - Creating a sequence that plays the snippet
//! - Uploading to the DA7280's waveform memory
//! - Triggering playback via register writes
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
//! - Waveforms may not play correctly
//! - You will see `ACTUATOR FAULT` warnings in the logs
//! - This is expected behavior - the DA7280 uses back-EMF sensing to detect
//!   abnormal actuator conditions
//!
//! To test properly, place the haptic motor between two solid objects (e.g.,
//! hold it pressed against a table with your finger).
//!
//! # Fault Recovery
//!
//! The driver enables `EMBEDDED_MODE` which allows automatic fault clearing
//! when the device enters IDLE state. If a fault occurs (e.g., from an unloaded
//! actuator), the example recovers by briefly disabling and re-enabling the
//! device, which triggers the auto-clear mechanism.
//!
//! # Waveform Memory Concepts
//!
//! - **Snippet**: A piecewise-linear (PWL) waveform shape defined by amplitude
//!   points over time. Snippet ID 0 is reserved for built-in silence.
//! - **Frame**: References a snippet with gain, timebase, and optional looping.
//! - **Sequence**: A series of frames played in order.
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example simple_waveform
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
use da728x::waveform::{
    FrameBuilder, Gain, SequenceBuilder, SnippetBuilder, Timebase, WaveformMemoryBuilder,
};
use da728x::{Variant, DA728x};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    info!("=== Simple Waveform (RTWM) Example ===");
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

    // Build a simple click waveform
    info!("Building waveform memory...");

    // Snippet 1: Simple click - quick rise and fall
    let click_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()  // Rise to 100% in 1 timebase
        .ramp(2, 0).unwrap()   // Fall to 0% in 2 timebases
        .build()
        .unwrap();

    // Sequence 0: Play the click snippet once
    let click_frame = FrameBuilder::new(1).unwrap()  // Snippet ID 1
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)  // ~22ms per timebase unit
        .build()
        .unwrap();

    let click_sequence = SequenceBuilder::new()
        .add_frame(click_frame).unwrap()
        .build()
        .unwrap();

    // Build memory with one snippet and one sequence
    let memory = WaveformMemoryBuilder::new(false)  // acceleration disabled
        .add_snippet(click_snippet).unwrap()
        .add_sequence(click_sequence).unwrap()
        .build()
        .unwrap();

    info!(
        "Memory built: {} bytes, {} snippet(s), {} sequence(s)",
        memory.len(),
        memory.num_snippets(),
        memory.num_sequences()
    );

    // Configure for RTWM mode
    let device_config = DeviceConfig {
        operation_mode: OperationMode::RTWM_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,
        rapid_stop: false,
    };

    haptics.configure(actuator_config, device_config).await.unwrap();
    info!("Configured for RTWM mode.");

    // Upload and verify waveform memory
    info!("Uploading waveform memory...");
    haptics.upload_waveform_memory(&memory, false).await.unwrap();

    // Verify upload
    let mut readback = [0u8; 16];
    haptics.read_waveform_memory(memory.len(), &mut readback).await.unwrap();
    let expected = memory.as_bytes();
    let verified = readback[..memory.len()] == expected[..memory.len()];
    if verified {
        info!("Memory verification: PASSED");
    } else {
        error!("Memory verification: FAILED");
    }

    // Lock memory and enable
    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();
    info!("RTWM enabled. Playing clicks...");

    loop {
        info!("Click!");

        // Play sequence 0 with no extra loops
        haptics.play_sequence(0, 0).await.unwrap();

        Timer::after_millis(500).await;

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

            info!("Recovery complete - playback will resume when actuator is loaded");
        }
        if events.E_WARNING() {
            warn!("Warning: {:?}", warnings);
        }
    }
}

embassy_rp::bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});
