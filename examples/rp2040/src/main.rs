//! RP2040 example demonstrating haptic feedback with DA7280.
//!
//! This example tests both DRO and RTWM modes.
//!
//! Hardware setup:
//! - DA7280 connected via I2C0 (SDA: GP16, SCL: GP17)
//! - I2C address: 0x4A (default)
//! - LRA actuator should be compressed between two surfaces!

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

    info!("=== DA7280 RTWM Mode Test ===");

    info!("Initializing I2C...");
    let sda = p.PIN_16;
    let scl = p.PIN_17;
    let mut config = Config::default();
    config.frequency = 400_000;

    let i2c = I2c::new_async(p.I2C0, scl, sda, Irqs, config);

    info!("Setting up DA7280 haptics driver...");
    let mut haptics = DA728x::new(i2c, 0x4A, Variant::DA7280)
        .await
        .unwrap();
    info!("DA7280 initialized successfully.");

    // G1040003D LRA specs - reduced drive to avoid overcurrent
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 1800,      // Reduced from 2500 to lower current
        absolute_max_mV: 2000,     // Reduced headroom
        max_current_mA: 120,       // Reduced from 170 mA
        impedance_mOhm: 15000,     // ~15Ω
        frequency_Hz: 170,         // 170 Hz resonant
    };

    // First do a quick DRO test to verify hardware
    // Disable acceleration to reduce current draw
    info!("=== Quick DRO test ===");
    let dro_config = DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,  // Disabled to reduce current
        rapid_stop: false,    // Disabled to reduce current
    };

    haptics.configure(actuator_config, dro_config).await.unwrap();
    haptics.enable().await.unwrap();

    for i in 0..3 {
        info!("DRO pulse {}/3", i + 1);
        haptics.set_override_value(127).await.unwrap();
        Timer::after_millis(100).await;
        haptics.set_override_value(0).await.unwrap();
        Timer::after_millis(200).await;
    }

    info!("DRO test complete");
    haptics.disable().await.unwrap();
    Timer::after_millis(100).await;

    // Now test RTWM mode
    info!("=== RTWM Mode Test ===");

    // Build simple waveform memory
    info!("Building waveform memory...");
    let memory = build_simple_waveform_memory();
    info!(
        "Memory: {} bytes, {} snippets, {} sequences",
        memory.len(),
        memory.num_snippets(),
        memory.num_sequences()
    );

    // Configure for RTWM mode - no acceleration to reduce current
    let rtwm_config = DeviceConfig {
        operation_mode: OperationMode::RTWM_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,
        rapid_stop: false,
    };

    haptics.configure(actuator_config, rtwm_config).await.unwrap();
    info!("Configured for RTWM mode.");

    // Upload waveform memory
    info!("Uploading waveform memory...");
    haptics.upload_waveform_memory(&memory, false).await.unwrap();

    // Verify memory
    let mut readback = [0u8; 32];
    let _ = haptics.read_waveform_memory(memory.len(), &mut readback).await.unwrap();
    let expected = memory.as_bytes();
    let mut ok = true;
    for i in 0..memory.len() {
        if readback[i] != expected[i] {
            warn!("Mismatch at {}: {:02X} vs {:02X}", i, expected[i], readback[i]);
            ok = false;
        }
    }
    if ok {
        info!("Memory verification: PASSED");
    }

    // Lock memory and enable
    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();

    // Clear any stale events
    let _ = haptics.get_events().await;

    info!("RTWM enabled. Playing sequences...");

    loop {
        // Sequence 0: Single click
        info!("Sequence 0: Click");
        haptics.play_sequence(0, 0).await.unwrap();
        Timer::after_millis(600).await;

        // Sequence 1: Double click
        info!("Sequence 1: Double click");
        haptics.play_sequence(1, 0).await.unwrap();
        Timer::after_millis(800).await;

        // Sequence 2: Buzz (looped)
        info!("Sequence 2: Buzz");
        haptics.play_sequence(2, 0).await.unwrap();
        Timer::after_millis(1500).await;

        // Check for all error types
        let (events, warnings, seq_diag) = haptics.get_events().await.unwrap();
        let mut has_error = false;

        if events.E_OC_FAULT() {
            warn!("OVERCURRENT FAULT!");
            has_error = true;
        }
        if events.E_ACTUATOR_FAULT() {
            warn!("ACTUATOR FAULT!");
            has_error = true;
        }
        if events.E_WARNING() {
            warn!("WARNING: {:?}", warnings);
            has_error = true;
        }
        if events.E_SEQ_FAULT() {
            warn!("SEQ_FAULT: {:?}", seq_diag);
            has_error = true;
        }
        if events.E_OVERTEMP_CRIT() {
            warn!("OVERTEMP!");
            has_error = true;
        }
        if events.E_UVLO() {
            warn!("UNDERVOLTAGE!");
            has_error = true;
        }

        if !has_error {
            info!("All OK");
        }

        Timer::after_millis(1000).await;
    }
}

/// Build waveform memory with multiple effects.
fn build_simple_waveform_memory() -> da728x::waveform::WaveformMemory {
    // Snippet 1: Quick rise, smooth fall
    let click_snippet = SnippetBuilder::new()
        .ramp(1, 15).unwrap()  // Fast rise (1 timebase)
        .ramp(2, 0).unwrap()   // Smooth fall (2 timebases)
        .build()
        .unwrap();

    // Snippet 2: Soft bump
    let bump_snippet = SnippetBuilder::new()
        .ramp(2, 10).unwrap()
        .step(1, 10).unwrap()
        .ramp(2, 0).unwrap()
        .build()
        .unwrap();

    // Snippet 3: Buzz
    let buzz_snippet = SnippetBuilder::new()
        .ramp(1, 12).unwrap()
        .step(4, 12).unwrap()
        .ramp(1, 0).unwrap()
        .build()
        .unwrap();

    // Sequence 0: Single click (using slower timebase for smoother feel)
    let click_frame = FrameBuilder::new(1).unwrap()
        .gain(Gain::Full)
        .timebase(Timebase::Ms21_76)  // Slower timebase for smoother transitions
        .build()
        .unwrap();
    let click_seq = SequenceBuilder::new()
        .add_frame(click_frame).unwrap()
        .build()
        .unwrap();

    // Sequence 1: Double click with pause between
    let frame1 = FrameBuilder::new(1).unwrap()
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    // Snippet 0 is built-in silence (2 timebases). Use slower timebase for longer pause.
    let silence = FrameBuilder::silence()
        .timebase(Timebase::Ms43_52)  // ~87ms pause
        .build()
        .unwrap();
    let frame2 = FrameBuilder::new(1).unwrap()
        .timebase(Timebase::Ms21_76)
        .build()
        .unwrap();
    let double_click_seq = SequenceBuilder::new()
        .add_frame(frame1).unwrap()
        .add_frame(silence).unwrap()
        .add_frame(frame2).unwrap()
        .build()
        .unwrap();

    // Sequence 2: Buzz with loop
    let buzz_frame = FrameBuilder::new(3).unwrap()
        .gain(Gain::Half)
        .timebase(Timebase::Ms21_76)
        .loop_count(2).unwrap()
        .build()
        .unwrap();
    let buzz_seq = SequenceBuilder::new()
        .add_frame(buzz_frame).unwrap()
        .build()
        .unwrap();

    WaveformMemoryBuilder::new(false)
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
