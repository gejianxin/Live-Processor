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
    pub selected_channel: usize,
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
            selected_channel: 0,
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
        let avail_h = ui.available_height();

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

            ui.add_space(4.0);

            // Responsive geometry calculation: 100% fitting in both windowed and maximized modes
            let card_h = (avail_h - 36.0).max(320.0);
            let channel_count = self.channels.len() as f32;
            let spacing = 6.0;
            let total_spacing = spacing * (channel_count - 1.0);
            let effective_avail_w = (avail_w - 10.0).max(480.0);
            let strip_outer_w = ((effective_avail_w - total_spacing) / channel_count).floor();
            // Card has 6px left and 6px right margin = 12px
            let strip_inner_w = (strip_outer_w - 12.0).max(90.0);

            let audio_ref = self.audio_state.clone();
            let mut newly_selected = self.selected_channel;

            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = spacing;
                    ui.horizontal(|ui| {
                        for (idx, ch) in self.channels.iter_mut().enumerate() {
                            let is_selected = idx == self.selected_channel;
                            let clicked = render_channel_strip(
                                ui,
                                ch,
                                idx,
                                is_selected,
                                strip_inner_w,
                                card_h,
                                time,
                                audio_ref.as_ref(),
                            );
                            if clicked {
                                newly_selected = idx;
                            }
                        }
                    });
                });

            self.selected_channel = newly_selected;
        });
    }
}

