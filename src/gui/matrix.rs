use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

use crate::i18n::tr;

pub const SOURCES_ZH: &[&str] = &[
    "ASIO IN 1-2 (麦克风/人声)",
    "ASIO IN 3-4 (乐器/硬件输入)",
    "VIRTUAL 1-2 (DAW 宿主总输出)",
    "VIRTUAL 3-4 (音乐播放器/媒体)",
    "VIRTUAL 5-6 (语音通话/连麦)",
    "VIRTUAL 7-8 (系统辅助/Aux)",
];

pub const SOURCES_EN: &[&str] = &[
    "ASIO IN 1-2 (Mic / Pre)",
    "ASIO IN 3-4 (Line / Synth)",
    "VIRTUAL 1-2 (DAW Master)",
    "VIRTUAL 3-4 (Music / Media)",
    "VIRTUAL 5-6 (Voice / Chat)",
    "VIRTUAL 7-8 (System / Aux)",
];

pub const DESTINATIONS_ZH: &[&str] = &[
    "ASIO OUT 1-2 (主监听音箱)",
    "ASIO OUT 3-4 (监听耳机)",
    "VIRTUAL 1-2 (OBS 直播流)",
    "VIRTUAL 3-4 (DAW 回环录音)",
    "VIRTUAL 5-6 (连麦麦克风)",
    "VIRTUAL 7-8 (辅助发送 Aux)",
];

pub const DESTINATIONS_EN: &[&str] = &[
    "ASIO OUT 1-2 (Monitors)",
    "ASIO OUT 3-4 (Headphones)",
    "VIRTUAL 1-2 (OBS Stream)",
    "VIRTUAL 3-4 (DAW Loopback)",
    "VIRTUAL 5-6 (Voice Mic)",
    "VIRTUAL 7-8 (Aux Send)",
];

pub struct RoutingMatrix {
    pub connections: Vec<Vec<bool>>,
    pub signal_levels: Vec<f32>,
    pub direct_monitoring: bool,
}

impl Default for RoutingMatrix {
    fn default() -> Self {
        let mut connections = vec![vec![false; DESTINATIONS_EN.len()]; SOURCES_EN.len()];

        // Default clean broadcast routing
        connections[0][0] = true; // Mic -> Monitors
        connections[0][2] = true; // Mic -> OBS
        connections[0][4] = true; // Mic -> Voice Mic
        connections[2][0] = true; // DAW -> Monitors
        connections[2][1] = true; // DAW -> Headphones
        connections[2][2] = true; // DAW -> OBS
        connections[3][0] = true; // BGM -> Monitors
        connections[3][2] = true; // BGM -> OBS
        connections[4][0] = true; // Chat -> Monitors
        connections[4][1] = true; // Chat -> Headphones

        Self {
            connections,
            signal_levels: vec![0.72, 0.45, 0.82, 0.58, 0.38, 0.20],
            direct_monitoring: true,
        }
    }
}

impl RoutingMatrix {
    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        let is_zh = crate::i18n::Language::current() == crate::i18n::Language::Chinese;
        let sources = if is_zh { SOURCES_ZH } else { SOURCES_EN };
        let destinations = if is_zh { DESTINATIONS_ZH } else { DESTINATIONS_EN };

        let avail_w = ui.available_width();
        let avail_h = (ui.available_height() - 10.0).max(400.0);

