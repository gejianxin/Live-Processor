use egui::{Color32, Stroke, Ui};

use crate::i18n::tr;

pub struct CueItem {
    pub id: usize,
    pub title_zh: &'static str,
    pub title_en: &'static str,
    pub description_zh: &'static str,
    pub description_en: &'static str,
    pub fade_time_s: f32,
    pub midi_trigger: &'static str,
    pub active: bool,
}

pub struct SnapshotManager {
    pub cues: Vec<CueItem>,
    pub current_cue_idx: usize,
    pub crossfade_progress: f32,
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self {
            cues: vec![
                CueItem {
                    id: 1,
                    title_zh: "01. 开场音乐 / 背景暖场",
                    title_en: "01. Pre-Show / House Music",
                    description_zh: "背景音乐直通总输出，麦克风静音，各路发送保持低电平",
                    description_en: "BGM routed to House Main, Mics muted, Submixes safe",
                    fade_time_s: 2.0,
                    midi_trigger: "Note C1 (36)",
                    active: false,
                },
                CueItem {
                    id: 2,
                    title_zh: "02. 原声吉他与主唱独唱",
                    title_en: "02. Acoustic Intro & Lead Vocal",
                    description_zh: "人声 80Hz 高通，温润压缩器激活，加载 3.2s 大厅混响",
                    description_en: "HPF 80Hz on Vocals, Warm Opto Comp, Long Hall Reverb (3.2s)",
                    fade_time_s: 1.5,
                    midi_trigger: "Note D1 (38)",
                    active: true,
                },
                CueItem {
                    id: 3,
                    title_zh: "03. 全乐队摇滚与电子鼓侧链",
                    title_en: "03. Full Band Rock & Sidechain",
                    description_zh: "DAW 合成器挂载底鼓侧链闪避，人声紧缩限制，直播流增益推满",
                    description_en: "DAW synth sidechained to kick, Vocal aggressive limiter, Stream feed hot",
                    fade_time_s: 1.0,
                    midi_trigger: "Note E1 (40)",
                    active: false,
                },
                CueItem {
                    id: 4,
                    title_zh: "04. 主持人讲话 / 舞台互动",
                    title_en: "04. Speech / MC & Stage Talk",
                    description_zh: "背景音乐自动闪避 -12dB，人声去齿音与小房间环境，舞台监听干声",
                    description_en: "Music ducked -12dB, Vocal de-esser + tight room reverb, Monitor dry",
                    fade_time_s: 0.8,
                    midi_trigger: "Note F1 (41)",
                    active: false,
                },
                CueItem {
                    id: 5,
                    title_zh: "05. 压轴返场 & 终场大合唱",
                    title_en: "05. Encore & Grand Finale",
                    description_zh: "所有子混音全开，总线 True Peak 限制天花板 -0.3dB，响度 -14 LUFS",
                    description_en: "All submixes active, True Peak Limiter ceiling -0.3dB, Loudness -14 LUFS",
                    fade_time_s: 2.5,
                    midi_trigger: "Note G1 (43)",
                    active: false,
                },
            ],
            current_cue_idx: 1,
            crossfade_progress: 1.0,
        }
    }
}

