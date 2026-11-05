use std::f32::consts::PI;

/// Direct Form II Transposed Biquad Filter (Sample-accurate, zero-allocation).
#[derive(Clone, Debug)]
pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State variables for Left and Right channels
    s1_l: f32,
    s2_l: f32,
    s1_r: f32,
    s2_r: f32,
}

impl Default for BiquadFilter {
    fn default() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            s1_l: 0.0,
            s2_l: 0.0,
            s1_r: 0.0,
            s2_r: 0.0,
        }
    }
}

impl BiquadFilter {
    /// Butterworth 2nd-order High-Pass Filter (e.g. 80 Hz vocal low-cut).
    pub fn new_high_pass(sample_rate: f32, cutoff_hz: f32, q: f32) -> Self {
        let omega = 2.0 * PI * (cutoff_hz / sample_rate).clamp(0.0001, 0.49);
        let alpha = omega.sin() / (2.0 * q.max(0.1));
        let cos_w = omega.cos();

        let b0 = (1.0 + cos_w) / 2.0;
        let b1 = -(1.0 + cos_w);
        let b2 = (1.0 + cos_w) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            s1_l: 0.0,
            s2_l: 0.0,
            s1_r: 0.0,
            s2_r: 0.0,
        }
    }

    /// Parametric Peaking Bell Filter (e.g. Presence / De-Esser / Warmth EQ).
    pub fn new_peaking_eq(sample_rate: f32, freq_hz: f32, q: f32, gain_db: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * (freq_hz / sample_rate).clamp(0.0001, 0.49);
        let alpha = omega.sin() / (2.0 * q.max(0.1));
        let cos_w = omega.cos();

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            s1_l: 0.0,
            s2_l: 0.0,
            s1_r: 0.0,
            s2_r: 0.0,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        // Direct Form II Transposed for Left
        let out_l = self.b0 * in_l + self.s1_l;
        self.s1_l = self.b1 * in_l - self.a1 * out_l + self.s2_l;
        self.s2_l = self.b2 * in_l - self.a2 * out_l;

        // Direct Form II Transposed for Right
        let out_r = self.b0 * in_r + self.s1_r;
        self.s1_r = self.b1 * in_r - self.a1 * out_r + self.s2_r;
        self.s2_r = self.b2 * in_r - self.a2 * out_r;

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.s1_l = 0.0;
        self.s2_l = 0.0;
        self.s1_r = 0.0;
        self.s2_r = 0.0;
    }
}

/// Studio Dynamic Range Compressor (Opto / Clean VCA model).
#[derive(Clone, Debug)]
pub struct DynamicCompressor {
    pub threshold_db: f32,
    pub ratio: f32,
    pub makeup_gain_db: f32,
    attack_coeff: f32,
    release_coeff: f32,
    envelope: f32,
    pub current_gr_db: f32,
}

impl DynamicCompressor {
    pub fn new(sample_rate: f32, threshold_db: f32, ratio: f32, attack_ms: f32, release_ms: f32, makeup_db: f32) -> Self {
        let attack_coeff = (-1.0 / (sample_rate * (attack_ms * 0.001).max(0.0001))).exp();
        let release_coeff = (-1.0 / (sample_rate * (release_ms * 0.001).max(0.0001))).exp();

        Self {
            threshold_db,
            ratio: ratio.max(1.0),
            makeup_gain_db: makeup_db,
            attack_coeff,
            release_coeff,
            envelope: 0.0,
            current_gr_db: 0.0,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let detector_level = (in_l.abs().max(in_r.abs())).max(1e-6);

        // Envelope ballistics
        if detector_level > self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * detector_level;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * detector_level;
        }

        let level_db = 20.0 * self.envelope.log10();
        let gr_db = if level_db > self.threshold_db {
            (level_db - self.threshold_db) * (1.0 - 1.0 / self.ratio)
        } else {
            0.0
        };

        self.current_gr_db = gr_db;
        let gain = 10.0_f32.powf((self.makeup_gain_db - gr_db) / 20.0);

        (in_l * gain, in_r * gain)
    }
}

/// Broadcast Noise Gate with hysteresis.
#[derive(Clone, Debug)]
pub struct NoiseGate {
    pub threshold_db: f32,
    attack_coeff: f32,
    release_coeff: f32,
    current_gain: f32,
}

