# `da728x`
An async and no_std rust library for the wide-bandwidth haptic driver IC DA7280/DA7281/DA7282 from Renesas.

## Supported ICs
- DA7280
- DA7281
- DA7282

## What works
- CHIP_REV verification
- Configuration with validation
- Setting resonant frequency
- Enable / disable playback
- Reading and clearing system events and diagnostics
- Driving an LRA in frequency track, wideband or custom waveform mode
- DRO mode, PWM mode, RTWMN mode

## What's missing
- GPI configuration and ETWM_MODE
- Uploading into the waveform memory
- Uploading a script (list of registers and values as exported by GUI)

## Features
- `debug` - Enable debug logging with the `defmt` crate

# Usage
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
        frequency_Hz: 170,
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

    // Enables the Operation Mode (default is INACTIVE after configuration)
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

# Devkits
- [SparkFun Haptic Driver (ROB-17590)](https://www.sparkfun.com/sparkfun-qwiic-haptic-driver-da7280.html)
- [Haptic 4 Click (MIKROE-6045)](https://www.mikroe.com/haptic-4-click)
- [Haptic 3 Click (MIKROE-5087)](https://www.mikroe.com/haptic-3-click)
- [DA728X-EVAL-KIT](https://www.renesas.com/en/design-resources/boards-kits/da728x-eval-kit)