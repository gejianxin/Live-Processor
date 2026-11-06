use std::f32::consts::PI;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::audio::stream::{LiveStreamHub, ServerConnectionState, ServerRegion};

pub struct LiveProcessorView {
    pub hub: LiveStreamHub,
    pub editing_destination_idx: Option<usize>,
    pub show_device_config: bool,
}

impl Default for LiveProcessorView {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveProcessorView {
    pub fn new() -> Self {
        Self {
            hub: LiveStreamHub::new(),
            editing_destination_idx: None,
            show_device_config: false,
        }
    }

    pub fn render(&mut self, ui: &mut Ui, time: f64) {
        self.hub.update_frame(time);

        let avail_w = ui.available_width();
        let avail_h = ui.available_height().max(560.0);

        // Studio Dark Theme Canvas
        egui::Frame::NONE
            .fill(Color32::from_rgb(14, 21, 30)) // #0E151E Authentic Dialog Audio Background
            .inner_margin(egui::Margin::symmetric(24, 18))
            .show(ui, |ui| {
                ui.set_width(avail_w);
                ui.set_height(avail_h);

                // 3-Column Studio Layout
                // Left: Server & Configuration (300px)
                // Center: Audio (Input, Monitor, Stream) (approx 520px)
                // Right: Stream Status & Telemetry (approx 280px)
                let left_w = 290.0;
                let right_w = 270.0;
                let center_w = (avail_w - left_w - right_w - 70.0).clamp(460.0, 600.0);

                ui.horizontal_top(|ui| {
                    // 1. LEFT COLUMN: Server & Multi-Destination Configuration
                    ui.allocate_ui(Vec2::new(left_w, avail_h), |ui| {
                        self.render_left_column(ui);
                    });

                    ui.add_space(16.0);

                    // 2. CENTER COLUMN: Audio (Audio Input | Monitor Output | Stream Output)
                    ui.allocate_ui(Vec2::new(center_w, avail_h), |ui| {
                        self.render_center_column(ui, center_w, time);
                    });

                    ui.add_space(16.0);

                    // 3. RIGHT COLUMN: Stream (Network Good, Waveform, Timer, Record/Mic)
                    ui.allocate_ui(Vec2::new(right_w, avail_h), |ui| {
                        self.render_right_column(ui, time);
                    });
                });
            });

        // Config Modal for Stream Destination
        if let Some(idx) = self.editing_destination_idx {
            if idx < self.hub.destinations.len() {
                self.render_destination_modal(ui, idx);
            }
        }

        // Config Modal for Audio Device
        if self.show_device_config {
            self.render_device_modal(ui);
        }
    }

    fn render_left_column(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Header: Server
            ui.label(
                egui::RichText::new("Server")
                    .size(17.0)
                    .strong()
                    .color(Color32::from_rgb(220, 230, 242)),
            );
            ui.add_space(12.0);

            // Hardware Audio Interface Selector
            ui.horizontal(|ui| {
                let mut active = true;
                ui.checkbox(&mut active, "");
                ui.label(
                    egui::RichText::new("Signature 200")
                        .size(14.0)
                        .color(Color32::from_rgb(200, 215, 230)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let gear_btn = egui::Button::new(
                        egui::RichText::new("⚙")
                            .size(14.0)
                            .color(Color32::from_rgb(148, 163, 184)),
                    )
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::NONE);

                    if ui.add(gear_btn).clicked() {
                        self.show_device_config = true;
                    }
                });
            });

            ui.add_space(32.0);

            // Header: Configuration
            ui.label(
                egui::RichText::new("Configuration")
                    .size(17.0)
                    .strong()
                    .color(Color32::from_rgb(220, 230, 242)),
            );
            ui.add_space(12.0);

            // Multi-Destination list
            let mut edit_req = None;
            for (idx, dest) in self.hub.destinations.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    ui.checkbox(&mut dest.enabled, "");

                    let name_color = if dest.enabled {
                        Color32::from_rgb(225, 235, 245)
                    } else {
                        Color32::from_rgb(120, 135, 150)
                    };