impl SnapshotManager {
    pub fn render(&mut self, ui: &mut Ui) {
        let avail_w = ui.available_width();
        let avail_h = (ui.available_height() - 10.0).max(400.0);

        ui.vertical(|ui| {
            // Header Row
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("现场快照与演出自动化", "Live Snapshots & Automation"))
                        .color(Color32::from_rgb(230, 236, 244))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(tr("(场景平滑过渡、MIDI 控制与 Cue 列表管理)", "(Scene Transitions, MIDI Triggers & Cue Lists)"))
                        .color(Color32::from_rgb(120, 130, 145))
                        .small(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("▶ 执行下一场景 (GO)", "▶ GO (Next Cue)")).strong().color(Color32::from_rgb(52, 211, 153))).clicked() {
                        if self.current_cue_idx + 1 < self.cues.len() {
                            self.set_active_cue(self.current_cue_idx + 1);
                        }
                    }
                    if ui.button(egui::RichText::new(tr("◀ 上一场景", "◀ Previous Cue")).size(11.0)).clicked() {
                        if self.current_cue_idx > 0 {
                            self.set_active_cue(self.current_cue_idx - 1);
                        }
                    }
                });
            });

            ui.add_space(8.0);

            // Responsive Two-Column Layout (Left: Cue Table, Right: Active Scene Inspector)
            let inspector_w = 300.0;
            let list_w = (avail_w - inspector_w - 16.0).max(480.0);

            ui.horizontal(|ui| {
                // Left Panel: Cue List
                egui::Frame::NONE
                    .fill(Color32::from_rgb(22, 25, 31))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 54)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_width(list_w);
                        ui.set_min_height(avail_h);

                        ui.label(egui::RichText::new(tr("演出场景列表 (Cue Stack)", "Cue Stack")).strong().size(13.0).color(Color32::from_rgb(220, 228, 238)));
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                for i in 0..self.cues.len() {
                                    let is_active = self.cues[i].active;
                                    let is_selected = self.current_cue_idx == i;

                                    let card_bg = if is_active {
                                        Color32::from_rgb(28, 38, 52)
                                    } else if is_selected {
                                        Color32::from_rgb(28, 33, 42)
                                    } else {
                                        Color32::from_rgb(20, 23, 29)
                                    };

                                    let border_color = if is_active {
                                        Color32::from_rgb(56, 189, 248)
                                    } else {
                                        Color32::from_rgb(34, 40, 50)
                                    };

                                    egui::Frame::NONE
                                        .fill(card_bg)
                                        .stroke(Stroke::new(1.0, border_color))
                                        .corner_radius(egui::CornerRadius::same(4))
                                        .inner_margin(10.0)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                // Status indicator
                                                let bullet = if is_active { "▶" } else { " " };
                                                ui.label(egui::RichText::new(bullet).strong().color(Color32::from_rgb(52, 211, 153)));

                                                let title = tr(self.cues[i].title_zh, self.cues[i].title_en);
                                                ui.label(egui::RichText::new(title).strong().size(12.0).color(if is_active { Color32::from_rgb(240, 245, 255) } else { Color32::from_rgb(180, 192, 210) }));

                                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                    if ui.button(egui::RichText::new(tr("召回", "RECALL")).size(10.5)).clicked() {
                                                        self.set_active_cue(i);
                                                    }

                                                    ui.add_space(8.0);
                                                    let fade_text = format!("{:.1}s", self.cues[i].fade_time_s);
                                                    ui.label(egui::RichText::new(fade_text).monospace().size(10.5).color(Color32::from_rgb(130, 140, 155)));

                                                    ui.add_space(8.0);
                                                    ui.label(egui::RichText::new(self.cues[i].midi_trigger).size(10.0).color(Color32::from_rgb(110, 120, 135)));
                                                });
                                            });

                                            ui.add_space(2.0);
                                            let desc = tr(self.cues[i].description_zh, self.cues[i].description_en);
                                            ui.label(egui::RichText::new(desc).size(10.5).color(Color32::from_rgb(130, 140, 155)));
                                        });

                                    ui.add_space(4.0);
                                }
                            });
                    });

                ui.add_space(12.0);

                // Right Panel: Selected Cue Inspector
                egui::Frame::NONE
                    .fill(Color32::from_rgb(22, 25, 31))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 54)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(inspector_w);
                        ui.set_min_height(avail_h);

                        ui.label(egui::RichText::new(tr("当前快照属性", "Active Cue Inspector")).strong().size(13.0).color(Color32::from_rgb(220, 228, 238)));
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(10.0);

                        let cur = &self.cues[self.current_cue_idx];
                        let cur_title = tr(cur.title_zh, cur.title_en);
                        let cur_desc = tr(cur.description_zh, cur.description_en);

                        render_field(ui, tr("场景编号", "Cue ID"), &format!("#{:02}", cur.id));
                        render_field(ui, tr("场景名称", "Title"), &cur_title);
                        render_field(ui, tr("MIDI 触发", "MIDI Trigger"), cur.midi_trigger);
                        render_field(ui, tr("淡入时间", "Fade Duration"), &format!("{:.1} s", cur.fade_time_s));

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(8.0);

                        ui.label(egui::RichText::new(tr("过渡进度", "Transition Curve")).size(11.0).color(Color32::from_rgb(130, 140, 155)));
                        ui.add_space(4.0);
                        ui.add(egui::ProgressBar::new(self.crossfade_progress).show_percentage());

                        ui.add_space(12.0);
                        ui.label(egui::RichText::new(tr("场景参数配置:", "Scene Details:")).size(11.0).color(Color32::from_rgb(130, 140, 155)));
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(cur_desc).size(10.5).color(Color32::from_rgb(190, 200, 215)));

                        ui.add_space(16.0);
                        if ui.button(egui::RichText::new(tr("保存为新快照", "Store Snapshot")).size(11.0)).clicked() {
                            // snapshot stored
                        }
                    });
            });
        });
    }

    fn set_active_cue(&mut self, idx: usize) {
        if idx < self.cues.len() {
            for c in &mut self.cues {
                c.active = false;
            }
            self.cues[idx].active = true;
            self.current_cue_idx = idx;
            self.crossfade_progress = 1.0;
        }
    }
}

fn render_field(ui: &mut Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}:", key)).size(11.0).color(Color32::from_rgb(120, 130, 145)));
        ui.label(egui::RichText::new(val).size(11.0).color(Color32::from_rgb(210, 220, 235)));
    });
    ui.add_space(3.0);
}
