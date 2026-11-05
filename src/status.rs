use colored::*;
use std::path::Path;

use crate::config::{get_system_drivers_dir, is_64bit_os};
use crate::driver::query_driver_service_status;
use crate::i18n::tr;
use crate::registry::get_installed_info;
#[cfg(windows)]
use crate::task::verify_registry_fix_task;

pub fn print_status_report() {
    println!("{}", "=======================================================".cyan());
    println!("{}", tr("       Live-Processor / ASIO Link Pro 诊断与状态报告   ", "       Live-Processor / ASIO Link Pro Status Report    ").cyan().bold());
    println!("{}", "=======================================================".cyan());

    #[cfg(windows)]
    let os_desc = "Windows";
    #[cfg(target_os = "linux")]
    let os_desc = "Linux";
    #[cfg(not(any(windows, target_os = "linux")))]
    let os_desc = "Unix/Other";

    let arch_desc = if cfg!(target_arch = "x86_64") {
        "x86_64 (64-bit)"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64 / arm64 (64-bit)"
    } else if cfg!(target_arch = "x86") {
        "x86 (32-bit)"
    } else {
        "Unknown Architecture"
    };

    println!(
        "{}:          {} - {}",
        tr("操作系统平台", "Host Platform"),
        os_desc.green().bold(),
        arch_desc.green()
    );

    let info = get_installed_info();
    let is_64bit = is_64bit_os();

    // 1. Install Directory
    match &info.install_dir {
        Some(dir) => {
            let path = Path::new(dir);
            if path.exists() {
                println!("{}: {} {}", tr("驱动安装目录", "Installation Directory"), dir.green(), tr("[已就绪]", "[FOUND]").green().bold());
                let main_tool = if is_64bit {
                    path.join("x64").join("asiolinktool.exe")
                } else {
                    path.join("asiolinktool.exe")
                };
                let dll = if is_64bit {
                    path.join("x64").join("asiolink.dll")
                } else {
                    path.join("asiolink.dll")
                };

                println!(" - ASIO Link Tool:      {}", if main_tool.exists() { "OK".green() } else { tr("缺失", "Missing").red() });
                println!(" - Patched DLL:         {}", if dll.exists() { "OK".green() } else { tr("缺失", "Missing").red() });
            } else {
                println!("{}: {} {}", tr("驱动安装目录", "Installation Directory"), dir.yellow(), tr("[路径不存在]", "[PATH DOES NOT EXIST]").red());
            }
        }
        None => {
            println!("{}: {}", tr("驱动安装目录", "Installation Directory"), tr("未安装", "Not installed").yellow());
        }
    }

    // 2. WDM Device Configuration
    match (info.num_devices, info.num_channels) {
        (Some(dev), Some(ch)) => {
            let layout_desc = if dev == 16 && ch == 2 {
                tr("16 独立立体声 WDM 设备 (16 设备 / 2 声道)", "16 stereo WDM devices (16 devices / 2 channels)")
            } else if dev == 4 && ch == 8 {
                tr("4 WDM 设备 / 8 声道 (默认推荐模式)", "4 WDM devices / 8 channels (Default)")
            } else {
                tr("自定义声道布局", "Custom Layout")
            };
            println!("{}:   {} ({}: {}, {}: {})", tr("WDM 设备布局", "Device Configuration"), layout_desc.green().bold(), tr("设备数", "Devices"), dev, tr("声道数", "Channels"), ch);
        }
        _ => {
            println!("{}:   {}", tr("WDM 设备布局", "Device Configuration"), tr("注册表中未配置", "Not configured in registry").yellow());
        }
    }

    // 3. Driver & Audio Subsystem Status
    let dat_path = get_system_drivers_dir().join("asiovadpro.dat");
    println!(
        "{}:     {} ({})",
        tr("系统驱动数据文件", "Driver Binary Data"),
        if dat_path.exists() { tr("存在", "Present").green() } else { tr("缺失 / 未激活", "Missing / Inactive").yellow() },
        dat_path.display()
    );

    let svc_status = query_driver_service_status();
    match svc_status {
        Some(status) => println!("{}:    {} ({})", tr("音频驱动宿主服务", "Audio Driver / Host"), tr("运行中", "Active").green(), status),
        None => println!("{}:    {}", tr("音频驱动宿主服务", "Audio Driver / Host"), tr("未启动或未注册", "Inactive or not running").yellow()),
    }

    // 4. Code Integrity / System Protection
    #[cfg(windows)]
    {
        println!(
            "{}: {}",
            tr("CI 代码完整性策略 (UpgradedSystem)", "CI Policy (UpgradedSystem)"),
            if info.ci_policy_applied {
                tr("已激活 (1) [受保护 - 防止声音中断]", "Active (1) [PROTECTED]").green().bold()
            } else {
                tr("未激活或缺失 [可能产生切音故障]", "Disabled or Missing [DRIVER MAY CUT OUT]").red().bold()
            }
        );

        let task_ok = verify_registry_fix_task();
        println!(
            "{}:   {}",
            tr("自主自愈计划任务", "Self-Heal Scheduled Task"),
            if task_ok {
                tr("运行正常已验证 [守护就绪]", "Active & Verified [AUTONOMOUS]").green().bold()
            } else {
                tr("未注册或已禁用", "Not registered or disabled").yellow()
            }
        );
    }

    #[cfg(not(windows))]
    {
        println!(
            "{}:       {}",
            tr("系统完整性状态", "System Integrity"),
            tr("Linux 原生内核 (无代码签名策略限制)", "Native Linux kernel (Unrestricted Code Integrity)").green()
        );
        let wine_installed = std::process::Command::new("wine").arg("--version").output().is_ok();
        println!(
            "{}:  {}",
            tr("Wine / Proton 兼容支持", "Wine / Proton Support"),
            if wine_installed {
                tr("已检测到 (已就绪，可加载 Windows DAW 与 VST)", "Detected (Ready for Windows audio / VST / DAW routing)").green().bold()
            } else {
                tr("未安装 (仅原生模式)", "Not installed (Native mode)").yellow()
            }
        );
    }

    println!("{}", "-------------------------------------------------------".cyan());
    println!("{}", tr("系统就绪，已支持跨平台音频处理与驱动守护。", "Ready for cross-platform processing and deployment.").green().bold());
    println!("{}", "=======================================================".cyan());
}
