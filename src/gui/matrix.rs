use egui::{Color32, Pos2, Rect, Stroke, Ui, Vec2};

use crate::i18n::tr;

pub const SOURCES_ZH: &[&str] = &[
    "ASIO IN 1-2 (麦克风/人声)",
    "ASIO IN 3-4 (乐器/合成器)",
    "VIRTUAL 1-2 (宿主 DAW 总输出)",
    "VIRTUAL 3-4 (音乐/媒体播放)",
    "VIRTUAL 5-6 (语音通话/连麦)",
    "VIRTUAL 7-8 (系统辅助/Aux)",
];

pub const SOURCES_EN: &[&str] = &[
    "ASIO IN 1-2 (Mic/Pre)",
    "ASIO IN 3-4 (Line/Synth)",
    "VIRTUAL 1-2 (DAW Master)",
    "VIRTUAL 3-4 (BGM/Media)",
    "VIRTUAL 5-6 (Chat/Discord)",
    "VIRTUAL 7-8 (System/Aux)",
];

pub const DESTINATIONS_ZH: &[&str] = &[
    "ASIO OUT 1-2 (主监听音箱)",
    "ASIO OUT 3-4 (监听耳机)",
    "VIRTUAL OUT 1-2 (OBS 直播流)",
    "VIRTUAL OUT 3-4 (DAW 回环录音)",
    "VIRTUAL OUT 5-6 (语音麦克风输出)",
    "VIRTUAL OUT 7-8 (辅助发送 Aux)",
];

pub const DESTINATIONS_EN: &[&str] = &[
    "ASIO OUT 1-2 (Monitors)",
    "ASIO OUT 3-4 (Headphones)",
    "VIRTUAL OUT 1-2 (OBS Stream)",
    "VIRTUAL OUT 3-4 (DAW Loopback)",
    "VIRTUAL OUT 5-6 (Voice Mic)",
    "VIRTUAL OUT 7-8 (Aux Send)",
];

pub struct RoutingMatrix {
    pub connections: Vec<Vec<bool>>,
    pub signal_levels: Vec<f32>,
}

impl Default for RoutingMatrix {
    fn default() -> Self {
        let mut connections = vec![vec![false; DESTINATIONS_EN.len()]; SOURCES_EN.len()];

        // Default routing preset
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
            signal_levels: vec![0.75, 0.45, 0.88, 0.60, 0.35, 0.15],
        }
    }
}

