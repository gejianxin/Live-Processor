use colored::*;
use std::path::Path;

use crate::config::{get_system_drivers_dir, is_64bit_os};
use crate::driver::query_driver_service_status;
use crate::registry::get_installed_info;
use crate::task::verify_registry_fix_task;

pub fn print_status_report() {
    println!("{}", "=======================================================".cyan());
    println!("{}", "       Live-Processor / ASIO Link Pro Status Report    ".cyan().bold());
    println!("{}", "=======================================================".cyan());

    let is_64bit = is_64bit_os();
    println!(
        "System Architecture: {}",
        if is_64bit { "64-bit (x64)".green() } else { "32-bit (x86)".green() }
    );

    let info = get_installed_info();

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

                println!(" - ASIO Link Tool: {}", if main_tool.exists() { "OK".green() } else { "Missing".red() });
                println!(" - Patched DLL:    {}", if dll.exists() { "OK".green() } else { "Missing".red() });
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

    // 3. System Driver Status
    let dat_path = get_system_drivers_dir().join("asiovadpro.dat");
    println!(
        "System Driver Data:     {} ({})",
        if dat_path.exists() { "Present".green() } else { "Missing".red() },
        dat_path.display()
    );

    let svc_status = query_driver_service_status();
    match svc_status {
        Some(status) => println!("Kernel Driver Service:  {} ({})", "Registered".green(), status),
        None => println!("Kernel Driver Service:  {}", "Not running or not registered".yellow()),
    }

    // 4. Code Integrity Registry Fix
    println!(
        "CI Policy (UpgradedSystem): {}",
        if info.ci_policy_applied {
            "Active (1) [PROTECTED]".green().bold()
        } else {
            "Disabled or Missing [DRIVER MAY CUT OUT]".red().bold()
        }
    );

    // 5. Scheduled Self-Heal Task
    let task_ok = verify_registry_fix_task();
    println!(
        "Self-Heal Scheduled Task:   {}",
        if task_ok {
            "Active & Verified [AUTONOMOUS]".green().bold()
        } else {
            "Not registered or disabled".yellow()
        }
    );

    println!("{}", "-------------------------------------------------------".cyan());
    if info.ci_policy_applied && task_ok && dat_path.exists() {
        println!("{}", "All core components are healthy and protected.".green().bold());
    } else {
        println!("{}", "Notice: Run 'live-processor fix' to repair CI policy or self-heal task.".yellow().bold());
    }
    println!("{}", "=======================================================".cyan());
}
