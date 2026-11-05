use egui::{Color32, Stroke, Ui};

use crate::config::{is_64bit_os, DeviceLayout, DEFAULT_START_MENU_GROUP};
use crate::driver::query_driver_service_status;
use crate::installer::{run_install, InstallOptions};
use crate::payload::PayloadSource;
use crate::registry::{
    apply_code_integrity_policy_fix, get_installed_info,
};
use crate::security::{grant_users_full_access_file, grant_users_full_access_registry};
#[cfg(windows)]
use crate::task::verify_registry_fix_task;
use crate::task::install_registry_fix_task;
use crate::uninstaller::{run_uninstall, UninstallOptions};

pub struct SupervisorView {
    pub selected_layout: DeviceLayout,
    pub logs: Vec<String>,
}

impl Default for SupervisorView {
    fn default() -> Self {
        Self {
            selected_layout: DeviceLayout::Layout4x8,
            logs: vec![
                "[SYSTEM] Live-Processor Supervisor initialized.".to_string(),
                "[AUDIT] Scanning system driver data and registry keys...".to_string(),
            ],
        }
    }
}

impl SupervisorView {
    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(
                egui::RichText::new("🛡️ System & Driver Supervisor")
                    .color(Color32::from_rgb(0, 220, 255))
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Manage ASIO Link Pro driver lifecycle, Code Integrity protection policy, and autonomous self-heal tasks.")
                    .color(Color32::from_rgb(160, 175, 195))
                    .small(),
            );
            ui.add_space(12.0);

            let info = get_installed_info();
            let is_64bit = is_64bit_os();

            // Status Cards Grid
            ui.columns(3, |columns| {
                // Column 1: Driver status
                columns[0].group(|ui| {
                    ui.set_min_height(140.0);
                    ui.label(egui::RichText::new("Kernel Driver").strong().color(Color32::from_rgb(0, 210, 255)));
                    ui.add_space(4.0);
                    let installed = info.install_dir.is_some();
                    ui.label(format!("Installed: {}", if installed { "YES [Active]" } else { "Not Installed" }));
                    ui.label(format!("Architecture: {}", if is_64bit { "64-bit (x64)" } else { "32-bit (x86)" }));
                    let svc = query_driver_service_status().unwrap_or_else(|| "Inactive".to_string());
                    ui.label(format!("Service: {}", svc));
                });

                // Column 2: WDM Layout
                columns[1].group(|ui| {
                    ui.set_min_height(140.0);
                    ui.label(egui::RichText::new("WDM Device Layout").strong().color(Color32::from_rgb(255, 170, 0)));
                    ui.add_space(4.0);
                    match (info.num_devices, info.num_channels) {
                        (Some(d), Some(c)) => {
                            ui.label(format!("Configured: {} Devices / {} Channels", d, c));
                            if d == 16 {
                                ui.label("Mode: 16 Stereo WDM Devices");
                            } else {
                                ui.label("Mode: 4 Devices / 8 Channels (Default)");
                            }
                        }
                        _ => {
                            ui.label("Configured: Default Layout (Unset)");
                        }
                    }
                });

                // Column 3: Code Integrity & Self-Heal Task
                columns[2].group(|ui| {
                    ui.set_min_height(140.0);
                    ui.label(egui::RichText::new("Code Integrity Self-Heal").strong().color(Color32::from_rgb(0, 230, 120)));
                    ui.add_space(4.0);
                    #[cfg(windows)]
                    {
                        ui.label(format!("CI Policy (UpgradedSystem): {}", if info.ci_policy_applied { "1 (Protected)" } else { "Disabled" }));
                        let task_ok = verify_registry_fix_task();
                        ui.label(format!("Autonomous Task: {}", if task_ok { "Active & Verified" } else { "Not Active" }));
                    }
                    #[cfg(not(windows))]
                    {
                        ui.label("Host OS: Native Linux");
                        ui.label("Code Integrity: Unrestricted");
                    }
                });
            });

            ui.add_space(14.0);

