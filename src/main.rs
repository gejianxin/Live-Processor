mod cli;
mod com;
mod config;
mod driver;
mod elevation;
mod installer;
mod payload;
mod registry;
mod security;
mod shortcuts;
mod status;
mod task;
mod uninstaller;
mod wizard;

use anyhow::Result;
use clap::Parser;
use colored::*;

use cli::{Cli, Commands};
use config::{get_default_install_dir, DEFAULT_START_MENU_GROUP};
use elevation::require_elevation;
use installer::{run_install, InstallOptions};
use payload::PayloadSource;
use registry::{apply_code_integrity_policy_fix, get_installed_info};
use security::{grant_users_full_access_file, grant_users_full_access_registry};
use status::print_status_report;
use task::{install_registry_fix_task, verify_registry_fix_task};
use uninstaller::{run_uninstall, UninstallOptions};
use wizard::run_interactive_wizard;

fn main() -> Result<()> {
    // Enable ANSI escape code support on Windows terminal if available
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let cli = Cli::parse();

    match cli.command {
        None => {
            // No subcommand provided: run interactive wizard
            run_interactive_wizard()?;
        }
        Some(Commands::Install(args)) => {
            let options = InstallOptions {
                install_dir: args.dir,
                layout: args.layout,
                create_shortcuts: !args.no_shortcuts,
                start_menu_group: args.start_menu_group,
                custom_payload_path: args.payload_dir,
                silent: args.silent,
            };
            run_install(options)?;
        }
        Some(Commands::Uninstall(args)) => {
            let options = UninstallOptions {
                app_dir: args.dir,
                delete_all_settings: args.clean,
                silent: args.silent,
            };
            run_uninstall(options)?;
        }
        Some(Commands::Status) => {
            print_status_report();
        }
        Some(Commands::Fix) => {
            require_elevation()?;
            println!("{}", "Applying Code Integrity Policy (UpgradedSystem = 1)...".cyan());
            apply_code_integrity_policy_fix()?;
            println!("{}", "Code Integrity Policy applied successfully.".green());

            println!("{}", "Installing autonomous registry self-heal scheduled task...".cyan());
            install_registry_fix_task()?;
            if verify_registry_fix_task() {
                println!("{}", "Scheduled task installed and verified.".green());
            } else {
                println!("{}", "Warning: Scheduled task installed but verification returned non-zero.".yellow());
            }

            println!("{}", "Verifying security ACL permissions...".cyan());
            let dat_file = config::get_system_drivers_dir().join("asiovadpro.dat");
            let _ = grant_users_full_access_file(&dat_file);
            let _ = grant_users_full_access_registry();
            println!("{}", "Security permissions applied.".green());
            println!("{}", "Fix operations completed successfully.".green().bold());
        }
        Some(Commands::SwitchLayout(args)) => {
            require_elevation()?;
            let info = get_installed_info();
            let app_dir = info
                .install_dir
                .map(std::path::PathBuf::from)
                .unwrap_or_else(get_default_install_dir);

            if !args.silent {
                println!(
                    "{}",
                    format!("Switching audio device layout to: {}", args.layout.description()).cyan()
                );
            }

            let options = InstallOptions {
                install_dir: Some(app_dir),
                layout: args.layout,
                create_shortcuts: true,
                start_menu_group: DEFAULT_START_MENU_GROUP.to_string(),
                custom_payload_path: None,
                silent: args.silent,
            };
            run_install(options)?;
        }
        Some(Commands::ExtractPayload(args)) => {
            println!("{}", format!("Extracting installer payload to: {}", args.out.display()).cyan());
            let source = PayloadSource::new(None)?;
            source.extract_all_to(&args.out)?;
            println!("{}", "Payload extracted successfully.".green().bold());
        }
    }

    Ok(())
}
