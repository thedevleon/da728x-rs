#![no_std]
#![no_main]

use defmt::unwrap;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_nrf::config::{self, ClockSpeed, HfclkSource, LfclkSource};
use embassy_nrf::cracen::{self};
use embassy_nrf::mode::Blocking;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl};
use static_cell::{ConstStaticCell, StaticCell};
use trouble_host::prelude::*;
use {defmt_rtt as _, panic_probe as _};

use da728x::config::ActuatorConfig;
use da728x::config::ActuatorType;
use da728x::waveform::WaveformMemory;
use da728x::{DA728x, Variant};
use da728x_examples_common as common;

// =============================================================================
// GATT Server Definition
// =============================================================================

#[gatt_server]
struct HapticsServer {
    haptics: HapticsService,
}

#[gatt_service(uuid = "DA728000-0000-1000-8000-00805F9B34FB")]
struct HapticsService {
    /// Chunked waveform data writes: [offset, data...]
    #[characteristic(uuid = "DA728001-0000-1000-8000-00805F9B34FB", write)]
    waveform_data: [u8; 20],

    /// Control commands: [cmd, arg1, arg2]
    #[characteristic(uuid = "DA728002-0000-1000-8000-00805F9B34FB", write)]
    waveform_control: [u8; 3],

    /// Device status (read-only, updated by firmware)
    #[characteristic(uuid = "DA728003-0000-1000-8000-00805F9B34FB", read, value = 1)]
    status: u8,
}

// =============================================================================
// Protocol Constants
// =============================================================================

const STATUS_READY: u8 = 0x01;
const STATUS_UPLOADING: u8 = 0x02;
const STATUS_UPLOAD_COMPLETE: u8 = 0x04;
const STATUS_ERROR: u8 = 0x08;

const CMD_START_UPLOAD: u8 = 0x01;
const CMD_PLAY: u8 = 0x02;
const CMD_COMMIT: u8 = 0x03;
const CMD_ENABLE: u8 = 0x04;
const CMD_DISABLE: u8 = 0x05;

// =============================================================================
// Shared State
// =============================================================================

struct UploadState {
    buffer: [u8; 100],
    expected_len: usize,
    received_len: usize,
    active: bool,
}

impl UploadState {
    const fn new() -> Self {
        Self {
            buffer: [0; 100],
            expected_len: 0,
            received_len: 0,
            active: false,
        }
    }
}

static UPLOAD_STATE: StaticCell<Mutex<NoopRawMutex, UploadState>> = StaticCell::new();

// =============================================================================
// Interrupt Bindings
// =============================================================================

bind_interrupts!(struct Irqs {
    SERIAL20 => twim::InterruptHandler<peripherals::SERIAL20>;
    SWI00 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    CLOCK_POWER => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO_0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER10 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    GRTC_3 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

// =============================================================================
// MPSL / SDC Setup
// =============================================================================

#[embassy_executor::task]
async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await
}

const L2CAP_TXQ: u8 = 3;
const L2CAP_RXQ: u8 = 3;

fn build_sdc<'d, const N: usize>(
    p: nrf_sdc::Peripherals<'d>,
    rng: &'d mut cracen::Cracen<'static, Blocking>,
    mpsl: &'d MultiprotocolServiceLayer,
    mem: &'d mut sdc::Mem<N>,
) -> Result<nrf_sdc::SoftdeviceController<'d>, nrf_sdc::Error> {
    sdc::Builder::new()?
        .support_adv()
        .support_peripheral()
        .peripheral_count(1)?
        .buffer_cfg(
            DefaultPacketPool::MTU as u16,
            DefaultPacketPool::MTU as u16,
            L2CAP_TXQ,
            L2CAP_RXQ,
        )?
        .build(p, rng, mpsl, mem)
}

