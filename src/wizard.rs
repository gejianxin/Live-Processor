use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::{get_default_install_dir, DeviceLayout, DEFAULT_START_MENU_GROUP};
use crate::elevation::is_elevated;
use crate::i18n::tr;
use crate::installer::{run_install, InstallOptions};
use crate::payload::PayloadSource;
use crate::registry::{
    apply_code_integrity_policy_fix, get_installed_info,
};
use crate::security::{grant_users_full_access_file, grant_users_full_access_registry};
use crate::status::print_status_report;
use crate::task::{install_registry_fix_task, verify_registry_fix_task};
use crate::uninstaller::{run_uninstall, UninstallOptions};

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}

pub fn run_interactive_wizard() -> Result<()> {
    println!("{}", "=======================================================".cyan());
    println!("{}", tr("        Live-Processor CLI - 交互式控制台向导          ", "             Live-Processor - Interactive Hub          ").cyan().bold());
    println!("{}", tr("      现代跨平台 ASIO Link Pro 音频驱动与机架守护系统  ", "   Modern Rust Manager for ASIO Link Pro Audio Driver  ").cyan());
    println!("{}", "=======================================================".cyan());

    if !is_elevated() {
        println!(
            "{}",
            tr(
                "提示: 当前未以管理员权限运行。\n在执行安装、修复或卸载操作时，需要管理员 (Administrator/root) 权限。",
                "Notice: You are not running with Administrator privileges.\nFor installation, repair, and uninstallation, Administrator privileges are required."
            ).yellow()
        );
        println!("{}", "-------------------------------------------------------".cyan());
    }

    loop {
        println!("\n{}", tr("请选择操作:", "Please select an option:"));
        println!("  1) {}", tr("安装 / 升级 Live-Processor", "Install / Upgrade Live-Processor"));
        println!("  2) {}", tr("系统健康与驱动诊断报告", "Diagnostics & Status Report"));
        println!("  3) {}", tr("切换设备声道布局 (4x8 <-> 16x2)", "Switch Device Layout (4x8 <-> 16x2)"));
        println!("  4) {}", tr("一键快速修复 (应用 CI 策略与自愈计划任务)", "Quick Repair (Code Integrity Fix & Scheduled Task)"));
        println!("  5) {}", tr("卸载 Live-Processor 驱动与组件", "Uninstall Live-Processor"));
        println!("  6) {}", tr("导出内置的驱动资源载荷到本地", "Extract Embedded Payload to Disk"));
        println!("  0) {}", tr("退出程序", "Exit"));

        let choice = read_line(&format!("\n{} [0-6]: ", tr("请输入序号", "Enter choice")));
        match choice.as_str() {
            "1" => {
                println!("\n--- {} ---", tr("安装配置", "Install Configuration"));
                println!("{}:", tr("选择音频设备声道布局", "Select Device Layout"));
                println!("  1) {}", tr("4 设备 / 8 声道 (默认推荐，兼容绝大多数宿主)", "4 devices / 8 channels (Default - recommended)"));
                println!("  2) {}", tr("16 独立立体声 WDM 设备 (适合复杂多路跳线路由)", "16 stereo WDM devices (16 devices / 2 channels)"));
                let layout_choice = read_line(&format!("{} [1]: ", tr("选择", "Choice")));
                let layout = if layout_choice == "2" {
                    DeviceLayout::Layout16x2
                } else {
                    DeviceLayout::Layout4x8
                };

                let default_dir = get_default_install_dir();
                println!("{}: {}", tr("默认安装路径", "Default install path"), default_dir.display());
                let dir_input = read_line(&format!("{}: ", tr("回车使用默认路径，或输入自定义路径", "Enter custom path or press Enter for default")));
                let install_dir = if dir_input.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(dir_input))
                };

                let options = InstallOptions {
                    install_dir,
                    layout,
                    create_shortcuts: true,
                    start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                    custom_payload_path: None,
                    silent: false,
                };

                if let Err(e) = run_install(options) {
                    eprintln!("{}", format!("{}: {}", tr("安装失败", "Installation error"), e).red().bold());
                }
            }
            "2" => {
                println!();
                print_status_report();
            }
            "3" => {
                println!("\n--- {} ---", tr("切换设备声道布局", "Switch Device Layout"));
                let info = get_installed_info();
                let app_dir = info
                    .install_dir
                    .map(PathBuf::from)
                    .unwrap_or_else(get_default_install_dir);

                if !app_dir.exists() {
                    eprintln!("{}", tr("错误: 尚未安装 Live-Processor，请先执行安装。", "Error: Live-Processor is not installed. Please install it first.").red());
                    continue;
                }

                println!("{}:", tr("选择目标布局", "Select new layout"));
                println!("  1) 4 设备 / 8 声道 (4x8)");
                println!("  2) 16 独立立体声设备 (16x2)");
                let layout_choice = read_line(&format!("{}: ", tr("选择", "Choice")));
                let new_layout = if layout_choice == "2" {
                    DeviceLayout::Layout16x2
                } else {
                    DeviceLayout::Layout4x8
                };

                println!("{} {}...", tr("正在切换布局为", "Switching layout to"), new_layout.description().cyan());
                let options = InstallOptions {
                    install_dir: Some(app_dir),
                    layout: new_layout,
                    create_shortcuts: true,
                    start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                    custom_payload_path: None,
                    silent: false,
                };

                if let Err(e) = run_install(options) {
                    eprintln!("{}", format!("{}: {}", tr("切换失败", "Layout switch failed"), e).red());
                }
            }
            "4" => {
                println!("\n--- {} ---", tr("一键快速修复", "Quick Repair"));
                println!("1. {}...", tr("应用代码完整性策略 (UpgradedSystem = 1)", "Applying Code Integrity Policy UpgradedSystem = 1"));
                match apply_code_integrity_policy_fix() {
                    Ok(_) => println!("{}", tr("   [成功] 注册表策略已应用。", "   [OK] Registry fix applied.").green()),
                    Err(e) => eprintln!("   [ERROR] {}: {}", tr("应用注册表策略失败", "Failed to apply registry fix"), e),
                }

                println!("2. {}...", tr("配置自主自愈计划任务", "Registering autonomous self-heal scheduled task"));
                match install_registry_fix_task() {
                    Ok(_) => {
                        if verify_registry_fix_task() {
                            println!("{}", tr("   [成功] 计划自愈任务已创建并验证激活。", "   [OK] Scheduled task installed and verified.").green());
                        } else {
                            println!("{}", tr("   [警告] 任务已创建但查询未返回成功。", "   [WARN] Scheduled task installed but verification failed.").yellow());
                        }
                    }
                    Err(e) => eprintln!("   [ERROR] {}: {}", tr("安装计划任务失败", "Failed to install scheduled task"), e),
                }

                println!("3. {}...", tr("重新配置安全 ACL 权限", "Re-applying security ACL permissions"));
                let dat_file = crate::config::get_system_drivers_dir().join("asiovadpro.dat");
                let _ = grant_users_full_access_file(&dat_file);
                let _ = grant_users_full_access_registry();
                println!("{}", tr("   [成功] 安全权限配置完成。", "   [OK] Permissions verified.").green());
            }
            "5" => {
                println!("\n--- {} ---", tr("卸载 Live-Processor", "Uninstall"));
                let confirm = read_line(&format!("{} [y/N]: ", tr("确定要卸载 Live-Processor 吗?", "Are you sure you want to uninstall Live-Processor?")));
                if confirm.eq_ignore_ascii_case("y") {
                    let clean = read_line(&format!("{} [y/N]: ", tr("是否同时清除所有配置、驱动数据与注册表项?", "Delete all audio settings and driver data as well?")));
                    let delete_all_settings = clean.eq_ignore_ascii_case("y");

                    let options = UninstallOptions {
                        app_dir: None,
                        delete_all_settings,
                        silent: false,
                    };

                    if let Err(e) = run_uninstall(options) {
                        eprintln!("{}", format!("{}: {}", tr("卸载失败", "Uninstallation error"), e).red());
                    }
                }
            }
            "6" => {
                println!("\n--- {} ---", tr("提取驱动资源", "Extract Payload"));
                let dest = read_line(&format!("{} [./installer_payload]: ", tr("请输入提取目标目录", "Target directory for extracted files")));
                let target = if dest.is_empty() {
                    PathBuf::from("./installer_payload")
                } else {
                    PathBuf::from(dest)
                };

                let source = PayloadSource::new(None)?;
                match source.extract_all_to(&target) {
                    Ok(_) => println!("{}", format!("{}: {}", tr("资源已成功提取至", "Payload extracted to"), target.display()).green()),
                    Err(e) => eprintln!("{}", format!("{}: {}", tr("提取失败", "Failed to extract payload"), e).red()),
                }
            }
            "0" | "exit" | "q" => {
                println!("{}", tr("再见!", "Goodbye!"));
                break;
            }
            _ => {
                println!("{}", tr("无效的选项，请重新输入。", "Invalid option, please try again.").yellow());
            }
        }
    }

    Ok(())
}
