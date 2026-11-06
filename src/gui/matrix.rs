use std::sync::atomic::Ordering;
use std::sync::Arc;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Ui, Vec2};

use crate::audio::AudioSharedState;
use crate::i18n::tr;

pub const SOURCES_ZH: &[(&str, &str)] = &[
    ("ASIO IN 1-2", "麦克风 / 人声"),
    ("ASIO IN 3-4", "乐器 / 硬件输入"),
    ("VIRTUAL 1-2", "DAW 宿主总输出"),
    ("VIRTUAL 3-4", "音乐播放器 / 媒体"),
    ("VIRTUAL 5-6", "语音通话 / 连麦"),
    ("VIRTUAL 7-8", "系统辅助 / Aux"),
];

pub const SOURCES_EN: &[(&str, &str)] = &[
    ("ASIO IN 1-2", "Mic / Preamp"),
    ("ASIO IN 3-4", "Line / Synth"),
    ("VIRTUAL 1-2", "DAW Master"),
    ("VIRTUAL 3-4", "Music / Media"),
    ("VIRTUAL 5-6", "Voice / Discord"),
    ("VIRTUAL 7-8", "System / Aux"),
];

pub const DESTINATIONS_ZH: &[(&str, &str)] = &[
    ("ASIO OUT 1-2", "主监听音箱"),
    ("ASIO OUT 3-4", "监听耳机"),
    ("VIRTUAL 1-2",  "OBS 直播流"),
    ("VIRTUAL 3-4",  "DAW 回环录音"),
    ("VIRTUAL 5-6",  "连麦麦克风"),
    ("VIRTUAL 7-8",  "辅助发送 Aux"),
];

pub const DESTINATIONS_EN: &[(&str, &str)] = &[
    ("ASIO OUT 1-2", "Main Monitors"),
    ("ASIO OUT 3-4", "Headphones"),
    ("VIRTUAL 1-2",  "OBS Stream"),
    ("VIRTUAL 3-4",  "DAW Loopback"),
    ("VIRTUAL 5-6",  "Voice Mic"),
    ("VIRTUAL 7-8",  "Aux Send"),
];

pub struct RoutingMatrix {
    pub connections: Vec<Vec<bool>>,
    pub audio_state: Option<Arc<AudioSharedState>>,
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
            audio_state: None,
            direct_monitoring: true,
        }
    }
}

impl RoutingMatrix {
    pub fn new(audio_state: Arc<AudioSharedState>) -> Self {
        let mut s = Self::default();
        s.audio_state = Some(audio_state);
        s.sync_to_audio_engine();
        s
    }

    pub fn sync_to_audio_engine(&self) {
        if let Some(state) = &self.audio_state {
            for r in 0..SOURCES_EN.len() {
                for c in 0..DESTINATIONS_EN.len() {
                    state.matrix_connections[r][c].store(self.connections[r][c], Ordering::Relaxed);
                }
            }
            state.direct_asio_monitoring.store(self.direct_monitoring, Ordering::Relaxed);
        }
    }

    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        let is_zh = crate::i18n::Language::current() == crate::i18n::Language::Chinese;
        let sources = if is_zh { SOURCES_ZH } else { SOURCES_EN };
        let destinations = if is_zh { DESTINATIONS_ZH } else { DESTINATIONS_EN };

