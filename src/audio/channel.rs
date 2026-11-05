use super::dsp::{BiquadFilter, BrickwallLimiter, DynamicCompressor, NoiseGate, StudioReverb};

pub struct ChannelStripProcessor {
    pub hpf: BiquadFilter,
    pub eq: BiquadFilter,
    pub compressor: DynamicCompressor,
    pub noise_gate: NoiseGate,
    pub reverb: StudioReverb,
    pub limiter: BrickwallLimiter,

    pub hpf_enabled: bool,
    pub eq_enabled: bool,
    pub comp_enabled: bool,
    pub gate_enabled: bool,
    pub reverb_enabled: bool,
    pub limiter_enabled: bool,

    pub volume_db: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,

    // Peak levels calculated from real PCM audio buffers
    pub real_peak_l: f32,
    pub real_peak_r: f32,
    pub real_rms_l: f32,
    pub real_rms_r: f32,
}

impl ChannelStripProcessor {
    pub fn new(sample_rate: f32, is_master: bool) -> Self {
        Self {
            hpf: BiquadFilter::new_high_pass(sample_rate, 80.0, 0.707),
            eq: BiquadFilter::new_peaking_eq(sample_rate, 3500.0, 1.2, 2.5),
            compressor: DynamicCompressor::new(sample_rate, -18.0, 3.5, 10.0, 100.0, 2.0),
            noise_gate: NoiseGate::new(sample_rate, -50.0, 5.0, 150.0),
            reverb: StudioReverb::new(sample_rate, 0.75, 0.25),
            limiter: BrickwallLimiter::new(sample_rate, -0.3),

            hpf_enabled: !is_master,
            eq_enabled: true,
            comp_enabled: true,
            gate_enabled: false,
            reverb_enabled: false,
            limiter_enabled: is_master,

            volume_db: 0.0,
            pan: 0.0,
            mute: false,
            solo: false,

            real_peak_l: 0.0,
            real_peak_r: 0.0,
            real_rms_l: 0.0,
            real_rms_r: 0.0,
        }
    }

    #[inline(always)]
    pub fn process_block(&mut self, buf_l: &mut [f32], buf_r: &mut [f32]) {
        if self.mute {
            for sample in buf_l.iter_mut() { *sample = 0.0; }
            for sample in buf_r.iter_mut() { *sample = 0.0; }
            self.real_peak_l = 0.0;
            self.real_peak_r = 0.0;
            self.real_rms_l = 0.0;
            self.real_rms_r = 0.0;
            return;
        }

        let gain = 10.0_f32.powf(self.volume_db / 20.0);

        // Equal power panning coefficients (-3dB center pan law)
        let pan_norm = (self.pan.clamp(-1.0, 1.0) + 1.0) * 0.25 * std::f32::consts::PI;
        let gain_l = pan_norm.cos() * gain;
        let gain_r = pan_norm.sin() * gain;

        let mut peak_l = 0.0_f32;
        let mut peak_r = 0.0_f32;
        let mut sum_sq_l = 0.0_f32;
        let mut sum_sq_r = 0.0_f32;

        let len = buf_l.len().min(buf_r.len());

        for i in 0..len {
            let mut s_l = buf_l[i];
            let mut s_r = buf_r[i];

            // 1. Noise Gate
            if self.gate_enabled {
                let (g_l, g_r) = self.noise_gate.process_sample(s_l, s_r);
                s_l = g_l;
                s_r = g_r;
            }

            // 2. High-Pass Filter (HPF)
            if self.hpf_enabled {
                let (f_l, f_r) = self.hpf.process_sample(s_l, s_r);
                s_l = f_l;
                s_r = f_r;
            }

            // 3. Parametric EQ
            if self.eq_enabled {
                let (e_l, e_r) = self.eq.process_sample(s_l, s_r);
                s_l = e_l;
                s_r = e_r;
            }

            // 4. Dynamic Compressor
            if self.comp_enabled {
                let (c_l, c_r) = self.compressor.process_sample(s_l, s_r);
                s_l = c_l;
                s_r = c_r;
            }

            // 5. Studio Reverb
            if self.reverb_enabled {
                let (r_l, r_r) = self.reverb.process_sample(s_l, s_r);
                s_l = r_l;
                s_r = r_r;
            }

            // 6. Volume & Pan
            s_l *= gain_l;
            s_r *= gain_r;

            // 7. Master Limiter
            if self.limiter_enabled {
                let (l_l, l_r) = self.limiter.process_sample(s_l, s_r);
                s_l = l_l;
                s_r = l_r;
            }

            buf_l[i] = s_l;
            buf_r[i] = s_r;

            // Metrics
            let abs_l = s_l.abs();
            let abs_r = s_r.abs();
            if abs_l > peak_l { peak_l = abs_l; }
            if abs_r > peak_r { peak_r = abs_r; }
            sum_sq_l += s_l * s_l;
            sum_sq_r += s_r * s_r;
        }

        // Smooth decay on peak levels
        self.real_peak_l = peak_l.max(self.real_peak_l * 0.85);
        self.real_peak_r = peak_r.max(self.real_peak_r * 0.85);
        self.real_rms_l = (sum_sq_l / len.max(1) as f32).sqrt();
        self.real_rms_r = (sum_sq_r / len.max(1) as f32).sqrt();
    }
}
