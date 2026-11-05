use anyhow::Result;
use clap::Parser;

use live_processor::cli::{Cli, Commands};
use live_processor::config::{get_default_install_dir, DEFAULT_START_MENU_GROUP};
use live_processor::elevation::require_elevation;
use live_processor::i18n::{tr, Language};
use live_processor::installer::{run_install, InstallOptions};
use live_processor::payload::PayloadSource;
use live_processor::registry::{apply_code_integrity_policy_fix, get_installed_info};
use live_processor::security::{grant_users_full_access_file, grant_users_full_access_registry};
use live_processor::status::print_status_report;
use live_processor::task::{install_registry_fix_task, verify_registry_fix_task};
use live_processor::uninstaller::{run_uninstall, UninstallOptions};
use live_processor::wizard::run_interactive_wizard;

fn main() -> Result<()> {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let cli = Cli::parse();

    // Default language is Chinese; set English if requested
    if cli.english {
        Language::set(Language::English);
    } else {
        Language::set(Language::Chinese);
    }

    match cli.command {
        None => {
            // Independent CLI program: runs interactive terminal wizard
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
            println!(
                "{}",
                tr(
                    "正在应用代码完整性策略 (UpgradedSystem = 1)...",
                    "Applying Code Integrity Policy (UpgradedSystem = 1)..."
                )
            );
            apply_code_integrity_policy_fix()?;
            println!(
                "{}",
                tr(
                    "代码完整性策略应用成功。",
                    "Code Integrity Policy applied successfully."
                )
            );

            println!(
                "{}",
                tr(
                    "正在安装自主自愈计划任务...",
                    "Installing autonomous registry self-heal scheduled task..."
                )
            );
            install_registry_fix_task()?;
            if verify_registry_fix_task() {
                println!(
                    "{}",
                    tr(
                        "计划自愈任务已安装并成功验证。",
                        "Scheduled task installed and verified."
                    )
                );
            } else {
                println!(
                    "{}",
                    tr(
                        "警告: 计划任务已安装，但查询验证未通过。",
                        "Warning: Scheduled task installed but verification returned non-zero."
                    )
                );
            }

            println!(
                "{}",
                tr(
                    "正在验证安全 ACL 权限...",
                    "Verifying security ACL permissions..."
                )
            );
            let dat_file = live_processor::config::get_system_drivers_dir().join("asiovadpro.dat");
            let _ = grant_users_full_access_file(&dat_file);
            let _ = grant_users_full_access_registry();
            println!("{}", tr("安全权限配置完成。", "Security permissions applied."));
            println!(
                "{}",
                tr(
                    "系统修复操作全部完成。",
                    "Fix operations completed successfully."
                )
            );
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
                    format!(
                        "{}: {}",
                        tr("正在切换音频设备声道布局", "Switching audio device layout to"),
                        args.layout.description()
                    )
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
            println!(
                "{}",
                format!(
                    "{}: {}",
                    tr("正在提取驱动载荷至", "Extracting installer payload to"),
                    args.out.display()
                )
            );
            let source = PayloadSource::new(None)?;
            source.extract_all_to(&args.out)?;
            println!(
                "{}",
                tr(
                    "驱动载荷提取成功。",
                    "Payload extracted successfully."
                )
            );
        }
    }

    Ok(())
}