impl RoutingMatrix {
    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        let is_zh = crate::i18n::Language::current() == crate::i18n::Language::Chinese;
        let sources = if is_zh { SOURCES_ZH } else { SOURCES_EN };
        let destinations = if is_zh { DESTINATIONS_ZH } else { DESTINATIONS_EN };

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("🎛️ 跳线矩阵（虚拟跳线盘与音频路由矩阵）", "🎛️ Wire Matrix (Virtual Patchbay & Audio Matrix)"))
                        .color(Color32::from_rgb(0, 220, 255))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(tr("清空所有", "Clear All")).clicked() {
                        for row in &mut self.connections {
                            for cell in row {
                                *cell = false;
                            }
                        }
                    }
                    if ui.button(tr("恢复默认", "Reset Default")).clicked() {
                        *self = Self::default();
                    }
                    if ui.button(tr("一键直播广播预设", "Broadcast Preset")).clicked() {
                        self.apply_broadcast_preset();
                    }
                });
            });

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(tr(
                    "交互式交叉跳线矩阵。点击任意交叉点以实现物理 ASIO 与虚拟 WDM 通道的零延迟互联，实时路由任意宿主与播放器音频。",
                    "Interactive cross-point matrix. Click any intersection node to route physical ASIO and virtual WDM channels with zero latency."
                ))
                .color(Color32::from_rgb(160, 175, 195))
                .small(),
            );
            ui.add_space(10.0);

            // Matrix Grid
            egui::Frame::NONE
                .fill(Color32::from_rgb(14, 17, 22))
                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 56)))
                .inner_margin(16.0)
                .show(ui, |ui| {
                    let cell_size = 32.0;
                    let label_width = 220.0;
                    let header_height = 80.0;

                    // Header row: Destinations
                    let (header_rect, _) = ui.allocate_exact_size(
                        Vec2::new(label_width + (destinations.len() as f32) * cell_size + 40.0, header_height),
                        egui::Sense::hover(),
                    );

                    let painter = ui.painter();
                    for (col_idx, dest) in destinations.iter().enumerate() {
                        let x = header_rect.min.x + label_width + (col_idx as f32) * cell_size + cell_size * 0.5;
                        let y = header_rect.max.y - 10.0;

                        painter.text(
                            Pos2::new(x, y),
                            egui::Align2::RIGHT_CENTER,
                            *dest,
                            egui::FontId::proportional(11.0),
                            Color32::from_rgb(0, 210, 255),
                        );
                    }

                    // Rows: Sources
                    for (row_idx, source) in sources.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Signal level indicator
                            let level = (self.signal_levels[row_idx] + (time * 3.0 + (row_idx as f64)).sin() as f32 * 0.15).clamp(0.05, 1.0);

                            // Mini VU meter bar
                            let (meter_rect, _) = ui.allocate_exact_size(Vec2::new(8.0, 24.0), egui::Sense::hover());
                            let meter_fill = if level > 0.85 {
                                Color32::from_rgb(255, 60, 60)
                            } else if level > 0.65 {
                                Color32::from_rgb(255, 200, 40)
                            } else {
                                Color32::from_rgb(0, 230, 120)
                            };
                            ui.painter().rect_filled(meter_rect, 2.0, Color32::from_rgb(25, 30, 38));
                            let filled_h = meter_rect.height() * level;
                            let active_rect = Rect::from_min_max(
                                Pos2::new(meter_rect.min.x, meter_rect.max.y - filled_h),
                                meter_rect.max,
                            );
                            ui.painter().rect_filled(active_rect, 2.0, meter_fill);

                            // Source Label
                            ui.add_sized(
                                [label_width - 24.0, 26.0],
                                egui::Label::new(
                                    egui::RichText::new(*source)
                                        .color(Color32::from_rgb(220, 228, 240))
                                        .strong(),
                                ),
                            );

                            // Cross points
                            for (col_idx, _) in destinations.iter().enumerate() {
                                let is_connected = self.connections[row_idx][col_idx];
                                let (rect, response) = ui.allocate_exact_size(Vec2::splat(cell_size), egui::Sense::click());

                                if response.clicked() {
                                    self.connections[row_idx][col_idx] = !is_connected;
                                }

                                let center = rect.center();
                                let bg_color = if response.hovered() {
                                    Color32::from_rgb(32, 40, 52)
                                } else {
                                    Color32::from_rgb(20, 24, 32)
                                };

                                ui.painter().rect_filled(rect.shrink(2.0), 4.0, bg_color);
                                ui.painter().rect_stroke(
                                    rect.shrink(2.0),
                                    4.0,
                                    Stroke::new(1.0, Color32::from_rgb(45, 52, 66)),
                                    egui::StrokeKind::Inside,
                                );

                                if is_connected {
                                    let pulse = ((time * 4.0).sin() * 0.15 + 0.85) as f32;
                                    let dot_color = if col_idx < 2 {
                                        Color32::from_rgb(
                                            (0.0 * pulse) as u8,
                                            (229.0 * pulse) as u8,
                                            (255.0 * pulse) as u8,
                                        )
                                    } else {
                                        Color32::from_rgb(
                                            (255.0 * pulse) as u8,
                                            (170.0 * pulse) as u8,
                                            (0.0 * pulse) as u8,
                                        )
                                    };

                                    ui.painter().circle_filled(center, 6.0, dot_color);
                                    ui.painter().circle_stroke(center, 9.0, Stroke::new(1.2, dot_color));
                                } else if response.hovered() {
                                    ui.painter().circle_stroke(
                                        center,
                                        4.0,
                                        Stroke::new(1.0, Color32::from_rgb(100, 115, 135)),
                                    );
                                }
                            }
                        });
                    }
                });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr("● 青色节点: 物理 ASIO 硬件输出端口", "● Cyan Nodes: Physical ASIO Hardware Out")).color(Color32::from_rgb(0, 220, 255)).small());
                ui.label(egui::RichText::new(tr("● 橙色节点: 系统虚拟 WDM 播放/录制端点", "● Amber Nodes: Virtual WDM Endpoints")).color(Color32::from_rgb(255, 170, 0)).small());
            });
        });
    }

    fn apply_broadcast_preset(&mut self) {
        for row in &mut self.connections {
            for cell in row {
                *cell = false;
            }
        }
        // Mic -> Stream + Discord
        self.connections[0][2] = true;
        self.connections[0][4] = true;
        // DAW -> Monitors + Stream
        self.connections[2][0] = true;
        self.connections[2][2] = true;
        // BGM -> Stream + Monitors
        self.connections[3][0] = true;
        self.connections[3][2] = true;
        // Discord -> Headphones only
        self.connections[4][1] = true;
    }
}
