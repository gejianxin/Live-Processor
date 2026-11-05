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
    "ASIO OUT 1-2\n(主监听音箱)",
    "ASIO OUT 3-4\n(监听耳机)",
    "VIRTUAL 1-2\n(OBS 直播流)",
    "VIRTUAL 3-4\n(DAW 回环录音)",
    "VIRTUAL 5-6\n(连麦麦克风)",
    "VIRTUAL 7-8\n(辅助发送 Aux)",
];

pub const DESTINATIONS_EN: &[&str] = &[
    "ASIO OUT 1-2\n(Monitors)",
    "ASIO OUT 3-4\n(Headphones)",
    "VIRTUAL 1-2\n(OBS Stream)",
    "VIRTUAL 3-4\n(DAW Loopback)",
    "VIRTUAL 5-6\n(Voice Mic)",
    "VIRTUAL 7-8\n(Aux Send)",
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
            signal_levels: vec![0.75, 0.42, 0.85, 0.58, 0.36, 0.18],
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
        let avail_h = ui.available_height().max(420.0);

        ui.vertical(|ui| {
            // Title & Global Preset Action Row
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("交叉跳线矩阵 (Wire Routing Matrix)", "Wire Routing Matrix"))
                        .color(Color32::from_rgb(235, 240, 248))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(tr("- 物理 ASIO 与虚拟 WDM 通道零延迟互联跳线盘", "- Zero-latency Physical & Virtual Audio Cross-point Patchbay"))
                        .color(Color32::from_rgb(120, 130, 145))
                        .small(),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("清空所有跳线", "Clear All")).size(11.0)).clicked() {
                        for row in &mut self.connections {
                            for cell in row {
                                *cell = false;
                            }
                        }
                    }
                    if ui.button(egui::RichText::new(tr("恢复默认预设", "Reset Default")).size(11.0)).clicked() {
                        *self = Self::default();
                    }
                });
            });

            ui.add_space(8.0);

            // Responsive Two-Column Layout
            // Left Panel (73% width): Visual Patchbay Matrix Table
            // Right Panel (27% width): Route Inspector & Preset Control Rack
            let inspector_w = 280.0;
            let matrix_w = (avail_w - inspector_w - 14.0).max(560.0);

            ui.horizontal(|ui| {
                // ==========================================
                // 1. LEFT PANEL: THE 2D PATCHBAY MATRIX GRID
                // ==========================================
                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 23, 29))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 56)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.set_width(matrix_w);
                        ui.set_min_height(avail_h);

                        ui.vertical(|ui| {
                            let label_col_w = 200.0;
                            let col_count = destinations.len() as f32;
                            let col_w = ((matrix_w - label_col_w - 28.0) / col_count).clamp(52.0, 120.0);

                            // Top Header Row: Column Destinations
                            ui.horizontal(|ui| {
                                // Corner Box: Input / Output indicator
                                egui::Frame::NONE
                                    .fill(Color32::from_rgb(26, 30, 38))
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(44, 52, 66)))
                                    .corner_radius(egui::CornerRadius::same(4))
                                    .inner_margin(egui::Margin::symmetric(6, 6))
                                    .show(ui, |ui| {
                                        ui.set_width(label_col_w);
                                        ui.set_height(44.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(egui::RichText::new(tr("输入来源 ↓ \\ 输出目标 →", "Inputs ↓ \\ Outputs →")).size(11.0).color(Color32::from_rgb(140, 152, 168)).strong());
                                        });
                                    });

                                ui.add_space(4.0);

                                // Destination Columns
                                for dest in destinations {
                                    egui::Frame::NONE
                                        .fill(Color32::from_rgb(26, 32, 42))
                                        .stroke(Stroke::new(1.0, Color32::from_rgb(46, 56, 72)))
                                        .corner_radius(egui::CornerRadius::same(4))
                                        .inner_margin(egui::Margin::symmetric(4, 4))
                                        .show(ui, |ui| {
                                            ui.set_width(col_w);
                                            ui.set_height(44.0);
                                            ui.vertical_centered(|ui| {
                                                for line in dest.lines() {
                                                    ui.label(
                                                        egui::RichText::new(line)
                                                            .size(10.0)
                                                            .color(Color32::from_rgb(220, 228, 238))
                                                            .strong(),
                                                    );
                                                }
                                            });
                                        });

                                    ui.add_space(4.0);
                                }
                            });

                            ui.add_space(6.0);

                            // Rows: Audio Sources
                            for (row_idx, source) in sources.iter().enumerate() {
                                let row_bg = if row_idx % 2 == 0 {
                                    Color32::from_rgb(23, 27, 34)
                                } else {
                                    Color32::from_rgb(18, 21, 27)
                                };

                                let level = (self.signal_levels[row_idx] + (time * 3.0 + (row_idx as f64) * 0.7).sin() as f32 * 0.14).clamp(0.04, 0.98);

                                egui::Frame::NONE
                                    .fill(row_bg)
                                    .stroke(Stroke::new(1.0, Color32::from_rgb(32, 38, 48)))
                                    .corner_radius(egui::CornerRadius::same(4))
                                    .inner_margin(egui::Margin::symmetric(6, 4))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            // 1. Source Channel Card (VU Meter + Name)
                                            ui.allocate_ui_with_layout(
                                                Vec2::new(label_col_w, 36.0),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    // 5-Segment LED Signal Bar
                                                    render_segment_meter(ui, level);
                                                    ui.add_space(6.0);

                                                    // Label
                                                    ui.label(
                                                        egui::RichText::new(*source)
                                                            .size(11.0)
                                                            .color(Color32::from_rgb(215, 222, 232))
                                                            .strong(),
                                                    );
                                                },
                                            );

                                            ui.add_space(4.0);

                                            // 2. Cross-point Sockets / Jacks
                                            for (col_idx, _) in destinations.iter().enumerate() {
                                                let is_connected = self.connections[row_idx][col_idx];
                                                let (rect, response) = ui.allocate_exact_size(Vec2::new(col_w, 36.0), egui::Sense::click());

                                                if response.clicked() {
                                                    self.connections[row_idx][col_idx] = !is_connected;
                                                }

                                                let center = rect.center();
                                                let is_hovered = response.hovered();

                                                // Draw Jack Cell Plate
                                                let cell_bg = if is_hovered {
                                                    Color32::from_rgb(34, 42, 54)
                                                } else if is_connected {
                                                    Color32::from_rgb(26, 33, 44)
                                                } else {
                                                    Color32::from_rgb(20, 24, 30)
                                                };

                                                ui.painter().rect_filled(rect.shrink(1.5), 3.0, cell_bg);
                                                ui.painter().rect_stroke(
                                                    rect.shrink(1.5),
                                                    3.0,
                                                    Stroke::new(1.0, if is_connected { Color32::from_rgb(46, 68, 92) } else { Color32::from_rgb(32, 38, 48) }),
                                                    egui::StrokeKind::Inside,
                                                );

                                                // Draw Physical Audio Jack / Plug
                                                if is_connected {
                                                    // CONNECTED STATE: Illuminated Blue Patch Cable Plug
                                                    let pulse = ((time * 4.0).sin() * 0.15 + 0.85) as f32;
                                                    let core_color = Color32::from_rgb(
                                                        (56.0 * pulse) as u8,
                                                        (189.0 * pulse) as u8,
                                                        (248.0 * pulse) as u8,
                                                    );

                                                    // Glow Halo
                                                    ui.painter().circle_filled(center, 9.0, Color32::from_rgba_premultiplied(56, 189, 248, 35));
                                                    // Metal Bezel Ring
                                                    ui.painter().circle_stroke(center, 7.5, Stroke::new(1.5, core_color));
                                                    // Center illuminated Pin
                                                    ui.painter().circle_filled(center, 4.5, core_color);
                                                    // Center White Reflection
                                                    ui.painter().circle_filled(center, 1.5, Color32::WHITE);
                                                } else {
                                                    // DISCONNECTED STATE: Dark Recessed Socket Port
                                                    let bezel_color = if is_hovered { Color32::from_rgb(90, 105, 128) } else { Color32::from_rgb(44, 52, 64) };
                                                    // Outer Socket Ring
                                                    ui.painter().circle_stroke(center, 6.5, Stroke::new(1.2, bezel_color));
                                                    // Inner Dark Socket Hole
                                                    ui.painter().circle_filled(center, 4.0, Color32::from_rgb(12, 14, 18));
                                                    // Center socket hole
                                                    ui.painter().circle_filled(center, 1.5, Color32::from_rgb(30, 36, 46));
                                                }

                                                ui.add_space(4.0);
                                            }
                                        });
                                    });

                                ui.add_space(4.0);
                            }

                            ui.add_space(10.0);

                            // Bottom Legend & Help Bar
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_rgb(56, 189, 248), "●");
                                ui.label(egui::RichText::new(tr("已连接音频通路", "Active Cable Patch")).size(10.5).color(Color32::from_rgb(170, 182, 198)));

                                ui.add_space(14.0);
                                ui.colored_label(Color32::from_rgb(60, 72, 88), "○");
                                ui.label(egui::RichText::new(tr("未连接空闲插口", "Unconnected Socket")).size(10.5).color(Color32::from_rgb(120, 130, 145)));

                                ui.add_space(14.0);
                                ui.label(
                                    egui::RichText::new(tr("提示: 点击任意交叉点可立即插入或拔出虚拟音频跳线", "Tip: Click any intersection node to connect or disconnect patch routing."))
                                        .size(10.5)
                                        .color(Color32::from_rgb(100, 110, 125)),
                                );
                            });
                        });
                    });

                ui.add_space(10.0);

                // ==========================================
                // 2. RIGHT PANEL: ROUTING INSPECTOR & PRESETS
                // ==========================================
                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 23, 29))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 56)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.set_width(inspector_w);
                        ui.set_min_height(avail_h);

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(tr("🎛️ 路由状态与控制台", "Routing Inspector & Controls")).strong().size(13.0).color(Color32::from_rgb(230, 236, 246)));
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(8.0);

                            // Active Route Counter Card
                            let mut active_nodes = 0;
                            for r in &self.connections {
                                for c in r {
                                    if *c { active_nodes += 1; }
                                }
                            }

                            egui::Frame::NONE
                                .fill(Color32::from_rgb(25, 30, 38))
                                .corner_radius(egui::CornerRadius::same(4))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(42, 50, 64)))
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{}", active_nodes)).strong().size(22.0).color(Color32::from_rgb(56, 189, 248)));
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(tr("活跃跳线通路", "Active Connections")).strong().size(11.0).color(Color32::from_rgb(220, 228, 238)));
                                            ui.label(egui::RichText::new(format!("{} / 36 {}", tr("共", "Total"), tr("最大矩阵节点", "matrix nodes"))).size(10.0).color(Color32::from_rgb(110, 120, 135)));
                                        });
                                    });
                                });

                            ui.add_space(10.0);
                            render_info_row(ui, tr("矩阵吞吐状态", "Throughput"), tr("零延迟直通 (0.0 ms)", "Zero Latency (0.0 ms)"));
                            render_info_row(ui, tr("核心路由服务", "Router Engine"), tr("已激活 (Active)", "Online"));
                            render_info_row(ui, tr("通道隔离保护", "Isolation"), tr("全通道立体声", "Discrete Stereo"));

                            ui.add_space(14.0);
                            ui.label(egui::RichText::new(tr("一键跳线应用场景预设", "One-Click Presets")).strong().size(11.5).color(Color32::from_rgb(180, 192, 210)));
                            ui.add_space(6.0);

                            // Preset Buttons
                            render_preset_card(
                                ui,
                                tr("🎙️ 网络直播标准模式", "🎙️ Live Stream Standard"),
                                tr("人声+DAW 混音送往 OBS 直播流与主监听", "Mix Mic & DAW to OBS & Monitors"),
                                || self.apply_broadcast_preset(),
                            );
                            ui.add_space(4.0);

                            render_preset_card(
                                ui,
                                tr("🎧 专业录音监听模式", "🎧 Studio Tracking Studio"),
                                tr("人声与乐器直接送至监听耳机与宿主", "Direct Mic & Inst to Headphones"),
                                || self.apply_studio_preset(),
                            );
                            ui.add_space(4.0);

                            render_preset_card(
                                ui,
                                tr("🎮 连麦游戏通话模式", "🎮 Discord / Gaming Voice"),
                                tr("游戏背景音送耳机，语音通道独立分流", "Separate Game Audio & Discord Mic"),
                                || self.apply_gaming_preset(),
                            );

                            ui.add_space(14.0);
                            ui.separator();
                            ui.add_space(10.0);

                            ui.checkbox(&mut self.direct_monitoring, tr("启用硬件零延迟监听 (Direct ASIO)", "Direct ASIO Monitoring"));
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(tr("绕开 Windows WDM 音频混音器，以声卡原生极低延迟实现输入直通。", "Bypass OS mixer for true hardware zero-latency monitoring."))
                                    .size(10.0)
                                    .color(Color32::from_rgb(110, 120, 135)),
                            );
                        });
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
        self.connections[0][4] = true; // Mic -> Voice Mic
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