                    ui.label(
                        egui::RichText::new(&dest.name)
                            .size(13.5)
                            .color(name_color),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gear = egui::Button::new(
                            egui::RichText::new("⚙")
                                .size(13.0)
                                .color(Color32::from_rgb(140, 155, 175)),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE);

                        if ui.add(gear).clicked() {
                            edit_req = Some(idx);
                        }

                        let proto_text = dest.protocol.as_str();
                        ui.label(
                            egui::RichText::new(proto_text)
                                .size(11.5)
                                .color(Color32::from_rgb(130, 145, 165)),
                        );
                    });
                });
                ui.add_space(8.0);
            }
            if let Some(idx) = edit_req {
                self.editing_destination_idx = Some(idx);
            }

            ui.add_space(40.0);

            // Bottom Server controls
            // Server Location Dropdown
            let cur_region = self.hub.server_region;
            egui::ComboBox::from_id_salt("server_region_select")
                .selected_text(
                    egui::RichText::new(cur_region.display_name())
                        .size(13.0)
                        .color(Color32::from_rgb(210, 225, 240)),
                )
                .width(260.0)
                .show_ui(ui, |ui| {
                    for region in ServerRegion::all() {
                        let label = format!("{} ({} ms)", region.display_name(), region.latency_ms());
                        if ui.selectable_label(self.hub.server_region == *region, label).clicked() {
                            self.hub.server_region = *region;
                        }
                    }
                });

            ui.add_space(12.0);

            // Start / Stop Stream Server button (Authentic Dialog Audio pill button)
            let server_online = self.hub.server_state != ServerConnectionState::Offline;
            let btn_text = if server_online {
                "Stop Stream Server"
            } else {
                "Start Stream Server"
            };

            let stop_btn = egui::Button::new(
                egui::RichText::new(btn_text)
                    .size(13.5)
                    .color(Color32::from_rgb(230, 240, 250)),
            )
            .fill(Color32::from_rgb(26, 36, 48))
            .stroke(Stroke::new(1.0, Color32::from_rgb(45, 60, 80)))
            .corner_radius(egui::CornerRadius::same(6));

            if ui.add_sized([260.0, 32.0], stop_btn).clicked() {
                self.hub.toggle_server_connection();
            }

            ui.add_space(14.0);

            // Status: Online / Offline
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Status:")
                        .size(13.5)
                        .color(Color32::from_rgb(200, 215, 230)),
                );
                ui.add_space(18.0);
                let (status_text, status_color) = match self.hub.server_state {
                    ServerConnectionState::Offline => ("Offline", Color32::from_rgb(148, 163, 184)),
                    ServerConnectionState::Connecting => ("Connecting...", Color32::from_rgb(251, 191, 36)),
                    ServerConnectionState::Online | ServerConnectionState::Streaming => ("Online", Color32::from_rgb(56, 189, 248)),
                };
                ui.label(
                    egui::RichText::new(status_text)
                        .size(13.5)
                        .strong()
                        .color(status_color),
                );
            });
        });
    }

    fn render_center_column(&mut self, ui: &mut Ui, total_w: f32, _time: f64) {
        ui.vertical(|ui| {
            // Header: Audio
            ui.label(
                egui::RichText::new("Audio")
                    .size(17.0)
                    .strong()
                    .color(Color32::from_rgb(220, 230, 242)),
            );
            ui.add_space(10.0);

            // Three vertical channel strips:
            // 1. Audio Input (approx 30%)
            // 2. Monitor Output (approx 30%)
            // 3. Stream Output (approx 40%, with prominent master meters)
            let col1_w = (total_w * 0.28).max(120.0);
            let col2_w = (total_w * 0.28).max(120.0);
            let col3_w = (total_w - col1_w - col2_w - 4.0).max(160.0);

            ui.horizontal_top(|ui| {
                // Strip 1: Audio Input
                ui.allocate_ui(Vec2::new(col1_w, 480.0), |ui| {
                    self.render_input_strip(ui);
                });

                // Divider 1
                let (div_rect1, _) = ui.allocate_exact_size(Vec2::new(2.0, 470.0), Sense::hover());
                ui.painter().line_segment(
                    [Pos2::new(div_rect1.center().x, div_rect1.min.y), Pos2::new(div_rect1.center().x, div_rect1.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(28, 42, 60)),
                );

                // Strip 2: Monitor Output
                ui.allocate_ui(Vec2::new(col2_w, 480.0), |ui| {
                    self.render_monitor_strip(ui);
                });

                // Divider 2
                let (div_rect2, _) = ui.allocate_exact_size(Vec2::new(2.0, 470.0), Sense::hover());
                ui.painter().line_segment(
                    [Pos2::new(div_rect2.center().x, div_rect2.min.y), Pos2::new(div_rect2.center().x, div_rect2.max.y)],
                    Stroke::new(1.0, Color32::from_rgb(28, 42, 60)),
                );

                // Strip 3: Stream Output
                ui.allocate_ui(Vec2::new(col3_w, 480.0), |ui| {
                    self.render_stream_strip(ui);
                });
            });
        });
    }

    fn render_input_strip(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            // Top: Pan Knob (Small metallic rotary knob)
            render_rotary_knob(ui, &mut self.hub.input_pan, -1.0, 1.0, 0.0, 20.0, false);
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!("{:.2}", self.hub.input_pan))
                    .size(12.5)
                    .color(Color32::from_rgb(210, 225, 240)),
            );

            ui.add_space(14.0);

            // Calibrated Dual VU Peak Meter
            render_channel_vu_meter(
                ui,
                self.hub.input_meter_l,
                self.hub.input_meter_r,
                false, // Standard 3-color gradient
            );

            ui.add_space(14.0);

            // Bottom: Big Input Gain Knob with Glowing Cyan Arc
            render_rotary_knob(ui, &mut self.hub.input_gain_db, -60.0, 12.0, 0.0, 34.0, true);
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("{:.1} dB", self.hub.input_gain_db))
                    .size(13.0)
                    .color(Color32::from_rgb(220, 235, 250)),
            );

            ui.add_space(18.0);
            ui.label(
                egui::RichText::new("Audio Input")
                    .size(13.5)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    fn render_monitor_strip(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            // Top: Pan Knob
            render_rotary_knob(ui, &mut self.hub.monitor_pan, -1.0, 1.0, 0.0, 20.0, false);
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(format!("{:.2}", self.hub.monitor_pan))
                    .size(12.5)
                    .color(Color32::from_rgb(210, 225, 240)),
            );

            ui.add_space(14.0);

            // Calibrated Dual VU Peak Meter
            render_channel_vu_meter(
                ui,
                self.hub.monitor_meter_l,
                self.hub.monitor_meter_r,
                false,
            );

            ui.add_space(14.0);

            // Big Monitor Gain Knob with Glowing Cyan Arc
            render_rotary_knob(ui, &mut self.hub.monitor_gain_db, -60.0, 12.0, -11.2, 34.0, true);
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("{:.1} dB", self.hub.monitor_gain_db))
                    .size(13.0)
                    .color(Color32::from_rgb(220, 235, 250)),
            );

            ui.add_space(18.0);
            ui.label(
                egui::RichText::new("Monitor Output")
                    .size(13.5)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    fn render_stream_strip(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            // Master Stream Output has prominent clip indicator blocks and extra tall high-res meter
            render_stream_master_vu_meter(
                ui,
                self.hub.stream_meter_l,
                self.hub.stream_meter_r,
            );

            ui.add_space(14.0);

            // Big Stream Master Gain Knob with Glowing Cyan Arc
            render_rotary_knob(ui, &mut self.hub.stream_gain_db, -60.0, 12.0, 0.0, 34.0, true);
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(format!("{:.1} dB", self.hub.stream_gain_db))
                    .size(13.0)
                    .color(Color32::from_rgb(220, 235, 250)),
            );

            ui.add_space(18.0);
            ui.label(
                egui::RichText::new("Stream Output")
                    .size(13.5)
                    .color(Color32::from_rgb(148, 163, 184)),
            );
        });
    }

    fn render_right_column(&mut self, ui: &mut Ui, time: f64) {
        ui.vertical(|ui| {
            // Header: Stream
            ui.label(
                egui::RichText::new("Stream")
                    .size(17.0)
                    .strong()
                    .color(Color32::from_rgb(220, 230, 242)),
            );
            ui.add_space(80.0);

            ui.vertical_centered(|ui| {
                // Network Good (Glowing Cyan Title)
                ui.label(
                    egui::RichText::new("Network Good")
                        .size(21.0)
                        .strong()
                        .color(Color32::from_rgb(56, 189, 248)),
                );
                ui.add_space(18.0);

                // Network / Audio Activity Spectrum Bars (32 bars)
                render_spectrum_oscilloscope(ui, &self.hub.spectrum_bars);

                ui.add_space(40.0);

                // Real-time Stream Session Timer (Large clean digital font)
                let timer_text = self.hub.formatted_timer();
                ui.label(
                    egui::RichText::new(timer_text)
                        .size(34.0)
                        .color(Color32::from_rgb(240, 246, 252)),
                );

                ui.add_space(34.0);

                // Bottom Action Buttons:
                // [ ● ] Record / Stream Button + [ 🎤 ] Microphone Toggle
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 24.0;
                    let total_btn_w = 46.0 + 24.0 + 46.0;
                    let left_pad = ((ui.available_width() - total_btn_w) / 2.0).max(0.0);
                    if left_pad > 0.0 {
                        ui.add_space(left_pad);
                    }

                    // Allocate rects for both action buttons
                    let rec_active = self.hub.is_recording;
                    let (rec_rect, rec_resp) = ui.allocate_exact_size(Vec2::new(48.0, 48.0), Sense::click());
                    let mic_muted = self.hub.mic_muted;
                    let (mic_rect, mic_resp) = ui.allocate_exact_size(Vec2::new(48.0, 48.0), Sense::click());

                    let p = ui.painter();

                    let rec_border = if rec_active {
                        Stroke::new(1.5, Color32::from_rgb(244, 63, 94))
                    } else {
                        Stroke::new(1.0, Color32::from_rgb(60, 75, 95))
                    };
                    p.rect_filled(rec_rect, egui::CornerRadius::same(10), Color32::from_rgb(22, 30, 42));
                    p.rect_stroke(rec_rect, egui::CornerRadius::same(10), rec_border, egui::StrokeKind::Inside);

                    // Glowing Pink-Red Record Dot
                    let dot_color = if rec_active {
                        let pulse = ((time * 6.0).sin() * 0.2 + 0.8) as f32;
                        Color32::from_rgb(
                            (244.0 * pulse) as u8,
                            (63.0 * pulse) as u8,
                            (94.0 * pulse) as u8,
                        )
                    } else {
                        Color32::from_rgb(236, 72, 153)
                    };
                    p.circle_filled(rec_rect.center(), 9.0, dot_color);

                    let mic_color = if mic_muted {
                        Color32::from_rgb(100, 116, 139)
                    } else {
                        Color32::from_rgb(236, 72, 153) // Magenta / Pink-red
                    };

                    draw_microphone_icon(p, mic_rect.center(), mic_color, mic_muted);

                    if rec_resp.clicked() {
                        self.hub.toggle_record();
                    }
                    if mic_resp.clicked() {
                        self.hub.toggle_mic_mute();
                    }
                });
            });
        });
    }

    fn render_destination_modal(&mut self, ui: &mut Ui, idx: usize) {
        let mut close_modal = false;
        let dest = &mut self.hub.destinations[idx];

        egui::Window::new(format!("⚙ Stream Configuration: {}", dest.name))
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(380.0, 240.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Protocol:").strong());
                    ui.label(dest.protocol.as_str());
                });

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Stream Server URL:").strong());
                ui.text_edit_singleline(&mut dest.url);

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Stream Key / Token:").strong());
                ui.text_edit_singleline(&mut dest.stream_key);

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Bitrate:").strong());
                    egui::ComboBox::from_id_salt("bitrate_select")
                        .selected_text(format!("{} kbps", dest.bitrate_kbps))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut dest.bitrate_kbps, 128, "128 kbps (Standard)");
                            ui.selectable_value(&mut dest.bitrate_kbps, 192, "192 kbps (HQ Audio)");
                            ui.selectable_value(&mut dest.bitrate_kbps, 256, "256 kbps (Pro Studio)");
                            ui.selectable_value(&mut dest.bitrate_kbps, 320, "320 kbps (Max HLS/CDN)");
                        });
                });

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button(egui::RichText::new("✔ Save & Apply").strong()).clicked() {
                        close_modal = true;
                    }
                    if ui.button("Close").clicked() {
                        close_modal = true;
                    }
                });
            });

        if close_modal {
            self.editing_destination_idx = None;
        }
    }

    fn render_device_modal(&mut self, ui: &mut Ui) {
        let mut close_modal = false;
        egui::Window::new("⚙ Audio Hardware Settings")
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(360.0, 200.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Audio Interface:").strong());
                ui.text_edit_singleline(&mut self.hub.selected_device);

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Driver Architecture:").strong());
                ui.label("ASIO / WASAPI Low Latency Engine (Active)");

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Direct Zero-Latency Monitor: Enabled").color(Color32::from_rgb(56, 189, 248)));

                ui.add_space(16.0);
                if ui.button("Done").clicked() {
                    close_modal = true;
                }
            });

        if close_modal {
            self.show_device_config = false;
        }
    }
}

