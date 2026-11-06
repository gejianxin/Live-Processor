use std::sync::atomic::Ordering;
use egui::{Color32, Stroke, Ui};

use super::live_processor::LiveProcessorView;
use super::matrix::RoutingMatrix;
use super::rack::VirtualRack;
use super::snapshots::SnapshotManager;
use super::supervisor::SupervisorView;
use crate::audio::AudioEngine;
#[cfg(windows)]
use crate::config::is_64bit_os;
use crate::i18n::{tr, Language};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ActiveTab {
    LiveProcessor,
    Matrix,
    Rack,
    Supervisor,
    Snapshots,
}

pub struct LiveProcessorApp {
    pub active_tab: ActiveTab,
    pub live_processor: LiveProcessorView,
    pub matrix: RoutingMatrix,
    pub rack: VirtualRack,
    pub supervisor: SupervisorView,
    pub snapshots: SnapshotManager,
    pub audio_engine: AudioEngine,
    pub master_volume: f32,
    pub panic_muted: bool,
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub has_centered: bool,
}

impl LiveProcessorApp {
    pub fn new() -> Self {
        let audio_engine = AudioEngine::new();
        audio_engine.start();
        let state = audio_engine.state.clone();

        Self {
            active_tab: ActiveTab::LiveProcessor,
            live_processor: LiveProcessorView::new(),
            matrix: RoutingMatrix::new(state.clone()),
            rack: VirtualRack::new(state.clone()),
            supervisor: SupervisorView::default(),
            snapshots: SnapshotManager::new(state.clone()),
            audio_engine,
            master_volume: 0.0,
            panic_muted: false,
            sample_rate: 48000,
            buffer_size: 256,
            has_centered: false,
        }
    }
}

impl eframe::App for LiveProcessorApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // Automatically center the window on screen at startup
        if !self.has_centered {
            if let Some(cmd) = egui::ViewportCommand::center_on_screen(ui.ctx()) {
                ui.ctx().send_viewport_cmd(cmd);
                self.has_centered = true;
            }
        }

        let time = ui.ctx().input(|i| i.time);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(30));

        // 1. Top Control Header
        egui::Frame::NONE
            .fill(Color32::from_rgb(14, 21, 30))
            .stroke(Stroke::new(1.0, Color32::from_rgb(30, 42, 60)))
            .inner_margin(egui::Margin::symmetric(14, 10))
            .show(ui, |ui| {
                self.render_header(ui, time);
            });

        ui.add_space(2.0);

        // 2. Central Full-Screen Workspace Area
        let available_space = (ui.available_height() - 34.0).max(200.0);
        egui::Frame::NONE
            .fill(Color32::from_rgb(14, 21, 30))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.set_min_height(available_space);
                match self.active_tab {
                    ActiveTab::LiveProcessor => self.live_processor.render(ui, time),
                    ActiveTab::Matrix => self.matrix.render(ui, time),
                    ActiveTab::Rack => self.rack.render(ui, time),
                    ActiveTab::Supervisor => self.supervisor.render(ui),
                    ActiveTab::Snapshots => self.snapshots.render(ui),
                }
            });

        ui.add_space(2.0);

        // 3. Bottom Status Bar
        egui::Frame::NONE
            .fill(Color32::from_rgb(11, 17, 24))
            .stroke(Stroke::new(1.0, Color32::from_rgb(26, 36, 50)))
            .inner_margin(egui::Margin::symmetric(14, 6))
            .show(ui, |ui| {
                self.render_status_bar(ui);
            });
    }
}

impl LiveProcessorApp {
    fn render_header(&mut self, ui: &mut Ui, time: f64) {
        ui.horizontal(|ui| {
            // Heartbeat pulse dot & Brand Logo
            let pulse = ((time * 3.0).sin() * 0.15 + 0.85) as f32;
            let dot_color = if self.panic_muted {
                Color32::from_rgb(248, 113, 113)
            } else {
                Color32::from_rgb(
                    (56.0 * pulse) as u8,
                    (189.0 * pulse) as u8,
                    (248.0 * pulse) as u8,
                )
            };

            ui.colored_label(dot_color, "●");
            ui.label(
                egui::RichText::new("LIVE-PROCESSOR")
                    .strong()
                    .size(13.5)
                    .color(Color32::from_rgb(235, 240, 248)),
            );
            egui::Frame::NONE
                .fill(Color32::from_rgb(28, 36, 48))
                .corner_radius(egui::CornerRadius::same(3))
                .inner_margin(egui::Margin::symmetric(4, 1))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("PRO")
                            .size(9.0)
                            .strong()
                            .color(Color32::from_rgb(56, 189, 248)),
                    );
                });

            ui.add_space(14.0);

