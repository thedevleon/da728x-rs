//! Upload exported waveform memory on nRF54L15.
//!
//! This example shows how to upload the waveform as exported from the waveform builder
//! See: tools/example_waveform_memory.txt
//!
//! # Hardware Setup
//!
//! - DA7280 connected via TWIM (SERIAL20, SDA: P1.10, SCL: P1.11)
//! - I2C address: 0x4A (default)
//!
//!
//! # Running
//!
//! ```bash
//! cargo run --release --example waveform_memory
//! ```

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Timer;
use static_cell::ConstStaticCell;
use {defmt_rtt as _, panic_probe as _};

use da728x::waveform::WaveformMemory;
use da728x::waveform::WaveformMemoryTimebase;
use da728x::{DA728x, Variant};
use da728x_examples_common as common;
use da728x::config::ActuatorType;
use da728x::config::ActuatorConfig;

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
});

pub const DA7280_WAVEFORM_MEMORY: WaveformMemory = WaveformMemory::from_bytes(
    [
        0x06, 0x0C, 0x14, 0x15, 0x19, 0x1D, 0x1E, 0x20, 0x27, 0x2E, 0x2F, 0x31, 0x36, 0x3A, 0x43,
        0x46, 0x48, 0x49, 0x4B, 0x4D, 0x77, 0x37, 0xF7, 0xF0, 0xF7, 0xF0, 0x77, 0x77, 0x70, 0x70,
        0xF7, 0x07, 0xF0, 0x6A, 0x18, 0x4A, 0x18, 0x2A, 0x18, 0x11, 0x01, 0x90, 0x08, 0x88, 0x41,
        0x90, 0x10, 0x13, 0x04, 0x88, 0x04, 0x18, 0x04, 0x04, 0x04, 0x1E, 0x18, 0x1E, 0x18, 0x1A,
        0x18, 0x90, 0x3A, 0x18, 0x90, 0x5A, 0x18, 0x90, 0x04, 0x18, 0xA0, 0x1D, 0x09, 0x1E, 0x09,
        0x08, 0x29, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ], //
    78, // number of bytes in memory
    6, // number of snippets
    12, // number of sequences
);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("=== Waveform Memory Example (nRF54L15) ===");

    info!("Initializing TWI...");
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = Twim::new(
        p.SERIAL20,
        Irqs,
        p.P1_10,
        p.P1_11,
        twim::Config::default(),
        RAM_BUFFER.take(),
    );

    info!("Setting up DA7280 haptics driver...");
    let mut haptics = DA728x::new(twi, 0x4A, Variant::DA7280).await.unwrap();
    info!("DA7280 initialized successfully.");

    // let actuator_config = common::config::sparkfun_lra_config();
    let actuator_config = ActuatorConfig {
        actuator_type: ActuatorType::LRA,
        nominal_max_mV: 1_240,
        absolute_max_mV: 1_240,
        max_current_mA: 80,
        impedance_mOhm: 21_000,
        inductance_uH: 50,
        frequency_Hz: 240,
        pid_Kp_Ki: None,
    };

    info!(
        "Waveform memory: {} bytes, {} snippets, {} sequences",
        DA7280_WAVEFORM_MEMORY.len(),
        DA7280_WAVEFORM_MEMORY.num_snippets(),
        DA7280_WAVEFORM_MEMORY.num_sequences()
    );

    let device_config = common::config::rtwm_frequency_track();

    haptics
        .configure(actuator_config, device_config)
        .await
        .unwrap();
    info!("Configured for RTWM mode.");

    // Upload and verify waveform memory
    info!("Uploading waveform memory...");
    haptics
        .upload_waveform_memory(&DA7280_WAVEFORM_MEMORY, false)
        .await
        .unwrap();

    // set timebase
    haptics
        .set_timebase(WaveformMemoryTimebase::TIMEBASE_136_544_2176_4352)
        .await
        .unwrap();

    haptics.lock_waveform_memory().await.unwrap();
    haptics.enable().await.unwrap();

    // Clear any stale events
    let _ = haptics.get_events().await;

    info!("RTWM enabled. Playing effects...");

    loop {
        for n in 0..DA7280_WAVEFORM_MEMORY.num_sequences() {
            info!("Sequence: {}", n);
            haptics.play_sequence(n, 0).await.unwrap();
            Timer::after_millis(4_000).await;
        }

        if !common::demo::handle_faults(&mut haptics).await {
            info!("All effects OK");
        }

        Timer::after_millis(1000).await;
    }
}
