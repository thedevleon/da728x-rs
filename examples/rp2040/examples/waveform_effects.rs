//! Complex waveform effects example demonstrating multiple haptic patterns.
//!
//! This example shows how to create and play multiple haptic effects:
//! - Click: Sharp, quick feedback
//! - Double-click: Two clicks with a pause
//! - Buzz: Sustained vibration with looping
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
//! - Effects may feel weak or not play at all
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
//! # Waveform Memory Layout
//!
//! This example creates:
//! - Snippet 1: Click shape (quick rise, smooth fall)
//! - Snippet 2: Bump shape (gradual rise, hold, fall)
//! - Snippet 3: Buzz shape (rise, sustain, fall)
//! - Sequence 0: Single click
//! - Sequence 1: Double click (two clicks with silence between)
//! - Sequence 2: Buzz (sustained vibration with loop)
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example waveform_effects
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

    info!("=== Waveform Effects Example ===");
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

    // Build waveform memory with multiple effects
    info!("Building waveform memory...");
    let memory = build_waveform_memory();
    info!(
        "Memory built: {} bytes, {} snippets, {} sequences",
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
    let mut readback = [0u8; 32];
    haptics.read_waveform_memory(memory.len(), &mut readback).await.unwrap();
    let expected = memory.as_bytes();
    let mut verified = true;
    for i in 0..memory.len() {
        if readback[i] != expected[i] {
            error!("Mismatch at byte {}: got {:02X}, expected {:02X}", i, readback[i], expected[i]);
            verified = false;
        }
    }
    if verified {
        info!("Memory verification: PASSED");
    } else {
        error!("Memory verification: FAILED");
    }

    // Lock memory and enable
    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();

    // Clear any stale events
    let _ = haptics.get_events().await;

    info!("RTWM enabled. Playing effects...");

    loop {
        // Effect 1: Single click
        info!("Effect: Click");
        haptics.play_sequence(0, 0).await.unwrap();
        Timer::after_millis(600).await;

        // Effect 2: Double click
        info!("Effect: Double click");
        haptics.play_sequence(1, 0).await.unwrap();
        Timer::after_millis(800).await;

        // Effect 3: Buzz
        info!("Effect: Buzz");
        haptics.play_sequence(2, 0).await.unwrap();
        Timer::after_millis(1500).await;

        // Check for errors and recover if needed
        // EMBEDDED_MODE is enabled, so faults auto-clear when going to IDLE.
        let (events, warnings, seq_diag) = haptics.get_events().await.unwrap();
        let mut has_error = false;
        let mut needs_recovery = false;

        if events.E_OC_FAULT() {
            warn!("OVERCURRENT FAULT!");
            has_error = true;
            needs_recovery = true;
        }
        if events.E_ACTUATOR_FAULT() {
            warn!("ACTUATOR FAULT - Is the actuator loaded? Auto-recovering...");
            has_error = true;
            needs_recovery = true;
        }
        if events.E_WARNING() {
            warn!("Warning: {:?}", warnings);
            has_error = true;
        }
        if events.E_SEQ_FAULT() {
            warn!("Sequence fault: {:?}", seq_diag);
            has_error = true;
            needs_recovery = true;
        }

        if needs_recovery {
            // Disable to enter IDLE state (triggers auto-clear via EMBEDDED_MODE)
            haptics.disable().await.unwrap();
            Timer::after_millis(50).await;
            haptics.enable().await.unwrap();
            info!("Recovery complete - effects will resume when actuator is loaded");
        } else if !has_error {
            info!("All effects OK");
        }

        Timer::after_millis(1000).await;
    }
}

/// Build waveform memory with click, bump, and buzz effects.
fn build_waveform_memory() -> da728x::waveform::WaveformMemory {
    // Snippet 1: Click - quick rise, smooth fall
    let click_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()  // Fast rise to 100%
        .ramp(2, 0).unwrap()   // Smooth fall to 0%
        .build()
        .unwrap();

    // Snippet 2: Bump - gradual rise, hold, gradual fall
    let bump_snippet = SnippetBuilder::new()
        .ramp(2, 15).unwrap()  // Rise to 100%
        .step(2, 15).unwrap()  // Hold for 2 timebases
        .ramp(2, 0).unwrap()   // Fall to 0%
        .build()
        .unwrap();

    // Snippet 3: Buzz - quick rise, sustain, quick fall
    let buzz_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()  // Quick rise to 100%
        .step(6, 15).unwrap()  // Sustain for 6 timebases
        .ramp(1, 0).unwrap()   // Quick fall
        .build()
        .unwrap();

    // Sequence 0: Single click
    let click_frame = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    let click_seq = SequenceBuilder::new()
        .add_frame(click_frame).unwrap()
        .build()
        .unwrap();

    // Sequence 1: Double click (click + silence + click)
    let frame1 = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    // Use built-in silence snippet (ID 0) for pause between clicks
    let silence = FrameBuilder::silence()
        .timebase(Timebase::Ms43_52)  // ~87ms pause
        .build()
        .unwrap();
    let frame2 = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    let double_click_seq = SequenceBuilder::new()
        .add_frame(frame1).unwrap()
        .add_frame(silence).unwrap()
        .add_frame(frame2).unwrap()
        .build()
        .unwrap();

    // Sequence 2: Buzz with loop for sustained vibration
    let buzz_frame = FrameBuilder::new(3).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)
        .loop_count(3).unwrap()  // Play 4 times total
        .build()
        .unwrap();
    let buzz_seq = SequenceBuilder::new()
        .add_frame(buzz_frame).unwrap()
        .build()
        .unwrap();

    // Build the complete waveform memory
    WaveformMemoryBuilder::new(false)  // acceleration disabled
        .add_snippet(click_snippet).unwrap()
        .add_snippet(bump_snippet).unwrap()
        .add_snippet(buzz_snippet).unwrap()
        .add_sequence(click_seq).unwrap()
        .add_sequence(double_click_seq).unwrap()
        .add_sequence(buzz_seq).unwrap()
        .build()
        .unwrap()
}

embassy_rp::bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});
