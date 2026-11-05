use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;

use crate::com::register_com_libraries;
use crate::config::{
    get_default_install_dir, get_system_drivers_dir, DeviceLayout, DEFAULT_START_MENU_GROUP,
};
use crate::driver::{install_driver, DriverInstallResult};
use crate::elevation::require_elevation;
use crate::payload::PayloadSource;
use crate::registry::{
    configure_installation_registry, get_installed_info, register_uninstall_entry,
};
use crate::security::{grant_users_full_access_file, grant_users_full_access_registry};
use crate::shortcuts::create_start_menu_shortcuts;
use crate::task::{install_registry_fix_task, verify_registry_fix_task};

#[derive(Debug)]
pub struct InstallOptions {
    pub install_dir: Option<PathBuf>,
    pub layout: DeviceLayout,
    pub create_shortcuts: bool,
    pub start_menu_group: String,
    pub custom_payload_path: Option<PathBuf>,
    pub silent: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            install_dir: None,
            layout: DeviceLayout::Layout4x8,
            create_shortcuts: true,
            start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
            custom_payload_path: None,
            silent: false,
        }
    }
}

pub fn run_install(options: InstallOptions) -> Result<()> {
    require_elevation()?;

    let target_dir = options
        .install_dir
        .unwrap_or_else(get_default_install_dir);

    if !options.silent {
        println!("{}", "=======================================================".green());
        println!("{}", "            Live-Processor / ASIO Link Pro Setup       ".green().bold());
        println!("{}", "=======================================================".green());
        println!("Target Directory:   {}", target_dir.display().to_string().cyan());
        println!("Device Layout:      {}", options.layout.description().cyan());
        println!("Start Menu Group:   {}", options.start_menu_group.cyan());
        println!("{}", "-------------------------------------------------------".green());
    }

    // Step 0: Check existing installation
    let existing_info = get_installed_info();
    if let Some(ref existing_dir) = existing_info.install_dir {
        if !options.silent {
            println!(
                "{}",
                format!("Existing installation detected at: {}", existing_dir).yellow()
            );
            println!("{}", "Proceeding with clean in-place upgrade...".yellow());
        }
    }

    let spinner = if !options.silent {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
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

    // Step 1: Prepare payload & copy files
    set_msg("Unpacking payload and copying runtime files...");
    let temp_dir = std::env::temp_dir().join(format!("live_proc_inst_{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    let payload_source = PayloadSource::new(options.custom_payload_path.as_deref())?;
    payload_source
        .install_files(&target_dir, options.layout, &temp_dir)
        .context("Failed during file extraction and installation")?;

    // Cleanup temp extraction folder
    let _ = fs::remove_dir_all(&temp_dir);

    // Step 2: Grant ACL permissions to asiovadpro.dat
    set_msg("Applying security permissions to system driver file...");
    let dat_file = get_system_drivers_dir().join("asiovadpro.dat");
    let _ = grant_users_full_access_file(&dat_file);

    // Step 3: Configure registry
    set_msg("Configuring registry and device parameters...");
    let sm_group = if options.create_shortcuts {
        Some(options.start_menu_group.as_str())
    } else {
        None
    };
    configure_installation_registry(&target_dir, sm_group, options.layout)
        .context("Failed to configure installation registry")?;

    // Step 4: Grant ACL permissions to registry
    set_msg("Applying security permissions to registry keys...");
    let _ = grant_users_full_access_registry();

    // Step 5: Register COM DLLs
    set_msg("Registering COM libraries (asiolink.dll)...");
    register_com_libraries(&target_dir)
        .context("Failed to register ASIO Link Pro COM libraries")?;

    // Step 6: Install Kernel Driver via bundled installer
    set_msg("Installing audio driver package (DIFxAPI)...");
    let driver_result = install_driver(&target_dir)
        .context("Driver installation failed")?;

    // Step 7: Install and verify self-heal scheduled task
    set_msg("Setting up autonomous registry self-heal scheduled task...");
    let task_installed = install_registry_fix_task().is_ok();
    let task_verified = task_installed && verify_registry_fix_task();

    // Step 8: Register Add/Remove programs entry
    set_msg("Registering Windows uninstall entry...");
    let _ = register_uninstall_entry(&target_dir);

    // Step 9: Create Start Menu shortcuts
    if options.create_shortcuts {
        set_msg("Creating Start Menu shortcuts...");
        let _ = create_start_menu_shortcuts(&target_dir, &options.start_menu_group);
    }

    if let Some(pb) = spinner {
        pb.finish_and_clear();
    }

    if !options.silent {
        println!("{}", "Installation completed successfully!".green().bold());
        match driver_result {
            DriverInstallResult::Success => {
                println!("{}", " - Audio driver: Installed and active.".green());
            }
            DriverInstallResult::AlreadyInstalled => {
                println!("{}", " - Audio driver: Already active, continued.".green());
            }
            DriverInstallResult::RebootRequired => {
                println!("{}", " - Audio driver: [REBOOT REQUIRED] Please restart your PC to finish driver loading.".yellow().bold());
            }
        }

        if task_verified {
            println!("{}", " - Self-Heal Task: Verified and running.".green());
        } else {
            println!("{}", " - Self-Heal Task: Warning: could not verify scheduled task. registryfix.reg is available in app folder.".yellow());
        }

        println!("{}", "-------------------------------------------------------".green());
        println!("Launch the tool from Start Menu or run:");
        let tool_exe = if crate::config::is_64bit_os() {
            target_dir.join("x64").join("asiolinktool.exe")
        } else {
            target_dir.join("asiolinktool.exe")
        };
        println!("  {}", tool_exe.display().to_string().cyan());
    }

    Ok(())
}
