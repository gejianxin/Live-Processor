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
                    volume_db: -2.5,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Pro-Q 3", true), ("Sidechain", true), ("Stereo Imager", false), ("Tape Sat", true)],
                    current_peak_l: 0.82,
                    current_peak_r: 0.84,
                },
                ChannelStrip {
                    name_zh: "背景音乐 / 媒体",
                    name_en: "BGM / MEDIA",
                    volume_db: -6.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Ducking", true), ("Warm EQ", false), ("Peak Limiter", true)],
                    current_peak_l: 0.55,
                    current_peak_r: 0.58,
                },
                ChannelStrip {
                    name_zh: "语音通话 / 连麦",
                    name_en: "VOICE / DISCORD",
                    volume_db: -1.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Noise Gate", true), ("Vocal Leveller", true), ("Output EQ", false)],
                    current_peak_l: 0.48,
                    current_peak_r: 0.45,
                },
                ChannelStrip {
                    name_zh: "主输出总线 MASTER",
                    name_en: "MASTER BUS",
                    volume_db: 0.0,
                    pan: 0.0,
                    mute: false,
                    solo: false,
                    fx_slots: vec![("Bus Glue", true), ("True Peak Limiter", true), ("Loudness LUFS", true)],
                    current_peak_l: 0.88,
                    current_peak_r: 0.89,
                },
            ],
        }
    }
}

impl VirtualRack {
    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("🎚️ 虚拟效果机架与通道条", "🎚️ Virtual Rack & Channel Strips"))
                        .color(Color32::from_rgb(0, 220, 255))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(tr("重置所有推子", "Reset All Faders")).clicked() {
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
            ui.label(
                egui::RichText::new(tr(
                    "专业调音台通道条，支持插入式效果器、高精度 dB 推子、声像平衡及立体声峰值电平表。",
                    "Live mixing console with insert processing slots, high-precision dB faders, and stereo peak meter monitoring."
                ))
                .color(Color32::from_rgb(160, 175, 195))
                .small(),
            );
            ui.add_space(10.0);

            // Horizontal strip layout
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (idx, ch) in self.channels.iter_mut().enumerate() {
                            render_channel_strip(ui, ch, idx, time);
                            ui.add_space(10.0);
                        }
                    });
                });
        });
    }
}

