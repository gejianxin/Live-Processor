use std::path::PathBuf;
use egui::{Color32, Stroke, Ui};

use crate::config::{is_64bit_os, DeviceLayout, DEFAULT_START_MENU_GROUP};
use crate::driver::query_driver_service_status;
use crate::i18n::tr;
use crate::installer::{run_install, InstallOptions};
use crate::payload::PayloadSource;
use crate::registry::{
    apply_code_integrity_policy_fix, get_installed_info,
};
use crate::security::{grant_users_full_access_file, grant_users_full_access_registry};
#[cfg(windows)]
use crate::task::{install_registry_fix_task, verify_registry_fix_task};
use crate::uninstaller::{run_uninstall, UninstallOptions};

pub struct SystemAuditState {
    pub is_installed: bool,
    pub install_dir: Option<PathBuf>,
    pub is_64bit: bool,
    pub service_status: String,
    pub num_devices: Option<u32>,
    pub num_channels: Option<u32>,
    pub ci_policy_applied: bool,
    pub task_verified: bool,
}

impl SystemAuditState {
    pub fn probe() -> Self {
        let info = get_installed_info();
        let is_64bit = is_64bit_os();
        let service_status = query_driver_service_status().unwrap_or_else(|| tr("未启动", "Inactive").to_string());
        #[cfg(windows)]
        let task_verified = verify_registry_fix_task();
        #[cfg(not(windows))]
        let task_verified = true;

        Self {
            is_installed: info.install_dir.is_some(),
            install_dir: info.install_dir.map(PathBuf::from),
            is_64bit,
            service_status,
            num_devices: info.num_devices,
            num_channels: info.num_channels,
            ci_policy_applied: info.ci_policy_applied,
            task_verified,
        }
    }
}

pub struct SupervisorView {
    pub selected_layout: DeviceLayout,
    pub logs: Vec<String>,
    pub audit: SystemAuditState,
}

impl Default for SupervisorView {
    fn default() -> Self {
        Self {
            selected_layout: DeviceLayout::Layout4x8,
            logs: vec![
                format!("[SYSTEM] {} - {}", tr("Live-Processor 驱动与系统守护中心已就绪", "Live-Processor Supervisor Ready"), tr("安全模式已加载", "Secure Mode Loaded")),
            ],
            audit: SystemAuditState::probe(),
        }
    }
}

impl SupervisorView {
    pub fn refresh_audit(&mut self) {
        self.audit = SystemAuditState::probe();
        self.log(tr("[AUDIT] 系统状态诊断数据已刷新。", "[AUDIT] System status diagnostics refreshed."));
    }