// ============================================================================
// Custom Skeuomorphic Metallic Rotary Knob Widget (Faithful to Dialog Audio)
// ============================================================================
fn render_rotary_knob(
    ui: &mut Ui,
    value: &mut f32,
    min: f32,
    max: f32,
    default_val: f32,
    radius: f32,
    show_arc: bool,
) {
    let size = Vec2::splat((radius + 4.0) * 2.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let center = rect.center();
    let p = ui.painter();

    // Interaction handling: drag vertically/horizontally
    if resp.dragged() {
        let delta = -resp.drag_delta().y + resp.drag_delta().x;
        let range = max - min;
        let step = range / 200.0;
        *value = (*value + delta * step).clamp(min, max);
    }
    if resp.double_clicked() {
        *value = default_val;
    }

    let norm = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    // Angular range: 220 degrees sweep (from -140 deg to +140 deg)
    let start_angle = -140.0_f32.to_radians() - PI / 2.0;
    let total_sweep = 280.0_f32.to_radians();
    let cur_angle = start_angle + norm * total_sweep;

    // 1. Draw glowing cyan outer arc gauge (for Gain knobs)
    if show_arc {
        let arc_r = radius + 3.0;
        let arc_steps = 32;
        let sweep_steps = (norm * (arc_steps as f32)).round() as usize;

        // Dark track behind arc
        for i in 0..arc_steps {
            let a1 = start_angle + (i as f32 / arc_steps as f32) * total_sweep;
            let a2 = start_angle + ((i + 1) as f32 / arc_steps as f32) * total_sweep;
            let p1 = center + Vec2::new(a1.cos(), a1.sin()) * arc_r;
            let p2 = center + Vec2::new(a2.cos(), a2.sin()) * arc_r;
            p.line_segment([p1, p2], Stroke::new(2.5, Color32::from_rgb(26, 36, 48)));
        }

        // Active Cyan Arc
        for i in 0..sweep_steps {
            let a1 = start_angle + (i as f32 / arc_steps as f32) * total_sweep;
            let a2 = start_angle + ((i + 1) as f32 / arc_steps as f32) * total_sweep;
            let p1 = center + Vec2::new(a1.cos(), a1.sin()) * arc_r;
            let p2 = center + Vec2::new(a2.cos(), a2.sin()) * arc_r;
            p.line_segment([p1, p2], Stroke::new(2.5, Color32::from_rgb(56, 189, 248)));
        }
    }

    // 2. Outer rim drop shadow
    p.circle_filled(center, radius + 1.0, Color32::from_rgb(10, 14, 20));

    // 3. Metallic 3D Dial Body (Dark Slate gradient)
    p.circle_filled(center, radius, Color32::from_rgb(42, 50, 62));
    p.circle_filled(center + Vec2::new(0.0, 1.0), radius - 1.5, Color32::from_rgb(32, 38, 48));

    // Outer dial stroke
    p.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_rgb(60, 72, 90)));

    // 4. Pointer notch line (Authentic Dialog Audio notch)
    let pointer_inner = center + Vec2::new(cur_angle.cos(), cur_angle.sin()) * (radius * 0.35);
    let pointer_outer = center + Vec2::new(cur_angle.cos(), cur_angle.sin()) * (radius * 0.88);
    let notch_color = if resp.hovered() || resp.dragged() {
        Color32::from_rgb(56, 189, 248)
    } else {
        Color32::from_rgb(220, 230, 242)
    };
    p.line_segment([pointer_inner, pointer_outer], Stroke::new(2.0, notch_color));
}

