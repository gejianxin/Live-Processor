use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

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
        }
    }
}

impl VirtualRack {
    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        let avail_w = ui.available_width();
        let avail_h = (ui.available_height() - 10.0).max(420.0);

        ui.vertical(|ui| {
            // Header Row
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("虚拟机架调音台", "Virtual Console Rack"))
                        .color(Color32::from_rgb(230, 236, 244))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(tr("(插入式效果器、高精度推子与立体声峰值表)", "(Insert Processing, dB Faders & Stereo Peak Meters)"))
                        .color(Color32::from_rgb(120, 130, 145))
                        .small(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("重置推子", "Reset Faders")).size(11.0)).clicked() {
                        for ch in &mut self.channels {
                            ch.volume_db = 0.0;
                            ch.pan = 0.0;
                            ch.mute = false;
                            ch.solo = false;
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // Responsive Horizontal Channel Strips (Expands to fill 100% width)
            let channel_count = self.channels.len() as f32;
            let spacing = 12.0;
            let total_spacing = spacing * (channel_count - 1.0);
            let strip_width = ((avail_w - total_spacing) / channel_count).max(180.0);

            let total_channels = self.channels.len();
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, ch) in self.channels.iter_mut().enumerate() {
                            render_channel_strip(ui, ch, idx, strip_width, avail_h, time);
                            if idx + 1 < total_channels {
                                ui.add_space(spacing);
                            }
                        }
                    });
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
) {
    let is_master = idx == 4;

    egui::Frame::NONE
        .fill(if is_master { Color32::from_rgb(25, 30, 38) } else { Color32::from_rgb(22, 25, 31) })
        .stroke(Stroke::new(1.0, if is_master { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(38, 44, 54) }))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_min_height(height);

            ui.vertical_centered(|ui| {
                // Channel Tag
                let channel_no = format!("{:02}", idx + 1);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(channel_no).monospace().size(10.0).color(Color32::from_rgb(110, 120, 135)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_master {
                            ui.label(egui::RichText::new("MAIN").size(9.0).strong().color(Color32::from_rgb(56, 189, 248)));
                        }
                    });
                });

                // Channel Name
                let display_name = tr(ch.name_zh, ch.name_en);
                ui.label(
                    egui::RichText::new(display_name)
                        .color(if is_master { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(220, 228, 238) })
                        .strong()
                        .size(12.0),
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Insert FX Slots List
                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 18, 22))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.set_width(width - 12.0);
                        for (fx_name, active) in ch.fx_slots.iter_mut() {
                            ui.horizontal(|ui| {
                                let dot_color = if *active {
                                    Color32::from_rgb(56, 189, 248)
                                } else {
                                    Color32::from_rgb(60, 68, 80)
                                };
                                if ui.button(egui::RichText::new("●").size(9.0).color(dot_color)).clicked() {
                                    *active = !*active;
                                }

                                ui.label(
                                    egui::RichText::new(*fx_name)
                                        .size(10.5)
                                        .color(if *active { Color32::from_rgb(200, 210, 225) } else { Color32::from_rgb(90, 100, 115) }),
                                );
                            });
                            ui.add_space(1.0);
                        }
                    });

                ui.add_space(8.0);

                // Pan Slider
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("L").size(9.0).color(Color32::from_rgb(110, 120, 135)));
                    ui.add(egui::Slider::new(&mut ch.pan, -1.0..=1.0).show_value(false));
                    ui.label(egui::RichText::new("R").size(9.0).color(Color32::from_rgb(110, 120, 135)));
                });

                ui.add_space(6.0);

                // Mute & Solo Buttons
                ui.horizontal(|ui| {
                    let m_btn = egui::Button::new(
                        egui::RichText::new("M")
                            .strong()
                            .size(11.0)
                            .color(if ch.mute { Color32::WHITE } else { Color32::from_rgb(160, 170, 185) }),
                    )
                    .fill(if ch.mute { Color32::from_rgb(220, 38, 38) } else { Color32::from_rgb(30, 35, 44) })
                    .stroke(Stroke::new(1.0, Color32::from_rgb(46, 54, 66)))
                    .corner_radius(egui::CornerRadius::same(3));

                    if ui.add_sized([((width - 40.0) / 2.0).max(30.0), 22.0], m_btn).clicked() {
                        ch.mute = !ch.mute;
                    }

                    let s_btn = egui::Button::new(
                        egui::RichText::new("S")
                            .strong()
                            .size(11.0)
                            .color(if ch.solo { Color32::BLACK } else { Color32::from_rgb(160, 170, 185) }),
                    )
                    .fill(if ch.solo { Color32::from_rgb(245, 158, 11) } else { Color32::from_rgb(30, 35, 44) })
                    .stroke(Stroke::new(1.0, Color32::from_rgb(46, 54, 66)))
                    .corner_radius(egui::CornerRadius::same(3));

                    if ui.add_sized([((width - 40.0) / 2.0).max(30.0), 22.0], s_btn).clicked() {
                        ch.solo = !ch.solo;
                    }
                });

                ui.add_space(8.0);

                // Main Fader & Stereo Peak VU Meters
                let fader_height = 140.0;
                ui.horizontal(|ui| {
                    // Vertical dB Fader
                    ui.add_sized(
                        [36.0, fader_height],
                        egui::Slider::new(&mut ch.volume_db, -60.0..=12.0)
                            .vertical()
                            .show_value(false),
                    );

                    ui.add_space(8.0);

                    // Dual Stereo VU Meters
                    let (vu_rect, _) = ui.allocate_exact_size(Vec2::new(18.0, fader_height), egui::Sense::hover());

                    let sim_l = if ch.mute { 0.0 } else {
                        (ch.current_peak_l + (time * 4.0 + (idx as f64)).sin() as f32 * 0.15).clamp(0.02, 0.96)
                    };
                    let sim_r = if ch.mute { 0.0 } else {
                        (ch.current_peak_r + (time * 4.2 + (idx as f64) + 1.0).sin() as f32 * 0.15).clamp(0.02, 0.96)
                    };

                    render_stereo_meter(ui, vu_rect, sim_l, sim_r);
                });

                ui.add_space(6.0);

                // Numerical dB label
                let val_text = if ch.volume_db <= -59.0 {
                    "-inf dB".to_string()
                } else {
                    format!("{:+.1} dB", ch.volume_db)
                };

                egui::Frame::NONE
                    .fill(Color32::from_rgb(16, 18, 22))
                    .corner_radius(egui::CornerRadius::same(3))
                    .inner_margin(egui::Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(val_text).monospace().size(11.0).color(Color32::from_rgb(56, 189, 248)));
                    });
            });
        });
}

fn render_stereo_meter(ui: &Ui, rect: Rect, level_l: f32, level_r: f32) {
    let p = ui.painter();
    p.rect_filled(rect, 2.0, Color32::from_rgb(16, 18, 22));

    let bar_w = (rect.width() - 4.0) / 2.0;
    let rect_l = Rect::from_min_size(rect.min + Vec2::new(1.0, 0.0), Vec2::new(bar_w, rect.height()));
    let rect_r = Rect::from_min_size(rect.min + Vec2::new(bar_w + 3.0, 0.0), Vec2::new(bar_w, rect.height()));

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