        ui.vertical(|ui| {
            // Header Row
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("交叉跳线矩阵", "Wire Routing Matrix"))
                        .color(Color32::from_rgb(230, 236, 244))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(tr("(物理 ASIO 与虚拟 WDM 通道零延迟互联)", "(Zero-latency ASIO & WDM Patchbay)"))
                        .color(Color32::from_rgb(120, 130, 145))
                        .small(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("清空跳线", "Clear All")).size(11.0)).clicked() {
                        for row in &mut self.connections {
                            for cell in row {
                                *cell = false;
                            }
                        }
                    }
                    if ui.button(egui::RichText::new(tr("重置默认", "Reset")).size(11.0)).clicked() {
                        *self = Self::default();
                    }
                });
            });

            ui.add_space(8.0);

            // Responsive Two-Column Layout (Left: Matrix Grid, Right: Inspector & Presets)
            let inspector_w = 260.0;
            let matrix_w = (avail_w - inspector_w - 16.0).max(520.0);

            ui.horizontal(|ui| {
                // Left Panel: Full Adaptive Matrix Grid
                egui::Frame::NONE
                    .fill(Color32::from_rgb(22, 25, 31))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 54)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(matrix_w);
                        ui.set_min_height(avail_h);

                        let label_width = 200.0;
                        let col_count = destinations.len() as f32;
                        let col_width = ((matrix_w - label_width - 32.0) / col_count).clamp(50.0, 95.0);

                        // Destination Headers
                        ui.horizontal(|ui| {
                            ui.allocate_exact_size(Vec2::new(label_width, 42.0), egui::Sense::hover());
                            for dest in destinations {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(col_width, 42.0),
                                    egui::Layout::centered_and_justified(egui::Direction::TopDown),
                                    |ui| {
                                        let short_name = simplify_dest_name(dest);
                                        egui::Frame::NONE
                                            .fill(Color32::from_rgb(28, 33, 42))
                                            .corner_radius(egui::CornerRadius::same(3))
                                            .inner_margin(egui::Margin::symmetric(4, 4))
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(short_name)
                                                        .size(10.5)
                                                        .color(Color32::from_rgb(180, 195, 215))
                                                        .strong(),
                                                );
                                            });
                                    },
                                );
                            }
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Source Rows
                        for (row_idx, source) in sources.iter().enumerate() {
                            let level = (self.signal_levels[row_idx] + (time * 2.5 + (row_idx as f64)).sin() as f32 * 0.12).clamp(0.05, 0.95);

                            ui.horizontal(|ui| {
                                // Mini signal meter (subtle emerald bar)
                                let (meter_rect, _) = ui.allocate_exact_size(Vec2::new(4.0, 32.0), egui::Sense::hover());
                                ui.painter().rect_filled(meter_rect, 1.0, Color32::from_rgb(32, 38, 48));
                                let filled_h = meter_rect.height() * level;
                                let active_rect = Rect::from_min_max(
                                    Pos2::new(meter_rect.min.x, meter_rect.max.y - filled_h),
                                    meter_rect.max,
                                );
                                let bar_color = if level > 0.88 {
                                    Color32::from_rgb(248, 113, 113)
                                } else {
                                    Color32::from_rgb(52, 211, 153)
                                };
                                ui.painter().rect_filled(active_rect, 1.0, bar_color);

                                ui.add_space(6.0);

                                // Source channel label
                                ui.add_sized(
                                    [label_width - 16.0, 32.0],
                                    egui::Label::new(
                                        egui::RichText::new(*source)
                                            .size(11.5)
                                            .color(Color32::from_rgb(215, 222, 232)),
                                    ),
                                );

                                // Cross-points
                                for (col_idx, _) in destinations.iter().enumerate() {
                                    let is_connected = self.connections[row_idx][col_idx];
                                    let (rect, response) = ui.allocate_exact_size(Vec2::new(col_width, 32.0), egui::Sense::click());

                                    if response.clicked() {
                                        self.connections[row_idx][col_idx] = !is_connected;
                                    }

                                    let center = rect.center();
                                    let cell_bg = if response.hovered() {
                                        Color32::from_rgb(34, 40, 52)
                                    } else {
                                        Color32::from_rgb(24, 28, 35)
                                    };

                                    ui.painter().rect_filled(rect.shrink(2.0), 3.0, cell_bg);

                                    // Round connection pin
                                    let pin_radius = 5.0;
                                    if is_connected {
                                        // Active Studio Sky Pin
                                        ui.painter().circle_filled(center, pin_radius + 2.0, Color32::from_rgba_premultiplied(56, 189, 248, 40));
                                        ui.painter().circle_filled(center, pin_radius, Color32::from_rgb(56, 189, 248));
                                    } else {
                                        // Unconnected muted pin
                                        ui.painter().circle_filled(center, 2.5, Color32::from_rgb(46, 54, 66));
                                    }
                                }
                            });

                            ui.add_space(4.0);
                        }
                    });

                ui.add_space(12.0);

                // Right Panel: Routing Inspector & Presets
                egui::Frame::NONE
                    .fill(Color32::from_rgb(22, 25, 31))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 54)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(inspector_w);
                        ui.set_min_height(avail_h);

                        ui.label(egui::RichText::new(tr("路由概览与预设", "Inspector & Presets")).strong().size(13.0).color(Color32::from_rgb(220, 228, 238)));
                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Stats
                        let mut active_nodes = 0;
                        for r in &self.connections {
                            for c in r {
                                if *c { active_nodes += 1; }
                            }
                        }

                        render_inspector_row(ui, tr("活跃跳线节点", "Active Patchpoints"), &format!("{} / 36", active_nodes));
                        render_inspector_row(ui, tr("内建延迟补偿", "Latency Offset"), "0.0 ms (Direct)");
                        render_inspector_row(ui, tr("信号状态", "Routing Engine"), tr("就绪 [正常]", "Online"));

                        ui.add_space(14.0);
                        ui.label(egui::RichText::new(tr("一键跳线预设", "Routing Presets")).strong().size(11.5).color(Color32::from_rgb(180, 192, 210)));
                        ui.add_space(6.0);

                        if ui.button(egui::RichText::new(tr("🎙️ 网络直播标准模式", "🎙️ Live Stream Standard")).size(11.0)).clicked() {
                            self.apply_broadcast_preset();
                        }
                        ui.add_space(4.0);

                        if ui.button(egui::RichText::new(tr("🎧 DAW 纯录音监听模式", "🎧 DAW Tracking Studio")).size(11.0)).clicked() {
                            self.apply_studio_preset();
                        }
                        ui.add_space(4.0);

                        if ui.button(egui::RichText::new(tr("🎮 连麦游戏连线模式", "🎮 Discord / Gaming")).size(11.0)).clicked() {
                            self.apply_gaming_preset();
                        }

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.checkbox(&mut self.direct_monitoring, tr("启用硬件零延迟直通", "Direct ASIO Monitoring"));
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(tr("绕过 Windows 音频引擎，以极致低延迟直通监听。", "Bypass Windows audio engine for lowest latency."))
                                .size(10.0)
                                .color(Color32::from_rgb(110, 120, 135)),
                        );
                    });
            });
        });
    }

    pub fn apply_broadcast_preset(&mut self) {
        for row in &mut self.connections {
            for cell in row {
                *cell = false;
            }
        }
        self.connections[0][0] = true; // Mic -> Monitors
        self.connections[0][2] = true; // Mic -> OBS
        self.connections[0][4] = true; // Mic -> Chat Mic
        self.connections[2][0] = true; // DAW -> Monitors
        self.connections[2][2] = true; // DAW -> OBS
        self.connections[3][0] = true; // BGM -> Monitors
        self.connections[3][2] = true; // BGM -> OBS
        self.connections[4][0] = true; // Chat -> Monitors
    }

    pub fn apply_studio_preset(&mut self) {
        for row in &mut self.connections {
            for cell in row {
                *cell = false;
            }
        }
        self.connections[0][0] = true; // Mic -> Monitors
        self.connections[0][1] = true; // Mic -> Headphones
        self.connections[1][0] = true; // Inst -> Monitors
        self.connections[1][1] = true; // Inst -> Headphones
        self.connections[2][0] = true; // DAW -> Monitors
        self.connections[2][1] = true; // DAW -> Headphones
    }

    pub fn apply_gaming_preset(&mut self) {
        for row in &mut self.connections {
            for cell in row {
                *cell = false;
            }
        }
        self.connections[0][4] = true; // Mic -> Discord
        self.connections[3][1] = true; // Music -> Headphones
        self.connections[4][1] = true; // Discord -> Headphones
    }
}

fn simplify_dest_name(full: &str) -> String {
    if let Some(pos) = full.find('(') {
        let name = &full[pos + 1..];
        name.trim_end_matches(')').to_string()
    } else {
        full.to_string()
    }
}

fn render_inspector_row(ui: &mut Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}:", key)).size(10.5).color(Color32::from_rgb(120, 130, 145)));
        ui.label(egui::RichText::new(val).size(10.5).color(Color32::from_rgb(205, 215, 228)));
    });
    ui.add_space(2.0);
}