// ============================================================================
// Channel Stereo VU Meter (Input & Monitor Output)
// ============================================================================
fn render_channel_vu_meter(ui: &mut Ui, level_l: f32, level_r: f32, _is_stream: bool) {
    let meter_h = 160.0;
    let bar_w = 6.0;
    let spacing = 3.0;
    let total_w = bar_w * 2.0 + spacing + 34.0; // includes dB markings
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, meter_h + 16.0), Sense::hover());
    let p = ui.painter();

    let meter_top_y = rect.min.y + 12.0;
    let meter_bot_y = meter_top_y + meter_h;
    let left_bar_x = rect.min.x + 16.0;
    let right_bar_x = left_bar_x + bar_w + spacing;

    // Peak Clip Indicators (Square boxes at top)
    let clip_l = level_l >= 1.0;
    let clip_r = level_r >= 1.0;
    p.rect_filled(
        Rect::from_min_size(Pos2::new(left_bar_x, rect.min.y), Vec2::new(bar_w, 6.0)),
        egui::CornerRadius::same(1),
        if clip_l { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(26, 32, 42) },
    );
    p.rect_filled(
        Rect::from_min_size(Pos2::new(right_bar_x, rect.min.y), Vec2::new(bar_w, 6.0)),
        egui::CornerRadius::same(1),
        if clip_r { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(26, 32, 42) },
    );

    // Meter Background Tracks
    p.rect_filled(
        Rect::from_min_size(Pos2::new(left_bar_x, meter_top_y), Vec2::new(bar_w, meter_h)),
        egui::CornerRadius::same(1),
        Color32::from_rgb(18, 24, 34),
    );
    p.rect_filled(
        Rect::from_min_size(Pos2::new(right_bar_x, meter_top_y), Vec2::new(bar_w, meter_h)),
        egui::CornerRadius::same(1),
        Color32::from_rgb(18, 24, 34),
    );

    // Draw active filled levels (Green -> Yellow -> Red)
    draw_vu_fill(p, left_bar_x, meter_top_y, meter_bot_y, bar_w, level_l);
    draw_vu_fill(p, right_bar_x, meter_top_y, meter_bot_y, bar_w, level_r);

    // dB Scale Markings (0, 6, 12, 20, 30, 40, 50, 60)
    let marks = [(0.0, "0"), (6.0, "6"), (12.0, "12"), (20.0, "20"), (30.0, "30"), (40.0, "40"), (50.0, "50"), (60.0, "60")];
    for (db, label) in marks {
        let t = db / 60.0;
        let y = meter_top_y + t * meter_h;
        p.text(
            Pos2::new(rect.min.x + 8.0, y),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::proportional(8.5),
            Color32::from_rgb(110, 125, 145),
        );
    }
}

