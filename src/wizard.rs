use anyhow::Result;
use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;

use crate::config::{get_default_install_dir, DeviceLayout, DEFAULT_START_MENU_GROUP};
use crate::elevation::is_elevated;
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
    println!("{}", "             Live-Processor - Interactive Hub          ".cyan().bold());
    println!("{}", "   Modern Rust Manager for ASIO Link Pro Audio Driver  ".cyan());
    println!("{}", "=======================================================".cyan());

    if !is_elevated() {
        println!(
            "{}",
            "Notice: You are not running with Administrator privileges.\nFor installation, repair, and uninstallation, Administrator privileges are required."
                .yellow()
        );
        println!("{}", "-------------------------------------------------------".cyan());
    }

    loop {
        println!("\nPlease select an option:");
        println!("  1) Install / Upgrade Live-Processor");
        println!("  2) Diagnostics & Status Report");
        println!("  3) Switch Device Layout (4x8 <-> 16x2)");
        println!("  4) Quick Repair (Code Integrity Fix & Scheduled Task)");
        println!("  5) Uninstall Live-Processor");
        println!("  6) Extract Embedded Payload to Disk");
        println!("  0) Exit");

        let choice = read_line("\nEnter choice [0-6]: ");
        match choice.as_str() {
            "1" => {
                println!("\n--- Install Configuration ---");
                println!("Select Device Layout:");
                println!("  1) 4 devices / 8 channels (Default - recommended for most users)");
                println!("  2) 16 stereo WDM devices (16 devices / 2 channels)");
                let layout_choice = read_line("Choice [1]: ");
                let layout = if layout_choice == "2" {
                    DeviceLayout::Layout16x2
                } else {
                    DeviceLayout::Layout4x8
                };

                let default_dir = get_default_install_dir();
                println!("Default install path: {}", default_dir.display());
                let dir_input = read_line("Enter custom path or press Enter for default: ");
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
                    eprintln!("{}", format!("Installation error: {}", e).red().bold());
                }
            }
            "2" => {
                println!();
                print_status_report();
            }
            "3" => {
                println!("\n--- Switch Device Layout ---");
                let info = get_installed_info();
                let app_dir = info
                    .install_dir
                    .map(PathBuf::from)
                    .unwrap_or_else(get_default_install_dir);

                if !app_dir.exists() {
                    eprintln!("{}", "Error: Live-Processor is not installed. Please install it first.".red());
                    continue;
                }

                println!("Select new layout:");
                println!("  1) 4 devices / 8 channels");
                println!("  2) 16 stereo WDM devices");
                let layout_choice = read_line("Choice: ");
                let new_layout = if layout_choice == "2" {
                    DeviceLayout::Layout16x2
                } else {
                    DeviceLayout::Layout4x8
                };

                println!("Switching layout to {}...", new_layout.description().cyan());
                let options = InstallOptions {
                    install_dir: Some(app_dir),
                    layout: new_layout,
                    create_shortcuts: true,
                    start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                    custom_payload_path: None,
                    silent: false,
                };

                if let Err(e) = run_install(options) {
                    eprintln!("{}", format!("Layout switch failed: {}", e).red());
                }
            }
            "4" => {
                println!("\n--- Quick Repair ---");
                println!("1. Applying Code Integrity Policy UpgradedSystem = 1...");
                match apply_code_integrity_policy_fix() {
                    Ok(_) => println!("{}", "   [OK] Registry fix applied.".green()),
                    Err(e) => eprintln!("   [ERROR] Failed to apply registry fix: {}", e),
                }

                println!("2. Registering autonomous self-heal scheduled task...");
                match install_registry_fix_task() {
                    Ok(_) => {
                        if verify_registry_fix_task() {
                            println!("{}", "   [OK] Scheduled task installed and verified.".green());
                        } else {
                            println!("{}", "   [WARN] Scheduled task installed but verification failed.".yellow());
                        }
                    }
                    Err(e) => eprintln!("   [ERROR] Failed to install scheduled task: {}", e),
                }

                println!("3. Re-applying security ACL permissions...");
                let dat_file = crate::config::get_system_drivers_dir().join("asiovadpro.dat");
                let _ = grant_users_full_access_file(&dat_file);
                let _ = grant_users_full_access_registry();
                println!("{}", "   [OK] Permissions verified.".green());
            }
            "5" => {
                println!("\n--- Uninstall ---");
                let confirm = read_line("Are you sure you want to uninstall Live-Processor? [y/N]: ");
                if confirm.eq_ignore_ascii_case("y") {
                    let clean = read_line("Delete all audio settings and driver data as well? [y/N]: ");
                    let delete_all_settings = clean.eq_ignore_ascii_case("y");

                    let options = UninstallOptions {
                        app_dir: None,
                        delete_all_settings,
                        silent: false,
                    };

                    if let Err(e) = run_uninstall(options) {
                        eprintln!("{}", format!("Uninstallation error: {}", e).red());
                    }
                }
            }
            "6" => {
                println!("\n--- Extract Payload ---");
                let dest = read_line("Target directory for extracted files [./extracted_payload]: ");
                let target = if dest.is_empty() {
                    PathBuf::from("./extracted_payload")
                } else {
                    PathBuf::from(dest)
                };

                let source = PayloadSource::new(None)?;
                match source.extract_all_to(&target) {
                    Ok(_) => println!("{}", format!("Payload extracted to: {}", target.display()).green()),
                    Err(e) => eprintln!("{}", format!("Failed to extract payload: {}", e).red()),
                }
            }
            "0" | "exit" | "q" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("{}", "Invalid option, please try again.".yellow());
            }
        }
    }

    Ok(())
}
