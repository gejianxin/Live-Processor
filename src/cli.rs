use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::config::{DeviceLayout, DEFAULT_START_MENU_GROUP};

#[derive(Parser, Debug)]
#[command(
    name = "live-processor",
    author = "gejianxin",
    version = "2.4.4-rev1",
    about = "Modern Rust reimagining of ASIO Link Pro installer, driver manager, and system supervisor"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run in interactive console wizard mode
    #[arg(short, long)]
    pub interactive: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install or upgrade Live-Processor / ASIO Link Pro
    Install(InstallArgs),

    /// Uninstall Live-Processor and clean system
    Uninstall(UninstallArgs),

    /// Display comprehensive diagnostic status of drivers, registry, and tasks
    Status,

    /// Apply the Code Integrity policy fix and register autonomous self-heal task
    Fix,

    /// Switch WDM audio device layout between 4x8 and 16x2
    SwitchLayout(SwitchLayoutArgs),

    /// Extract embedded installer payload to a local directory
    ExtractPayload(ExtractPayloadArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Custom target installation directory
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// WDM device layout: '4x8' (default) or '16x2' (16 stereo devices)
    #[arg(short, long, default_value = "4x8")]
    pub layout: DeviceLayout,

    /// Do not create Start Menu shortcuts
    #[arg(long)]
    pub no_shortcuts: bool,

    /// Custom Start Menu folder name
    #[arg(long, default_value = DEFAULT_START_MENU_GROUP)]
    pub start_menu_group: String,

    /// Custom external payload directory (overrides embedded payload)
    #[arg(long)]
    pub payload_dir: Option<PathBuf>,

    /// Silent installation without interactive prompts
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// Target directory to uninstall
    #[arg(short, long)]
    pub dir: Option<PathBuf>,

    /// Delete all user settings, driver data, and startup entries
    #[arg(short = 'c', long)]
    pub clean: bool,

    /// Silent uninstallation without confirmation prompts
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args, Debug)]
pub struct SwitchLayoutArgs {
    /// New WDM device layout: '4x8' or '16x2'
    #[arg(short, long)]
    pub layout: DeviceLayout,

    /// Silent execution
    #[arg(short, long)]
    pub silent: bool,
}

#[derive(Args, Debug)]
pub struct ExtractPayloadArgs {
    /// Output directory for extracted payload files
    #[arg(short, long, default_value = "./installer_payload")]
    pub out: PathBuf,
}