// ============================================================================
// Stream Output Master VU Meter (Tall 4-Color Gradient Blue -> Green -> Yellow -> Red)
// ============================================================================
fn render_stream_master_vu_meter(ui: &mut Ui, level_l: f32, level_r: f32) {
    let meter_h = 220.0;
    let bar_w = 8.0;
    let spacing = 4.0;
    let total_w = bar_w * 2.0 + spacing + 50.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, meter_h + 20.0), Sense::hover());
    let p = ui.painter();

    let meter_top_y = rect.min.y + 14.0;
    let meter_bot_y = meter_top_y + meter_h;
    let left_bar_x = rect.center().x - bar_w - spacing / 2.0;
    let right_bar_x = rect.center().x + spacing / 2.0;

    // Top Clip Indicators
    let clip_l = level_l >= 1.0;
    let clip_r = level_r >= 1.0;
    p.rect_filled(
        Rect::from_min_size(Pos2::new(left_bar_x, rect.min.y), Vec2::new(bar_w, 8.0)),
        egui::CornerRadius::same(1),
        if clip_l { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(26, 32, 42) },
    );
    p.rect_filled(
        Rect::from_min_size(Pos2::new(right_bar_x, rect.min.y), Vec2::new(bar_w, 8.0)),
        egui::CornerRadius::same(1),
        if clip_r { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(26, 32, 42) },
    );

    // Background tracks
    p.rect_filled(
        Rect::from_min_size(Pos2::new(left_bar_x, meter_top_y), Vec2::new(bar_w, meter_h)),
        egui::CornerRadius::same(1),
        Color32::from_rgb(18, 24, 34),
    );
    p.rect_filled(
        Rect::from_min_size(Pos2::new(right_bar_x, meter_top_y), Vec2::new(bar_w, meter_h)),
        egui::CornerRadius::same(1),
        Color32::from_rgb(18, 24, 34),
    );

    // Multi-color 4-stage gradient fill (Blue at bottom -> Green -> Yellow -> Red)
    draw_stream_vu_fill(p, left_bar_x, meter_top_y, meter_bot_y, bar_w, level_l);
    draw_stream_vu_fill(p, right_bar_x, meter_top_y, meter_bot_y, bar_w, level_r);

    // Marks on left: 0, 3, 6, 9, 12, 16, 20, 25, 30, 40, 50, 60
    let marks = [
        (0.0, "0"), (3.0, "3"), (6.0, "6"), (9.0, "9"), (12.0, "12"),
        (16.0, "16"), (20.0, "20"), (25.0, "25"), (30.0, "30"),
        (40.0, "40"), (50.0, "50"), (60.0, "60"),
    ];
    for (db, label) in marks {
        let t = db / 60.0;
        let y = meter_top_y + t * meter_h;
        // Left scale
        p.text(
            Pos2::new(left_bar_x - 6.0, y),
            egui::Align2::RIGHT_CENTER,
            label,
            egui::FontId::proportional(8.5),
            Color32::from_rgb(120, 135, 155),
        );
        // Right scale
        p.text(
            Pos2::new(right_bar_x + bar_w + 6.0, y),
            egui::Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(8.5),
            Color32::from_rgb(120, 135, 155),
        );
    }
}

