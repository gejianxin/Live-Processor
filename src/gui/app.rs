use egui::{Color32, Stroke, Ui};

use super::matrix::RoutingMatrix;
use super::rack::VirtualRack;
use super::snapshots::SnapshotManager;
use super::supervisor::SupervisorView;
#[cfg(windows)]
use crate::config::is_64bit_os;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ActiveTab {
    Matrix,
    Rack,
    Supervisor,
    Snapshots,
}

pub struct LiveProcessorApp {
    pub active_tab: ActiveTab,
    pub matrix: RoutingMatrix,
    pub rack: VirtualRack,
    pub supervisor: SupervisorView,
    pub snapshots: SnapshotManager,
    pub master_volume: f32,
    pub panic_muted: bool,
    pub sample_rate: u32,
    pub buffer_size: u32,
}

impl LiveProcessorApp {
    pub fn new() -> Self {
        Self {
            active_tab: ActiveTab::Matrix,
            matrix: RoutingMatrix::default(),
            rack: VirtualRack::default(),
            supervisor: SupervisorView::default(),
            snapshots: SnapshotManager::default(),
            master_volume: 0.0,
            panic_muted: false,
            sample_rate: 48000,
            buffer_size: 256,
        }
    }
}

impl eframe::App for LiveProcessorApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let time = ui.ctx().input(|i| i.time);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(16));

        // 1. Top Control Header
        egui::Frame::NONE
            .fill(Color32::from_rgb(14, 16, 21))
            .stroke(Stroke::new(1.0, Color32::from_rgb(32, 38, 48)))
            .inner_margin(12.0)
            .show(ui, |ui| {
                self.render_header(ui, time);
            });

        ui.add_space(6.0);

        // 2. Central Workspace Area
        let available_space = ui.available_height() - 36.0;
        egui::Frame::NONE
            .fill(Color32::from_rgb(18, 22, 28))
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.set_min_height(available_space.max(200.0));
                match self.active_tab {
                    ActiveTab::Matrix => self.matrix.render(ui, time),
                    ActiveTab::Rack => self.rack.render(ui, time),
                    ActiveTab::Supervisor => self.supervisor.render(ui),
                    ActiveTab::Snapshots => self.snapshots.render(ui),
                }
            });

        ui.add_space(6.0);

        // 3. Bottom Status Bar
        egui::Frame::NONE
            .fill(Color32::from_rgb(12, 14, 18))
            .stroke(Stroke::new(1.0, Color32::from_rgb(28, 32, 42)))
            .inner_margin(8.0)
            .show(ui, |ui| {
                self.render_status_bar(ui);
            });
    }
}

impl LiveProcessorApp {
    fn render_header(&mut self, ui: &mut Ui, time: f64) {
        ui.horizontal(|ui| {
            // Animated Pulse Dot
            let pulse = ((time * 4.0).sin() * 0.2 + 0.8) as f32;
            let dot_color = if self.panic_muted {
                Color32::from_rgb(255, 60, 60)
            } else {
                Color32::from_rgb(
                    (0.0 * pulse) as u8,
                    (240.0 * pulse) as u8,
                    (255.0 * pulse) as u8,
                )
            };

            ui.colored_label(dot_color, "●");
            ui.label(
                egui::RichText::new("LIVE-PROCESSOR")
                    .strong()
                    .size(16.0)
                    .color(Color32::from_rgb(240, 245, 255)),
            );
            ui.label(
                egui::RichText::new("PRO")
                    .size(10.0)
                    .color(Color32::from_rgb(0, 210, 255))
                    .background_color(Color32::from_rgb(20, 36, 48)),
            );

            ui.add_space(20.0);

            // Tab Navigation Buttons
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Matrix, "🎛️ Wire Matrix");
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Rack, "🎚️ Virtual Rack");
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Supervisor, "🛡️ Driver Supervisor");
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Snapshots, "💾 Show Cues");

            // Right-aligned audio hardware telemetry
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Panic Button
                let panic_btn = egui::Button::new(
                    egui::RichText::new(if self.panic_muted { "MUTED (PANIC)" } else { "PANIC" })
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(if self.panic_muted {
                    Color32::from_rgb(220, 30, 30)
                } else {
                    Color32::from_rgb(45, 52, 66)
                });

                if ui.add(panic_btn).clicked() {
                    self.panic_muted = !self.panic_muted;
                }

                ui.add_space(8.0);

                // Master Volume
                ui.add(egui::Slider::new(&mut self.master_volume, -40.0..=6.0).show_value(false));
                ui.label(egui::RichText::new(format!("{:+.1} dB", self.master_volume)).size(11.0).color(Color32::from_rgb(0, 220, 255)));
                ui.label(egui::RichText::new("Master:").size(11.0).color(Color32::GRAY));

                ui.add_space(14.0);

                // Latency Badge
                let latency_ms = (self.buffer_size as f32 / self.sample_rate as f32) * 1000.0;
                let badge_text = format!("{} Hz | {} smp (~{:.1} ms)", self.sample_rate, self.buffer_size, latency_ms);
                ui.label(
                    egui::RichText::new(badge_text)
                        .size(11.0)
                        .color(Color32::from_rgb(0, 230, 140))
                        .monospace(),
                );
            });
        });
    }

    fn render_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            #[cfg(windows)]
            let os_text = format!("Platform: Windows x64 ({})", if is_64bit_os() { "64-bit Kernel" } else { "32-bit" });
            #[cfg(not(windows))]
            let os_text = "Platform: Linux Audio Host (PipeWire/JACK/ALSA)".to_string();

            ui.label(egui::RichText::new(os_text).size(11.0).color(Color32::from_rgb(140, 155, 175)));
            ui.separator();

            ui.label(
                egui::RichText::new("ASIO Link Pro 2.4.4.2 [Give Academy Patched]")
                    .size(11.0)
                    .color(Color32::from_rgb(0, 210, 255)),
            );
            ui.separator();

            #[cfg(windows)]
            ui.label(egui::RichText::new("CI Policy UpgradedSystem = 1 [ACTIVE]").size(11.0).color(Color32::from_rgb(0, 230, 140)));
            #[cfg(not(windows))]
            ui.label(egui::RichText::new("Linux Audio Subsystem: Realtime Ready").size(11.0).color(Color32::from_rgb(0, 230, 140)));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("v2.4.4-rev1").size(10.0).color(Color32::GRAY));
            });
        });
    }
}

fn render_nav_tab(ui: &mut Ui, current: &mut ActiveTab, tab: ActiveTab, label: &str) {
    let is_selected = *current == tab;
    let text = egui::RichText::new(label)
        .size(12.0)
        .color(if is_selected {
            Color32::from_rgb(0, 230, 255)
        } else {
            Color32::from_rgb(170, 180, 200)
        })
        .strong();

    let btn = egui::Button::new(text).fill(if is_selected {
        Color32::from_rgb(28, 36, 48)
    } else {
        Color32::from_rgb(16, 20, 26)
    });

    if ui.add(btn).clicked() {
        *current = tab;
    }
}