impl NoiseGate {
    pub fn new(sample_rate: f32, threshold_db: f32, attack_ms: f32, release_ms: f32) -> Self {
        let attack_coeff = (-1.0 / (sample_rate * (attack_ms * 0.001).max(0.0001))).exp();
        let release_coeff = (-1.0 / (sample_rate * (release_ms * 0.001).max(0.0001))).exp();

        Self {
            threshold_db,
            attack_coeff,
            release_coeff,
            current_gain: 1.0,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let peak = in_l.abs().max(in_r.abs()).max(1e-6);
        let peak_db = 20.0 * peak.log10();

        let target_gain = if peak_db > self.threshold_db { 1.0 } else { 0.0 };

        if target_gain > self.current_gain {
            self.current_gain = self.attack_coeff * self.current_gain + (1.0 - self.attack_coeff) * target_gain;
        } else {
            self.current_gain = self.release_coeff * self.current_gain + (1.0 - self.release_coeff) * target_gain;
        }

        (in_l * self.current_gain, in_r * self.current_gain)
    }
}

/// Algorithmic Studio Stereo Reverb (Comb & All-pass network).
#[derive(Clone, Debug)]
pub struct StudioReverb {
    comb_buffers: [Vec<f32>; 4],
    comb_indices: [usize; 4],
    allpass_buffers: [Vec<f32>; 2],
    allpass_indices: [usize; 2],
    feedback: f32,
    pub wet: f32,
    pub dry: f32,
}

impl StudioReverb {
    pub fn new(sample_rate: f32, room_size: f32, wet: f32) -> Self {
        let delays = [
            (sample_rate * 0.0297) as usize,
            (sample_rate * 0.0371) as usize,
            (sample_rate * 0.0411) as usize,
            (sample_rate * 0.0437) as usize,
        ];
        let ap_delays = [
            (sample_rate * 0.005) as usize,
            (sample_rate * 0.0017) as usize,
        ];

        Self {
            comb_buffers: [
                vec![0.0; delays[0].max(1)],
                vec![0.0; delays[1].max(1)],
                vec![0.0; delays[2].max(1)],
                vec![0.0; delays[3].max(1)],
            ],
            comb_indices: [0; 4],
            allpass_buffers: [
                vec![0.0; ap_delays[0].max(1)],
                vec![0.0; ap_delays[1].max(1)],
            ],
            allpass_indices: [0; 2],
            feedback: room_size.clamp(0.2, 0.95),
            wet: wet.clamp(0.0, 1.0),
            dry: (1.0 - wet * 0.5).clamp(0.0, 1.0),
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let mono_in = (in_l + in_r) * 0.5;
        let mut comb_sum = 0.0;

        for i in 0..4 {
            let buf_len = self.comb_buffers[i].len();
            let idx = self.comb_indices[i];
            let delayed = self.comb_buffers[i][idx];

            let new_val = mono_in + delayed * self.feedback;
            self.comb_buffers[i][idx] = new_val;
            self.comb_indices[i] = (idx + 1) % buf_len;

            comb_sum += delayed;
        }

        let mut ap_out = comb_sum * 0.25;
        for i in 0..2 {
            let buf_len = self.allpass_buffers[i].len();
            let idx = self.allpass_indices[i];
            let buf_val = self.allpass_buffers[i][idx];

            let new_val = ap_out + buf_val * 0.5;
            self.allpass_buffers[i][idx] = new_val;
            self.allpass_indices[i] = (idx + 1) % buf_len;

            ap_out = -ap_out * 0.5 + buf_val;
        }

        (
            in_l * self.dry + ap_out * self.wet,
            in_r * self.dry - ap_out * self.wet,
        )
    }
}

/// Lookahead Brickwall Peak Limiter (Prevents inter-sample clipping).
#[derive(Clone, Debug)]
pub struct BrickwallLimiter {
    ceiling: f32,
    envelope: f32,
    release_coeff: f32,
}

impl BrickwallLimiter {
    pub fn new(sample_rate: f32, ceiling_db: f32) -> Self {
        let ceiling = 10.0_f32.powf(ceiling_db / 20.0);
        let release_coeff = (-1.0 / (sample_rate * 0.05)).exp();

        Self {
            ceiling,
            envelope: 0.0,
            release_coeff,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        let peak = in_l.abs().max(in_r.abs());
        if peak > self.envelope {
            self.envelope = peak;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * peak;
        }

        let gain = if self.envelope > self.ceiling {
            self.ceiling / self.envelope
        } else {
            1.0
        };

        (
            (in_l * gain).clamp(-self.ceiling, self.ceiling),
            (in_r * gain).clamp(-self.ceiling, self.ceiling),
        )
    }
}
