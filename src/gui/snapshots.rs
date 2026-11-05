use egui::{Color32, Ui};

use crate::i18n::tr;

pub struct CueItem {
    #[allow(dead_code)]
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
        }
    }
}

impl SnapshotManager {
    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("💾 现场演出快照与 Cue 列表自动化", "💾 Snapshot Automation & Live Cue List"))
                        .color(Color32::from_rgb(0, 220, 255))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("▶ 下一场景 (GO)", "▶ GO (Next Cue)")).strong().color(Color32::from_rgb(0, 240, 160))).clicked() {
                        if self.current_cue_idx + 1 < self.cues.len() {
                            self.set_active_cue(self.current_cue_idx + 1);
                        }
                    }
                    if ui.button(tr("◀ 上一场景", "◀ Previous Cue")).clicked() {
                        if self.current_cue_idx > 0 {
                            self.set_active_cue(self.current_cue_idx - 1);
                        }
                    }
                });
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(tr(
                    "参考 LiveProfessor 的演出场景自动化设计。存储并在不同演出阶段即时召回完整的跳线路由、推子电平与效果器链，支持平滑过桥交叉淡化。",
                    "Live scene automation inspired by LiveProfessor. Store and recall complete matrix routings, fader balances, and plugin chains with automated crossfades."
                ))
                .color(Color32::from_rgb(160, 175, 195))
                .small(),
            );
            ui.add_space(12.0);

            // Cue Table
            egui::Frame::NONE
                .fill(Color32::from_rgb(14, 17, 22))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    for i in 0..self.cues.len() {
                        let is_current = i == self.current_cue_idx;
                        let cue = &self.cues[i];

                        let bg_color = if is_current {
                            Color32::from_rgb(26, 36, 48)
                        } else {
                            Color32::from_rgb(18, 22, 28)
                        };

                        egui::Frame::NONE
                            .fill(bg_color)
                            .corner_radius(4.0)
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Cue Indicator
                                    let status_text = if is_current { tr("● 当前激活", "● ACTIVE") } else { tr("○ 待命", "○ READY") };
                                    let status_color = if is_current {
                                        Color32::from_rgb(0, 240, 160)
                                    } else {
                                        Color32::GRAY
                                    };
                                    ui.label(egui::RichText::new(status_text).color(status_color).strong().size(11.0));

                                    ui.add_space(10.0);
                                    let title = tr(cue.title_zh, cue.title_en);
                                    ui.label(
                                        egui::RichText::new(title)
                                            .strong()
                                            .color(if is_current { Color32::WHITE } else { Color32::from_rgb(200, 210, 225) }),
                                    );

                                    ui.add_space(14.0);
                                    let desc = tr(cue.description_zh, cue.description_en);
                                    ui.label(
                                        egui::RichText::new(desc)
                                            .size(11.0)
                                            .color(Color32::from_rgb(150, 165, 185)),
                                    );

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if !is_current && ui.button(tr("调用场景", "Recall")).clicked() {
                                            self.current_cue_idx = i;
                                        }
                                        ui.label(
                                            egui::RichText::new(format!("{}: {:.1}s | MIDI: {}", tr("淡化", "Fade"), cue.fade_time_s, cue.midi_trigger))
                                                .size(10.0)
                                                .color(Color32::from_rgb(120, 135, 155)),
                                        );
                                    });
                                });
                            });

                        ui.add_space(4.0);
                    }
                });
        });
    }

    fn set_active_cue(&mut self, idx: usize) {
        self.current_cue_idx = idx;
        for (i, cue) in self.cues.iter_mut().enumerate() {
            cue.active = i == idx;
        }
    }
}
