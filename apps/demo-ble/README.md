# DA728x BLE Haptics Demo

An nRF54L15 application that exposes a custom BLE GATT service for uploading
waveform memory and controlling haptic playback on the DA7280/DA7281/DA7282
driver ICs.

## Features

- **Custom Haptics GATT Service**
  - `DA728000-0000-1000-8000-00805F9B34FB`
- **Waveform Data** characteristic (`DA728001...`) — chunked write upload
- **Waveform Control** characteristic (`DA728002...`) — commands:
  - `0x01 <len>` — start upload
  - `0x02 <seq_id> <loops>` — play sequence
  - `0x03` — commit/upload waveform to DA728x
  - `0x04` — enable haptics
  - `0x05` — disable haptics
- **Status** characteristic (`DA728003...`) — read-only device status

## Hardware

- nRF54L15 development kit
- DA7280 connected via I2C (SERIAL20, SDA: P1.10, SCL: P1.11)
- I2C address: 0x4A

## Prerequisites

- [Rust](https://rustup.rs/) with the `thumbv8m.main-none-eabihf` target
- [probe-rs](https://probe.rs) for flashing
- LLVM/Clang (required by nrf-sdc build scripts)

## Build & Run

```bash
cd apps/demo-ble
cargo run --release
```

The device advertises as **"DA728x Haptics"**.

## Pairing with the Web Tool

Open `apps/waveform-builder/waveform_builder.html` in a browser that supports
Web Bluetooth (Chrome/Edge). Click **Connect BLE**, then **Upload Waveform**
and use the sequence playback buttons to trigger haptics in real time.