            // Tab Navigation Switcher (Studio console style)
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::LiveProcessor, tr("推流主控", "Live Stream"));
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Matrix, tr("跳线矩阵", "Wire Matrix"));
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Rack, tr("虚拟机架", "Virtual Rack"));
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Supervisor, tr("驱动与守护", "Supervisor"));
            render_nav_tab(ui, &mut self.active_tab, ActiveTab::Snapshots, tr("现场快照", "Show Cues"));

            // Right-aligned audio controls with guaranteed padding
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(16.0); // Generous safety right margin to guarantee zero clipping

                // Language Switcher Button
                let lang_label = match Language::current() {
                    Language::Chinese => "🌐 简体中文",
                    Language::English => "🌐 English",
                };
                if ui.button(egui::RichText::new(lang_label).size(10.5).color(Color32::from_rgb(180, 192, 210))).clicked() {
                    Language::toggle();
                }

                ui.add_space(8.0);

                // Panic Button
                let panic_text = if self.panic_muted {
                    tr("已紧急静音", "MUTED")
                } else {
                    tr("紧急静音", "PANIC")
                };
                let panic_btn = egui::Button::new(
                    egui::RichText::new(panic_text)
                        .strong()
                        .size(10.5)
                        .color(if self.panic_muted { Color32::WHITE } else { Color32::from_rgb(248, 113, 113) }),
                )
                .fill(if self.panic_muted {
                    Color32::from_rgb(185, 28, 28)
                } else {
                    Color32::from_rgb(36, 26, 28)
                })
                .stroke(Stroke::new(1.0, if self.panic_muted { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(76, 36, 40) }));

                if ui.add(panic_btn).clicked() {
                    self.panic_muted = !self.panic_muted;
                    self.audio_engine.state.panic_muted.store(self.panic_muted, Ordering::Relaxed);
                }

                ui.add_space(8.0);

                // Master Volume Slider & dB text
                ui.label(egui::RichText::new(format!("{:+.1} dB", self.master_volume)).size(10.5).monospace().color(Color32::from_rgb(56, 189, 248)));
                if ui.add_sized([50.0, 16.0], egui::Slider::new(&mut self.master_volume, -40.0..=6.0).show_value(false)).changed() {
                    self.audio_engine.state.set_master_volume(self.master_volume);
                }
                ui.label(egui::RichText::new(tr("总音量", "Master")).size(10.5).color(Color32::from_rgb(120, 130, 145)));

                ui.add_space(8.0);

                // Latency & DSP Load Badge (Responsive to available header width)
                let sr = self.audio_engine.state.telemetry.sample_rate.load(Ordering::Relaxed);
                let buf = self.audio_engine.state.telemetry.buffer_size.load(Ordering::Relaxed);
                let lat = self.audio_engine.state.telemetry.get_latency_ms();
                let dsp = self.audio_engine.state.telemetry.get_dsp_load();
                let is_hw = self.audio_engine.state.telemetry.is_hardware_active.load(Ordering::Relaxed);
                let hw_label = if is_hw { "WASAPI/ASIO" } else { "DSP" };

                let avail_for_badge = ui.available_width();
                if avail_for_badge > 50.0 {
                    let badge_text = if avail_for_badge > 260.0 {
                        format!("{} | {} Hz | {} smp (~{:.1} ms) | DSP: {:.0}%", hw_label, sr, buf, lat, dsp)
                    } else if avail_for_badge > 150.0 {
                        format!("{} | {:.1} ms | DSP: {:.0}%", hw_label, lat, dsp)
                    } else {
                        format!("{:.1} ms | {:.0}%", lat, dsp)
                    };
                    ui.label(
                        egui::RichText::new(badge_text)
                            .size(10.0)
                            .color(Color32::from_rgb(52, 211, 153))
                            .monospace(),
                    );
                }
            });
        });
    }

    fn render_status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            #[cfg(windows)]
            let os_text = format!("{}: Windows x64 ({})", tr("平台", "Host"), if is_64bit_os() { tr("64位内核", "64-bit") } else { tr("32位", "32-bit") });
            #[cfg(not(windows))]
            let os_text = format!("{}: Linux (PipeWire / ALSA)", tr("平台", "Host"));

            ui.label(egui::RichText::new(os_text).size(10.5).color(Color32::from_rgb(120, 130, 145)));
            ui.separator();

            ui.label(
                egui::RichText::new(tr("Dialog Audio 集中推流引擎 [CDN / RTMP 多路就绪]", "Dialog Audio Stream Engine [CDN / RTMP Ready]"))
                    .size(10.5)
                    .color(Color32::from_rgb(56, 189, 248)),
            );
            ui.separator();

            ui.label(
                egui::RichText::new(tr("ASIO Link Pro [低延迟驱动就绪]", "ASIO Link Pro [Driver Ready]"))
                    .size(10.5)
                    .color(Color32::from_rgb(175, 188, 205)),
            );
            ui.separator();

            #[cfg(windows)]
            ui.label(egui::RichText::new(tr("CI 安全守护: 激活", "CI Guard: Active")).size(10.5).color(Color32::from_rgb(52, 211, 153)));
            #[cfg(not(windows))]
            ui.label(egui::RichText::new(tr("Linux 实时音频已就绪", "Linux Audio Ready")).size(10.5).color(Color32::from_rgb(52, 211, 153)));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);
                ui.label(egui::RichText::new("Dialog Audio Live-Processor v2.4.4").size(10.0).color(Color32::from_rgb(110, 125, 145)));
            });
        });
    }
}

fn render_nav_tab(ui: &mut Ui, current: &mut ActiveTab, tab: ActiveTab, label: &str) {
    let is_selected = *current == tab;
    let text = egui::RichText::new(label)
        .size(11.5)
        .color(if is_selected {
            Color32::from_rgb(240, 248, 255)
        } else {
            Color32::from_rgb(140, 155, 175)
        })
        .strong();

    let btn = egui::Button::new(text)
        .fill(if is_selected {
            Color32::from_rgb(24, 38, 56)
        } else {
            Color32::from_rgb(18, 24, 34)
        })
        .stroke(Stroke::new(
            1.0,
            if is_selected {
                Color32::from_rgb(56, 189, 248)
            } else {
                Color32::from_rgb(34, 46, 62)
            },
        ))
        .corner_radius(egui::CornerRadius::same(5));

    if ui.add(btn).clicked() {
        *current = tab;
    }
}