fn render_segment_meter(ui: &mut Ui, level: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(6.0, 26.0), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(rect, 1.0, Color32::from_rgb(16, 18, 22));

    let segments = 5;
    let seg_gap = 1.5;
    let seg_h = (rect.height() - (segments as f32 - 1.0) * seg_gap) / (segments as f32);

    for s in 0..segments {
        let seg_idx_from_bottom = segments - 1 - s;
        let seg_thresh = (seg_idx_from_bottom as f32 + 0.5) / (segments as f32);
        let is_lit = level >= seg_thresh;

        let y_top = rect.min.y + (s as f32) * (seg_h + seg_gap);
        let seg_rect = Rect::from_min_size(Pos2::new(rect.min.x, y_top), Vec2::new(rect.width(), seg_h));

        let color = if is_lit {
            if s == 0 {
                Color32::from_rgb(248, 113, 113) // Red peak
            } else if s == 1 {
                Color32::from_rgb(245, 158, 11) // Amber
            } else {
                Color32::from_rgb(52, 211, 153) // Green
            }
        } else {
            Color32::from_rgb(28, 34, 42) // Unlit
        };

        p.rect_filled(seg_rect, 1.0, color);
    }
}

fn render_preset_card(ui: &mut Ui, title: &str, subtitle: &str, on_click: impl FnOnce()) {
    let btn_res = ui.add(
        egui::Button::new(
            egui::RichText::new(title)
                .strong()
                .size(11.0)
                .color(Color32::from_rgb(220, 230, 242)),
        )
        .fill(Color32::from_rgb(26, 31, 40))
        .stroke(Stroke::new(1.0, Color32::from_rgb(42, 50, 64)))
        .corner_radius(egui::CornerRadius::same(4)),
    );

    if btn_res.clicked() {
        on_click();
    }

    ui.label(egui::RichText::new(subtitle).size(9.5).color(Color32::from_rgb(115, 125, 140)));
}

fn render_info_row(ui: &mut Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}:", key)).size(10.5).color(Color32::from_rgb(120, 130, 145)));
        ui.label(egui::RichText::new(val).size(10.5).color(Color32::from_rgb(210, 220, 235)));
    });
    ui.add_space(2.0);
}
