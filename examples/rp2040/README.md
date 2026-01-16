# DA728x RP2040 Examples

Examples demonstrating the DA728x haptic driver library on RP2040 microcontrollers.

## Hardware Setup

### Required Components

- RP2040-based board (e.g., Raspberry Pi Pico, Pico W)
- [SparkFun Qwiic Haptic Driver - DA7280](https://www.sparkfun.com/products/17590)
- Debug probe (e.g., Raspberry Pi Debug Probe, Picoprobe)

### I2C Wiring

Connect the DA7280 to the RP2040 using I2C0:

| DA7280 Pin | RP2040 Pin | Function |
|------------|------------|----------|
| SDA        | GP16       | I2C Data |
| SCL        | GP17       | I2C Clock |
| VCC        | 3V3        | Power (3.3V) |
| GND        | GND        | Ground |

The I2C address is **0x4A** (default for DA7280).

If using the SparkFun Qwiic connector, you can use a Qwiic-to-breadboard cable
or adapter.

## Important: Actuator Loading

The LRA (Linear Resonant Actuator) on the SparkFun board **must be mechanically
loaded** for proper operation. This means the motor should be pressed between
two surfaces.

### Why Loading Matters

The DA7280 uses back-EMF (electromotive force) sensing to:
- Track the actuator's resonant frequency
- Detect actuator faults
- Optimize power delivery

When the actuator is **unloaded** (free-floating):
- The mechanical behavior differs from normal operation
- The DA7280 detects this as an abnormal condition
- You will see `ACTUATOR FAULT` warnings in the logs

### How to Load the Actuator

Simply hold the motor pressed against a table with your finger, or sandwich it
between two solid objects. The haptic feedback will be much stronger and the
fault warnings will stop.

## Building and Running

### Prerequisites

1. Install Rust and the thumbv6m target:
   ```bash
   rustup target add thumbv6m-none-eabi
   ```

2. Install probe-rs:
   ```bash
   cargo install probe-rs-tools
   ```

### Running Examples

Connect your debug probe and run any example with:

```bash
# Simple DRO (basic pulses)
cargo run --release --example simple_dro

# Wideband melody (Tetris theme)
cargo run --release --example wideband_melody

# Simple waveform (single click effect)
cargo run --release --example simple_waveform

# Waveform effects (click, double-click, buzz)
cargo run --release --example waveform_effects
```

## Example Descriptions

### `simple_dro`

Basic DRO (Direct Register Override) mode example. Generates simple haptic
pulses by directly controlling the output amplitude. Uses FREQUENCY_TRACK mode
for optimal efficiency at the LRA's resonant frequency.

### `wideband_melody`

Plays the Tetris theme using WIDEBAND mode, which allows driving the actuator
at arbitrary frequencies. Demonstrates how to use `set_frequency()` to create
musical tones.

### `simple_waveform`

Minimal RTWM (Register-Triggered Waveform Memory) example. Shows how to:
- Create a single snippet (waveform shape)
- Build a sequence that plays the snippet
- Upload waveform memory to the DA7280
- Trigger playback

### `waveform_effects`

Comprehensive waveform example with multiple haptic effects:
- **Click**: Sharp, quick feedback
- **Double-click**: Two clicks with a pause between
- **Buzz**: Sustained vibration using frame looping

Demonstrates advanced features like the built-in silence snippet and frame
loops.

## Troubleshooting

### "ACTUATOR FAULT" warnings

This is normal when the actuator is unloaded. Press the motor against a surface
to load it mechanically. You may need to power cycle the board to clear the warning before it will operate again.

### No haptic output

1. Check I2C wiring (SDA/SCL not swapped)
2. Verify the I2C address (0x4A)
3. Make sure the actuator is loaded
4. Try power-cycling the board

### "Debug probe not found"

1. Check USB connection to debug probe
2. Verify probe-rs is installed correctly
3. Try unplugging and reconnecting the probe

## License

MIT License - see the main repository for details.
