use std::time::Instant;

pub struct SnapshotTransition {
    pub is_fading: bool,
    pub start_time: Instant,
    pub duration_s: f32,
    pub source_cue_idx: usize,
    pub target_cue_idx: usize,
}

impl Default for SnapshotTransition {
    fn default() -> Self {
        Self {
            is_fading: false,
            start_time: Instant::now(),
            duration_s: 1.0,
            source_cue_idx: 0,
            target_cue_idx: 0,
        }
    }
}

impl SnapshotTransition {
    pub fn trigger_fade(&mut self, from_cue: usize, to_cue: usize, duration_s: f32) {
        self.is_fading = true;
        self.start_time = Instant::now();
        self.duration_s = duration_s.max(0.01);
        self.source_cue_idx = from_cue;
        self.target_cue_idx = to_cue;
    }

    pub fn get_progress(&mut self) -> f32 {
        if !self.is_fading {
            return 1.0;
        }

        let elapsed = self.start_time.elapsed().as_secs_f32();
        let progress = (elapsed / self.duration_s).clamp(0.0, 1.0);

        if progress >= 1.0 {
            self.is_fading = false;
        }

        progress
    }
}
