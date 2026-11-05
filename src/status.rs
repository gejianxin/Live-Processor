use colored::*;
use std::path::Path;

use crate::config::{get_system_drivers_dir, is_64bit_os};
use crate::driver::query_driver_service_status;
use crate::registry::get_installed_info;
#[cfg(windows)]
use crate::task::verify_registry_fix_task;

pub fn print_status_report() {
    println!("{}", "=======================================================".cyan());
    println!("{}", "       Live-Processor / ASIO Link Pro Status Report    ".cyan().bold());
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
        "Host Platform:          {} - {}",
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
                println!("Installation Directory: {} {}", dir.green(), "[FOUND]".green().bold());
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

                println!(" - ASIO Link Tool:      {}", if main_tool.exists() { "OK".green() } else { "Missing".red() });
                println!(" - Patched DLL:         {}", if dll.exists() { "OK".green() } else { "Missing".red() });
            } else {
                println!("Installation Directory: {} {}", dir.yellow(), "[PATH DOES NOT EXIST]".red());
            }
        }
        None => {
            println!("Installation Directory: {}", "Not installed".yellow());
        }
    }

    // 2. WDM Device Configuration
    match (info.num_devices, info.num_channels) {
        (Some(dev), Some(ch)) => {
            let layout_desc = if dev == 16 && ch == 2 {
                "16 stereo WDM devices (16 devices / 2 channels)"
            } else if dev == 4 && ch == 8 {
                "4 WDM devices / 8 channels (Default)"
            } else {
                "Custom Layout"
            };
            println!("Device Configuration:   {} (Devices: {}, Channels: {})", layout_desc.green().bold(), dev, ch);
        }
        _ => {
            println!("Device Configuration:   {}", "Not configured in registry".yellow());
        }
    }

    // 3. Driver & Audio Subsystem Status
    let dat_path = get_system_drivers_dir().join("asiovadpro.dat");
    println!(
        "Driver Binary Data:     {} ({})",
        if dat_path.exists() { "Present".green() } else { "Missing / Inactive".yellow() },
        dat_path.display()
    );

    let svc_status = query_driver_service_status();
    match svc_status {
        Some(status) => println!("Audio Driver / Host:    {} ({})", "Active".green(), status),
        None => println!("Audio Driver / Host:    {}", "Inactive or not running".yellow()),
    }

    // 4. Code Integrity / System Protection
    #[cfg(windows)]
    {
        println!(
            "CI Policy (UpgradedSystem): {}",
            if info.ci_policy_applied {
                "Active (1) [PROTECTED]".green().bold()
            } else {
                "Disabled or Missing [DRIVER MAY CUT OUT]".red().bold()
            }
        );

        let task_ok = verify_registry_fix_task();
        println!(
            "Self-Heal Scheduled Task:   {}",
            if task_ok {
                "Active & Verified [AUTONOMOUS]".green().bold()
            } else {
                "Not registered or disabled".yellow()
            }
        );
    }

    #[cfg(not(windows))]
    {
        println!(
            "System Integrity:       {}",
            "Native Linux kernel (Unrestricted Code Integrity)".green()
        );
        let wine_installed = std::process::Command::new("wine").arg("--version").output().is_ok();
        println!(
            "Wine / Proton Support:  {}",
            if wine_installed {
                "Detected (Ready for Windows audio / VST / DAW routing)".green().bold()
            } else {
                "Not installed (Native mode)".yellow()
            }
        );
    }

    println!("{}", "-------------------------------------------------------".cyan());
    println!("{}", "Ready for cross-platform processing and deployment.".green().bold());
    println!("{}", "=======================================================".cyan());
}