fn render_channel_strip(
    ui: &mut Ui,
    ch: &mut ChannelStrip,
    idx: usize,
    is_selected: bool,
    width: f32,
    card_h: f32,
    time: f64,
    audio_ref: Option<&Arc<AudioSharedState>>,
) -> bool {
    let is_master = idx == 4;
    let mut clicked = false;

    // Distinct card frame styling: high contrast borders for unselected channels so racks are clearly separated!
    let (bg_color, border_stroke) = if is_selected {
        (
            Color32::from_rgb(26, 34, 48),
            Stroke::new(2.0, Color32::from_rgb(56, 189, 248)), // Vibrant Sky Blue for selected
        )
    } else if is_master {
        (
            Color32::from_rgb(24, 28, 38),
            Stroke::new(1.5, Color32::from_rgb(14, 165, 233)), // Sky blue master accent
        )
    } else {
        (
            Color32::from_rgb(22, 26, 35),
            Stroke::new(1.5, Color32::from_rgb(70, 84, 108)), // High-contrast clear steel border for unselected!
        )
    };

    let frame_resp = egui::Frame::NONE
        .fill(bg_color)
        .stroke(border_stroke)
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(6, 6))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_max_width(width);
            ui.set_min_height(card_h - 12.0);
            ui.set_max_height(card_h - 12.0);

            ui.vertical_centered(|ui| {
                // 1. Channel Header Plate (Clickable to select this channel)
                let header_rect = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    let channel_no = format!("CH {:02}", idx + 1);
                    ui.label(
                        egui::RichText::new(channel_no)
                            .monospace()
                            .strong()
                            .size(10.5)
                            .color(if is_selected {
                                Color32::from_rgb(56, 189, 248)
                            } else if is_master {
                                Color32::from_rgb(14, 165, 233)
                            } else {
                                Color32::from_rgb(170, 185, 205)
                            }),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_selected {
                            ui.label(egui::RichText::new(tr("● 选中", "● EDIT")).size(9.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        } else if is_master {
                            ui.label(egui::RichText::new("MASTER").size(9.0).strong().color(Color32::from_rgb(14, 165, 233)));
                        } else {
                            ui.label(egui::RichText::new(tr("就绪", "READY")).size(8.5).color(Color32::from_rgb(100, 115, 135)));
                        }
                    });
                }).response;

                if header_rect.interact(egui::Sense::click()).clicked() {
                    clicked = true;
                }

                ui.add_space(3.0);

                // 2. Channel Name Card
                let display_name = tr(ch.name_zh, ch.name_en);
                egui::Frame::NONE
                    .fill(if is_selected {
                        Color32::from_rgb(32, 44, 62)
                    } else if is_master {
                        Color32::from_rgb(28, 38, 52)
                    } else {
                        Color32::from_rgb(28, 33, 44)
                    })
                    .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(45, 54, 70) }))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(4, 3))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.label(
                            egui::RichText::new(display_name)
                                .color(if is_selected || is_master { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(230, 238, 248) })
                                .strong()
                                .size(10.5),
                        );
                    });

                ui.add_space(3.0);

                // 3. Insert FX Slots Rack Card (Responsive height)
                let is_compact_height = card_h < 430.0;
                let slot_padding = if is_compact_height { 1 } else { 2 };
                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 20, 27))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(42, 50, 65)))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(4, if is_compact_height { 3 } else { 4 }))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.label(egui::RichText::new(tr("效果器插槽 (INSERTS)", "INSERTS")).size(8.5).color(Color32::from_rgb(120, 135, 155)).strong());
                        ui.add_space(2.0);

                        for (fx_idx, (fx_name, active)) in ch.fx_slots.iter_mut().enumerate() {
                            egui::Frame::NONE
                                .fill(if *active { Color32::from_rgb(24, 32, 44) } else { Color32::from_rgb(18, 21, 28) })
                                .stroke(Stroke::new(0.5, if *active { Color32::from_rgb(56, 100, 140) } else { Color32::from_rgb(32, 38, 48) }))
                                .corner_radius(egui::CornerRadius::same(3))
                                .inner_margin(egui::Margin::symmetric(3, slot_padding))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        let dot_color = if *active {
                                            Color32::from_rgb(56, 189, 248)
                                        } else {
                                            Color32::from_rgb(70, 80, 95)
                                        };
                                        if ui.button(egui::RichText::new("●").size(8.0).color(dot_color)).clicked() {
                                            *active = !*active;
                                            if let Some(state) = audio_ref {
                                                if idx < state.channel_fx_active.len() && fx_idx < 4 {
                                                    state.channel_fx_active[idx][fx_idx].store(*active, Ordering::Relaxed);
                                                }
                                            }
                                        }

                                        ui.label(
                                            egui::RichText::new(*fx_name)
                                                .size(8.8)
                                                .color(if *active { Color32::from_rgb(225, 235, 248) } else { Color32::from_rgb(110, 122, 138) }),
                                        );

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            let tag = if *active { "ON" } else { "BYP" };
                                            let tag_color = if *active { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(75, 85, 100) };
                                            ui.label(egui::RichText::new(tag).monospace().size(7.5).color(tag_color));
                                        });
                                    });
                                });
                            ui.add_space(1.0);
                        }
                    });

                ui.add_space(3.0);

                // 4. Pan Slider Row
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;
                    ui.label(egui::RichText::new("L").size(8.5).color(Color32::from_rgb(130, 142, 160)));
                    let pan_w = (width - 24.0).max(40.0);
                    if ui.add_sized([pan_w, 16.0], egui::Slider::new(&mut ch.pan, -1.0..=1.0).show_value(false)).changed() {
                        if let Some(state) = audio_ref {
                            if idx < state.channel_pans.len() {
                                state.channel_pans[idx].store(ch.pan.to_bits(), Ordering::Relaxed);
                            }
                        }
                    }
                    ui.label(egui::RichText::new("R").size(8.5).color(Color32::from_rgb(130, 142, 160)));
                });

                ui.add_space(3.0);

                // 5. Mute & Solo Buttons (Full width distribution)
                let btn_gap = 4.0;
                let btn_w = ((width - btn_gap) / 2.0).floor().max(28.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = btn_gap;
                    let m_btn = egui::Button::new(
                        egui::RichText::new("MUTE")
                            .strong()
                            .size(9.0)
                            .color(if ch.mute { Color32::WHITE } else { Color32::from_rgb(180, 192, 210) }),
                    )
                    .fill(if ch.mute { Color32::from_rgb(220, 38, 38) } else { Color32::from_rgb(32, 38, 50) })
                    .stroke(Stroke::new(1.0, if ch.mute { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(55, 66, 84) }))
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
                            .size(9.0)
                            .color(if ch.solo { Color32::BLACK } else { Color32::from_rgb(180, 192, 210) }),
                    )
                    .fill(if ch.solo { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(32, 38, 50) })
                    .stroke(Stroke::new(1.0, if ch.solo { Color32::from_rgb(251, 191, 36) } else { Color32::from_rgb(55, 66, 84) }))
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

                ui.add_space(4.0);

                // 6. Dynamic Main Fader + dB Scale Ticks + Dual Stereo Peak VU Meters
                // Non-fader height is dynamically accounted for to guarantee 100% vertical fit on any window
                let non_fader_h = if is_compact_height { 210.0 } else { 235.0 };
                let fader_height = (card_h - non_fader_h).clamp(70.0, 360.0);

                // Pro Console Assembly: [Fader Track] [dB Markings] [Dual Stereo VU Meter]
                // Centered horizontally within the channel card!
                let fader_w = (width * 0.30).clamp(28.0, 48.0);
                let scale_w = (width * 0.22).clamp(24.0, 36.0);
                let meter_w = 20.0;
                let col_gap = 3.0;
                let total_assembly_w = fader_w + col_gap + scale_w + col_gap + meter_w;
                let left_pad = ((width - total_assembly_w) / 2.0).floor().max(0.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = col_gap;
                    if left_pad > 0.0 {
                        ui.add_space(left_pad);
                    }

                    // Fader Column
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

                    // dB Scale Column
                    let (scale_rect, _) = ui.allocate_exact_size(Vec2::new(scale_w, fader_height), egui::Sense::hover());
                    render_db_scale(ui.painter(), scale_rect);

                    // Dual Stereo VU Meter Column
                    let (vu_rect, _) = ui.allocate_exact_size(Vec2::new(meter_w, fader_height), egui::Sense::hover());

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
                            (ch.current_peak_l + (time * 4.0 + (idx as f64) * 0.8).sin() as f32 * 0.14).clamp(0.05, 0.96)
                        };
                        let sim_r = if ch.mute { 0.0 } else {
                            (ch.current_peak_r + (time * 4.2 + (idx as f64) * 0.8 + 1.2).sin() as f32 * 0.14).clamp(0.05, 0.96)
                        };
                        (sim_l, sim_r)
                    };

                    render_stereo_meter(ui, vu_rect, meter_l, meter_r);
                });

                ui.add_space(4.0);

                // 7. Numerical dB Readout Display Badge (Full width)
                let val_text = if ch.volume_db <= -59.0 {
                    "-inf dB".to_string()
                } else {
                    format!("{:+.1} dB", ch.volume_db)
                };

                egui::Frame::NONE
                    .fill(Color32::from_rgb(12, 16, 22))
                    .stroke(Stroke::new(1.0, if is_selected { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(45, 55, 72) }))
                    .corner_radius(egui::CornerRadius::same(3))
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(val_text)
                                    .monospace()
                                    .strong()
                                    .size(10.0)
                                    .color(if is_selected { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(100, 200, 255) }),
                            );
                        });
                    });
            });
        });

    if frame_resp.response.interact(egui::Sense::click()).clicked() {
        clicked = true;
    }

    clicked
}

