# `da728x`

[![Crate](https://img.shields.io/crates/v/da728x.svg)](https://crates.io/crates/da728x)
[![API](https://docs.rs/da728x/badge.svg)](https://docs.rs/da728x)


An async (and optionally blocking) `no_std` Rust library for the wide-bandwidth haptic driver IC DA7280/DA7281/DA7282 from Renesas.

## Supported ICs
- DA7280
- DA7281
- DA7282

## What works
- CHIP_REV verification
- Configuration with validation
- Setting (resonant) frequency
- Enable / disable playback
- Reading and clearing system events and diagnostics
- Driving an LRA in frequency track, wideband or custom waveform mode
- DRO mode
- Uploading into the waveform memory and RTWM_MODE

## What's missing
- Testing of PWM_MODE
- GPI configuration and ETWM_MODE
- Uploading a script (list of registers and values as exported by GUI)

## Features
- `debug` - Enable debug logging with the `defmt` crate
- `blocking` - Build a synchronous/blocking API instead of the default async API. Make sure default features are disabled.


# Basic Usage
For simple patterns and basic use, we can directly write to a register in DRO mode to change the intensity of the LRA.

```rust
    use da728x::{DA728x, Variant};
    use da728x::config::{ActuatorConfig, ActuatorType, DeviceConfig, OperationMode, DrivingMode};

    // Setup I2C
    // let i2c = ...
    // let address = ...

    let mut haptics = DA728x::new(i2c, address, Variant::DA7280)
        .await
        .unwrap();

    // Values for the G1040003D LRA on the SparkFun board
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 2_106,
        absolute_max_mV: 2_260,
        max_current_mA: 165,
        impedance_mOhm: 13_800,
        inductance_uH: 50,
        frequency_Hz: 170,
        pid_Kp_Ki: None, // Optional
    };

    // DRO Mode, which means we can set the amplitude via set_override_value()
    let device_config = DeviceConfig {
        operation_mode: OperationMode::DRO_MODE,
        driving_mode: DrivingMode::FREQUENCY_TRACK,
        acceleration: false,
        rapid_stop: false,
    };

    // Sets all registers as needed depending on the actuator type, operation mode and driving mode
    haptics.configure(actuator_config, device_config).await.unwrap();

    // Enables the configured Operation Mode (default is INACTIVE after configuration)
    haptics.enable().await.unwrap();

    loop {
        info!("100%");
        haptics.set_override_value(127).await.unwrap();
        Timer::after_millis(800).await;
        info!("33%");
        haptics.set_override_value(42).await.unwrap();
        Timer::after_millis(800).await;
        info!("0%");
        haptics.set_override_value(0).await.unwrap();
        Timer::after_millis(800).await;

        let status = haptics.get_status().await.unwrap();
        info!("Haptics Status: {:?}", status);
    }

```

# Patterns
Two build more advanced haptic patterns that will be programmed into the waveform memory of the IC, you have two options: 
- Waveform Builder (exports waveform memory)
- Builder Pattern (building patterns programmatically)

## Waveform Builder
![a screenshot of the builder UI](media/builder1.png "Waveform Builder")
![a screenshot of the builder UI](media/builder.png "Waveform Builder")

Included is a standalone waveform builder in the form of a web application to interactively build the snippets and sequences to build haptic patterns.
Just open the [the waveform builder](tools/waveform_builder.html) file in your web-browser. Disclaimer: This was built with an LLM. If you don't want to use the tool or you want to build the waveform memory programmatically, you can use the builder pattern.

This tool generates a rust snippet which you can paste in your code and load into waveform memory.
```rust
// Auto-generated DA7280 waveform memory
pub const DA7280_WAVEFORM_MEMORY: WaveformMemory = WaveformMemory::from_bytes(
    [
        0x07, 0x0B, 0x14, 0x15, 0x16, 0x1A, 0x1E, 0x1F, 0x20, 0x23, 0x25, 0x27, 0x2D, 0x38, 0x39,
        0x40, 0x43, 0x47, 0x48, 0x49, 0x77, 0x29, 0x47, 0xF7, 0xF0, 0xF9, 0xF0, 0x77, 0x77, 0x70,
        0x70, 0xF7, 0xF0, 0x01, 0x88, 0x18, 0x01, 0x18, 0x03, 0x18, 0x03, 0x10, 0xB8, 0x31, 0x88,
        0x18, 0x01, 0x88, 0x10, 0x90, 0x01, 0x88, 0x10, 0x90, 0x29, 0xA8, 0x18, 0x14, 0x05, 0xA8,
        0x18, 0x0D, 0x88, 0x05, 0xA8, 0x01, 0x02, 0x18, 0x1E, 0x11, 0x1F, 0x00, 0x19, 0x39, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ],  //
    74, // number of bytes in memory 
    7,  // number of snippets
    11, // number of sequences
);

// main
// let mut haptics = DA728x::new(...);                   hy
// haptics.configure(...);

// set timebase for waveform memory
haptics
    .set_timebase(WaveformMemoryTimebase::TIMEBASE_136_544_2176_4352)
    .await?;

// upload waveform memory and lock after writing
haptics.upload_waveform_memory(&DA7280_WAVEFORM_MEMORY, true).await?;

haptics.enable().await?; // enable

haptics.play_sequence(n, 0).await?; // play any sequence from memory
```

## Builder Pattern
```rust
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
let memory = WaveformMemoryBuilder::new(false)  // acceleration disabled
    .add_snippet(click_snippet).unwrap()
    .add_snippet(bump_snippet).unwrap()
    .add_snippet(buzz_snippet).unwrap()
    .add_sequence(click_seq).unwrap()
    .add_sequence(double_click_seq).unwrap()
    .add_sequence(buzz_seq).unwrap()
    .build()
    .unwrap();

// set timebase for waveform memory
haptics
    .set_timebase(WaveformMemoryTimebase::TIMEBASE_136_544_2176_4352)
    .await?;

// upload waveform memory and lock after writing
haptics.upload_waveform_memory(&DA7280_WAVEFORM_MEMORY, true).await?;

haptics.enable().await?; // enable

haptics.play_sequence(n, 0).await?; // play any sequence from memory
```


# Devkits
- [SparkFun Haptic Driver (ROB-17590)](https://www.sparkfun.com/sparkfun-qwiic-haptic-driver-da7280.html)
- [Haptic 4 Click (MIKROE-6045)](https://www.mikroe.com/haptic-4-click)
- [Haptic 3 Click (MIKROE-5087)](https://www.mikroe.com/haptic-3-click)
- [DA728X-EVAL-KIT](https://www.renesas.com/en/design-resources/boards-kits/da728x-eval-kit)