use std::sync::atomic::Ordering;
use std::sync::Arc;
use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

use crate::audio::AudioSharedState;
use crate::i18n::tr;

#[derive(Clone)]
pub struct ChannelStrip {
    pub name_zh: &'static str,
    pub name_en: &'static str,
    pub volume_db: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub fx_slots: Vec<(&'static str, bool)>,
    pub current_peak_l: f32,
    pub current_peak_r: f32,
}

pub struct VirtualRack {
    pub channels: Vec<ChannelStrip>,
    pub audio_state: Option<Arc<AudioSharedState>>,
}

impl Default for VirtualRack {
    fn default() -> Self {
        Self {
            channels: vec![
                ChannelStrip {
                    name_zh: "麦克风 / 人声",
                    name_en: "MIC / VOCAL",
                    volume_db: 0.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("HPF 80Hz", true), ("Opto Comp", true), ("De-Esser", false), ("Studio Verb", true)],
                    current_peak_l: 0.65,
                    current_peak_r: 0.62,
                },
                ChannelStrip {
                    name_zh: "宿主 DAW / 合成器",
                    name_en: "DAW / SYNTH",
                    volume_db: -2.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Pro-Q 3", true), ("Sidechain", true), ("Stereo Imager", false), ("Tape Sat", true)],
                    current_peak_l: 0.80,
                    current_peak_r: 0.82,
                },
                ChannelStrip {
                    name_zh: "背景音乐 / 媒体",
                    name_en: "BGM / MEDIA",
                    volume_db: -6.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Ducking", true), ("Warm EQ", false), ("Peak Limiter", true)],
                    current_peak_l: 0.52,
                    current_peak_r: 0.55,
                },
                ChannelStrip {
                    name_zh: "语音通话 / 连麦",
                    name_en: "VOICE / CHAT",
                    volume_db: -1.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Noise Gate", true), ("Vocal Leveller", true), ("Output EQ", false)],
                    current_peak_l: 0.45,
                    current_peak_r: 0.42,
                },
                ChannelStrip {
                    name_zh: "主输出总线 MASTER",
                    name_en: "MASTER BUS",
                    volume_db: 0.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Bus Glue", true), ("True Peak Limiter", true), ("Loudness LUFS", true)],
                    current_peak_l: 0.85,
                    current_peak_r: 0.86,
                },
            ],
            audio_state: None,
        }
    }
}

impl VirtualRack {
    pub fn new(audio_state: Arc<AudioSharedState>) -> Self {
        let mut r = Self::default();
        r.audio_state = Some(audio_state);
        r.sync_to_audio_engine();
        r
    }

    pub fn sync_to_audio_engine(&self) {
        if let Some(state) = &self.audio_state {
            for (idx, ch) in self.channels.iter().enumerate() {
                if idx < state.channel_faders.len() {
                    state.channel_faders[idx].store(ch.volume_db.to_bits(), Ordering::Relaxed);
                }
                if idx < state.channel_pans.len() {
                    state.channel_pans[idx].store(ch.pan.to_bits(), Ordering::Relaxed);
                }
                if idx < state.channel_mutes.len() {
                    state.channel_mutes[idx].store(ch.mute, Ordering::Relaxed);
                }
                if idx < state.channel_solos.len() {
                    state.channel_solos[idx].store(ch.solo, Ordering::Relaxed);
                }
                for (s_idx, (_, active)) in ch.fx_slots.iter().enumerate() {
                    if s_idx < 4 && idx < state.channel_fx_active.len() {
                        state.channel_fx_active[idx][s_idx].store(*active, Ordering::Relaxed);
                    }
                }
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        let avail_w = ui.available_width();
        let avail_h = ui.available_height().max(420.0);

        ui.vertical(|ui| {
            // Header Row
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("虚拟机架调音台 (Virtual Audio Console Rack)", "Virtual Console Rack"))
                        .color(Color32::from_rgb(235, 240, 248))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(tr("- 插入式效果器插槽、高精度 dB 推子与立体声双通道峰值 VU 电平表", "- Insert Processing Slots, Precision dB Faders & Stereo Peak Meters"))
                        .color(Color32::from_rgb(120, 130, 145))
                        .small(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("重置所有推子至 0dB", "Reset All Faders")).size(11.0)).clicked() {
                        for ch in &mut self.channels {
                            ch.volume_db = 0.0;
                            ch.pan = 0.0;
                            ch.mute = false;
                            ch.solo = false;
                        }
                        self.sync_to_audio_engine();
                    }
                });
            });

            ui.add_space(8.0);

            // Responsive Horizontal Channel Strips (Guaranteed 100% visible, no cut-off)
            let channel_count = self.channels.len() as f32;
            let spacing = 8.0;
            let total_spacing = spacing * (channel_count - 1.0);
            let strip_outer_w = (avail_w - total_spacing - 4.0) / channel_count;
            let strip_inner_w = (strip_outer_w - 16.0).max(120.0);

            let total_channels = self.channels.len();
            let audio_ref = self.audio_state.clone();
            ui.horizontal(|ui| {
                for (idx, ch) in self.channels.iter_mut().enumerate() {
                    render_channel_strip(ui, ch, idx, strip_inner_w, avail_h, time, audio_ref.as_ref());
                    if idx + 1 < total_channels {
                        ui.add_space(spacing);
                    }
                }
            });
        });
    }
}

