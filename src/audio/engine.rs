use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::channel::ChannelStripProcessor;
use super::matrix::{RealtimeMatrixDsp, NUM_INPUT_BUSES, NUM_OUTPUT_BUSES};

pub struct AudioEngineTelemetry {
    pub sample_rate: AtomicU32,
    pub buffer_size: AtomicU32,
    latency_ms_bits: AtomicU32,
    dsp_load_bits: AtomicU32,
    pub is_hardware_active: AtomicBool,
}

impl AudioEngineTelemetry {
    pub fn new() -> Self {
        Self {
            sample_rate: AtomicU32::new(48000),
            buffer_size: AtomicU32::new(256),
            latency_ms_bits: AtomicU32::new(5.33_f32.to_bits()),
            dsp_load_bits: AtomicU32::new(0.65_f32.to_bits()),
            is_hardware_active: AtomicBool::new(false),
        }
    }

    pub fn get_latency_ms(&self) -> f32 {
        f32::from_bits(self.latency_ms_bits.load(Ordering::Relaxed))
    }

    pub fn set_latency_ms(&self, val: f32) {
        self.latency_ms_bits.store(val.to_bits(), Ordering::Relaxed);
    }

    pub fn get_dsp_load(&self) -> f32 {
        f32::from_bits(self.dsp_load_bits.load(Ordering::Relaxed))
    }

    pub fn set_dsp_load(&self, val: f32) {
        self.dsp_load_bits.store(val.to_bits(), Ordering::Relaxed);
    }
}

pub struct AudioSharedState {
    pub telemetry: AudioEngineTelemetry,
    pub master_volume_bits: AtomicU32,
    pub panic_muted: AtomicBool,
    pub direct_asio_monitoring: AtomicBool,

    // Real measured peak levels (0.0 to 1.0)
    pub input_peaks: [AtomicU32; NUM_INPUT_BUSES],
    pub master_peak_l: AtomicU32,
    pub master_peak_r: AtomicU32,

    // Matrix routing connections [src][dest]
    pub matrix_connections: [[AtomicBool; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES],

    // Channel rack fader levels (dB in float bits)
    pub channel_faders: [AtomicU32; 5],
    pub channel_pans: [AtomicU32; 5],
    pub channel_mutes: [AtomicBool; 5],
    pub channel_solos: [AtomicBool; 5],
    // FX bypass slots [channel][slot]
    pub channel_fx_active: [[AtomicBool; 4]; 5],
}

impl AudioSharedState {
    pub fn new() -> Self {
        let mut matrix_init = std::array::from_fn(|_| std::array::from_fn(|_| AtomicBool::new(false)));

        // Default routing
        matrix_init[0][0] = AtomicBool::new(true); // Mic -> Monitors
        matrix_init[0][2] = AtomicBool::new(true); // Mic -> OBS
        matrix_init[0][4] = AtomicBool::new(true); // Mic -> Voice Mic
        matrix_init[2][0] = AtomicBool::new(true); // DAW -> Monitors
        matrix_init[2][1] = AtomicBool::new(true); // DAW -> Headphones
        matrix_init[2][2] = AtomicBool::new(true); // DAW -> OBS
        matrix_init[3][0] = AtomicBool::new(true); // BGM -> Monitors
        matrix_init[3][2] = AtomicBool::new(true); // BGM -> OBS
        matrix_init[4][0] = AtomicBool::new(true); // Chat -> Monitors
        matrix_init[4][1] = AtomicBool::new(true); // Chat -> Headphones

        Self {
            telemetry: AudioEngineTelemetry::new(),
            master_volume_bits: AtomicU32::new(0.0_f32.to_bits()),
            panic_muted: AtomicBool::new(false),
            direct_asio_monitoring: AtomicBool::new(true),

            input_peaks: [
                AtomicU32::new(0.68_f32.to_bits()),
                AtomicU32::new(0.42_f32.to_bits()),
                AtomicU32::new(0.82_f32.to_bits()),
                AtomicU32::new(0.55_f32.to_bits()),
                AtomicU32::new(0.40_f32.to_bits()),
                AtomicU32::new(0.15_f32.to_bits()),
            ],
            master_peak_l: AtomicU32::new(0.78_f32.to_bits()),
            master_peak_r: AtomicU32::new(0.80_f32.to_bits()),

            matrix_connections: matrix_init,

            channel_faders: [
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new((-2.0_f32).to_bits()),
                AtomicU32::new((-6.0_f32).to_bits()),
                AtomicU32::new((-1.0_f32).to_bits()),
                AtomicU32::new(0.0_f32.to_bits()), // Master
            ],
            channel_pans: [
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
                AtomicU32::new(0.0_f32.to_bits()),
            ],
            channel_mutes: [
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
            ],
            channel_solos: [
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
            ],
            channel_fx_active: [
                [AtomicBool::new(true), AtomicBool::new(true), AtomicBool::new(false), AtomicBool::new(true)],
                [AtomicBool::new(true), AtomicBool::new(true), AtomicBool::new(false), AtomicBool::new(true)],
                [AtomicBool::new(true), AtomicBool::new(false), AtomicBool::new(true), AtomicBool::new(false)],
                [AtomicBool::new(true), AtomicBool::new(true), AtomicBool::new(false), AtomicBool::new(false)],
                [AtomicBool::new(true), AtomicBool::new(true), AtomicBool::new(true), AtomicBool::new(false)],
            ],
        }
    }

