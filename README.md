# `da728x`
An async and no_std rust library for the wide-bandwidth haptic driver IC DA7280/DA7281/DA7282 from Renesas.

## Supported ICs
- DA7280
- DA7281
- DA7282

## What works
- Device initialization and configuration
- Operation mode control (DRO, PWM, RTWM, ETWM)
- IRQ event handling
- Waveform memory management
- GPI configuration
- Activation/deactivation control

## Features
- `debug` - Enable debug logging with the `log` crate

# Usage

## Basic Example

```rust,ignore
use da728x::{DA728x, Variant, DeviceConfig, DeviceType, OperationMode};

async fn example() -> Result<(), da728x::Error> {
    // Assuming you have i2c, int_pin, and delay providers set up
    let config = DeviceConfig::new()
        .with_device_type(DeviceType::LRA)
        .with_resonant_freq_hz(235)
        .with_bemf_sense(true)
        .with_freq_track(true)
        .with_acceleration(true);

    // Initialize the driver (requires I2C, interrupt pin, and delay provider)
    let mut driver = DA728x::new(
        i2c,           // I2C bus
        0x4A,          // I2C address
        int_pin,       // Interrupt pin
        delay,         // Delay provider
        Variant::DA7280,
        config,
    ).await?;

    // Activate in DRO (Direct Register Override) mode
    driver.activate(OperationMode::DRO).await?;

    // Set the drive level (0-255)
    driver.set_dro_level(128).await?;

    // Deactivate
    driver.deactivate().await?;
    
    Ok(())
}
```

## Waveform Memory Example

```rust,ignore
use da728x::{DA728x, Variant, DeviceConfig, DeviceType, OperationMode, SNP_MEM_SIZE};

async fn example() -> Result<(), da728x::Error> {
    // Create configuration with waveform data
    let mut waveform = [0u8; SNP_MEM_SIZE];
    // Fill waveform with pattern data...

    let config = DeviceConfig::new()
        .with_device_type(DeviceType::LRA)
        .with_mem_data(waveform);

    let mut driver = DA728x::new(i2c, 0x4A, int_pin, delay, Variant::DA7280, config).await?;

    // Set sequence parameters
    driver.set_ps_sequence(1, 3).await?;  // Sequence ID 1, loop 3 times

    // Activate in RTWM (Real-Time Waveform Memory) mode
    driver.activate(OperationMode::RTWM).await?;
    
    Ok(())
}
```

## Interrupt Handling Example

```rust,ignore
use da728x::DA728x;

async fn check_interrupts(
    driver: &mut DA728x<
        impl embedded_hal_async::i2c::I2c,
        impl embedded_hal_async::digital::Wait,
        impl embedded_hal_async::delay::DelayNs
    >
) -> Result<(), da728x::Error> {
    // Read IRQ events
    let (event1, warning, seq_diag) = driver.read_irq_events().await?;

    if event1.E_SEQ_DONE() {
        // Sequence completed
    }

    if event1.E_WARNING() {
        if warning.E_LIM_DRIVE() {
            // Drive limit reached
        }
    }

    // Clear interrupt events
    driver.clear_irq_events().await?;
    
    Ok(())
}
```

## Configuration Builder

The `DeviceConfig` struct provides a builder pattern for configuring the device:

```rust,no_run
use da728x::{DeviceConfig, DeviceType};

let config = DeviceConfig::new()
    .with_device_type(DeviceType::LRA)
    .with_nom_microvolt(2_000_000)      // 2V nominal
    .with_abs_max_microvolt(2_800_000)  // 2.8V absolute max
    .with_imax_microamp(85_000)          // 85mA max current
    .with_impd_micro_ohms(8_000_000)     // 8Ω impedance
    .with_resonant_freq_hz(235)          // 235Hz resonant frequency
    .with_bemf_sense(true)               // Enable back-EMF sensing
    .with_freq_track(true)               // Enable frequency tracking
    .with_acceleration(true)             // Enable acceleration
    .with_rapid_stop(true)               // Enable rapid stop
    .with_amp_pid(true)                  // Enable amplitude PID
    .with_ps_seq_id(0)                   // Pre-stored sequence ID
    .with_ps_seq_loop(0);                // Pre-stored sequence loop count
```

# Devkits
- [SparkFun Haptic Driver (ROB-17590)](https://www.sparkfun.com/sparkfun-qwiic-haptic-driver-da7280.html)
- [Haptic 4 Click (MIKROE-6045)](https://www.mikroe.com/haptic-4-click)
- [Haptic 3 Click (MIKROE-5087)](https://www.mikroe.com/haptic-3-click)
- [DA728X-EVAL-KIT](https://www.renesas.com/en/design-resources/boards-kits/da728x-eval-kit)