fn render_channel_strip(
    ui: &mut Ui,
    ch: &mut ChannelStrip,
    idx: usize,
    width: f32,
    height: f32,
    time: f64,
    audio_ref: Option<&Arc<AudioSharedState>>,
) {
    let is_master = idx == 4;

    egui::Frame::NONE
        .fill(if is_master { Color32::from_rgb(24, 28, 36) } else { Color32::from_rgb(20, 23, 29) })
        .stroke(Stroke::new(1.0, if is_master { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(38, 44, 56) }))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_min_height(height - 16.0);

            ui.vertical_centered(|ui| {
                // Channel Tag
                let channel_no = format!("CH {:02}", idx + 1);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(channel_no).monospace().size(10.0).color(Color32::from_rgb(120, 132, 148)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_master {
                            ui.label(egui::RichText::new("MASTER").size(9.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        }
                    });
                });

                // Channel Name Card
                let display_name = tr(ch.name_zh, ch.name_en);
                egui::Frame::NONE
                    .fill(if is_master { Color32::from_rgb(28, 38, 52) } else { Color32::from_rgb(26, 30, 38) })
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(4, 3))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.label(
                            egui::RichText::new(display_name)
                                .color(if is_master { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(225, 232, 242) })
                                .strong()
                                .size(11.0),
                        );
                    });

                ui.add_space(4.0);

                // Insert FX Slots Rack Card
                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 18, 23))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(32, 36, 46)))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.label(egui::RichText::new(tr("效果器插槽 (INSERTS)", "INSERTS")).size(8.5).color(Color32::from_rgb(100, 110, 125)).strong());
                        ui.add_space(2.0);

                        for (fx_idx, (fx_name, active)) in ch.fx_slots.iter_mut().enumerate() {
                            egui::Frame::NONE
                                .fill(if *active { Color32::from_rgb(24, 30, 40) } else { Color32::from_rgb(18, 20, 26) })
                                .corner_radius(egui::CornerRadius::same(3))
                                .inner_margin(egui::Margin::symmetric(4, 2))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let dot_color = if *active {
                                             Color32::from_rgb(56, 189, 248)
                                        } else {
                                            Color32::from_rgb(60, 68, 80)
                                        };
                                        if ui.button(egui::RichText::new("●").size(8.5).color(dot_color)).clicked() {
                                            *active = !*active;
                                            if let Some(state) = audio_ref {
                                                if idx < state.channel_fx_active.len() && fx_idx < 4 {
                                                    state.channel_fx_active[idx][fx_idx].store(*active, Ordering::Relaxed);
                                                }
                                            }
                                        }

                                        ui.label(
                                            egui::RichText::new(*fx_name)
                                                .size(9.5)
                                                .color(if *active { Color32::from_rgb(210, 220, 235) } else { Color32::from_rgb(100, 110, 125) }),
                                        );
                                    });
                                });
                            ui.add_space(1.5);
                        }
                    });

                ui.add_space(6.0);

                // Pan Slider
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("L").size(8.5).color(Color32::from_rgb(120, 130, 145)));
                    if ui.add(egui::Slider::new(&mut ch.pan, -1.0..=1.0).show_value(false)).changed() {
                        if let Some(state) = audio_ref {
                            if idx < state.channel_pans.len() {
                                state.channel_pans[idx].store(ch.pan.to_bits(), Ordering::Relaxed);
                            }
                        }
                    }
                    ui.label(egui::RichText::new("R").size(8.5).color(Color32::from_rgb(120, 130, 145)));
                });

                ui.add_space(4.0);

                // Mute & Solo Buttons
                let btn_w = ((width - 4.0) / 2.0).max(34.0);
                ui.horizontal(|ui| {
                    let m_btn = egui::Button::new(
                        egui::RichText::new("MUTE")
                            .strong()
                            .size(9.5)
                            .color(if ch.mute { Color32::WHITE } else { Color32::from_rgb(160, 170, 185) }),
                    )
                    .fill(if ch.mute { Color32::from_rgb(220, 38, 38) } else { Color32::from_rgb(28, 32, 40) })
                    .stroke(Stroke::new(1.0, Color32::from_rgb(46, 54, 66)))
                    .corner_radius(egui::CornerRadius::same(3));

                    if ui.add_sized([btn_w, 20.0], m_btn).clicked() {
                        ch.mute = !ch.mute;
                        if let Some(state) = audio_ref {
                            if idx < state.channel_mutes.len() {
                                state.channel_mutes[idx].store(ch.mute, Ordering::Relaxed);
                            }
                        }
                    }

                    let s_btn = egui::Button::new(
                        egui::RichText::new("SOLO")
                            .strong()
                            .size(9.5)
                            .color(if ch.solo { Color32::BLACK } else { Color32::from_rgb(160, 170, 185) }),
                    )
                    .fill(if ch.solo { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(28, 32, 40) })
                    .stroke(Stroke::new(1.0, Color32::from_rgb(46, 54, 66)))
                    .corner_radius(egui::CornerRadius::same(3));

                    if ui.add_sized([btn_w, 20.0], s_btn).clicked() {
                        ch.solo = !ch.solo;
                        if let Some(state) = audio_ref {
                            if idx < state.channel_solos.len() {
                                state.channel_solos[idx].store(ch.solo, Ordering::Relaxed);
                            }
                        }
                    }
                });

                ui.add_space(6.0);

                // Main Fader & Stereo Peak VU Meters
                let fader_height = (height - 310.0).clamp(140.0, 320.0);
                let fader_w = ((width - 24.0) * 0.6).clamp(26.0, 38.0);
                ui.horizontal(|ui| {
                    // Vertical dB Fader
                    if ui.add_sized(
                        [fader_w, fader_height],
                        egui::Slider::new(&mut ch.volume_db, -60.0..=12.0)
                            .vertical()
                            .show_value(false),
                    ).changed() {
                        if let Some(state) = audio_ref {
                            if idx < state.channel_faders.len() {
                                state.channel_faders[idx].store(ch.volume_db.to_bits(), Ordering::Relaxed);
                            }
                        }
                    }

                    ui.add_space(4.0);

                    // Dual Stereo VU Meters
                    let (vu_rect, _) = ui.allocate_exact_size(Vec2::new(16.0, fader_height), egui::Sense::hover());

                    let (meter_l, meter_r) = if let Some(state) = audio_ref {
                        if is_master {
                            let l = f32::from_bits(state.master_peak_l.load(Ordering::Relaxed));
                            let r = f32::from_bits(state.master_peak_r.load(Ordering::Relaxed));
                            if ch.mute { (0.0, 0.0) } else { (l, r) }
                        } else {
                            if ch.mute {
                                (0.0, 0.0)
                            } else {
                                let in_peak = if idx < state.input_peaks.len() {
                                    f32::from_bits(state.input_peaks[idx].load(Ordering::Relaxed))
                                } else {
                                    0.5
                                };
                                let gain = if ch.volume_db <= -59.0 { 0.0 } else { 10.0_f32.powf(ch.volume_db / 20.0) };
                                let pan_l = ((1.0 - ch.pan) * 0.5).sqrt();
                                let pan_r = ((1.0 + ch.pan) * 0.5).sqrt();
                                let l = (in_peak * gain * pan_l).clamp(0.0, 1.0);
                                let r = (in_peak * gain * pan_r).clamp(0.0, 1.0);
                                (l, r)
                            }
                        }
                    } else {
                        let sim_l = if ch.mute { 0.0 } else {
                            (ch.current_peak_l + (time * 4.0 + (idx as f64) * 0.8).sin() as f32 * 0.14).clamp(0.02, 0.96)
                        };
                        let sim_r = if ch.mute { 0.0 } else {
                            (ch.current_peak_r + (time * 4.2 + (idx as f64) * 0.8 + 1.2).sin() as f32 * 0.14).clamp(0.02, 0.96)
                        };
                        (sim_l, sim_r)
                    };

                    render_stereo_meter(ui, vu_rect, meter_l, meter_r);
                });

                ui.add_space(4.0);

                // Numerical dB readout badge
                let val_text = if ch.volume_db <= -59.0 {
                    "-inf dB".to_string()
                } else {
                    format!("{:+.1} dB", ch.volume_db)
                };

                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 18, 23))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(34, 40, 52)))
                    .corner_radius(egui::CornerRadius::same(3))
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(val_text).monospace().size(10.0).color(Color32::from_rgb(56, 189, 248)));
                        });
                    });
            });
        });
}