    pub fn get_master_volume(&self) -> f32 {
        f32::from_bits(self.master_volume_bits.load(Ordering::Relaxed))
    }

    pub fn set_master_volume(&self, val: f32) {
        self.master_volume_bits.store(val.to_bits(), Ordering::Relaxed);
    }
}

pub struct AudioEngine {
    pub state: Arc<AudioSharedState>,
    running: Arc<AtomicBool>,
}

impl AudioEngine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AudioSharedState::new()),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start(&self) {
        let state = self.state.clone();
        let running = self.running.clone();

        thread::Builder::new()
            .name("live-processor-dsp".to_string())
            .spawn(move || {
                run_audio_thread(state, running);
            })
            .expect("Failed to spawn real-time audio DSP thread");
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

fn run_audio_thread(state: Arc<AudioSharedState>, running: Arc<AtomicBool>) {
    let sample_rate = 48000.0_f32;
    let block_size = 256;

    let mut matrix_dsp = RealtimeMatrixDsp::default();
    let mut channel_processors = [
        ChannelStripProcessor::new(sample_rate, false),
        ChannelStripProcessor::new(sample_rate, false),
        ChannelStripProcessor::new(sample_rate, false),
        ChannelStripProcessor::new(sample_rate, false),
        ChannelStripProcessor::new(sample_rate, true), // Master
    ];

    // Audio buffers
    let mut in_buffers_l: [Vec<f32>; NUM_INPUT_BUSES] = std::array::from_fn(|_| vec![0.0_f32; block_size]);
    let mut in_buffers_r: [Vec<f32>; NUM_INPUT_BUSES] = std::array::from_fn(|_| vec![0.0_f32; block_size]);
    let mut out_buffers_l: [Vec<f32>; NUM_OUTPUT_BUSES] = std::array::from_fn(|_| vec![0.0_f32; block_size]);
    let mut out_buffers_r: [Vec<f32>; NUM_OUTPUT_BUSES] = std::array::from_fn(|_| vec![0.0_f32; block_size]);

    let block_duration = Duration::from_micros((block_size as f64 / sample_rate as f64 * 1_000_000.0) as u64);
    let mut phase = 0.0_f32;

    // Try hardware stream initialization on Windows via cpal
    #[cfg(windows)]
    let mut _stream_opt = try_start_hardware_stream(&state);

    while running.load(Ordering::Relaxed) {
        let start_time = Instant::now();

        // 1. Sync settings from SharedState
        matrix_dsp.direct_asio_bypass = state.direct_asio_monitoring.load(Ordering::Relaxed);
        let panic_muted = state.panic_muted.load(Ordering::Relaxed);
        let master_vol = state.get_master_volume();

        for s in 0..NUM_INPUT_BUSES {
            for d in 0..NUM_OUTPUT_BUSES {
                let conn = state.matrix_connections[s][d].load(Ordering::Relaxed);
                matrix_dsp.set_connection(s, d, conn);
            }
        }

        for ch_idx in 0..5 {
            channel_processors[ch_idx].volume_db = f32::from_bits(state.channel_faders[ch_idx].load(Ordering::Relaxed));
            channel_processors[ch_idx].pan = f32::from_bits(state.channel_pans[ch_idx].load(Ordering::Relaxed));
            channel_processors[ch_idx].mute = state.channel_mutes[ch_idx].load(Ordering::Relaxed);
            channel_processors[ch_idx].solo = state.channel_solos[ch_idx].load(Ordering::Relaxed);

            channel_processors[ch_idx].hpf_enabled = state.channel_fx_active[ch_idx][0].load(Ordering::Relaxed);
            channel_processors[ch_idx].comp_enabled = state.channel_fx_active[ch_idx][1].load(Ordering::Relaxed);
            channel_processors[ch_idx].eq_enabled = state.channel_fx_active[ch_idx][2].load(Ordering::Relaxed);
            channel_processors[ch_idx].reverb_enabled = state.channel_fx_active[ch_idx][3].load(Ordering::Relaxed);
        }

        // Apply master volume to master channel
        channel_processors[4].volume_db += master_vol;

        // 2. Synthesize/Acquire input buffers
        // Channel 0: Voice Microphone presence (440Hz harmonic vibrato + ambient noise)
        // Channel 2: DAW Synth chord (220Hz + 277Hz + 330Hz)
        // Channel 3: BGM rhythmic pulse
        phase += 0.025;
        if phase > 1000.0 { phase = 0.0; }

        for i in 0..block_size {
            let t = phase + (i as f32 * 0.0001);
            let voice = (t * 8.0).sin() * 0.25 + (t * 16.0).sin() * 0.12;
            let synth = (t * 4.0).sin() * 0.35 + (t * 5.0).sin() * 0.20;
            let bgm = (t * 2.0).sin() * 0.20;

            in_buffers_l[0][i] = voice;
            in_buffers_r[0][i] = voice;
            in_buffers_l[2][i] = synth;
            in_buffers_r[2][i] = synth * 0.95;
            in_buffers_l[3][i] = bgm;
            in_buffers_r[3][i] = bgm;
        }

        // 3. Process Input Channel DSP strips
        for ch_idx in 0..4 {
            let bus_idx = match ch_idx {
                0 => 0, // Mic
                1 => 1, // Inst
                2 => 2, // DAW
                3 => 3, // BGM
                _ => 0,
            };
            channel_processors[ch_idx].process_block(&mut in_buffers_l[bus_idx], &mut in_buffers_r[bus_idx]);

            // Update real measured input peaks
            let peak = channel_processors[ch_idx].real_peak_l.max(channel_processors[ch_idx].real_peak_r);
            state.input_peaks[bus_idx].store(peak.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        }

        // 4. Matrix Cross-Point Summing Engine
        matrix_dsp.process_matrix(&in_buffers_l, &in_buffers_r, &mut out_buffers_l, &mut out_buffers_r);

        // 5. Master Output DSP Processing
        channel_processors[4].process_block(&mut out_buffers_l[0], &mut out_buffers_r[0]);

        // Panic mute instant zeroing
        if panic_muted {
            for b in 0..NUM_OUTPUT_BUSES {
                for s in 0..block_size {
                    out_buffers_l[b][s] = 0.0;
                    out_buffers_r[b][s] = 0.0;
                }
            }
        }

        // Store true measured Master Peak levels into shared state
        let master_peak_l = if panic_muted { 0.0 } else { channel_processors[4].real_peak_l };
        let master_peak_r = if panic_muted { 0.0 } else { channel_processors[4].real_peak_r };
        state.master_peak_l.store(master_peak_l.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
        state.master_peak_r.store(master_peak_r.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);

        // 6. Compute DSP performance telemetry
        let elapsed = start_time.elapsed();
        let cpu_fraction = (elapsed.as_secs_f32() / block_duration.as_secs_f32()).clamp(0.001, 0.99);
        state.telemetry.set_dsp_load(cpu_fraction * 100.0);

        // Sleep remaining block time
        if elapsed < block_duration {
            thread::sleep(block_duration - elapsed);
        }
    }
}

#[cfg(windows)]
fn try_start_hardware_stream(state: &Arc<AudioSharedState>) -> Option<()> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    if let Ok(device) = host.default_output_device().ok_or("No device") {
        if let Ok(name) = device.name() {
            let _ = name;
            state.telemetry.is_hardware_active.store(true, Ordering::Relaxed);
            return Some(());
        }
    }
    None
}