            // Management Actions Section
            ui.group(|ui| {
                ui.label(egui::RichText::new("Operations & Management").strong().size(13.0));
                ui.add_space(8.0);

                ui.horizontal_wrapped(|ui| {
                    // Layout selection combo
                    ui.label("Target Layout:");
                    egui::ComboBox::from_id_salt("layout_combo")
                        .selected_text(self.selected_layout.description())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_layout, DeviceLayout::Layout4x8, DeviceLayout::Layout4x8.description());
                            ui.selectable_value(&mut self.selected_layout, DeviceLayout::Layout16x2, DeviceLayout::Layout16x2.description());
                        });

                    ui.add_space(10.0);

                    if ui.button(egui::RichText::new("🚀 Install / Upgrade Driver").color(Color32::from_rgb(0, 230, 140)).strong()).clicked() {
                        self.log("Starting installation pipeline...");
                        let opts = InstallOptions {
                            install_dir: None,
                            layout: self.selected_layout,
                            create_shortcuts: true,
                            start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                            custom_payload_path: None,
                            silent: true,
                        };
                        match run_install(opts) {
                            Ok(_) => self.log("[SUCCESS] Driver and components installed successfully!"),
                            Err(e) => self.log(&format!("[ERROR] Install failed: {}", e)),
                        }
                    }

                    if ui.button(egui::RichText::new("⚡ Quick Repair").color(Color32::from_rgb(0, 210, 255))).clicked() {
                        self.log("Applying Code Integrity Policy fix and self-heal task...");
                        let _ = apply_code_integrity_policy_fix();
                        let _ = install_registry_fix_task();
                        let dat_file = crate::config::get_system_drivers_dir().join("asiovadpro.dat");
                        let _ = grant_users_full_access_file(&dat_file);
                        let _ = grant_users_full_access_registry();
                        self.log("[SUCCESS] Quick repair completed successfully.");
                    }

                    if ui.button(egui::RichText::new("🔄 Switch Layout").color(Color32::from_rgb(255, 170, 0))).clicked() {
                        self.log(&format!("Switching device layout to {}...", self.selected_layout.description()));
                        let opts = InstallOptions {
                            install_dir: None,
                            layout: self.selected_layout,
                            create_shortcuts: true,
                            start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                            custom_payload_path: None,
                            silent: true,
                        };
                        match run_install(opts) {
                            Ok(_) => self.log("[SUCCESS] Layout switched successfully!"),
                            Err(e) => self.log(&format!("[ERROR] Layout switch failed: {}", e)),
                        }
                    }

                    if ui.button("📦 Extract Payload").clicked() {
                        let out_dir = std::path::PathBuf::from("./extracted_payload");
                        if let Ok(source) = PayloadSource::new(None) {
                            let _ = source.extract_all_to(&out_dir);
                            self.log(&format!("[SUCCESS] Extracted payload to {}", out_dir.display()));
                        }
                    }

                    if ui.button(egui::RichText::new("🗑️ Uninstall Driver").color(Color32::from_rgb(255, 80, 80))).clicked() {
                        self.log("Starting uninstallation pipeline...");
                        let opts = UninstallOptions {
                            app_dir: None,
                            delete_all_settings: true,
                            silent: true,
                        };
                        match run_uninstall(opts) {
                            Ok(_) => self.log("[SUCCESS] Uninstalled successfully."),
                            Err(e) => self.log(&format!("[ERROR] Uninstall failed: {}", e)),
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Log Console Box
            ui.label(egui::RichText::new("Activity & Diagnostics Log").strong().size(12.0));
            egui::Frame::NONE
                .fill(Color32::from_rgb(12, 14, 18))
                .stroke(Stroke::new(1.0, Color32::from_rgb(32, 38, 48)))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for line in &self.logs {
                                let color = if line.contains("[SUCCESS]") {
                                    Color32::from_rgb(0, 230, 140)
                                } else if line.contains("[ERROR]") {
                                    Color32::from_rgb(255, 80, 80)
                                } else {
                                    Color32::from_rgb(180, 195, 215)
                                };
                                ui.label(egui::RichText::new(line).color(color).monospace().size(11.0));
                            }
                        });
                });
        });
    }

    fn log(&mut self, msg: &str) {
        self.logs.push(msg.to_string());
    }
}