fn render_channel_strip(ui: &mut Ui, ch: &mut ChannelStrip, idx: usize, time: f64) {
    egui::Frame::NONE
        .fill(Color32::from_rgb(18, 22, 28))
        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 46, 58)))
        .corner_radius(6.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(170.0);
            ui.vertical_centered(|ui| {
                // Header Name
                let header_color = if idx == 4 {
                    Color32::from_rgb(255, 170, 0) // Master
                } else {
                    Color32::from_rgb(0, 220, 255)
                };
                let display_name = tr(ch.name_zh, ch.name_en);
                ui.label(
                    egui::RichText::new(display_name)
                        .color(header_color)
                        .strong()
                        .size(13.0),
                );
                ui.add_space(4.0);

                // Insert FX Slots
                egui::Frame::NONE
                    .fill(Color32::from_rgb(13, 16, 21))
                    .corner_radius(4.0)
                    .inner_margin(6.0)
                    .show(ui, |ui| {
                        ui.set_width(150.0);
                        for (fx_name, enabled) in &mut ch.fx_slots {
                            let slot_color = if *enabled {
                                Color32::from_rgb(0, 230, 180)
                            } else {
                                Color32::from_rgb(90, 100, 115)
                            };
                            let btn = egui::Button::new(
                                egui::RichText::new(*fx_name).size(10.0).color(slot_color),
                            )
                            .fill(if *enabled {
                                Color32::from_rgb(24, 34, 42)
                            } else {
                                Color32::from_rgb(18, 22, 28)
                            });
                            if ui.add(btn).clicked() {
                                *enabled = !*enabled;
                            }
                        }
                    });

                ui.add_space(8.0);

                // Pan knob / slider
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(tr("声像", "Pan")).size(10.0).color(Color32::GRAY));
                    ui.add(egui::Slider::new(&mut ch.pan, -1.0..=1.0).show_value(false));
                });

                ui.add_space(8.0);

                // Fader and Meters Side-by-Side
                ui.horizontal(|ui| {
                    // Vertical Fader
                    ui.vertical(|ui| {
                        ui.add(
                            egui::Slider::new(&mut ch.volume_db, -60.0..=6.0)
                                .vertical()
                                .show_value(false),
                        );
                        let db_text = if ch.volume_db <= -59.0 {
                            "-inf dB".to_string()
                        } else {
                            format!("{:+.1} dB", ch.volume_db)
                        };
                        ui.label(
                            egui::RichText::new(db_text)
                                .size(10.0)
                                .color(Color32::from_rgb(180, 195, 215)),
                        );
                    });

                    ui.add_space(8.0);

                    // Dual Stereo VU Meters
                    let (meter_rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 130.0), egui::Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(meter_rect, 2.0, Color32::from_rgb(10, 12, 16));

                    // Animated wave level
                    let jitter = (time * 6.0 + (idx as f64 * 1.5)).sin() as f32 * 0.12;
                    let level_l = if ch.mute { 0.0 } else { (ch.current_peak_l + jitter).clamp(0.02, 0.98) };
                    let level_r = if ch.mute { 0.0 } else { (ch.current_peak_r + jitter * 0.9).clamp(0.02, 0.98) };

                    // Left channel meter
                    render_meter_bar(&p, meter_rect.min.x, meter_rect.max.y, 7.0, meter_rect.height(), level_l);
                    // Right channel meter
                    render_meter_bar(&p, meter_rect.min.x + 8.5, meter_rect.max.y, 7.0, meter_rect.height(), level_r);
                });

                ui.add_space(8.0);

                // Mute and Solo buttons
                ui.horizontal(|ui| {
                    let mute_btn = egui::Button::new(
                        egui::RichText::new("M")
                            .strong()
                            .color(if ch.mute { Color32::WHITE } else { Color32::GRAY }),
                    )
                    .fill(if ch.mute { Color32::from_rgb(220, 40, 40) } else { Color32::from_rgb(28, 34, 42) });
                    if ui.add_sized([34.0, 24.0], mute_btn).clicked() {
                        ch.mute = !ch.mute;
                    }

                    let solo_btn = egui::Button::new(
                        egui::RichText::new("S")
                            .strong()
                            .color(if ch.solo { Color32::BLACK } else { Color32::GRAY }),
                    )
                    .fill(if ch.solo { Color32::from_rgb(255, 200, 0) } else { Color32::from_rgb(28, 34, 42) });
                    if ui.add_sized([34.0, 24.0], solo_btn).clicked() {
                        ch.solo = !ch.solo;
                    }
                });
            });
        });
}

fn render_meter_bar(p: &egui::Painter, x: f32, bottom_y: f32, width: f32, height: f32, level: f32) {
    let filled_h = height * level;
    let green_h = (height * 0.65).min(filled_h);
    let yellow_h = if filled_h > height * 0.65 { (height * 0.85 - height * 0.65).min(filled_h - height * 0.65) } else { 0.0 };
    let red_h = if filled_h > height * 0.85 { filled_h - height * 0.85 } else { 0.0 };

    if green_h > 0.0 {
        p.rect_filled(
            Rect::from_min_max(Pos2::new(x, bottom_y - green_h), Pos2::new(x + width, bottom_y)),
            1.0,
            Color32::from_rgb(0, 230, 100),
        );
    }
    if yellow_h > 0.0 {
        let base_y = bottom_y - height * 0.65;
        p.rect_filled(
            Rect::from_min_max(Pos2::new(x, base_y - yellow_h), Pos2::new(x + width, base_y)),
            1.0,
            Color32::from_rgb(255, 210, 0),
        );
    }
    if red_h > 0.0 {
        let base_y = bottom_y - height * 0.85;
        p.rect_filled(
            Rect::from_min_max(Pos2::new(x, base_y - red_h), Pos2::new(x + width, base_y)),
            1.0,
            Color32::from_rgb(255, 50, 50),
        );
    }
}
