//! RP2040 example demonstrating waveform memory playback with DA7280.
//!
//! This example shows how to:
//! 1. Build waveform snippets and sequences
//! 2. Upload waveform memory to the DA7280
//! 3. Play sequences using RTWM (Register-Triggered Waveform Memory) mode
//!
//! Hardware setup:
//! - DA7280 connected via I2C (SDA: GP4, SCL: GP5)
//! - I2C address: 0x4A (default)

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

    info!("Initializing I2C...");
    let sda = p.PIN_4;
    let scl = p.PIN_5;
    let mut config = Config::default();
    config.frequency = 400_000; // 400kHz

    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, config);

    info!("Setting up DA7280 haptics driver...");
    let mut haptics = DA728x::new(i2c, 0x4A, Variant::DA7280)
        .await
        .unwrap();
    info!("DA7280 initialized successfully.");

    // Build waveform memory
    info!("Building waveform memory...");
    let memory = build_waveform_memory();
    info!(
        "Waveform memory built: {} bytes, {} snippets, {} sequences",
        memory.len(),
        memory.num_snippets(),
        memory.num_sequences()
    );

    // Configure for RTWM mode with frequency tracking
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 2106,
        absolute_max_mV: 2260,
        max_current_mA: 165,
        impedance_mOhm: 13800,
        frequency_Hz: 170,
    };

    let device_config = DeviceConfig {
        operation_mode: OperationMode::RTWM_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: true,
        rapid_stop: true,
    };

    haptics
        .configure(actuator_config, device_config)
        .await
        .unwrap();
    info!("DA7280 configured for RTWM mode.");

    // Upload waveform memory
    info!("Uploading waveform memory...");
    haptics.upload_waveform_memory(&memory, true).await.unwrap();
    info!("Waveform memory uploaded and locked.");

    // Enable the driver
    haptics.enable().await.unwrap();
    info!("DA7280 enabled. Starting playback demo...");

    loop {
        // Play sequence 0: Click effect
        info!("Playing click effect (sequence 0)...");
        haptics.play_sequence(0, 0).await.unwrap();
        Timer::after_millis(500).await;

        // Play sequence 1: Double click
        info!("Playing double click (sequence 1)...");
        haptics.play_sequence(1, 0).await.unwrap();
        Timer::after_millis(800).await;

        // Play sequence 2: Buzz pattern (looped 3 times)
        info!("Playing buzz pattern (sequence 2, 3 loops)...");
        haptics.play_sequence(2, 3).await.unwrap();
        Timer::after_millis(2000).await;

        // Check status
        let status = haptics.get_status().await.unwrap();
        info!("Status: {:?}", status);

        // Check for any errors
        let (events, warnings, seq_diag) = haptics.get_events().await.unwrap();
        if events.E_SEQ_FAULT() {
            warn!("Sequence fault detected!");
            info!("Seq diag: {:?}", seq_diag);
        }
        if events.E_WARNING() {
            warn!("Warning detected: {:?}", warnings);
        }

        Timer::after_millis(2000).await;
    }
}

/// Build the waveform memory with example haptic patterns.
///
/// Creates:
/// - Snippet 1: Sharp click (quick rise/fall)
/// - Snippet 2: Soft bump (gradual rise/fall)
/// - Snippet 3: Short buzz (sustained vibration)
///
/// - Sequence 0: Single click
/// - Sequence 1: Double click
/// - Sequence 2: Buzz pattern
fn build_waveform_memory() -> da728x::waveform::WaveformMemory {
    // Snippet 1: Sharp click - quick rise to max, quick fall
    let click_snippet = SnippetBuilder::new()
        .ramp(1, 15) // Ramp to 100% in 1 timebase
        .unwrap()
        .ramp(1, 0) // Ramp to 0% in 1 timebase
        .unwrap()
        .build()
        .unwrap();

    // Snippet 2: Soft bump - gradual rise and fall
    let bump_snippet = SnippetBuilder::new()
        .ramp(2, 10) // Ramp to ~67% in 2 timebases
        .unwrap()
        .step(1, 10) // Hold for 1 timebase
        .unwrap()
        .ramp(2, 0) // Ramp down in 2 timebases
        .unwrap()
        .build()
        .unwrap();

    // Snippet 3: Short buzz - quick on, sustain, quick off
    let buzz_snippet = SnippetBuilder::new()
        .ramp(1, 12) // Quick rise to ~80%
        .unwrap()
        .step(4, 12) // Hold for 4 timebases
        .unwrap()
        .ramp(1, 0) // Quick fall
        .unwrap()
        .build()
        .unwrap();

    // Sequence 0: Single click using snippet 1
    let click_frame = FrameBuilder::new(1) // Snippet ID 1
        .unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms5_44)
        .build()
        .unwrap();

    let click_sequence = SequenceBuilder::new()
        .add_frame(click_frame)
        .unwrap()
        .build()
        .unwrap();

    // Sequence 1: Double click using snippet 1 twice
    let click_frame1 = FrameBuilder::new(1)
        .unwrap()
        .timebase(Timebase::Ms5_44)
        .build()
        .unwrap();

    let click_frame2 = FrameBuilder::new(1)
        .unwrap()
        .timebase(Timebase::Ms5_44)
        .build()
        .unwrap();

    let double_click_sequence = SequenceBuilder::new()
        .add_frame(click_frame1)
        .unwrap()
        .add_frame(click_frame2)
        .unwrap()
        .build()
        .unwrap();

    // Sequence 2: Buzz pattern using snippet 3 with loop
    let buzz_frame = FrameBuilder::new(3) // Snippet ID 3
        .unwrap()
        .gain(Gain::ThreeQuarter)
        .timebase(Timebase::Ms10_88)
        .loop_count(2) // Loop 2 extra times (3 total plays)
        .unwrap()
        .build()
        .unwrap();

    let buzz_sequence = SequenceBuilder::new()
        .add_frame(buzz_frame)
        .unwrap()
        .build()
        .unwrap();

    // Build the complete waveform memory
    WaveformMemoryBuilder::new(true) // acceleration enabled
        .add_snippet(click_snippet) // ID 1
        .unwrap()
        .add_snippet(bump_snippet) // ID 2
        .unwrap()
        .add_snippet(buzz_snippet) // ID 3
        .unwrap()
        .add_sequence(click_sequence) // ID 0
        .unwrap()
        .add_sequence(double_click_sequence) // ID 1
        .unwrap()
        .add_sequence(buzz_sequence) // ID 2
        .unwrap()
        .build()
        .unwrap()
}

// I2C interrupt binding for RP2040
embassy_rp::bind_interrupts!(struct Irqs {
    I2C0_IRQ => i2c::InterruptHandler<I2C0>;
});