fn render_db_scale(p: &egui::Painter, rect: Rect) {
    let marks = [
        (12.0_f32, "+12"),
        (6.0, "+6"),
        (0.0, " 0"),
        (-6.0, "-6"),
        (-12.0, "-12"),
        (-24.0, "-24"),
        (-40.0, "-40"),
        (-60.0, "-∞"),
    ];

    let v_min = -60.0_f32;
    let v_max = 12.0_f32;

    for (val, label) in marks {
        let t = 1.0 - (val - v_min) / (v_max - v_min);
        let y = rect.min.y + t * rect.height();

        let is_zero = (val - 0.0).abs() < 0.1;
        let line_color = if is_zero {
            Color32::from_rgb(56, 189, 248)
        } else {
            Color32::from_rgb(65, 78, 98)
        };
        let line_w = if is_zero { 1.5 } else { 1.0 };

        // Tick line
        p.line_segment(
            [Pos2::new(rect.min.x, y), Pos2::new(rect.min.x + 4.0, y)],
            Stroke::new(line_w, line_color),
        );

        // Label
        let text_color = if is_zero {
            Color32::from_rgb(56, 189, 248)
        } else {
            Color32::from_rgb(115, 130, 150)
        };
        p.text(
            Pos2::new(rect.min.x + 6.0, y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::monospace(7.8),
            text_color,
        );
    }
}

fn render_stereo_meter(ui: &Ui, rect: Rect, level_l: f32, level_r: f32) {
    let p = ui.painter();

    // Dark enclosure with clear hardware console border
    p.rect_filled(rect, 2.0, Color32::from_rgb(10, 12, 16));
    p.rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0, Color32::from_rgb(55, 68, 88)),
        egui::StrokeKind::Inside,
    );

    let inner = rect.shrink(2.0);
    let bar_w = ((inner.width() - 2.0) / 2.0).floor().max(4.0);
    let rect_l = Rect::from_min_size(inner.min, Vec2::new(bar_w, inner.height()));
    let rect_r = Rect::from_min_size(
        Pos2::new(inner.min.x + bar_w + 2.0, inner.min.y),
        Vec2::new(bar_w, inner.height()),
    );

    render_segmented_bar(p, rect_l, level_l);
    render_segmented_bar(p, rect_r, level_r);
}

fn render_segmented_bar(p: &egui::Painter, rect: Rect, level: f32) {
    let num_segments = 18;
    let seg_gap = 1.0;
    let total_gap = seg_gap * (num_segments - 1) as f32;
    let seg_h = ((rect.height() - total_gap) / num_segments as f32).max(1.5);

    for i in 0..num_segments {
        let seg_idx_from_bottom = num_segments - 1 - i;
        let frac_bottom = seg_idx_from_bottom as f32 / num_segments as f32;

        let y = rect.min.y + i as f32 * (seg_h + seg_gap);
        let seg_rect = Rect::from_min_size(Pos2::new(rect.min.x, y), Vec2::new(rect.width(), seg_h));

        let is_lit = level >= frac_bottom;

        let color = if frac_bottom > 0.88 {
            if is_lit {
                Color32::from_rgb(239, 68, 68) // Peak Clip Red
            } else {
                Color32::from_rgb(48, 18, 22) // Unlit LED Red
            }
        } else if frac_bottom > 0.70 {
            if is_lit {
                Color32::from_rgb(251, 191, 36) // Warning Amber
            } else {
                Color32::from_rgb(48, 36, 16) // Unlit LED Amber
            }
        } else {
            if is_lit {
                Color32::from_rgb(52, 211, 153) // Signal Emerald Green
            } else {
                Color32::from_rgb(16, 38, 28) // Unlit LED Green
            }
        };

        p.rect_filled(seg_rect, 1.0, color);
    }
}
