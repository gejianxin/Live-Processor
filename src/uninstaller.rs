use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

use crate::com::unregister_com_libraries;
use crate::config::{get_default_install_dir, get_system_drivers_dir, DEFAULT_START_MENU_GROUP};
use crate::driver::{uninstall_driver, DriverUninstallResult};
use crate::elevation::require_elevation;
use crate::registry::{
    delete_all_audio_settings, get_installed_info, remove_uninstall_entries,
};
use crate::shortcuts::remove_start_menu_shortcuts;
use crate::task::remove_registry_fix_task;

#[derive(Debug)]
pub struct UninstallOptions {
    pub app_dir: Option<std::path::PathBuf>,
    pub delete_all_settings: bool,
    pub silent: bool,
}

impl Default for UninstallOptions {
    fn default() -> Self {
        Self {
            app_dir: None,
            delete_all_settings: false,
            silent: false,
        }
    }
}

pub fn run_uninstall(options: UninstallOptions) -> Result<()> {
    require_elevation()?;

    let installed_info = get_installed_info();
    let app_dir = options
        .app_dir
        .or_else(|| installed_info.install_dir.map(std::path::PathBuf::from))
        .unwrap_or_else(get_default_install_dir);

    if !options.silent {
        println!("{}", "=======================================================".red());
        println!("{}", "         Live-Processor / ASIO Link Pro Uninstall      ".red().bold());
        println!("{}", "=======================================================".red());
        println!("Uninstalling from:  {}", app_dir.display().to_string().cyan());
        println!("Clean all settings: {}", if options.delete_all_settings { "YES".red() } else { "NO (keep settings)".yellow() });
        println!("{}", "-------------------------------------------------------".red());
    }

    let spinner = if !options.silent {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.red} {msg}")
                .unwrap(),
        );
        Some(pb)
    } else {
        None
    };

    let set_msg = |msg: &str| {
        if let Some(ref pb) = spinner {
            pb.set_message(msg.to_string());
            pb.tick();
        }
    };

    // 1. Remove scheduled self-heal task
    set_msg("Removing scheduled self-heal task...");
    let _ = remove_registry_fix_task();

    // 2. Unregister COM DLLs
    set_msg("Unregistering COM libraries...");
    let _ = unregister_com_libraries(&app_dir);

    // 3. Uninstall driver
    set_msg("Uninstalling kernel audio driver package...");
    let driver_result = uninstall_driver(&app_dir)?;

    // 4. Delete settings and system driver dat if clean requested
    if options.delete_all_settings {
        set_msg("Cleaning all audio driver settings and registry keys...");
        delete_all_audio_settings();

        let dat_file = get_system_drivers_dir().join("asiovadpro.dat");
        if dat_file.exists() {
            let _ = fs::remove_file(dat_file);
        }
    }

    // 5. Remove Start Menu shortcuts
    set_msg("Removing Start Menu shortcuts...");
    let group_name = installed_info
        .start_menu_folder
        .unwrap_or_else(|| DEFAULT_START_MENU_GROUP.to_string());
    remove_start_menu_shortcuts(&group_name);

    // 6. Remove registry uninstall keys
    set_msg("Removing Windows Add/Remove Programs entry...");
    remove_uninstall_entries();

    // 7. Remove installed application files
    set_msg("Removing installed files...");
    if app_dir.exists() {
        // Remove individual files
        let _ = fs::remove_file(app_dir.join("ASIO Link Technology.pdf"));
        let _ = fs::remove_file(app_dir.join("asiolinktool.exe"));
        let _ = fs::remove_file(app_dir.join("asiolink.dll"));
        let _ = fs::remove_file(app_dir.join("registryfix.reg"));
        let _ = fs::remove_file(app_dir.join("installer.exe"));
        let _ = fs::remove_file(app_dir.join("difxapi.dll"));
        let _ = fs::remove_file(app_dir.join("asiovadpro.cat"));
        let _ = fs::remove_file(app_dir.join("asiovadpro.inf"));
        let _ = fs::remove_file(app_dir.join("asiovadpro.sys"));

        let x64_dir = app_dir.join("x64");
        if x64_dir.exists() {
            let _ = fs::remove_dir_all(&x64_dir);
        }

        // Try removing app dir if empty or not containing active running exe
        let _ = fs::remove_dir(&app_dir);
    }

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if !options.silent {
        println!("{}", "Uninstallation completed successfully!".green().bold());
        match driver_result {
            DriverUninstallResult::Success => {
                println!("{}", " - Driver successfully removed.".green());
            }
            DriverUninstallResult::RebootRequired => {
                println!("{}", " - [REBOOT REQUIRED] Please restart your PC to complete driver removal.".yellow().bold());
            }
            DriverUninstallResult::ManualRemovalRequired => {
                println!("{}", " - Note: You may need to manually remove ASIOVADPRO from Windows Device Manager.".yellow());
            }
        }
    }

    Ok(())
}