fn draw_vu_fill(p: &egui::Painter, x: f32, top_y: f32, bot_y: f32, w: f32, level: f32) {
    let h = bot_y - top_y;
    let active_h = (level * h).clamp(0.0, h);
    if active_h <= 0.0 {
        return;
    }

    let steps = 40;
    let step_h = h / (steps as f32);
    let active_steps = (level * (steps as f32)).round() as usize;

    for i in 0..active_steps.min(steps) {
        let seg_y = bot_y - (i as f32 + 1.0) * step_h;
        let seg_norm = (i as f32) / (steps as f32);

        let color = if seg_norm > 0.85 {
            Color32::from_rgb(239, 68, 68) // Red
        } else if seg_norm > 0.65 {
            Color32::from_rgb(245, 158, 11) // Yellow
        } else {
            Color32::from_rgb(34, 197, 94) // Green
        };

        p.rect_filled(
            Rect::from_min_size(Pos2::new(x, seg_y), Vec2::new(w, step_h - 0.8)),
            egui::CornerRadius::same(1),
            color,
        );
    }
}

fn draw_stream_vu_fill(p: &egui::Painter, x: f32, top_y: f32, bot_y: f32, w: f32, level: f32) {
    let h = bot_y - top_y;
    let steps = 50;
    let step_h = h / (steps as f32);
    let active_steps = (level * (steps as f32)).round() as usize;

    for i in 0..active_steps.min(steps) {
        let seg_y = bot_y - (i as f32 + 1.0) * step_h;
        let seg_norm = (i as f32) / (steps as f32);

        let color = if seg_norm > 0.90 {
            Color32::from_rgb(239, 68, 68) // Red
        } else if seg_norm > 0.75 {
            Color32::from_rgb(245, 158, 11) // Yellow
        } else if seg_norm > 0.35 {
            Color32::from_rgb(34, 197, 94) // Green
        } else {
            Color32::from_rgb(14, 165, 233) // Blue at base
        };

        p.rect_filled(
            Rect::from_min_size(Pos2::new(x, seg_y), Vec2::new(w, step_h - 0.8)),
            egui::CornerRadius::same(1),
            color,
        );
    }
}