// =============================================================================
// Main
// =============================================================================

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut econfig: config::Config = Default::default();
    econfig.clock_speed = ClockSpeed::CK128;
    econfig.hfclk_source = HfclkSource::ExternalXtal;
    econfig.lfclk_source = LfclkSource::ExternalXtal;
    let p = embassy_nrf::init(econfig);

    info!("=== DA728x BLE Haptics Demo ===");

    // --- MPSL ---
    let mpsl_p = mpsl::Peripherals::new(
        p.GRTC_CH7,
        p.GRTC_CH8,
        p.GRTC_CH9,
        p.GRTC_CH10,
        p.GRTC_CH11,
        p.TIMER10,
        p.TIMER20,
        p.TEMP,
        p.PPI10_CH0,
        p.PPI20_CH1,
        p.PPIB11_CH0,
        p.PPIB21_CH0,
    );
    let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
        source: mpsl::raw::MPSL_CLOCK_LF_SRC_XTAL as u8,
        rc_ctiv: 0,
        rc_temp_ctiv: 0,
        accuracy_ppm: 50,
        skip_wait_lfclk_started: false,
    };
    static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
    let mpsl = MPSL.init(unwrap!(mpsl::MultiprotocolServiceLayer::new(
        mpsl_p, Irqs, lfclk_cfg
    )));
    unwrap!(spawner.spawn(mpsl_task(&*mpsl)));

    // --- SDC ---
    let sdc_p = sdc::Peripherals::new(
        p.PPI00_CH1,
        p.PPI00_CH3,
        p.PPI10_CH1,
        p.PPI10_CH2,
        p.PPI10_CH3,
        p.PPI10_CH4,
        p.PPI10_CH5,
        p.PPI10_CH6,
        p.PPI10_CH7,
        p.PPI10_CH8,
        p.PPI10_CH9,
        p.PPI10_CH10,
        p.PPI10_CH11,
        p.PPIB00_CH1,
        p.PPIB00_CH2,
        p.PPIB00_CH3,
        p.PPIB10_CH1,
        p.PPIB10_CH2,
        p.PPIB10_CH3,
    );
    let mut rng = cracen::Cracen::new_blocking(p.CRACEN);
    let mut sdc_mem = sdc::Mem::<4720>::new();
    let sdc = unwrap!(build_sdc(sdc_p, &mut rng, mpsl, &mut sdc_mem));

    // --- TWIM / DA728x ---
    static RAM_BUFFER: ConstStaticCell<[u8; 16]> = ConstStaticCell::new([0; 16]);
    let twi = Twim::new(
        p.SERIAL20,
        Irqs,
        p.P1_10,
        p.P1_11,
        twim::Config::default(),
        RAM_BUFFER.take(),
    );

    info!("Initializing DA7280...");
    let mut haptics = unwrap!(DA728x::new(twi, 0x4A, Variant::DA7280).await);
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

    // RTWM Mode
    let device_config = common::config::rtwm_frequency_track();

    // Wideband Mode
    // let device_config = common::config::rtwm_wideband();

    unwrap!(haptics.configure(actuator_config, device_config).await);
    unwrap!(haptics.enable().await);
    info!("DA7280 configured and enabled in RTWM mode.");

    // --- Shared upload state ---
    let upload_state = UPLOAD_STATE.init(Mutex::new(UploadState::new()));

    // --- BLE Stack ---
    let address: Address = Address::random([0xff, 0x8f, 0x1a, 0x05, 0xe4, 0xff]);
    let mut resources: HostResources<DefaultPacketPool, 1, 2> = HostResources::new();
    let stack = trouble_host::new(sdc, &mut resources).set_random_address(address);
    let Host {
        mut peripheral,
        runner,
        ..
    } = stack.build();

    let server = unwrap!(HapticsServer::new_with_config(GapConfig::Peripheral(
        PeripheralConfig {
            name: "DA728x Haptics",
            appearance: &appearance::UNKNOWN,
        }
    )));

    // Run BLE host task concurrently with application logic
    let _ = select(ble_task(runner), async {
        loop {
            match advertise("DA728x Haptics", &mut peripheral, &server).await {
                Ok(conn) => {
                    info!("[main] connection established");
                    let a = gatt_events_task(&server, &conn, &mut haptics, upload_state);
                    let b = connection_keepalive();
                    match select(a, b).await {
                        Either::First(Ok(())) => info!("[main] gatt task ended"),
                        Either::First(Err(e)) => warn!("[main] gatt error: {:?}", e),
                        Either::Second(()) => info!("[main] keepalive ended"),
                    }
                }
                Err(e) => {
                    let e = defmt::Debug2Format(&e);
                    core::panic!("[adv] error: {:?}", e);
                }
            }
        }
    })
    .await;
}

// =============================================================================
// BLE Background Task
// =============================================================================

async fn ble_task<C: Controller, P: PacketPool>(mut runner: Runner<'_, C, P>) {
    loop {
        if let Err(e) = runner.run().await {
            let e = defmt::Debug2Format(&e);
            core::panic!("[ble_task] error: {:?}", e);
        }
    }
}

// =============================================================================
// Advertising
// =============================================================================

async fn advertise<'values, 'server, C: Controller>(
    name: &'values str,
    peripheral: &mut Peripheral<'values, C, DefaultPacketPool>,
    server: &'server HapticsServer<'values>,
) -> Result<GattConnection<'values, 'server, DefaultPacketPool>, BleHostError<C::Error>> {
    let mut advertiser_data = [0; 31];
    let len = AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(name.as_bytes()),
        ],
        &mut advertiser_data[..],
    )?;
    let advertiser = peripheral
        .advertise(
            &Default::default(),
            Advertisement::ConnectableScannableUndirected {
                adv_data: &advertiser_data[..len],
                scan_data: &[],
            },
        )
        .await?;
    info!("[adv] advertising");
    let conn = advertiser.accept().await?.with_attribute_server(server)?;
    info!("[adv] connection established");
    Ok(conn)
}

// =============================================================================
// Connection Keepalive
// =============================================================================

async fn connection_keepalive() {
    // Loop until the connection is closed (this task is cancelled by select)
    loop {
        Timer::after_secs(1).await;
    }
}

// =============================================================================
// GATT Event Handler
// =============================================================================

