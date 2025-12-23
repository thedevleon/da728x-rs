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
- Driving an LRA in frequency track, wideband or custom waveform mode
- DRO mode, PWM mode, RTWMN mode

## What's missing
- GPI configuration and ETWM_MODE
- Uploading into the waveform memory
- Uploading a script (list of registers and values as exported by GUI)

## Features
- `debug` - Enable debug logging with the `defmt` crate

# Usage
// TODO

# Devkits
- [SparkFun Haptic Driver (ROB-17590)](https://www.sparkfun.com/sparkfun-qwiic-haptic-driver-da7280.html)
- [Haptic 4 Click (MIKROE-6045)](https://www.mikroe.com/haptic-4-click)
- [Haptic 3 Click (MIKROE-5087)](https://www.mikroe.com/haptic-3-click)
- [DA728X-EVAL-KIT](https://www.renesas.com/en/design-resources/boards-kits/da728x-eval-kit)