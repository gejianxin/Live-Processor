pub const NUM_INPUT_BUSES: usize = 6;
pub const NUM_OUTPUT_BUSES: usize = 6;

/// Real-time 6x6 Cross-Point Matrix Summing Engine with click-free gain ramping.
#[derive(Clone, Debug)]
pub struct RealtimeMatrixDsp {
    // Current smoothed gain coefficients [input][output]
    current_gains: [[f32; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES],
    // Target gains [input][output] (0.0 or 1.0)
    target_gains: [[f32; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES],
    // Ramp speed per sample
    ramp_step: f32,
    pub direct_asio_bypass: bool,
}

impl Default for RealtimeMatrixDsp {
    fn default() -> Self {
        let mut target = [[0.0_f32; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES];

        // Default broadcast routing connections
        target[0][0] = 1.0; // Mic -> Monitors
        target[0][2] = 1.0; // Mic -> OBS Stream
        target[0][4] = 1.0; // Mic -> Voice Mic
        target[2][0] = 1.0; // DAW -> Monitors
        target[2][1] = 1.0; // DAW -> Headphones
        target[2][2] = 1.0; // DAW -> OBS Stream
        target[3][0] = 1.0; // BGM -> Monitors
        target[3][2] = 1.0; // BGM -> OBS Stream
        target[4][0] = 1.0; // Chat -> Monitors
        target[4][1] = 1.0; // Chat -> Headphones

        Self {
            current_gains: target,
            target_gains: target,
            ramp_step: 0.005, // Smooth ~5ms de-zippering
            direct_asio_bypass: false,
        }
    }
}

impl RealtimeMatrixDsp {
    pub fn new_silent() -> Self {
        Self {
            current_gains: [[0.0; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES],
            target_gains: [[0.0; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES],
            ramp_step: 0.005,
            direct_asio_bypass: false,
        }
    }

    pub fn set_connection(&mut self, src: usize, dest: usize, connected: bool) {
        if src < NUM_INPUT_BUSES && dest < NUM_OUTPUT_BUSES {
            self.target_gains[src][dest] = if connected { 1.0 } else { 0.0 };
        }
    }

    pub fn set_matrix_from_bools(&mut self, connections: &[[bool; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES]) {
        for s in 0..NUM_INPUT_BUSES {
            for d in 0..NUM_OUTPUT_BUSES {
                self.target_gains[s][d] = if connections[s][d] { 1.0 } else { 0.0 };
            }
        }
    }

    /// Process a block of 6 stereo input streams into 6 stereo output streams.
    #[inline(always)]
    pub fn process_matrix(
        &mut self,
        inputs_l: &[Vec<f32>; NUM_INPUT_BUSES],
        inputs_r: &[Vec<f32>; NUM_INPUT_BUSES],
        outputs_l: &mut [Vec<f32>; NUM_OUTPUT_BUSES],
        outputs_r: &mut [Vec<f32>; NUM_OUTPUT_BUSES],
    ) {
        let block_size = outputs_l[0].len();

        // 1. Clear output buffers
        for d in 0..NUM_OUTPUT_BUSES {
            for s in 0..block_size {
                outputs_l[d][s] = 0.0;
                outputs_r[d][s] = 0.0;
            }
        }

        // 2. Hardware direct bypass mode
        if self.direct_asio_bypass {
            // Direct pass-through 1:1
            for i in 0..NUM_INPUT_BUSES.min(NUM_OUTPUT_BUSES) {
                outputs_l[i].copy_from_slice(&inputs_l[i]);
                outputs_r[i].copy_from_slice(&inputs_r[i]);
            }
            return;
        }

        // 3. Click-free matrix summation across all cross-points
        for s in 0..block_size {
            for src in 0..NUM_INPUT_BUSES {
                let sample_l = inputs_l[src][s];
                let sample_r = inputs_r[src][s];

                for dest in 0..NUM_OUTPUT_BUSES {
                    // Smooth gain de-zippering
                    let cur = &mut self.current_gains[src][dest];
                    let tgt = self.target_gains[src][dest];
                    if (*cur - tgt).abs() > self.ramp_step {
                        if *cur < tgt {
                            *cur += self.ramp_step;
                        } else {
                            *cur -= self.ramp_step;
                        }
                    } else {
                        *cur = tgt;
                    }

                    let g = *cur;
                    if g > 1e-4 {
                        outputs_l[dest][s] += sample_l * g;
                        outputs_r[dest][s] += sample_r * g;
                    }
                }
            }
        }
    }
}