        let total_avail_w = ui.available_width();

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
                    ui.add_space(4.0);
                    if ui.button(egui::RichText::new(tr("清空所有跳线", "Clear All")).size(11.0)).clicked() {
                        for row in &mut self.connections {
                            for cell in row {
                                *cell = false;
                            }
                        }
                        self.sync_to_audio_engine();
                    }
                    if ui.button(egui::RichText::new(tr("恢复默认预设", "Reset Default")).size(11.0)).clicked() {
                        let audio_state = self.audio_state.clone();
                        *self = Self::default();
                        self.audio_state = audio_state;
                        self.sync_to_audio_engine();
                    }
                });
            });

            ui.add_space(8.0);

            // Responsive Two-Column Layout:
            // Left Panel: Visual Patchbay Matrix Grid (Flexible, shrinks/grows gracefully)
            // Right Panel: Route Inspector & Preset Control Rack (Fixed 230px width)
            let inspector_w = 230.0;
            let gap_between_panels = 10.0;
            let matrix_w = (total_avail_w - inspector_w - gap_between_panels).max(480.0);

            ui.horizontal(|ui| {
                // ==========================================
                // 1. LEFT PANEL: THE 2D PATCHBAY MATRIX GRID
                // ==========================================
                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 23, 29))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 56)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(matrix_w - 20.0);

                        ui.vertical(|ui| {
                            // Column geometry:
                            // Fixed label column width guarantees ZERO misalignment across rows
                            let label_col_w = 160.0;
                            let col_count = destinations.len() as f32;
                            let col_gap = 4.0;
                            let total_gaps = col_gap * col_count;
                            let cell_w = ((matrix_w - label_col_w - 24.0 - total_gaps) / col_count).max(46.0);
                            let cell_h = 40.0;
                            let header_h = 44.0;

                            // ------------------------------------
                            // Header Row: Output Destination Cards
                            // ------------------------------------
                            ui.horizontal(|ui| {
                                // Corner Box: Input \ Output Legend (Centered text)
                                let (corner_rect, _) = ui.allocate_exact_size(Vec2::new(label_col_w, header_h), egui::Sense::hover());
                                let p = ui.painter();
                                p.rect_filled(corner_rect, 4.0, Color32::from_rgb(26, 30, 38));
                                p.rect_stroke(corner_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(44, 52, 66)), egui::StrokeKind::Inside);
                                p.text(
                                    corner_rect.center(),
                                    Align2::CENTER_CENTER,
                                    tr("输入来源 ↓  \\  输出目标 →", "Inputs ↓  \\  Outputs →"),
                                    FontId::proportional(10.5),
                                    Color32::from_rgb(150, 162, 178),
                                );

                                ui.add_space(col_gap);

                                // Destination Header Cards
                                for (tag, name) in destinations {
                                    let (hdr_rect, _) = ui.allocate_exact_size(Vec2::new(cell_w, header_h), egui::Sense::hover());
                                    let p = ui.painter();
                                    p.rect_filled(hdr_rect, 4.0, Color32::from_rgb(26, 32, 42));
                                    p.rect_stroke(hdr_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(46, 56, 72)), egui::StrokeKind::Inside);

                                    // Line 1: Port Tag (Top, Cyan, bold)
                                    p.text(
                                        Pos2::new(hdr_rect.center().x, hdr_rect.min.y + 13.0),
                                        Align2::CENTER_CENTER,
                                        *tag,
                                        FontId::proportional(10.0),
                                        Color32::from_rgb(56, 189, 248),
                                    );

                                    // Line 2: Port Name (Bottom, White/Silver, bold)
                                    p.text(
                                        Pos2::new(hdr_rect.center().x, hdr_rect.min.y + 30.0),
                                        Align2::CENTER_CENTER,
                                        *name,
                                        FontId::proportional(10.5),
                                        Color32::from_rgb(225, 235, 246),
                                    );

                                    ui.add_space(col_gap);
                                }
                            });

                            ui.add_space(5.0);

                            // ------------------------------------
                            // Data Rows: Audio Sources & Sockets
                            // ------------------------------------
                            for (row_idx, (tag, name)) in sources.iter().enumerate() {
                                let row_bg = if row_idx % 2 == 0 {
                                    Color32::from_rgb(23, 27, 34)
                                } else {
                                    Color32::from_rgb(18, 21, 27)
                                };

                                let level = if let Some(state) = &self.audio_state {
                                    f32::from_bits(state.input_peaks[row_idx].load(Ordering::Relaxed))
                                } else {
                                    (0.5 + (time * 3.0 + (row_idx as f64) * 0.7).sin() as f32 * 0.14).clamp(0.04, 0.98)
                                };

                                ui.horizontal(|ui| {
                                    // 1. Source Channel Card
                                    let (src_rect, _) = ui.allocate_exact_size(Vec2::new(label_col_w, cell_h), egui::Sense::hover());
                                    let p = ui.painter();
                                    p.rect_filled(src_rect, 4.0, row_bg);
                                    p.rect_stroke(src_rect, 4.0, Stroke::new(1.0, Color32::from_rgb(34, 40, 52)), egui::StrokeKind::Inside);

                                    // 5-Segment LED Signal Bar on left
                                    let meter_rect = Rect::from_min_size(
                                        Pos2::new(src_rect.min.x + 8.0, src_rect.min.y + 7.0),
                                        Vec2::new(6.0, 26.0),
                                    );
                                    render_segment_meter(p, meter_rect, level);

                                    // Text: Tag + Name
                                    let full_title = format!("{} {}", tag, name);
                                    p.text(
                                        Pos2::new(src_rect.min.x + 18.0, src_rect.center().y),
                                        Align2::LEFT_CENTER,
                                        full_title,
                                        FontId::proportional(10.5),
                                        Color32::from_rgb(220, 228, 238),
                                    );

                                    ui.add_space(col_gap);

                                    // 2. Cross-point Sockets / Jacks
                                    for (col_idx, _) in destinations.iter().enumerate() {
                                        let is_connected = self.connections[row_idx][col_idx];
                                        let (rect, response) = ui.allocate_exact_size(Vec2::new(cell_w, cell_h), egui::Sense::click());

                                        if response.clicked() {
                                            self.connections[row_idx][col_idx] = !is_connected;
                                            if let Some(state) = &self.audio_state {
                                                state.matrix_connections[row_idx][col_idx].store(!is_connected, Ordering::Relaxed);
                                            }
                                        }

                                        let center = rect.center();
                                        let is_hovered = response.hovered();

                                        let p = ui.painter();

                                        // Draw Jack Cell Plate
                                        let cell_bg = if is_hovered {
                                            Color32::from_rgb(36, 44, 56)
                                        } else if is_connected {
                                            Color32::from_rgb(26, 33, 44)
                                        } else {
                                            Color32::from_rgb(22, 26, 32)
                                        };

                                        p.rect_filled(rect, 3.0, cell_bg);
                                        p.rect_stroke(
                                            rect,
                                            3.0,
                                            Stroke::new(1.0, if is_connected { Color32::from_rgb(52, 76, 104) } else { Color32::from_rgb(38, 44, 54) }),
                                            egui::StrokeKind::Inside,
                                        );

                                        // Physical Audio Jack / Plug Graphic
                                        if is_connected {
                                            // CONNECTED STATE: Highly Visible Illuminated Studio Patch Plug
                                            let pulse = ((time * 4.0).sin() * 0.15 + 0.85) as f32;
                                            let core_color = Color32::from_rgb(
                                                (56.0 * pulse) as u8,
                                                (189.0 * pulse) as u8,
                                                (248.0 * pulse) as u8,
                                            );

                                            // Cyan Glow Aura
                                            p.circle_filled(center, 11.0, Color32::from_rgba_premultiplied(56, 189, 248, 50));
                                            // Outer Jack Collar
                                            p.circle_stroke(center, 8.0, Stroke::new(1.8, core_color));
                                            // Center Plug Pin
                                            p.circle_filled(center, 5.5, core_color);
                                            // High-intensity center highlight
                                            p.circle_filled(center, 2.0, Color32::WHITE);
                                        } else {
                                            // DISCONNECTED STATE: High-Contrast Brushed Metal Jack Socket
                                            let bezel_color = if is_hovered {
                                                Color32::from_rgb(140, 160, 190) // Bright highlight on hover
                                            } else {
                                                Color32::from_rgb(95, 112, 135) // High-contrast visible metal ring
                                            };

                                            // Outer Metal Socket Bezel (Crisp & clearly visible)
                                            p.circle_stroke(center, 7.5, Stroke::new(1.6, bezel_color));
                                            // Recessed Socket Chamber
                                            p.circle_filled(center, 5.5, Color32::from_rgb(14, 17, 22));
                                            // Center Metal Contact Pin Hole (Solid silver dot)
                                            p.circle_filled(center, 2.2, Color32::from_rgb(130, 148, 170));
                                        }

                                        ui.add_space(col_gap);
                                    }
                                });

                                ui.add_space(4.0);
                            }

                            ui.add_space(8.0);

                            // Bottom Legend & Help Bar
                            ui.horizontal(|ui| {
                                ui.colored_label(Color32::from_rgb(56, 189, 248), "●");
                                ui.label(egui::RichText::new(tr("已连接跳线", "Active Patch")).size(10.5).color(Color32::from_rgb(180, 192, 210)));

                                ui.add_space(12.0);
                                ui.colored_label(Color32::from_rgb(110, 128, 150), "○");
                                ui.label(egui::RichText::new(tr("可用插口", "Available Socket")).size(10.5).color(Color32::from_rgb(140, 152, 168)));

                                ui.add_space(14.0);
                                ui.label(
                                    egui::RichText::new(tr("提示: 点击任意交叉点可立即插入或拔出虚拟音频跳线", "Tip: Click any intersection node to connect or disconnect patch routing."))
                                        .size(10.5)
                                        .color(Color32::from_rgb(110, 120, 135)),
                                );
                            });
                        });
                    });

                ui.add_space(gap_between_panels);

                // ==========================================
                // 2. RIGHT PANEL: ROUTING INSPECTOR & PRESETS
                // ==========================================
                egui::Frame::NONE
                    .fill(Color32::from_rgb(20, 23, 29))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 56)))
                    .corner_radius(egui::CornerRadius::same(6))
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(inspector_w - 20.0);

                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(tr("🎛️ 路由状态与控制台", "Routing Inspector & Controls")).strong().size(12.0).color(Color32::from_rgb(235, 240, 248)));
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
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    ui.set_width(inspector_w - 36.0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new(format!("{}", active_nodes)).strong().size(20.0).color(Color32::from_rgb(56, 189, 248)));
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(tr("活跃跳线通路", "Active Connections")).strong().size(10.5).color(Color32::from_rgb(220, 228, 238)));
                                            ui.label(egui::RichText::new(format!("{} / 36 {}", tr("共", "Total"), tr("最大矩阵节点", "matrix nodes"))).size(9.5).color(Color32::from_rgb(110, 120, 135)));
                                        });
                                    });
                                });

                            ui.add_space(8.0);
                            render_info_row(ui, tr("矩阵吞吐", "Throughput"), tr("零延迟 (0.0 ms)", "Zero Latency"));
                            render_info_row(ui, tr("路由引擎", "Router Engine"), tr("已激活", "Online"));
                            render_info_row(ui, tr("通道保护", "Isolation"), tr("全独立立体声", "Discrete Stereo"));

                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(tr("一键跳线场景预设", "One-Click Presets")).strong().size(10.5).color(Color32::from_rgb(180, 192, 210)));
                            ui.add_space(4.0);

                            // Preset Buttons
                            render_preset_card(
                                ui,
                                tr("🎙️ 网络直播标准模式", "🎙️ Live Stream Standard"),
                                tr("人声+DAW 混音送往 OBS 与监听", "Mix Mic & DAW to OBS & Monitors"),
                                || self.apply_broadcast_preset(),
                            );
                            ui.add_space(3.0);

                            render_preset_card(
                                ui,
                                tr("🎧 专业录音监听模式", "🎧 Studio Tracking Studio"),
                                tr("人声与乐器直接送至监听与宿主", "Direct Mic & Inst to Headphones"),
                                || self.apply_studio_preset(),
                            );
                            ui.add_space(3.0);

                            render_preset_card(
                                ui,
                                tr("🎮 连麦游戏通话模式", "🎮 Discord / Gaming Voice"),
                                tr("游戏声送耳机，语音独立分流", "Separate Game Audio & Discord Mic"),
                                || self.apply_gaming_preset(),
                            );

                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(6.0);

                            if ui.checkbox(&mut self.direct_monitoring, tr("启用硬件零延迟监听", "Direct ASIO Monitoring")).changed() {
                                self.sync_to_audio_engine();
                            }
                            ui.add_space(2.0);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(tr("绕开 Windows 混音器，声卡原生超低延迟直通。", "Bypass OS mixer for true zero-latency monitoring."))
                                        .size(9.0)
                                        .color(Color32::from_rgb(110, 120, 135)),
                                )
                                .wrap(),
                            );
                        });
                    });
            });

            ui.add_space(10.0);

            // ==========================================
            // 3. BOTTOM PANEL: REAL-TIME BUS & I/O METER BRIDGE
            // ==========================================
            egui::Frame::NONE
                .fill(Color32::from_rgb(20, 23, 29))
                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 56)))
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_width(total_avail_w - 20.0);

                    ui.vertical(|ui| {
                        // Title row
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(tr("📊 物理与虚拟总线实时信号监视 (Bus & Stream Level Monitor)", "Real-Time Bus & Stream Level Monitor"))
                                    .strong()
                                    .size(11.5)
                                    .color(Color32::from_rgb(225, 235, 246)),
                            );
                            ui.label(
                                egui::RichText::new(tr("- 6路输入源与6路输出总线实时 Peak / RMS 动态电平监测", "- 6 Inputs & 6 Output Buses Real-time dBFS Levels"))
                                    .size(10.0)
                                    .color(Color32::from_rgb(120, 130, 145)),
                            );
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        let card_spacing = 6.0;
                        let total_meter_cols = 6.0;
                        let meter_card_w = ((total_avail_w - 44.0 - card_spacing * 5.0) / total_meter_cols).max(75.0);

                        // Row 1: 6 Inputs
                        ui.horizontal(|ui| {
                            for (s_idx, (tag, name)) in sources.iter().enumerate() {
                                let peak = if let Some(state) = &self.audio_state {
                                    f32::from_bits(state.input_peaks[s_idx].load(Ordering::Relaxed))
                                } else {
                                    (0.4 + (time * 2.5 + (s_idx as f64) * 0.9).sin() as f32 * 0.15).clamp(0.02, 0.95)
                                };
                                render_bus_meter_card(ui, meter_card_w, tag, name, peak, true);
                                if s_idx < 5 {
                                    ui.add_space(card_spacing);
                                }
                            }
                        });

                        ui.add_space(6.0);

                        // Row 2: 6 Outputs
                        ui.horizontal(|ui| {
                            for (d_idx, (tag, name)) in destinations.iter().enumerate() {
                                let mut out_peak: f32 = 0.0;
                                if let Some(state) = &self.audio_state {
                                    for s in 0..6 {
                                        if self.connections[s][d_idx] {
                                            let p = f32::from_bits(state.input_peaks[s].load(Ordering::Relaxed));
                                            out_peak = (out_peak + p * 0.7).min(1.0);
                                        }
                                    }
                                } else {
                                    out_peak = (0.5 + (time * 3.0 + (d_idx as f64) * 0.8).sin() as f32 * 0.18).clamp(0.02, 0.95);
                                }
                                render_bus_meter_card(ui, meter_card_w, tag, name, out_peak, false);
                                if d_idx < 5 {
                                    ui.add_space(card_spacing);
                                }
                            }
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
        self.sync_to_audio_engine();
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
        self.sync_to_audio_engine();
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
        self.sync_to_audio_engine();
    }
}

fn render_segment_meter(p: &egui::Painter, rect: Rect, level: f32) {
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

fn render_bus_meter_card(ui: &mut Ui, width: f32, tag: &str, name: &str, level: f32, is_input: bool) {
    egui::Frame::NONE
        .fill(Color32::from_rgb(23, 28, 36))
        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 46, 58)))
        .corner_radius(egui::CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(6, 4))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(tag)
                            .strong()
                            .size(9.5)
                            .color(if is_input { Color32::from_rgb(56, 189, 248) } else { Color32::from_rgb(52, 211, 153) }),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let db_text = if level < 0.05 {
                            "-inf".to_string()
                        } else {
                            format!("{:.1}dB", 20.0 * level.log10())
                        };
                        ui.label(egui::RichText::new(db_text).monospace().size(8.5).color(Color32::from_rgb(130, 142, 158)));
                    });
                });
                ui.label(egui::RichText::new(name).size(9.5).color(Color32::from_rgb(210, 220, 235)));

                // Horizontal meter bar
                let (meter_rect, _) = ui.allocate_exact_size(Vec2::new(width - 4.0, 5.0), egui::Sense::hover());
                let p = ui.painter();
                p.rect_filled(meter_rect, 1.0, Color32::from_rgb(12, 14, 18));
                let active_w = (meter_rect.width() * level.clamp(0.0, 1.0)).max(0.0);
                let fill_rect = Rect::from_min_size(meter_rect.min, Vec2::new(active_w, meter_rect.height()));
                let col = if level > 0.88 {
                    Color32::from_rgb(248, 113, 113)
                } else if level > 0.65 {
                    Color32::from_rgb(245, 158, 11)
                } else {
                    Color32::from_rgb(52, 211, 153)
                };
                p.rect_filled(fill_rect, 1.0, col);
            });
        });
}