fn render_stereo_meter(ui: &Ui, rect: Rect, level_l: f32, level_r: f32) {
    let p = ui.painter();
    p.rect_filled(rect, 2.0, Color32::from_rgb(14, 16, 20));
    p.rect_stroke(rect, 2.0, Stroke::new(1.0, Color32::from_rgb(32, 38, 48)), egui::StrokeKind::Inside);

    let bar_w = (rect.width() - 5.0) / 2.0;
    let rect_l = Rect::from_min_size(rect.min + Vec2::new(1.5, 0.0), Vec2::new(bar_w, rect.height()));
    let rect_r = Rect::from_min_size(rect.min + Vec2::new(bar_w + 3.5, 0.0), Vec2::new(bar_w, rect.height()));

    render_meter_channel(p, rect_l, level_l);
    render_meter_channel(p, rect_r, level_r);
}

fn render_meter_channel(p: &egui::Painter, rect: Rect, level: f32) {
    let filled_h = rect.height() * level;
    let active_rect = Rect::from_min_max(
        Pos2::new(rect.min.x, rect.max.y - filled_h),
        rect.max,
    );

    let color = if level > 0.88 {
        Color32::from_rgb(248, 113, 113) // Peak red
    } else if level > 0.70 {
        Color32::from_rgb(245, 158, 11) // Amber
    } else {
        Color32::from_rgb(52, 211, 153) // Subtle emerald
    };

    p.rect_filled(active_rect, 1.0, color);
}