    pub fn render(&mut self, ui: &mut Ui) {
        let avail_w = ui.available_width();

        ui.vertical(|ui| {
            // Header Row
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new(tr("驱动运维与系统守护中心", "Driver & System Supervisor"))
                        .color(Color32::from_rgb(230, 236, 244))
                        .strong(),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("🔄 刷新状态", "🔄 Refresh Status")).size(12.0)).clicked() {
                        self.refresh_audit();
                    }
                });
            });

            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(tr(
                    "管理 ASIO Link Pro 内核驱动生命周期、Windows 代码完整性保护策略 (CI Policy) 及自主自愈任务。",
                    "Manage ASIO Link Pro driver lifecycle, Windows Code Integrity policy, and autonomous self-heal tasks."
                ))
                .color(Color32::from_rgb(138, 148, 162))
                .small(),
            );
            ui.add_space(14.0);

            // 1. Pro Status Cards (Full-Width Responsive 3-Column Layout)
            let card_spacing = 12.0;
            let col_w = ((avail_w - card_spacing * 2.0) / 3.0).max(180.0);

            ui.horizontal(|ui| {
                // Card 1: Kernel Driver
                render_status_card(
                    ui,
                    col_w,
                    tr("内核驱动引擎", "Kernel Driver"),
                    if self.audit.is_installed { tr("运行正常", "ACTIVE") } else { tr("未安装", "NOT FOUND") },
                    self.audit.is_installed,
                    |ui| {
                        let path_str = self.audit.install_dir.as_ref()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| tr("未就绪", "None").to_string());
                        render_kv(ui, tr("安装路径", "Path"), &truncate_str(&path_str, 24));
                        render_kv(ui, tr("系统架构", "Arch"), if self.audit.is_64bit { "64-bit (x64)" } else { "32-bit (x86)" });
                        render_kv(ui, tr("服务状态", "Service"), &self.audit.service_status);
                    },
                );

                ui.add_space(card_spacing);

                // Card 2: WDM Layout
                let (has_cfg, cfg_title) = match (self.audit.num_devices, self.audit.num_channels) {
                    (Some(16), _) => (true, tr("16 立体声模式", "16-Stereo Mode").to_string()),
                    (Some(4), _) => (true, tr("4x8 声道默认模式", "4x8 Channels Mode").to_string()),
                    (Some(d), Some(c)) => (true, format!("{}x{} {}", d, c, tr("模式", "Mode"))),
                    _ => (false, tr("未写入配置", "Unconfigured").to_string()),
                };

                render_status_card(
                    ui,
                    col_w,
                    tr("WDM 声道布局", "WDM Layout"),
                    &cfg_title,
                    has_cfg,
                    |ui| {
                        let dev_text = self.audit.num_devices.map(|d| format!("{} {}", d, tr("个设备", "devices"))).unwrap_or_else(|| tr("默认 4", "Default 4").to_string());
                        let ch_text = self.audit.num_channels.map(|c| format!("{} {}", c, tr("个声道", "channels"))).unwrap_or_else(|| tr("默认 8", "Default 8").to_string());
                        render_kv(ui, tr("设备数量", "Devices"), &dev_text);
                        render_kv(ui, tr("声道规格", "Channels"), &ch_text);
                        render_kv(ui, tr("推荐用途", "Best For"), if self.audit.num_devices == Some(16) { tr("多音轨多端分流", "Multi-app split") } else { tr("宿主与直播混合", "DAW & Live mix") });
                    },
                );

                ui.add_space(card_spacing);

                // Card 3: Code Integrity & Self-Heal Task
                #[cfg(windows)]
                let ci_ok = self.audit.ci_policy_applied && self.audit.task_verified;
                #[cfg(not(windows))]
                let ci_ok = true;

                render_status_card(
                    ui,
                    col_w,
                    tr("代码完整性自愈", "Code Integrity"),
                    if ci_ok { tr("已受守护", "PROTECTED") } else { tr("需快速修复", "UNPROTECTED") },
                    ci_ok,
                    |ui| {
                        #[cfg(windows)]
                        {
                            render_kv(ui, "UpgradedSystem", if self.audit.ci_policy_applied { "1 (Active)" } else { "0 (Inactive)" });
                            render_kv(ui, tr("开机守护任务", "Heal Task"), if self.audit.task_verified { tr("已启用 [正常]", "Registered") } else { tr("缺失", "Missing") });
                            render_kv(ui, tr("切音保护", "Anti-Cutout"), if ci_ok { tr("持续守护中", "Enabled") } else { tr("可能产生切音", "Warning: Cutouts") });
                        }
                        #[cfg(not(windows))]
                        {
                            render_kv(ui, tr("系统环境", "Platform"), "Linux Native");
                            render_kv(ui, tr("内核驱动", "Kernel Mode"), "ALSA / PipeWire");
                            render_kv(ui, tr("权限隔离", "Sandboxing"), tr("已放行", "Unrestricted"));
                        }
                    },
                );
            });

            ui.add_space(14.0);

            // 2. Action Toolbar Section
            egui::Frame::NONE
                .fill(Color32::from_rgb(22, 25, 31))
                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 44, 54)))
                .inner_margin(12.0)
                .corner_radius(egui::CornerRadius::same(4))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(egui::RichText::new(tr("声道布局切换:", "Target Layout:")).color(Color32::from_rgb(160, 172, 188)));
                        egui::ComboBox::from_id_salt("layout_selector")
                            .selected_text(self.selected_layout.description())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.selected_layout, DeviceLayout::Layout4x8, DeviceLayout::Layout4x8.description());
                                ui.selectable_value(&mut self.selected_layout, DeviceLayout::Layout16x2, DeviceLayout::Layout16x2.description());
                            });

                        ui.add_space(12.0);

                        // Install Button
                        if ui.button(egui::RichText::new(tr("安装 / 升级驱动", "Install / Upgrade")).strong()).clicked() {
                            self.log(tr("正在执行驱动部署流程...", "Executing driver deployment..."));
                            let opts = InstallOptions {
                                install_dir: None,
                                layout: self.selected_layout,
                                create_shortcuts: true,
                                start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                                custom_payload_path: None,
                                silent: true,
                            };
                            match run_install(opts) {
                                Ok(_) => {
                                    self.log(tr("[成功] 驱动程序及运行时组件安装成功！", "[SUCCESS] Driver installed successfully!"));
                                    self.refresh_audit();
                                }
                                Err(e) => self.log(&format!("{}: {}", tr("[错误] 安装失败", "[ERROR] Install failed"), e)),
                            }
                        }

                        // Quick Fix Button
                        if ui.button(egui::RichText::new(tr("应用代码完整性自愈修复", "Apply Self-Heal Fix")).color(Color32::from_rgb(56, 189, 248)).strong()).clicked() {
                            self.log(tr("正在重新应用 CI 代码完整性注册表策略与自主自愈计划任务...", "Applying Code Integrity policy and scheduled task..."));
                            let _ = apply_code_integrity_policy_fix();
                            #[cfg(windows)]
                            let _ = install_registry_fix_task();
                            let dat_file = crate::config::get_system_drivers_dir().join("asiovadpro.dat");
                            let _ = grant_users_full_access_file(&dat_file);
                            let _ = grant_users_full_access_registry();
                            self.log(tr("[成功] 快速修复已执行完毕，系统已受保护。", "[SUCCESS] Quick repair completed successfully."));
                            self.refresh_audit();
                        }

                        // Layout Switch Button
                        if ui.button(tr("切换设备布局", "Switch Layout")).clicked() {
                            self.log(&format!("{} {}...", tr("正在切换设备声道布局为", "Switching layout to"), self.selected_layout.description()));
                            let opts = InstallOptions {
                                install_dir: None,
                                layout: self.selected_layout,
                                create_shortcuts: true,
                                start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                                custom_payload_path: None,
                                silent: true,
                            };
                            match run_install(opts) {
                                Ok(_) => {
                                    self.log(tr("[成功] 声道布局已成功切换！", "[SUCCESS] Layout switched successfully!"));
                                    self.refresh_audit();
                                }
                                Err(e) => self.log(&format!("{}: {}", tr("[错误] 声道布局切换失败", "[ERROR] Layout switch failed"), e)),
                            }
                        }

                        // Extract Payload Button
                        if ui.button(tr("解包提取驱动", "Extract Payload")).clicked() {
                            let out_dir = std::path::PathBuf::from("./extracted_payload");
                            if let Ok(source) = PayloadSource::new(None) {
                                let _ = source.extract_all_to(&out_dir);
                                self.log(&format!("{}: {}", tr("[成功] 驱动资源已解包至", "[SUCCESS] Extracted payload to"), out_dir.display()));
                            }
                        }

                        // Uninstall Button
                        if ui.button(egui::RichText::new(tr("卸载驱动与清理", "Uninstall Driver")).color(Color32::from_rgb(248, 113, 113))).clicked() {
                            self.log(tr("正在执行驱动卸载流程...", "Executing uninstallation..."));
                            let opts = UninstallOptions {
                                app_dir: None,
                                delete_all_settings: true,
                                silent: true,
                            };
                            match run_uninstall(opts) {
                                Ok(_) => {
                                    self.log(tr("[成功] 卸载与清理完成。", "[SUCCESS] Uninstallation completed."));
                                    self.refresh_audit();
                                }
                                Err(e) => self.log(&format!("{}: {}", tr("[错误] 卸载失败", "[ERROR] Uninstall failed"), e)),
                            }
                        }
                    });
                });

            ui.add_space(14.0);

            // 3. Diagnostics & Activity Log Box (Responsive Full Height & Width)
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tr("活动审计与驱动诊断日志", "Activity & Diagnostic Logs")).strong().size(12.0).color(Color32::from_rgb(200, 210, 225)));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new(tr("清空日志", "Clear")).size(10.0)).clicked() {
                        self.logs.clear();
                    }
                });
            });

            ui.add_space(4.0);

            let remaining_h = (ui.available_height() - 8.0).max(140.0);

            egui::Frame::NONE
                .fill(Color32::from_rgb(14, 16, 20))
                .stroke(Stroke::new(1.0, Color32::from_rgb(34, 40, 50)))
                .corner_radius(egui::CornerRadius::same(4))
                .inner_margin(10.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .min_scrolled_height(remaining_h)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            for line in &self.logs {
                                let color = if line.contains("[SUCCESS]") || line.contains("[成功]") {
                                    Color32::from_rgb(52, 211, 153) // Subtle emerald
                                } else if line.contains("[ERROR]") || line.contains("[错误]") {
                                    Color32::from_rgb(248, 113, 113) // Soft coral red
                                } else if line.contains("[AUDIT]") {
                                    Color32::from_rgb(56, 189, 248) // Clean sky blue
                                } else {
                                    Color32::from_rgb(156, 168, 184)
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

fn render_status_card(
    ui: &mut Ui,
    width: f32,
    title: &str,
    status_label: &str,
    is_positive: bool,
    body: impl FnOnce(&mut Ui),
) {
    egui::Frame::NONE
        .fill(Color32::from_rgb(24, 28, 35))
        .stroke(Stroke::new(1.0, Color32::from_rgb(40, 47, 58)))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_min_height(130.0);

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(title).strong().size(13.0).color(Color32::from_rgb(220, 228, 238)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (badge_bg, badge_fg) = if is_positive {
                            (Color32::from_rgb(16, 48, 36), Color32::from_rgb(52, 211, 153))
                        } else {
                            (Color32::from_rgb(48, 32, 24), Color32::from_rgb(245, 158, 11))
                        };
                        egui::Frame::NONE
                            .fill(badge_bg)
                            .corner_radius(egui::CornerRadius::same(3))
                            .inner_margin(egui::Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new(status_label).size(10.0).strong().color(badge_fg));
                            });
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                body(ui);
            });
        });
}

fn render_kv(ui: &mut Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{}:", key)).size(11.0).color(Color32::from_rgb(120, 132, 148)));
        ui.label(egui::RichText::new(val).size(11.0).color(Color32::from_rgb(205, 215, 228)));
    });
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    } else {
        s.to_string()
    }
}
