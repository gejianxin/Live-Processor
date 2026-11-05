use live_processor::audio::{
    BiquadFilter, BrickwallLimiter, DynamicCompressor, RealtimeMatrixDsp,
    SnapshotTransition, NUM_INPUT_BUSES, NUM_OUTPUT_BUSES,
};

#[test]
fn test_matrix_summing_dsp() {
    let mut matrix = RealtimeMatrixDsp::new_silent();
    let num_samples = 128;

    let in_l: [Vec<f32>; NUM_INPUT_BUSES] = std::array::from_fn(|_| vec![0.5; num_samples]);
    let in_r: [Vec<f32>; NUM_INPUT_BUSES] = std::array::from_fn(|_| vec![0.5; num_samples]);
    let mut out_l: [Vec<f32>; NUM_OUTPUT_BUSES] = std::array::from_fn(|_| vec![0.0; num_samples]);
    let mut out_r: [Vec<f32>; NUM_OUTPUT_BUSES] = std::array::from_fn(|_| vec![0.0; num_samples]);

    // Connect Source 0 -> Output 0
    let mut active = [[false; NUM_OUTPUT_BUSES]; NUM_INPUT_BUSES];
    active[0][0] = true;
    matrix.set_matrix_from_bools(&active);

    matrix.process_matrix(&in_l, &in_r, &mut out_l, &mut out_r);

    // Output 0 should have summed signal (smooth de-zippering warms up gain)
    assert!(out_l[0][num_samples - 1] > 0.0);
    assert!(out_r[0][num_samples - 1] > 0.0);
    // Output 1 should be silent (no connection)
    assert_eq!(out_l[1][num_samples - 1], 0.0);
    assert_eq!(out_r[1][num_samples - 1], 0.0);
}

#[test]
fn test_biquad_highpass_filter() {
    let mut hpf = BiquadFilter::new_high_pass(48000.0, 80.0, 0.707);

    // Pass DC signal (0 Hz): highpass should attenuate DC to near 0
    let mut val: f32 = 1.0;
    for _ in 0..1000 {
        let (out_l, _) = hpf.process_sample(1.0, 1.0);
        val = out_l;
    }
    assert!(val.abs() < 0.05, "DC signal should be attenuated by 80Hz HPF, got: {}", val);
}

#[test]
fn test_dynamic_compressor_gain_reduction() {
    let mut comp = DynamicCompressor::new(48000.0, -10.0, 4.0, 5.0, 50.0, 0.0);

    // Feed sustained signal that exceeds threshold
    let mut last_l = 1.0;
    for _ in 0..512 {
        let (out_l, _) = comp.process_sample(1.0, 1.0);
        last_l = out_l;
    }

    // Compressor should attenuate signal that exceeds threshold
    assert!(last_l < 0.95, "Signal should be compressed below 1.0, got: {}", last_l);
    assert!(comp.current_gr_db > 0.0, "Gain reduction should be active");
}

#[test]
fn test_brickwall_limiter_ceiling() {
    let mut limiter = BrickwallLimiter::new(48000.0, -0.3);
    let ceiling_linear = 10.0_f32.powf(-0.3 / 20.0);

    // Send hot signal that would otherwise clip (+8 dBFS)
    for _ in 0..256 {
        let (out_l, out_r) = limiter.process_sample(2.5, 2.5);
        assert!(out_l.abs() <= ceiling_linear + 1e-4, "L must not exceed ceiling: {} vs {}", out_l, ceiling_linear);
        assert!(out_r.abs() <= ceiling_linear + 1e-4, "R must not exceed ceiling: {} vs {}", out_r, ceiling_linear);
    }
}

#[test]
fn test_snapshot_transition_interpolation() {
    let mut transition = SnapshotTransition::default();
    transition.trigger_fade(0, 1, 0.5);

    assert!(transition.is_fading);
    assert!(transition.get_progress() >= 0.0);
    assert!(transition.get_progress() <= 1.0);
}