// ============================================================================
// Real-time Spectrum Oscilloscope Bars (32 Bars in Right Column)
// ============================================================================
fn render_spectrum_oscilloscope(ui: &mut Ui, bars: &[f32; 32]) {
    let total_w = 200.0;
    let total_h = 36.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::hover());
    let p = ui.painter();

    let num_bars = bars.len();
    let bar_w = 3.5;
    let step = total_w / (num_bars as f32);
    let center_y = rect.center().y;

    for (i, &amp) in bars.iter().enumerate() {
        let bar_x = rect.min.x + (i as f32) * step + (step - bar_w) / 2.0;
        let half_h = (amp * (total_h / 2.0)).clamp(2.0, total_h / 2.0);

        // Highlight spikes in orange (e.g. index 7 and 21 like genuine reference screenshot)
        let bar_color = if i == 7 || i == 21 {
            Color32::from_rgb(245, 158, 11)
        } else {
            Color32::from_rgb(56, 189, 248)
        };

        p.rect_filled(
            Rect::from_min_max(Pos2::new(bar_x, center_y - half_h), Pos2::new(bar_x + bar_w, center_y + half_h)),
            egui::CornerRadius::same(1),
            bar_color,
        );
    }
}

// ============================================================================
// Sleek Microphone Icon Rendering
// ============================================================================
fn draw_microphone_icon(p: &egui::Painter, center: Pos2, color: Color32, muted: bool) {
    let top_y = center.y - 12.0;
    // Mic Capsule
    let capsule_rect = Rect::from_center_size(Pos2::new(center.x, top_y + 6.0), Vec2::new(10.0, 15.0));
    p.rect_filled(capsule_rect, egui::CornerRadius::same(5), color);

    // Mic U-cradle
    let u_left = Pos2::new(center.x - 8.0, top_y + 6.0);
    let u_bot_left = Pos2::new(center.x - 8.0, top_y + 14.0);
    let u_bot_right = Pos2::new(center.x + 8.0, top_y + 14.0);
    let u_right = Pos2::new(center.x + 8.0, top_y + 6.0);
    p.line_segment([u_left, u_bot_left], Stroke::new(1.8, color));
    p.line_segment([u_bot_left, Pos2::new(center.x, top_y + 17.0)], Stroke::new(1.8, color));
    p.line_segment([Pos2::new(center.x, top_y + 17.0), u_bot_right], Stroke::new(1.8, color));
    p.line_segment([u_bot_right, u_right], Stroke::new(1.8, color));

    // Stand Stem & Base
    p.line_segment([Pos2::new(center.x, top_y + 17.0), Pos2::new(center.x, top_y + 22.0)], Stroke::new(2.0, color));
    p.line_segment([Pos2::new(center.x - 6.0, top_y + 22.0), Pos2::new(center.x + 6.0, top_y + 22.0)], Stroke::new(2.0, color));

    // Muted Slash
    if muted {
        p.line_segment(
            [Pos2::new(center.x - 10.0, top_y - 2.0), Pos2::new(center.x + 10.0, top_y + 24.0)],
            Stroke::new(2.0, Color32::from_rgb(239, 68, 68)),
        );
    }
}