async fn gatt_events_task(
    server: &HapticsServer<'_>,
    conn: &GattConnection<'_, '_, DefaultPacketPool>,
    driver: &mut DA728x<Twim<'static>>,
    upload_state: &'static Mutex<NoopRawMutex, UploadState>,
) -> Result<(), Error> {
    let waveform_data = server.haptics.waveform_data;
    let waveform_control = server.haptics.waveform_control;
    let status_char = server.haptics.status;

    let reason = loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => break reason,
            GattConnectionEvent::Gatt { event } => {
                match &event {
                    GattEvent::Read(event) => {
                        if event.handle() == status_char.handle {
                            let status = server.get(&status_char).unwrap_or(STATUS_READY);
                            info!("[gatt] Status read: {}", status);
                        }
                    }
                    GattEvent::Write(event) => {
                        if event.handle() == waveform_data.handle {
                            let data = event.data();
                            // Protocol: [offset, data...]
                            if data.len() >= 2 {
                                let offset = data[0] as usize;
                                let chunk = &data[1..];
                                let mut state = upload_state.lock().await;
                                if state.active
                                    && offset + chunk.len() <= 100
                                    && offset + chunk.len() <= state.expected_len
                                {
                                    state.buffer[offset..offset + chunk.len()]
                                        .copy_from_slice(chunk);
                                    state.received_len += chunk.len();
                                    info!(
                                        "[gatt] Received {} bytes at offset {} ({}/{})",
                                        chunk.len(),
                                        offset,
                                        state.received_len,
                                        state.expected_len
                                    );
                                } else {
                                    warn!(
                                        "[gatt] Ignored chunk: offset={}, len={}, active={}, expected={}",
                                        offset,
                                        chunk.len(),
                                        state.active,
                                        state.expected_len
                                    );
                                }
                            }
                        } else if event.handle() == waveform_control.handle {
                            let data = event.data();
                            if !data.is_empty() {
                                match data[0] {
                                    CMD_START_UPLOAD if data.len() >= 2 => {
                                        let len = data[1] as usize;
                                        let mut state = upload_state.lock().await;
                                        state.buffer = [0; 100];
                                        state.expected_len = len.min(100);
                                        state.received_len = 0;
                                        state.active = true;
                                        drop(state);
                                        server.set(&status_char, &STATUS_UPLOADING).ok();
                                        info!("[gatt] Start upload: {} bytes", len);
                                    }
                                    CMD_PLAY if data.len() >= 3 => {
                                        let seq_id = data[1];
                                        let loops = data[2];
                                        info!("[gatt] Play sequence {} x{}", seq_id, loops + 1);
                                        if let Err(e) = driver.play_sequence(seq_id, loops).await {
                                            warn!("[gatt] Play failed: {:?}", e);
                                            server.set(&status_char, &STATUS_ERROR).ok();
                                        }
                                    }
                                    CMD_COMMIT => {
                                        info!("[gatt] Commit upload");
                                        let mut status = STATUS_UPLOAD_COMPLETE;
                                        {
                                            let state = upload_state.lock().await;
                                            if state.active
                                                && state.received_len >= state.expected_len
                                            {
                                                let expected_len = state.expected_len;
                                                let num_snippets = state.buffer[0];
                                                let num_sequences = state.buffer[1];
                                                let mut mem_bytes = [0u8; 100];
                                                mem_bytes[..expected_len]
                                                    .copy_from_slice(&state.buffer[..expected_len]);
                                                drop(state);
                                                let memory = WaveformMemory::from_bytes(
                                                    mem_bytes,
                                                    expected_len as u8,
                                                    num_snippets,
                                                    num_sequences,
                                                );
                                                if let Err(e) = driver
                                                    .upload_waveform_memory(&memory, false)
                                                    .await
                                                {
                                                    warn!("[gatt] Upload failed: {:?}", e);
                                                    status = STATUS_ERROR;
                                                } else {
                                                    info!("[gatt] Upload complete");
                                                }
                                            } else {
                                                warn!("[gatt] Commit with incomplete data");
                                                status = STATUS_ERROR;
                                            }
                                        }
                                        server.set(&status_char, &status).ok();
                                    }
                                    CMD_ENABLE => {
                                        info!("[gatt] Enable haptics");
                                        if let Err(e) = driver.enable().await {
                                            warn!("[gatt] Enable failed: {:?}", e);
                                            server.set(&status_char, &STATUS_ERROR).ok();
                                        }
                                    }
                                    CMD_DISABLE => {
                                        info!("[gatt] Disable haptics");
                                        if let Err(e) = driver.disable().await {
                                            warn!("[gatt] Disable failed: {:?}", e);
                                            server.set(&status_char, &STATUS_ERROR).ok();
                                        }
                                    }
                                    _ => {
                                        warn!("[gatt] Unknown command: {}", data[0]);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                match event.accept() {
                    Ok(reply) => reply.send().await,
                    Err(e) => warn!("[gatt] error sending response: {:?}", e),
                }
            }
            _ => {}
        }
    };
    info!("[gatt] disconnected: {:?}", reason);
    Ok(())
}
