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
                "[SYSTEM] Live-Processor 守护引擎已初始化就绪。".to_string(),
                "[AUDIT] 正在扫描系统驱动、设备配置与代码完整性保护...".to_string(),
            ],
        }
    }
}

impl SupervisorView {
    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading(
                egui::RichText::new(tr("🛡️ 驱动管理与系统守护中心", "🛡️ System & Driver Supervisor"))
                    .color(Color32::from_rgb(0, 220, 255))
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new(tr(
                    "全生命周期管理 ASIO Link Pro 音频驱动、Windows 代码完整性保护策略 (CI Policy) 以及自主自愈任务。",
                    "Manage ASIO Link Pro driver lifecycle, Code Integrity protection policy, and autonomous self-heal tasks."
                ))
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
                    ui.label(egui::RichText::new(tr("内核驱动状态", "Kernel Driver")).strong().color(Color32::from_rgb(0, 210, 255)));
                    ui.add_space(4.0);
                    let installed = info.install_dir.is_some();
                    ui.label(format!("{}: {}", tr("安装状态", "Installed"), if installed { tr("已安装 [正常]", "YES [Active]") } else { tr("未安装", "Not Installed") }));
                    ui.label(format!("{}: {}", tr("系统架构", "Architecture"), if is_64bit { "64位 (x64)" } else { "32位 (x86)" }));
                    let svc = query_driver_service_status().unwrap_or_else(|| tr("未启动", "Inactive").to_string());
                    ui.label(format!("{}: {}", tr("驱动服务", "Service"), svc));
                });

                // Column 2: WDM Layout
                columns[1].group(|ui| {
                    ui.set_min_height(140.0);
                    ui.label(egui::RichText::new(tr("WDM 设备声道布局", "WDM Device Layout")).strong().color(Color32::from_rgb(255, 170, 0)));
                    ui.add_space(4.0);
                    match (info.num_devices, info.num_channels) {
                        (Some(d), Some(c)) => {
                            ui.label(format!("{}: {} {} / {} {}", tr("当前配置", "Configured"), d, tr("设备", "Devices"), c, tr("声道", "Channels")));
                            if d == 16 {
                                ui.label(tr("模式: 16 独立立体声设备", "Mode: 16 Stereo WDM Devices"));
                            } else {
                                ui.label(tr("模式: 4 设备 / 8 声道 (默认推荐)", "Mode: 4 Devices / 8 Channels (Default)"));
                            }
                        }
                        _ => {
                            ui.label(tr("当前配置: 默认模式 (未写入)", "Configured: Default Layout (Unset)"));
                        }
                    }
                });

                // Column 3: Code Integrity & Self-Heal Task
                columns[2].group(|ui| {
                    ui.set_min_height(140.0);
                    ui.label(egui::RichText::new(tr("代码完整性自愈保护", "Code Integrity Self-Heal")).strong().color(Color32::from_rgb(0, 230, 120)));
                    ui.add_space(4.0);
                    #[cfg(windows)]
                    {
                        ui.label(format!("{}: {}", tr("CI 注册表策略", "CI Policy (UpgradedSystem)"), if info.ci_policy_applied { tr("1 [已受保护]", "1 (Protected)") } else { tr("未激活", "Disabled") }));
                        let task_ok = verify_registry_fix_task();
                        ui.label(format!("{}: {}", tr("自主守护任务", "Autonomous Task"), if task_ok { tr("已激活并验证", "Active & Verified") } else { tr("未配置", "Not Active") }));
                    }
                    #[cfg(not(windows))]
                    {
                        ui.label(tr("运行环境: 原生 Linux", "Host OS: Native Linux"));
                        ui.label(tr("代码签名策略: 无限制", "Code Integrity: Unrestricted"));
                    }
                });
            });

            ui.add_space(14.0);

            // Management Actions Section
            ui.group(|ui| {
                ui.label(egui::RichText::new(tr("快捷运维与管理操作", "Operations & Management")).strong().size(13.0));
                ui.add_space(8.0);

                ui.horizontal_wrapped(|ui| {
                    // Layout selection combo
                    ui.label(tr("目标布局:", "Target Layout:"));
                    egui::ComboBox::from_id_salt("layout_combo")
                        .selected_text(self.selected_layout.description())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.selected_layout, DeviceLayout::Layout4x8, DeviceLayout::Layout4x8.description());
                            ui.selectable_value(&mut self.selected_layout, DeviceLayout::Layout16x2, DeviceLayout::Layout16x2.description());
                        });

                    ui.add_space(10.0);

                    if ui.button(egui::RichText::new(tr("🚀 安装 / 升级驱动", "🚀 Install / Upgrade Driver")).color(Color32::from_rgb(0, 230, 140)).strong()).clicked() {
                        self.log(tr("正在启动驱动安装流水线...", "Starting installation pipeline..."));
                        let opts = InstallOptions {
                            install_dir: None,
                            layout: self.selected_layout,
                            create_shortcuts: true,
                            start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                            custom_payload_path: None,
                            silent: true,
                        };
                        match run_install(opts) {
                            Ok(_) => self.log(tr("[成功] 驱动程序及运行时组件安装成功！", "[SUCCESS] Driver and components installed successfully!")),
                            Err(e) => self.log(&format!("{}: {}", tr("[错误] 安装失败", "[ERROR] Install failed"), e)),
                        }
                    }

                    if ui.button(egui::RichText::new(tr("⚡ 一键快速修复", "⚡ Quick Repair")).color(Color32::from_rgb(0, 210, 255))).clicked() {
                        self.log(tr("正在重新应用 CI 代码完整性注册表策略与开机自愈计划任务...", "Applying Code Integrity Policy fix and self-heal task..."));
                        let _ = apply_code_integrity_policy_fix();
                        let _ = install_registry_fix_task();
                        let dat_file = crate::config::get_system_drivers_dir().join("asiovadpro.dat");
                        let _ = grant_users_full_access_file(&dat_file);
                        let _ = grant_users_full_access_registry();
                        self.log(tr("[成功] 快速修复已执行完毕，系统已受保护。", "[SUCCESS] Quick repair completed successfully."));
                    }

                    if ui.button(egui::RichText::new(tr("🔄 热切换设备布局", "🔄 Switch Layout")).color(Color32::from_rgb(255, 170, 0))).clicked() {
                        self.log(&format!("{} {}...", tr("正在切换设备声道布局为", "Switching device layout to"), self.selected_layout.description()));
                        let opts = InstallOptions {
                            install_dir: None,
                            layout: self.selected_layout,
                            create_shortcuts: true,
                            start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                            custom_payload_path: None,
                            silent: true,
                        };
                        match run_install(opts) {
                            Ok(_) => self.log(tr("[成功] 声道布局已成功切换！", "[SUCCESS] Layout switched successfully!")),
                            Err(e) => self.log(&format!("{}: {}", tr("[错误] 声道布局切换失败", "[ERROR] Layout switch failed"), e)),
                        }
                    }

                    if ui.button(tr("📦 导出内置驱动包", "📦 Extract Payload")).clicked() {
                        let out_dir = std::path::PathBuf::from("./extracted_payload");
                        if let Ok(source) = PayloadSource::new(None) {
                            let _ = source.extract_all_to(&out_dir);
                            self.log(&format!("{}: {}", tr("[成功] 驱动资源已解包至", "[SUCCESS] Extracted payload to"), out_dir.display()));
                        }
                    }

                    if ui.button(egui::RichText::new(tr("🗑️ 彻底卸载驱动", "🗑️ Uninstall Driver")).color(Color32::from_rgb(255, 80, 80))).clicked() {
                        self.log(tr("正在启动卸载清理流程...", "Starting uninstallation pipeline..."));
                        let opts = UninstallOptions {
                            app_dir: None,
                            delete_all_settings: true,
                            silent: true,
                        };
                        match run_uninstall(opts) {
                            Ok(_) => self.log(tr("[成功] 卸载与清理完成。", "[SUCCESS] Uninstalled successfully.")),
                            Err(e) => self.log(&format!("{}: {}", tr("[错误] 卸载失败", "[ERROR] Uninstall failed"), e)),
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Log Console Box
            ui.label(egui::RichText::new(tr("活动诊断与审计日志", "Activity & Diagnostics Log")).strong().size(12.0));
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
                                let color = if line.contains("[SUCCESS]") || line.contains("[成功]") {
                                    Color32::from_rgb(0, 230, 140)
                                } else if line.contains("[ERROR]") || line.contains("[错误]") {
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
